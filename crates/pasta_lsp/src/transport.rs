//! WASM の入口。VSCode 拡張から呼ばれる 3 つの関数だけを持つ。
//!
//! 結果は `serde-wasm-bindgen` で JS の値にして返す。

/// WASM entry point (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use crate::analysis::AnalysisEngine;
    use pasta_check::validate::{FileText, GhostInput};
    use serde::Serialize;
    use std::collections::BTreeMap;
    use wasm_bindgen::prelude::*;

    /// Analyze a pasta source string and return semantic tokens + diagnostics.
    ///
    /// `wasm-bindgen` converts JavaScript strings into Rust `&str`.
    /// JavaScript strings are UTF-16; unpaired surrogates are replaced with
    /// U+FFFD during conversion, so the Rust side always receives valid UTF-8.
    ///
    /// 解析の結果をそのまま JS の値にする。直列化に失敗したら `null`。
    #[wasm_bindgen]
    pub fn wasm_analyze(source: &str) -> JsValue {
        serde_wasm_bindgen::to_value(&AnalysisEngine::analyze(source)).unwrap_or(JsValue::NULL)
    }

    /// `GhostInput`（camelCase）から、辞書ファイルとして読むべきパスの配列を返す。
    /// 入力の形が違うときは `null`。
    #[wasm_bindgen]
    pub fn wasm_select_ghost_files(input: JsValue) -> JsValue {
        let Ok(input) = serde_wasm_bindgen::from_value::<GhostInput>(input) else {
            return JsValue::NULL;
        };
        to_js(&pasta_check::validate::select_dictionary_files(&input))
    }

    /// `GhostInput` と `Record<string, FileText>` からゴースト全体を検査し、
    /// 核の `Report` をそのまま返す。入力の形が違うときは `null`。
    #[wasm_bindgen]
    pub fn wasm_validate_ghost(input: JsValue, files: JsValue) -> JsValue {
        let (Ok(input), Ok(files)) = (
            serde_wasm_bindgen::from_value::<GhostInput>(input),
            serde_wasm_bindgen::from_value::<BTreeMap<String, FileText>>(files),
        ) else {
            return JsValue::NULL;
        };
        to_js(&pasta_check::validate::validate_ghost(&input, &files))
    }

    /// `None` を `undefined` でなく `null` にする（`line: number | null`）。失敗したら `null`
    fn to_js<T: Serialize>(value: &T) -> JsValue {
        value
            .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
            .unwrap_or(JsValue::NULL)
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{
        AnalysisResult, Diagnostic, Position, Range, SceneKind, SceneNode, SemanticToken,
    };

    fn range(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range {
            start: Position {
                line: sl,
                character: sc,
            },
            end: Position {
                line: el,
                character: ec,
            },
        }
    }

    /// `wasm_analyze` が JS へ渡す形（欄の名前・順・型）が今の拡張の型と同じこと
    #[test]
    fn test_analysis_result_json_shape_stable() {
        let analysis_result = AnalysisResult {
            tokens: vec![SemanticToken {
                delta_line: 0,
                delta_start: 1,
                length: 2,
                token_type: 3,
                token_modifiers_bitset: 4,
            }],
            diagnostics: vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                message: "test warning".to_string(),
                severity: 1,
            }],
            scenes: vec![SceneNode {
                name: "挨拶".to_string(),
                kind: SceneKind::Global,
                heading: range(0, 0, 0, 3),
                range: range(0, 0, 3, 12),
                children: vec![SceneNode {
                    name: "朝".to_string(),
                    kind: SceneKind::Local,
                    heading: range(2, 2, 2, 4),
                    range: range(2, 0, 3, 12),
                    children: vec![],
                }],
            }],
        };

        let json = serde_json::to_string(&analysis_result).expect("should serialize to JSON");

        assert_eq!(
            json,
            r#"{"tokens":[{"deltaLine":0,"deltaStartCharacter":1,"length":2,"tokenType":3,"tokenModifiers":4}],"diagnostics":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"message":"test warning","severity":1}],"scenes":[{"name":"挨拶","kind":"global","heading":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},"range":{"start":{"line":0,"character":0},"end":{"line":3,"character":12}},"children":[{"name":"朝","kind":"local","heading":{"start":{"line":2,"character":2},"end":{"line":2,"character":4}},"range":{"start":{"line":2,"character":0},"end":{"line":3,"character":12}},"children":[]}]}]}"#
        );
    }

    /// ゴースト全体の検査の入出力が、JS の形（camelCase・タグ `kind`）で読み書きできること
    #[test]
    fn test_ghost_validate_js_shape() {
        use pasta_check::validate::{
            FileText, GhostInput, select_dictionary_files, validate_ghost,
        };
        use std::collections::BTreeMap;

        let input: GhostInput = serde_json::from_str(
            r#"{"pastaToml":{"kind":"missing"},"entries":["dic/a.pasta","dic/b.pasta","dic/c.pasta"],"dirs":["dic"]}"#,
        )
        .expect("GhostInput should deserialize from the JS shape");
        // dirs は必須（既定値なし）。付け忘れは形の誤り
        assert!(
            serde_json::from_str::<GhostInput>(
                r#"{"pastaToml":{"kind":"missing"},"entries":["dic/a.pasta"]}"#
            )
            .is_err(),
            "GhostInput without dirs must not deserialize"
        );
        let files: BTreeMap<String, FileText> = serde_json::from_str(
            r#"{"dic/a.pasta":{"kind":"text","text":"＊会話\n"},
                "dic/b.pasta":{"kind":"notUtf8"},
                "dic/c.pasta":{"kind":"unreadable","reason":"denied"}}"#,
        )
        .expect("FileText should deserialize from the JS shape");
        for kind in [
            r#"{"kind":"text","text":"x"}"#,
            r#"{"kind":"unreadable","reason":"r"}"#,
        ] {
            serde_json::from_str::<GhostInput>(&format!(
                r#"{{"pastaToml":{kind},"entries":[],"dirs":[]}}"#
            ))
            .expect("TomlSource should deserialize from the JS shape");
        }

        let selected = select_dictionary_files(&input);
        assert_eq!(
            serde_json::to_value(&selected).unwrap(),
            serde_json::json!(["dic/a.pasta", "dic/b.pasta", "dic/c.pasta"])
        );

        let report = serde_json::to_value(validate_ghost(&input, &files)).unwrap();
        let stats = report["stats"].as_object().expect("stats object");
        let mut stat_keys: Vec<&str> = stats.keys().map(String::as_str).collect();
        stat_keys.sort();
        assert_eq!(
            stat_keys,
            [
                "dictionaryFiles",
                "errors",
                "skipReasons",
                "uncheckedDynamicCalls",
                "warnings"
            ]
        );
        // 省く理由は kebab-case の文字列の並び（読めない辞書ファイルがあるので省く）
        assert_eq!(stats["skipReasons"], serde_json::json!(["unreadable-file"]));
        let findings = report["findings"].as_array().expect("findings array");
        for f in findings {
            let mut keys: Vec<&str> = f.as_object().unwrap().keys().map(String::as_str).collect();
            keys.sort();
            assert_eq!(
                keys,
                ["code", "column", "line", "message", "path", "severity"]
            );
        }
        let not_utf8 = findings
            .iter()
            .find(|f| f["code"] == "file-not-utf8")
            .expect("file-not-utf8 finding");
        assert_eq!(not_utf8["severity"], "error");
        assert_eq!(not_utf8["path"], "dic/b.pasta");
        assert!(not_utf8["line"].is_null());
        assert!(findings.iter().any(|f| f["code"] == "file-unreadable"));
    }
}
