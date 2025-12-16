//! Test while-let-yield pattern for proper yield propagation

use rune::{Context, Vm};
use std::sync::Arc;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = rune::sources! {
        entry => {
            pub fn inner() {
                yield "inner_1";
                yield "inner_2";
            }

            pub fn middle() {
                yield "middle_start";

                // Pattern: Using for loop (Rune's idiomatic way)
                for value in inner() {
                    yield value;
                }

                yield "middle_end";
            }

            pub fn outer() {
                yield "outer_start";

                // Pattern: Using for loop for nested propagation
                for value in middle() {
                    yield value;
                }

                yield "outer_end";
            }
        }
    };

    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;
    let mut vm = Vm::new(runtime, Arc::new(unit));

    let execution = vm.execute(rune::Hash::type_hash(&["outer"]), ())?;
    let mut generator = execution.into_generator();

    println!("=== While-let-yield pattern test ===");
    let unit_value = rune::to_value(())?;
    let mut count = 0;

    loop {
        match generator.resume(unit_value.clone()) {
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                count += 1;
                let s: String = rune::from_value(value)?;
                println!("Event {}: {}", count, s);
            }
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                println!("\nTotal events: {}", count);
                println!("Expected: 6 (outer_start, middle_start, inner_1, inner_2, middle_end, outer_end)");
                break;
            }
            rune::runtime::VmResult::Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
