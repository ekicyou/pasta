/// Test: Rune module behavior with Source::memory() for same module additions
///
/// Purpose: Verify if we can add functions to the same module incrementally using Source::memory()
///
/// Scenario:
/// - Use Source::memory() to add pasta module functions incrementally
/// - Check if later additions extend or overwrite the module
use rune::{Context, Diagnostics, Source, Sources, Vm};
use std::sync::Arc;

#[test]
#[should_panic(expected = "Compilation failed")]
fn test_incremental_module_with_memory() {
    // Test if we can build up a module incrementally with Source::memory()
    let context = Context::with_default_modules().expect("Failed to create context");
    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    // First addition: pasta::func_a
    let code1 = r#"
        pub mod pasta {
            pub fn func_a() {
                "from_first_addition"
            }
        }
    "#;

    sources
        .insert(Source::memory(code1).expect("Failed to create source1"))
        .expect("Failed to insert source1");

    // Second addition: pasta::func_b (same module name)
    let code2 = r#"
        pub mod pasta {
            pub fn func_b() {
                "from_second_addition"
            }
        }
    "#;

    sources
        .insert(Source::memory(code2).expect("Failed to create source2"))
        .expect("Failed to insert source2");

    // Main script
    let main_code = r#"
        pub fn main() {
            let a = pasta::func_a();
            let b = pasta::func_b();
            (a, b)
        }
    "#;

    sources
        .insert(Source::memory(main_code).expect("Failed to create main"))
        .expect("Failed to insert main");

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
            println!("✅ Compilation succeeded with Source::memory()");
            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    println!("✅ Execution succeeded: {:?}", output);
                    println!("✅ CONFIRMED: Source::memory() allows incremental module building");
                }
                Err(e) => {
                    panic!("❌ Execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Compilation failed: {:?}\nSource::memory() does NOT support incremental module building", e);
        }
    }
}

#[test]
fn test_single_source_memory_with_complete_module() {
    // Test if we can put everything in one Source::memory() call
    let context = Context::with_default_modules().expect("Failed to create context");
    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    // Single source with complete module
    let code = r#"
        pub mod pasta {
            pub fn jump(ctx, scene, filters, args) {
                let scene_fn = scene_selector(scene, filters);
                for event in scene_fn(ctx, args) { yield event; }
            }
            
            pub fn call(ctx, scene, filters, args) {
                let scene_fn = scene_selector(scene, filters);
                for event in scene_fn(ctx, args) { yield event; }
            }
            
            pub fn scene_selector(scene, filters) {
                let id = select_scene_to_id(scene, filters);
                match id {
                    1 => crate::test_func,
                    _ => panic("Unknown scene"),
                }
            }
            
            pub fn select_scene_to_id(scene, filters) {
                1
            }
        }
        
        pub fn test_func(ctx, args) {
            "test_result"
        }
        
        pub fn main() {
            let ctx = #{};
            pasta::call(ctx, "test", #{}, [])
        }
    "#;

    sources
        .insert(Source::memory(code).expect("Failed to create source"))
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
            println!("✅ Single Source::memory() with complete pasta module compiled successfully");
            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let _ = result.complete().expect("Failed to complete");
                    println!("✅ Execution succeeded - this is our target implementation pattern");
                }
                Err(e) => {
                    println!("⚠️  Compilation succeeded but execution failed: {:?}", e);
                    println!("    (May need to adjust generator/yield handling)");
                }
            }
        }
        Err(e) => {
            panic!("❌ Compilation failed: {:?}", e);
        }
    }
}

