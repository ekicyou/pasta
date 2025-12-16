use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
struct PastaParser;

#[test]
fn test_global_label_with_statement() {
    let input = "*test\n  a:hello\n";
    let result = PastaParser::parse(Rule::global_label, input);
    println!("Parse result: {:#?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_file_with_statement() {
    let input = "*test\n  a:hello\n";
    let result = PastaParser::parse(Rule::file, input);
    println!("Parse result: {:#?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
