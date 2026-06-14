//! `source_map` の **任意ディスクサイドカー** I/O（task 6.1・3.2・design "SidecarWriter"
//! 478-485 / Data Contracts 601-602）。
//!
//! メモリ内 [`ChunkSourceMap`](super::ChunkSourceMap) の前方写像を最小・独自スキーマの
//! JSON（[`SidecarFile`]）へ落とし、生成 `.lua` の隣（`<lua_path>.map`）へ
//! 出力・読み戻しする（[`write_sidecar`]/[`read_sidecar`]）。本番可視性は不変で、
//! ハブ [`super`] が `pub use sidecar::*` で再エクスポートするため、外部の
//! `crate::debug::source_map::{SidecarFile, write_sidecar, read_sidecar, ...}` 経路は
//! 変わらない。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{ChunkSourceMap, PastaPos, map_forward_iter};

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
