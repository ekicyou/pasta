use super::*;
use crate::code_gen::source_map::SourceMapSink;
use crate::config::LineEnding;
use crate::context::TranspileContext;
use pasta_dsl::parser::{
    Action, ActionLine, CallScene, CallTarget, ChoiceNode, CodeBlock, CueArgToken,
    CueCommandNode, GlobalSceneScope, KeyWords, Span,
};

fn gen_to_string<F>(f: F) -> String
where
    F: FnOnce(&mut LuaCodeGenerator<'_, Vec<u8>>) -> Result<(), TranspileError>,
{
    let mut output = Vec::new();
    {
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        f(&mut cg).unwrap();
    }
    String::from_utf8(output).unwrap()
}

/// Test sink capturing each `record_scene(join_key, span)` notification as
/// `(join_key, span.start_line)`. Leaves `record_line` as the only required
/// method; `record_scene` is overridden to observe scene recording (task 2.1).
#[derive(Default)]
struct CapturingSceneSink {
    scenes: Vec<(String, u32)>,
}

impl SourceMapSink for CapturingSceneSink {
    fn record_line(&mut self, _lua_line: u32, _pasta_line: u32) {}
    fn record_scene(&mut self, scene_join_key: &str, span: Span) {
        self.scenes.push((scene_join_key.to_string(), span.start_line as u32));
    }
}

fn talk_line(actor: &str, text: &str) -> LocalSceneItem {
    LocalSceneItem::ActionLine(ActionLine {
        actor: actor.to_string(),
        actions: vec![Action::Talk {
            text: text.to_string(),
            span: Span::default(),
        }],
        span: Span::default(),
    })
}

fn call_item(target: &str) -> LocalSceneItem {
    LocalSceneItem::CallScene(CallScene {
        target: CallTarget::Static(target.to_string()),
        args: None,
        span: Span::default(),
    })
}

// ------------------------------------------------------------------
// generate_actor filters
// ------------------------------------------------------------------

/// Actor generation skips empty word definitions and expands ONLY
/// `lua`-language code blocks (other languages are silently dropped).
#[test]
fn actor_skips_empty_words_and_non_lua_code_blocks() {
    let actor = ActorScope {
        name: "さくら".to_string(),
        attrs: vec![],
        words: vec![
            KeyWords {
                names: vec!["空".to_string()],
                words: vec![], // empty -> skipped entirely
                span: Span::default(),
            },
            KeyWords {
                names: vec!["通常".to_string()],
                words: vec!["\\s[0]".to_string()],
                span: Span::default(),
            },
        ],
        var_sets: vec![],
        code_blocks: vec![
            CodeBlock {
                language: Some("rust".to_string()),
                content: "fn ignored() {}".to_string(),
                span: Span::default(),
            },
            CodeBlock {
                language: Some("lua".to_string()),
                content: "function ACTOR.時刻(act)\nend".to_string(),
                span: Span::default(),
            },
            CodeBlock {
                language: None,
                content: "also ignored".to_string(),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let text = gen_to_string(|cg| cg.generate_actor(&actor));

    assert!(
        text.contains("local ACTOR = PASTA.create_actor(\"さくら\")"),
        "actor header missing: {}",
        text
    );
    assert!(
        !text.contains("ACTOR:create_word(\"空\")"),
        "empty word definition must be skipped: {}",
        text
    );
    assert!(
        text.contains("ACTOR:create_word(\"通常\")"),
        "non-empty word definition must be emitted: {}",
        text
    );
    assert!(
        text.contains("function ACTOR.時刻(act)"),
        "lua code block must be expanded: {}",
        text
    );
    assert!(
        !text.contains("fn ignored") && !text.contains("also ignored"),
        "non-lua code blocks must be dropped: {}",
        text
    );
    assert!(text.ends_with("end\n\n"), "block closed via end_block: {}", text);
}

// ------------------------------------------------------------------
// generate_local_scene: naming and spot initialization
// ------------------------------------------------------------------

fn local_scene(name: Option<&str>) -> LocalSceneScope {
    LocalSceneScope {
        name: name.map(|s| s.to_string()),
        attrs: vec![],
        items: vec![],
        code_blocks: vec![],
        span: Span::default(),
    }
}

fn scene_actors() -> Vec<SceneActorItem> {
    vec![
        SceneActorItem {
            name: "さくら".to_string(),
            number: 0,
            span: Span::default(),
        },
        SceneActorItem {
            name: "うにゅう".to_string(),
            number: 10,
            span: Span::default(),
        },
    ]
}

/// The anonymous start scene (name=None, counter=0) is named `__start__`
/// and emits actor initialization: `clear_spot` THEN each `set_spot` in
/// declaration order with the precomputed numbers.
#[test]
fn start_scene_emits_clear_spot_then_set_spots_in_order() {
    let text =
        gen_to_string(|cg| cg.generate_local_scene(&local_scene(None), 0, &scene_actors(), "会話#1"));

    assert!(
        text.contains("function SCENE.__start__(act, ...)"),
        "start scene fn name: {}",
        text
    );
    let clear = text.find("act:clear_spot()").expect("clear_spot present");
    let spot1 = text
        .find("act:set_spot(\"さくら\", 0)")
        .expect("first set_spot present");
    let spot2 = text
        .find("act:set_spot(\"うにゅう\", 10)")
        .expect("second set_spot present");
    assert!(
        clear < spot1 && spot1 < spot2,
        "order must be clear_spot -> set_spot(さくら) -> set_spot(うにゅう): {}",
        text
    );
}

/// A named local scene uses `{sanitized}_{counter}` and does NOT emit the
/// spot-initialization block even when actors exist (counter != 0).
#[test]
fn named_scene_uses_counter_suffix_and_skips_spot_init() {
    let text = gen_to_string(|cg| {
        cg.generate_local_scene(&local_scene(Some("会話")), 2, &scene_actors(), "会話#1")
    });

    assert!(
        text.contains("function SCENE.会話_2(act, ...)"),
        "per-name counter suffix: {}",
        text
    );
    assert!(
        !text.contains("clear_spot") && !text.contains("set_spot"),
        "spot init is __start__-only: {}",
        text
    );
    // Session initialization is always present.
    assert!(text.contains("local args = { ... }"), "{}", text);
    assert!(text.contains("local save, var = act:init_scene(SCENE)"), "{}", text);
}

/// A start scene with NO actors emits no spot block at all.
#[test]
fn start_scene_without_actors_emits_no_spot_block() {
    let text = gen_to_string(|cg| cg.generate_local_scene(&local_scene(None), 0, &[], "会話#1"));
    assert!(
        !text.contains("clear_spot") && !text.contains("set_spot"),
        "no actors -> no spot init: {}",
        text
    );
}

// ------------------------------------------------------------------
// Tail call optimization in generate_local_scene_items
// ------------------------------------------------------------------

/// When the LAST item is a scene call, it gets the `return ` TCO prefix;
/// a scene call in non-tail position does not.
#[test]
fn tco_return_only_for_trailing_call_scene() {
    let tail = gen_to_string(|cg| {
        cg.generate_local_scene_items(&[talk_line("さくら", "やあ"), call_item("次")])
    });
    assert!(
        tail.contains("return act:call(SCENE.__global_name__, \"次\""),
        "trailing call must be a tail call: {}",
        tail
    );

    let non_tail = gen_to_string(|cg| {
        cg.generate_local_scene_items(&[call_item("次"), talk_line("さくら", "やあ")])
    });
    assert!(
        non_tail.contains("act:call(SCENE.__global_name__, \"次\"")
            && !non_tail.contains("return act:call"),
        "non-trailing call must NOT get return prefix: {}",
        non_tail
    );
}

// ------------------------------------------------------------------
// Choice and !select cue command
// ------------------------------------------------------------------

fn choice_item(target: &str, label: Option<&str>) -> LocalSceneItem {
    LocalSceneItem::Choice(ChoiceNode {
        target: target.to_string(),
        label: label.map(|s| s.to_string()),
        span: Span::new(1, 1, 1, 5, 0, 9),
    })
}

/// Choice display text: explicit label wins; otherwise the target id is
/// reused as the display string.
#[test]
fn choice_uses_label_or_falls_back_to_target() {
    let labeled =
        gen_to_string(|cg| cg.generate_local_scene_items(&[choice_item("はい", Some("Yes"))]));
    assert_eq!(labeled, "act:choice(\"はい\", \"Yes\")\n");

    let fallback =
        gen_to_string(|cg| cg.generate_local_scene_items(&[choice_item("はい", None)]));
    assert_eq!(fallback, "act:choice(\"はい\", \"はい\")\n");
}

fn select_cmd(command: &str, args: Vec<CueArgToken>) -> LocalSceneItem {
    LocalSceneItem::CueCommand(CueCommandNode {
        command: command.to_string(),
        scope: None,
        args,
        span: Span::new(1, 1, 1, 8, 0, 12),
    })
}

/// `!select` argument rendering: a float WITH a fractional part is kept
/// as-is (`2.5`), while a non-numeric first arg renders as `nil`.
/// (Integer and fract==0 float cases are covered by the transpile
/// integration tests.)
#[test]
fn choice_timeout_keeps_fractional_float_and_defaults_to_nil() {
    let fractional = gen_to_string(|cg| {
        cg.generate_local_scene_items(&[select_cmd("select", vec![CueArgToken::Float(2.5)])])
    });
    assert_eq!(fractional, "act:choice_timeout(2.5)\n");

    let ident_arg = gen_to_string(|cg| {
        cg.generate_local_scene_items(&[select_cmd(
            "select",
            vec![CueArgToken::Ident("fast".to_string())],
        )])
    });
    assert_eq!(ident_arg, "act:choice_timeout(nil)\n");
}

/// Non-`select` cue commands generate NO Lua output (handled elsewhere).
#[test]
fn non_select_cue_command_emits_nothing() {
    let text = gen_to_string(|cg| {
        cg.generate_local_scene_items(&[select_cmd(
            "emote",
            vec![CueArgToken::Ident("smile".to_string())],
        )])
    });
    assert!(text.is_empty(), "non-select cue must emit nothing: {:?}", text);
}

// ------------------------------------------------------------------
// Scene recording for source-map join (task 2.1)
//
// `generate_global_scene` / `generate_local_scene` must funnel each scene's
// deterministic join key + `.pasta` span to the sink via `record_scene`,
// adjacent to the existing `record_span` header recording. The join_key encoding
// (see scope_gen.rs `record_scene_span` doc) carries: global/local kind + nesting
// level, the parent-global reference for locals, and per-base occurrence order
// (globals' occurrence == runtime `create_scene` counter order). The captured
// (join_key, start_line) pairs let task 2.2 reconstruct end_line/level and join
// the runtime identity (e.g. `会話1`).
// ------------------------------------------------------------------

/// Build a named local scene with an explicit span (start_line carried into the
/// join-key capture). `end_byte` is non-zero so the span is `is_valid()`.
fn named_local_with_span(name: &str, start_line: usize) -> LocalSceneScope {
    LocalSceneScope {
        name: Some(name.to_string()),
        attrs: vec![],
        items: vec![],
        code_blocks: vec![],
        span: Span::new(start_line, 1, start_line + 1, 1, (start_line - 1) * 10, start_line * 10),
    }
}

/// A global scene with a valid header span at `start_line` and the given locals.
fn global_scene_with(name: &str, start_line: usize, locals: Vec<LocalSceneScope>) -> GlobalSceneScope {
    let mut scene = GlobalSceneScope::new(name.to_string());
    scene.span = Span::new(start_line, 1, start_line + 1, 1, (start_line - 1) * 10, start_line * 10);
    scene.local_scenes = locals;
    scene
}

/// `record_scene` is invoked for BOTH the global header and each local scene
/// function, with deterministic join keys and the `.pasta` header `start_line`.
///
/// - Global key encodes kind (`G`), sanitized base, and per-base occurrence order
///   (the `counter` argument == runtime `create_scene` order).
/// - Local key encodes kind (`L`), the parent base + parent occurrence, and the
///   code_gen-computed local fn name (`{sanitize}_{counter}`), so nesting level
///   and parent reference are recoverable at finalize (task 2.2).
#[test]
fn global_and_local_scenes_record_join_keys_and_header_lines() {
    let mut sink = CapturingSceneSink::default();
    let ctx = TranspileContext::new();
    let attrs = std::collections::HashMap::new();

    // 会話 at .pasta line 3, containing a named local 挨拶 at line 5, plus the
    // anonymous start scene (line == global header span line, counter 0).
    let scene = global_scene_with(
        "会話",
        3,
        vec![
            LocalSceneScope {
                name: None,
                attrs: vec![],
                items: vec![],
                code_blocks: vec![],
                span: Span::new(3, 1, 4, 1, 20, 30),
            },
            named_local_with_span("挨拶", 5),
        ],
    );

    {
        let mut output = Vec::new();
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        cg.set_source_map(&mut sink);
        // counter == 1: first occurrence of base 会話 (matches runtime 会話1).
        cg.generate_global_scene(&scene, 1, &ctx, &attrs).unwrap();
    }

    // Global header recorded first, then locals in source order.
    assert_eq!(
        sink.scenes,
        vec![
            ("G:会話#1".to_string(), 3),
            ("L:会話#1:__start__".to_string(), 3),
            ("L:会話#1:挨拶_1".to_string(), 5),
        ],
        "captured (join_key, start_line) entries: {:?}",
        sink.scenes
    );
}

/// Two global scenes sharing a base name get distinct per-base occurrence
/// counters in their join keys, matching the runtime `create_scene` counter
/// order (会話1, 会話2). This is the property task 2.2 relies on to join the
/// runtime identity back to the recorded span.
#[test]
fn same_base_global_scenes_get_incrementing_occurrence_in_join_key() {
    let mut sink = CapturingSceneSink::default();
    let ctx = TranspileContext::new();
    let attrs = std::collections::HashMap::new();

    let first = global_scene_with("会話", 3, vec![]);
    let second = global_scene_with("会話", 10, vec![]);

    {
        let mut output = Vec::new();
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        cg.set_source_map(&mut sink);
        cg.generate_global_scene(&first, 1, &ctx, &attrs).unwrap();
        cg.generate_global_scene(&second, 2, &ctx, &attrs).unwrap();
    }

    assert_eq!(
        sink.scenes,
        vec![("G:会話#1".to_string(), 3), ("G:会話#2".to_string(), 10)],
        "per-base occurrence order must match runtime counter order: {:?}",
        sink.scenes
    );
}
