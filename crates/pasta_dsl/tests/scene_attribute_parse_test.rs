//! 属性の 3 種の形（宣言・条件・フラグ）と、名前なしの `＊` 行への付記のパーステスト
//!
//! Requirements 1.5, 2.1, 2.2, 2.3, 2.4, 2.6, 2.7, 4.6

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
