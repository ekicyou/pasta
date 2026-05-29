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

mod action;
mod cue;
mod scene;
mod span;

pub use action::*;
pub use cue::*;
pub use scene::*;
pub use span::*;

use std::path::PathBuf;

// ============================================================================
// FileItem - File-Level Item
// ============================================================================

/// ファイルレベルで出現するアイテムの統一表現
///
/// grammar.pest の `file = ( file_scope | global_scene_scope | actor_scope )*` に対応。
/// file_scope 内の attrs と words は個別のバリアントとして分離。
///
/// # grammar.pest 対応関係
///
/// - `FileAttr`: file_scope 内の attr（ファイルレベル属性）
/// - `GlobalWord`: file_scope 内の key_words（ファイルレベル単語定義）
/// - `GlobalSceneScope`: global_scene_scope（グローバルシーン）
/// - `ActorScope`: actor_scope（アクター定義）
///
/// # 使用例
///
/// ```ignore
/// for item in &file.items {
///     match item {
///         FileItem::FileAttr(attr) => { /* 属性処理 */ }
///         FileItem::GlobalWord(word) => { /* 単語定義処理 */ }
///         FileItem::GlobalSceneScope(scene) => { /* シーン処理 */ }
///         FileItem::ActorScope(actor) => { /* アクター処理 */ }
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
    /// アクター定義（actor_scope）
    ActorScope(ActorScope),
}

// ============================================================================
// Top-Level AST: PastaFile
// ============================================================================

/// Complete AST representation of a Pasta file.
///
/// grammar.pest `file = ( file_scope | global_scene_scope )*` に完全準拠。
/// ファイル内の全アイテムを記述順序で保持します。
///
/// # 使用例
///
/// ```ignore
/// // 順序保持アクセス（transpiler向け - 推奨）
/// for item in &file.items {
///     match item {
///         FileItem::FileAttr(attr) => { /* 属性処理 */ }
///         FileItem::GlobalWord(word) => { /* 単語定義処理 */ }
///         FileItem::GlobalSceneScope(scene) => { /* シーン処理 */ }
///         FileItem::ActorScope(actor) => { /* アクター処理 */ }
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
}

// ============================================================================
// ActorScope - Actor Definition Scope
// ============================================================================

/// アクター定義スコープ
///
/// grammar.pest の `actor_scope = { actor_line ~ actor_scope_item* }` に対応。
/// アクター（キャラクター）の名前とその属性・単語定義・変数設定を保持します。
///
/// # 例
///
/// ```pasta
/// ％さくら
///   ＠通常  ：\s[0]
///   ＠照れ  ：\s[1]
///   ＄デフォルト表情＝0
/// ```
#[derive(Debug, Clone)]
pub struct ActorScope {
    /// アクター名
    pub name: String,
    /// アクターの属性
    pub attrs: Vec<Attr>,
    /// アクターの単語定義（表情など）
    pub words: Vec<KeyWords>,
    /// アクターの変数設定
    pub var_sets: Vec<VarSet>,
    /// コードブロック（Lua関数定義など）
    pub code_blocks: Vec<CodeBlock>,
    /// ソース位置
    pub span: Span,
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
