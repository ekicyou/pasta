//! Integration test for standard library functions

use pasta::stdlib;
use rune::{Context, Vm};
use std::sync::Arc;

#[test]
fn test_stdlib_module_creation() {
    // Create dummy label table for testing
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);

    let result = stdlib::create_module(table);
    assert!(
        result.is_ok(),
        "Failed to create stdlib module: {:?}",
        result.err()
    );
}

#[test]
fn test_emit_text_via_rune() -> Result<(), Box<dyn std::error::Error>> {
    // Create dummy label table for testing
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);

    // Create context with stdlib
    let mut context = Context::with_default_modules()?;
    context.install(stdlib::create_module(table)?)?;

    // Compile a simple script that uses emit_text
    let mut sources = rune::sources! {
        entry => {
            use pasta_stdlib::*;

            pub fn main() {
                emit_text("Hello")
            }
        }
    };

    let runtime = Arc::new(context.runtime()?);
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));
    let output = vm.call(rune::Hash::type_hash(&["main"]), ())?;

    // Note: We can't directly convert ScriptEvent from Rune value yet
    // This test just verifies compilation and execution work
    println!("Output value: {:?}", output);

    Ok(())
}

#[test]
fn test_sync_functions_via_rune() -> Result<(), Box<dyn std::error::Error>> {
    // Create dummy label table for testing
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);

    // Create context with stdlib
    let mut context = Context::with_default_modules()?;
    context.install(stdlib::create_module(table)?)?;

    // Compile a script that uses sync functions
    let mut sources = rune::sources! {
        entry => {
            use pasta_stdlib::*;

            pub fn main() {
                begin_sync("sync1");
                sync_point("sync1");
                end_sync("sync1")
            }
        }
    };

    let runtime = Arc::new(context.runtime()?);
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));
    let output = vm.call(rune::Hash::type_hash(&["main"]), ())?;

    println!("Output value: {:?}", output);

    Ok(())
}
