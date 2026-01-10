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
use mlua::{Lua, Result as LuaResult, StdLib, Table, Value};
use std::path::Path;
use std::sync::Arc;

/// Configuration for which standard libraries to enable in the Lua runtime.
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    /// Enable Lua standard libraries (base, table, string, math, etc.)
    /// Default: true
    pub enable_std_libs: bool,
    /// Enable mlua-stdlib assertions module (@assertions)
    /// Provides assertion functions for testing and validation.
    /// Default: true
    pub enable_assertions: bool,
    /// Enable mlua-stdlib testing module (@testing)
    /// Provides a testing framework with hooks and reporting.
    /// Default: true
    pub enable_testing: bool,
    /// Enable mlua-stdlib env module (@env)
    /// Provides environment variable and filesystem path access.
    /// Default: false (security-sensitive, opt-in)
    pub enable_env: bool,
    /// Enable mlua-stdlib regex module (@regex)
    /// Default: true
    pub enable_regex: bool,
    /// Enable mlua-stdlib json module (@json)
    /// Default: true
    pub enable_json: bool,
    /// Enable mlua-stdlib yaml module (@yaml)
    /// Default: true
    pub enable_yaml: bool,
}

impl RuntimeConfig {
    /// Create a new configuration with all safe features enabled (default).
    ///
    /// Note: `enable_env` is disabled by default for security reasons.
    pub fn new() -> Self {
        Self {
            enable_std_libs: true,
            enable_assertions: true,
            enable_testing: true,
            enable_env: false, // Disabled by default for security
            enable_regex: true,
            enable_json: true,
            enable_yaml: true,
        }
    }

    /// Create a configuration with all features enabled, including security-sensitive ones.
    pub fn full() -> Self {
        Self {
            enable_std_libs: true,
            enable_assertions: true,
            enable_testing: true,
            enable_env: true,
            enable_regex: true,
            enable_json: true,
            enable_yaml: true,
        }
    }

    /// Create a minimal configuration with only pasta modules.
    pub fn minimal() -> Self {
        Self {
            enable_std_libs: false,
            enable_assertions: false,
            enable_testing: false,
            enable_env: false,
            enable_regex: false,
            enable_json: false,
            enable_yaml: false,
        }
    }
}

/// Pasta Lua Runtime - hosts a Lua VM with pasta modules.
///
/// Each instance owns an independent Lua VM and SearchContext.
/// Multiple instances can coexist without interference.
pub struct PastaLuaRuntime {
    lua: Lua,
}

impl PastaLuaRuntime {
    /// Create a new runtime from a TranspileContext with default configuration.
    ///
    /// Initializes a Lua VM with standard libraries enabled and registers:
    /// - `@pasta_search` module with scene and word registries
    /// - `@assertions` module for testing and validation
    /// - `@testing` module for testing framework with hooks and reporting
    /// - `@regex` module for regular expression support
    /// - `@json` module for JSON encoding/decoding
    /// - `@yaml` module for YAML encoding/decoding
    ///
    /// Note: `@env` module is disabled by default for security reasons.
    /// Use `RuntimeConfig::full()` or enable it explicitly to access environment variables.
    ///
    /// # Arguments
    /// * `context` - TranspileContext from LuaTranspiler::transpile()
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized successfully
    /// * `Err(e)` - Lua VM or module registration failed
    pub fn new(context: TranspileContext) -> LuaResult<Self> {
        Self::with_config(context, RuntimeConfig::new())
    }

    /// Create a new runtime from a TranspileContext with custom configuration.
    ///
    /// # Arguments
    /// * `context` - TranspileContext from LuaTranspiler::transpile()
    /// * `config` - Runtime configuration for library loading
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized successfully
    /// * `Err(e)` - Lua VM or module registration failed
    pub fn with_config(context: TranspileContext, config: RuntimeConfig) -> LuaResult<Self> {
        // Create Lua VM with appropriate standard libraries
        let lua = if config.enable_std_libs {
            // Load all safe standard libraries (excluding debug and ffi)
            // SAFETY: ALL_SAFE only loads safe standard libraries
            unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }
        } else {
            Lua::new()
        };

        // Extract registries from context
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;

        // Register @pasta_search module
        crate::search::register(&lua, scene_registry, word_registry)?;

        // Register mlua-stdlib core modules based on configuration
        if config.enable_assertions {
            mlua_stdlib::assertions::register(&lua, None)?;
        }
        if config.enable_testing {
            mlua_stdlib::testing::register(&lua, None)?;
        }
        if config.enable_env {
            mlua_stdlib::env::register(&lua, None)?;
        }

        // Register mlua-stdlib feature-gated modules
        if config.enable_regex {
            mlua_stdlib::regex::register(&lua, None)?;
        }
        if config.enable_json {
            mlua_stdlib::json::register(&lua, None)?;
        }
        if config.enable_yaml {
            mlua_stdlib::yaml::register(&lua, None)?;
        }

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
