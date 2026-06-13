//! Partial parse support for Pasta DSL.
//!
//! Provides `parse_str_partial()` which attempts 3-phase fallback parsing:
//! - Phase 1: Full parse via `parse_str()`
//! - Phase 2: Scope boundary split and per-chunk parse
//! - Phase 3: Line-by-line fallback with rule inference

use crate::parser::ast::{
    Action, ActionLine, ActorScope, Args, Attr, CallScene, ChoiceNode, CodeBlock, ContinueAction,
    CueCommandNode, FileItem, GlobalSceneScope, KeyWords, LocalSceneItem, LocalSceneScope,
    SceneActorItem, Span, VarSet,
};
use crate::parser::{parse_str, Rule};

/// 部分パース結果
#[derive(Debug, Clone)]
pub struct PartialParseResult {
    /// パース成功した部分のASTアイテム
    pub items: Vec<FileItem>,
    /// 各行/スコープのパースエラー
    pub errors: Vec<PartialParseError>,
}

/// 部分パースエラー
#[derive(Debug, Clone)]
pub struct PartialParseError {
    /// エラーが発生した行番号（1-based）
    pub line: usize,
    /// エラーメッセージ
    pub message: String,
    /// エラー範囲のSpan（取得できた場合）
    pub span: Option<Span>,
}

/// 行頭パターンからpest Ruleを推論する
pub fn infer_rule_from_line(line: &str) -> Option<Rule> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let first_char = trimmed.chars().next()?;
    match first_char {
        '＊' | '*' => Some(Rule::global_scene_scope),
        '・' | '-' => Some(Rule::local_scene_line),
        '＆' | '&' => Some(Rule::file_attr_line),
        '＠' | '@' => Some(Rule::file_word_line),
        '％' | '%' => Some(Rule::actor_scope),
        '＄' | '$' => Some(Rule::var_set_line),
        '＞' | '>' => Some(Rule::call_scene_line),
        '!' | '！' => Some(Rule::cue_cmd_line),
        '＃' | '#' => Some(Rule::or_comment_eol),
        '`' if trimmed.starts_with("```") => Some(Rule::code_scope),
        _ => {
            // Check if it looks like an action line (identifier followed by colon)
            if trimmed.contains('：') || trimmed.contains(':') {
                Some(Rule::action_line)
            } else {
                None
            }
        }
    }
}

/// スコープ境界マーカーで分割するためのチャンク
///
/// `text` は元ソースの部分スライスであり、`\r\n` などの行終端を **そのまま保持** する。
/// これにより `start_byte` を起点とした定数シフトだけで、チャンク内 span の
/// バイトオフセットを元ソース座標へ正確に復元できる（CRLF でも 1 バイト/行ずれない）。
#[derive(Debug)]
struct SourceChunk<'a> {
    /// チャンク本文。元ソースの部分スライス（行終端を保持）。
    text: &'a str,
    /// チャンク先頭の行番号（1-based、元ソース基準）
    start_line: usize,
    /// チャンク先頭のバイトオフセット（0-based、元ソース基準）
    start_byte: usize,
}

/// ソースをスコープ境界マーカーで分割する
///
/// 各チャンクは元ソースの連続した部分スライスとして切り出され、行終端
/// （`\n` / `\r\n`）を保持する。これにより呼び出し側はチャンク相対 span を
/// `start_byte` / `start_line` の定数シフトでフルソース座標へ補正できる。
fn split_by_scope_markers(source: &str) -> Vec<SourceChunk<'_>> {
    let mut chunks = Vec::new();
    let mut chunk_start_byte: usize = 0;
    let mut chunk_start_line: usize = 1;
    let mut has_content = false;
    let mut byte_pos: usize = 0;
    let mut line_num: usize = 0;

    // `split_inclusive('\n')` は行終端を残したまま行を列挙する（`\r\n` も保持）。
    for raw_line in source.split_inclusive('\n') {
        line_num += 1;
        let line_start_byte = byte_pos;
        byte_pos += raw_line.len();

        let trimmed = raw_line.trim_start();
        let is_scope_boundary = if trimmed.is_empty() {
            false
        } else {
            let first_char = trimmed.chars().next().unwrap_or(' ');
            matches!(
                first_char,
                '＊' | '*' | '％' | '%' | '＆' | '&' | '＠' | '@'
            )
        };

        if is_scope_boundary && has_content {
            // 直前までのチャンクをフラッシュ（境界行は次チャンクの先頭になる）
            chunks.push(SourceChunk {
                text: &source[chunk_start_byte..line_start_byte],
                start_line: chunk_start_line,
                start_byte: chunk_start_byte,
            });
            has_content = false;
        }

        if !has_content {
            chunk_start_byte = line_start_byte;
            chunk_start_line = line_num;
            has_content = true;
        }
    }

    // 末尾チャンクをフラッシュ
    if has_content {
        chunks.push(SourceChunk {
            text: &source[chunk_start_byte..],
            start_line: chunk_start_line,
            start_byte: chunk_start_byte,
        });
    }

    chunks
}

/// 部分パースAPI — 3段階フォールバック戦略
///
/// Phase 1: `parse_str()`による全体パースを試行
/// Phase 2: スコープ境界分割 → 各チャンクを個別にパース
/// Phase 3: 行単位フォールバック → 各行を個別にパース
pub fn parse_str_partial(source: &str) -> PartialParseResult {
    // Phase 1: Full Parse
    if let Ok(pasta_file) = parse_str(source, "<partial>") {
        return PartialParseResult {
            items: pasta_file.items,
            errors: vec![],
        };
    }

    let mut partial_items = Vec::new();
    let mut partial_errors = Vec::new();

    // Phase 2: Scope Boundary Split
    let chunks = split_by_scope_markers(source);
    for chunk in chunks {
        // チャンク相対 span をフルソース座標へ補正するためのオフセット。
        let byte_offset = chunk.start_byte;
        let line_offset = chunk.start_line - 1;

        // Try to infer rule from the first line of the chunk
        let first_line = chunk.text.lines().next().unwrap_or("");
        let rule = infer_rule_from_line(first_line);

        if rule.is_some() {
            // Try parsing with a file-level wrapper for scope rules
            let parse_result = try_parse_chunk(chunk.text);
            match parse_result {
                Ok(mut items) => {
                    // チャンクは部分文字列としてパースされるため span がチャンク相対に
                    // なる。フルソース座標へ補正してから積む。
                    for item in &mut items {
                        shift_file_item(item, byte_offset, line_offset);
                    }
                    partial_items.extend(items);
                    continue;
                }
                Err(_) => {
                    // Phase 3: Line-by-Line Fallback for this chunk
                    for (line, line_num, line_start_byte) in chunk_lines(&chunk) {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if infer_rule_from_line(line).is_some() {
                            match try_parse_chunk(line) {
                                Ok(mut items) => {
                                    // 行単位パースも span が行相対になるため、行頭の
                                    // フルソースバイト / 行番号で補正する。
                                    for item in &mut items {
                                        shift_file_item(item, line_start_byte, line_num - 1);
                                    }
                                    partial_items.extend(items);
                                }
                                Err(e) => {
                                    partial_errors.push(PartialParseError {
                                        line: line_num,
                                        message: e.to_string(),
                                        span: None,
                                    });
                                }
                            }
                        } else {
                            partial_errors.push(PartialParseError {
                                line: line_num,
                                message: "Unable to infer parse rule for line".to_string(),
                                span: None,
                            });
                        }
                    }
                }
            }
        } else {
            // No rule inferred for this chunk, try line-by-line
            for (line, line_num, _) in chunk_lines(&chunk) {
                if line.trim().is_empty() {
                    continue;
                }
                partial_errors.push(PartialParseError {
                    line: line_num,
                    message: "Unable to infer parse rule for line".to_string(),
                    span: None,
                });
            }
        }
    }

    PartialParseResult {
        items: partial_items,
        errors: partial_errors,
    }
}

/// チャンクをファイルとしてパースしてFileItemsを抽出
///
/// grammar.pest の行ルールは末尾 `eol`（NEWLINE）を要求するため、チャンクが
/// 改行で終わらない場合（末尾改行なしのソース最終チャンク等）のみ補う。
/// 補う `\n` は本文の **後ろ** に付くだけなので、実トークンの span バイト
/// オフセットには影響しない。
fn try_parse_chunk(text: &str) -> Result<Vec<FileItem>, String> {
    if text.ends_with('\n') {
        match parse_str(text, "<partial>") {
            Ok(file) => Ok(file.items),
            Err(e) => Err(format!("{}", e)),
        }
    } else {
        let mut owned = String::with_capacity(text.len() + 1);
        owned.push_str(text);
        owned.push('\n');
        match parse_str(&owned, "<partial>") {
            Ok(file) => Ok(file.items),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

/// チャンク内の各行を `(行終端を除いた行文字列, 行番号(1-based, 元ソース基準),
/// 行頭バイトオフセット(0-based, 元ソース基準))` として列挙する。
///
/// `chunk.text` は元ソースの部分スライスなので、行頭バイトはチャンク内の
/// 累積オフセットに `chunk.start_byte` を足すだけで求まる。
fn chunk_lines<'a>(chunk: &SourceChunk<'a>) -> Vec<(&'a str, usize, usize)> {
    let mut out = Vec::new();
    let mut byte_in_chunk: usize = 0;
    for (idx, raw_line) in chunk.text.split_inclusive('\n').enumerate() {
        let line_num = chunk.start_line + idx;
        let line_start_byte = chunk.start_byte + byte_in_chunk;
        byte_in_chunk += raw_line.len();
        // 行終端（\n / \r\n）を除いた本文。行頭は維持する（バイト整合のため）。
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        out.push((line, line_num, line_start_byte));
    }
    out
}

// ============================================================================
// Span shifting — チャンク相対 span をフルソース座標へ補正する
// ============================================================================

/// 単一 span をシフトする。
///
/// - バイトオフセット: `byte_offset` を加算
/// - 行番号: `line_offset` を加算
/// - 列番号: チャンク内の行は元ソースの行と内容が同一のため補正不要
///
/// デフォルト（未初期化）span は補正しない。プレースホルダ span
/// （`Args::empty()` 等）を誤って有効化しないため。
fn shift_span(span: &mut Span, byte_offset: usize, line_offset: usize) {
    if !span.is_valid() {
        return;
    }
    span.start_byte += byte_offset;
    span.end_byte += byte_offset;
    span.start_line += line_offset;
    span.end_line += line_offset;
}

fn shift_file_item(item: &mut FileItem, b: usize, l: usize) {
    match item {
        FileItem::FileAttr(attr) => shift_attr(attr, b, l),
        FileItem::GlobalWord(kw) => shift_keywords(kw, b, l),
        FileItem::GlobalSceneScope(scene) => shift_global_scene(scene, b, l),
        FileItem::ActorScope(actor) => shift_actor_scope(actor, b, l),
    }
}

fn shift_attr(attr: &mut Attr, b: usize, l: usize) {
    shift_span(&mut attr.span, b, l);
}

fn shift_keywords(kw: &mut KeyWords, b: usize, l: usize) {
    shift_span(&mut kw.span, b, l);
}

fn shift_scene_actor(actor: &mut SceneActorItem, b: usize, l: usize) {
    shift_span(&mut actor.span, b, l);
}

fn shift_code_block(cb: &mut CodeBlock, b: usize, l: usize) {
    shift_span(&mut cb.span, b, l);
}

fn shift_args(args: &mut Args, b: usize, l: usize) {
    // Arg / Expr は span を持たないため、Args 自体の span のみ補正。
    shift_span(&mut args.span, b, l);
}

fn shift_action(action: &mut Action, b: usize, l: usize) {
    match action {
        Action::Talk { span, .. }
        | Action::WordRef { span, .. }
        | Action::VarRef { span, .. }
        | Action::SakuraScript { span, .. }
        | Action::Escape { span, .. } => shift_span(span, b, l),
        Action::FnCall { span, args, .. } => {
            shift_span(span, b, l);
            shift_args(args, b, l);
        }
    }
}

fn shift_var_set(vs: &mut VarSet, b: usize, l: usize) {
    // value (SetValue / Expr) は span を持たない。
    shift_span(&mut vs.span, b, l);
}

fn shift_call_scene(cs: &mut CallScene, b: usize, l: usize) {
    shift_span(&mut cs.span, b, l);
    if let Some(args) = &mut cs.args {
        shift_args(args, b, l);
    }
    // target (CallTarget::Dynamic(Expr)) は span を持たない。
}

fn shift_cue(cue: &mut CueCommandNode, b: usize, l: usize) {
    shift_span(&mut cue.span, b, l);
    if let Some(scope) = &mut cue.scope {
        shift_span(&mut scope.span, b, l);
    }
}

fn shift_choice(choice: &mut ChoiceNode, b: usize, l: usize) {
    shift_span(&mut choice.span, b, l);
}

fn shift_action_line(al: &mut ActionLine, b: usize, l: usize) {
    shift_span(&mut al.span, b, l);
    for action in &mut al.actions {
        shift_action(action, b, l);
    }
}

fn shift_continue_action(ca: &mut ContinueAction, b: usize, l: usize) {
    shift_span(&mut ca.span, b, l);
    for action in &mut ca.actions {
        shift_action(action, b, l);
    }
}

fn shift_local_scene_item(item: &mut LocalSceneItem, b: usize, l: usize) {
    match item {
        LocalSceneItem::VarSet(vs) => shift_var_set(vs, b, l),
        LocalSceneItem::CallScene(cs) => shift_call_scene(cs, b, l),
        LocalSceneItem::ActionLine(al) => shift_action_line(al, b, l),
        LocalSceneItem::ContinueAction(ca) => shift_continue_action(ca, b, l),
        LocalSceneItem::CueCommand(cue) => shift_cue(cue, b, l),
        LocalSceneItem::Choice(choice) => shift_choice(choice, b, l),
    }
}

fn shift_local_scene(ls: &mut LocalSceneScope, b: usize, l: usize) {
    shift_span(&mut ls.span, b, l);
    for attr in &mut ls.attrs {
        shift_attr(attr, b, l);
    }
    for item in &mut ls.items {
        shift_local_scene_item(item, b, l);
    }
    for cb in &mut ls.code_blocks {
        shift_code_block(cb, b, l);
    }
}

fn shift_global_scene(scene: &mut GlobalSceneScope, b: usize, l: usize) {
    shift_span(&mut scene.span, b, l);
    for attr in &mut scene.attrs {
        shift_attr(attr, b, l);
    }
    for kw in &mut scene.words {
        shift_keywords(kw, b, l);
    }
    for actor in &mut scene.actors {
        shift_scene_actor(actor, b, l);
    }
    for cb in &mut scene.code_blocks {
        shift_code_block(cb, b, l);
    }
    for ls in &mut scene.local_scenes {
        shift_local_scene(ls, b, l);
    }
}

fn shift_actor_scope(actor: &mut ActorScope, b: usize, l: usize) {
    shift_span(&mut actor.span, b, l);
    for attr in &mut actor.attrs {
        shift_attr(attr, b, l);
    }
    for kw in &mut actor.words {
        shift_keywords(kw, b, l);
    }
    for vs in &mut actor.var_sets {
        shift_var_set(vs, b, l);
    }
    for cb in &mut actor.code_blocks {
        shift_code_block(cb, b, l);
    }
}
