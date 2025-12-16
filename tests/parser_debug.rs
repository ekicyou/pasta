//! Simple parser debugging

use pasta::parse_str;

#[test]
fn test_minimal() {
    let source = "*test\n";
    let result = parse_str(source, "test.pasta");
    println!("Result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_with_speech_halfwidth() {
    let source = "*test\n  a:hello\n";
    let result = parse_str(source, "test.pasta");
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    println!("Result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_with_speech_fullwidth() {
    let source = "＊テスト\n  ａ：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    println!("Result: {:?}", result);
    assert!(result.is_ok());
}
