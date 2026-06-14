use super::*;
use crate::config::LineEnding;
use pasta_dsl::parser::{
    Action, ActionLine, CallScene, CallTarget, ChoiceNode, CodeBlock, CueArgToken,
    CueCommandNode, KeyWords, Span,
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
        gen_to_string(|cg| cg.generate_local_scene(&local_scene(None), 0, &scene_actors()));

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
        cg.generate_local_scene(&local_scene(Some("会話")), 2, &scene_actors())
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
    let text = gen_to_string(|cg| cg.generate_local_scene(&local_scene(None), 0, &[]));
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
