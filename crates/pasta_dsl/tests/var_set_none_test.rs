//! `var_set_none`（ExprStmt: `＄＝expr`）のパーサーテスト
//!
//! fn-call-expr-stmt 仕様 Requirement 2 の受入基準を検証。

use pasta_dsl::parser::*;

/// Helper to get global scene scopes from PastaFile items
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

/// Helper to extract VarSet items from a scene's first local scene
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

// ============================================================================
// Requirement 2.1: `＄＝＠fn()` がパースされる
// ============================================================================

#[test]
fn test_var_set_none_basic_fn_call() {
    let input = "＊テスト\n　＄＝＠func（）\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert!(vs.name.is_none(), "var_set_none should have name=None");
    match &vs.value {
        SetValue::Expr(Expr::FnCall { name, scope, .. }) => {
            assert_eq!(name, "func");
            assert_eq!(*scope, FnScope::Local);
        }
        other => panic!("Expected Expr::FnCall, got {:?}", other),
    }
}

// ============================================================================
// Requirement 2.2: 引数付き `＄＝＠fn（x：10）`
// ============================================================================

#[test]
fn test_var_set_none_fn_call_with_args() {
    let input = "＊テスト\n　＄＝＠func（x：10）\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert!(vs.name.is_none());
    match &vs.value {
        SetValue::Expr(Expr::FnCall { name, args, .. }) => {
            assert_eq!(name, "func");
            assert_eq!(args.items.len(), 1);
        }
        other => panic!("Expected Expr::FnCall with args, got {:?}", other),
    }
}

// ============================================================================
// Requirement 2.3: 半角混在 `$=@fn()`
// ============================================================================

#[test]
fn test_var_set_none_halfwidth() {
    let input = "＊テスト\n　$=@func()\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert!(vs.name.is_none());
    match &vs.value {
        SetValue::Expr(Expr::FnCall { name, .. }) => assert_eq!(name, "func"),
        other => panic!("Expected halfwidth fn_call, got {:?}", other),
    }
}

// ============================================================================
// Requirement 2.5: `＄＝＠＊fn()` （グローバル関数呼び出し式文）
// ============================================================================

#[test]
fn test_var_set_none_global_fn_call() {
    let input = "＊テスト\n　＄＝＠＊gfunc（）\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert!(vs.name.is_none());
    match &vs.value {
        SetValue::Expr(Expr::FnCall { name, scope, .. }) => {
            assert_eq!(name, "gfunc");
            assert_eq!(*scope, FnScope::Global);
        }
        other => panic!("Expected global FnCall, got {:?}", other),
    }
}

// ============================================================================
// Requirement 4.1, 4.2: 既存構文のリグレッション確認
// ============================================================================

#[test]
fn test_existing_var_set_local_still_works() {
    let input = "＊テスト\n　＄カウンタ＝10\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.name.as_deref(), Some("カウンタ"));
    assert_eq!(vs.scope, VarScope::Local);
}

#[test]
fn test_existing_var_set_global_still_works() {
    let input = "＊テスト\n　＄＊フラグ＝1\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.name.as_deref(), Some("フラグ"));
    assert_eq!(vs.scope, VarScope::Global);
}

#[test]
fn test_existing_var_set_fn_call_rhs_still_works() {
    let input = "＊テスト\n　＄結果＝＠func（）\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert_eq!(vs.name.as_deref(), Some("結果"));
    match &vs.value {
        SetValue::Expr(Expr::FnCall { name, .. }) => assert_eq!(name, "func"),
        other => panic!("Expected FnCall on RHS, got {:?}", other),
    }
}

// ============================================================================
// Requirement 2.6: var_set_none は var_set_line の一部として認識
// ============================================================================

#[test]
fn test_var_set_none_with_integer_expr() {
    // 理論上は書かれないが、文法的には許容される
    let input = "＊テスト\n　＄＝42\n";
    let file = parse_str(input, "test.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);
    let var_sets = find_var_sets_in_scene(scenes[0]);
    assert_eq!(var_sets.len(), 1);
    let vs = var_sets[0];
    assert!(vs.name.is_none());
    match &vs.value {
        SetValue::Expr(Expr::Integer(42)) => {}
        other => panic!("Expected Integer(42), got {:?}", other),
    }
}
