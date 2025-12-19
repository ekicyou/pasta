use rune::{Context, Sources};

#[test]
fn test_string_vs_variable_actor() {
    let context = Context::with_default_modules().unwrap();

    // Test 1: String assignment (current transpiler output)
    let code_string = r#"
        pub fn test_string(ctx) {
            ctx.actor = "さくら";
        }
    "#;

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("test", code_string).unwrap())
        .unwrap();
    let result_string = rune::prepare(&mut sources).with_context(&context).build();

    println!("String assignment: {:?}", result_string.is_ok());

    // Test 2: Variable assignment (reference implementation)
    let code_var = r#"
        pub const さくら = #{ name: "さくら" };
        
        pub fn test_var(ctx) {
            ctx.actor = さくら;
        }
    "#;

    let mut sources2 = Sources::new();
    sources2
        .insert(rune::Source::new("test2", code_var).unwrap())
        .unwrap();
    let result_var = rune::prepare(&mut sources2).with_context(&context).build();

    println!("Variable assignment: {:?}", result_var.is_ok());

    assert!(result_string.is_ok(), "String assignment should compile");
    assert!(result_var.is_ok(), "Variable assignment should compile");
}
