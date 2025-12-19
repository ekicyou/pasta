//! Test transpiling comprehensive_control_flow.pasta

use pasta::parser::parse_file;
use pasta::transpiler::Transpiler;
use std::path::Path;

#[test]
fn test_transpile_comprehensive_control_flow() {
    let pasta_path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");

    println!("Parsing {}...", pasta_path.display());
    let ast = match parse_file(pasta_path) {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Global labels: {}", ast.labels.len());
            println!("Global words: {}", ast.global_words.len());
            ast
        }
        Err(e) => {
            panic!("❌ Parse failed: {:?}", e);
        }
    };

    println!("\nTranspiling...");
    match Transpiler::transpile_to_string(&ast) {
        Ok(rune_code) => {
            println!("✅ Transpile successful!");
            println!("\n=== Generated Rune Code ===\n");
            println!("{}", rune_code);

            // Write output to file for inspection
            std::fs::write(
                "tests/fixtures/comprehensive_control_flow.transpiled.rn",
                &rune_code,
            )
            .expect("Failed to write transpiled output");
            println!("\n✅ Written to tests/fixtures/comprehensive_control_flow.transpiled.rn");
        }
        Err(e) => {
            panic!("❌ Transpile failed: {:?}", e);
        }
    }
}
