//! シーン同一性索引（[`SceneIdentityIndex`]）— `.pasta` (ファイル, 行範囲) →
//! シーン識別子の逆引き索引（task 1.1・requirements 2.1/2.2/2.5/2.6/3.2）。
//!
//! # 役割
//!
//! 実行中エンジンが保持するロード済みソースマップの一部として、各 `.pasta`
//! ファイルのシーン宣言を「(宣言行, 終端行, 入れ子レベル, シーン識別子)」として
//! 保持し、クリック位置 (ファイル, 行) から所属シーンを確定する逆引きを提供する。
//! 行マッピング API（[`super::SourceMap::resolve_lua_to_pasta`] 等）には一切触れず、
//! 新規型として **追加** する（requirements 3.4 非破壊）。
//!
//! # 行範囲の定義（requirements 2.1）
//!
//! シーンの行範囲は「宣言行 〜 次の同レベル以上のシーン宣言の直前」であり、本索引は
//! 既に確定済みの `[start_line, end_line]`（両端 **inclusive**・1 始まり）を投入される。
//! 終端行の算出（次の同レベル以上宣言の直前 / ファイル末尾）は構築側（loader /
//! finalize join・task 2.2）が責務を持ち、本索引は確定済み範囲を保持・参照する。
//!
//! # 解決ポリシー（requirements 2.2 > 2.5 > 2.6）
//!
//! 1. **包含**: クリック行を範囲 `[start, end]` に含むシーンが 1 つ以上あれば、その中の
//!    **最内**（入れ子レベル最大 = local 優先）を選ぶ（2.2）。包含による確定を後方
//!    フォールバックに優先する。
//! 2. **後方フォールバック**: 包含が無い場合、クリック行と同じか **下方**（より大きい
//!    行番号）にある最近接の宣言を持つシーンを選ぶ（2.5）。
//! 3. **未検出**: 下方にも有効シーンが無ければ未検出（2.6）。空ファイル・未登録
//!    ファイルも未検出。
//!
//! # 計算量（requirements 8.3）
//!
//! 1 `.pasta` ファイルの宣言開始行 → [`SceneSpan`] を `BTreeMap` で昇順保持し、
//! 包含探索は「クリック行以下の開始行」を `range(..=line)` の後方走査で、後方
//! フォールバックは「クリック行以上の開始行」を `range(line..)` の先頭で引く。いずれも
//! `BTreeMap` の対数オーダ操作であり、SHIORI GET 処理ブロックを延ばさない（8.3）。

// NOTE: 本モジュールは foundation データ構造であり、構築側（loader の
// build_source_map・finalize join）と参照側（PositionResolver）への結線は
// task 2.2 / 3.1 で行う。結線完了までは公開項目が未使用のため、本モジュール内の
// dead_code を明示許可する（結線後に各項目が消費され警告は自然消滅する）。
#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};

use super::canonicalize_pasta_file;

/// 確定したシーン同一性（`scene_at` の返り値）。
///
/// ランタイム実 identity を **(scene_id, parent)** で表す（task 2.2・kick 解決の
/// 解決対象）。global は `scene_id=会話1, parent=None`、local は
/// `scene_id=挨拶_1, parent=Some(会話1)`。kick の resolver（task 3.1）は parent の
/// 有無で global 分岐 / local 分岐を選ぶため、両者を一体で返す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneIdentity {
    /// シーン識別子（global = `会話1`、local = `挨拶_1`）。
    pub scene_id: String,
    /// 親グローバルシーン（local のみ `Some(会話1)`・global は `None`）。
    pub parent: Option<String>,
}

/// 1 シーンの行範囲（両端 inclusive・1 始まり）・入れ子レベル・識別子。
///
/// `level` は入れ子の深さ（global = 0, その直下の local = 1, ...）。包含が複数該当する
/// とき最内（`level` 最大）を選ぶための判定に用いる（requirements 2.2）。`scene_id` は
/// `SceneRegistry` 由来のランタイム実 identity（例 `会話1`）を **そのまま** 保持する。
/// `parent` は local シーンの親グローバル（例 `会話1`）で、global は `None`（task 2.2）。
/// 本索引は識別子の FORMAT に非依存で、投入された文字列を格納・返却するのみ
/// （design Decision 1「runtime-authoritative capture」）。
#[derive(Debug, Clone)]
struct SceneSpan {
    /// 宣言行（1 始まり・inclusive）。`BTreeMap` のキーと一致する。
    start_line: u32,
    /// 終端行（1 始まり・inclusive）。「次の同レベル以上宣言の直前 / ファイル末尾」。
    end_line: u32,
    /// 入れ子レベル（global = 0, local = 1, ...）。最内優先判定に用いる。
    level: u32,
    /// シーン識別子（`SceneRegistry` 由来・FORMAT 非依存で格納）。
    scene_id: String,
    /// 親グローバルシーン（local のみ `Some`・global は `None`）。
    parent: Option<String>,
}

impl SceneSpan {
    /// 指定行が本シーンの範囲 `[start_line, end_line]`（両端 inclusive）に含まれるか。
    fn contains(&self, line: u32) -> bool {
        self.start_line <= line && line <= self.end_line
    }

    /// 確定したシーン同一性 `(scene_id, parent)` の所有コピーを返す。
    fn identity(&self) -> SceneIdentity {
        SceneIdentity {
            scene_id: self.scene_id.clone(),
            parent: self.parent.clone(),
        }
    }
}

/// シーン同一性索引（イミュータブル・ロード時に [`SceneIdentityIndexBuilder`] から確定）。
///
/// `pasta_file`（正規化キー）→ (`start_line` → [`SceneSpan`]) の二段構造。外側 `HashMap`
/// でファイル引き O(1)、内側 `BTreeMap` で開始行昇順保持し対数オーダ lookup（8.3）。
/// ファイルキーは [`canonicalize_pasta_file`] で STORE/QUERY 双方を同一正規形へ落とし、
/// VSCode 由来パスとの突合を可能にする（既存ソースマップと同一規則）。
#[derive(Debug, Clone, Default)]
pub struct SceneIdentityIndex {
    /// 正規化 `.pasta` ファイルキー → (宣言開始行 → シーン範囲)。
    files: HashMap<String, BTreeMap<u32, SceneSpan>>,
}

impl SceneIdentityIndex {
    /// 索引を組み立てる [`SceneIdentityIndexBuilder`] を返す。
    pub fn builder() -> SceneIdentityIndexBuilder {
        SceneIdentityIndexBuilder::default()
    }

    /// 位置 (`pasta_file`, `line`) からシーン識別子を確定する（requirements 2.1/2.2/2.5/2.6）。
    ///
    /// 解決順:
    /// 1. **包含**（2.1/2.2）: `line` を範囲に含むシーン群から最内（`level` 最大）を選ぶ。
    /// 2. **後方フォールバック**（2.5）: 包含が無ければ `line` と同じか下方の最近接宣言。
    /// 3. **未検出**（2.6）: 下方にも無ければ `None`。未登録ファイル・空ファイルも `None`。
    ///
    /// `line` は 1 始まり。返り値は格納された識別子の所有コピー（[`SceneIdentity`]・
    /// `(scene_id, parent)`）。
    pub fn scene_at(&self, pasta_file: &str, line: u32) -> Option<SceneIdentity> {
        let file_key = canonicalize_pasta_file(pasta_file);
        let scenes = self.files.get(&file_key)?;

        // (1) 包含探索: 開始行が `line` 以下のシーンを開始行降順（直近の宣言から）に
        // 走査し、範囲に含むものの中から最内（level 最大）を選ぶ。開始行が `line` を
        // 超えるシーンは `line` を範囲に含み得ない（start > line なら contains は偽）
        // ため `range(..=line)` で枝刈りできる（8.3）。
        let mut innermost: Option<&SceneSpan> = None;
        for (_, span) in scenes.range(..=line).rev() {
            if span.contains(line) {
                match innermost {
                    Some(cur) if cur.level >= span.level => {}
                    _ => innermost = Some(span),
                }
            }
        }
        if let Some(span) = innermost {
            return Some(span.identity());
        }

        // (2) 後方フォールバック（2.5）: 包含が無ければ、`line` と同じか下方（開始行 >=
        // line）の最近接シーン宣言を採る。`range(line..)` の先頭が最近接（8.3）。
        if let Some((_, span)) = scenes.range(line..).next() {
            return Some(span.identity());
        }

        // (3) 未検出（2.6）: 下方に有効シーンが無い。
        None
    }
}

/// [`SceneIdentityIndex`] のビルダ。
///
/// ファイルごとにシーン範囲を蓄積し、[`finish`](Self::finish) で不変な索引へ確定する。
/// 同一ファイル・同一開始行への再投入は last-write-wins（`BTreeMap` キー一意性）。
#[derive(Debug, Default)]
pub struct SceneIdentityIndexBuilder {
    /// 正規化 `.pasta` ファイルキー → (宣言開始行 → シーン範囲)。
    files: HashMap<String, BTreeMap<u32, SceneSpan>>,
}

impl SceneIdentityIndexBuilder {
    /// `.pasta` ファイルを索引に登録する（シーンが 0 件でも空エントリを作る）。
    ///
    /// シーンの無いファイル（空ファイル相当）も `scene_at` が未検出を返す対象として
    /// 明示的に存在させたい場合に用いる。`add_scene` も内部でファイルを生成するため、
    /// シーンを 1 件以上投入する場合は本メソッドの事前呼び出しは必須ではない。
    pub fn insert_file(&mut self, pasta_file: &str) {
        let file_key = canonicalize_pasta_file(pasta_file);
        self.files.entry(file_key).or_default();
    }

    /// 1 シーンの確定済み行範囲・入れ子レベル・識別子・親を投入する。
    ///
    /// `start_line` / `end_line` は両端 inclusive・1 始まり。`end_line` は「次の同
    /// レベル以上宣言の直前 / ファイル末尾」を構築側が算出済みで渡す（requirements
    /// 2.1）。`level` は入れ子の深さ（global = 0, local = 1, ...）。`parent` は local の
    /// 親グローバル（例 `会話1`）で、global は `None`（task 2.2・kick の local 分岐
    /// 解決に渡す）。同一ファイル内で同一 `start_line` を再投入した場合は last-write-wins
    /// で上書きする。
    pub fn add_scene(
        &mut self,
        pasta_file: &str,
        scene_id: &str,
        parent: Option<&str>,
        start_line: u32,
        end_line: u32,
        level: u32,
    ) {
        let file_key = canonicalize_pasta_file(pasta_file);
        self.files.entry(file_key).or_default().insert(
            start_line,
            SceneSpan {
                start_line,
                end_line,
                level,
                scene_id: scene_id.to_string(),
                parent: parent.map(|p| p.to_string()),
            },
        );
    }

    /// 蓄積した内容から不変な [`SceneIdentityIndex`] を確定する。
    pub fn finish(self) -> SceneIdentityIndex {
        SceneIdentityIndex { files: self.files }
    }
}

#[cfg(test)]
#[path = "scene_index_tests.rs"]
mod tests;
