//! PastaEngine - Main entry point for executing Pasta DSL scripts.
//!
//! This module provides the integrated engine that combines parsing, transpiling,
//! and runtime execution to provide a high-level API for running Pasta scripts.

use crate::{
    error::{PastaError, Result, Transpiler2Pass},
    ir::ScriptEvent,
    loader::{DirectoryLoader, ErrorLogWriter},
    parser2::{self, FileItem, PastaFile},
    registry::{SceneRegistry, WordDefRegistry},
    runtime::{DefaultRandomSelector, RandomSelector, SceneTable, WordTable},
    transpiler2::Transpiler2,
};
use rune::{Context, Vm};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Main Pasta script engine.
///
/// This engine integrates all layers of the Pasta stack:
/// - Parser2: Parses Pasta DSL to AST using `grammar.pest`
/// - Transpiler2: Converts AST to Rune source code (2-pass strategy)
/// - Runtime: Executes Rune code with generators
///
/// # Instance Independence
///
/// Each PastaEngine instance is completely independent and owns all its data,
/// including its own parse cache. Multiple engine instances can coexist safely
/// in the same process or across threads.
pub struct PastaEngine {
    /// The compiled Rune unit.
    unit: Arc<rune::Unit>,
    /// The Rune runtime context.
    runtime: Arc<rune::runtime::RuntimeContext>,
    /// Persistence directory path (optional).
    persistence_path: Option<PathBuf>,
}

impl PastaEngine {
    /// Create a new PastaEngine from script and persistence directories.
    ///
    /// This is the primary constructor for production use. It loads all `.pasta` files
    /// from the `dic/` directory and `main.rn` from the script root, following
    /// areka-P0-script-engine conventions.
    ///
    /// # Directory Structure
    ///
    /// ```text
    /// script_root/
    ///   ├── main.rn             # Rune entry point
    ///   └── dic/                # Pasta scripts
    ///       ├── *.pasta
    ///       └── ...
    ///
    /// persistence_root/
    ///   ├── variables.toml      # Persisted variables
    ///   └── ...                 # Other runtime data
    /// ```
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory (must be absolute path)
    /// * `persistence_root` - Persistence root directory (absolute or relative)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Script path is not absolute
    /// - Script directory does not exist or is not readable
    /// - Persistence directory does not exist
    /// - `dic/` directory not found
    /// - `main.rn` not found
    /// - Parse errors in `.pasta` files
    /// - Rune compilation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pasta::PastaEngine;
    /// use std::path::Path;
    ///
    /// let engine = PastaEngine::new(
    ///     Path::new("/path/to/script_root"),
    ///     Path::new("/path/to/persistence_root")
    /// )?;
    /// # Ok::<(), pasta::PastaError>(())
    /// ```
    pub fn new(script_root: impl AsRef<Path>, persistence_root: impl AsRef<Path>) -> Result<Self> {
        Self::with_random_selector(
            script_root,
            persistence_root,
            Box::new(DefaultRandomSelector::new()),
        )
    }

    /// Create a new PastaEngine with a custom random selector.
    ///
    /// This is primarily useful for testing with deterministic random selection.
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory (must be absolute path)
    /// * `persistence_root` - Persistence root directory (absolute or relative)
    /// * `random_selector` - Custom random selector implementation
    ///
    /// # Errors
    ///
    /// Same as `new()`
    pub fn with_random_selector(
        script_root: impl AsRef<Path>,
        persistence_root: impl AsRef<Path>,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self> {
        let path = script_root.as_ref();

        // Step 1: Load files from directory
        let loaded = DirectoryLoader::load(path)?;

        // Step 2: Parse all .pasta files using parser2 (collect errors)
        let mut parsed_files: Vec<PastaFile> = Vec::new();
        let mut parse_errors = Vec::new();

        for pasta_file in &loaded.pasta_files {
            match parser2::parse_file(pasta_file) {
                Ok(ast) => {
                    parsed_files.push(ast);
                }
                Err(e) => {
                    // Collect parse errors, fail-fast on other errors
                    if let Some(parse_err) = Option::from(&e) {
                        parse_errors.push(parse_err);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Step 3: If parse errors exist, log and return error
        if !parse_errors.is_empty() {
            ErrorLogWriter::log(&loaded.script_root, &parse_errors);
            return Err(PastaError::MultipleParseErrors {
                errors: parse_errors,
            });
        }

        // Step 4: Merge all ASTs into items-based structure
        let mut all_items: Vec<FileItem> = Vec::new();
        for file in &parsed_files {
            all_items.extend(file.items.iter().cloned());
        }

        // Create merged PastaFile
        let merged_file = PastaFile {
            path: loaded.script_root.clone().into(),
            items: all_items,
            span: parser2::Span::default(), // Merged file has no meaningful span
        };

        // Step 5: Transpile using 2-pass strategy (transpiler2)
        // Prepare registries and buffer
        let mut scene_registry = SceneRegistry::new();
        let mut word_def_registry = WordDefRegistry::new();
        let mut buffer: Vec<u8> = Vec::new();

        // Pass 1: Register definitions and generate modules
        Transpiler2::transpile_pass1(
            &merged_file,
            &mut scene_registry,
            &mut word_def_registry,
            &mut buffer,
        )
        .map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass1))?;

        // Pass 2: Generate selectors
        Transpiler2::transpile_pass2(&scene_registry, &mut buffer)
            .map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass2))?;

        // Convert buffer to String
        let rune_source = String::from_utf8(buffer)
            .map_err(|e| PastaError::RuneRuntimeError(format!("UTF-8 conversion error: {}", e)))?;

        #[cfg(debug_assertions)]
        {
            eprintln!("=== Generated Rune Source (from directory) ===");
            eprintln!("{}", rune_source);
            eprintln!("===============================================");
        }

        // Step 6: Build scene table for Rune module (ownership moved to closure)
        let scene_table = SceneTable::from_scene_registry(scene_registry, random_selector)?;

        // Step 7: Build word table for word expansion (with its own random selector)
        let word_random_selector: Box<dyn RandomSelector> = Box::new(DefaultRandomSelector::new());
        let word_table = WordTable::from_word_def_registry(word_def_registry, word_random_selector);

        // Step 8: Build Rune sources with main.rn
        let mut context = Context::with_default_modules().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create Rune context: {}", e))
        })?;

        // Install standard library with scene table and word table (moved into module)
        context
            .install(
                crate::stdlib::create_module(scene_table, word_table).map_err(|e| {
                    PastaError::RuneRuntimeError(format!("Failed to install stdlib: {}", e))
                })?,
            )
            .map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to install context: {}", e))
            })?;

        let runtime = Arc::new(context.runtime().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create runtime: {}", e))
        })?);

        let mut sources = rune::Sources::new();

        // Combine main.rn and transpiled code into a single source
        // This is necessary because Rune's `use crate::actors::*;` needs to reference
        // the actors module defined in the same compilation unit
        let main_rn_content = std::fs::read_to_string(&loaded.main_rune)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to read main.rn: {}", e)))?;
        let combined_source = format!("{}\n\n{}", main_rn_content, rune_source);

        // Add combined source
        sources
            .insert(rune::Source::new("entry", &combined_source).map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to create source: {}", e))
            })?)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to insert source: {}", e)))?;

        // Step 8: Compile Rune code
        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()
            .map_err(|e| PastaError::RuneCompileError(format!("Failed to compile Rune: {}", e)))?;

        // Step 9: Validate persistence path
        let validated_persistence_path =
            Self::validate_persistence_path(persistence_root.as_ref())?;

        // Step 10: Construct engine
        Ok(Self {
            unit: Arc::new(unit),
            runtime,
            persistence_path: Some(validated_persistence_path),
        })
    }

    /// Validate and canonicalize the persistence path.
    fn validate_persistence_path(path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            tracing::error!(
                path = %path.display(),
                error = "Directory not found",
                "[PastaEngine::validate_persistence_path] Persistence directory does not exist"
            );
            return Err(PastaError::PersistenceDirectoryNotFound {
                path: path.display().to_string(),
            });
        }

        if !path.is_dir() {
            tracing::error!(
                path = %path.display(),
                error = "Not a directory",
                "[PastaEngine::validate_persistence_path] Path is not a directory"
            );
            return Err(PastaError::InvalidPersistencePath {
                path: path.display().to_string(),
            });
        }

        let canonical = path.canonicalize().map_err(|e| {
            tracing::error!(
                path = %path.display(),
                error = %e,
                "[PastaEngine::validate_persistence_path] Failed to canonicalize path"
            );
            PastaError::InvalidPersistencePath {
                path: path.display().to_string(),
            }
        })?;

        tracing::info!(
            path = %canonical.display(),
            "[PastaEngine::validate_persistence_path] Persistence path configured"
        );

        Ok(canonical)
    }

    /// Build execution context with persistence path.
    fn build_execution_context(&self) -> Result<rune::Value> {
        let mut ctx = HashMap::new();

        let path_str = if let Some(ref path) = self.persistence_path {
            path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        ctx.insert("persistence_path".to_string(), path_str.clone());

        tracing::debug!(
            persistence_path = %path_str,
            "[PastaEngine::build_execution_context] Building execution context"
        );

        rune::to_value(ctx)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to build context: {}", e)))
    }

    /// Execute a scene and return all events synchronously.
    ///
    /// This looks up the scene (with optional attribute filters), selects one
    /// if multiple scenes match, and executes it to completion, returning all events.
    ///
    /// # Arguments
    ///
    /// * `label_name` - The name of the scene to execute
    ///
    /// # Returns
    ///
    /// A vector of all `ScriptEvent`s generated by the scene.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The scene is not found
    /// - No scenes match the filters
    /// - A runtime error occurs during execution
    pub fn execute_label(&mut self, label_name: &str) -> Result<Vec<ScriptEvent>> {
        self.execute_label_with_filters(label_name, &HashMap::new())
    }

    /// Execute a scene with attribute filters and return all events.
    ///
    /// This is the full version of `execute_label` that accepts filters.
    ///
    /// Note: After pasta-label-resolution-runtime implementation, scene resolution
    /// is handled by Rune's label_selector at runtime. This method attempts to
    /// execute the most common scene format.
    pub fn execute_label_with_filters(
        &mut self,
        label_name: &str,
        _filters: &HashMap<String, String>,
    ) -> Result<Vec<ScriptEvent>> {
        // Attempt to execute with common naming pattern: {label_name}_1::__start__
        let fn_name = format!("{}_1::__start__", label_name);

        // Create a new VM for this execution
        let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());

        // Split fn_name into path components for Rune
        // fn_name format: "module_name::function_name"
        // Rune expects: ["module_name", "function_name"]
        let parts: Vec<&str> = fn_name.split("::").collect();
        let hash = rune::Hash::type_hash(&parts);

        // Build execution context
        let context = self.build_execution_context()?;

        // Execute and get a generator
        // Note: Generated functions expect (ctx, args) signature
        // args is currently an empty array for future argument support
        let args = rune::to_value(Vec::<rune::Value>::new()).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create args array: {}", e))
        })?;

        let execution = vm.execute(hash, (context, args)).map_err(|e| {
            // Convert function not found errors to SceneNotFound
            let err_msg = format!("{:?}", e);
            if err_msg.contains("MissingEntry") || err_msg.contains("MissingFunction") {
                PastaError::SceneNotFound {
                    scene: label_name.to_string(),
                }
            } else {
                PastaError::VmError(e)
            }
        })?;

        let mut generator = execution.into_generator();

        // Collect all yielded events
        let mut events = Vec::new();
        let unit_value = rune::to_value(()).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create unit value: {}", e))
        })?;

        loop {
            match generator.resume(unit_value.clone()) {
                rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                    let event: ScriptEvent = rune::from_value(value).map_err(|e| {
                        PastaError::RuneRuntimeError(format!(
                            "Failed to convert yielded value: {}",
                            e
                        ))
                    })?;
                    events.push(event);
                }
                rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                    break;
                }
                rune::runtime::VmResult::Err(e) => {
                    return Err(PastaError::VmError(e));
                }
            }
        }

        Ok(events)
    }

    /// Fire a custom event and return the FireEvent script event.
    ///
    /// This is a convenience method that creates a `ScriptEvent::FireEvent`
    /// to be yielded by scripts.
    ///
    /// # Arguments
    ///
    /// * `event_name` - The name of the event to fire
    /// * `params` - Key-value parameters for the event
    ///
    /// # Returns
    ///
    /// A `ScriptEvent::FireEvent` that can be yielded or processed.
    ///
    /// # Example
    ///
    /// This would typically be called from within Rune scripts via the
    /// standard library `fire_event` function.
    pub fn create_fire_event(event_name: String, params: Vec<(String, String)>) -> ScriptEvent {
        ScriptEvent::FireEvent { event_name, params }
    }
}

impl Drop for PastaEngine {
    /// Persist engine state on destruction.
    ///
    /// This implementation saves:
    /// - Global variables (if variable manager is added in future)
    /// - scene execution history and caches
    ///
    /// Currently, this is a placeholder for Task 5.5 implementation.
    /// Full persistence will be added when VariableManager is integrated
    /// into PastaEngine.
    fn drop(&mut self) {
        // TODO: Persist global variables when VariableManager is integrated
        // self.variables.save_to_disk().ok();

        // TODO: Persist scene execution history/cache
        // self.scene_table.save_cache().ok();

        // For now, we just log that the engine is being dropped
        #[cfg(debug_assertions)]
        {
            eprintln!("[PastaEngine] Dropping engine (persistence not yet implemented)");
        }
    }
}
