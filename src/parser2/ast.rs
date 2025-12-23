//! AST type definitions for parser2 based on pasta2.pest grammar.
//!
//! This module defines all AST types corresponding to the grammar rules
//! defined in grammar.pest. The types follow a 3-layer scope hierarchy:
//! FileScope ⊃ GlobalSceneScope ⊃ LocalSceneScope.
//!
//! # Differences from pasta.pest
//!
//! - `ContinueAction`: Continuation lines now explicitly start with `：` or `:`
//!   (pasta2.pest specification change from pasta.pest)

use std::path::PathBuf;

// ============================================================================
// FileItem - File-Level Item
// ============================================================================

/// ファイルレベルで出現するアイテムの統一表現
///
/// grammar.pest の `file = ( file_scope | global_scene_scope )*` に対応。
/// file_scope 内の attrs と words は個別のバリアントとして分離。
///
/// # grammar.pest 対応関係
///
/// - `FileAttr`: file_scope 内の attr（ファイルレベル属性）
/// - `GlobalWord`: file_scope 内の key_words（ファイルレベル単語定義）
/// - `GlobalSceneScope`: global_scene_scope（グローバルシーン）
///
/// # 使用例
///
/// ```ignore
/// for item in &file.items {
///     match item {
///         FileItem::FileAttr(attr) => { /* 属性処理 */ }
///         FileItem::GlobalWord(word) => { /* 単語定義処理 */ }
///         FileItem::GlobalSceneScope(scene) => { /* シーン処理 */ }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum FileItem {
    /// ファイルレベル属性（file_scope 内の attr）
    FileAttr(Attr),
    /// ファイルレベル単語定義（file_scope 内の key_words）
    GlobalWord(KeyWords),
    /// グローバルシーン
    GlobalSceneScope(GlobalSceneScope),
}

// ============================================================================
// Span - Source Location
// ============================================================================

/// Source location in the original file.
///
/// All AST nodes carry span information for error reporting and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// 1-based start line number
    pub start_line: usize,
    /// 1-based start column number
    pub start_col: usize,
    /// 1-based end line number
    pub end_line: usize,
    /// 1-based end column number
    pub end_col: usize,
}

impl Span {
    /// Create a new span with explicit coordinates.
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Create a span from pest's position tuples.
    ///
    /// Pest uses 1-based line numbers and 1-based column numbers.
    pub fn from_pest(start: (usize, usize), end: (usize, usize)) -> Self {
        Self::new(start.0, start.1, end.0, end.1)
    }
}

// ============================================================================
// Top-Level AST: PastaFile
// ============================================================================

/// Complete AST representation of a Pasta file.
///
/// grammar.pest `file = ( file_scope | global_scene_scope )*` に完全準拠。
/// ファイル内の全アイテムを記述順序で保持します。
///
/// # Migration Guide (移行ガイド)
///
/// 旧APIからの移行:
/// - `file.file_scope.attrs` → `file.file_attrs()`
/// - `file.file_scope.words` → `file.words()`
/// - `file.global_scenes` → `file.global_scene_scopes()`
///
/// # 使用例
///
/// ```ignore
/// // 型別アクセス（ヘルパーメソッド）
/// let attrs = file.file_attrs();
/// let words = file.words();
/// let scenes = file.global_scene_scopes();
///
/// // 順序保持アクセス（transpiler2向け）
/// for item in &file.items {
///     match item {
///         FileItem::FileAttr(attr) => { /* コンテキスト積算 */ }
///         FileItem::GlobalWord(word) => { /* 単語定義積算 */ }
///         FileItem::GlobalSceneScope(scene) => { /* シーン処理 */ }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PastaFile {
    /// Source file path
    pub path: PathBuf,
    /// ファイル内の全アイテム（記述順序を保持）
    ///
    /// grammar.pest `( file_scope | global_scene_scope )*` に対応。
    /// 複数の file_scope と global_scene_scope を任意順序で格納。
    pub items: Vec<FileItem>,
    /// Source location
    pub span: Span,
}

impl PastaFile {
    /// Create a new PastaFile with the given path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            items: Vec::new(),
            span: Span::default(),
        }
    }

    /// ファイルレベル属性を取得（FileAttr バリアントのみ抽出）
    ///
    /// 複数の file_scope に分散した属性を記述順で返します。
    pub fn file_attrs(&self) -> Vec<&Attr> {
        self.items
            .iter()
            .filter_map(|item| {
                if let FileItem::FileAttr(attr) = item {
                    Some(attr)
                } else {
                    None
                }
            })
            .collect()
    }

    /// ファイルレベル単語定義を取得（GlobalWord バリアントのみ抽出）
    ///
    /// 複数の file_scope に分散した単語定義を記述順で返します。
    pub fn words(&self) -> Vec<&KeyWords> {
        self.items
            .iter()
            .filter_map(|item| {
                if let FileItem::GlobalWord(word) = item {
                    Some(word)
                } else {
                    None
                }
            })
            .collect()
    }

    /// グローバルシーンを取得（GlobalSceneScope バリアントのみ抽出）
    ///
    /// 記述順で全グローバルシーンを返します。
    pub fn global_scene_scopes(&self) -> Vec<&GlobalSceneScope> {
        self.items
            .iter()
            .filter_map(|item| {
                if let FileItem::GlobalSceneScope(scene) = item {
                    Some(scene)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ============================================================================
// FileScope - File-Level Scope
// ============================================================================

/// File-level scope containing attributes and word definitions.
///
/// Corresponds to the `file_scope` rule in grammar.pest.
#[derive(Debug, Clone, Default)]
pub struct FileScope {
    /// File-level attributes
    pub attrs: Vec<Attr>,
    /// File-level word definitions
    pub words: Vec<KeyWords>,
}

// ============================================================================
// GlobalSceneScope - Global Scene Scope
// ============================================================================

/// Global scene scope containing local scenes and scene-level definitions.
///
/// Corresponds to the `global_scene_scope` rule in grammar.pest.
/// Global scenes form the second layer of the 3-layer scope hierarchy.
#[derive(Debug, Clone)]
pub struct GlobalSceneScope {
    /// Scene name (inherited from previous scene if continuation)
    pub name: String,
    /// True if this is a continuation scene (global_scene_continue_line)
    pub is_continuation: bool,
    /// Scene attributes
    pub attrs: Vec<Attr>,
    /// Scene-level word definitions
    pub words: Vec<KeyWords>,
    /// Code blocks at global scene level
    pub code_blocks: Vec<CodeBlock>,
    /// List of local scenes
    pub local_scenes: Vec<LocalSceneScope>,
    /// Source location
    pub span: Span,
}

impl GlobalSceneScope {
    /// Create a new named global scene.
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_continuation: false,
            attrs: Vec::new(),
            words: Vec::new(),
            code_blocks: Vec::new(),
            local_scenes: Vec::new(),
            span: Span::default(),
        }
    }

    /// Create a continuation scene inheriting the given name.
    pub fn continuation(name: String) -> Self {
        Self {
            name,
            is_continuation: true,
            attrs: Vec::new(),
            words: Vec::new(),
            code_blocks: Vec::new(),
            local_scenes: Vec::new(),
            span: Span::default(),
        }
    }
}

// ============================================================================
// LocalSceneScope - Local Scene Scope
// ============================================================================

/// Local scene scope containing scene items.
///
/// Corresponds to the `local_scene_scope` and `local_start_scene_scope` rules.
/// Local scenes form the third layer of the 3-layer scope hierarchy.
#[derive(Debug, Clone)]
pub struct LocalSceneScope {
    /// Scene name (None for local_start_scene_scope)
    pub name: Option<String>,
    /// Scene attributes
    pub attrs: Vec<Attr>,
    /// Local scene items (statements)
    pub items: Vec<LocalSceneItem>,
    /// Code blocks at local scene level
    pub code_blocks: Vec<CodeBlock>,
    /// Source location
    pub span: Span,
}

impl LocalSceneScope {
    /// Create a start scene (no name).
    pub fn start() -> Self {
        Self {
            name: None,
            attrs: Vec::new(),
            items: Vec::new(),
            code_blocks: Vec::new(),
            span: Span::default(),
        }
    }

    /// Create a named local scene.
    pub fn named(name: String) -> Self {
        Self {
            name: Some(name),
            attrs: Vec::new(),
            items: Vec::new(),
            code_blocks: Vec::new(),
            span: Span::default(),
        }
    }
}

// ============================================================================
// LocalSceneItem - Items within Local Scene
// ============================================================================

/// Items that can appear within a local scene.
///
/// Corresponds to the `local_scene_item` rule in grammar.pest.
#[derive(Debug, Clone)]
pub enum LocalSceneItem {
    /// Variable assignment (var_set_line)
    VarSet(VarSet),
    /// Scene call (call_scene_line)
    CallScene(CallScene),
    /// Action line (action_line)
    ActionLine(ActionLine),
    /// Continuation action line (continue_action_line)
    ContinueAction(ContinueAction),
}

// ============================================================================
// ActionLine and ContinueAction
// ============================================================================

/// Action line with actor.
///
/// Corresponds to the `action_line` rule: `actor：actions`
#[derive(Debug, Clone)]
pub struct ActionLine {
    /// Actor name
    pub actor: String,
    /// List of actions
    pub actions: Vec<Action>,
    /// Source location
    pub span: Span,
}

/// Continuation action line without speaker.
///
/// Corresponds to the `continue_action_line` rule: `：actions`
///
/// # pasta2.pest Specification Change
///
/// In pasta2.pest, continuation lines explicitly start with `：` or `:`.
/// This is a change from pasta.pest where continuation lines had no explicit prefix.
#[derive(Debug, Clone)]
pub struct ContinueAction {
    /// List of actions
    pub actions: Vec<Action>,
    /// Source location
    pub span: Span,
}

// ============================================================================
// Action - Individual Actions
// ============================================================================

/// Individual action within an action line.
///
/// Corresponds to the `action` rule alternatives in grammar.pest.
#[derive(Debug, Clone)]
pub enum Action {
    /// Plain text (talk)
    Talk(String),
    /// Word reference (@word)
    WordRef(String),
    /// Variable reference ($var or $*var)
    VarRef { name: String, scope: VarScope },
    /// Function call (@func() or @*func())
    FnCall {
        name: String,
        args: Args,
        scope: FnScope,
    },
    /// Sakura script (\\command[args])
    SakuraScript(String),
    /// Escape sequence (@@, $$, \\\\)
    ///
    /// In pasta2.pest, these are atomic rules that match the literal text.
    Escape(String),
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

/// Variable assignment.
///
/// Corresponds to the `var_set` rule: `$var = expr` or `$*var = expr`
#[derive(Debug, Clone)]
pub struct VarSet {
    /// Variable name
    pub name: String,
    /// Variable scope
    pub scope: VarScope,
    /// Value expression
    pub value: Expr,
    /// Source location
    pub span: Span,
}

// ============================================================================
// CallScene - Scene Call
// ============================================================================

/// Scene call.
///
/// Corresponds to the `call_scene` rule: `>scene_name args?`
#[derive(Debug, Clone)]
pub struct CallScene {
    /// Target scene name
    pub target: String,
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
#[derive(Debug, Clone)]
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

// ============================================================================
// KeyWords - Word Definition
// ============================================================================

/// Word definition for random selection.
///
/// Corresponds to the `key_words` rule: `@name：word1、word2、...`
#[derive(Debug, Clone)]
pub struct KeyWords {
    /// Word name
    pub name: String,
    /// List of word values
    pub words: Vec<String>,
    /// Source location
    pub span: Span,
}

// ============================================================================
// Args and Arg - Function/Call Arguments
// ============================================================================

/// Argument list.
///
/// Corresponds to the `args` rule: `(arg1, arg2, ...)`
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
// Scope Enums
// ============================================================================

/// Variable scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarScope {
    /// Local variable ($var)
    Local,
    /// Global variable ($*var)
    Global,
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(1, 1, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 1);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 10);
    }

    #[test]
    fn test_span_from_pest() {
        let span = Span::from_pest((5, 3), (10, 15));
        assert_eq!(span.start_line, 5);
        assert_eq!(span.start_col, 3);
        assert_eq!(span.end_line, 10);
        assert_eq!(span.end_col, 15);
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start_line, 0);
        assert_eq!(span.start_col, 0);
        assert_eq!(span.end_line, 0);
        assert_eq!(span.end_col, 0);
    }

    #[test]
    fn test_file_scope_default() {
        let scope = FileScope::default();
        assert!(scope.attrs.is_empty());
        assert!(scope.words.is_empty());
    }

    #[test]
    fn test_pasta_file_new() {
        let file = PastaFile::new(PathBuf::from("test.pasta"));
        assert_eq!(file.path, PathBuf::from("test.pasta"));
        assert!(file.file_attrs().is_empty());
        assert!(file.global_scene_scopes().is_empty());
    }

    #[test]
    fn test_global_scene_scope_new() {
        let scene = GlobalSceneScope::new("挨拶".to_string());
        assert_eq!(scene.name, "挨拶");
        assert!(!scene.is_continuation);
    }

    #[test]
    fn test_global_scene_scope_continuation() {
        let scene = GlobalSceneScope::continuation("挨拶".to_string());
        assert_eq!(scene.name, "挨拶");
        assert!(scene.is_continuation);
    }

    #[test]
    fn test_local_scene_scope_start() {
        let scene = LocalSceneScope::start();
        assert!(scene.name.is_none());
    }

    #[test]
    fn test_local_scene_scope_named() {
        let scene = LocalSceneScope::named("hello".to_string());
        assert_eq!(scene.name, Some("hello".to_string()));
    }

    #[test]
    fn test_args_empty() {
        let args = Args::empty();
        assert!(args.items.is_empty());
    }

    #[test]
    fn test_var_scope_equality() {
        assert_eq!(VarScope::Local, VarScope::Local);
        assert_ne!(VarScope::Local, VarScope::Global);
    }

    #[test]
    fn test_fn_scope_equality() {
        assert_eq!(FnScope::Local, FnScope::Local);
        assert_ne!(FnScope::Local, FnScope::Global);
    }

    #[test]
    fn test_bin_op_equality() {
        assert_eq!(BinOp::Add, BinOp::Add);
        assert_ne!(BinOp::Add, BinOp::Sub);
    }

    #[test]
    fn test_ast_types_clone() {
        // Test that all AST types implement Clone
        let span = Span::new(1, 1, 1, 1);
        let _span2 = span.clone();

        let file = PastaFile::new(PathBuf::from("test.pasta"));
        let _file2 = file.clone();

        let attr = Attr {
            key: "test".to_string(),
            value: AttrValue::Integer(42),
            span: Span::default(),
        };
        let _attr2 = attr.clone();
    }

    #[test]
    fn test_ast_types_debug() {
        // Test that all AST types implement Debug
        let span = Span::new(1, 1, 1, 1);
        let _ = format!("{:?}", span);

        let file = PastaFile::new(PathBuf::from("test.pasta"));
        let _ = format!("{:?}", file);

        let expr = Expr::Integer(42);
        let _ = format!("{:?}", expr);

        let action = Action::Talk("hello".to_string());
        let _ = format!("{:?}", action);
    }
}
