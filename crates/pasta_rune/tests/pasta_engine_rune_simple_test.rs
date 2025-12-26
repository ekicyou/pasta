//! Simple test to understand Rune 0.14 FFI

use rune::{Context, Vm};
use std::sync::Arc;

#[test]
fn test_simple_rust_function() -> Result<(), Box<dyn std::error::Error>> {
    // Create a module with a simple Rust function
    let mut module = rune::Module::with_crate("mymod")?;
    module.function("add", |a: i64, b: i64| a + b).build()?;

    // Create context
    let mut context = Context::with_default_modules()?;
    context.install(module)?;

    // Compile script that uses the function
    let mut sources = rune::sources! {
        entry => {
            pub fn main() {
                mymod::add(1, 2)
            }
        }
    };

    let runtime = Arc::new(context.runtime()?);
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));
    let output = vm.call(rune::Hash::type_hash(&["main"]), ())?;
    let result: i64 = rune::from_value(output)?;

    assert_eq!(result, 3);
    Ok(())
}

