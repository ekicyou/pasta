//! Abstract Syntax Tree (AST) types for Pasta DSL
//!
//! This module defines the AST node types that represent the parsed structure
//! of Pasta DSL scripts. The AST is created by the parser and consumed by the
//! transpiler to generate Rune code.

use std::path::PathBuf;

/// Function scope specification for function resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionScope {
    /// Automatic local→global search (`＠関数名`)
    Auto,
    /// Global-only search (`＠＊関数名`)
    GlobalOnly,
}

/// Represents a complete Pasta script file
#[derive(Debug, Clone)]
pub struct PastaFile {
    /// Path to the source file (for error reporting)
    pub path: PathBuf,
    /// Global word definitions
    pub global_words: Vec<WordDef>,
    /// All global scenes defined in the file
    pub scenes: Vec<SceneDef>,
    /// Source location span
    pub span: Span,
}

/// Word definition (global or local)
#[derive(Debug, Clone)]
pub struct WordDef {
    /// Word name
    pub name: String,
    /// Possible values for this word
    pub values: Vec<String>,
    /// Source location span
    pub span: Span,
}

/// Scene definition (global or local)
#[derive(Debug, Clone)]
pub struct SceneDef {
    /// Scene name (without marker prefix)
    pub name: String,
    /// Scope of the scene (global or local)
    pub scope: SceneScope,
    /// Parameters for this label (e.g., `＄値` in `ーカウント表示　＄値`)
    pub params: Vec<String>,
    /// Attributes attached to this label
    pub attributes: Vec<Attribute>,
    /// Local word definitions within this scene
    pub local_words: Vec<WordDef>,
    /// Local scenes nested within this scene (only for global scenes)
    pub local_scenes: Vec<SceneDef>,
    /// Statements in this scene's body
    pub statements: Vec<Statement>,
    /// Source location span
    pub span: Span,
}

/// Scene scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneScope {
    /// Global scene (accessible from anywhere)
    Global,
    /// Local scene (accessible only within parent global scene)
    Local,
}

/// Attribute definition for label filtering
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Attribute key
    pub key: String,
    /// Attribute value
    pub value: AttributeValue,
    /// Source location span
    pub span: Span,
}

/// Attribute value (literal or variable reference)
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// Literal string value
    Literal(String),
    /// Variable reference (will be evaluated at runtime)
    VarRef(String),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::Literal(s) => write!(f, "{}", s),
            AttributeValue::VarRef(name) => write!(f, "@{}", name),
        }
    }
}

/// Statement in a label body
#[derive(Debug, Clone)]
pub enum Statement {
    /// Speech/dialogue line
    Speech {
        /// Speaker name
        speaker: String,
        /// Speech content parts (text, variables, function calls)
        content: Vec<SpeechPart>,
        /// Source location span
        span: Span,
    },
    /// Call statement (subroutine call with return)
    Call {
        /// Jump target
        target: JumpTarget,
        /// Attribute filters for label selection
        filters: Vec<Attribute>,
        /// Arguments to pass to the called label
        args: Vec<Expr>,
        /// Source location span
        span: Span,
    },
    /// Variable assignment
    VarAssign {
        /// Variable name
        name: String,
        /// Variable scope
        scope: VarScope,
        /// Value expression
        value: Expr,
        /// Source location span
        span: Span,
    },
    /// Inline Rune code block
    RuneBlock {
        /// Raw Rune code content (as string, not parsed)
        content: String,
        /// Source location span
        span: Span,
    },
}

/// Part of speech content
#[derive(Debug, Clone)]
pub enum SpeechPart {
    /// Plain text
    Text(String),
    /// Variable reference (@var_name)
    VarRef(String),
    /// Function call (@func_name(args) or @*func_name(args))
    FuncCall {
        /// Function name
        name: String,
        /// Arguments
        args: Vec<Argument>,
        /// Function scope (Auto or GlobalOnly)
        scope: FunctionScope,
    },
    /// Sakura script escape sequence (\command)
    SakuraScript(String),
}

/// Jump target specification
#[derive(Debug, Clone)]
pub enum JumpTarget {
    /// Local label in current global label scope
    Local(String),
    /// Global label
    Global(String),
    /// Long jump to local label in specified global label
    LongJump {
        /// Global label name
        global: String,
        /// Local label name
        local: String,
    },
    /// Dynamic target (resolved from variable at runtime)
    Dynamic(String),
}

/// Variable scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarScope {
    /// Local variable (label scope)
    Local,
    /// Global variable (script scope)
    Global,
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    VarRef {
        /// Variable name
        name: String,
        /// Variable scope
        scope: VarScope,
    },
    /// Function call
    FuncCall {
        /// Function name
        name: String,
        /// Arguments
        args: Vec<Argument>,
        /// Function scope (Auto or GlobalOnly)
        scope: FunctionScope,
    },
    /// Binary operation
    BinaryOp {
        /// Operator
        op: BinOp,
        /// Left operand
        lhs: Box<Expr>,
        /// Right operand
        rhs: Box<Expr>,
    },
    /// Parenthesized expression
    Paren(Box<Expr>),
}

/// Argument (positional or named)
#[derive(Debug, Clone)]
pub enum Argument {
    /// Positional argument
    Positional(Expr),
    /// Named argument
    Named {
        /// Argument name
        name: String,
        /// Argument value
        value: Expr,
    },
}

/// Literal value
#[derive(Debug, Clone)]
pub enum Literal {
    /// Numeric literal
    Number(f64),
    /// String literal
    String(String),
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*)
    Mul,
    /// Division (/)
    Div,
    /// Modulo (%)
    Mod,
}

/// Source code span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting column number (1-indexed)
    pub start_col: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Ending column number (1-indexed)
    pub end_col: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Create a span from pest's line_col positions
    pub fn from_pest(start: (usize, usize), end: (usize, usize)) -> Self {
        Self::new(start.0, start.1, end.0, end.1)
    }
}
