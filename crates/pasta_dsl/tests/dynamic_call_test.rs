//! 動的コール（＞expr）のパーサーテスト
//!
//! Requirements: 1.1, 1.2, 1.3, 4.1

use pasta_dsl::parser::*;

fn get_call_scenes(source: &str) -> Vec<CallScene> {
    let file = parse_str(source, "test.pasta").unwrap();
    file.items
        .into_iter()
        .filter_map(|item| {
            if let FileItem::GlobalSceneScope(gs) = item {
                Some(gs)
            } else {
                None
            }
        })
        .flat_map(|gs| gs.local_scenes)
        .flat_map(|ls| ls.items)
        .filter_map(|item| {
            if let LocalSceneItem::CallScene(cs) = item {
                Some(cs)
            } else {
                None
            }
        })
        .collect()
}

// ========================================================================
// R1.1: ＞expr パース→ Dynamic(Expr)
// ========================================================================

#[test]
fn test_dynamic_call_local_var_fullwidth() {
    let source = "＊テスト\n　＞＄target\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(
        matches!(&calls[0].target, CallTarget::Dynamic(Expr::VarRef { name, scope: VarScope::Local }) if name == "target")
    );
}

#[test]
fn test_dynamic_call_local_var_halfwidth() {
    let source = "＊テスト\n　>$target\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(
        matches!(&calls[0].target, CallTarget::Dynamic(Expr::VarRef { name, scope: VarScope::Local }) if name == "target")
    );
}

// ========================================================================
// R1.2: 全角/半角自動対応
// ========================================================================

#[test]
fn test_dynamic_call_global_var_fullwidth() {
    let source = "＊テスト\n　＞＄＊global\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(
        matches!(&calls[0].target, CallTarget::Dynamic(Expr::VarRef { name, scope: VarScope::Global }) if name == "global")
    );
}

#[test]
fn test_dynamic_call_global_var_halfwidth() {
    let source = "＊テスト\n　>$*global\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(
        matches!(&calls[0].target, CallTarget::Dynamic(Expr::VarRef { name, scope: VarScope::Global }) if name == "global")
    );
}

// ========================================================================
// R1.3: Static/Dynamic 型区別 + R4.1: 静的コール不変
// ========================================================================

#[test]
fn test_static_call_unchanged() {
    let source = "＊テスト\n　＞シーン名\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(matches!(&calls[0].target, CallTarget::Static(name) if name == "シーン名"));
}

#[test]
fn test_static_call_halfwidth_unchanged() {
    let source = "＊テスト\n　>target\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 1);
    assert!(matches!(&calls[0].target, CallTarget::Static(name) if name == "target"));
}

#[test]
fn test_mixed_static_and_dynamic_calls() {
    let source = "＊テスト\n　＞静的シーン\n　＞＄動的target\n";
    let calls = get_call_scenes(source);
    assert_eq!(calls.len(), 2);
    assert!(matches!(&calls[0].target, CallTarget::Static(_)));
    assert!(matches!(&calls[1].target, CallTarget::Dynamic(_)));
}
