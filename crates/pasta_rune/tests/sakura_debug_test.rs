//! Debug test for sakura script parsing

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta_rune::PastaEngine;

#[test]
fn debug_sakura_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：こんにちは\w8お元気ですか
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    println!("Total events: {}", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {:?}", i, event);
    }

    Ok(())
}
