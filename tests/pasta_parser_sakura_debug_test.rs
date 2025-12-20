//! Debug sakura script parser

use pasta::parser::parse_str;

#[test]
fn test_parse_sakura() {
    let source = r#"＊test
  さくら：こんにちは\s[0]
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let file = result.unwrap();
    println!("Parsed {} labels", file.scenes.len());

    for label in &file.scenes {
        println!("Label: {}", label.name);
        for stmt in &label.statements {
            println!("  Statement: {:?}", stmt);
        }
    }
}
