//! Scene-related AST types: global/local scene scopes, actors, action lines.

use super::*;

// ============================================================================
// SceneActorItem - Actor Item in Scene
// ============================================================================

/// シーンに登場するアクター項目
///
/// grammar.pestの`actors_item = { id ~ ( s ~ set_marker ~ s ~ digit_id )? }`に対応。
/// 番号はC#のenum採番ルールで計算済みの値を保持する。
///
/// # C# enum採番ルール
///
/// - 番号指定なし: 0から連番（または直前の番号+1）
/// - 番号指定あり: その番号を使用
/// - 再び番号なし: 直前の番号+1
///
/// # 例
///
/// `％さくら、うにゅう＝２、まりか` → さくら=0, うにゅう=2, まりか=3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneActorItem {
    /// アクター名
    pub name: String,
    /// アクター番号（C#のenum採番ルールで計算済み）
    pub number: u32,
    /// ソース位置
    pub span: Span,
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
    /// シーンに登場するアクター一覧
    pub actors: Vec<SceneActorItem>,
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
            actors: Vec::new(),
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
            actors: Vec::new(),
            code_blocks: Vec::new(),
            local_scenes: Vec::new(),
            span: Span::default(),
        }
    }

    /// いずれかのローカルシーン内にキューコマンド行が存在するかを返す。
    ///
    /// dola がキューシートモード判定に使用する。
    pub fn has_cue_commands(&self) -> bool {
        self.local_scenes.iter().any(|ls| ls.has_cue_commands())
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

    /// シーン内にキューコマンド行が 1 つ以上存在するかを返す。
    ///
    /// dola がキューシートモード判定に使用する。
    pub fn has_cue_commands(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, LocalSceneItem::CueCommand(_)))
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
    /// Cue command line (cue_cmd_line)
    CueCommand(CueCommandNode),
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
