// End-to-end test: Parse -> Transpile -> Compile -> Execute

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;
use rune::{Context, Sources, Vm};

#[test]
fn test_simple_end_to_end() {
    // Simple pasta script
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    // Parse
    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");

    // Transpile (two-pass)
    let rune_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Generated Rune Code ===");
    println!("{}", rune_code);
    println!("===========================");

    // Compile (add actors module required for use crate::actors::*)
    let actors_def = r#"
pub mod actors {
    pub const さくら = #{
        name: "さくら",
        id: "sakura",
    };
}
"#;
    let combined_code = format!("{}\n\n{}", actors_def, rune_code);

    // Create dummy label table for testing
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);

    let mut context = Context::with_default_modules().expect("Failed to create context");
    context
        .install(pasta::stdlib::create_module(table).expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("entry", &combined_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let unit = match rune::prepare(&mut sources).with_context(&context).build() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("=== Rune Build Error ===");
            eprintln!("{:?}", e);
            panic!("Failed to compile Rune code: {:?}", e);
        }
    };

    println!("Rune compilation succeeded!");

    // Execute
    let mut vm = Vm::new(
        std::sync::Arc::new(context.runtime().expect("Failed to get runtime")),
        std::sync::Arc::new(unit),
    );

    // Create a simple context object
    let ctx = rune::to_value(rune::runtime::Object::new()).expect("Failed to create context");

    // Create empty args array for the second parameter
    let args = rune::to_value(Vec::<rune::Value>::new()).expect("Failed to create args");

    // Execute __start__ function
    let hash = rune::Hash::type_hash(&["会話_1", "__start__"]);
    let result = vm.execute(hash, (ctx, args));

    match result {
        Ok(_execution) => {
            println!("Execution succeeded!");
        }
        Err(e) => {
            panic!("Execution failed: {:?}", e);
        }
    }
}

#[test]
fn test_simple_generator_execution() {
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let rune_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    // Add actors module required for use crate::actors::*
    let actors_def = r#"
pub mod actors {
    pub const さくら = #{
        name: "さくら",
        id: "sakura",
    };
}
"#;
    let combined_code = format!("{}\n\n{}", actors_def, rune_code);

    // Create dummy label table for testing
    let selector = Box::new(pasta::runtime::random::DefaultRandomSelector::new());
    let table = pasta::runtime::labels::LabelTable::new(selector);

    let mut context = Context::with_default_modules().expect("Failed to create context");
    context
        .install(pasta::stdlib::create_module(table).expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("entry", &combined_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .build()
        .expect("Failed to compile Rune code");

    let mut vm = Vm::new(
        std::sync::Arc::new(context.runtime().expect("Failed to get runtime")),
        std::sync::Arc::new(unit),
    );

    // Create context with required fields
    let mut ctx_obj = rune::runtime::Object::new();
    ctx_obj
        .insert(
            rune::alloc::String::try_from("actor").unwrap(),
            rune::to_value("").unwrap(),
        )
        .unwrap();
    let ctx = rune::to_value(ctx_obj).expect("Failed to create context");

    // Create empty args array for the second parameter
    let args = rune::to_value(Vec::<rune::Value>::new()).expect("Failed to create args");

    // Execute as generator - vm.execute returns VmExecution
    let hash = rune::Hash::type_hash(&["会話_1", "__start__"]);
    let execution = vm.execute(hash, (ctx, args)).expect("Failed to execute");
    let mut generator = execution.into_generator();

    // Iterate through events using resume
    let mut events = Vec::new();
    let unit_value = rune::to_value(()).expect("Failed to create unit value");
    loop {
        match generator.resume(unit_value.clone()) {
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                println!("Event: {:?}", value);
                events.push(value);
            }
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => break,
            rune::runtime::VmResult::Err(e) => panic!("Generator error: {:?}", e),
        }
    }

    assert!(!events.is_empty(), "Generator should yield events");
}
