//! 属性の 3 種の形（宣言・条件・フラグ）と、名前なしの `＊` 行への付記のパーステスト
//!
//! Requirements 1.1〜1.6, 2.1, 2.2, 2.3, 2.4, 2.6, 2.7, 4.1〜4.4, 4.6

use pasta_dsl::parser::*;

/// 1 つ目のグローバルシーンの初期部の属性
fn scene_attrs(src: &str) -> Vec<Attr> {
    let file = parse_str(src, "test.pasta").unwrap();
    file.items
        .iter()
        .find_map(|item| match item {
            FileItem::GlobalSceneScope(scene) => Some(scene.attrs.clone()),
            _ => None,
        })
        .unwrap()
}

fn init_attr(attr: &str) -> Attr {
    let src = format!("＊会話\n　{attr}\n　Alice：やあ\n");
    let attrs = scene_attrs(&src);
    assert_eq!(attrs.len(), 1, "{attr}");
    attrs.into_iter().next().unwrap()
}

#[test]
fn condition_forms() {
    let a = init_attr("＆＄＊経験値≧500");
    assert_eq!(a.key, "＄＊経験値");
    assert_eq!(
        a.body,
        AttrBody::Cond {
            op: CmpOp::Ge,
            value: AttrValue::Integer(500)
        }
    );

    let a = init_attr("＆＄hour>5");
    assert_eq!(a.key, "＄hour");
    assert_eq!(
        a.body,
        AttrBody::Cond {
            op: CmpOp::Gt,
            value: AttrValue::Integer(5)
        }
    );

    let a = init_attr("＆＄ｒ０＝「changed」");
    assert_eq!(a.key, "＄ｒ０");
    assert_eq!(
        a.body,
        AttrBody::Cond {
            op: CmpOp::Eq,
            value: AttrValue::String("changed".into())
        }
    );

    // 半角の印も全角に揃える
    let a = init_attr("&$*exp>=5");
    assert_eq!(a.key, "＄＊exp");
}

#[test]
fn flag_form() {
    let a = init_attr("＆＄＊初回待ち");
    assert_eq!(a.key, "＄＊初回待ち");
    assert_eq!(a.body, AttrBody::Flag);
    assert!(a.value().is_none());
}

#[test]
fn cmp_ops_normalized() {
    let cases = [
        ("≧", ">="),
        (">=", ">="),
        ("≦", "<="),
        ("<=", "<="),
        ("＞", ">"),
        (">", ">"),
        ("＜", "<"),
        ("<", "<"),
        ("＝", "="),
        ("=", "="),
        ("≠", "!="),
        ("!=", "!="),
    ];
    for (written, canonical) in cases {
        let a = init_attr(&format!("＆＄x{written}1"));
        match a.body {
            AttrBody::Cond { op, value } => {
                assert_eq!(op.as_str(), canonical, "{written}");
                assert_eq!(value, AttrValue::Integer(1), "{written}");
            }
            other => panic!("{written}: {other:?}"),
        }
    }
}

#[test]
fn file_level_condition_and_flag() {
    let file = parse_str(
        "＆＄＊経験値＜1500＆＄＊初回待ち＆背景：教室\n＊会話\n　Alice：やあ\n",
        "test.pasta",
    )
    .unwrap();
    let attrs: Vec<&Attr> = file
        .items
        .iter()
        .filter_map(|i| match i {
            FileItem::FileAttr(a) => Some(a),
            _ => None,
        })
        .collect();
    assert_eq!(attrs.len(), 3);
    assert!(matches!(
        attrs[0].body,
        AttrBody::Cond { op: CmpOp::Lt, .. }
    ));
    assert_eq!(attrs[1].body, AttrBody::Flag);
    assert_eq!(attrs[2].key, "背景");
}

#[test]
fn condition_on_declaration_lines_is_accepted() {
    parse_str(
        "＊会話＆＄＊経験値≧500\n　Alice：やあ\n　・選択肢＆＄hour>5\n　Alice：うん\n",
        "test.pasta",
    )
    .unwrap();
}

#[test]
fn unnamed_scene_line_accepts_attrs() {
    parse_str(
        "＊会話\n　Alice：やあ\n＊＆順番：２０\n　Alice：うん\n＊　＆重み：５\n　Alice：ええ\n",
        "test.pasta",
    )
    .unwrap();
}

#[test]
fn variable_on_right_hand_side_is_error() {
    assert!(parse_str("＊会話\n　＆＄ｒ０＝＄ｒ１\n　Alice：やあ\n", "test.pasta").is_err());
}

#[test]
fn argument_number_condition_is_error() {
    assert!(parse_str("＊会話\n　＆＄０＞1\n　Alice：やあ\n", "test.pasta").is_err());
}

#[test]
fn unquoted_value_keeps_equals() {
    let a = init_attr("＆k：a＝b");
    assert_eq!(a.key, "k");
    assert_eq!(a.body, AttrBody::Decl(AttrValue::AttrString("a＝b".into())));
}

#[test]
fn comma_value_is_one_string() {
    let a = init_attr("＆背景：教室、夕方");
    assert_eq!(
        a.body,
        AttrBody::Decl(AttrValue::AttrString("教室、夕方".into()))
    );
}

// ============================================================================
// 宣言行の属性を構文木に残す（Requirements 1.1〜1.6, 4.1〜4.4）
// ============================================================================

fn global_scenes(src: &str) -> Vec<GlobalSceneScope> {
    parse_str(src, "test.pasta")
        .unwrap()
        .items
        .into_iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(scene) => Some(scene),
            _ => None,
        })
        .collect()
}

fn keys(attrs: &[Attr]) -> Vec<&str> {
    attrs.iter().map(|a| a.key.as_str()).collect()
}

#[test]
fn global_declaration_attrs_come_before_init_attrs() {
    let scenes = global_scenes(
        "＊会話＆作者：Alice＆季節：夏
　＆背景：教室
　Alice：やあ
",
    );
    assert_eq!(keys(&scenes[0].attrs), ["作者", "季節", "背景"]);
    assert_eq!(scenes[0].attrs[0].span.start_line, 1);
    assert_eq!(scenes[0].attrs[2].span.start_line, 2);
}

#[test]
fn unnamed_scene_keeps_own_attrs_only() {
    let scenes = global_scenes(
        "＊会話＆作者：Alice
　Alice：やあ
＊＆順番：２０
　＆重み：５
　Alice：うん
＊
　Alice：ええ
",
    );
    assert_eq!(scenes[1].name, "会話");
    assert!(scenes[1].is_continuation);
    assert_eq!(keys(&scenes[1].attrs), ["順番", "重み"]);
    assert!(scenes[2].attrs.is_empty());
}

#[test]
fn local_declaration_attrs_are_kept() {
    let scenes = global_scenes(
        "＊会話
　Alice：やあ
　・選択肢＆優先：3＆＄hour>5
　Alice：うん
",
    );
    let local = scenes[0]
        .local_scenes
        .iter()
        .find(|l| l.name.as_deref() == Some("選択肢"))
        .unwrap();
    assert_eq!(keys(&local.attrs), ["優先", "＄hour"]);
    assert_eq!(local.attrs[0].body, AttrBody::Decl(AttrValue::Integer(3)));
}

#[test]
fn declaration_line_value_types() {
    let scenes = global_scenes(
        "＊会話＆a：12＆b：１２＆c：1.5＆d：「x y」＆e：教室＆f：「」
　Alice：やあ
",
    );
    let values: Vec<_> = scenes[0].attrs.iter().map(|a| a.body.clone()).collect();
    assert_eq!(
        values,
        [
            AttrBody::Decl(AttrValue::Integer(12)),
            AttrBody::Decl(AttrValue::Integer(12)),
            AttrBody::Decl(AttrValue::Float(1.5)),
            AttrBody::Decl(AttrValue::String("x y".into())),
            AttrBody::Decl(AttrValue::AttrString("教室".into())),
            AttrBody::Decl(AttrValue::String(String::new())),
        ]
    );
}

#[test]
fn attr_line_after_local_declaration_is_error() {
    assert!(
        parse_str(
            "＊会話
　Alice：やあ
　・選択肢
　＆優先：3
　Alice：うん
",
            "test.pasta"
        )
        .is_err()
    );
}

#[test]
fn attr_line_after_content_line_is_error() {
    assert!(
        parse_str(
            "＊会話
　Alice：やあ
　＆背景：教室
",
            "test.pasta"
        )
        .is_err()
    );
}

#[test]
fn partial_parse_shifts_declaration_attr_spans() {
    let src = "＊壊れ
　＄＝
＊会話＆作者：Alice
　・選択肢＆優先：3
　Alice：やあ
";
    let result = pasta_dsl::parse_str_partial(src);
    let scene = result
        .items
        .iter()
        .find_map(|item| match item {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .unwrap();
    let attr = &scene.attrs[0];
    assert_eq!(attr.span.start_line, 3);
    assert_eq!(
        &src[attr.span.start_byte..attr.span.end_byte],
        "＆作者：Alice"
    );
    let local = &scene
        .local_scenes
        .iter()
        .find(|l| l.name.is_some())
        .unwrap()
        .attrs[0];
    assert_eq!(local.span.start_line, 4);
    assert_eq!(
        &src[local.span.start_byte..local.span.end_byte],
        "＆優先：3"
    );
}

// ============================================================================
// ファイル属性の 2 行目（Requirement 3.3）
// ============================================================================

fn second_file_attr_error(src: &str) -> (usize, usize, String) {
    match parse_str(src, "test.pasta") {
        Err(pasta_dsl::ParseError::SyntaxError {
            line,
            column,
            message,
            ..
        }) => (line, column, message),
        other => panic!("expected SyntaxError, got {other:?}"),
    }
}

#[test]
fn consecutive_file_attr_lines_are_error() {
    let (line, column, message) =
        second_file_attr_error("＆背景：教室\n＆時間：朝\n＊会話\n　Alice：やあ\n");
    assert_eq!((line, column), (2, 1));
    assert!(
        message.contains(
            "ファイル属性の行が続いています。ファイル属性は 1 行にまとめて書きます（前の行: 1 行目）"
        ),
        "{message}"
    );
}

#[test]
fn file_attr_lines_around_actor_and_word_are_error() {
    let (line, _, message) = second_file_attr_error(
        "＆a：1\n％さくら\n　＠通常：にこ\n＠挨拶：やあ\n\n＆b：2\n＊会話\n　さくら：やあ\n",
    );
    assert_eq!(line, 6);
    assert!(message.contains("（前の行: 1 行目）"), "{message}");
}

#[test]
fn file_attr_lines_separated_by_global_scene_are_ok() {
    let file = parse_str(
        "＆a：1\n＊会話\n　Alice：やあ\n＆b：2\n＊別\n　Alice：うん\n",
        "test.pasta",
    )
    .unwrap();
    let attrs = file
        .items
        .iter()
        .filter(|i| matches!(i, FileItem::FileAttr(_)))
        .count();
    assert_eq!(attrs, 2);
}

#[test]
fn several_attrs_on_one_file_attr_line_are_ok() {
    assert!(parse_str("＆a：1＆b：2\n＊会話\n　Alice：やあ\n", "test.pasta").is_ok());
}
