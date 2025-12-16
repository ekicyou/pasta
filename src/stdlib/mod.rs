//! Standard library for Pasta scripts.
//!
//! This module provides the standard library functions that are available to
//! Pasta scripts running in the Rune VM, including emit functions, wait functions,
//! and synchronization functions.

pub mod persistence;

use crate::ir::{ContentPart, ScriptEvent};
use crate::runtime::labels::LabelTable;
use rune::{ContextError, Module};
use std::collections::HashMap;
use std::sync::Mutex;

/// Create the Pasta standard library module for Rune.
pub fn create_module(label_table: LabelTable) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("pasta_stdlib")?;

    // Register emit functions
    module.function("emit_text", emit_text).build()?;
    module
        .function("emit_sakura_script", emit_sakura_script)
        .build()?;
    module.function("change_speaker", change_speaker).build()?;
    module.function("change_surface", change_surface).build()?;
    module.function("wait", wait).build()?;

    // Register synchronization functions
    module.function("begin_sync", begin_sync).build()?;
    module.function("sync_point", sync_point).build()?;
    module.function("end_sync", end_sync).build()?;

    // Register utility functions
    module.function("fire_event", fire_event).build()?;
    module.function("emit_error", emit_error).build()?;

    // Register persistence functions
    persistence::register_persistence_functions(&mut module)?;

    // Register label resolution functions
    // Wrap in Mutex for interior mutability (resolve_label_id needs &mut self)
    let label_table_mutex = Mutex::new(label_table);
    module
        .function(
            "select_label_to_id",
            move |label: String, filters: rune::runtime::Value| {
                select_label_to_id(label, filters, &label_table_mutex)
            },
        )
        .build()?;

    // Register word expansion functions (P0: stub implementation)
    module.function("word", word_expansion).build()?;

    // Register event constructor functions
    module.function("Actor", actor_event).build()?;
    module.function("Talk", talk_event).build()?;
    module.function("Error", error_event).build()?;

    Ok(module)
}

/// Label resolution with prefix matching and attribute filtering.
///
/// # Arguments
/// * `label` - Label name to resolve (search key)
/// * `filters` - Attribute filters (Rune Object or Unit)
/// * `label_table` - Shared reference to the label table
///
/// # Returns
/// Label ID as i64
///
/// # Panics
/// Panics if label resolution fails (no matching labels, lock error, etc.)
fn select_label_to_id(
    label: String,
    filters: rune::runtime::Value,
    label_table: &Mutex<LabelTable>,
) -> Result<i64, String> {
    // Phase 1: Parse Rune filters to HashMap
    let filter_map = parse_rune_filters(filters)?;

    // Phase 2: Lock and resolve label ID
    let mut table = label_table
        .lock()
        .map_err(|e| format!("Failed to lock label_table: {}", e))?;

    let label_id = table
        .resolve_label_id(&label, &filter_map)
        .map_err(|e| format!("Label resolution failed: {}", e))?;

    // Convert LabelId (0-based) to transpiler ID (1-based)
    Ok((label_id.0 + 1) as i64)
}

/// Parse Rune Value filters to Rust HashMap.
///
/// # Arguments
/// * `value` - Rune Value (Unit/(), Object/HashMap, or other)
///
/// # Returns
/// HashMap<String, String> or error message
fn parse_rune_filters(value: rune::Value) -> Result<HashMap<String, String>, String> {
    // Try to convert to HashMap using rune::from_value
    match rune::from_value::<HashMap<String, rune::Value>>(value.clone()) {
        Ok(map) => {
            // Convert HashMap<String, rune::Value> to HashMap<String, String>
            let mut result = HashMap::new();
            for (key, val) in map {
                // Try to convert value to String
                let val_str = rune::from_value::<String>(val)
                    .map_err(|e| format!("Filter value must be string for key '{}': {}", key, e))?;
                result.insert(key, val_str);
            }
            Ok(result)
        }
        Err(_) => {
            // Try unit type (empty filters)
            match rune::from_value::<()>(value.clone()) {
                Ok(_) => Ok(HashMap::new()),
                Err(_) => Err(format!("Filters must be object or unit")),
            }
        }
    }
}

/// P0 implementation: Stub word expansion that returns the word name as-is.
///
/// This allows basic testing without implementing the full word dictionary.
/// P1 will implement proper word expansion with random selection.
///
/// # Arguments
/// * `_ctx` - Context object (unused in P0)
/// * `word` - Word name to expand
/// * `_args` - Arguments (unused in P0)
///
/// # Returns
/// A Talk event with the word name
fn word_expansion(
    _ctx: rune::runtime::Value,
    word: String,
    _args: rune::runtime::Value,
) -> ScriptEvent {
    // P0: Just return the word name as text
    // P1 will implement:
    // - Word dictionary lookup
    // - Random selection from alternatives
    // - Cache-based exhaustion
    ScriptEvent::Talk {
        speaker: String::new(),
        content: vec![ContentPart::Text(word)],
    }
}

/// Create an Actor event (speaker change).
///
/// # Arguments
/// * `name` - Speaker name
///
/// # Returns
/// A ChangeSpeaker event
fn actor_event(name: String) -> ScriptEvent {
    ScriptEvent::ChangeSpeaker { name }
}

/// Create a Talk event.
///
/// # Arguments
/// * `text` - Text content
///
/// # Returns
/// A Talk event
fn talk_event(text: String) -> ScriptEvent {
    ScriptEvent::Talk {
        speaker: String::new(),
        content: vec![ContentPart::Text(text)],
    }
}

/// Create an Error event.
///
/// # Arguments
/// * `message` - Error message
///
/// # Returns
/// An Error event
fn error_event(message: String) -> ScriptEvent {
    ScriptEvent::Error { message }
}

/// Emit text content as a Talk event.
///
/// This function should be called within a generator context and will yield
/// a ScriptEvent::Talk with the current speaker and text content.
pub fn emit_text(text: String) -> ScriptEvent {
    // Note: In actual implementation, this needs to be aware of the current speaker
    // For now, we'll create a simplified version
    ScriptEvent::Talk {
        speaker: String::new(), // Speaker is set by change_speaker
        content: vec![ContentPart::Text(text)],
    }
}

/// Emit a sakura script escape sequence.
///
/// This passes through sakura script commands without interpretation.
fn emit_sakura_script(script: String) -> ScriptEvent {
    ScriptEvent::Talk {
        speaker: String::new(),
        content: vec![ContentPart::SakuraScript(script)],
    }
}

/// Change the current speaker.
///
/// This emits a ChangeSpeaker event to set the speaker for subsequent Talk events.
fn change_speaker(name: String) -> ScriptEvent {
    ScriptEvent::ChangeSpeaker { name }
}

/// Change a character's surface (expression/pose).
fn change_surface(character: String, surface_id: i64) -> ScriptEvent {
    ScriptEvent::ChangeSurface {
        character,
        surface_id: surface_id as u32,
    }
}

/// Wait for a specified duration (in seconds).
fn wait(duration: f64) -> ScriptEvent {
    ScriptEvent::Wait { duration }
}

/// Begin a synchronized section.
///
/// All events between begin_sync and end_sync will be buffered and
/// played simultaneously when all participants reach the sync point.
fn begin_sync(sync_id: String) -> ScriptEvent {
    ScriptEvent::BeginSync { sync_id }
}

/// Mark a synchronization point.
///
/// When all participants in a synchronized section reach this point,
/// buffered events will be played simultaneously.
fn sync_point(sync_id: String) -> ScriptEvent {
    ScriptEvent::SyncPoint { sync_id }
}

/// End a synchronized section.
fn end_sync(sync_id: String) -> ScriptEvent {
    ScriptEvent::EndSync { sync_id }
}

/// Fire a custom event.
fn fire_event(event_name: String, params: Vec<(String, String)>) -> ScriptEvent {
    ScriptEvent::FireEvent { event_name, params }
}

/// Emit a runtime error event.
///
/// This function allows scripts to yield error events that can be handled
/// by the application layer. The generator continues execution after yielding
/// an error, allowing for error recovery.
///
/// # Example (Rune code)
///
/// ```rune
/// pub fn risky_operation() {
///     if something_wrong {
///         yield emit_error("Something went wrong!");
///         // Execution continues after the error
///     }
///     yield emit_text("Continuing normally");
/// }
/// ```
fn emit_error(message: String) -> ScriptEvent {
    ScriptEvent::Error { message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_text() {
        let event = emit_text("Hello".to_string());
        assert!(event.is_talk());
        if let ScriptEvent::Talk {
            speaker: _,
            content,
        } = event
        {
            assert_eq!(content.len(), 1);
            assert_eq!(content[0], ContentPart::Text("Hello".to_string()));
        }
    }

    #[test]
    fn test_emit_sakura_script() {
        let event = emit_sakura_script("\\s[0]".to_string());
        assert!(event.is_talk());
        if let ScriptEvent::Talk {
            speaker: _,
            content,
        } = event
        {
            assert_eq!(content.len(), 1);
            assert_eq!(content[0], ContentPart::SakuraScript("\\s[0]".to_string()));
        }
    }

    #[test]
    fn test_change_speaker() {
        let event = change_speaker("sakura".to_string());
        if let ScriptEvent::ChangeSpeaker { name } = event {
            assert_eq!(name, "sakura");
        } else {
            panic!("Expected ChangeSpeaker event");
        }
    }

    #[test]
    fn test_wait() {
        let event = wait(1.5);
        assert!(event.is_wait());
        if let ScriptEvent::Wait { duration } = event {
            assert_eq!(duration, 1.5);
        }
    }

    #[test]
    fn test_sync_markers() {
        let begin = begin_sync("sync1".to_string());
        let point = sync_point("sync1".to_string());
        let end = end_sync("sync1".to_string());

        assert!(begin.is_sync_marker());
        assert!(point.is_sync_marker());
        assert!(end.is_sync_marker());
    }

    #[test]
    fn test_create_module() {
        use crate::runtime::labels::LabelTable;
        use crate::runtime::random::DefaultRandomSelector;

        // Create a test label table
        let selector = Box::new(DefaultRandomSelector::new());
        let table = LabelTable::new(selector);

        // Test that module creation succeeds
        let result = create_module(table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_error() {
        let event = emit_error("Test error message".to_string());
        assert!(event.is_error());
        if let ScriptEvent::Error { message } = event {
            assert_eq!(message, "Test error message");
        } else {
            panic!("Expected Error event");
        }
    }
}
