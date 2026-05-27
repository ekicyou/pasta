//! Action-related AST types: actions, expressions, operators, and their components.

use super::*;

// ============================================================================
// Action - Individual Actions
// ============================================================================

/// Individual action within an action line.
///
/// Corresponds to the `action` rule alternatives in grammar.pest.
/// Each action carries a Span for precise source location mapping.
#[derive(Debug, Clone)]
pub enum Action {
    /// Plain text (talk)
    Talk { text: String, span: Span },
    /// Word reference (@word)
    WordRef { name: String, span: Span },
    /// Variable reference ($var or $*var)
    VarRef {
        name: String,
        scope: VarScope,
        span: Span,
    },
    /// Function call (@func() or @*func())
    FnCall {
        name: String,
        args: Args,
        scope: FnScope,
        span: Span,
    },
    /// Sakura script (\\command[args])
    SakuraScript { script: String, span: Span },
    /// Escape sequence (@@, $$, \\\\)
    ///
    /// In pasta2.pest, these are atomic rules that match the literal text.
    Escape { sequence: String, span: Span },
}

// ============================================================================
// CodeBlock
// ============================================================================

/// Code block with optional language identifier.
///
/// Corresponds to the `code_block` rule: ` ```language ... ``` `
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Language identifier (e.g., "rune", "rust")
    pub language: Option<String>,
    /// Code content
    pub content: String,
    /// Source location
    pub span: Span,
}

// ============================================================================
// VarSet - Variable Assignment
// ============================================================================

/// Variable assignment or expression statement.
///
/// Corresponds to the `var_set` rule: `$var = expr`, `$*var = expr`, or `$= expr`
#[derive(Debug, Clone)]
pub struct VarSet {
    /// Variable name. `None` for expression statements (`var_set_none`).
    pub name: Option<String>,
    /// Variable scope
    pub scope: VarScope,
    /// Value (expression or word reference)
    pub value: SetValue,
    /// Source location
    pub span: Span,
}

// ============================================================================
// CallScene - Scene Call
// ============================================================================

/// コールターゲット: 静的（リテラルシーン名）または動的（式評価結果）
#[derive(Debug, Clone, PartialEq)]
pub enum CallTarget {
    /// 静的コール: リテラルのシーン名（従来の ＞シーン名）
    Static(String),
    /// 動的コール: 式の評価結果をシーン名として使用（＞expr）
    Dynamic(Expr),
}

/// Scene call.
///
/// Corresponds to the `call_scene` rule: `>scene_name args?`
#[derive(Debug, Clone)]
pub struct CallScene {
    /// Target: static scene name or dynamic expression
    pub target: CallTarget,
    /// Optional arguments
    pub args: Option<Args>,
    /// Source location
    pub span: Span,
}

// ============================================================================
// Attr - Attribute
// ============================================================================

/// Attribute key-value pair.
///
/// Corresponds to the `attr` rule: `&key：value`
#[derive(Debug, Clone)]
pub struct Attr {
    /// Attribute key
    pub key: String,
    /// Attribute value
    pub value: AttrValue,
    /// Source location
    pub span: Span,
}

/// Attribute value types.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    /// Integer literal (no decimal point)
    Integer(i64),
    /// Floating point literal (has decimal point)
    Float(f64),
    /// String literal (quoted)
    String(String),
    /// Attribute string (unquoted)
    AttrString(String),
}

impl std::fmt::Display for AttrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrValue::Integer(v) => write!(f, "{}", v),
            AttrValue::Float(v) => write!(f, "{}", v),
            AttrValue::String(v) => write!(f, "{}", v),
            AttrValue::AttrString(v) => write!(f, "{}", v),
        }
    }
}

// ============================================================================
// KeyWords - Word Definition
// ============================================================================

/// Word definition for random selection.
///
/// Corresponds to the `key_words` rule: `@key1、key2：word1、word2、...`
#[derive(Debug, Clone)]
pub struct KeyWords {
    /// Key name list (at least one element guaranteed)
    pub names: Vec<String>,
    /// List of word values
    pub words: Vec<String>,
    /// Source location
    pub span: Span,
}

impl KeyWords {
    /// Returns the first (primary) key name.
    ///
    /// Equivalent to the former `name` field for backward compatibility.
    pub fn name(&self) -> &str {
        &self.names[0]
    }
}

// ============================================================================
// Args and Arg - Function/Call Arguments
// ============================================================================

/// Argument list.
///
/// Corresponds to the `args` rule: `(arg1, arg2, ...)`
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    /// List of arguments
    pub items: Vec<Arg>,
    /// Source location
    pub span: Span,
}

impl Args {
    /// Create an empty argument list.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            span: Span::default(),
        }
    }
}

/// Single argument (positional or keyword).
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// Positional argument
    Positional(Expr),
    /// Keyword argument (key: value)
    Keyword { key: String, value: Expr },
}

// ============================================================================
// Expr - Expressions
// ============================================================================

/// Expression types.
///
/// Corresponds to the `expr` rule and its alternatives.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal (no decimal point)
    Integer(i64),
    /// Floating point literal (has decimal point)
    Float(f64),
    /// String literal
    String(String),
    /// Empty string literal ("" or 「」)
    BlankString,
    /// Variable reference
    VarRef { name: String, scope: VarScope },
    /// Function call
    FnCall {
        name: String,
        args: Args,
        scope: FnScope,
    },
    /// Parenthesized expression
    Paren(Box<Expr>),
    /// Binary operation
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

// ============================================================================
// SetValue - VarSet value type
// ============================================================================

/// Value type for variable assignment (VarSet).
///
/// Corresponds to grammar.pest's `set = ( expr | word_ref )`.
/// Distinguishes whether the right-hand side is an expression or a word reference.
#[derive(Debug, Clone, PartialEq)]
pub enum SetValue {
    /// Expression value (numbers, strings, variable references, function calls, binary operations, etc.)
    Expr(Expr),
    /// Word reference (`@word_name` form)
    WordRef { name: String },
}

// ============================================================================
// Scope Enums
// ============================================================================

/// Variable scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarScope {
    /// Local variable ($var)
    Local,
    /// Global variable ($*var)
    Global,
    /// Scene argument reference ($0, $1, ...)
    Args(u8),
    /// Property variable ($%prop)
    Property,
}

/// Function scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnScope {
    /// Local-first search (@func)
    Local,
    /// Global only (@*func)
    Global,
}

// ============================================================================
// Binary Operators
// ============================================================================

/// Binary operators.
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
