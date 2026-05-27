//! プロパティスコープ（`＄％prop`）のパーサーテスト
//!
//! Requirements 1.1〜1.6（プロパティスコープ修飾子と名前の文法）
//! Requirements 6.1〜6.2（構文エラー報告）

use pasta_dsl::parser::*;

fn get_global_scene_scopes(file: &PastaFile) -> Vec<&GlobalSceneScope> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::GlobalSceneScope(scene) = item {
                Some(scene)
            } else {
                None
            }
        })
        .collect()
}

fn find_var_sets_in_scene(scene: &GlobalSceneScope) -> Vec<&VarSet> {
    let mut sets = Vec::new();
    for local_scene in &scene.local_scenes {
        for item in &local_scene.items {
            if let LocalSceneItem::VarSet(vs) = item {
                sets.push(vs);
            }
        }
    }
    sets
}

fn find_actions_in_scene(scene: &GlobalSceneScope) -> Vec<&Action> {
    let mut actions = Vec::new();
    for local_scene in &scene.local_scenes {
        for item in &local_scene.items {
            if let LocalSceneItem::ActionLine(al) = item {
                for a in &al.actions {
                    actions.push(a);
                }
            }
        }
    }
    actions
}

// ============================================================================
// VarSet: ＄％prop＝value
// ============================================================================

#[test]
fn test_property_scope_simple_var_set() {
    let input = "＊テスト\n　＄％simple＝123\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Property);
    assert_eq!(vs.name.as_deref(), Some("simple"));
    match &vs.value {
        SetValue::Expr(Expr::Integer(123)) => {}
        other => panic!("Expected Expr::Integer(123), got {:?}", other),
    }
}

#[test]
fn test_property_scope_dotted_name() {
    let input = "＊テスト\n　＄％system.name＝「テスト」\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Property);
    assert_eq!(vs.name.as_deref(), Some("system.name"));
}

#[test]
fn test_property_scope_complex_name_with_parens() {
    let input = "＊テスト\n　＄％scope(0).validwidth.initial＝400\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Property);
    assert_eq!(vs.name.as_deref(), Some("scope(0).validwidth.initial"));
    match &vs.value {
        SetValue::Expr(Expr::Integer(400)) => {}
        other => panic!("Expected Expr::Integer(400), got {:?}", other),
    }
}

#[test]
fn test_property_scope_var_set_with_varref_value() {
    let input = "＊テスト\n　＄％prop＝＄var\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Property);
    assert_eq!(vs.name.as_deref(), Some("prop"));
    match &vs.value {
        SetValue::Expr(Expr::VarRef { scope, .. }) => {
            assert_eq!(*scope, VarScope::Local);
        }
        other => panic!("Expected Expr::VarRef Local, got {:?}", other),
    }
}

// ============================================================================
// VarRef in actions: さくら：＄％prop
// ============================================================================

#[test]
fn test_property_scope_varref_in_action() {
    let input = "＊テスト\n　さくら：＄％simple\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let actions = find_actions_in_scene(scenes[0]);
    let var_refs: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::VarRef { .. }))
        .collect();
    assert_eq!(var_refs.len(), 1);
    match var_refs[0] {
        Action::VarRef { name, scope, .. } => {
            assert_eq!(name, "simple");
            assert_eq!(*scope, VarScope::Property);
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_property_scope_varref_followed_by_talk() {
    let input = "＊テスト\n　さくら：＄％system.name　テスト\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let actions = find_actions_in_scene(scenes[0]);

    // Should have VarRef then Talk
    let var_ref = actions
        .iter()
        .find(|a| matches!(a, Action::VarRef { .. }));
    assert!(var_ref.is_some(), "Expected VarRef action");
    match var_ref.unwrap() {
        Action::VarRef { name, scope, .. } => {
            assert_eq!(name, "system.name");
            assert_eq!(*scope, VarScope::Property);
        }
        _ => unreachable!(),
    }

    let talk = actions.iter().find(|a| matches!(a, Action::Talk { .. }));
    assert!(talk.is_some(), "Expected Talk action after VarRef");
}

// ============================================================================
// GET assignment: ＄var＝＄％prop
// ============================================================================

#[test]
fn test_property_scope_get_assignment_local() {
    let input = "＊テスト\n　＄var＝＄％currentghost.name\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Local);
    assert_eq!(vs.name.as_deref(), Some("var"));
    match &vs.value {
        SetValue::Expr(Expr::VarRef { name, scope }) => {
            assert_eq!(name, "currentghost.name");
            assert_eq!(*scope, VarScope::Property);
        }
        other => panic!("Expected Expr::VarRef Property, got {:?}", other),
    }
}

#[test]
fn test_property_scope_get_assignment_global() {
    let input = "＊テスト\n　＄＊var＝＄％prop\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.scope, VarScope::Global);
    assert_eq!(vs.name.as_deref(), Some("var"));
    match &vs.value {
        SetValue::Expr(Expr::VarRef { scope, .. }) => {
            assert_eq!(*scope, VarScope::Property);
        }
        other => panic!("Expected Expr::VarRef Property, got {:?}", other),
    }
}

// ============================================================================
// Half-width equivalence: $%half == ＄％half
// ============================================================================

#[test]
fn test_property_scope_halfwidth_equivalence() {
    let fullwidth = "＊テスト\n　さくら：＄％half\n";
    let halfwidth = "＊テスト\n　さくら：$%half\n";

    let full_file = parse_str(fullwidth, "test.pasta").unwrap();
    let half_file = parse_str(halfwidth, "test.pasta").unwrap();

    let full_actions = find_actions_in_scene(get_global_scene_scopes(&full_file)[0]);
    let half_actions = find_actions_in_scene(get_global_scene_scopes(&half_file)[0]);

    let full_var = full_actions
        .iter()
        .find(|a| matches!(a, Action::VarRef { .. }))
        .expect("fullwidth VarRef");
    let half_var = half_actions
        .iter()
        .find(|a| matches!(a, Action::VarRef { .. }))
        .expect("halfwidth VarRef");

    match (full_var, half_var) {
        (
            Action::VarRef {
                name: fn_name,
                scope: fs,
                ..
            },
            Action::VarRef {
                name: hn_name,
                scope: hs,
                ..
            },
        ) => {
            assert_eq!(fn_name, hn_name, "names should match");
            assert_eq!(fs, hs, "scopes should match");
        }
        _ => panic!("Expected VarRef for both"),
    }
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_property_scope_error_digit_start() {
    let input = "＊テスト\n　＄％1abc＝123\n";
    let result = parse_str(input, "test.pasta");
    assert!(
        result.is_err(),
        "property_id starting with digit should fail"
    );
}

#[test]
fn test_property_scope_error_empty_name() {
    let input = "＊テスト\n　＄％ ＝123\n";
    let result = parse_str(input, "test.pasta");
    assert!(
        result.is_err(),
        "property_id with space after ＄％ should fail"
    );
}

// ============================================================================
// Compatibility regression: 既存スコープ不変
// ============================================================================

#[test]
fn test_property_scope_regression_local() {
    let input = "＊テスト\n　＄var＝123\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    assert_eq!(var_sets[0].scope, VarScope::Local);
    assert_eq!(var_sets[0].name.as_deref(), Some("var"));
}

#[test]
fn test_property_scope_regression_global() {
    let input = "＊テスト\n　＄＊var＝123\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    assert_eq!(var_sets[0].scope, VarScope::Global);
    assert_eq!(var_sets[0].name.as_deref(), Some("var"));
}
