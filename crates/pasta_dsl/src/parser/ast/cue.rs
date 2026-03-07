//! Cue command AST types for `!` command lines.
//!
//! `!id[@scoped_ident][(args)]` 形式のキューコマンド行を構造的に表現する AST ノード型。
//! コマンド名の意味解釈は行わない（dola 側の責務）。

use super::Span;

// ============================================================================
// CueCommandNode - Cue Command Line AST Node
// ============================================================================

/// キューコマンド行の AST ノード。
///
/// `!id[@scoped_ident][(args)]` 形式のキューコマンドを構造的に保持する。
/// コマンド名の意味解釈は行わない（dola 側の責務）。
///
/// # grammar.pest 対応
///
/// `cue_cmd_line = { pad ~ cue_cmd_marker ~ cue_cmd_name ~ cue_cmd_scope? ~ cue_cmd_args? ~ or_comment_eol }`
///
/// # 例
///
/// | DSL 記法 | command | scope | args |
/// |---------|---------|-------|------|
/// | `!emote@普通(normal)` | `"emote"` | `Some(ScopedName { actor: None, name: "普通" })` | `[Ident("normal")]` |
/// | `!mark@挨拶後` | `"mark"` | `Some(ScopedName { actor: None, name: "挨拶後" })` | `[]` |
/// | `!yield(10.0)` | `"yield"` | `None` | `[Float(10.0)]` |
/// | `!clear` | `"clear"` | `None` | `[]` |
#[derive(Debug, Clone)]
pub struct CueCommandNode {
    /// コマンド名（任意の `id`）。例: "emote", "mark", "choice", "yield"
    pub command: String,
    /// オプショナルなスコープ付き識別子。`@name` または `@actor:name`
    pub scope: Option<ScopedName>,
    /// 引数トークン列（カンマ区切り）
    pub args: Vec<CueArgToken>,
    /// ソース位置
    pub span: Span,
}

// ============================================================================
// ScopedName - Scoped Identifier
// ============================================================================

/// スコープ付き識別子。
///
/// `@name` または `@actor:name` 形式を表現する。
/// `:` で区切られた場合、前半が `actor`、後半が `name`。
///
/// PEG 文法の `cue_scoped_ident` はアトミックルールであるため、
/// Rust 側で `:` 分割を行う。
///
/// # 例
///
/// - `@笑顔` → `ScopedName { actor: None, name: "笑顔" }`
/// - `@さくら:笑顔` → `ScopedName { actor: Some("さくら"), name: "笑顔" }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedName {
    /// アクター名（`:` 区切りの前半）。None の場合はグローバルスコープ
    pub actor: Option<String>,
    /// 識別子名（`:` 区切りの後半、または全体）
    pub name: String,
    /// ソース位置
    pub span: Span,
}

// ============================================================================
// CueArgToken - Cue Command Argument Token
// ============================================================================

/// キューコマンド引数のトークン。
///
/// 既存文法のプリミティブを組み合わせた引数トークンを表現する。
/// 各トークンは構文解析レベルの型情報のみを持ち、意味解釈は dola 側で行う。
///
/// # grammar.pest 対応
///
/// `cue_arg = _{ cue_arg_at_ref | number_literal | string_literal | cue_arg_id }`
///
/// # バリアント
///
/// - `Ident`: 汎用識別子（`cue_arg_id`）
/// - `StringLiteral`: 文字列リテラル（`string_literal`）
/// - `Integer`: 整数リテラル（`number_literal` で小数点なし）
/// - `Float`: 浮動小数点リテラル（`number_literal` で小数点あり）
/// - `AtRef`: @参照（`cue_arg_at_ref`）
#[derive(Debug, Clone, PartialEq)]
pub enum CueArgToken {
    /// 識別子トークン（例: "normal", "yes", "shell"）
    Ident(String),
    /// 文字列リテラル（例: 「はい、行きましょう！」, "hello"）
    StringLiteral(String),
    /// 数値リテラル — 整数（小数点なし）
    Integer(i64),
    /// 数値リテラル — 浮動小数点（小数点あり）
    Float(f64),
    /// @参照トークン（例: @name）— @ を除いた名前を保持
    AtRef(String),
}
