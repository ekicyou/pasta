//! 条件・フラグの評価（要件 1.1–1.11・7.1・7.3）
//!
//! シーンの条件（`＆＄＊経験値≧500`）とフラグ（`＆＄＊初回待ち`）を、
//! 選ぶ時点の Lua の変数表（`＄＊` は save、`＄` は var）の値で評価する。

use mlua::{Table, Value};
use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue, SceneInfo, has_conditions};

/// 比べる値。Lua の値と右辺の値を同じ形に揃える
#[derive(Debug, Clone, PartialEq)]
enum Operand {
    Nil,
    Number(f64),
    Text(String),
    /// 表・関数など
    Other,
}

/// 1 つの条件の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CondOutcome {
    Pass,
    Fail,
    /// 数でない値の大小（満たさない＋warn）
    NotNumber,
}

impl Operand {
    #[allow(clippy::useless_conversion)] // mlua::Integer は x64 で i64・x86 で i32
    fn from_lua(value: Value) -> Self {
        match value {
            Value::Nil => Operand::Nil,
            Value::Boolean(b) => Operand::Number(if b { 1.0 } else { 0.0 }),
            Value::Integer(i) => Operand::Number(i64::from(i) as f64),
            Value::Number(n) => Operand::Number(n),
            Value::String(s) => Operand::from_text(s.to_string_lossy()),
            _ => Operand::Other,
        }
    }

    fn from_attr(value: &SceneAttrValue) -> Self {
        match value {
            SceneAttrValue::Integer(i) => Operand::Number(*i as f64),
            SceneAttrValue::Float(f) => Operand::Number(*f),
            SceneAttrValue::String(s) => Operand::from_text(s.clone()),
        }
    }

    fn from_text(s: String) -> Self {
        parse_decimal(&s).map_or(Operand::Text(s), Operand::Number)
    }

    /// 文字列として比べるときの表記。Other はどの文字列とも等しくない
    fn as_text(&self) -> Option<String> {
        match self {
            Operand::Nil => Some(String::new()),
            Operand::Number(n) => Some(SceneAttrValue::Float(*n).to_string()),
            Operand::Text(s) => Some(s.clone()),
            Operand::Other => None,
        }
    }
}

/// ログに出す値の表記
impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Nil => f.write_str("nil"),
            Operand::Other => f.write_str("<non-scalar>"),
            other => f.write_str(&other.as_text().unwrap_or_default()),
        }
    }
}

/// 半角の十進の数の形（`-?[0-9]+(\.[0-9]+)?`）だけを数として読む
fn parse_decimal(s: &str) -> Option<f64> {
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let body = s.strip_prefix('-').unwrap_or(s);
    let ok = match body.split_once('.') {
        Some((int, frac)) => digits(int) && digits(frac),
        None => digits(body),
    };
    if ok { s.parse().ok() } else { None }
}

/// 1 つの条件・フラグの評価。宣言は条件でないので Pass
fn evaluate(attr: &SceneAttr, value: &Operand) -> CondOutcome {
    let pass = |b: bool| {
        if b {
            CondOutcome::Pass
        } else {
            CondOutcome::Fail
        }
    };
    let (op, right) = match attr {
        SceneAttr::Decl { .. } => return CondOutcome::Pass,
        SceneAttr::Flag { .. } => {
            return pass(match value {
                Operand::Nil => false,
                Operand::Number(n) => *n != 0.0,
                Operand::Text(s) => !s.is_empty(),
                Operand::Other => true,
            });
        }
        SceneAttr::Cond { op, value, .. } => (*op, Operand::from_attr(value)),
    };
    let left = match value {
        Operand::Nil if matches!(right, Operand::Number(_)) => Operand::Number(0.0),
        other => other.clone(),
    };
    if let (Operand::Number(l), Operand::Number(r)) = (&left, &right) {
        return pass(match op {
            CmpOp::Ge => l >= r,
            CmpOp::Le => l <= r,
            CmpOp::Gt => l > r,
            CmpOp::Lt => l < r,
            CmpOp::Eq => l == r,
            CmpOp::Ne => l != r,
        });
    }
    let equal = match (left.as_text(), right.as_text()) {
        (Some(l), Some(r)) => l == r,
        _ => false,
    };
    match op {
        CmpOp::Eq => pass(equal),
        CmpOp::Ne => pass(!equal),
        _ => CondOutcome::NotNumber,
    }
}

/// `＄＊名前` は save、`＄名前` は var から素の表として引く（メタメソッドを呼ばない）
fn lookup(key: &str, var: Option<&Table>, save: Option<&Table>) -> Operand {
    let (table, name) = match key.strip_prefix("＄＊") {
        Some(name) => (save, name),
        None => (var, key.strip_prefix("＄").unwrap_or(key)),
    };
    table
        .and_then(|t| t.raw_get::<Value>(name).ok())
        .map_or(Operand::Nil, Operand::from_lua)
}

/// 条件の表記（`＄＊経験値 >= 500`・フラグは `＄＊初回待ち`）
fn cond_text(attr: &SceneAttr) -> String {
    match attr {
        SceneAttr::Cond { key, op, value } => format!("{key} {} {value}", op.as_str()),
        other => other.key().to_string(),
    }
}

/// 変数表から述語を作る。`name` はログに出す探した名前。
/// 条件の無いシーンは表を読まずに真、条件は書いた順に見て最初の外れで止める（かつ）。
pub(crate) fn cond_predicate<'a>(
    name: &'a str,
    var: Option<Table>,
    save: Option<Table>,
) -> impl FnMut(&SceneInfo) -> bool + 'a {
    move |scene| {
        if !has_conditions(&scene.attributes) {
            return true;
        }
        scene.attributes.iter().all(|attr| {
            if matches!(attr, SceneAttr::Decl { .. }) {
                return true;
            }
            let value = lookup(attr.key(), var.as_ref(), save.as_ref());
            match evaluate(attr, &value) {
                CondOutcome::Pass => true,
                CondOutcome::Fail => {
                    tracing::trace!(
                        target: "pasta_lua::search",
                        name,
                        scene = %scene.fn_name,
                        cond = %cond_text(attr),
                        value = %value,
                        "Scene candidate excluded by condition"
                    );
                    false
                }
                CondOutcome::NotNumber => {
                    tracing::warn!(
                        target: "pasta_lua::search",
                        name,
                        scene = %scene.fn_name,
                        cond = %cond_text(attr),
                        value = %value,
                        "Condition compares a non-number with an ordering operator; treated as unmet"
                    );
                    false
                }
            }
        })
    }
}

#[cfg(test)]
#[path = "scene_cond_tests.rs"]
mod tests;
