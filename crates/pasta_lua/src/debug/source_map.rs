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
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use crate::code_gen::source_map::{PastaPos, SourceMapSink};
use crate::normalize::LineShift;

/// チャンク（1 つの生成 `.lua` ファイル）を識別するキー。
///
/// ランタイムのラインフックが報告する `lua_Debug.source`（`@<絶対 .lua パス>` 想定）
/// に **正規化キー**で一致させる（design "Source Identity" 437-440・
/// [`canonicalize_chunk_name`]）。マルチチャンク集約 `SourceMap` のキー型として
/// 後続タスク（3.4）が用いる。本タスク（3.2）では型エイリアスのみ提供する。
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
/// ビルドパイプライン・`finish(shift)` rebase）は後続タスク（3.3）の担当であり、
/// 本タスクでは [`from_forward`](Self::from_forward) による構築のみを提供する。
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
    /// 後続タスク（3.3）の担当であり、本コンストラクタは主にテスト・3.3 の
    /// `finish` から `forward` を直接渡す用途に供する。
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
    /// 本タスク（3.3）では per-chunk マップの確定までを担い、マルチチャンク集約
    /// `SourceMap` への登録（task 3.4）でこのキーを用いる。`finish` の戻り値
    /// [`ChunkSourceMap`] 自体はチャンク名を保持しないため、本フィールドは集約側が
    /// 参照する識別子として保持する。
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
/// stepper）へ読み取り専用共有される（design 434・`Arc` 化は後続タスク 4.x の担当で
/// 本タスクは型・構築・解決のみを提供する）。
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

    /// 1 チャンクを集約へ登録する（loader（task 4.3）が per-`.pasta` の `finish` 結果を
    /// 投入する想定の builder API）。
    ///
    /// `chunk_name`（生フック源／ローダ由来パス）を [`canonicalize_chunk_name`] で
    /// 正規化して [`chunks`](Self::chunks) のキーとし、`pasta_file` を
    /// [`canonicalize_pasta_file`] で正規化して [`reverse`](Self::reverse) のファイル
    /// キーとする。さらにチャンクの前方写像を **`.lua` 行昇順**で走査して逆引きへ追記
    /// する。
    ///
    /// 逆引き各 `.pasta` 行の `Vec` は、追記後に **チャンク名昇順 → `.lua` 行昇順**で
    /// 安定ソートし、提示順序を決定的にする（design 435・requirements 8.3）。
    pub fn insert_chunk(&mut self, chunk_name: ChunkName, pasta_file: String, map: ChunkSourceMap) {
        let chunk_key = canonicalize_chunk_name(&chunk_name);
        let file_key = canonicalize_pasta_file(&pasta_file);

        // 逆引き索引を、このチャンクの前方写像（`.lua` 行昇順の `BTreeMap`）から構築。
        let per_file = self.reverse.entry(file_key).or_default();
        // `forward` の反復は `.lua` 行昇順（BTreeMap）。各 `.pasta` 行へ
        // (chunk_key, lua_line) を追記する。
        for lua_line in map_forward_iter(&map) {
            let (lua_line, pasta_line) = lua_line;
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
fn map_forward_iter(map: &ChunkSourceMap) -> impl Iterator<Item = (u32, u32)> + '_ {
    map.forward.iter().map(|(&lua_line, pos)| (lua_line, pos.line))
}

// ===========================================================================
// SidecarWriter（task 6.1・任意ディスクサイドカー出力・3.2）
// ===========================================================================

/// 現行サイドカースキーマのバージョン（前方/後方互換のため・design Data Contracts
/// 602）。`version` フィールドに載せ、将来スキーマ変更時に読み手が判別できるようにする。
pub const SIDECAR_VERSION: u32 = 1;

/// 生成 `.lua` の隣に出力する **任意ディスクサイドカー**の serde 表現（3.2・design
/// "SidecarWriter" 478-485 / Data Contracts 601-602）。
///
/// メモリ内 [`ChunkSourceMap`] の前方写像（最終 `.lua` 行 → `.pasta` 位置）を、最小・
/// 独自スキーマの JSON へ落とす（**Source Map v3 非採用**・research.md D-4）。スキーマ:
///
/// - `version`: スキーマ版（[`SIDECAR_VERSION`]）。前方/後方互換のため（602）。
/// - `pasta_file`: 由来 `.pasta` ファイルパス（[`PastaPos::file`]）。
/// - `pairs`: `[lua_line, pasta_line]` の行ペア列。`.lua` 行の **昇順かつ決定的**
///   （`ChunkSourceMap.forward` の `BTreeMap` がキー昇順のため・8.3）。
///
/// 単一の `.pasta` ファイルから 1 チャンクを生成する前提のため、ペアは `pasta_line`
/// のみを持ち、ファイルは `pasta_file` に一本化する（チャンク内で `PastaPos::file` は
/// 常に `pasta_file` と一致する）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarFile {
    /// スキーマ版（[`SIDECAR_VERSION`]）。
    pub version: u32,
    /// 由来 `.pasta` ファイルパス。
    pub pasta_file: String,
    /// `[lua_line, pasta_line]` の行ペア列（`.lua` 行昇順・決定的）。
    pub pairs: Vec<[u32; 2]>,
}

impl SidecarFile {
    /// メモリ内 [`ChunkSourceMap`] と由来 `.pasta` ファイルからサイドカー表現を構築する。
    ///
    /// 前方写像（`BTreeMap`・`.lua` 行昇順）を走査して `[lua_line, pasta_line]` を
    /// 収集するため、`pairs` は決定論的（同一マップから常に同一バイト列・冪等性
    /// design 484）。
    pub fn from_chunk(pasta_file: impl Into<String>, map: &ChunkSourceMap) -> Self {
        let pairs = map_forward_iter(map)
            .map(|(lua_line, pasta_line)| [lua_line, pasta_line])
            .collect();
        Self {
            version: SIDECAR_VERSION,
            pasta_file: pasta_file.into(),
            pairs,
        }
    }

    /// サイドカー表現をメモリ内 [`ChunkSourceMap`] へ復元する（再読込・往復同一性の
    /// 検証経路）。
    ///
    /// 各 `[lua_line, pasta_line]` を `pasta_file` 由来の [`PastaPos`] として
    /// `forward` へ復元する。`from_chunk` の逆操作であり、`write_sidecar`→`read_sidecar`
    /// の往復でメモリ写像と一致する（3.2 完了条件）。
    pub fn to_chunk(&self) -> ChunkSourceMap {
        let mut forward = BTreeMap::new();
        for &[lua_line, pasta_line] in &self.pairs {
            forward.insert(
                lua_line,
                PastaPos {
                    file: self.pasta_file.clone(),
                    line: pasta_line,
                },
            );
        }
        ChunkSourceMap::from_forward(forward)
    }
}

/// 生成 `.lua` パスに対応するサイドカーパス `<lua_path>.map` を導出する（design
/// "SidecarWriter" Output 483: 各生成 `.lua` の隣に `<chunk>.lua.map`）。
///
/// 例: `.../scene/sys.lua` → `.../scene/sys.lua.map`。拡張子 `.map` を **付加**する
/// （`.lua` を置換しない）ため、生成 `.lua` の真隣に決定的なファイル名で並ぶ。
pub fn sidecar_path_for_lua(lua_path: &Path) -> PathBuf {
    let mut name = lua_path.as_os_str().to_os_string();
    name.push(".map");
    PathBuf::from(name)
}

/// 1 チャンクのサイドカーを生成 `.lua` の隣（`<lua_path>.map`）へ出力する（3.2・
/// design "SidecarWriter" 478-485）。
///
/// メモリ内 [`ChunkSourceMap`] を [`SidecarFile`]（`version`＋`pasta_file`＋行ペア列）
/// として serde_json で直列化し、[`sidecar_path_for_lua`] が指すパスへ書き込む。出力は
/// `BTreeMap` 昇順走査で **決定論的**（再トランスパイルで同一内容・冪等性 design 484）。
///
/// # 失敗は非致命（3.1 / design Error 611, 616）
///
/// この関数は I/O 失敗を [`Result`] で返すが、**致命ではない**。呼び出し側（loader）は
/// `Err` を `tracing::warn!` でログして握り潰し、メモリ既定経路（メモリ内 `SourceMap`）
/// を一切変更せず継続しなければならない。本関数自身はメモリ写像を読むだけで変更せず、
/// 書き込み失敗時も `Err` を返すのみで panic/abort しない。
pub fn write_sidecar(
    lua_path: &Path,
    pasta_file: &str,
    map: &ChunkSourceMap,
) -> std::io::Result<()> {
    let sidecar = SidecarFile::from_chunk(pasta_file, map);
    // serde_json::to_vec はメモリ内シリアライズのみで I/O を伴わない。失敗（理論上
    // 起こらない）も I/O エラーへ写して非致命の単一経路に統一する。
    let bytes = serde_json::to_vec(&sidecar)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let path = sidecar_path_for_lua(lua_path);
    // 生成 `.lua` の親ディレクトリは本番経路（loader の `save_cache`）が先に作るが、
    // 呼び出し順に依存せず堅牢にするため `save_cache` と同様に親を idempotent に作る。
    // 作成失敗も I/O エラーとして返り（非致命・呼び出し側が warn 継続・3.1/611）。
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, bytes)
}

/// `<lua_path>.map` サイドカーを読み戻し、メモリ内 [`ChunkSourceMap`] へ復元する
/// （往復同一性の検証経路・3.2 完了条件）。
///
/// [`sidecar_path_for_lua`] が指すサイドカーを読み、serde_json で [`SidecarFile`] へ
/// デシリアライズし、[`SidecarFile::to_chunk`] でメモリ写像へ戻す。
/// [`write_sidecar`] の逆操作であり、往復で元のメモリ写像と一致する（同一 `lua_line`
/// → `pasta_line` ペア）。
pub fn read_sidecar(lua_path: &Path) -> std::io::Result<ChunkSourceMap> {
    let path = sidecar_path_for_lua(lua_path);
    let bytes = std::fs::read(&path)?;
    let sidecar: SidecarFile = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(sidecar.to_chunk())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_dsl::parser::Span;

    // =======================================================================
    // canonicalize_chunk_name（task 1.1 で確定した正規化規則の単体仕様）
    // =======================================================================

    /// `@` 接頭辞・パス区切り混在・大小文字の正規化を、実機（task 1.1）で実測した
    /// 形に基づいて固定する。フック source（`@` 付き・区切り混在）とローダ由来キー
    /// （区切り混在）が **同一正規形** へ落ちることを表明する（requirements 4.2/5.1・
    /// design "Source Identity" 437-440）。
    #[test]
    fn canonicalize_chunk_name_unifies_at_separators_and_case() {
        // 実機実測のフック source 形（`@` 付き・前置部 `/`・モジュール展開部 `\`）。
        let hook = r"@C:/base/profile/pasta/cache/lua/pasta\scene\sys.lua";
        // 実機実測のローダ由来キー形（区切り混在の絶対パス）。
        let loader = r"C:\base\profile/pasta/cache/lua\pasta/scene\sys.lua";

        let hook_canon = canonicalize_chunk_name(hook);
        let loader_canon = canonicalize_chunk_name(loader);

        // `@` は除去される。
        assert!(!hook_canon.starts_with('@'));
        // 区切りは `/` に統一される（`\` は残らない）。
        assert!(!hook_canon.contains('\\'));
        assert!(!loader_canon.contains('\\'));
        // フック source とローダ由来キーは同一正規形へ落ちる（突合可能）。
        assert_eq!(
            hook_canon, loader_canon,
            "正規化後はフック source とローダ由来キーが一致しなければならない"
        );
    }

    /// Windows は大小文字無視（ドライブレター/フォルダ名の大小差を吸収）。非 Windows
    /// は大小を保持する。`#[cfg]` で実機規約に整合させる。
    #[test]
    fn canonicalize_chunk_name_case_policy_matches_platform() {
        let upper = canonicalize_chunk_name(r"@C:/Base/SYS.lua");
        let lower = canonicalize_chunk_name(r"@c:/base/sys.lua");
        #[cfg(windows)]
        assert_eq!(upper, lower, "Windows は大小文字無視");
        #[cfg(not(windows))]
        assert_ne!(upper, lower, "非 Windows は大小を区別する");
    }

    // =======================================================================
    // ChunkSourceMap（task 3.2・1 チャンクの双方向行写像）
    // =======================================================================

    /// テスト用の `.pasta` 位置を構築する小ヘルパ（`file` は固定）。
    fn pos(line: u32) -> PastaPos {
        PastaPos {
            file: "dict.pasta".to_string(),
            line,
        }
    }

    /// 既知 forward マップから `ChunkSourceMap` を構築する小ヘルパ。
    ///
    /// マップ内容: 最終 `.lua` 行 10→`.pasta` 3, 12→`.pasta` 7, 13→`.pasta` 7,
    /// 15→`.pasta` 7, 20→`.pasta` 9。`.lua` 11/14 など gap を残す。
    fn sample_map() -> ChunkSourceMap {
        let mut forward = BTreeMap::new();
        forward.insert(10u32, pos(3));
        forward.insert(12u32, pos(7));
        forward.insert(13u32, pos(7));
        forward.insert(15u32, pos(7));
        forward.insert(20u32, pos(9));
        ChunkSourceMap::from_forward(forward)
    }

    /// 2.2: 対応を持つ最終 `.lua` 行は、由来 `.pasta` 位置を一意に解決できる。
    #[test]
    fn chunk_source_map_resolves_mapped_lua_line() {
        let map = sample_map();
        assert_eq!(map.pasta_for_lua(10), Some(&pos(3)));
        assert_eq!(map.pasta_for_lua(20), Some(&pos(9)));
        // 同一 `.pasta` 行 7 へ集約される複数 `.lua` 行も各々正しく解決する。
        assert_eq!(map.pasta_for_lua(12), Some(&pos(7)));
        assert_eq!(map.pasta_for_lua(15), Some(&pos(7)));
    }

    /// 1.2 / 2.3: 対応を持たない（補助/挿入行＝gap）最終 `.lua` 行は「対応なし」
    /// （`None`）を返し、誤った `.pasta` 位置へ対応づけない。
    #[test]
    fn chunk_source_map_unmapped_lua_line_returns_none() {
        let map = sample_map();
        // gap（記録されていない `.lua` 行）。
        assert_eq!(map.pasta_for_lua(11), None);
        assert_eq!(map.pasta_for_lua(14), None);
        // 範囲外の行も「対応なし」。
        assert_eq!(map.pasta_for_lua(1), None);
        assert_eq!(map.pasta_for_lua(999), None);
    }

    /// 8.2: 単一 `.pasta` 行が複数 `.lua` 行へ展開される場合、逆引きはそれら全ての
    /// `.lua` 行を返す。さらに 8.3: 返り値は `.lua` 行の昇順かつ決定的順序。
    #[test]
    fn chunk_source_map_reverse_returns_all_lua_lines_ascending() {
        let map = sample_map();
        // `.pasta` 行 7 へは `.lua` 12/13/15 が展開される（昇順）。
        assert_eq!(map.lua_lines_for_pasta(7), vec![12, 13, 15]);
        // 単一対応の `.pasta` 行も Vec で返る。
        assert_eq!(map.lua_lines_for_pasta(3), vec![10]);
        assert_eq!(map.lua_lines_for_pasta(9), vec![20]);
        // 対応を持たない `.pasta` 行は空 Vec。
        assert_eq!(map.lua_lines_for_pasta(100), Vec::<u32>::new());
    }

    /// 8.1: forward は最終 `.lua` 行をキーとし、1 `.lua` 行 → 高々 1 `.pasta` 位置。
    /// 同一キーへの後勝ち（last-write-wins）を `from_forward` 経由で確認する
    /// （BTreeMap キー一意性が不変条件を担保）。
    #[test]
    fn chunk_source_map_one_lua_line_maps_to_at_most_one_pasta() {
        let mut forward = BTreeMap::new();
        forward.insert(10u32, pos(3));
        // 同一 `.lua` 行への再挿入は上書き（last-write-wins）。
        forward.insert(10u32, pos(8));
        let map = ChunkSourceMap::from_forward(forward);
        assert_eq!(map.pasta_for_lua(10), Some(&pos(8)));
        // 逆引きでも後勝ちのみが見える（行 3 へは対応なし）。
        assert_eq!(map.lua_lines_for_pasta(3), Vec::<u32>::new());
        assert_eq!(map.lua_lines_for_pasta(8), vec![10]);
    }

    /// 8.3: 逆引きの提示順序は決定的（同一入力で繰り返し呼んでも同一 Vec）。
    #[test]
    fn chunk_source_map_reverse_is_deterministic_across_calls() {
        let map = sample_map();
        let first = map.lua_lines_for_pasta(7);
        let second = map.lua_lines_for_pasta(7);
        let third = map.lua_lines_for_pasta(7);
        assert_eq!(first, second);
        assert_eq!(second, third);
        assert_eq!(first, vec![12, 13, 15]);
    }

    // =======================================================================
    // MapBuilderSink（task 3.3・記録→補正でチャンク写像を確定）
    // =======================================================================

    use crate::normalize::normalize_output_with_shift;

    /// 2.1: 記録（pre-normalize 行）→ `finish(shift)` 補正で、チャンク写像が最終
    /// `.lua` 行に整合する。削除行（normalize が落とした空行）に紐づく記録は除外される。
    ///
    /// 実 `normalize_output_with_shift` で本物の `LineShift` を得る（テスト用の
    /// LineShift 直接構築は不可＝`deleted` 非公開のため、実 normalize 経路を使う）。
    /// 入力 `"l1\nl2\nl3\n\nend\n"` の pre-normalize 行（1 始まり）:
    ///   1: "l1"  -> 最終 1
    ///   2: "l2"  -> 最終 2
    ///   3: "l3"  -> 最終 3
    ///   4: ""    -> 削除（`end` 直前の空行）-> None
    ///   5: "end" -> 最終 4
    #[test]
    fn map_builder_sink_finish_rebases_to_final_lua_lines() {
        let input = "l1\nl2\nl3\n\nend\n";
        let (out, shift) = normalize_output_with_shift(input);
        // 前提の確認: 空行(pre 4)が削除され "end" が最終 4 行へ繰り上がる。
        assert_eq!(out, "l1\nl2\nl3\nend\n");
        assert_eq!(shift.map(1), Some(1));
        assert_eq!(shift.map(4), None); // 削除行
        assert_eq!(shift.map(5), Some(4)); // "end" が繰り上がる

        let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk-a".to_string());
        // producer は pre-normalize の out_line を記録する。
        sink.record_line(1, 10); // pre 1 -> .pasta 10
        sink.record_line(2, 11); // pre 2 -> .pasta 11
        sink.record_line(3, 12); // pre 3 -> .pasta 12
        sink.record_line(4, 99); // pre 4（削除行）-> finish で除外されるべき
        sink.record_line(5, 20); // pre 5 -> .pasta 20（"end" 行）

        let map = sink.finish(&shift);

        // pre-normalize 行が最終 `.lua` 行へ rebase された。
        assert_eq!(map.pasta_for_lua(1), Some(&pos(10)));
        assert_eq!(map.pasta_for_lua(2), Some(&pos(11)));
        assert_eq!(map.pasta_for_lua(3), Some(&pos(12)));
        // pre 5 は最終 4 へ繰り上がる。
        assert_eq!(map.pasta_for_lua(4), Some(&pos(20)));
        // 削除行（pre 4）の記録は除外され、最終 .lua には現れない。
        assert!(
            !map.pasta_for_lua(5).is_some(),
            "削除行に紐づく記録は最終写像に残ってはならない（2.1）"
        );
        // 削除によって生じる旧位置（pre 5 のままの行 5）は存在しない。
        assert_eq!(map.len(), 4);
    }

    /// 8.1: 同一 pre-normalize 行を 2 回記録すると、後勝ち（last-write-wins）で
    /// 単一の決定論的 `.pasta` 位置になる。BTreeMap キー一意性が担保。
    #[test]
    fn map_builder_sink_same_pre_line_is_last_write_wins() {
        // 削除なし（恒等）の本物 shift を使う。
        let (out, shift) = normalize_output_with_shift("a\nb\n");
        assert_eq!(out, "a\nb\n");
        assert_eq!(shift.map(1), Some(1));
        assert_eq!(shift.map(2), Some(2));

        let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());
        sink.record_line(2, 5); // 先の記録
        sink.record_line(2, 8); // 同一 pre-line を再記録 -> 後勝ち

        let map = sink.finish(&shift);
        // 最終行 2 は後勝ちの `.pasta` 8 に確定（決定論的・単一位置）。
        assert_eq!(map.pasta_for_lua(2), Some(&pos(8)));
        // 旧位置（`.pasta` 5）は残らない。
        assert_eq!(map.lua_lines_for_pasta(5), Vec::<u32>::new());
        assert_eq!(map.lua_lines_for_pasta(8), vec![2]);
    }

    /// 8.1: 確定結果は決定論的（同一記録列・同一 shift で繰り返し finish しても同一）。
    #[test]
    fn map_builder_sink_finish_is_deterministic() {
        let (_out, shift) = normalize_output_with_shift("a\nb\nc\n");

        let build = || {
            let mut sink =
                MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());
            sink.record_line(3, 7);
            sink.record_line(1, 3);
            sink.record_line(2, 5);
            let map = sink.finish(&shift);
            (
                map.pasta_for_lua(1).cloned(),
                map.pasta_for_lua(2).cloned(),
                map.pasta_for_lua(3).cloned(),
            )
        };

        assert_eq!(build(), build());
        assert_eq!(
            build(),
            (Some(pos(3)), Some(pos(5)), Some(pos(7)))
        );
    }

    /// `.pasta` 行は `span.start_line` を **直接**採用する（research.md D-3）。trait 既定の
    /// `record(lua_line, span)` が `record_line(lua_line, span.start_line)` へ委譲する
    /// ことを、`finish` 後の最終写像で表明する（byte 走査廃止）。
    #[test]
    fn map_builder_sink_record_uses_span_start_line() {
        let (_out, shift) = normalize_output_with_shift("a\nb\n");
        let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());

        // Span::new(start_line, start_col, end_line, end_col, start_byte, end_byte)
        // start_line=42 を採用すべき（start_byte=9999 のバイト走査は使わない）。
        let span = Span::new(42, 1, 42, 9, 9999, 10001);
        sink.record(1, span); // pre 1 -> .pasta start_line(42)

        let map = sink.finish(&shift);
        assert_eq!(
            map.pasta_for_lua(1),
            Some(&pos(42)),
            "record は span.start_line を直接 .pasta 行として採用する（D-3）"
        );
    }

    /// `chunk_name` アクセサがコンストラクタ引数を保持する（3.4 集約が参照する識別子）。
    #[test]
    fn map_builder_sink_retains_chunk_name() {
        let sink = MapBuilderSink::new("dict.pasta".to_string(), "my-chunk".to_string());
        assert_eq!(sink.chunk_name(), "my-chunk");
    }

    // =======================================================================
    // SourceMap（task 3.4・マルチチャンク集約と双方向解決（正規化キー））
    // =======================================================================

    /// 指定 `.pasta` ファイルの位置を作る小ヘルパ。
    fn pos_in(file: &str, line: u32) -> PastaPos {
        PastaPos {
            file: file.to_string(),
            line,
        }
    }

    /// 2 チャンク（異なるチャンク名・異なる `.pasta` ファイル）を持つ集約 `SourceMap`
    /// を構築する。
    ///
    /// - chunk "a"（`.pasta` ファイル `C:/proj/scene/a.pasta`）:
    ///     最終 `.lua` 10→`.pasta` 3, 12→`.pasta` 7, 13→`.pasta` 7, 20→`.pasta` 9
    /// - chunk "b"（`.pasta` ファイル `C:/proj/scene/b.pasta`）:
    ///     最終 `.lua` 5→`.pasta` 7（chunk a と同じ `.pasta` 行番号だが別ファイル）
    ///
    /// さらに、**同一 `.pasta` ファイルが複数チャンクへ写像される**ケース（4.1 クロス
    /// チャンク）を表現するため chunk "c" も同じ `C:/proj/scene/a.pasta` を使い、
    /// 最終 `.lua` 4→`.pasta` 7 を持たせる。
    fn sample_source_map() -> SourceMap {
        let file_a = "C:/proj/scene/a.pasta";
        let file_b = "C:/proj/scene/b.pasta";

        let mut fwd_a = BTreeMap::new();
        fwd_a.insert(10u32, pos_in(file_a, 3));
        fwd_a.insert(12u32, pos_in(file_a, 7));
        fwd_a.insert(13u32, pos_in(file_a, 7));
        fwd_a.insert(20u32, pos_in(file_a, 9));

        let mut fwd_b = BTreeMap::new();
        fwd_b.insert(5u32, pos_in(file_b, 7));

        let mut fwd_c = BTreeMap::new();
        fwd_c.insert(4u32, pos_in(file_a, 7));

        let mut sm = SourceMap::new();
        // チャンク名は呼び出し側があえて非正規形（`@`・`\`・大小混在）で渡す。
        sm.insert_chunk(
            r"@C:\proj\cache\A.lua".to_string(),
            file_a.to_string(),
            ChunkSourceMap::from_forward(fwd_a),
        );
        sm.insert_chunk(
            r"@C:\proj\cache\b.lua".to_string(),
            file_b.to_string(),
            ChunkSourceMap::from_forward(fwd_b),
        );
        sm.insert_chunk(
            r"@C:\proj\cache\c.lua".to_string(),
            file_a.to_string(),
            ChunkSourceMap::from_forward(fwd_c),
        );
        sm
    }

    /// 正規化キーで chunk を引いて期待される解決後 `.pasta` を返す共通アサート。
    ///
    /// 3.3 / Source Identity: `resolve_lua_to_pasta` の `chunk` 引数を `@` 付き・区切り
    /// 混在・大小違いで渡しても、格納時の正規形と一致して解決できることを表明する。
    #[test]
    fn source_map_resolve_lua_to_pasta_matches_normalized_chunk_key() {
        let sm = sample_source_map();
        let file_a = "C:/proj/scene/a.pasta";

        // 格納時とは異なる等価形（前方スラッシュ・大小違い・`@` 無し）でも一致する。
        assert_eq!(
            sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 10),
            Some(&pos_in(file_a, 3))
        );
        // 別の等価形（`@` 付き・別大小）でも一致。
        #[cfg(windows)]
        assert_eq!(
            sm.resolve_lua_to_pasta(r"@C:\PROJ\CACHE\A.LUA", 20),
            Some(&pos_in(file_a, 9))
        );
        // 1 `.pasta` 行 7 へ集約される複数 `.lua` 行も各々解決できる。
        assert_eq!(
            sm.resolve_lua_to_pasta("c:/proj/cache/a.lua", 12),
            Some(&pos_in(file_a, 7))
        );
    }

    /// 整合性エラー（design 610/617）: フック source が `SourceMap` キーに無い場合は
    /// `None`（誤った `.pasta` 対応づけを禁止・2.3）。対応の無い `.lua` 行も `None`。
    #[test]
    fn source_map_resolve_lua_to_pasta_chunk_or_line_miss_returns_none() {
        let sm = sample_source_map();
        // チャンク名不一致 -> None（.lua フォールバック）。
        assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/unknown.lua", 10), None);
        // 既知チャンクだが対応の無い `.lua` 行 -> None。
        assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 11), None);
        assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 999), None);
    }

    /// 4.1 / 3.3: `.pasta` 行 → 全 `(chunk, lua_line)` を昇順・決定的に返す。
    /// 1 `.pasta` 行が複数 `.lua` 行・複数チャンクへ展開されるケースを含む。
    /// 照合は正規化された `.pasta` ファイルキーで行う（区切り・大小違いでも一致）。
    #[test]
    fn source_map_resolve_pasta_to_lua_returns_all_ascending() {
        let sm = sample_source_map();

        // `C:/proj/scene/a.pasta` の `.pasta` 行 7:
        //   - chunk a の `.lua` 12, 13
        //   - chunk c の `.lua` 4
        // 期待: `(chunk, lua_line)` を **チャンク名昇順 → `.lua` 行昇順**で決定的に
        // （design 435・8.3）。正規化キーで集約されるため chunk キーは格納時の正規形
        // （Windows は小文字・`/`・`@` 無し）になる。チャンク名 "a.lua" < "c.lua"。
        let resolved = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
        #[cfg(windows)]
        let expected = vec![
            ("c:/proj/cache/a.lua".to_string(), 12u32),
            ("c:/proj/cache/a.lua".to_string(), 13u32),
            ("c:/proj/cache/c.lua".to_string(), 4u32),
        ];
        // 非 Windows では正規形は元の大小（小文字化しない）。"a.lua" < "c.lua"。
        #[cfg(not(windows))]
        let expected = vec![
            ("C:/proj/cache/a.lua".to_string(), 12u32),
            ("C:/proj/cache/a.lua".to_string(), 13u32),
            ("C:/proj/cache/c.lua".to_string(), 4u32),
        ];
        assert_eq!(resolved, expected);

        // 別ファイル `b.pasta` の同じ `.pasta` 行 7 は chunk b の `.lua` 5 のみ
        // （ファイルキーで分離されている）。区切り違いの query でも一致。
        let resolved_b = sm.resolve_pasta_to_lua(r"C:\proj\scene\b.pasta", 7);
        #[cfg(windows)]
        assert_eq!(resolved_b, vec![("c:/proj/cache/b.lua".to_string(), 5u32)]);
        #[cfg(not(windows))]
        assert_eq!(resolved_b, vec![("C:/proj/cache/b.lua".to_string(), 5u32)]);

        // 対応の無い `.pasta` 行は空 Vec。
        assert!(sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 100).is_empty());
        // 未知ファイルも空 Vec。
        assert!(sm.resolve_pasta_to_lua("C:/proj/scene/zzz.pasta", 7).is_empty());
    }

    /// 4.3 最近接調整: `from_line` に対応が無ければ、それ以上で対応を持つ最初の
    /// `.pasta` 行を返す。`from_line` 自身が対応を持てば `from_line`。対応が
    /// `from_line` 以降に無ければ `None`。照合は正規化ファイルキー。
    #[test]
    fn source_map_nearest_pasta_line_with_mapping() {
        let sm = sample_source_map();
        // a.pasta の対応 `.pasta` 行は {3, 7, 9}。
        // from_line=3（自身が対応を持つ）-> 3。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 3), Some(3));
        // from_line=4（対応なし）-> 次の対応行 7。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4), Some(7));
        // from_line=8（対応なし）-> 次の対応行 9。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 8), Some(9));
        // from_line=1（最小対応行 3 より前）-> 3。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 1), Some(3));
        // from_line=10（最大対応行 9 より後）-> None。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 10), None);
        // 正規化キー（区切り・大小違い）でも一致。
        #[cfg(windows)]
        assert_eq!(sm.nearest_pasta_line_with_mapping(r"C:\PROJ\Scene\A.pasta", 4), Some(7));
        // 未知ファイル -> None。
        assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/zzz.pasta", 1), None);
    }

    /// 8.3: 双方向解決の提示順序は決定的（同一入力で繰り返し呼んでも同一）。
    #[test]
    fn source_map_resolution_is_deterministic_across_calls() {
        let sm = sample_source_map();
        let first = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
        let second = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
        assert_eq!(first, second);
        assert_eq!(
            sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4),
            sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4)
        );
    }

    // =======================================================================
    // SidecarWriter（task 6.1・任意ディスクサイドカー出力・3.2）
    // =======================================================================

    use tempfile::TempDir;

    /// サイドカーパスは生成 `.lua` の真隣（`<lua_path>.map`・design 483）。
    #[test]
    fn sidecar_path_appends_map_next_to_lua() {
        let lua = Path::new("/cache/pasta/scene/sys.lua");
        assert_eq!(
            sidecar_path_for_lua(lua),
            PathBuf::from("/cache/pasta/scene/sys.lua.map"),
            "サイドカーは生成 .lua の隣に <lua>.map として並ぶ（3.2）"
        );
    }

    /// 3.2 完了条件（往復同一性）: `write_sidecar` で出力 → `read_sidecar` で再読込し、
    /// 再読込した行ペア写像がメモリ内 [`ChunkSourceMap`] と一致する。さらに JSON が
    /// `version`＋`pasta_file`＋行ペア列を含むことを表明する（design 601-602）。
    #[test]
    fn write_then_read_sidecar_round_trips_to_memory_map() {
        let dir = TempDir::new().unwrap();
        let lua_path = dir.path().join("sys.lua");
        let map = sample_map(); // 10→3, 12→7, 13→7, 15→7, 20→9
        let pasta_file = "dict.pasta";

        write_sidecar(&lua_path, pasta_file, &map).expect("write_sidecar must succeed");

        // サイドカーファイルが生成 .lua の隣に存在する。
        let sidecar = sidecar_path_for_lua(&lua_path);
        assert!(sidecar.exists(), "サイドカー <lua>.map が生成されること（3.2）");

        // JSON が version + pasta_file + ペア列を含む（自己記述スキーマ・602）。
        let raw = std::fs::read_to_string(&sidecar).unwrap();
        let parsed: SidecarFile = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.version, SIDECAR_VERSION, "version フィールドを持つ（602）");
        assert_eq!(parsed.pasta_file, pasta_file, "pasta_file フィールドを持つ（602）");
        // 行ペアは .lua 行昇順かつ決定的（8.3）。
        assert_eq!(
            parsed.pairs,
            vec![[10, 3], [12, 7], [13, 7], [15, 7], [20, 9]],
            "行ペア列は .lua 行昇順で [lua_line, pasta_line]（602/8.3）"
        );

        // 再読込した写像がメモリ内写像と一致する（往復同一性・3.2 完了条件）。
        let reread = read_sidecar(&lua_path).expect("read_sidecar must succeed");
        for lua_line in [10u32, 12, 13, 15, 20] {
            assert_eq!(
                reread.pasta_for_lua(lua_line),
                map.pasta_for_lua(lua_line),
                "再読込写像はメモリ写像と同一の lua_line→pasta_line ペアを持つ（3.2）"
            );
        }
        // gap も一致（対応なしは双方 None）。
        assert_eq!(reread.pasta_for_lua(11), None);
        assert_eq!(reread.len(), map.len());
        // 逆引きも一致（同一 .pasta 行 7 へ 12/13/15 が昇順で復元）。
        assert_eq!(reread.lua_lines_for_pasta(7), vec![12, 13, 15]);
    }

    /// 冪等性（design 484）: 同一マップを 2 回書くと **同一バイト列**になる
    /// （決定論的出力・再トランスパイルで同一内容）。
    #[test]
    fn write_sidecar_is_byte_deterministic() {
        let dir = TempDir::new().unwrap();
        let lua_a = dir.path().join("a.lua");
        let lua_b = dir.path().join("b.lua");
        let map = sample_map();

        write_sidecar(&lua_a, "dict.pasta", &map).unwrap();
        write_sidecar(&lua_b, "dict.pasta", &map).unwrap();

        let bytes_a = std::fs::read(sidecar_path_for_lua(&lua_a)).unwrap();
        let bytes_b = std::fs::read(sidecar_path_for_lua(&lua_b)).unwrap();
        assert_eq!(bytes_a, bytes_b, "同一マップ → 同一バイト列（冪等・484）");

        // 同一パスへの再書き込みも同一バイト列。
        write_sidecar(&lua_a, "dict.pasta", &map).unwrap();
        let bytes_a2 = std::fs::read(sidecar_path_for_lua(&lua_a)).unwrap();
        assert_eq!(bytes_a, bytes_a2, "再書き込みも同一バイト列（冪等・484）");
    }

    /// 非致命（3.1 / design Error 611, 616）: 書き込み不能パスへの `write_sidecar` は
    /// **panic せず** `Err` を返し、メモリ内写像は一切変更されない。呼び出し側はこの
    /// `Err` を warn でログして握り潰し、メモリ既定経路を継続する想定。
    #[test]
    fn write_sidecar_failure_is_non_fatal_and_leaves_memory_map_intact() {
        let dir = TempDir::new().unwrap();
        // 親パスを **ファイル**として先に作る。`write_sidecar` は親を
        // `create_dir_all` で作ろうとするが、同名ファイルが既にあるため失敗する
        // （`create_dir_all` がディレクトリを作れない・決定論的に I/O エラー・OS 非依存）。
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, b"i am a file, not a dir").unwrap();
        // サイドカーの親（= blocker/sub）がファイル配下になり作成不能。
        let bad_lua = blocker.join("sub").join("x.lua");
        assert!(blocker.is_file(), "前提: 親パスはファイル（ディレクトリ作成不能）");

        let map = sample_map();
        let before = map.len();

        // panic せず Err を返す（catch_unwind は不要だが、Err 経路であることを表明）。
        let result = write_sidecar(&bad_lua, "dict.pasta", &map);
        assert!(
            result.is_err(),
            "書き込み不能パスは Err を返す（非致命・611）。panic/abort しない"
        );

        // メモリ写像は一切変更されていない（読み取り専用・3.1）。
        assert_eq!(map.len(), before, "メモリ写像は不変（3.1）");
        assert_eq!(map.pasta_for_lua(10), Some(&pos(3)));
        assert_eq!(map.pasta_for_lua(20), Some(&pos(9)));
        // サイドカーファイルは生成されていない。
        assert!(!sidecar_path_for_lua(&bad_lua).exists());
    }
}
