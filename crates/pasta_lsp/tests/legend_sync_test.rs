//! 解析器と拡張の凡例の並びが食い違ったら落とすテスト（要件 7.1）
//!
//! 拡張の `semanticTokensProvider.ts` の `PASTA_TOKEN_TYPES`・`PASTA_TOKEN_MODIFIERS` を読み、
//! 解析器の `TOKEN_TYPES`・`TOKEN_MODIFIERS` と数・順・文字列を比べる。

use pasta_lsp::analysis::{TOKEN_MODIFIERS, TOKEN_TYPES};

fn provider_source() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../editors/vscode/src/semanticTokensProvider.ts"
    );
    // ファイルが無いときは黙って通さずに落とす
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path} を読めない: {e}"))
}

/// `export const {name}` の配列 `= [ … ]` の中の引用の文字列を順に取り出す（行末の `//` コメントは読まない）
fn ts_array(source: &str, name: &str) -> Vec<String> {
    let decl = format!("export const {name}");
    let start = source
        .find(&decl)
        .unwrap_or_else(|| panic!("{decl} が見つからない"));
    let body = &source[start..];
    let open = body.find("= [").expect("配列の始まり `= [` が無い") + 3;
    let close = open + body[open..].find(']').expect("配列の終わり `]` が無い");
    body[open..close]
        .lines()
        .flat_map(|line| {
            let code = line.split("//").next().unwrap_or("");
            // 引用で割ると奇数番目が引用の中身
            code.split(['\'', '"'])
                .skip(1)
                .step_by(2)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn token_types_match_extension() {
    assert_eq!(
        ts_array(&provider_source(), "PASTA_TOKEN_TYPES"),
        TOKEN_TYPES,
        "解析器の TOKEN_TYPES と拡張の PASTA_TOKEN_TYPES が食い違っている"
    );
}

#[test]
fn token_modifiers_match_extension() {
    assert_eq!(
        ts_array(&provider_source(), "PASTA_TOKEN_MODIFIERS"),
        TOKEN_MODIFIERS,
        "解析器の TOKEN_MODIFIERS と拡張の PASTA_TOKEN_MODIFIERS が食い違っている"
    );
}
