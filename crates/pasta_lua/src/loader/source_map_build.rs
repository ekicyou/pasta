//! ローダの Phase 5.5（シーンマップを作る）。`loader/mod.rs` から分けた兄弟モジュール。

use super::PastaLoader;
use crate::debug::source_map::{MapBuilderSink, SourceMap};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::sync::Arc;
use tracing::{debug, warn};

impl PastaLoader {
    /// 見つけた `.pasta` ごとにシーンの宣言の記録を控え、シーンマップを作る（Phase 5.5）。
    ///
    /// ローダはデバッグが有効なときだけ呼ぶ。無効なら呼ばないので、シンクも付かず
    /// [`SourceMap`] も作らない（Phase 4 の生成 `.lua` はシンク無しの
    /// [`LuaTranspiler::transpile`] のまま）。
    ///
    /// `.pasta` ごとに読み直して解析し、[`MapBuilderSink`] を付けて変換し直し（出力は
    /// 捨てる）、控えた記録を [`SourceMap::insert_scenes`] に入れる。宣言の行は `.pasta`
    /// の行なので、`LineShift` の付け替えは要らない。読めない・解析できない・変換できない
    /// ファイルは警告して飛ばす（本当の失敗は Phase 4 の変換が出す）。
    ///
    /// `transpiler` は生成 `.lua` と同じもの（照合の鍵が同じ別名の表を使うため）。
    pub fn build_source_map(
        pasta_files: &[std::path::PathBuf],
        transpiler: &LuaTranspiler,
    ) -> Arc<SourceMap> {
        let mut source_map = SourceMap::new();
        for file_path in pasta_files {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!(file = %file_path.display(), error = %e, "source-map: skip (read error)");
                    continue;
                }
            };

            let filename = file_path.to_string_lossy().to_string();
            let pasta_file = match pasta_dsl::parse_str(&content, &filename) {
                Ok(pf) => pf,
                Err(e) => {
                    warn!(file = %file_path.display(), error = %e, "source-map: skip (parse error)");
                    continue;
                }
            };

            let mut sink = MapBuilderSink::default();
            if let Err(e) =
                transpiler.transpile_with_source_map(&pasta_file, &mut Vec::new(), Some(&mut sink))
            {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (transpile error)");
                continue;
            }
            source_map.insert_scenes(&filename, sink.into_scenes());
        }

        debug!(files = pasta_files.len(), "source-map: built scene map");
        Arc::new(source_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_pasta(base: &std::path::Path, rel: &str, src: &str) -> std::path::PathBuf {
        let path = base.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, src).unwrap();
        path
    }

    /// 3.5: 2 つの `.pasta` を通すと、ファイルごとにシーンの宣言の記録が入る。
    #[test]
    fn loader_builds_scene_records_per_pasta_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let a = write_pasta(
            temp.path(),
            "dic/a.pasta",
            "＊あいさつ\n  さくら：「おはよう！」\n  ・朝\n    さくら：「げんき？」\n",
        );
        let b = write_pasta(
            temp.path(),
            "dic/b.pasta",
            "\n＊おやすみ\n  さくら：「おやすみ。」\n",
        );

        let map = PastaLoader::build_source_map(&[a, b], &LuaTranspiler::default());

        let records = map.scene_records();
        assert_eq!(records.len(), 2, "{records:?}");
        // 鍵は正規化した `.pasta` のパス（区切りは `/`）。値は宣言順の (照合の鍵, 宣言の行)。
        let scenes = |name: &str| {
            let (_, scenes) = records.iter().find(|(k, _)| k.ends_with(name)).unwrap();
            scenes
                .iter()
                .map(|(k, l)| (k.as_str(), *l))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            scenes("/a.pasta"),
            [
                ("G:あいさつ#1", 1),
                ("L:あいさつ#1:__start__", 2),
                ("L:あいさつ#1:朝_1", 3)
            ],
            "{records:?}"
        );
        assert_eq!(
            scenes("/b.pasta"),
            [("G:おやすみ#1", 2), ("L:おやすみ#1:__start__", 3)],
            "{records:?}"
        );
    }

    /// 何も渡さなければシーンの記録は空。
    #[test]
    fn no_scene_records_when_no_files_passed() {
        let map = PastaLoader::build_source_map(&[], &LuaTranspiler::default());
        assert!(map.scene_records().is_empty());
    }
}
