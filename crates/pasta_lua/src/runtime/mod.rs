//! Pasta Lua Runtime - Lua VM host for Pasta scripts.
//!
//! This module provides the PastaLuaRuntime struct which hosts a Lua VM
//! and integrates pasta modules for script execution.
//!
//! # Example
//!
//! ```rust,ignore
//! use pasta_lua::{LuaTranspiler, PastaLuaRuntime};
//!
//! let transpiler = LuaTranspiler::default();
//! let mut output = Vec::new();
//! let context = transpiler.transpile(&pasta_file, &mut output)?;
//!
//! let runtime = PastaLuaRuntime::new(context)?;
//! let result = runtime.exec("return 1 + 1")?;
//! ```

use crate::context::TranspileContext;
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::path::Path;
use std::sync::Arc;

/// Pasta Lua Runtime - hosts a Lua VM with pasta modules.
///
/// Each instance owns an independent Lua VM and SearchContext.
/// Multiple instances can coexist without interference.
pub struct PastaLuaRuntime {
    lua: Lua,
}

impl PastaLuaRuntime {
    /// Create a new runtime from a TranspileContext.
    ///
    /// Initializes a Lua VM and registers the `@pasta_search` module
    /// with the scene and word registries from the context.
    ///
    /// # Arguments
    /// * `context` - TranspileContext from LuaTranspiler::transpile()
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized successfully
    /// * `Err(e)` - Lua VM or module registration failed
    pub fn new(context: TranspileContext) -> LuaResult<Self> {
        let lua = Lua::new();

        // Extract registries from context
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;

        // Register @pasta_search module
        crate::search::register(&lua, scene_registry, word_registry)?;

        Ok(Self { lua })
    }

    /// Execute a Lua script string.
    ///
    /// # Arguments
    /// * `script` - Lua code to execute
    ///
    /// # Returns
    /// * `Ok(Value)` - Execution result
    /// * `Err(e)` - Execution error
    pub fn exec(&self, script: &str) -> LuaResult<Value> {
        self.lua.load(script).eval()
    }

    /// Execute a Lua script from a file.
    ///
    /// # Arguments
    /// * `path` - Path to the Lua script file
    ///
    /// # Returns
    /// * `Ok(Value)` - Execution result
    /// * `Err(e)` - File read or execution error
    pub fn exec_file(&self, path: &Path) -> LuaResult<Value> {
        let script =
            std::fs::read_to_string(path).map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
        self.exec(&script)
    }

    /// Get a reference to the internal Lua instance.
    ///
    /// This allows advanced operations on the Lua VM.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Register a custom module with the runtime.
    ///
    /// # Arguments
    /// * `name` - Module name (e.g., "@my_module")
    /// * `module` - Module table
    ///
    /// # Returns
    /// * `Ok(())` - Module registered successfully
    /// * `Err(e)` - Registration failed
    pub fn register_module(&self, name: &str, module: Table) -> LuaResult<()> {
        let package: Table = self.lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set(name, module)?;
        Ok(())
    }
}
