//! `.pasta`↔生成 `.lua` 行対応の **本番マップ表現**（consumer 側・R2/R4/R5/R8）。
//!
//! このモジュールは default features で常時コンパイルされる（feature gate なし・7.3）。
//! マルチチャンク本番マップ（[`ChunkSourceMap`]/[`SourceMap`]/[`MapBuilderSink`]）と
//! 双方向解決・任意ディスクサイドカー（[`write_sidecar`]/[`read_sidecar`]）を提供する。
//!
//! # 構築フロー（design "MapBuilderSink → ChunkSourceMap → SourceMap"）
//!
//! producer 側のシーム（[`crate::code_gen::source_map`] の [`SourceMapSink`]）は
//! トランスパイル中に `record_line(out_line, pasta_line)`（trait 既定の
//! `record(out_line, span)` 経由なら `span.start_line`）を通知する。consumer 側は:
//!
//! 1. [`MapBuilderSink`] がその通知を **pre-normalize** の `lua_line → PastaPos` として
//!    蓄積する。`.pasta` 行は `span.start_line` を **直接**採用する（byte 走査廃止・
//!    research.md D-3）。
//! 2. トランスパイル完了後、[`MapBuilderSink::finish`] が `normalize_output_with_shift`
//!    の返す [`LineShift`] を適用し、各 pre-normalize 行を **最終 `.lua` 行**へ rebase
//!    した 1 チャンクの [`ChunkSourceMap`] を確定する（normalize 行ズレ補正）。
//! 3. [`SourceMap`] が複数チャンクの [`ChunkSourceMap`] を **正規化チャンク名**で集約し、
//!    [`SourceMap::resolve_lua_to_pasta`]（`.lua`→`.pasta`・R5）と
//!    [`SourceMap::resolve_pasta_to_lua`]（`.pasta`→`.lua` 逆引き・R4）を提供する。
//!
//! # normalize 行ズレ補正
//!
//! producer の `out_line` は `normalize_output` **適用前**のバッファ行を数えるため、
//! 最終 `.lua` の行番号と一般にはズレ得る。[`MapBuilderSink::finish`] が
//! [`LineShift`] を介して pre-normalize 行 → 最終 `.lua` 行へ rebase し、normalize が
//! 削除した行に紐づく記録は最終写像から除外する（requirements 2.1）。

use std::collections::{BTreeMap, HashMap};
// `Path`/`PathBuf` はサイドカー I/O を child `sidecar` へ分離後（task 7.5・C5）、本ハブ
// 本番では未使用。外出しした `source_map_sidecar_tests` クラスタが `use super::*;` で
// 参照するため、test-only で再導入する（本番 import を増やさない・public 不変）。
#[cfg(test)]
use std::path::{Path, PathBuf};

pub use crate::code_gen::source_map::{PastaPos, SourceMapSink};
use crate::normalize::LineShift;

/// チャンク（1 つの生成 `.lua` ファイル）を識別するキー。
///
/// ランタイムのラインフックが報告する `lua_Debug.source`（`@<絶対 .lua パス>` 想定）
/// に **正規化キー**で一致させる（design "Source Identity" 437-440・
/// [`canonicalize_chunk_name`]）。マルチチャンク集約 [`SourceMap`] のキー型として
/// 用いる。
pub type ChunkName = String;

/// 1 チャンクの **双方向**行写像（design "ChunkSourceMap" 450-456）。
///
/// 「最終 `.lua` 行 → `.pasta` 位置」の前方写像 [`forward`](Self::forward) を保持し、
/// 次の 2 方向の引きを提供する:
///
/// 1. [`pasta_for_lua`](Self::pasta_for_lua): 最終 `.lua` 行 → `.pasta` 位置。対応の
///    無い行（生成器が挿入した補助/挿入行）には `None`（requirements 1.2/2.2/2.3）。
/// 2. [`lua_lines_for_pasta`](Self::lua_lines_for_pasta): `.pasta` 行 → 対応する最終
///    `.lua` 行群。1 `.pasta` 行が複数 `.lua` 行へ展開され得る（requirements 8.2）ため
///    `Vec` を返し、`.lua` 行の **昇順かつ決定的順序**で返す（requirements 8.3）。
///
/// # 不変条件（requirements 8.1）
///
/// 前方写像は最終 `.lua` 行をキーとする `BTreeMap` であり、1 つの最終 `.lua` 行は
/// **高々 1 つ**の `.pasta` 位置に対応する（複数 `.pasta` 行が同一 `.lua` 行へ集約
/// される場合は last-write-wins・キー一意性が担保）。`BTreeMap` は決定的反復順
/// （キー昇順）を持つため、逆引きの提示順序の安定性（8.3）も自然に満たす。
///
/// トランスパイル完了後は不変（design 434）。生成（producer → `ChunkSourceMap` の
/// ビルドパイプライン・`finish(shift)` rebase）は同モジュールの
/// [`MapBuilderSink::finish`] が担い、[`from_forward`](Self::from_forward) は既知
/// forward からの直接構築（`finish` 内部・テスト）に供する。
#[derive(Debug, Clone, Default)]
pub struct ChunkSourceMap {
    /// 最終 `.lua` 行（1 始まり）→ 対応する `.pasta` 位置。
    ///
    /// `BTreeMap` を用いるのは決定的反復順（キー昇順）のため。これにより
    /// [`lua_lines_for_pasta`](Self::lua_lines_for_pasta) の返り値が `.lua` 行の昇順
    /// かつ決定的になる（requirements 8.3）。
    forward: BTreeMap<u32, PastaPos>,
}

impl ChunkSourceMap {
    /// 空の写像を構築する。
    pub fn new() -> Self {
        Self::default()
    }

    /// 既知の前方写像（最終 `.lua` 行 → `.pasta` 位置）から構築する。
    ///
    /// `BTreeMap` のキー一意性により 1 `.lua` 行 → 高々 1 `.pasta` 位置の不変条件
    /// （requirements 8.1）が担保される。producer からの本番ビルドパイプラインは
    /// [`MapBuilderSink::finish`] が担い、本コンストラクタは `finish` から `forward`
    /// を直接渡す内部用途とテストに供する。
    pub fn from_forward(forward: BTreeMap<u32, PastaPos>) -> Self {
        Self { forward }
    }

    /// 記録済みの対応件数（最終 `.lua` 行の数）。
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// 対応が 1 件も無いか。
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }

    /// 最終 `.lua` 行 → 由来 `.pasta` 位置を一意に解決する（requirements 2.2）。
    ///
    /// 対応を持たない行（生成器が挿入した補助/挿入行・記録されていない行）には
    /// `None` を返し、「対応なし」を明示する（requirements 1.2/2.3）。`BTreeMap`
    /// のキー一意性により返り値は高々 1 件で確定的（requirements 8.1）。
    pub fn pasta_for_lua(&self, lua_line: u32) -> Option<&PastaPos> {
        self.forward.get(&lua_line)
    }

    /// `.pasta` 行 → 対応する最終 `.lua` 行群（逆引き）。
    ///
    /// 1 つの `.pasta` 行が複数の最終 `.lua` 行へ展開され得る（requirements 8.2）
    /// ため `Vec` を返す。前方写像（`BTreeMap`）を **キー昇順**で反復し、`PastaPos.line`
    /// が `pasta_line` に一致する `.lua` 行を収集するため、返り値は `.lua` 行の昇順
    /// かつ決定的順序になる（requirements 8.3）。対応の無い `.pasta` 行には空 `Vec`。
    pub fn lua_lines_for_pasta(&self, pasta_line: u32) -> Vec<u32> {
        self.forward
            .iter()
            .filter(|(_, pos)| pos.line == pasta_line)
            .map(|(lua_line, _)| *lua_line)
            .collect()
    }
}

/// producer の `record` コールバックから 1 チャンクのマップを構築する
/// [`SourceMapSink`] 実装（design "MapBuilderSink" 389-423）。
///
/// # 役割
///
/// producer（[`crate::code_gen::LuaCodeGenerator`] の `generate_*`）が、`normalize`
/// **適用前**の中間バッファ行 `out_line`（1 始まり）を `.pasta` span と共に `record`
/// する（前提条件・design 421）。`MapBuilderSink` はこれを **pre-normalize** の
/// `lua_line → PastaPos` として [`pre_norm`](Self::pre_norm) に蓄積する。
///
/// トランスパイル完了後、`normalize_output_with_shift` が返す [`LineShift`] を
/// [`finish`](Self::finish) に渡すと、各 pre-normalize 行を **最終 `.lua` 行**へ
/// rebase した [`ChunkSourceMap`] を確定する（Flow 1・design 204-213）。
///
/// # `.pasta` 行の採用（research.md D-3）
///
/// `.pasta` 行は `span.start_line`（または code_block の行オフセット）を **直接**
/// 採用する。`span.start_byte` から `\n` を数えるバイト走査方式は採らない。
/// trait 既定の [`record`](SourceMapSink::record) が `record_line(lua_line,
/// span.start_line)` へ委譲するため、本実装は core の
/// [`record_line`](SourceMapSink::record_line) のみを提供する。
///
/// # 不変条件（requirements 8.1）
///
/// - **同一 pre-normalize 行は last-write-wins**: `pre_norm` は `BTreeMap` であり、
///   同一 `lua_line` キーへの再 `record_line` は後勝ちで上書きする（決定論的・
///   design 423）。
/// - **rebase 後の最終行の衝突解決**: 仮に複数の pre-normalize 行が同一の最終 `.lua`
///   行へ rebase された場合、[`finish`](Self::finish) は `pre_norm` を **キー昇順**
///   （pre-line 昇順）で反復して forward `BTreeMap` へ挿入するため、**最大の
///   pre-line が後勝ち**となる（決定論的・8.1）。ただし [`LineShift::map`] は生存行
///   上で単調増加かつ単射であるため、生存する 2 つの異なる pre-line が同一最終行へ
///   衝突することは構造上起こり得ない（衝突解決規則は防御的かつ決定論的な後勝ちとして
///   定義する）。
pub struct MapBuilderSink {
    /// 元 `.pasta` ファイルパス（[`PastaPos::file`] に載せる）。
    pasta_file: String,
    /// このチャンク（生成 `.lua` ファイル）を識別するキー（design "Source Identity"）。
    ///
    /// マルチチャンク集約 [`SourceMap`] への登録
    /// （[`SourceMap::insert_chunk`]・loader の `build_source_map`）でこのキーを
    /// 用いる。`finish` の戻り値 [`ChunkSourceMap`] 自体はチャンク名を保持しないため、
    /// 本フィールドは集約側が参照する識別子として保持する。
    chunk_name: ChunkName,
    /// pre-normalize の `lua_line`（1 始まり・`out_line`）→ `.pasta` 位置。
    ///
    /// `BTreeMap` を用いるのは (1) 同一 pre-line の last-write-wins（キー一意性・8.1）
    /// と (2) `finish` の rebase 反復が pre-line 昇順で決定論的になるため。
    pre_norm: BTreeMap<u32, PastaPos>,
}

impl MapBuilderSink {
    /// `.pasta` ファイルパスとチャンク名から空のシンクを構築する。
    pub fn new(pasta_file: String, chunk_name: ChunkName) -> Self {
        Self {
            pasta_file,
            chunk_name,
            pre_norm: BTreeMap::new(),
        }
    }

    /// このシンクが構築するチャンクの識別キーを借用する。
    pub fn chunk_name(&self) -> &ChunkName {
        &self.chunk_name
    }

    /// normalize の [`LineShift`] を適用し、最終 [`ChunkSourceMap`] を確定する
    /// （design 409-410, 422・requirements 2.1）。
    ///
    /// `pre_norm` の各 `(pre_line, pos)` を [`LineShift::map`] で **最終 `.lua` 行**へ
    /// rebase する:
    ///
    /// - `Some(final_line)`: forward マップへ `final_line → pos` を挿入する。
    /// - `None`（normalize が削除した行）: 由来 `.pasta` の無い空行であり、最終 `.lua`
    ///   には存在しないため **除外**する（design 422・requirements 2.1）。
    ///
    /// `pre_norm` を **キー昇順**で反復するため、仮に複数 pre-line が同一最終行へ
    /// rebase されても **最大 pre-line が後勝ち**で決定論的（8.1）。生存行上の
    /// `LineShift::map` は単射なので通常この衝突は発生しない。
    pub fn finish(self, shift: &LineShift) -> ChunkSourceMap {
        let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
        // BTreeMap::into_iter() yields entries in ascending key (pre-line) order,
        // so a collided final line is resolved deterministically to the largest
        // pre-line (last-write-wins・8.1).
        for (pre_line, pos) in self.pre_norm {
            if let Some(final_line) = shift.map(pre_line) {
                forward.insert(final_line, pos);
            }
            // 削除行（None）は最終 .lua に存在しないため除外（requirements 2.1）。
        }
        ChunkSourceMap::from_forward(forward)
    }
}

impl SourceMapSink for MapBuilderSink {
    /// pre-normalize の `lua_line` → `.pasta` 位置を `pre_norm` へ挿入する（core 操作・
    /// design 414-416）。同一 `lua_line` への再記録は last-write-wins で上書き（8.1）。
    ///
    /// `.pasta` 行は呼び出し側（trait 既定の `record` 経由なら `span.start_line`）が
    /// 与えた `pasta_line` を **直接**採用する（byte 走査廃止・research.md D-3）。
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.pre_norm.insert(
            lua_line,
            PastaPos {
                file: self.pasta_file.clone(),
                line: pasta_line,
            },
        );
    }
    // record(lua_line, span) は trait 既定（record_line(lua_line, span.start_line)）を使用。
}

/// チャンク名キーの **正規化**（design "Source Identity（議題2 確定）" 437-440 /
/// requirements 4.2, 5.1）。
///
/// ラインフックが報告する `lua_Debug.source`（`@<絶対 .lua パス>` 想定）と、ローダ
/// が [`crate::loader::CacheManager::source_to_cache_path`] から算出するキャッシュ
/// パス由来キーとを、**照合可能な単一の正規形**へ落とす。本仕様の全 `resolve_*` /
/// 格納はこの正規化キーで行う（design 439「残差吸収の保険」）。
///
/// 正規化規則（design 439 と実機実測 = task 1.1 Validation Hook に基づく）:
/// 1. **`@` 接頭辞の除去**: フック source は `@` 付き・ローダ由来キーは無し。
/// 2. **パス区切りの統一**: `\\` を `/` へ。**Windows 実機では本番 `require` 経路の
///    チャンク名が *混在区切り*（`package.path` 前置部は `/`・モジュール名展開部は
///    `\\`）になることを実測で確認**したため、この統一は必須（片側のみでは不一致）。
/// 3. **Windows の大小文字無視**: Windows は大小文字非依存ファイルシステムのため
///    小文字化する（`#[cfg(windows)]`）。非 Windows は大小区別を保持する。
///
/// 絶対化（相対→絶対）は呼び出し側がローダの絶対キャッシュパス／フックの絶対 source
/// を渡す前提で満たされる（双方とも構築時点で絶対パス）。本関数は文字列正規化に専念し、
/// FS への問い合わせ（`canonicalize`）は行わない（フック source 側のパスはロード後に
/// 必ずしも実在判定可能とは限らないため・決定論性のため）。
pub fn canonicalize_chunk_name(raw: &str) -> String {
    // (1) `@` 接頭辞を除去。
    let without_at = raw.strip_prefix('@').unwrap_or(raw);
    // (2) パス区切りを `/` へ統一（Windows 混在区切り対策・実測）。
    let unified = without_at.replace('\\', "/");
    // (3) Windows は大小文字無視。
    #[cfg(windows)]
    {
        unified.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        unified
    }
}

/// `.pasta` ファイルパスの照合用 **正規化キー**。
///
/// `.pasta` パスには `@` 接頭辞は付かないが、区切り統一（`\`→`/`）と Windows 大小
/// 文字無視は chunk 名と **同一規則**で行う必要がある（双方の格納側＝producer の
/// [`PastaPos::file`] と、query 側＝VSCode `source.path` を同一正規形へ落として一致
/// させるため・design Validation Hook 475「`.pasta` 側は VSCode source.path と
/// PastaPos.file の正規化一致」）。
///
/// chunk 名キーと `.pasta` ファイルキーを **同一の canonicalizer** で正規化するため、
/// 本関数は [`canonicalize_chunk_name`] を再利用する。`canonicalize_chunk_name` は
/// `@` 接頭辞が無い入力では strip を素通りする（`strip_prefix('@')` が `None` →
/// 元文字列）ため、`.pasta` パスへの適用でも区切り統一・大小規則のみが効く。STORE
/// と QUERY の双方が本関数（＝同一規則）を通ることで突合可能性が担保される
/// （design "Source Identity" 437-439）。
fn canonicalize_pasta_file(raw: &str) -> String {
    // `@` の無い `.pasta` パスでも、区切り統一・Windows 大小無視は chunk 名と同一規則。
    canonicalize_chunk_name(raw)
}

/// マルチチャンク集約ソースマップと **双方向解決**（design "SourceMap State
/// Management" 458-468・requirements 2.2/3.3/4.1/4.3/5.1）。
///
/// 複数チャンク（各生成 `.lua` ファイル）の [`ChunkSourceMap`] を **正規化チャンク名**
/// で集約し、`.lua`→`.pasta` の前方解決と `.pasta`→`.lua` の逆引き解決を提供する。
/// トランスパイル完了後は不変で、`Arc<SourceMap>` として consumer（resolver/BP 翻訳/
/// stepper）へ読み取り専用共有される（design 434・`Arc` 化は loader の
/// `build_source_map` が行い、本モジュールは型・構築・解決を提供する）。
///
/// # キー正規化（design "Source Identity" 437-440）
///
/// 全 `resolve_*` と格納はこの **正規化キー**で行う。
/// - チャンク名キー: [`canonicalize_chunk_name`]（`@` 除去・`\`→`/`・Windows 大小無視）。
/// - `.pasta` ファイルキー: [`canonicalize_pasta_file`]（chunk 名と同一規則を再利用）。
///
/// STORE（[`insert_chunk`](Self::insert_chunk)）と QUERY（`resolve_*`）の双方が同一
/// canonicalizer を通るため、フック source／ローダ由来キー（chunk 名）・VSCode
/// `source.path`／[`PastaPos::file`]（`.pasta` パス）が突合できる。
///
/// # 逆引き索引の昇順・決定性（design 435・requirements 8.3）
///
/// `reverse` は `.pasta` ファイル（正規化）→ (`.pasta` 行 → `[(ChunkName, lua_line)]`)。
/// [`insert_chunk`](Self::insert_chunk) は各チャンクの前方写像を **`.lua` 行昇順**
/// （`BTreeMap` 反復順）で走査して逆引きへ追記し、`(ChunkName, lua_line)` を
/// チャンク名昇順 → `.lua` 行昇順で安定ソートして保持する。これにより
/// [`resolve_pasta_to_lua`](Self::resolve_pasta_to_lua) の提示順序は決定的になる
/// （複数チャンク・1`.pasta`→複数`.lua` のいずれでも・8.3/4.1）。
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// 正規化チャンク名 → 1 チャンクの前方写像。
    ///
    /// `HashMap` を用いるのは O(1) のチャンク引きのため（design 459）。キーは
    /// [`canonicalize_chunk_name`] による正規形（STORE/QUERY 共通）。
    chunks: HashMap<ChunkName, ChunkSourceMap>,
    /// 逆引き索引: 正規化 `.pasta` ファイル → (`.pasta` 行 → `[(ChunkName, lua_line)]`)。
    ///
    /// 外側は `HashMap`（ファイル引き O(1)）、内側は `BTreeMap`（`.pasta` 行昇順反復＋
    /// [`nearest_pasta_line_with_mapping`](Self::nearest_pasta_line_with_mapping) の
    /// `range` クエリのため）。値 `Vec` はチャンク名昇順 →`.lua` 行昇順で安定（8.3）。
    reverse: HashMap<String, BTreeMap<u32, Vec<(ChunkName, u32)>>>,
}

impl SourceMap {
    /// 空の集約マップを構築する。
    pub fn new() -> Self {
        Self::default()
    }

    /// 1 チャンクを集約へ登録する（loader の `build_source_map` が per-`.pasta` の
    /// `finish` 結果を投入する builder API）。
    ///
    /// `chunk_name`（生フック源／ローダ由来パス）を [`canonicalize_chunk_name`] で
    /// 正規化して [`chunks`](Self::chunks) のキーとし、`pasta_file` を
    /// [`canonicalize_pasta_file`] で正規化して [`reverse`](Self::reverse) のファイル
    /// キーとする。さらにチャンクの前方写像を **`.lua` 行昇順**で走査して逆引きへ追記
    /// する。
    ///
    /// 逆引き各 `.pasta` 行の `Vec` は、追記後に **チャンク名昇順 → `.lua` 行昇順**で
    /// 安定ソートし、提示順序を決定的にする（design 435・requirements 8.3）。
    ///
    /// # 再投入は置換セマンティクス（防御的ハードニング）
    ///
    /// 同一チャンク名（正規化後）を再投入した場合、前方写像（`HashMap` 上書き）と
    /// 整合するよう **逆引き索引からも旧チャンクのエントリを除去**してから登録する。
    /// 本番経路（loader）は 1 チャンク 1 回投入のため正常系では no-op だが、公開 API
    /// として再投入時に stale な `(chunk, lua_line)` を残さない。
    pub fn insert_chunk(&mut self, chunk_name: ChunkName, pasta_file: String, map: ChunkSourceMap) {
        let chunk_key = canonicalize_chunk_name(&chunk_name);
        let file_key = canonicalize_pasta_file(&pasta_file);

        // 再投入（同一正規化チャンク名）なら、旧チャンク由来の逆引きエントリを全
        // ファイルキーから除去する（forward の HashMap 上書きと整合・stale 残留防止）。
        // 旧投入時の `.pasta` ファイルが今回と異なる可能性があるため全キーを掃く。
        if self.chunks.contains_key(&chunk_key) {
            self.reverse.retain(|_, per_file| {
                per_file.retain(|_, entries| {
                    entries.retain(|(ck, _)| *ck != chunk_key);
                    !entries.is_empty()
                });
                !per_file.is_empty()
            });
        }

        // 逆引き索引を、このチャンクの前方写像（`.lua` 行昇順の `BTreeMap`）から構築。
        let per_file = self.reverse.entry(file_key).or_default();
        // `forward` の反復は `.lua` 行昇順（BTreeMap）。各 `.pasta` 行へ
        // (chunk_key, lua_line) を追記する。
        for (lua_line, pasta_line) in map_forward_iter(&map) {
            per_file
                .entry(pasta_line)
                .or_default()
                .push((chunk_key.clone(), lua_line));
        }
        // 追記後に各 `.pasta` 行の Vec を決定的順序（チャンク名昇順 → `.lua` 行昇順）へ。
        for entries in per_file.values_mut() {
            entries.sort();
        }

        self.chunks.insert(chunk_key, map);
    }

    /// 最終 `.lua` 行 → 由来 `.pasta` 位置を解決する（requirements 5.1, 3.3）。
    ///
    /// `chunk` 引数を [`canonicalize_chunk_name`] で正規化してチャンクを引き、
    /// [`ChunkSourceMap::pasta_for_lua`] へ委譲する。チャンクが見つからない
    /// （整合性エラー・design 610/617）か `.lua` 行が未対応なら `None` を返し、誤った
    /// `.pasta` 対応づけを行わない（`.lua` フォールバック・requirements 2.3）。
    pub fn resolve_lua_to_pasta(&self, chunk: &str, lua_line: u32) -> Option<&PastaPos> {
        let chunk_key = canonicalize_chunk_name(chunk);
        self.chunks.get(&chunk_key)?.pasta_for_lua(lua_line)
    }

    /// `.pasta` 行 → 対応する全 `(ChunkName, lua_line)` を返す（requirements 4.1, 3.3）。
    ///
    /// `pasta_file` を [`canonicalize_pasta_file`] で正規化して逆引き索引を引く。1 つの
    /// `.pasta` 行が複数 `.lua` 行・複数チャンクへ展開され得る（4.1）。返り値は
    /// **チャンク名昇順 → `.lua` 行昇順**で決定的（requirements 8.3）。対応が無ければ
    /// 空 `Vec`。
    pub fn resolve_pasta_to_lua(&self, pasta_file: &str, pasta_line: u32) -> Vec<(ChunkName, u32)> {
        let file_key = canonicalize_pasta_file(pasta_file);
        self.reverse
            .get(&file_key)
            .and_then(|per_file| per_file.get(&pasta_line))
            .cloned()
            .unwrap_or_default()
    }

    /// `from_line` 以上で対応を持つ **最初の** `.pasta` 行を返す（requirements 4.3
    /// 最近接調整）。
    ///
    /// `pasta_file` を [`canonicalize_pasta_file`] で正規化し、逆引き索引（内側
    /// `BTreeMap`）の `range(from_line..)` の先頭キーを返す。`from_line` 自身が対応を
    /// 持てば `from_line`。`from_line` 以降に対応が無ければ `None`。指定 `.pasta` 行に
    /// 対応 `.lua` 行が無いブレークポイントを、後続最近接の有効位置へ調整するために
    /// 用いる。
    pub fn nearest_pasta_line_with_mapping(&self, pasta_file: &str, from_line: u32) -> Option<u32> {
        let file_key = canonicalize_pasta_file(pasta_file);
        self.reverse
            .get(&file_key)?
            .range(from_line..)
            .next()
            .map(|(&pasta_line, _)| pasta_line)
    }
}

/// 1 チャンクの前方写像を `(lua_line, pasta_line)` の **`.lua` 行昇順**反復子として
/// 借用する内部ヘルパ。
///
/// [`SourceMap::insert_chunk`] が逆引き索引を構築する際に用いる。[`ChunkSourceMap`]
/// は前方写像 `forward`（`BTreeMap<u32, PastaPos>`）を非公開で保持するため、
/// 同一モジュール内の本ヘルパは公開 API [`ChunkSourceMap::pasta_for_lua`] /
/// [`ChunkSourceMap::lua_lines_for_pasta`] では取り出せない「全 `(lua_line,
/// pasta_line)` ペアの昇順列」を、`forward` フィールドへ直接アクセスして提供する。
pub(super) fn map_forward_iter(map: &ChunkSourceMap) -> impl Iterator<Item = (u32, u32)> + '_ {
    map.forward.iter().map(|(&lua_line, pos)| (lua_line, pos.line))
}


// ===========================================================================
// 任意ディスクサイドカー I/O（task 6.1・3.2）を child module へ分離（task 7.5・C5）。
// `SidecarFile`/`SIDECAR_VERSION`/`sidecar_path_for_lua`/`write_sidecar`/
// `read_sidecar` の **公開パス**（`crate::debug::source_map::*`）を維持するため、
// child の公開項目をハブから glob 再エクスポートする（外部 consumer = loader の
// `crate::debug::source_map::write_sidecar` 経路は不変）。
// ===========================================================================
mod sidecar;
pub use sidecar::{SIDECAR_VERSION, SidecarFile, read_sidecar, sidecar_path_for_lua, write_sidecar};

// ===========================================================================
// インラインテストの外出し（task 2.3・C1）。論理クラスタ別の FLAT 兄弟テスト
// ファイルへ分割する。クラスタ跨ぎ共有ヘルパー（`pos`/`sample_map`）は
// `source_map_test_support.rs` に `pub(super)` で集約し、各クラスタが
// `use super::source_map_test_support::*;` で参照する（本番可視性は不変）。
// ===========================================================================

/// クラスタ跨ぎ共有テストヘルパー（`pub(super)`・test-only）。
#[cfg(test)]
#[path = "../source_map_test_support.rs"]
mod source_map_test_support;

/// 解決系: `canonicalize_chunk_name` / `ChunkSourceMap` / `SourceMap`。
#[cfg(test)]
#[path = "../source_map_resolve_tests.rs"]
mod source_map_resolve_tests;

/// 構築系: `MapBuilderSink`（記録 → `finish` 補正）。
#[cfg(test)]
#[path = "../source_map_builder_tests.rs"]
mod source_map_builder_tests;

/// サイドカー系: `SidecarWriter`（任意ディスク出力・往復同一性・非致命）。
#[cfg(test)]
#[path = "../source_map_sidecar_tests.rs"]
mod source_map_sidecar_tests;

