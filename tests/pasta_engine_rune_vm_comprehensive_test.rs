use pasta::parser::parse_file;
use pasta::transpiler::Transpiler;
use rune::{Context, Sources};
use std::path::Path;

/// Helper to create a test word table
fn create_test_word_table() -> pasta::runtime::words::WordTable {
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let registry = pasta::transpiler::WordDefRegistry::new();
    pasta::runtime::words::WordTable::from_word_def_registry(registry, selector)
}

#[test]
fn test_comprehensive_control_flow_rune_compile() {
    let pasta_path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");
    let main_rn_path = Path::new("tests/fixtures/test-project/main.rn");

    // Parse Pasta file
    let ast = parse_file(pasta_path).expect("Failed to parse");

    // Transpile to Rune code
    let transpiled_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled Rune Code (first 100 lines) ===");
    for (i, line) in transpiled_code.lines().take(100).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("... ({} lines total)", transpiled_code.lines().count());

    // Read main.rn (actor definitions)
    let main_rn = std::fs::read_to_string(main_rn_path).expect("Failed to read main.rn");

    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Install pasta_stdlib
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);
    let word_table = create_test_word_table();

    context
        .install(pasta::stdlib::create_module(table, word_table).expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    // Combine main.rn and transpiled code into a single source
    // This is necessary because Rune's `use crate::actors::*;` needs to reference
    // the actors module defined in the same compilation unit
    let combined_code = format!("{}\n\n{}", main_rn, transpiled_code);

    // Debug: save combined code to file
    if let Err(e) = std::fs::write("test_combined_code.rn", &combined_code) {
        eprintln!("Warning: Failed to write debug file: {}", e);
    }

    // Add combined source
    let mut sources = Sources::new();
    sources
        .insert(
            rune::Source::new("combined", &combined_code)
                .expect("Failed to create combined source"),
        )
        .expect("Failed to add combined code");

    // Compile
    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_unit) => {
            println!("\n✅ Rune VM compilation SUCCEEDED for comprehensive_control_flow.pasta!");
            println!("   ✓ main.rn (actor definitions) compiled successfully");
            println!("   ✓ __pasta_trans2__ module compiled successfully");
            println!("   ✓ pasta module compiled successfully");
            println!("   ✓ All label functions (6 labels) compiled successfully");
            println!("   ✓ Rune blocks with actor variables resolved successfully");
        }
        Err(e) => {
            println!("\n❌ Rune VM compilation FAILED!");
            eprintln!("Error: {}", e);
            panic!("Rune compilation failed for comprehensive_control_flow.pasta");
        }
    }
}
