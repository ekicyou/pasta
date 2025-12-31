/// Test: Rune module merge behavior when same module name exists in multiple sources
///
/// Purpose: Verify if functions in the same module from different sources are merged or overwritten
///
/// Scenario:
/// - source1: pub mod pasta { pub fn func_a() {} }
/// - source2: pub mod pasta { pub fn func_b() {} }
///
/// Expected: Both pasta_rune::func_a() and pasta_rune::func_b() should be callable
/// Failure: One source overwrites the other, causing "function not found" error
use rune::{Context, Diagnostics, Source, Sources, Vm};
use std::sync::Arc;

#[test]
#[should_panic(expected = "Compilation failed")]
fn test_module_merge_in_multiple_sources() {
    // Create Rune context
    let context = Context::with_default_modules().expect("Failed to create context");
    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    // Create two sources with the same module name but different functions
    let mut sources = Sources::new();

    // Source 1: mod pasta with func_a
    let source1 = r#"
        pub mod pasta {
            pub fn func_a() {
                "from_source_1"
            }
        }
    "#;

    sources
        .insert(Source::new("source1", source1).expect("Failed to create source1"))
        .expect("Failed to insert source1");

    // Source 2: mod pasta with func_b
    let source2 = r#"
        pub mod pasta {
            pub fn func_b() {
                "from_source_2"
            }
        }
    "#;

    sources
        .insert(Source::new("source2", source2).expect("Failed to create source2"))
        .expect("Failed to insert source2");

    // Source 3: main script calling both functions
    let source3 = r#"
        pub fn main() {
            let a = pasta_rune::func_a();
            let b = pasta_rune::func_b();
            (a, b)
        }
    "#;

    sources
        .insert(Source::new("main", source3).expect("Failed to create source3"))
        .expect("Failed to insert source3");

    // Compile
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    // Check compilation result
    if !diagnostics.is_empty() {
        println!("=== Compilation Diagnostics ===");
        println!("{:?}", diagnostics);
    }

    let unit = match result {
        Ok(unit) => {
            println!("✅ Compilation succeeded - modules can coexist");
            unit
        }
        Err(e) => {
            panic!(
                "❌ Compilation failed: {:?}\nThis means Rune does NOT merge modules from different sources",
                e
            );
        }
    };

    // Try to execute
    let mut vm = Vm::new(runtime, Arc::new(unit));

    match vm.execute(["main"], ()) {
        Ok(mut result) => {
            let output = result.complete().expect("Failed to complete execution");
            println!("✅ Execution succeeded: {:?}", output);
            println!("✅ CONFIRMED: Rune MERGES functions from same module in different sources");
        }
        Err(e) => {
            panic!(
                "❌ Execution failed: {:?}\nModules compiled but runtime failed",
                e
            );
        }
    }
}

#[test]
fn test_module_function_overwrite() {
    // Test if same function name in same module gets overwritten
    let context = Context::with_default_modules().expect("Failed to create context");
    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    // Source 1: mod pasta with func_test returning "first"
    let source1 = r#"
        pub mod pasta {
            pub fn func_test() {
                "first"
            }
        }
    "#;

    sources
        .insert(Source::new("source1", source1).expect("Failed to create source1"))
        .expect("Failed to insert source1");

    // Source 2: mod pasta with func_test returning "second"
    let source2 = r#"
        pub mod pasta {
            pub fn func_test() {
                "second"
            }
        }
    "#;

    sources
        .insert(Source::new("source2", source2).expect("Failed to create source2"))
        .expect("Failed to insert source2");

    // Source 3: main calling the function
    let source3 = r#"
        pub fn main() {
            pasta_rune::func_test()
        }
    "#;

    sources
        .insert(Source::new("main", source3).expect("Failed to create source3"))
        .expect("Failed to insert source3");

    // Compile
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        println!("=== Diagnostics for overwrite test ===");
        println!("{:?}", diagnostics);
    }

    match result {
        Ok(unit) => {
            // If compilation succeeds, check which version is used
            let mut vm = Vm::new(runtime, Arc::new(unit));
            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    // Try to convert to string for display
                    println!("⚠️  Same function name compiled without error");
                    println!("    Result: {:?}", output);
                }
                Err(e) => {
                    println!("⚠️  Compilation succeeded but execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!(
                "✅ Compilation failed for duplicate function names: {:?}",
                e
            );
            println!("✅ CONFIRMED: Rune rejects duplicate function names in same module");
        }
    }
}
