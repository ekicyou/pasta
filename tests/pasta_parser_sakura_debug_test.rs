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
    println!("Parsed {} scenes", file.scenes.len());

    for scene in &file.scenes {
        println!("Scene: {}", scene.name);
        for stmt in &scene.statements {
            println!("  Statement: {:?}", stmt);
        }
    }
}
