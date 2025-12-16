//! Test file to understand Rune 0.14 Module API

use rune::{Context, Vm};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile and run a test script - define function in Rune code itself
    let mut sources = rune::sources! {
        entry => {
            fn test_fn() {
                "Hello from Rune!"
            }

            pub fn main() {
                test_fn()
            }
        }
    };

    let context = Context::with_default_modules()?;

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::without_runtime(Arc::new(unit));
    let output = vm.call(rune::Hash::type_hash(&["main"]), ())?;
    let output: String = rune::from_value(output)?;

    println!("Output: {}", output);

    Ok(())
}
