//! surfaces.txt 生成
//!
//! シェルの surfaces.txt を生成します。
//! 画像IDとの整合性を保証するため、コード生成を維持します。

/// surfaces.txt を生成
pub fn generate_surfaces_txt() -> String {
    let mut content = String::from("charset,UTF-8\n\n");

    // sakura サーフェス (0-8) / kero サーフェス (10-18)
    for i in (0..=8).chain(10..=18) {
        content.push_str(&format!(
            r#"surface{i}
{{
element0,overlay,surface{i}.png,0,0
}}

"#
        ));
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surfaces_txt() {
        let content = generate_surfaces_txt();
        assert!(content.contains("surface0"));
        assert!(content.contains("surface8"));
        assert!(content.contains("surface10"));
        assert!(content.contains("surface18"));
    }

    #[test]
    fn surfaces_txt_starts_with_charset_header() {
        let content = generate_surfaces_txt();
        assert!(
            content.starts_with("charset,UTF-8\n"),
            "charset ヘッダで始まっていません"
        );
    }

    #[test]
    fn surfaces_txt_defines_exactly_18_blocks_with_matching_png_ids() {
        let content = generate_surfaces_txt();

        // 画像IDとの整合（モジュールの存在目的）: sakura 0-8 / kero 10-18 の
        // 各サーフェスが、同じ ID の png を参照する定義ブロックを持つこと
        for id in (0..=8).chain(10..=18) {
            let block = format!("surface{id}\n{{\nelement0,overlay,surface{id}.png,0,0\n}}");
            assert!(
                content.contains(&block),
                "surface{id} の定義ブロックが期待書式と一致しません"
            );
        }

        // 定義ブロックはちょうど 18 個（余剰定義がない）
        let block_count = content.matches("element0,overlay,").count();
        assert_eq!(block_count, 18, "定義ブロック数が18ではありません");

        // 欠番 ID（surface9 / surface19）が定義されていないこと
        assert!(
            !content.contains("surface9\n"),
            "欠番 surface9 が定義されています"
        );
        assert!(
            !content.contains("surface19"),
            "範囲外 surface19 が定義されています"
        );
    }
}
