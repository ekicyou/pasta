//! シーン同一性索引（[`SceneIdentityIndex`]）— `.pasta` の (ファイル, 行) から
//! 所属シーンを引く逆引き索引。
//!
//! # 行範囲
//!
//! シーンの行範囲は「宣言行 〜 次の同レベル以上のシーン宣言の直前（またはファイル末尾）」。
//! 本索引は構築側が算出した `[start_line, end_line]`（両端 **inclusive**・1 始まり）を
//! そのまま保持する。
//!
//! # 解決ポリシー
//!
//! 1. **包含**: クリック行を範囲に含むシーンのうち **最内**（入れ子レベル最大 = local 優先）。
//!    包含を後方フォールバックに優先する。
//! 2. **後方フォールバック**: 包含が無ければ、クリック行と同じか **下方**（より大きい
//!    行番号）の最近接の宣言。
//! 3. **未検出**: 下方にも無ければ未検出。空ファイル・未登録ファイルも未検出。
//!
//! # 計算量
//!
//! どちらの探索も `BTreeMap` の `range` で対数オーダに収め、SHIORI GET の処理を延ばさない。

use std::collections::{BTreeMap, HashMap};

use super::canonicalize_pasta_file;

/// 確定したシーン同一性（`scene_at` の返り値）。
///
/// global は `scene_id=会話_1, parent=None`、local は `scene_id=挨拶_1, parent=Some(会話_1)`。
/// `OnPastaSceneAt` の再生は `parent` の有無で global / local を選ぶ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneIdentity {
    /// シーン識別子（global = `会話_1`、local = `挨拶_1`）。
    pub scene_id: String,
    /// 親グローバルシーン（local のみ `Some(会話_1)`・global は `None`）。
    pub parent: Option<String>,
}

/// 1 シーンの行範囲（両端 inclusive・1 始まり）・入れ子レベル・識別子。
///
/// `scene_id` は `SceneRegistry` 由来のランタイムの識別子（例 `会話_1`）を **そのまま**
/// 保持する。索引は識別子の書式に依存しない。
#[derive(Debug, Clone)]
struct SceneSpan {
    /// `BTreeMap` のキーと一致する。
    start_line: u32,
    end_line: u32,
    /// 入れ子の深さ（global = 0, local = 1, ...）。最内優先の判定に用いる。
    level: u32,
    scene_id: String,
    /// 親グローバルシーン（local のみ `Some`）。
    parent: Option<String>,
}

impl SceneSpan {
    fn contains(&self, line: u32) -> bool {
        self.start_line <= line && line <= self.end_line
    }

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
/// でファイル引き、内側 `BTreeMap` で開始行の昇順。ファイルキーは
/// [`canonicalize_pasta_file`] で登録と問い合わせを同じ正規形へ落とす。
#[derive(Debug, Clone, Default)]
pub struct SceneIdentityIndex {
    files: HashMap<String, BTreeMap<u32, SceneSpan>>,
}

impl SceneIdentityIndex {
    /// 索引を組み立てる [`SceneIdentityIndexBuilder`] を返す。
    pub fn builder() -> SceneIdentityIndexBuilder {
        SceneIdentityIndexBuilder::default()
    }

    /// 位置 (`pasta_file`, `line`) からシーン識別子を確定する。`line` は 1 始まり。
    ///
    /// 解決順:
    /// 1. **包含**: `line` を範囲に含むシーン群から最内（`level` 最大）を選ぶ。
    /// 2. **後方フォールバック**: 包含が無ければ `line` と同じか下方の最近接宣言。
    /// 3. **未検出**: 下方にも無ければ `None`。未登録ファイル・空ファイルも `None`。
    pub fn scene_at(&self, pasta_file: &str, line: u32) -> Option<SceneIdentity> {
        let file_key = canonicalize_pasta_file(pasta_file);
        let scenes = self.files.get(&file_key)?;

        // 開始行が `line` を超えるシーンは `line` を含み得ないので `range(..=line)` で枝刈りする。
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

        if let Some((_, span)) = scenes.range(line..).next() {
            return Some(span.identity());
        }

        None
    }
}

/// [`SceneIdentityIndex`] のビルダ。
///
/// ファイルごとにシーン範囲を蓄積し、[`finish`](Self::finish) で不変な索引へ確定する。
/// 同一ファイル・同一開始行への再投入は last-write-wins（`BTreeMap` キー一意性）。
#[derive(Debug, Default)]
pub struct SceneIdentityIndexBuilder {
    files: HashMap<String, BTreeMap<u32, SceneSpan>>,
}

impl SceneIdentityIndexBuilder {
    /// 1 シーンの確定済み行範囲・入れ子レベル・識別子・親を投入する。
    ///
    /// `start_line` / `end_line` は両端 inclusive・1 始まり。`level` は入れ子の深さ
    /// （global = 0, local = 1, ...）。`parent` は local の親グローバル（例 `会話_1`）で、
    /// global は `None`。
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
