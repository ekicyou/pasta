//! Transpile context for Lua code generation.
//!
//! This module provides context management for the transpilation process.

use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue, SceneRegistry, WordDefRegistry};
use pasta_dsl::parser::{Attr, AttrBody, AttrValue, LocalSceneScope, merge_attrs};
use std::collections::HashMap;

/// Transpile context for sharing state during transpilation.
#[derive(Default)]
pub struct TranspileContext {
    /// Scene registry for global/local scene registration
    pub scene_registry: SceneRegistry,
    /// Word definition registry for global/local word registration
    pub word_registry: WordDefRegistry,
}

impl TranspileContext {
    /// Create a new transpile context.
    pub fn new() -> Self {
        Self::default()
    }

    /// 宣言名（別名を置き換えた後の名前）でグローバルシーンを登録する。
    ///
    /// 通し番号は照合用の名前ごとに採番される。Returns (id, counter).
    pub fn register_global_scene_named(&mut self, name: &str, attrs: &[Attr]) -> (i64, usize) {
        self.scene_registry
            .register_global(name, scene_attrs_of(attrs))
    }

    /// Register a local scene (Task 3.1).
    ///
    /// Registers the local scene under the parent global scene.
    /// `local_counter` は照合用の名前ごとの通し番号（[`local_scene_counters`] の値。
    /// 生成器の関数名と同じ番号）。
    /// Returns the assigned scene ID.
    pub fn register_local_scene(
        &mut self,
        local_scene: &LocalSceneScope,
        parent_name: &str,
        parent_counter: usize,
        local_counter: usize,
    ) -> i64 {
        // ローカルはファイル属性も親の属性も受け継がない（宣言行の中だけ畳む）
        let attrs = scene_attrs_of(&merge_attrs(&[], &local_scene.attrs));

        // Use scene name if present, otherwise use "__start__"
        let name = local_scene.name.as_deref().unwrap_or("__start__");

        self.scene_registry
            .register_local(name, parent_name, parent_counter, local_counter, attrs)
    }
}

/// グローバルシーンの中のローカルシーンの通し番号を、定義順に返す。
///
/// 名前つきのローカルシーンは照合用の名前（`SceneRegistry::sanitize_name`）ごとに 1 から数える
/// （`・挨拶・1` と `・挨拶_1` はどちらも `挨拶_1` なので 1・2）。無名の開始シーンは 0。
/// 生成器の関数名とトランスパイル時のレジストリの登録の両方がこの番号を使う（1.7・2.12）。
pub fn local_scene_counters(local_scenes: &[LocalSceneScope]) -> Vec<usize> {
    let mut counters: HashMap<String, usize> = HashMap::new();
    local_scenes
        .iter()
        .map(|local| match &local.name {
            Some(name) => {
                let count = counters
                    .entry(SceneRegistry::sanitize_name(name))
                    .or_insert(0);
                *count += 1;
                *count
            }
            None => 0,
        })
        .collect()
}

/// 構文木の属性 → 倉の項目（pasta_check の validate/scenes.rs に同じ写しがある）
pub fn scene_attrs_of(attrs: &[Attr]) -> Vec<SceneAttr> {
    attrs
        .iter()
        .map(|a| {
            let key = a.key.clone();
            match &a.body {
                AttrBody::Decl(v) => SceneAttr::Decl {
                    key,
                    value: scene_attr_value(v),
                },
                AttrBody::Cond { op, value } => SceneAttr::Cond {
                    key,
                    // 正規形の文字列 6 つが pasta_dsl と pasta_core の CmpOp の約束
                    op: CmpOp::from_normalized(op.as_str())
                        .expect("pasta_dsl と pasta_core の CmpOp の正規形は一致する"),
                    value: scene_attr_value(value),
                },
                AttrBody::Flag => SceneAttr::Flag { key },
            }
        })
        .collect()
}

/// 値の型の規則（要件 4.5）: 小数は整数の値なら整数、引用の有無は区別しない
pub(crate) fn scene_attr_value(v: &AttrValue) -> SceneAttrValue {
    match v {
        AttrValue::Integer(i) => SceneAttrValue::Integer(*i),
        AttrValue::Float(f) => SceneAttrValue::from_f64(*f),
        AttrValue::String(s) | AttrValue::AttrString(s) => SceneAttrValue::String(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_dsl::parser::{GlobalSceneScope, KeyWords, Span};

    fn create_test_scene(name: &str) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![],
            actors: vec![],
            code_blocks: vec![],
            local_scenes: vec![],
            span: Span::default(),
        }
    }

    fn create_test_local_scene(name: &str) -> LocalSceneScope {
        LocalSceneScope::named(name.to_string())
    }

    #[test]
    fn test_register_global_scene() {
        let mut ctx = TranspileContext::new();
        let scene = create_test_scene("メイン");

        let (id, counter) = ctx.register_global_scene_named(&scene.name, &scene.attrs);
        assert_eq!(id, 1);
        assert_eq!(counter, 1);

        let scenes = ctx.scene_registry.all_scenes();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].name, "メイン");
    }

    #[test]
    fn test_register_local_scene() {
        let mut ctx = TranspileContext::new();

        // First register parent
        let parent = create_test_scene("メイン");
        let (_, parent_counter) = ctx.register_global_scene_named(&parent.name, &parent.attrs);

        // Then register local scene
        let local = create_test_local_scene("自己紹介");
        let id = ctx.register_local_scene(&local, "メイン", parent_counter, 1);

        assert_eq!(id, 2);
        let scenes = ctx.scene_registry.all_scenes();
        assert_eq!(scenes.len(), 2);
        // 第 4 引数は照合用の名前ごとの通し番号（生成器の関数名 `自己紹介_1` と同じ番号）
        assert_eq!(scenes[1].fn_name, "メイン_1::自己紹介_1");
    }

    #[test]
    fn test_local_scene_counters_count_per_sanitized_name() {
        // 開始シーンは 0、名前つきは照合用の名前ごとに 1 から数える（1.7）
        let locals: Vec<LocalSceneScope> =
            [None, Some("A"), Some("挨拶・1"), Some("A"), Some("挨拶_1")]
                .into_iter()
                .map(|name| LocalSceneScope {
                    name: name.map(str::to_string),
                    ..create_test_local_scene("x")
                })
                .collect();
        assert_eq!(local_scene_counters(&locals), vec![0, 1, 1, 2, 2]);
    }

    #[test]
    fn test_register_global_words() {
        let mut ctx = TranspileContext::new();
        // 単一キー: word_registry 直接呼び出し
        ctx.word_registry
            .register_global("挨拶", vec!["こんにちは".to_string(), "やあ".to_string()]);

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "挨拶");
    }

    #[test]
    fn test_register_global_words_multi_key() {
        let mut ctx = TranspileContext::new();
        // 複数キー: 各キーに対して登録
        let kw = KeyWords {
            names: vec!["女性".to_string(), "水の妖精".to_string()],
            words: vec!["水無灯里".to_string(), "アリス・キャロル".to_string()],
            span: Span::default(),
        };
        for name in &kw.names {
            ctx.word_registry.register_global(name, kw.words.clone());
        }

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "女性");
        assert_eq!(entries[1].key, "水の妖精");
    }

    #[test]
    fn test_register_local_words() {
        let mut ctx = TranspileContext::new();
        // 単一キー: word_registry 直接呼び出し
        ctx.word_registry.register_local(
            "メイン_1",
            "場所",
            vec!["東京".to_string(), "大阪".to_string()],
        );

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].key.contains(":メイン_1:場所"));
    }

    #[test]
    fn test_register_local_words_multi_key() {
        let mut ctx = TranspileContext::new();
        // 複数キー: 各キーに対して登録
        let kw = KeyWords {
            names: vec!["場所".to_string(), "地名".to_string()],
            words: vec!["東京".to_string(), "大阪".to_string()],
            span: Span::default(),
        };
        for name in &kw.names {
            ctx.word_registry
                .register_local("メイン_1", name, kw.words.clone());
        }

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].key.contains(":メイン_1:場所"));
        assert!(entries[1].key.contains(":メイン_1:地名"));
    }

    #[test]
    fn test_scene_attrs_of_maps_all_three_kinds() {
        use pasta_dsl::parser::CmpOp as DslCmpOp;
        let attr = |key: &str, body| Attr {
            key: key.to_string(),
            body,
            span: Span::default(),
        };
        let attrs = [
            attr("季節", AttrBody::Decl(AttrValue::String("夏".into()))),
            attr("率", AttrBody::Decl(AttrValue::Float(1.0))),
            attr("重み", AttrBody::Decl(AttrValue::Float(0.5))),
            attr(
                "＄＊経験値",
                AttrBody::Cond {
                    op: DslCmpOp::Ge,
                    value: AttrValue::Integer(500),
                },
            ),
            attr("＄初回", AttrBody::Flag),
        ];
        assert_eq!(
            scene_attrs_of(&attrs),
            vec![
                SceneAttr::Decl {
                    key: "季節".into(),
                    value: SceneAttrValue::String("夏".into()),
                },
                SceneAttr::Decl {
                    key: "率".into(),
                    value: SceneAttrValue::Integer(1),
                },
                SceneAttr::Decl {
                    key: "重み".into(),
                    value: SceneAttrValue::Float(0.5),
                },
                SceneAttr::Cond {
                    key: "＄＊経験値".into(),
                    op: CmpOp::Ge,
                    value: SceneAttrValue::Integer(500),
                },
                SceneAttr::Flag {
                    key: "＄初回".into()
                },
            ]
        );
    }
}
