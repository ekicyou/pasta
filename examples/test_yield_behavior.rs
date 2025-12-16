//! Test Rune yield behavior with nested function calls

use rune::{Context, Vm};
use std::sync::Arc;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = rune::sources! {
        entry => {
            fn b() {
                yield 21;
                yield 22;
            }

            pub fn a() {
                yield 11;
                yield 12;
                b();
                yield 13;
            }
        }
    };

    let context = Context::with_default_modules()?;
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;
    let mut vm = Vm::without_runtime(Arc::new(unit));

    let execution = vm.execute(rune::Hash::type_hash(&["a"]), ())?;
    let mut generator = execution.into_generator();

    println!("Yield values:");
    let unit_value = rune::to_value(())?;

    loop {
        match generator.resume(unit_value.clone()) {
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                let n: i64 = rune::from_value(value)?;
                println!("{}", n);
            }
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                break;
            }
            rune::runtime::VmResult::Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
