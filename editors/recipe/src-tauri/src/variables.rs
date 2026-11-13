//! 保存変数の表の行（`variables.toml` → 行）と、ローカル変数のインライン表（組 → Reference2）。
//! ランタイムと同じ `toml` と、名前の決まり `pasta_dsl::parser::is_id` を使う。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use toml::Value;

/// 変数の型（editors/recipe/src/ipc.ts の VarRow・LocalPair の kind と同じ綴り）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VarKind {
    String,
    Number,
    Boolean,
    /// 表の行だけ（ローカル変数には来ない）
    Table,
}

/// 保存変数の表の 1 行
#[derive(Debug, PartialEq, Serialize)]
pub struct VarRow {
    pub name: String,
    pub kind: VarKind,
    pub value: String,
}

/// 窓で入れたローカル変数の 1 組
#[derive(Debug, Deserialize)]
pub struct LocalPair {
    pub name: String,
    pub kind: VarKind,
    pub value: String,
}

/// 弾いた組（`index` は渡された組の並びの 0 から数えた番号）
#[derive(Debug, PartialEq, Serialize)]
pub struct LocalsProblem {
    pub index: usize,
    pub message: String,
}

/// インライン表を組めたか、弾いたか（ipc.ts の LocalsOutcome と同じ形）
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LocalsOutcome {
    Ok { table: String },
    Invalid { problems: Vec<LocalsProblem> },
}

/// `variables.toml` の文を、最上位のキーごとの行（名前の順）にする。TOML でなければ理由（6.5）
pub fn variable_rows(text: &str) -> Result<Vec<VarRow>, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| e.message().to_string())?;
    let mut rows: Vec<VarRow> = table
        .into_iter()
        .map(|(name, v)| {
            let (kind, value) = match v {
                Value::String(s) => (VarKind::String, s),
                Value::Integer(_) | Value::Float(_) => (VarKind::Number, v.to_string()),
                Value::Boolean(b) => (VarKind::Boolean, b.to_string()),
                Value::Table(_) | Value::Array(_) => (VarKind::Table, v.to_string()),
                // ランタイムは書かないが、来たら文字として出す
                Value::Datetime(d) => (VarKind::String, d.to_string()),
            };
            VarRow { name, kind, value }
        })
        .collect();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rows)
}

/// 組を全部見て、問題が無ければ `{ "キー" = 値, … }` のインライン表を組む（8.3・8.4・8.6〜8.8）
pub fn build_locals(pairs: &[LocalPair]) -> LocalsOutcome {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    let mut parts = Vec::new();
    for (index, p) in pairs.iter().enumerate() {
        let mut problem = |message: String| problems.push(LocalsProblem { index, message });
        if !pasta_dsl::parser::is_id(&p.name) {
            problem(format!(
                "＄{} と書ける名前ではありません（空・数字だけ・__名前__・空白や記号を含む）",
                p.name
            ));
        }
        if !seen.insert(p.name.as_str()) {
            problem(format!("同じ名前の組があります: {}", p.name));
        }
        let value = match p.kind {
            VarKind::Number => match parse_number(&p.value) {
                Some(v) => v,
                None => {
                    problem(format!("数として読めません: {}", p.value));
                    continue;
                }
            },
            VarKind::Boolean => Value::Boolean(p.value == "true"),
            VarKind::String | VarKind::Table => Value::String(p.value.clone()),
        };
        // キーもいつも引用符で包む（日本語のキーも TOML として正しく書かれる）
        parts.push(format!("{} = {}", Value::String(p.name.clone()), value));
    }
    if problems.is_empty() {
        LocalsOutcome::Ok {
            table: format!("{{ {} }}", parts.join(", ")),
        }
    } else {
        LocalsOutcome::Invalid { problems }
    }
}

/// `i64` か有限の `f64` として読めるときだけ数にする（`inf`・`nan`・空は弾く）
fn parse_number(s: &str) -> Option<Value> {
    if let Ok(i) = s.parse::<i64>() {
        return Some(Value::Integer(i));
    }
    s.parse::<f64>()
        .ok()
        .filter(|f| f.is_finite())
        .map(Value::Float)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(name: &str, kind: VarKind, value: &str) -> LocalPair {
        LocalPair {
            name: name.into(),
            kind,
            value: value.into(),
        }
    }

    fn table_of(outcome: LocalsOutcome) -> toml::Table {
        let LocalsOutcome::Ok { table } = outcome else {
            panic!("弾かれた: {outcome:?}");
        };
        let mut doc: toml::Table = toml::from_str(&format!("x = {table}")).unwrap();
        match doc.remove("x") {
            Some(Value::Table(t)) => t,
            other => panic!("表でない: {other:?}"),
        }
    }

    fn problems_of(outcome: LocalsOutcome) -> Vec<LocalsProblem> {
        match outcome {
            LocalsOutcome::Invalid { problems } => problems,
            ok => panic!("通ってしまった: {ok:?}"),
        }
    }

    #[test]
    fn 組み立てて読み直すと入れたとおりに戻る() {
        let s = "a'b\"c\\d e\tf 日本語";
        let t = table_of(build_locals(&[
            pair("天気", VarKind::String, s),
            pair("回数", VarKind::Number, "3"),
            pair("率", VarKind::Number, "-1.5"),
            pair("晴れ", VarKind::Boolean, "true"),
            pair("雨", VarKind::Boolean, "false"),
            pair("name_1", VarKind::String, ""),
        ]));
        assert_eq!(t.len(), 6);
        assert_eq!(t["天気"], Value::String(s.into()));
        assert_eq!(t["回数"], Value::Integer(3));
        assert_eq!(t["率"], Value::Float(-1.5));
        assert_eq!(t["晴れ"], Value::Boolean(true));
        assert_eq!(t["雨"], Value::Boolean(false));
        assert_eq!(t["name_1"], Value::String(String::new()));
    }

    #[test]
    fn キーはいつも引用符で包む() {
        let LocalsOutcome::Ok { table } = build_locals(&[pair("a", VarKind::Number, "1")]) else {
            panic!();
        };
        assert_eq!(table, "{ \"a\" = 1 }");
    }

    #[test]
    fn 組が無ければ空の表() {
        assert!(table_of(build_locals(&[])).is_empty());
    }

    #[test]
    fn 書けない名前を弾き書ける名前を通す() {
        for bad in ["", "1", "__x__", "a b", "a-b", "a.b"] {
            let p = problems_of(build_locals(&[pair(bad, VarKind::String, "v")]));
            assert_eq!(p.len(), 1, "{bad:?}");
            assert_eq!(p[0].index, 0);
            assert_eq!(
                p[0].message,
                format!(
                    "＄{bad} と書ける名前ではありません（空・数字だけ・__名前__・空白や記号を含む）"
                )
            );
        }
        for good in ["名前", "name_1"] {
            assert!(matches!(
                build_locals(&[pair(good, VarKind::String, "v")]),
                LocalsOutcome::Ok { .. }
            ));
        }
    }

    #[test]
    fn 重なりを弾く() {
        let p = problems_of(build_locals(&[
            pair("a", VarKind::String, "1"),
            pair("b", VarKind::String, "2"),
            pair("a", VarKind::Number, "3"),
        ]));
        assert_eq!(
            p,
            vec![LocalsProblem {
                index: 2,
                message: "同じ名前の組があります: a".into()
            }]
        );
    }

    #[test]
    fn 数でない値を弾き全部の組を返す() {
        let p = problems_of(build_locals(&[
            pair("a", VarKind::Number, "abc"),
            pair("ok", VarKind::Number, "2"),
            pair("b", VarKind::Number, "inf"),
            pair("c", VarKind::Number, ""),
            pair("d d", VarKind::Number, "nan"),
        ]));
        let got: Vec<_> = p.iter().map(|x| (x.index, x.message.as_str())).collect();
        assert_eq!(
            got,
            vec![
                (0, "数として読めません: abc"),
                (2, "数として読めません: inf"),
                (3, "数として読めません: "),
                (
                    4,
                    "＄d d と書ける名前ではありません（空・数字だけ・__名前__・空白や記号を含む）"
                ),
                (4, "数として読めません: nan"),
            ]
        );
    }

    #[test]
    fn 表の行は4つの型で名前の順() {
        let rows = variable_rows(
            r#"
s = "雨 'x' \"y\""
i = 3
f = -1.5
b = true
"a.b" = "引用符付き"
arr = [1, "x"]
[t]
k = 1
[t.u]
v = "w"
"#,
        )
        .unwrap();
        let got: Vec<_> = rows
            .iter()
            .map(|r| (r.name.as_str(), r.kind, r.value.as_str()))
            .collect();
        assert_eq!(got[0], ("a.b", VarKind::String, "引用符付き"));
        assert_eq!(got[1], ("arr", VarKind::Table, "[1, \"x\"]"));
        assert_eq!(got[2], ("b", VarKind::Boolean, "true"));
        assert_eq!(got[3], ("f", VarKind::Number, "-1.5"));
        assert_eq!(got[4], ("i", VarKind::Number, "3"));
        assert_eq!(got[5], ("s", VarKind::String, "雨 'x' \"y\""));
        assert_eq!(got[6].0, "t");
        assert_eq!(got[6].1, VarKind::Table);
        // 入れ子の表も読める形（TOML のインラインの書き方）で出す
        let back: toml::Table = toml::from_str(&format!("x = {}", got[6].2)).unwrap();
        assert_eq!(back["x"]["k"], Value::Integer(1));
        assert_eq!(back["x"]["u"]["v"], Value::String("w".into()));
        assert_eq!(rows.len(), 7);
    }

    #[test]
    fn 空の文は行が無い() {
        assert!(variable_rows("").unwrap().is_empty());
    }

    #[test]
    fn tomlでない文は理由で断る() {
        let e = variable_rows("a = = 1").unwrap_err();
        assert!(!e.is_empty());
    }

    #[test]
    fn 形は画面の型と同じ() {
        let row = VarRow {
            name: "a".into(),
            kind: VarKind::Table,
            value: "{}".into(),
        };
        assert_eq!(
            serde_json::to_string(&row).unwrap(),
            r#"{"name":"a","kind":"table","value":"{}"}"#
        );
        let p: LocalPair =
            serde_json::from_str(r#"{"name":"a","kind":"number","value":"1"}"#).unwrap();
        assert_eq!(p.kind, VarKind::Number);
        assert_eq!(
            serde_json::to_string(&LocalsOutcome::Ok { table: "{}".into() }).unwrap(),
            r#"{"kind":"ok","table":"{}"}"#
        );
        assert_eq!(
            serde_json::to_string(&LocalsOutcome::Invalid {
                problems: vec![LocalsProblem {
                    index: 1,
                    message: "m".into()
                }]
            })
            .unwrap(),
            r#"{"kind":"invalid","problems":[{"index":1,"message":"m"}]}"#
        );
    }
}
