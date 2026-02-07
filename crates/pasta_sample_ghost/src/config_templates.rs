//! surfaces.txt 生成
//!
//! シェルの surfaces.txt を生成します。
//! 画像IDとの整合性を保証するため、コード生成を維持します。

/// surfaces.txt を生成
pub fn generate_surfaces_txt() -> String {
    let mut content = String::from("charset,UTF-8\n\n");

    // sakura サーフェス (0-8)
    for i in 0..=8 {
        content.push_str(&format!(
            r#"surface{i}
{{
element0,overlay,surface{i}.png,0,0
}}

"#
        ));
    }

    // kero サーフェス (10-18)
    for i in 10..=18 {
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
}
