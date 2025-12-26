/// Test: Can we add Rust functions to Rune-generated modules?
///
/// Purpose: Verify if we can register Rust functions into a module that was defined in Rune code
///
/// Scenario:
/// - Define `pub mod pasta { pub fn rune_func() }` in Rune source
/// - Try to add Rust function to the same `pasta` module via Context
/// - Check if both functions are accessible
use rune::{Context, Diagnostics, Module, Source, Sources, Vm};
use std::sync::Arc;

#[test]
fn test_add_rust_function_to_rune_module() {
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Try to create a module named "pasta" with Rust function
    let mut rust_module = Module::with_crate("pasta").expect("Failed to create pasta module");

    // Register Rust function
    rust_module
        .function("rust_function", rust_test_function)
        .build()
        .expect("Failed to register rust_function");

    context
        .install(rust_module)
        .expect("Failed to install pasta module");

    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    // Rune code defining the same module name
    let rune_code = r#"
        pub mod pasta {
            pub fn rune_function() {
                "from_rune"
            }
        }
        
        pub fn main() {
            let a = pasta::rune_function();
            let b = pasta::rust_function();
            (a, b)
        }
    "#;

    sources
        .insert(Source::memory(rune_code).expect("Failed to create source"))
        .expect("Failed to insert source");

    // Compile
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        println!("=== Diagnostics ===");
        println!("{:?}", diagnostics);
    }

    match result {
        Ok(unit) => {
            println!("✅ Compilation succeeded - Rust module and Rune module can coexist");
            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    println!("✅ Execution succeeded: {:?}", output);
                    println!("✅ CONFIRMED: Can register Rust functions to Rune module namespace");
                }
                Err(e) => {
                    println!("⚠️  Compilation succeeded but execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}", e);
            println!("❌ Cannot add Rust functions to Rune-defined module");
        }
    }
}

fn rust_test_function() -> &'static str {
    "from_rust"
}

#[test]
fn test_rust_module_then_rune_calls() {
    // Test reverse order: Register Rust module first, then call from Rune
    let mut context = Context::with_default_modules().expect("Failed to create context");

    let mut rust_module = Module::with_crate("pasta").expect("Failed to create pasta module");

    rust_module
        .function("rust_helper", rust_helper_function)
        .build()
        .expect("Failed to register rust_helper");

    context
        .install(rust_module)
        .expect("Failed to install pasta module");

    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    // Rune code that only calls Rust function (no module definition)
    let rune_code = r#"
        pub fn main() {
            pasta::rust_helper("test")
        }
    "#;

    sources
        .insert(Source::memory(rune_code).expect("Failed to create source"))
        .expect("Failed to insert source");

    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        println!("=== Diagnostics (Rust module usage) ===");
        println!("{:?}", diagnostics);
    }

    match result {
        Ok(unit) => {
            println!("✅ Can call Rust-registered module from Rune");
            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    println!("✅ Execution result: {:?}", output);
                }
                Err(e) => {
                    println!("⚠️  Execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}", e);
        }
    }
}

fn rust_helper_function(input: String) -> String {
    format!("rust_processed: {}", input)
}

