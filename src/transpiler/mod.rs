//! Transpiler module for converting Pasta AST to Rune source code.
//!
//! This module converts the Pasta AST into Rune source code that can be executed
//! by the Rune VM to generate ScriptEvent IR.

mod label_registry;

pub use label_registry::{LabelInfo, LabelRegistry};

use crate::{
    Argument, BinOp, Expr, FunctionScope, JumpTarget, LabelDef, LabelScope, Literal, PastaError,
    PastaFile, SpeechPart, Statement, VarScope,
};
use std::collections::HashMap;

/// Transpile context that holds scope information during transpilation.
#[derive(Clone)]
pub struct TranspileContext {
    /// List of local function names defined in the current label
    local_functions: Vec<String>,
    /// List of global function names (standard library + user-defined)
    global_functions: Vec<String>,
}

impl TranspileContext {
    /// Create a new transpile context.
    pub fn new() -> Self {
        Self {
            local_functions: Vec::new(),
            global_functions: Self::default_global_functions(),
        }
    }

    /// Get default global functions (standard library).
    fn default_global_functions() -> Vec<String> {
        vec![
            "emit_text".to_string(),
            "emit_sakura_script".to_string(),
            "change_speaker".to_string(),
            "change_surface".to_string(),
            "wait".to_string(),
            "begin_sync".to_string(),
            "sync_point".to_string(),
            "end_sync".to_string(),
            "fire_event".to_string(),
        ]
    }

    /// Set local functions for the current label scope.
    pub fn set_local_functions(&mut self, functions: Vec<String>) {
        self.local_functions = functions;
    }

    /// Add a global function to the list.
    pub fn add_global_function(&mut self, name: String) {
        if !self.global_functions.contains(&name) {
            self.global_functions.push(name);
        }
    }

    /// Resolve function name with scope rules (local→global search).
    ///
    /// Note: If the function is not found in tracked scopes, it is still returned as-is
    /// because it might be defined in a Rune block that we haven't parsed. The Rune
    /// runtime will handle the error if the function truly doesn't exist.
    pub fn resolve_function(
        &self,
        func_name: &str,
        scope: FunctionScope,
    ) -> Result<String, PastaError> {
        match scope {
            FunctionScope::Auto => {
                // 1. Search local functions first
                if self.local_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                }
                // 2. Search global functions
                else if self.global_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                } else {
                    // 3. Function not in tracked scopes, but might be defined in Rune block
                    // Allow it and let Rune runtime handle errors
                    Ok(func_name.to_string())
                }
            }
            FunctionScope::GlobalOnly => {
                // Search global functions only
                if self.global_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                } else {
                    // Not in global scope, but might be in Rune block
                    // For GlobalOnly, we're stricter - return error
                    Err(PastaError::function_not_found(func_name))
                }
            }
        }
    }
}

impl Default for TranspileContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Transpiler that converts Pasta AST to Rune source code.
pub struct Transpiler;

impl Transpiler {
    /// Transpile a Pasta file AST to Rune source code (Pass 1).
    ///
    /// This performs Pass 1 of the two-pass transpilation strategy:
    /// - Registers all labels in the LabelRegistry
    /// - Generates Rune modules for each global label
    /// - Generates function code for labels
    ///
    /// # Arguments
    ///
    /// * `file` - The parsed Pasta file AST
    /// * `registry` - The label registry for tracking labels across files
    /// * `writer` - Output destination implementing Write trait
    ///
    /// # Notes
    ///
    /// - This can be called multiple times for multiple Pasta files
    /// - Each call accumulates labels in the registry
    /// - The output does NOT include `mod pasta {}` (generated in Pass 2)
    pub fn transpile_pass1<W: std::io::Write>(
        file: &PastaFile,
        registry: &mut LabelRegistry,
        writer: &mut W,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        // Note: Top-level use statement is not needed
        // Each module has its own use statements

        // Register all labels and generate modules
        for label in &file.labels {
            Self::transpile_global_label(label, registry, writer)?;
        }

        Ok(())
    }

    /// Transpile Pass 2: Generate `mod __pasta_trans2__` and `mod pasta` modules.
    ///
    /// This performs Pass 2 of the two-pass transpilation strategy:
    /// - Generates `mod __pasta_trans2__` with label_selector() function
    /// - Generates `mod pasta` with jump() and call() wrapper functions
    /// - Creates ID→function path mapping from the registry
    ///
    /// # Arguments
    ///
    /// * `registry` - The label registry containing all registered labels
    /// * `writer` - Output destination implementing Write trait
    ///
    /// # Notes
    ///
    /// - This should be called ONCE after all Pass 1 calls are complete
    /// - The output is appended to the Pass 1 output
    pub fn transpile_pass2<W: std::io::Write>(
        registry: &LabelRegistry,
        writer: &mut W,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        // Generate __pasta_trans2__ module with label_selector function
        writeln!(writer, "pub mod __pasta_trans2__ {{")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "    use pasta_stdlib::*;")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "    pub fn label_selector(label, filters) {{")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(
            writer,
            "        let id = pasta_stdlib::select_label_to_id(label, filters);"
        )
        .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "        match id {{").map_err(|e| PastaError::io_error(e.to_string()))?;

        for label in registry.all_labels() {
            writeln!(writer, "            {} => {},", label.id, label.fn_path)
                .map_err(|e| PastaError::io_error(e.to_string()))?;
        }

        writeln!(writer, "            _ => |_ctx, _args| {{ yield pasta_stdlib::Error(`ラベルID ${{id}} が見つかりませんでした。`); }},")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "        }}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "    }}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "}}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;

        // Generate pasta module with wrapper functions
        writeln!(writer, "pub mod pasta {{").map_err(|e| PastaError::io_error(e.to_string()))?;
        // Phase 1 (REQ-BC-1): Jump function removed - use call() instead
        // writeln!(writer, "    pub fn jump(ctx, label, filters, args) {{ ... }}")?;

        writeln!(writer, "    pub fn call(ctx, label, filters, args) {{")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(
            writer,
            "        let func = crate::__pasta_trans2__::label_selector(label, filters);"
        )
        .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "    }}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "}}").map_err(|e| PastaError::io_error(e.to_string()))?;

        Ok(())
    }

    /// Helper function for testing: transpile to string (single file only).
    ///
    /// # Warning
    ///
    /// This is a convenience function for testing only. Do NOT use in production code.
    /// It only handles a single PastaFile and doesn't support multiple files.
    #[doc(hidden)]
    pub fn transpile_to_string(file: &PastaFile) -> Result<String, PastaError> {
        let mut registry = LabelRegistry::new();
        let mut output = Vec::new();

        Self::transpile_pass1(file, &mut registry, &mut output)?;
        Self::transpile_pass2(&registry, &mut output)?;

        String::from_utf8(output).map_err(|e| PastaError::io_error(e.to_string()))
    }

    /// Transpile and return both the Rune source and the label registry.
    ///
    /// This is used by PastaEngine to get the label registry that matches
    /// the generated Rune source code.
    pub fn transpile_with_registry(
        file: &PastaFile,
    ) -> Result<(String, LabelRegistry), PastaError> {
        let mut registry = LabelRegistry::new();
        let mut output = Vec::new();

        Self::transpile_pass1(file, &mut registry, &mut output)?;
        Self::transpile_pass2(&registry, &mut output)?;

        let source = String::from_utf8(output).map_err(|e| PastaError::io_error(e.to_string()))?;
        Ok((source, registry))
    }

    /// Transpile a global label and register it.
    fn transpile_global_label<W: std::io::Write>(
        label: &LabelDef,
        registry: &mut LabelRegistry,
        writer: &mut W,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        // Register the global label
        let (_id, counter) = registry.register_global(&label.name, HashMap::new());
        let module_name = format!("{}_{}", Self::sanitize_identifier(&label.name), counter);

        // Generate module
        writeln!(writer, "pub mod {} {{", module_name)
            .map_err(|e| PastaError::io_error(e.to_string()))?;

        // Import pasta_stdlib functions and actor definitions into module scope
        writeln!(writer, "    use pasta_stdlib::*;")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer, "    use crate::actors::*;")
            .map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;

        // Generate __start__ function
        writeln!(writer, "    pub fn __start__(ctx, args) {{")
            .map_err(|e| PastaError::io_error(e.to_string()))?;

        // Transpile statements before first local label
        for stmt in &label.statements {
            Self::transpile_statement_to_writer(writer, stmt)?;
        }

        writeln!(writer, "    }}").map_err(|e| PastaError::io_error(e.to_string()))?;

        // Register and generate local labels
        for local_label in &label.local_labels {
            Self::transpile_local_label(local_label, &label.name, counter, registry, writer)?;
        }

        writeln!(writer, "}}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;

        Ok(())
    }

    /// Transpile a local label and register it.
    fn transpile_local_label<W: std::io::Write>(
        label: &LabelDef,
        parent_name: &str,
        parent_counter: usize,
        registry: &mut LabelRegistry,
        writer: &mut W,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        // Register the local label
        let (_id, counter) =
            registry.register_local(&label.name, parent_name, parent_counter, HashMap::new());
        let fn_name = format!("{}_{}", Self::sanitize_identifier(&label.name), counter);

        // Generate function
        writeln!(writer, "    pub fn {}(ctx, args) {{", fn_name)
            .map_err(|e| PastaError::io_error(e.to_string()))?;

        // Transpile statements
        for stmt in &label.statements {
            Self::transpile_statement_to_writer(writer, stmt)?;
        }

        writeln!(writer, "    }}").map_err(|e| PastaError::io_error(e.to_string()))?;
        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;

        Ok(())
    }

    /// Transpile a statement to a writer.
    fn transpile_statement_to_writer<W: std::io::Write>(
        writer: &mut W,
        stmt: &Statement,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        let context = TranspileContext::new();

        match stmt {
            Statement::Speech {
                speaker,
                content,
                span: _,
            } => {
                // Generate speaker change (use variable reference to actor object)
                writeln!(writer, "        ctx.actor = {};", speaker)
                    .map_err(|e| PastaError::io_error(e.to_string()))?;
                writeln!(writer, "        yield Actor(ctx.actor.name);")
                    .map_err(|e| PastaError::io_error(e.to_string()))?;

                // Generate talk content
                for part in content {
                    Self::transpile_speech_part_to_writer(writer, part, &context)?;
                }
            }
            Statement::Call {
                target,
                filters,
                args,
                span: _,
            } => {
                // Generate call statement: for a in crate::pasta::call(ctx, "label", #{}, [args]) { yield a; }
                let search_key = Self::transpile_jump_target_to_search_key(target);
                let args_str = Self::transpile_exprs_to_args(args, &context)?;
                let filters_str = Self::transpile_attributes_to_map(filters);
                writeln!(
                    writer,
                    "        for a in crate::pasta::call(ctx, \"{}\", {}, [{}]) {{ yield a; }}",
                    search_key, filters_str, args_str
                )
                .map_err(|e| PastaError::io_error(e.to_string()))?;
            }
            Statement::VarAssign {
                name,
                scope,
                value,
                span: _,
            } => {
                let value_expr = Self::transpile_expr(value, &context)?;
                match scope {
                    VarScope::Local => {
                        writeln!(writer, "        let {} = {};", name, value_expr)
                            .map_err(|e| PastaError::io_error(e.to_string()))?;
                    }
                    VarScope::Global => {
                        writeln!(writer, "        ctx.var.{} = {};", name, value_expr)
                            .map_err(|e| PastaError::io_error(e.to_string()))?;
                    }
                }
            }
            Statement::RuneBlock { content, span: _ } => {
                // Output the Rune code inline with proper indentation
                for line in content.lines() {
                    if line.trim().is_empty() {
                        writeln!(writer).map_err(|e| PastaError::io_error(e.to_string()))?;
                    } else {
                        writeln!(writer, "        {}", line.trim_start())
                            .map_err(|e| PastaError::io_error(e.to_string()))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Transpile a speech part to writer.
    fn transpile_speech_part_to_writer<W: std::io::Write>(
        writer: &mut W,
        part: &SpeechPart,
        context: &TranspileContext,
    ) -> Result<(), PastaError> {
        #[allow(unused_imports)]
        use std::io::Write;

        match part {
            SpeechPart::Text(text) => {
                writeln!(
                    writer,
                    "        yield Talk(\"{}\");",
                    Self::escape_string(text)
                )
                .map_err(|e| PastaError::io_error(e.to_string()))?;
            }
            SpeechPart::VarRef(var_name) => {
                writeln!(writer, "        yield Talk(`${{ctx.var.{}}}`);", var_name)
                    .map_err(|e| PastaError::io_error(e.to_string()))?;
            }
            SpeechPart::FuncCall {
                name,
                args,
                scope: _,
            } => {
                // Word expansion: yield Talk(pasta_stdlib::word(ctx, "word", []))
                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => Self::transpile_expr(expr, context),
                        Argument::Named { name, value } => Ok(format!(
                            "{}={}",
                            name,
                            Self::transpile_expr(value, context)?
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                writeln!(
                    writer,
                    "        yield Talk(pasta_stdlib::word(ctx, \"{}\", [{}]));",
                    name, args_str
                )
                .map_err(|e| PastaError::io_error(e.to_string()))?;
            }
            SpeechPart::SakuraScript(script) => {
                writeln!(
                    writer,
                    "        yield emit_sakura_script(\"{}\");",
                    Self::escape_string(script)
                )
                .map_err(|e| PastaError::io_error(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Transpile jump target to search key.
    fn transpile_jump_target_to_search_key(target: &JumpTarget) -> String {
        match target {
            JumpTarget::Local(name) => name.clone(),
            JumpTarget::Global(name) => name.clone(),
            JumpTarget::LongJump { global, local } => format!("{}::{}", global, local),
            JumpTarget::Dynamic(var_name) => format!("@{}", var_name),
        }
    }

    /// Transpile expressions to argument list string.
    fn transpile_exprs_to_args(
        exprs: &[Expr],
        context: &TranspileContext,
    ) -> Result<String, PastaError> {
        exprs
            .iter()
            .map(|expr| Self::transpile_expr(expr, context))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.join(", "))
    }

    /// Transpile attributes to Rune map syntax.
    fn transpile_attributes_to_map(_attrs: &[crate::Attribute]) -> String {
        // P0: filters are not used, always return empty map
        "#{}".to_string()
    }

    /// Transpile a Pasta file AST to Rune source code (Legacy single-pass).
    ///
    /// # Deprecated
    ///
    /// This is the old single-pass transpiler. Use `transpile_pass1` and `transpile_pass2` instead.
    pub fn transpile(file: &PastaFile) -> Result<String, PastaError> {
        let mut output = String::new();

        // Add imports for standard library functions
        output.push_str("use pasta_stdlib::*;\n\n");

        // Create transpile context
        let mut context = TranspileContext::new();

        // Collect all global label names as global functions
        for label in &file.labels {
            let fn_name = Self::sanitize_identifier(&label.name);
            context.add_global_function(fn_name);
        }

        // Track label counters to generate unique function names for duplicates
        let mut label_counters: HashMap<String, usize> = HashMap::new();

        // Transpile each global label
        for label in &file.labels {
            let counter = label_counters.entry(label.name.clone()).or_insert(0);
            Self::transpile_label_with_counter(&mut output, label, None, *counter, &context)?;
            *counter += 1;
        }

        Ok(output)
    }

    /// Transpile a single label definition to a Rune function with a counter for duplicates.
    fn transpile_label_with_counter(
        output: &mut String,
        label: &LabelDef,
        parent_name: Option<&str>,
        counter: usize,
        global_context: &TranspileContext,
    ) -> Result<(), PastaError> {
        let fn_name = Self::label_to_fn_name_with_counter(label, parent_name, counter);

        // Create a context for this label with local functions
        let mut label_context = global_context.clone();

        // Collect local function names from Rune blocks (TODO: parse Rune blocks to extract function names)
        // For now, local functions would be extracted from RuneBlock statements
        // This is a placeholder - actual implementation would need to parse Rune code
        let local_functions = Vec::new(); // TODO: Extract from label.statements
        label_context.set_local_functions(local_functions);

        // Function signature - generators don't need async keyword in Rune
        output.push_str(&format!("pub fn {}(ctx) {{\n", fn_name));

        // Transpile statements
        for stmt in &label.statements {
            Self::transpile_statement(output, stmt, &label_context)?;
        }

        // Transpile local labels (with their own counter tracking)
        let mut local_counters: HashMap<String, usize> = HashMap::new();
        for local_label in &label.local_labels {
            let counter = local_counters.entry(local_label.name.clone()).or_insert(0);
            Self::transpile_label_with_counter(
                output,
                local_label,
                Some(&label.name),
                *counter,
                global_context,
            )?;
            *counter += 1;
        }

        output.push_str("}\n\n");
        Ok(())
    }

    /// Generate a function name from a label definition with counter for duplicates.
    fn label_to_fn_name_with_counter(
        label: &LabelDef,
        parent_name: Option<&str>,
        counter: usize,
    ) -> String {
        let base_name = match label.scope {
            LabelScope::Global => {
                // Global labels use their name directly
                Self::sanitize_identifier(&label.name)
            }
            LabelScope::Local => {
                // Local labels are prefixed with parent name
                if let Some(parent) = parent_name {
                    format!(
                        "{}__{}",
                        Self::sanitize_identifier(parent),
                        Self::sanitize_identifier(&label.name)
                    )
                } else {
                    Self::sanitize_identifier(&label.name)
                }
            }
        };

        // Append counter if this is a duplicate (counter > 0)
        if counter > 0 {
            format!("{}_{}", base_name, counter)
        } else {
            base_name
        }
    }

    /// Sanitize identifier to be valid Rune function name.
    fn sanitize_identifier(name: &str) -> String {
        // For now, just replace invalid characters with underscores
        // In the future, this might need more sophisticated handling
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    /// Transpile a statement to Rune code.
    fn transpile_statement(
        output: &mut String,
        stmt: &Statement,
        context: &TranspileContext,
    ) -> Result<(), PastaError> {
        match stmt {
            Statement::Speech {
                speaker,
                content,
                span: _,
            } => {
                // Emit change speaker
                output.push_str(&format!("    yield change_speaker(\"{}\");\n", speaker));

                // Emit each content part
                for part in content {
                    Self::transpile_speech_part(output, part, context)?;
                }
            }
            Statement::Call {
                target,
                filters: _,
                args: _,
                span: _,
            } => {
                // Generate call statement
                let target_fn = Self::transpile_jump_target(target);
                output.push_str(&format!("    {}();\n", target_fn));
            }
            Statement::VarAssign {
                name,
                scope,
                value,
                span: _,
            } => {
                let value_expr = Self::transpile_expr(value, context)?;
                match scope {
                    VarScope::Local => {
                        output.push_str(&format!("    let {} = {};\n", name, value_expr));
                    }
                    VarScope::Global => {
                        output
                            .push_str(&format!("    set_global(\"{}\", {});\n", name, value_expr));
                    }
                }
            }
            Statement::RuneBlock { content, span: _ } => {
                // Output the Rune code inline with proper indentation
                for line in content.lines() {
                    if line.trim().is_empty() {
                        output.push('\n');
                    } else {
                        output.push_str("    ");
                        output.push_str(line.trim_start());
                        output.push('\n');
                    }
                }
            }
        }
        Ok(())
    }

    /// Transpile a speech part to Rune code.
    fn transpile_speech_part(
        output: &mut String,
        part: &SpeechPart,
        context: &TranspileContext,
    ) -> Result<(), PastaError> {
        match part {
            SpeechPart::Text(text) => {
                output.push_str(&format!(
                    "    yield emit_text(\"{}\");\n",
                    Self::escape_string(text)
                ));
            }
            SpeechPart::VarRef(var_name) => {
                output.push_str(&format!(
                    "    yield emit_text(&format!(\"{{}}\", get_variable(\"{}\")));\n",
                    var_name
                ));
            }
            SpeechPart::FuncCall { name, args, scope } => {
                // Resolve function name using scope rules
                let resolved_name = context.resolve_function(name, *scope)?;

                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => Self::transpile_expr(expr, context),
                        Argument::Named { name, value } => Ok(format!(
                            "{}={}",
                            name,
                            Self::transpile_expr(value, context)?
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                output.push_str(&format!("    yield {}({});\n", resolved_name, args_str));
            }
            SpeechPart::SakuraScript(script) => {
                output.push_str(&format!(
                    "    yield emit_sakura_script(\"{}\");\n",
                    Self::escape_string(script)
                ));
            }
        }
        Ok(())
    }

    /// Transpile a jump target to a function name.
    fn transpile_jump_target(target: &JumpTarget) -> String {
        match target {
            JumpTarget::Local(name) => Self::sanitize_identifier(name),
            JumpTarget::Global(name) => Self::sanitize_identifier(name),
            JumpTarget::LongJump { global, local } => {
                format!(
                    "{}_{}",
                    Self::sanitize_identifier(global),
                    Self::sanitize_identifier(local)
                )
            }
            JumpTarget::Dynamic(var_name) => {
                // Dynamic targets need to be resolved at runtime
                format!("resolve_label(\"{}\")", var_name)
            }
        }
    }

    /// Transpile an expression to Rune code.
    fn transpile_expr(expr: &Expr, context: &TranspileContext) -> Result<String, PastaError> {
        match expr {
            Expr::Literal(lit) => Ok(Self::transpile_literal(lit)),
            Expr::VarRef { name, scope } => match scope {
                VarScope::Local => Ok(name.clone()),
                VarScope::Global => Ok(format!("get_global(\"{}\")", name)),
            },
            Expr::FuncCall { name, args, scope } => {
                // Resolve function name using scope rules
                let resolved_name = context.resolve_function(name, *scope)?;

                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => Self::transpile_expr(expr, context),
                        Argument::Named { name, value } => Ok(format!(
                            "{}={}",
                            name,
                            Self::transpile_expr(value, context)?
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", resolved_name, args_str))
            }
            Expr::BinaryOp { op, lhs, rhs } => {
                let lhs_str = Self::transpile_expr(lhs, context)?;
                let rhs_str = Self::transpile_expr(rhs, context)?;
                let op_str = Self::transpile_binop(*op);
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            Expr::Paren(inner) => {
                let inner_str = Self::transpile_expr(inner, context)?;
                Ok(format!("({})", inner_str))
            }
        }
    }

    /// Transpile a literal to Rune code.
    fn transpile_literal(lit: &Literal) -> String {
        match lit {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => format!("\"{}\"", Self::escape_string(s)),
        }
    }

    /// Transpile a binary operator to Rune code.
    fn transpile_binop(op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
        }
    }

    /// Escape a string for use in Rune code.
    fn escape_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(Transpiler::sanitize_identifier("hello"), "hello");
        assert_eq!(
            Transpiler::sanitize_identifier("hello-world"),
            "hello_world"
        );
        assert_eq!(Transpiler::sanitize_identifier("＊挨拶"), "_挨拶"); // Full-width asterisk replaced, Japanese kept
        assert_eq!(Transpiler::sanitize_identifier("挨拶"), "挨拶"); // Pure Japanese unchanged
    }

    #[test]
    fn test_transpile_simple_label() {
        let file = PastaFile {
            path: "test.pasta".into(),
            global_words: vec![],
            labels: vec![LabelDef {
                name: "greeting".to_string(),
                scope: LabelScope::Global,
                params: vec![],
                attributes: vec![],
                local_words: vec![],
                local_labels: vec![],
                statements: vec![Statement::Speech {
                    speaker: "sakura".to_string(),
                    content: vec![SpeechPart::Text("Hello!".to_string())],
                    span: Span::new(1, 1, 1, 10),
                }],
                span: Span::new(1, 1, 2, 1),
            }],
            span: Span::new(1, 1, 2, 1),
        };

        let result = Transpiler::transpile(&file).unwrap();
        assert!(result.contains("pub fn greeting(ctx)"));
        assert!(result.contains("yield change_speaker(\"sakura\")"));
        assert!(result.contains("yield emit_text(\"Hello!\")"));
    }

    #[test]
    fn test_transpile_expr() {
        let expr = Expr::BinaryOp {
            op: BinOp::Add,
            lhs: Box::new(Expr::Literal(Literal::Number(1.0))),
            rhs: Box::new(Expr::Literal(Literal::Number(2.0))),
        };
        let context = TranspileContext::new();
        let result = Transpiler::transpile_expr(&expr, &context).unwrap();
        assert_eq!(result, "(1 + 2)");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(Transpiler::escape_string("hello"), "hello");
        assert_eq!(Transpiler::escape_string("hello\"world"), "hello\\\"world");
        assert_eq!(Transpiler::escape_string("hello\\world"), "hello\\\\world");
        assert_eq!(Transpiler::escape_string("hello\nworld"), "hello\\nworld");
    }
}
