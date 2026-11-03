//! Public type definitions for scene management.
//!
//! SceneId, SceneScope, SceneInfo are used across multiple crates
//! and separated from the SceneTable implementation for clarity.

/// Unique identifier for a scene (Vec index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub usize);

/// Scene scope type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneScope {
    /// Global scene (accessible from any scope).
    Global,
    /// Local scene (accessible only within parent scene).
    Local,
}

/// Information about a single scene.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    /// Unique identifier for this scene.
    pub id: SceneId,
    /// Scene name.
    pub name: String,
    /// Scene scope.
    pub scope: SceneScope,
    /// 書いた順の属性（宣言・条件・フラグ）
    pub attributes: Vec<SceneAttr>,
    /// Scene function name in `global::local` form (from `SceneEntry::fn_name`).
    pub fn_name: String,
    /// Parent scene name (for local scenes).
    pub parent: Option<String>,
}

/// 属性の値。数は Lua と同じ見え方にそろえる（整数の値になる小数は整数）。
#[derive(Debug, Clone, PartialEq)]
pub enum SceneAttrValue {
    Integer(i64),
    Float(f64),
    /// 引用の有無は区別しない。空の引用は空文字列
    String(String),
}

impl SceneAttrValue {
    /// 整数値で i64 に収まる小数は Integer、それ以外は Float
    pub fn from_f64(f: f64) -> Self {
        // i64::MIN as f64 は -2^63 ちょうど、i64::MAX as f64 は 2^63 に丸まるので上は未満
        if f.fract() == 0.0 && f >= i64::MIN as f64 && f < i64::MAX as f64 {
            SceneAttrValue::Integer(f as i64)
        } else {
            SceneAttrValue::Float(f)
        }
    }
}

/// Integer は数の表記、Float は LuaJIT の tostring と同じ表記、String はそのまま
impl std::fmt::Display for SceneAttrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneAttrValue::Integer(v) => write!(f, "{}", v),
            SceneAttrValue::Float(v) => f.write_str(&lua_number(*v)),
            SceneAttrValue::String(v) => f.write_str(v),
        }
    }
}

/// C の `%.14g`（LuaJIT の tostring）。14 桁に丸めてから指数で固定表記か指数表記かを決める
fn lua_number(v: f64) -> String {
    // DSL からは出ない（大きすぎる数は読み込みエラー）が、Lua と同じ見え方にしておく
    if !v.is_finite() {
        let sign = if v.is_sign_negative() { "-" } else { "" };
        return format!("{sign}{}", if v.is_nan() { "nan" } else { "inf" });
    }
    let trim = |s: &str| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s.to_string()
        }
    };
    let sci = format!("{:.13e}", v);
    let (mantissa, exp) = sci.split_once('e').expect("LowerExp always has 'e'");
    let exp: i32 = exp.parse().expect("LowerExp exponent is an integer");
    if (-4..14).contains(&exp) {
        trim(&format!("{:.*}", (13 - exp) as usize, v))
    } else {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim(mantissa), exp.abs())
    }
}

/// 条件の比較演算子（pasta_dsl の CmpOp と正規形の文字列で行き来する）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

impl CmpOp {
    /// 正規形（`>=`・`<=`・`>`・`<`・`=`・`!=`）
    pub fn as_str(self) -> &'static str {
        match self {
            CmpOp::Ge => ">=",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Lt => "<",
            CmpOp::Eq => "=",
            CmpOp::Ne => "!=",
        }
    }

    /// 正規形の文字列から。それ以外は None
    pub fn from_normalized(s: &str) -> Option<Self> {
        match s {
            ">=" => Some(CmpOp::Ge),
            "<=" => Some(CmpOp::Le),
            ">" => Some(CmpOp::Gt),
            "<" => Some(CmpOp::Lt),
            "=" => Some(CmpOp::Eq),
            "!=" => Some(CmpOp::Ne),
            _ => None,
        }
    }
}

/// シーンの属性の項目
#[derive(Debug, Clone, PartialEq)]
pub enum SceneAttr {
    /// `＆名前：値`
    Decl { key: String, value: SceneAttrValue },
    /// `＆＄変数 演算子 値`
    Cond {
        key: String,
        op: CmpOp,
        value: SceneAttrValue,
    },
    /// `＆＄変数`
    Flag { key: String },
}

impl SceneAttr {
    pub fn key(&self) -> &str {
        match self {
            SceneAttr::Decl { key, .. } | SceneAttr::Cond { key, .. } | SceneAttr::Flag { key } => {
                key
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_f64_integer_valued_becomes_integer() {
        assert_eq!(SceneAttrValue::from_f64(1.0), SceneAttrValue::Integer(1));
        assert_eq!(SceneAttrValue::from_f64(-3.0), SceneAttrValue::Integer(-3));
        assert_eq!(SceneAttrValue::from_f64(0.0), SceneAttrValue::Integer(0));
    }

    #[test]
    fn from_f64_keeps_fraction_and_out_of_range() {
        assert_eq!(SceneAttrValue::from_f64(1.5), SceneAttrValue::Float(1.5));
        assert_eq!(SceneAttrValue::from_f64(1e20), SceneAttrValue::Float(1e20));
        assert!(
            matches!(SceneAttrValue::from_f64(f64::NAN), SceneAttrValue::Float(f) if f.is_nan())
        );
        assert_eq!(
            SceneAttrValue::from_f64(f64::INFINITY),
            SceneAttrValue::Float(f64::INFINITY)
        );
    }

    #[test]
    fn display_number_notation() {
        assert_eq!(SceneAttrValue::Integer(500).to_string(), "500");
        assert_eq!(SceneAttrValue::Integer(-2).to_string(), "-2");
        assert_eq!(SceneAttrValue::Float(1.5).to_string(), "1.5");
        assert_eq!(SceneAttrValue::Float(0.25).to_string(), "0.25");
        assert_eq!(SceneAttrValue::Float(0.5).to_string(), "0.5");
        assert_eq!(SceneAttrValue::Float(-1.5).to_string(), "-1.5");
        // 以下は LuaJIT の tostring（%.14g）と同じ表記
        assert_eq!(SceneAttrValue::Float(1e20).to_string(), "1e+20");
        assert_eq!(
            SceneAttrValue::Float(0.123456789012345).to_string(),
            "0.12345678901234"
        );
        assert_eq!(SceneAttrValue::Float(1e-5).to_string(), "1e-05");
        assert_eq!(SceneAttrValue::Float(0.0001).to_string(), "0.0001");
        assert_eq!(SceneAttrValue::Float(1e14).to_string(), "1e+14");
        assert_eq!(
            SceneAttrValue::Float(12345678901234.0).to_string(),
            "12345678901234"
        );
        // 14 桁に丸めてから桁を決める
        assert_eq!(
            SceneAttrValue::Float(99999999999999.99).to_string(),
            "1e+14"
        );
        assert_eq!(SceneAttrValue::Float(1.5e300).to_string(), "1.5e+300");
        assert_eq!(SceneAttrValue::Float(-2.5e-7).to_string(), "-2.5e-07");
        assert_eq!(SceneAttrValue::Float(-0.0).to_string(), "-0");
        assert_eq!(SceneAttrValue::Float(f64::INFINITY).to_string(), "inf");
        assert_eq!(SceneAttrValue::Float(f64::NEG_INFINITY).to_string(), "-inf");
        assert_eq!(SceneAttrValue::Float(f64::NAN).to_string(), "nan");
        assert_eq!(SceneAttrValue::String("夏".into()).to_string(), "夏");
        assert_eq!(SceneAttrValue::String(String::new()).to_string(), "");
    }

    #[test]
    fn cmp_op_round_trips_normal_form() {
        let all = [
            CmpOp::Ge,
            CmpOp::Le,
            CmpOp::Gt,
            CmpOp::Lt,
            CmpOp::Eq,
            CmpOp::Ne,
        ];
        let forms: Vec<&str> = all.iter().map(|op| op.as_str()).collect();
        assert_eq!(forms, [">=", "<=", ">", "<", "=", "!="]);
        for op in all {
            assert_eq!(CmpOp::from_normalized(op.as_str()), Some(op));
        }
        assert_eq!(CmpOp::from_normalized("≧"), None);
        assert_eq!(CmpOp::from_normalized("=="), None);
        assert_eq!(CmpOp::from_normalized(""), None);
    }

    #[test]
    fn scene_attr_key() {
        let v = SceneAttrValue::Integer(1);
        assert_eq!(
            SceneAttr::Decl {
                key: "季節".into(),
                value: v.clone()
            }
            .key(),
            "季節"
        );
        assert_eq!(
            SceneAttr::Cond {
                key: "＄経験値".into(),
                op: CmpOp::Ge,
                value: v
            }
            .key(),
            "＄経験値"
        );
        assert_eq!(
            SceneAttr::Flag {
                key: "＄＊初回".into()
            }
            .key(),
            "＄＊初回"
        );
    }
}
