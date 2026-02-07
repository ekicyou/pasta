//! 全角数字変数（digit_id）のパーサーテスト
//!
//! `＄０`〜`＄９` がパーサーで正しく `VarScope::Args(N)` に解決されることを検証。
//! req-var-expansion ギャップ分析の実証テスト。

use pasta_core::parser::{Action, FileItem, GlobalSceneScope, LocalSceneItem, VarScope, parse_str};

/// Helper to get global scene scopes from PastaFile items
fn get_global_scene_scopes(file: &pasta_core::parser::PastaFile) -> Vec<&GlobalSceneScope> {
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

/// Helper to extract VarRef actions from a scene's local scenes
fn find_var_refs_in_scene(scene: &GlobalSceneScope) -> Vec<(String, VarScope)> {
    let mut refs = Vec::new();
    for local_scene in &scene.local_scenes {
        for item in &local_scene.items {
            let actions = match item {
                LocalSceneItem::ActionLine(al) => &al.actions,
                LocalSceneItem::ContinueAction(ca) => &ca.actions,
                _ => continue,
            };
            for action in actions {
                if let Action::VarRef { name, scope, .. } = action {
                    refs.push((name.clone(), *scope));
                }
            }
        }
    }
    refs
}

// ============================================================================
// 全角数字 → VarScope::Args 変換テスト
// ============================================================================

#[test]
fn test_fullwidth_digit_0_parsed_as_args_0() {
    // ＄０ がパースされて VarScope::Args(0) になることを確認
    let input = "＊テスト\n　さくら：＄０\n";
    let result = parse_str(input, "test.pasta");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert!(!scenes.is_empty(), "No scenes found");

    let var_refs = find_var_refs_in_scene(scenes[0]);
    assert!(!var_refs.is_empty(), "No VarRef found");

    let (name, scope) = &var_refs[0];
    assert_eq!(
        scope,
        &VarScope::Args(0),
        "Expected Args(0), got {:?}",
        scope
    );
    assert_eq!(name, "０", "Expected full-width '０' but got '{}'", name);
}

#[test]
fn test_fullwidth_digit_1_to_9_parsed_as_args() {
    // ＄１〜＄９ がそれぞれ VarScope::Args(1)〜Args(9) になることを確認
    for i in 1u8..=9 {
        let zenkaku = char::from_u32('０' as u32 + i as u32).unwrap();
        let input = format!("＊テスト{}\n　さくら：＄{}\n", i, zenkaku);
        let result = parse_str(&input, "test.pasta");
        assert!(
            result.is_ok(),
            "Parse failed for ＄{}: {:?}",
            zenkaku,
            result.err()
        );

        let file = result.unwrap();
        let scenes = get_global_scene_scopes(&file);
        assert!(!scenes.is_empty(), "No scenes for ＄{}", zenkaku);

        let var_refs = find_var_refs_in_scene(scenes[0]);
        let found = var_refs
            .iter()
            .any(|(name, scope)| *scope == VarScope::Args(i) && *name == zenkaku.to_string());
        assert!(
            found,
            "VarScope::Args({}) not found for ＄{}. Found: {:?}",
            i, zenkaku, var_refs
        );
    }
}

#[test]
fn test_halfwidth_digit_0_also_parsed_as_args() {
    // 半角 $0 も VarScope::Args(0) に解決されることを確認（既存動作の回帰テスト）
    let input = "＊テスト\n　さくら：$0\n";
    let result = parse_str(input, "test.pasta");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);

    let var_refs = find_var_refs_in_scene(scenes[0]);
    let found = var_refs
        .iter()
        .any(|(_, scope)| *scope == VarScope::Args(0));
    assert!(found, "VarScope::Args(0) not found for $0");
}

#[test]
fn test_multidigit_fullwidth_parsed_as_args() {
    // ＄１０ のような多桁全角数字もパースされることを確認
    let input = "＊テスト\n　さくら：＄１０\n";
    let result = parse_str(input, "test.pasta");
    assert!(
        result.is_ok(),
        "Parse failed for ＄１０: {:?}",
        result.err()
    );

    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);

    let var_refs = find_var_refs_in_scene(scenes[0]);
    let found = var_refs
        .iter()
        .any(|(name, scope)| *scope == VarScope::Args(10) && name == "１０");
    assert!(
        found,
        "VarScope::Args(10) not found for ＄１０. Found: {:?}",
        var_refs
    );
}
