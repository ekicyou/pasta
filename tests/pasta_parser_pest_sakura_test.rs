//! Test pest grammar directly for sakura script

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
struct PastaParser;

#[test]
fn test_sakura_script_direct() {
    let input = r#"\s[0]"#;
    let result = PastaParser::parse(Rule::sakura_script, input);

    match result {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("Rule: {:?}", pair.as_rule());
            println!("Text: {:?}", pair.as_str());

            for inner in pair.into_inner() {
                println!("  Inner rule: {:?}", inner.as_rule());
                println!("  Inner text: {:?}", inner.as_str());
            }
        }
        Err(e) => {
            panic!("Parse error: {}", e);
        }
    }
}

#[test]
fn test_speech_content_with_sakura() {
    let input = r#"こんにちは\s[0]"#;
    let result = PastaParser::parse(Rule::speech_content, input);

    match result {
        Ok(pairs) => {
            println!("Speech content parsed:");
            for pair in pairs {
                println!("Rule: {:?}, Text: {:?}", pair.as_rule(), pair.as_str());
                for inner in pair.into_inner() {
                    println!(
                        "  Inner rule: {:?}, Text: {:?}",
                        inner.as_rule(),
                        inner.as_str()
                    );
                }
            }
        }
        Err(e) => {
            panic!("Parse error: {}", e);
        }
    }
}
