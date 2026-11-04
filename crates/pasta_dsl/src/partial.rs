//! 色のための部分解析。
//!
//! 全体が解けないときに、ソースを字下げする前の行頭で塊に区切り、塊ごとに解く。
//! 解けた塊の AST（span は元のソースの座標）と、解けない塊の行の範囲を返す。

use std::ops::Range;

use crate::parser::ast::{
    Action, ActionLine, ActorScope, Args, Attr, CallScene, ChoiceNode, CodeBlock, ContinueAction,
    CueCommandNode, FileItem, GlobalSceneScope, KeyWords, LocalSceneItem, LocalSceneScope,
    SceneActorItem, Span, VarSet,
};
use crate::parser::parse_str;

/// 部分解析の結果
#[derive(Debug, Clone)]
pub struct PartialParseResult {
    /// 解けた塊の AST（span は元のソースの座標）
    pub items: Vec<FileItem>,
    /// 解けない塊の行（1 始まり・半開区間）。昇順で重ならない
    pub failed_lines: Vec<Range<usize>>,
}

/// 塊の区切りになる行か。字下げする前の行頭の 1 文字で決める。
///
/// 名前の無い `＊` の行（続きのシーン）は、前の名前付きのシーンと一緒に解くため区切りにしない。
/// shortcut: Lua ブロックの本文の 0 桁目の `＊`・`％`・`＆`・`＠` も区切りにする。
/// Lua の文の頭には来ない文字なので追わない。困る例が出たら、フェンスを数えて区切りを止める。
fn is_chunk_boundary(raw_line: &str) -> bool {
    let mut chars = raw_line.chars();
    match chars.next() {
        Some('＊' | '*') => {
            let rest = chars.as_str().trim_start();
            !(rest.is_empty() || rest.starts_with(['＃', '#']))
        }
        Some('％' | '%' | '＆' | '&' | '＠' | '@') => true,
        _ => false,
    }
}

/// ソースを区切りの行で塊に分ける。
///
/// 塊は元のソースの部分の文字列で、行の終わり（`\n`・`\r\n`）を保つ。
/// 戻り値は `(塊の本文, 先頭の行番号（1 始まり）, 先頭のバイト位置)`。
fn split_chunks(source: &str) -> Vec<(&str, usize, usize)> {
    let mut chunks = Vec::new();
    let mut chunk_start_byte = 0;
    let mut chunk_start_line = 1;
    let mut byte_pos = 0;

    for (idx, raw_line) in source.split_inclusive('\n').enumerate() {
        if idx > 0 && is_chunk_boundary(raw_line) {
            chunks.push((
                &source[chunk_start_byte..byte_pos],
                chunk_start_line,
                chunk_start_byte,
            ));
            chunk_start_byte = byte_pos;
            chunk_start_line = idx + 1;
        }
        byte_pos += raw_line.len();
    }
    if chunk_start_byte < source.len() {
        chunks.push((
            &source[chunk_start_byte..],
            chunk_start_line,
            chunk_start_byte,
        ));
    }
    chunks
}

/// 塊ごとに解いて、色のための AST と解けない塊の行の範囲を返す。
///
/// 全体を解く段は持たない。呼び手が全体の失敗を知っているときに呼ぶ
/// （全体が正しいソースでも、塊ごとに解いた同じ AST を返す）。
pub fn parse_str_partial(source: &str) -> PartialParseResult {
    let mut items = Vec::new();
    let mut failed_lines = Vec::new();

    for (text, start_line, start_byte) in split_chunks(source) {
        match try_parse_chunk(text) {
            Some(mut chunk_items) => {
                for item in &mut chunk_items {
                    shift_file_item(item, start_byte, start_line - 1);
                }
                items.extend(chunk_items);
            }
            None => {
                let line_count = text.split_inclusive('\n').count();
                failed_lines.push(start_line..start_line + line_count);
            }
        }
    }

    PartialParseResult {
        items,
        failed_lines,
    }
}

/// 塊をファイルとして解いて FileItem を取り出す。
///
/// 文法の行の規則は末尾の改行を要るため、改行で終わらない塊（改行で終わらない
/// ソースの最後の塊）にだけ補う。補う `\n` は本文の後ろに付くので、span には響かない。
fn try_parse_chunk(text: &str) -> Option<Vec<FileItem>> {
    let parsed = if text.ends_with('\n') {
        parse_str(text, "<partial>")
    } else {
        parse_str(&format!("{text}\n"), "<partial>")
    };
    parsed.ok().map(|file| file.items)
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
    for s in &mut kw.weight_spans {
        shift_span(s, b, l);
    }
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
        | Action::Escape { span, .. }
        | Action::DynamicWordRef { span, .. } => shift_span(span, b, l),
        Action::FnCall { span, args, .. } | Action::DynamicFnCall { span, args, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::VarScope;

    #[test]
    fn shift_action_shifts_dynamic_variants() {
        let span = Span::new(1, 1, 1, 5, 0, 10);
        let mut word = Action::DynamicWordRef {
            var_name: "x".into(),
            var_scope: VarScope::Local,
            span,
        };
        shift_action(&mut word, 100, 3);
        let Action::DynamicWordRef { span: s, .. } = word else {
            unreachable!()
        };
        assert_eq!((s.start_byte, s.end_byte, s.start_line), (100, 110, 4));

        let mut call = Action::DynamicFnCall {
            var_name: "f".into(),
            var_scope: VarScope::Global,
            args: Args {
                items: vec![],
                span: Span::new(1, 3, 1, 5, 4, 8),
            },
            span,
        };
        shift_action(&mut call, 100, 3);
        let Action::DynamicFnCall { span: s, args, .. } = call else {
            unreachable!()
        };
        assert_eq!((s.start_byte, s.end_byte, s.start_line), (100, 110, 4));
        assert_eq!((args.span.start_byte, args.span.end_byte), (104, 108));
        assert_eq!(args.span.end_line, 4);
    }

    #[test]
    fn shift_keywords_shifts_weight_spans() {
        let mut kw = KeyWords {
            names: vec!["挨拶".into()],
            words: vec!["こんにちは".into()],
            weight_spans: vec![Span::new(1, 5, 1, 7, 4, 8)],
            span: Span::new(1, 1, 1, 10, 0, 20),
        };
        shift_keywords(&mut kw, 100, 3);
        let w = kw.weight_spans[0];
        assert_eq!(
            (w.start_byte, w.end_byte, w.start_line, w.end_line),
            (104, 108, 4, 4)
        );
        assert_eq!((kw.span.start_byte, kw.span.end_byte), (100, 120));
    }
}
