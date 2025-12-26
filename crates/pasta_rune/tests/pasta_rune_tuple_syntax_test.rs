// Test to verify Rune's tuple syntax support (0, 1, and 2+ element tuples)
// This test validates that Rune supports all tuple forms needed for the transpiler conversion.

use rune::{Context, Vm};
use std::sync::Arc;

#[test]
fn test_rune_zero_element_tuple() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Empty tuple () is valid in Rune
    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);

    let mut sources = rune::sources! {
        entry => {
            pub fn test_empty_tuple() {
                let empty = ();
                empty
            }
        }
    };

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));

    let output = vm.call(rune::Hash::type_hash(&["test_empty_tuple"]), ())?;
    let output: () = rune::from_value(output)?;

    // If we reach here, Rune supports empty tuples
    assert_eq!(output, ());
    Ok(())
}

#[test]
fn test_rune_single_element_tuple() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Single element tuple (x,) is valid in Rune
    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);

    let mut sources = rune::sources! {
        entry => {
            pub fn test_single_tuple(arg) {
                let single = (arg,);
                single
            }
        }
    };

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));

    let output = vm.call(rune::Hash::type_hash(&["test_single_tuple"]), (42i64,))?;
    let output: (i64,) = rune::from_value(output)?;

    // If we reach here, Rune supports single element tuples
    assert_eq!(output, (42,));
    Ok(())
}

#[test]
fn test_rune_multi_element_tuple() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Multi-element tuple (x, y, z) is valid in Rune
    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);

    let mut sources = rune::sources! {
        entry => {
            pub fn test_multi_tuple(a, b, c) {
                let triple = (a, b, c);
                triple
            }
        }
    };

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    let mut vm = Vm::new(runtime, Arc::new(unit));

    let output = vm.call(
        rune::Hash::type_hash(&["test_multi_tuple"]),
        (10i64, 20i64, 30i64),
    )?;
    let output: (i64, i64, i64) = rune::from_value(output)?;

    // If we reach here, Rune supports multi-element tuples
    assert_eq!(output, (10, 20, 30));
    Ok(())
}

#[test]
fn test_rune_tuple_as_function_argument() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Passing tuples as function arguments
    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);

    let mut sources = rune::sources! {
        entry => {
            pub fn accepts_tuple(args) {
                // args is a tuple passed as parameter
                args
            }
        }
    };

    let unit = Arc::new(rune::prepare(&mut sources).with_context(&context).build()?);

    // Test with empty tuple
    let mut vm = Vm::new(runtime.clone(), unit.clone());
    let output_empty = vm.call(rune::Hash::type_hash(&["accepts_tuple"]), ((),))?;
    let result_empty: () = rune::from_value(output_empty)?;
    assert_eq!(result_empty, ());

    // Test with single-element tuple
    let mut vm2 = Vm::new(runtime.clone(), unit.clone());
    let output_single = vm2.call(rune::Hash::type_hash(&["accepts_tuple"]), ((100i64,),))?;
    let result_single: (i64,) = rune::from_value(output_single)?;
    assert_eq!(result_single, (100,));

    // Test with multi-element tuple
    let mut vm3 = Vm::new(runtime, unit);
    let output_multi = vm3.call(
        rune::Hash::type_hash(&["accepts_tuple"]),
        ((1i64, 2i64, 3i64),),
    )?;
    let result_multi: (i64, i64, i64) = rune::from_value(output_multi)?;
    assert_eq!(result_multi, (1, 2, 3));

    Ok(())
}

