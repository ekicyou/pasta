//! Persistence-related functions for Rune scripts.
//!
//! This module provides TOML serialization and file I/O functions
//! for Rune scripts to implement persistence.

use rune::{ContextError, Module};
use std::fs;

/// Register persistence functions to an existing module.
pub fn register_persistence_functions(module: &mut Module) -> Result<(), ContextError> {
    module.function("toml_to_string", toml_to_string).build()?;
    module
        .function("toml_from_string", toml_from_string)
        .build()?;
    module.function("read_text_file", read_text_file).build()?;
    module
        .function("write_text_file", write_text_file)
        .build()?;

    Ok(())
}

/// Serialize a Rune value to TOML string.
///
/// # Example (Rune code)
///
/// ```rune
/// let data = #{ level: 5, gold: 100 };
/// let toml_str = toml_to_string(data)?;
/// ```
fn toml_to_string(data: rune::Value) -> Result<String, String> {
    // Convert Rune Value to HashMap
    let map: std::collections::HashMap<String, rune::Value> = rune::from_value(data)
        .map_err(|e| format!("Failed to convert Rune value to map: {}", e))?;

    // Convert to TOML table
    let mut toml_table = toml::map::Map::new();
    for (key, value) in map {
        toml_table.insert(key, rune_value_to_toml_value(value)?);
    }

    // Serialize TOML to string
    toml::to_string(&toml::Value::Table(toml_table))
        .map_err(|e| format!("TOML serialization failed: {}", e))
}

/// Deserialize TOML string to Rune value.
///
/// # Example (Rune code)
///
/// ```rune
/// let toml_str = "level = 5\ngold = 100\n";
/// let data = toml_from_string(toml_str)?;
/// ```
fn toml_from_string(toml_str: &str) -> Result<rune::Value, String> {
    // Parse TOML string
    let toml_value: toml::Value =
        toml::from_str(toml_str).map_err(|e| format!("TOML parsing failed: {}", e))?;

    // Convert TOML to Rune-compatible HashMap
    toml_value_to_rune_value(&toml_value)
}

/// Convert a Rune Value to a TOML Value.
fn rune_value_to_toml_value(value: rune::Value) -> Result<toml::Value, String> {
    // Try different types using rune::from_value
    if let Ok(s) = rune::from_value::<String>(value.clone()) {
        return Ok(toml::Value::String(s));
    }
    if let Ok(i) = rune::from_value::<i64>(value.clone()) {
        return Ok(toml::Value::Integer(i));
    }
    if let Ok(f) = rune::from_value::<f64>(value.clone()) {
        return Ok(toml::Value::Float(f));
    }
    if let Ok(b) = rune::from_value::<bool>(value.clone()) {
        return Ok(toml::Value::Boolean(b));
    }
    // Try Vec
    if let Ok(vec) = rune::from_value::<Vec<rune::Value>>(value.clone()) {
        let mut toml_array = Vec::new();
        for item in vec {
            toml_array.push(rune_value_to_toml_value(item)?);
        }
        return Ok(toml::Value::Array(toml_array));
    }
    // Try HashMap (object)
    if let Ok(map) =
        rune::from_value::<std::collections::HashMap<String, rune::Value>>(value.clone())
    {
        let mut toml_table = toml::map::Map::new();
        for (key, val) in map {
            toml_table.insert(key, rune_value_to_toml_value(val)?);
        }
        return Ok(toml::Value::Table(toml_table));
    }

    Err(format!("Unsupported Rune value type for TOML conversion"))
}

/// Convert a TOML Value to a Rune Value.
fn toml_value_to_rune_value(value: &toml::Value) -> Result<rune::Value, String> {
    match value {
        toml::Value::String(s) => {
            rune::to_value(s.clone()).map_err(|e| format!("String conversion failed: {}", e))
        }
        toml::Value::Integer(i) => {
            rune::to_value(*i).map_err(|e| format!("Integer conversion failed: {}", e))
        }
        toml::Value::Float(f) => {
            rune::to_value(*f).map_err(|e| format!("Float conversion failed: {}", e))
        }
        toml::Value::Boolean(b) => {
            rune::to_value(*b).map_err(|e| format!("Boolean conversion failed: {}", e))
        }
        toml::Value::Array(arr) => {
            let mut result: Vec<rune::Value> = Vec::new();
            for item in arr {
                result.push(toml_value_to_rune_value(item)?);
            }
            rune::to_value(result).map_err(|e| format!("Array conversion failed: {}", e))
        }
        toml::Value::Table(table) => {
            let mut result = std::collections::HashMap::new();
            for (key, val) in table {
                result.insert(key.clone(), toml_value_to_rune_value(val)?);
            }
            rune::to_value(result).map_err(|e| format!("Table conversion failed: {}", e))
        }
        toml::Value::Datetime(dt) => {
            rune::to_value(dt.to_string()).map_err(|e| format!("Datetime conversion failed: {}", e))
        }
    }
}

/// Read text file as string.
///
/// # Example (Rune code)
///
/// ```rune
/// let content = read_text_file("/path/to/file.txt")?;
/// ```
fn read_text_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

/// Write text to file.
///
/// # Example (Rune code)
///
/// ```rune
/// write_text_file("/path/to/file.txt", "content")?;
/// ```
fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write file '{}': {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_persistence_functions() {
        let mut module = Module::with_crate("test").unwrap();
        let result = register_persistence_functions(&mut module);
        assert!(result.is_ok());
    }
}
