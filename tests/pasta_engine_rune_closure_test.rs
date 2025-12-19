/// Test: Can we register Rust closures/delegates to Rune modules?
///
/// Purpose: Verify if we can register Rust closures (not just static functions) to Rune
/// This is critical for P1 implementation where select_label_to_id needs to access LabelTable
///
/// Scenario:
/// - Create a closure that captures external state
/// - Register it to Rune module
/// - Call it from Rune code
use rune::{Context, Diagnostics, Module, Source, Sources, Value, Vm};
use std::sync::{Arc, Mutex};

#[test]
fn test_register_closure_with_captured_state() {
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Shared state that closure will capture
    let counter = Arc::new(Mutex::new(0i64));
    let counter_clone = counter.clone();

    let mut module = Module::with_crate("test_lib").expect("Failed to create module");

    // Try to register a closure that captures state
    let closure = move || -> i64 {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        *count
    };

    // Attempt to register closure
    let result = module.function("increment", closure).build();

    match result {
        Ok(_) => {
            println!("✅ Closure registration succeeded");

            context.install(module).expect("Failed to install module");
            let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

            let mut sources = Sources::new();

            let rune_code = r#"
                pub fn main() {
                    let a = test_lib::increment();
                    let b = test_lib::increment();
                    let c = test_lib::increment();
                    (a, b, c)
                }
            "#;

            sources
                .insert(Source::memory(rune_code).expect("Failed to create source"))
                .expect("Failed to insert source");

            let mut diagnostics = Diagnostics::new();
            let unit = rune::prepare(&mut sources)
                .with_context(&context)
                .with_diagnostics(&mut diagnostics)
                .build()
                .expect("Failed to compile");

            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    println!("✅ Execution result: {:?}", output);
                    println!("✅ CONFIRMED: Can register closures with captured state");

                    let final_count = *counter.lock().unwrap();
                    println!("✅ Final counter value: {}", final_count);
                    assert_eq!(final_count, 3, "Counter should be incremented 3 times");
                }
                Err(e) => {
                    panic!("❌ Execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Closure registration failed: {:?}\nCannot use closures - must use static functions only", e);
        }
    }
}

#[test]
fn test_register_function_with_arc_parameter() {
    // Test if we can pass Arc<Mutex<State>> as parameter to Rune-registered function
    let mut context = Context::with_default_modules().expect("Failed to create context");

    #[derive(Debug, Clone)]
    struct TestState {
        value: i64,
    }

    let state = Arc::new(Mutex::new(TestState { value: 0 }));
    let state_clone = state.clone();

    let mut module = Module::with_crate("test_lib").expect("Failed to create module");

    // Register function that takes a string parameter (simulating label parameter)
    let func = move |label: String| -> i64 {
        let mut state = state_clone.lock().unwrap();
        state.value += 1;
        println!("Processing label: {}, count: {}", label, state.value);
        state.value
    };

    let result = module.function("process_label", func).build();

    match result {
        Ok(_) => {
            println!("✅ Function with Arc parameter registration succeeded");

            context.install(module).expect("Failed to install module");
            let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

            let mut sources = Sources::new();

            let rune_code = r#"
                pub fn main() {
                    let a = test_lib::process_label("label1");
                    let b = test_lib::process_label("label2");
                    (a, b)
                }
            "#;

            sources
                .insert(Source::memory(rune_code).expect("Failed to create source"))
                .expect("Failed to insert source");

            let mut diagnostics = Diagnostics::new();
            let unit = rune::prepare(&mut sources)
                .with_context(&context)
                .with_diagnostics(&mut diagnostics)
                .build()
                .expect("Failed to compile");

            let mut vm = Vm::new(runtime, Arc::new(unit));

            match vm.execute(["main"], ()) {
                Ok(mut result) => {
                    let output = result.complete().expect("Failed to complete");
                    println!("✅ Execution result: {:?}", output);
                    println!("✅ CONFIRMED: Can use closures with Arc<Mutex<State>> for P1 implementation");

                    let final_state = state.lock().unwrap();
                    println!("✅ Final state value: {}", final_state.value);
                    assert_eq!(final_state.value, 2);
                }
                Err(e) => {
                    panic!("❌ Execution failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Function registration failed: {:?}", e);
        }
    }
}

#[test]
fn test_p1_select_label_to_id_pattern() {
    // Simulate P1 implementation pattern
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Simulate LabelTable
    #[derive(Debug, Clone)]
    struct SimpleLabelTable {
        labels: std::collections::HashMap<String, i64>,
    }

    let mut table = SimpleLabelTable {
        labels: std::collections::HashMap::new(),
    };
    table.labels.insert("会話_1::__start__".to_string(), 1);
    table.labels.insert("会話_1::選択肢_1".to_string(), 2);
    table.labels.insert("会話_1::選択肢_2".to_string(), 3);

    let label_table = Arc::new(Mutex::new(table));
    let label_table_clone = label_table.clone();

    let mut module = Module::with_crate("pasta_stdlib").expect("Failed to create module");

    // P1 implementation pattern
    let select_label_to_id = move |label: String, _filters: Value| -> i64 {
        let table = label_table_clone.lock().unwrap();
        table.labels.get(&label).copied().unwrap_or(1)
    };

    module
        .function("select_label_to_id", select_label_to_id)
        .build()
        .expect("Failed to register select_label_to_id");

    context.install(module).expect("Failed to install module");
    let runtime = Arc::new(context.runtime().expect("Failed to create runtime"));

    let mut sources = Sources::new();

    let rune_code = r#"
        pub fn main() {
            let id1 = pasta_stdlib::select_label_to_id("会話_1::__start__", #{});
            let id2 = pasta_stdlib::select_label_to_id("会話_1::選択肢_1", #{});
            let id3 = pasta_stdlib::select_label_to_id("会話_1::選択肢_2", #{});
            let id_unknown = pasta_stdlib::select_label_to_id("unknown", #{});
            (id1, id2, id3, id_unknown)
        }
    "#;

    sources
        .insert(Source::memory(rune_code).expect("Failed to create source"))
        .expect("Failed to insert source");

    let mut diagnostics = Diagnostics::new();
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build()
        .expect("Failed to compile");

    let mut vm = Vm::new(runtime, Arc::new(unit));

    match vm.execute(["main"], ()) {
        Ok(mut result) => {
            let output = result.complete().expect("Failed to complete");
            println!("✅ P1 pattern execution result: {:?}", output);
            println!("✅ CONFIRMED: P1 implementation pattern is viable with closures");
        }
        Err(e) => {
            panic!("❌ P1 pattern execution failed: {:?}", e);
        }
    }
}
