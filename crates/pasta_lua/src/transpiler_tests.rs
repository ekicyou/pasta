use super::*;
use pasta_dsl::parser::{ActorScope, GlobalSceneScope, KeyWords, LocalSceneScope, Span};
use std::path::PathBuf;

fn create_simple_actor(name: &str) -> ActorScope {
    ActorScope {
        name: name.to_string(),
        attrs: vec![],
        words: vec![KeyWords {
            names: vec!["通常".to_string()],
            words: vec!["\\s[0]".to_string()],
            weight_spans: vec![],
            span: Span::default(),
        }],
        var_sets: vec![],
        code_blocks: vec![],
        span: Span::default(),
    }
}

fn create_simple_scene(name: &str) -> GlobalSceneScope {
    GlobalSceneScope {
        name: name.to_string(),
        is_continuation: false,
        attrs: vec![],
        words: vec![],
        actors: vec![],
        code_blocks: vec![],
        local_scenes: vec![LocalSceneScope::start()],
        span: Span::default(),
    }
}

fn create_scene_with_words(
    name: &str,
    word_name: &str,
    word_values: Vec<&str>,
) -> GlobalSceneScope {
    GlobalSceneScope {
        name: name.to_string(),
        is_continuation: false,
        attrs: vec![],
        words: vec![KeyWords {
            names: vec![word_name.to_string()],
            words: word_values.iter().map(|s| s.to_string()).collect(),
            weight_spans: vec![],
            span: Span::default(),
        }],
        actors: vec![],
        code_blocks: vec![],
        local_scenes: vec![LocalSceneScope::start()],
        span: Span::default(),
    }
}

fn create_scene_with_local(name: &str, local_name: &str) -> GlobalSceneScope {
    GlobalSceneScope {
        name: name.to_string(),
        is_continuation: false,
        attrs: vec![],
        words: vec![],
        actors: vec![],
        code_blocks: vec![],
        local_scenes: vec![
            LocalSceneScope::start(),
            LocalSceneScope::named(local_name.to_string()),
        ],
        span: Span::default(),
    }
}

/// Create a PastaFile with actors and scenes
fn create_pasta_file(actors: Vec<ActorScope>, scenes: Vec<GlobalSceneScope>) -> PastaFile {
    let mut items: Vec<FileItem> = Vec::new();
    for actor in actors {
        items.push(FileItem::ActorScope(actor));
    }
    for scene in scenes {
        items.push(FileItem::GlobalSceneScope(scene));
    }
    PastaFile {
        path: PathBuf::from("test.pasta"),
        items,
    }
}

#[test]
fn test_transpile_empty() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let file = create_pasta_file(vec![], vec![]);

    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();
    assert!(lua_code.contains("local PASTA = require \"pasta\""));
}

#[test]
fn test_transpile_actor() {
    let transpiler = LuaTranspiler::default();
    let actors = vec![create_simple_actor("さくら")];
    let file = create_pasta_file(actors, vec![]);
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();
    assert!(lua_code.contains("PASTA.create_actor(\"さくら\")"));
    // Symmetric API: ACTOR:create_word(key):entry(...) (actor-word-dictionary)
    assert!(lua_code.contains("ACTOR:create_word(\"通常\"):entry([=[\\s[0]]=])"));
}

#[test]
fn test_transpile_scene() {
    let transpiler = LuaTranspiler::default();
    let scenes = vec![create_simple_scene("メイン")];
    let file = create_pasta_file(vec![], scenes);
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();
    // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
    assert!(lua_code.contains("PASTA.create_scene(\"メイン\")"));
    assert!(lua_code.contains("function SCENE.__start__(act, ...)"));
}

#[test]
fn test_transpile_multiple_scenes() {
    let transpiler = LuaTranspiler::default();
    let scenes = vec![
        create_simple_scene("メイン"),
        create_simple_scene("会話分岐"),
    ];
    let file = create_pasta_file(vec![], scenes);
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();
    // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
    // Both scenes use base name only - Lua will add counters at runtime
    assert!(lua_code.contains("PASTA.create_scene(\"メイン\")"));
    assert!(lua_code.contains("PASTA.create_scene(\"会話分岐\")"));
}

// Task 3.1: SceneRegistry integration tests
#[test]
fn test_transpile_registers_global_scene() {
    let transpiler = LuaTranspiler::default();
    let scenes = vec![create_simple_scene("メイン")];
    let file = create_pasta_file(vec![], scenes);
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let registered_scenes = context.scene_registry.all_scenes();
    assert_eq!(registered_scenes.len(), 1);
    assert_eq!(registered_scenes[0].name, "メイン");
    assert_eq!(registered_scenes[0].id, 1);
}

#[test]
fn test_transpile_registers_local_scene() {
    let transpiler = LuaTranspiler::default();
    let scenes = vec![create_scene_with_local("メイン", "自己紹介")];
    let file = create_pasta_file(vec![], scenes);
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let registered_scenes = context.scene_registry.all_scenes();
    assert_eq!(registered_scenes.len(), 2); // global + local
    assert_eq!(registered_scenes[1].name, "自己紹介");
}

// Task 3.2: WordDefRegistry integration tests
#[test]
fn test_transpile_registers_local_words() {
    let transpiler = LuaTranspiler::default();
    let scenes = vec![create_scene_with_words(
        "メイン",
        "場所",
        vec!["東京", "大阪"],
    )];
    let file = create_pasta_file(vec![], scenes);
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let entries = context.word_registry.all_entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].key.contains("場所"));
}

// MAJOR-2.2: GlobalWord processing test
#[test]
fn test_transpile_with_global_words() {
    let transpiler = LuaTranspiler::default();
    let global_words = KeyWords {
        names: vec!["挨拶".to_string()],
        words: vec!["こんにちは".to_string(), "やあ".to_string()],
        weight_spans: vec![],
        span: Span::default(),
    };
    let scenes = vec![create_simple_scene("メイン")];

    // Create PastaFile with GlobalWord item
    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![
            FileItem::GlobalWord(global_words),
            FileItem::GlobalSceneScope(scenes.into_iter().next().unwrap()),
        ],
    };
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let entries = context.word_registry.all_entries();
    // グローバル単語 + シーン内ローカル単語はないので1つ
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "挨拶");
}

// Task 2.3: ActorScope word registration test
#[test]
fn test_transpile_registers_actor_words() {
    let transpiler = LuaTranspiler::default();
    let actor = ActorScope {
        name: "さくら".to_string(),
        attrs: vec![],
        words: vec![
            KeyWords {
                names: vec!["通常".to_string()],
                words: vec!["\\s[0]".to_string(), "\\s[1]".to_string()],
                weight_spans: vec![],
                span: Span::default(),
            },
            KeyWords {
                names: vec!["照れ".to_string()],
                words: vec!["\\s[2]".to_string()],
                weight_spans: vec![],
                span: Span::default(),
            },
        ],
        var_sets: vec![],
        code_blocks: vec![],
        span: Span::default(),
    };
    let file = create_pasta_file(vec![actor], vec![]);
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let entries = context.word_registry.all_entries();
    assert_eq!(entries.len(), 2, "Expected 2 actor word entries");
    // Check key format: :__actor_{name}__:{word}
    assert_eq!(entries[0].key, ":__actor_さくら__:通常");
    assert_eq!(entries[1].key, ":__actor_さくら__:照れ");
    // Check values
    assert_eq!(entries[0].values, vec!["\\s[0]", "\\s[1]"]);
    assert_eq!(entries[1].values, vec!["\\s[2]"]);
}

// word-multi-key: 複数キーの統合テスト
#[test]
fn test_transpile_multi_key_global_word_registration() {
    let transpiler = LuaTranspiler::default();
    let global_words = KeyWords {
        names: vec!["女性".to_string(), "水の妖精".to_string()],
        words: vec!["水無灯里".to_string(), "アリス・キャロル".to_string()],
        weight_spans: vec![],
        span: Span::default(),
    };

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![FileItem::GlobalWord(global_words)],
    };
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let entries = context.word_registry.all_entries();
    assert_eq!(entries.len(), 2, "複数キーでそれぞれ登録されること");
    assert_eq!(entries[0].key, "女性");
    assert_eq!(entries[1].key, "水の妖精");
    assert_eq!(entries[0].values, vec!["水無灯里", "アリス・キャロル"]);
    assert_eq!(entries[1].values, vec!["水無灯里", "アリス・キャロル"]);
}

#[test]
fn test_transpile_multi_key_actor_word_registration() {
    let transpiler = LuaTranspiler::default();
    let actor = ActorScope {
        name: "さくら".to_string(),
        attrs: vec![],
        words: vec![KeyWords {
            names: vec!["通常".to_string(), "普通".to_string()],
            words: vec!["\\s[0]".to_string()],
            weight_spans: vec![],
            span: Span::default(),
        }],
        var_sets: vec![],
        code_blocks: vec![],
        span: Span::default(),
    };
    let file = create_pasta_file(vec![actor], vec![]);
    let mut output = Vec::new();

    let context = transpiler.transpile(&file, &mut output).unwrap();

    let entries = context.word_registry.all_entries();
    assert_eq!(entries.len(), 2, "複数キーでそれぞれ登録されること");
    assert_eq!(entries[0].key, ":__actor_さくら__:通常");
    assert_eq!(entries[1].key, ":__actor_さくら__:普通");
}

#[test]
fn test_transpile_multi_key_global_word_lua_output() {
    let transpiler = LuaTranspiler::default();
    let global_words = KeyWords {
        names: vec!["女性".to_string(), "水の妖精".to_string()],
        words: vec!["水無灯里".to_string(), "アリス・キャロル".to_string()],
        weight_spans: vec![],
        span: Span::default(),
    };

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![FileItem::GlobalWord(global_words)],
    };
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // 各キーに対して create_word が出力されること
    assert!(
        lua_code.contains("PASTA.create_word(\"女性\"):entry(\"水無灯里\", \"アリス・キャロル\")"),
        "女性キーの create_word が出力されること: {lua_code}"
    );
    assert!(
        lua_code
            .contains("PASTA.create_word(\"水の妖精\"):entry(\"水無灯里\", \"アリス・キャロル\")"),
        "水の妖精キーの create_word が出力されること: {lua_code}"
    );
}

#[test]
fn test_transpile_multi_key_actor_word_lua_output() {
    let transpiler = LuaTranspiler::default();
    let actor = ActorScope {
        name: "さくら".to_string(),
        attrs: vec![],
        words: vec![KeyWords {
            names: vec!["通常".to_string(), "普通".to_string()],
            words: vec!["\\s[0]".to_string()],
            weight_spans: vec![],
            span: Span::default(),
        }],
        var_sets: vec![],
        code_blocks: vec![],
        span: Span::default(),
    };
    let file = create_pasta_file(vec![actor], vec![]);
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains("ACTOR:create_word(\"通常\"):entry([=[\\s[0]]=])"),
        "通常キーの create_word: {lua_code}"
    );
    assert!(
        lua_code.contains("ACTOR:create_word(\"普通\"):entry([=[\\s[0]]=])"),
        "普通キーの create_word: {lua_code}"
    );
}

/// 登録された各シーンの (fn_name, 属性の字句) を登録順に返す
fn registered_attrs(src: &str) -> Vec<(String, Vec<String>)> {
    use pasta_core::registry::SceneAttr;

    let file = pasta_dsl::parse_str(src, "test.pasta").unwrap();
    let context = LuaTranspiler::default()
        .transpile(&file, &mut Vec::new())
        .unwrap();
    context
        .scene_registry
        .all_scenes()
        .into_iter()
        .map(|e| {
            let attrs = e
                .attributes
                .iter()
                .map(|a| match a {
                    SceneAttr::Decl { key, value } => format!("{key}={value}"),
                    SceneAttr::Cond { key, op, value } => format!("{key}{}{value}", op.as_str()),
                    SceneAttr::Flag { key } => key.clone(),
                })
                .collect();
            (e.fn_name.clone(), attrs)
        })
        .collect()
}

#[test]
fn test_transpile_file_attr_line_replaces_previous_set() {
    // 後のファイル属性の行は前の組を捨てる（累積しない・要件 3.2）
    let got = registered_attrs(
        "＆author：Alice＆version：1\n＊メイン\n　さくら：やあ\n\n＆author：Bob\n＊メイン\n　さくら：ほい\n",
    );
    assert_eq!(
        got,
        vec![
            (
                "メイン_1::__start__".to_string(),
                vec!["author=Alice".to_string(), "version=1".to_string()]
            ),
            (
                "メイン_2::__start__".to_string(),
                vec!["author=Bob".to_string()]
            ),
        ]
    );
}

#[test]
fn test_transpile_same_declared_name_keeps_attrs_per_declaration() {
    // 同じ宣言名の 2 つの宣言の属性は混ざらない（要件 1.7）。ローカルは受け継がず宣言行の中だけ畳む（要件 3.8）
    let got = registered_attrs(
        "＆f：1\n＊会話＆k：1\n　さくら：やあ\n　・選択＆p：3＆p：4\n　さくら：ほい\n＊会話＆＄＊x≧5\n　さくら：よう\n",
    );
    assert_eq!(
        got,
        vec![
            (
                "会話_1::__start__".to_string(),
                vec!["f=1".to_string(), "k=1".to_string()]
            ),
            ("会話_1::選択_1".to_string(), vec!["p=4".to_string()]),
            (
                "会話_2::__start__".to_string(),
                vec!["f=1".to_string(), "＄＊x>=5".to_string()]
            ),
        ]
    );
}

#[test]
fn test_transpile_actor_attrs_are_not_registered() {
    // アクター辞書の配下の属性はどのシーンにも入らない（要件 5.6）
    let got = registered_attrs("％さくら\n　＆x：9\n＊会話\n　さくら：やあ\n");
    assert_eq!(got, vec![("会話_1::__start__".to_string(), vec![])]);
}

/// A writer that always fails, to exercise the IoError conversion path.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "simulated write failure",
        ))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_transpile_writer_failure_yields_io_error() {
    let transpiler = LuaTranspiler::default();
    let file = create_pasta_file(vec![], vec![create_simple_scene("メイン")]);

    match transpiler.transpile(&file, &mut FailingWriter) {
        Err(TranspileError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe);
        }
        Err(other) => panic!("expected IoError, got: {:?}", other),
        Ok(_) => panic!("expected IoError, but transpile succeeded"),
    }
}

/// The transpiled bytes always use LF line endings.
#[test]
fn test_transpile_output_is_lf_only() {
    let transpiler = LuaTranspiler::default();
    let file = create_pasta_file(vec![], vec![create_simple_scene("メイン")]);
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        !lua_code.contains('\r'),
        "normalized output must not contain CR"
    );
    assert!(lua_code.ends_with('\n'));
    assert!(!lua_code.ends_with("\n\n"), "exactly one trailing newline");
}

#[test]
fn test_transpile_single_key_backward_compat_lua_output() {
    let transpiler = LuaTranspiler::default();
    let global_words = KeyWords {
        names: vec!["挨拶".to_string()],
        words: vec!["こんにちは".to_string()],
        weight_spans: vec![],
        span: Span::default(),
    };

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![FileItem::GlobalWord(global_words)],
    };
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // 単一キーでも従来と同一の出力
    assert!(
        lua_code.contains("PASTA.create_word(\"挨拶\"):entry(\"こんにちは\")"),
        "単一キーの出力: {lua_code}"
    );
}
