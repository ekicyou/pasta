//! Pest 2.8 Span API 検証テスト
//!
//! `Span::start()`, `Span::end()` メソッドが利用可能か確認

use pasta_rune::parser::{PastaParser2, Rule};
use pest::Parser;

#[test]
fn verify_pest_span_byte_offset_api() {
    // 簡単なテストケース
    let source = "＊テスト\n  Alice：こんにちは\n";

    // パース実行
    let pairs = PastaParser2::parse(Rule::file, source).expect("Parse failed");

    // 最初の pair を取得
    let first_pair = pairs.into_iter().next().expect("No pairs found");

    // Pest の Span を取得
    let pest_span = first_pair.as_span();

    // ★ 検証対象：start(), end() メソッドが存在するか
    let start_byte = pest_span.start();
    let end_byte = pest_span.end();

    // 検証
    println!("Pest Span API verification:");
    println!("  start_byte: {}", start_byte);
    println!("  end_byte: {}", end_byte);
    println!("  source length: {}", source.len());

    // 基本的な妥当性チェック
    assert!(
        start_byte < end_byte,
        "start_byte should be less than end_byte"
    );
    assert!(
        end_byte <= source.len(),
        "end_byte should not exceed source length"
    );

    // バイトオフセットでソースを抽出できるか確認
    let extracted = &source[start_byte..end_byte];
    println!("  extracted text: {:?}", extracted);

    // 成功メッセージ
    println!("✅ Pest 2.8 Span::start() and Span::end() are available!");
}
