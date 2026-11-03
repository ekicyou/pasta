//! シーンマップのシンクを付けても生成 `.lua` が変わらないこと（3.7 の守り）。
//!
//! ローダの Phase 5.5 がシーンの記録を控えることの確かめは、`scene_records` が crate の
//! 中だけの口なので `loader/source_map_build.rs` の単体テストに置く。

use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;

/// 代表 `.pasta` A（シーン 1 本・トーク複数行）。
const PASTA_A: &str = "\
＊あいさつ
  さくら：「おはよう！」
  さくら：「げんき？」
";

/// 代表 `.pasta` B（別のシーン）。
const PASTA_B: &str = "\
＊おやすみ
  さくら：「おやすみなさい。」
";

/// 7.1: デバッグ無効＝バイト不変。記録シンクを装着した構築経路の生成 `.lua` バイトは、
/// シンク無し（本番ローダの `transpile`）の生成 `.lua` バイトと完全一致する。
///
/// これは「シンク装着が出力を 1 バイトも変えない」ことの直接検証であり、無効時に
/// ローダがシンクを装着しなければ生成 `.lua` は従来と完全一致する（7.1）ことの根拠。
#[test]
fn sink_attachment_is_byte_invariant_against_sinkless_transpile() {
    for src in [PASTA_A, PASTA_B] {
        let pasta_file = parse_str(src, "dic/x.pasta").expect("parse");
        let transpiler = LuaTranspiler::default();

        // シンク無し（本番ローダの transpile＝debug 無効経路）。
        let mut out_sinkless = Vec::new();
        transpiler
            .transpile(&pasta_file, &mut out_sinkless)
            .expect("transpile sinkless");

        // シンク有り（debug 有効の構築経路）。
        let mut sink = pasta_lua::debug::source_map::MapBuilderSink::default();
        let mut out_with_sink = Vec::new();
        transpiler
            .transpile_with_source_map(&pasta_file, &mut out_with_sink, Some(&mut sink))
            .expect("transpile with sink");

        assert_eq!(
            out_sinkless, out_with_sink,
            "7.1: シンク装着は生成 `.lua` を 1 バイトも変えない（無効時バイト不変の根拠）"
        );
    }
}
