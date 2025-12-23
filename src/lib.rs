//! Pasta - Script engine for areka desktop mascot application.
//!
//! This crate provides a script engine that interprets the Pasta DSL (inspired by Satori/Satoru)
//! and generates conversation control events. The engine is designed to be fully unit-testable
//! without user testing, as it contains no UI rendering or timing control logic.
//!
//! # Architecture
//!
//! The Pasta engine consists of several layers:
//!
//! - **Parser Layer**: Parses Pasta DSL using pest (PEG parser)
//! - **Transpiler Layer**: Converts Pasta AST to Rune source code
//! - **Runtime Layer**: Executes Rune code using Rune VM with generators
//! - **IR Output**: Generates `ScriptEvent` intermediate representation
//!
//! # Design Principles
//!
//! - **No timing control**: Wait events are markers; areka handles timing
//! - **No buffering**: Events are yielded sequentially; areka handles buffering
//! - **No synchronization logic**: Sync markers only; areka implements sync control
//! - **No UI dependencies**: Pure script generation; areka handles rendering
//!
//! # Example
//!
//! ```no_run
//! use pasta::ir::ScriptEvent;
//!
//! // Parse and execute a Pasta script (API to be implemented)
//! // let engine = PastaEngine::new(script_source)?;
//! // let mut generator = engine.execute_label("挨拶")?;
//! //
//! // while let Some(event) = generator.resume() {
//! //     match event {
//! //         ScriptEvent::Talk { speaker, content } => {
//! //             println!("{}: {:?}", speaker, content);
//! //         }
//! //         _ => {}
//! //     }
//! // }
//! ```

pub mod cache;
pub mod engine;
pub mod error;
pub mod ir;
mod loader;
pub mod parser;
pub mod parser2;
pub mod runtime;
pub mod stdlib;
pub mod transpiler;

// Re-export commonly used types
pub use cache::ParseCache;
pub use engine::PastaEngine;
pub use error::{ParseErrorInfo, PastaError, Result};
pub use ir::{ContentPart, ScriptEvent};
pub use loader::{DirectoryLoader, LoadedFiles};
pub use parser::{
    Argument, Attribute, AttributeValue, BinOp, Expr, FunctionScope, JumpTarget, Literal,
    PastaFile, SceneDef, SceneScope, Span, SpeechPart, Statement, VarScope, parse_file, parse_str,
};
pub use runtime::{
    DefaultRandomSelector, RandomSelector, SceneInfo, SceneTable, ScriptGenerator,
    ScriptGeneratorState, VariableManager, VariableScope, VariableValue,
};
pub use transpiler::{TranspileContext, Transpiler};
