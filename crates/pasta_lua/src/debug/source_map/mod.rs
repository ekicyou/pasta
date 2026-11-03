//! シーンマップ（`OnPastaSceneAt` の位置からシーンを引くための、シーンの宣言の記録と索引）。
//!
//! 有効なときだけローダが作る。`.pasta` ごとの `(照合の鍵, 宣言の行)` の記録を
//! [`SourceMap::insert_scenes`] で入れ、ランタイムの構築時に突合した索引を
//! [`SourceMap::scene_at`] で引く。

use std::collections::HashMap;
use std::sync::OnceLock;

pub use crate::code_gen::source_map::SourceMapSink;

/// シーンの宣言だけを控えるシンク。行の記録は捨てる。
///
/// code_gen が宣言順（global ヘッダ → その local 群）に呼ぶ
/// [`record_scene`](SourceMapSink::record_scene) の `(照合の鍵, .pasta の宣言の行)` を
/// そのまま積む。宣言の行は `.pasta` の行のまま使う。
#[derive(Default)]
pub struct MapBuilderSink {
    scenes: Vec<(String, u32)>,
}

impl MapBuilderSink {
    /// 控えたシーンの記録を宣言順で返す。
    pub fn into_scenes(self) -> Vec<(String, u32)> {
        self.scenes
    }
}

impl SourceMapSink for MapBuilderSink {
    fn record_scene(&mut self, scene_join_key: &str, span: pasta_dsl::parser::Span) {
        self.scenes
            .push((scene_join_key.to_string(), span.start_line as u32));
    }

    // 行の記録は使わない。
    fn record_line(&mut self, _lua_line: u32, _pasta_line: u32) {}
}

/// `.pasta` のパスの照合の正規化。`@` を除き、`\` を `/` にそろえ、Windows では小文字にする
/// （記録・索引・問い合わせの全部がこれを通る）。
fn canonicalize_pasta_file(raw: &str) -> String {
    let unified = raw.strip_prefix('@').unwrap_or(raw).replace('\\', "/");
    // Windows のファイルシステムは大小を区別しない。
    #[cfg(windows)]
    {
        unified.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        unified
    }
}

/// シーンマップ。シーンの宣言の記録と、位置からシーンを引く索引だけを持つ。
#[derive(Debug, Default)]
pub struct SourceMap {
    /// 正規化した `.pasta` のパス → `(照合の鍵, 宣言の行)` の宣言順の列。
    scene_records: HashMap<String, Vec<(String, u32)>>,
    /// 突合の後に 1 度だけ書く索引（`Arc<SourceMap>` の共有参照から書くため `OnceLock`）。
    scene_index: OnceLock<SceneIdentityIndex>,
}

impl SourceMap {
    /// 空のシーンマップを作る。
    pub fn new() -> Self {
        Self::default()
    }

    /// 1 つの `.pasta` のシーンの記録を入れる（同じファイルは置き換え）。
    pub fn insert_scenes(&mut self, pasta_file: &str, scenes: Vec<(String, u32)>) {
        self.scene_records
            .insert(canonicalize_pasta_file(pasta_file), scenes);
    }

    /// 正規化した `.pasta` のパス → `(照合の鍵, 宣言の行)` の宣言順の列（突合の素材）。
    pub(crate) fn scene_records(&self) -> &HashMap<String, Vec<(String, u32)>> {
        &self.scene_records
    }

    /// 位置 (`pasta_file`, `line`) のシーンを索引で引く。突合の前・突合できなかったときは `None`。
    pub fn scene_at(&self, pasta_file: &str, line: u32) -> Option<SceneIdentity> {
        self.scene_index.get()?.scene_at(pasta_file, line)
    }

    /// 突合した索引を 1 度だけ書く。書いてあれば `Err(index)` を返す。
    pub(crate) fn set_scene_index(
        &self,
        index: SceneIdentityIndex,
    ) -> Result<(), SceneIdentityIndex> {
        self.scene_index.set(index)
    }
}

/// 位置 → シーンの索引。
mod scene_index;
pub use scene_index::SceneIdentity;
pub(crate) use scene_index::SceneIdentityIndex;

/// 記録と実行時のシーン表の突合。
mod scene_join;
pub(crate) use scene_join::build_scene_index;
