//! Diagnostic tests for pest grammar

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
struct PastaParser;

#[test]
fn test_argument_value_number() {
    let input = "300";
    let result = PastaParser::parse(Rule::arg_value, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_arg_list_simple() {
    let input = "(300)";
    let result = PastaParser::parse(Rule::arg_list, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_arg_list_fullwidth() {
    let input = "（300）";
    let result = PastaParser::parse(Rule::arg_list, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_identifier() {
    let input = "関数名";
    let result = PastaParser::parse(Rule::ident, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_func_name() {
    let input = "関数名";
    let result = PastaParser::parse(Rule::func_name, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_at_marker() {
    let input = "＠";
    let result = PastaParser::parse(Rule::at_marker, input);
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_ideographic_space() {
    // Full-width ideographic space
    let input = "　";
    assert_eq!(input.len(), 3); // U+3000 is 3 bytes in UTF-8

    let result = PastaParser::parse(Rule::indent, input);
    println!("Ideographic space parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse ideographic space as indent"
    );
}

#[test]
fn test_two_args_with_ideographic_space() {
    let input = "（「引数１」　「引数２」）";
    let result = PastaParser::parse(Rule::arg_list, input);
    println!("Arg list parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse arg list with ideographic space: {:?}",
        result.err()
    );
}

#[test]
fn test_ws_rule() {
    let input = "　"; // Ideographic space
    let result = PastaParser::parse(Rule::ws, input);
    println!("WS parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse ideographic space as ws: {:?}",
        result.err()
    );
}

#[test]
fn test_ja_string() {
    let input = "「引数１」";
    let result = PastaParser::parse(Rule::string_literal, input);
    println!("JA string parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse Japanese string: {:?}",
        result.err()
    );
}

#[test]
fn test_arg_value_ja_string() {
    let input = "「引数１」";
    let result = PastaParser::parse(Rule::arg_value, input);
    println!("Arg value (JA string) parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse arg_value with Japanese string: {:?}",
        result.err()
    );
}

#[test]
fn test_argument_ja_string() {
    let input = "「引数１」";
    let result = PastaParser::parse(Rule::argument, input);
    println!("Argument parse: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse argument with Japanese string: {:?}",
        result.err()
    );
}

#[test]
fn test_two_args_simple() {
    // Two numbered arguments with ASCII space
    let input = "(123 456)";
    let result = PastaParser::parse(Rule::arg_list, input);
    println!("Two args (ASCII space): {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_two_strings_ascii_space() {
    let input = "(\"arg1\" \"arg2\")";
    let result = PastaParser::parse(Rule::arg_list, input);
    println!("Two strings (ASCII space): {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_rune_start() {
    let input = "```rune";
    let result = PastaParser::parse(Rule::rune_start, input);
    println!("Rune start: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_rune_block_minimal() {
    let input = "\n\n    ```\n";
    let result = PastaParser::parse(Rule::rune_block_content, input);
    println!("Rune block minimal: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
