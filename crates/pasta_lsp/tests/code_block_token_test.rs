//! CodeBlock トークンのフェンス行限定化と無回帰を固定するテスト (Task 3.2)
//!
//! Lua コードブロックのセマンティックトークン `codeBlock`（CODE_BLOCK, index 9）が、
//! 開始フェンス行と「実際の」終了フェンス行の 2 行のみに出力され、本文行には
//! 一切出力されないことを検証する。あわせて、`codeBlock` 以外のトークン列が
//! Lua ブロックの有無に左右されないこと（無回帰）と、凡例（トークン種別の
//! 並び順・`CODE_BLOCK` の凡例位置）の不変性をガードする。
//!
//! 重要: parser の `CodeBlock.span.end_line` は終端フェンスの「1 行先」を指す
//! ため、終了フェンス行は `end_line` ではなく、テスト側で構築したソース文字列を
//! 数えて求めた「実際の ``` 行」を期待値とする。

use pasta_lsp::analysis::{token_type, AnalysisEngine, TOKEN_TYPES};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

/// delta エンコード済みトークン列を絶対位置 (line, start_char) へ復号する。
///
/// LSP のセマンティックトークンは直前トークンからの差分で表現されるため、
/// 行・開始列を累積して絶対位置に戻す（同一行内なら start_char も加算する）。
fn decode_absolute(tokens: &[SemanticToken]) -> Vec<AbsToken> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut line = 0u32;
    let mut start = 0u32;
    for t in tokens {
        if t.delta_line == 0 {
            start += t.delta_start;
        } else {
            line += t.delta_line;
            start = t.delta_start;
        }
        out.push(AbsToken {
            line,
            start_char: start,
            length: t.length,
            token_type: t.token_type,
        });
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AbsToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
}

/// 指定種別のトークンが乗っている絶対行番号（0-based）の昇順リストを返す。
fn lines_with_type(tokens: &[AbsToken], ty: u32) -> Vec<u32> {
    let mut v: Vec<u32> = tokens
        .iter()
        .filter(|t| t.token_type == ty)
        .map(|t| t.line)
        .collect();
    v.sort_unstable();
    v
}

// ============================================================================
// 観点1: 開始/終了フェンス行に codeBlock が出力される
// ============================================================================

#[test]
fn test_codeblock_on_open_and_close_fence_single_body_line() {
    // 行構成 (0-based):
    //   0: ＊ｓ
    //   1: ```lua        ← 開始フェンス
    //   2: return 1      ← 本文（codeBlock 無し）
    //   3: ```           ← 終了フェンス（実フェンス行）
    //   4: （末尾空行）
    let source = "＊ｓ\n```lua\nreturn 1\n```\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        result.diagnostics.is_empty(),
        "正常な Lua ブロックは診断を生成しない: {:?}",
        result.diagnostics
    );

    let abs = decode_absolute(&result.tokens);
    let cb_lines = lines_with_type(&abs, token_type::CODE_BLOCK);

    assert_eq!(
        cb_lines,
        vec![1, 3],
        "codeBlock は開始フェンス行(1)と実終了フェンス行(3)のみに出力される: got {:?}",
        cb_lines
    );
}

// ============================================================================
// 観点2: 本文行（単数/複数）に codeBlock が無く、フェンス2行のみに出力される
// ============================================================================

#[test]
fn test_codeblock_not_on_single_body_line() {
    // 本文1行ブロック。本文の行(2)には codeBlock が乗ってはならない。
    let source = "＊ｓ\n```lua\nreturn 1\n```\n";
    let abs = decode_absolute(&AnalysisEngine::analyze(source).tokens);

    let cb_lines = lines_with_type(&abs, token_type::CODE_BLOCK);

    // フェンス2行ちょうど。
    assert_eq!(
        cb_lines.len(),
        2,
        "codeBlock はフェンス2行のみ: got {:?}",
        cb_lines
    );
    // 本文行(2)には無い。
    assert!(
        !cb_lines.contains(&2),
        "本文行(2)に codeBlock があってはならない: got {:?}",
        cb_lines
    );
    // 末尾空行(4)にも無い（end_line off-by-one の誤検出ガード）。
    assert!(
        !cb_lines.contains(&4),
        "末尾空行(4)に codeBlock があってはならない: got {:?}",
        cb_lines
    );
}

#[test]
fn test_codeblock_not_on_multiple_body_lines() {
    // 行構成 (0-based):
    //   0: ＊ｓ
    //   1: ```lua          ← 開始フェンス
    //   2: local a = 1     ← 本文1
    //   3: return a        ← 本文2
    //   4: ```             ← 終了フェンス（実フェンス行）
    //   5: （末尾空行）
    let source = "＊ｓ\n```lua\nlocal a = 1\nreturn a\n```\n";
    let abs = decode_absolute(&AnalysisEngine::analyze(source).tokens);

    let cb_lines = lines_with_type(&abs, token_type::CODE_BLOCK);

    assert_eq!(
        cb_lines,
        vec![1, 4],
        "複数本文ブロックでも codeBlock は開始(1)/実終了(4)フェンスのみ: got {:?}",
        cb_lines
    );
    // すべての本文行に codeBlock が無いことを明示。
    for body_line in [2u32, 3u32] {
        assert!(
            !cb_lines.contains(&body_line),
            "本文行({})に codeBlock があってはならない: got {:?}",
            body_line,
            cb_lines
        );
    }
}

// ============================================================================
// 観点3: 混在文書で codeBlock 以外のトークン列が無回帰（Lua ブロック有無で不変）
// ============================================================================

#[test]
fn test_non_codeblock_tokens_unaffected_by_lua_block() {
    // シーン（グローバル）・アクター辞書・単語定義・アクション行を含み、
    // 末尾に Lua ブロックを置いた文書と、同文書から Lua ブロックのみを
    // 取り除いた文書とで、codeBlock 以外のトークン列が完全一致することを固定する。
    //
    // これにより「フェンス行限定化が pasta 固有トークン（シーン/アクター/単語）に
    // 一切干渉しない」ことを保証する。
    // グローバルシーン（NAMESPACE）＋アクション行（actorName/talk）＋
    // アクター辞書（ACTOR）＋変数代入内の単語参照（WORD）を含む、
    // 診断ゼロでパースできる文書。末尾に Lua ブロックを足し引きして比較する。
    let common = "\
＊挨拶
　Alice：こんにちは
％さくら
　＄好感度＝＠乱数
";
    let lua_block = "```lua\nreturn 1\n```\n";

    let with_lua = format!("{common}{lua_block}");
    let without_lua = common.to_string();

    let with_abs = decode_absolute(&AnalysisEngine::analyze(&with_lua).tokens);
    let without_abs = decode_absolute(&AnalysisEngine::analyze(&without_lua).tokens);

    // Lua ブロック付き文書には codeBlock が存在する（前提の健全性チェック）。
    let with_cb = lines_with_type(&with_abs, token_type::CODE_BLOCK);
    assert_eq!(
        with_cb.len(),
        2,
        "混在文書では codeBlock がフェンス2行に出る: got {:?}",
        with_cb
    );

    // codeBlock を除いた非 codeBlock トークン列を比較する。
    // Lua ブロックは common の末尾に追記しているだけなので、common 由来の
    // 非 codeBlock トークンの (line, start_char, length, token_type) は不変のはず。
    let with_non_cb: Vec<AbsToken> = with_abs
        .into_iter()
        .filter(|t| t.token_type != token_type::CODE_BLOCK)
        .collect();
    let without_non_cb: Vec<AbsToken> = without_abs
        .into_iter()
        .filter(|t| t.token_type != token_type::CODE_BLOCK)
        .collect();

    assert_eq!(
        with_non_cb, without_non_cb,
        "codeBlock 以外のトークン列は Lua ブロックの有無に影響されない"
    );

    // 念のため、pasta 固有の主要トークンが実際に含まれていることを確認し、
    // 「両者とも空で一致」という退化ケースで通っていないことを排除する。
    let has = |ty: u32| with_non_cb.iter().any(|t| t.token_type == ty);
    assert!(has(token_type::NAMESPACE), "グローバルシーンマーカーが含まれる");
    assert!(has(token_type::ACTOR), "アクター辞書マーカーが含まれる");
    assert!(has(token_type::WORD), "単語マーカーが含まれる");
}

// ============================================================================
// 観点4: 凡例不変ガード（種別の並び順と CODE_BLOCK の凡例位置 index 9）
// ============================================================================

#[test]
fn test_legend_codeblock_index_is_nine() {
    assert_eq!(
        token_type::CODE_BLOCK,
        9,
        "CODE_BLOCK の凡例位置は index 9 で不変"
    );
    assert_eq!(
        TOKEN_TYPES[token_type::CODE_BLOCK as usize],
        SemanticTokenType::new("codeBlock"),
        "凡例 index 9 は codeBlock 種別である"
    );
}

#[test]
fn test_legend_order_is_stable() {
    // トークン種別の並び順が変わると、エディタ側の凡例マッピングが破綻し、
    // 全トークンの色が一斉にずれる。並び順全体を文字列表現で固定する。
    let expected: &[SemanticTokenType] = &[
        SemanticTokenType::COMMENT,             // 0
        SemanticTokenType::NAMESPACE,           // 1
        SemanticTokenType::new("scene"),        // 2
        SemanticTokenType::DECORATOR,           // 3
        SemanticTokenType::new("word"),         // 4
        SemanticTokenType::VARIABLE,            // 5
        SemanticTokenType::new("call"),         // 6
        SemanticTokenType::new("actor"),        // 7
        SemanticTokenType::new("actorName"),    // 8
        SemanticTokenType::new("codeBlock"),    // 9
        SemanticTokenType::new("talk"),         // 10
        SemanticTokenType::new("sakuraScript"), // 11
        SemanticTokenType::new("escape"),       // 12
        SemanticTokenType::OPERATOR,            // 13
        SemanticTokenType::NUMBER,              // 14
        SemanticTokenType::new("cueMarker"),    // 15
        SemanticTokenType::new("cueCommand"),   // 16
    ];

    assert_eq!(
        TOKEN_TYPES.len(),
        expected.len(),
        "トークン種別数が不変"
    );
    assert_eq!(
        TOKEN_TYPES, expected,
        "トークン種別の並び順が不変（凡例 index の安定性）"
    );

    // 周辺 index の隣接関係も明示的に固定（CODE_BLOCK の前後）。
    assert_eq!(token_type::ACTOR_NAME, 8, "codeBlock の直前は actorName(8)");
    assert_eq!(token_type::TALK, 10, "codeBlock の直後は talk(10)");
}
