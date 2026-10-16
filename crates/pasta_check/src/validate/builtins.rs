//! pasta.dll が組み込みで持つ Call の飛び先の名前（5.4・6.3）。
//!
//! 実物は pasta_lua の pasta_scripts。一致テストが実行時の `GLOBAL` と act の関数の名前を列挙して照らす。

/// `GLOBAL` の関数（`pasta/global.lua`・`pasta/shiori/entry.lua`）
pub const GLOBAL_NAMES: [&str; 4] = ["yield", "チェイントーク", "close_ghost", "ゴースト終了"];

/// act のメソッド（`pasta/act.lua` の `ACT_IMPL` の関数。メタメソッドの `__index` は除く）
pub const ACT_METHODS: [&str; 27] = [
    "init_scene",
    "talk",
    "sakura_script",
    "raw_script",
    "surface",
    "wait",
    "newline",
    "clear",
    "choice",
    "choice_timeout",
    "find_act_handler",
    "find_handler",
    "word",
    "expr_fn",
    "expr_fn_var",
    "actor_proxy",
    "global_fn",
    "build",
    "yield",
    "find_scene",
    "call",
    "call_key",
    "restore_scene",
    "call_restore",
    "failure",
    "set_spot",
    "clear_spot",
];

/// SHIORI 用の act のメソッド（`pasta/shiori/act.lua` の `SHIORI_ACT_IMPL` の関数。`__index` は除く）
pub const SHIORI_ACT_METHODS: [&str; 5] = [
    "build",
    "transfer_date_to_var",
    "transfer_req_to_var",
    "set_property",
    "get_property",
];

/// 名前が組み込みの飛び先のいずれかと完全に同じか
pub(crate) fn is_builtin(name: &str) -> bool {
    [&GLOBAL_NAMES[..], &ACT_METHODS, &SHIORI_ACT_METHODS]
        .iter()
        .any(|names| names.contains(&name))
}
