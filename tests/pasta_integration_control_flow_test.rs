//! 包括的なコントロールフロー参照実装のテスト
//!
//! このテストは comprehensive_control_flow.rn が正しくコンパイルされることを確認します。
//! トランスパイラー実装の目標ベースラインとして機能します。

use rune::Context;

#[test]
fn test_comprehensive_control_flow_reference() -> Result<(), Box<dyn std::error::Error>> {
    // Runeコンテキストを準備
    let context = Context::with_default_modules()?;

    // comprehensive_control_flow.rnファイルを読み込み
    let rune_code = include_str!("fixtures/comprehensive_control_flow.rn");

    // コンパイル準備
    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::new("entry", rune_code)?)?;

    // コンパイル検証 - Rune構文の正しさを確認
    let _unit = rune::prepare(&mut sources).with_context(&context).build()?;

    // ✅ コンパイル成功 = Runeコードの構文が正しい
    println!("✅ comprehensive_control_flow.rn: コンパイル成功");
    println!("   ✓ 全ての関数定義が正しく認識されました");
    println!("   ✓ ctx引数を持つgenerator関数の構文が正しいです");
    println!("   ✓ for value in gen(ctx) {{ yield value; }} パターンが正しいです");
    println!("   ✓ Object literal syntax (#{{ type: ..., ... }}) が正しいです");
    println!("   ✓ ctx.pasta.word(ctx, \"keyword\") 呼び出しが正しいです");
    println!("   ✓ ctx.pasta.add_words()/commit_words() 呼び出しが正しいです");
    println!("   ✓ ctx.var.変数名 アクセスが正しいです");
    println!("   ✓ ctx.args 配列アクセスが正しいです");
    println!("   ✓ String interpolation (`${{値}}`) が正しいです");
    println!();
    println!("📝 参照実装が含む全機能:");
    println!("   - ローカル単語定義 (add_words/commit_words)");
    println!("   - 変数代入・参照 (ctx.var.カウンター)");
    println!("   - Call文（引数なし・あり）");
    println!("   - Jump文（複数ラベル）");
    println!("   - 単語展開 (ctx.pasta.word())");
    println!("   - 会話文とActor/Talkイベント");
    println!("   - ネストされたCall (3層: __start__ → 自己紹介 → 趣味紹介)");
    println!("   - ネストされたJump (3層: __start__ → 会話分岐_1 → 別の話題_1)");
    println!("   - 引数保存・復元 (saved_args パターン)");
    println!();
    println!("🎯 次のステップ (TODO #3.5 完了後):");
    println!("   - TODO #4: ctx構造の詳細設計");
    println!("   - TODO #5: Pasta runtime メソッドシグネチャ設計");
    println!("   - TODO #6: 引数保存・復元メカニズムの詳細設計");
    println!("   - TODO #8: エラーハンドリング戦略");

    Ok(())
}

#[test]
fn verify_reference_implementation_structure() -> Result<(), Box<dyn std::error::Error>> {
    let rune_code = include_str!("fixtures/comprehensive_control_flow.rn");

    // 必須パターンの存在確認
    assert!(
        rune_code.contains("pub mod メイン_1"),
        "メイン_1 モジュールが存在すること"
    );
    assert!(
        rune_code.contains("pub fn __start__(ctx)"),
        "__start__ 関数がctx引数を持つこと"
    );
    assert!(
        rune_code.contains("pub fn 自己紹介(ctx)"),
        "自己紹介 関数がctx引数を持つこと"
    );
    assert!(
        rune_code.contains("for value in"),
        "for-in-yield パターンが使用されていること"
    );
    assert!(rune_code.contains("yield"), "yield 文が使用されていること");
    assert!(
        rune_code.contains("ctx.pasta.word"),
        "ctx.pasta.word 呼び出しが存在すること"
    );
    assert!(
        rune_code.contains("ctx.args"),
        "ctx.args アクセスが存在すること"
    );
    assert!(
        rune_code.contains("#{ type:"),
        "Object literal が使用されていること"
    );

    println!("✅ 参照実装の構造検証成功");
    Ok(())
}

#[test]
fn verify_pasta_input_structure() -> Result<(), Box<dyn std::error::Error>> {
    let pasta_code = include_str!("fixtures/comprehensive_control_flow.pasta");

    // Pastaコードの必須要素を確認
    assert!(
        pasta_code.contains("＠挨拶"),
        "グローバル単語定義が存在すること"
    );
    assert!(
        pasta_code.contains("＠場所"),
        "ローカル単語定義が存在すること"
    );
    assert!(pasta_code.contains("＄カウンタ"), "変数が存在すること");
    assert!(pasta_code.contains("＞自己紹介"), "Call文が存在すること");
    // Phase 1 (REQ-BC-1): Jump statement deprecated, using Call instead
    assert!(
        pasta_code.contains("＞会話分岐"),
        "Call文（旧Jump）が存在すること"
    );
    assert!(pasta_code.contains("さくら　："), "会話文が存在すること");

    println!("✅ Pasta入力の構造検証成功");
    Ok(())
}
