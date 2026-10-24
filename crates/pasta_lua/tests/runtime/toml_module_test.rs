//! Integration tests for the `@toml` module (mlua-stdlib-removal 2.1).

use crate::common::create_empty_context;

use pasta_lua::{PastaLuaRuntime, RuntimeConfig};

#[test]
fn test_toml_decode_available_in_minimal_config() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    let result = runtime
        .exec(
            r#"
            local t = require("@toml").decode('a = 1')
            return t.a == 1 and next(t, next(t)) == nil
        "#,
        )
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}

#[test]
fn test_toml_decode_error_contract() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    let result = runtime
        .exec(
            r#"
            local toml = require "@toml"
            local t1, e1 = toml.decode('a = ')
            local t2, e2 = toml.decode('a = "\255"')
            local ok = pcall(toml.decode, 42)
            local d = toml.decode('d = 1979-05-27 07:32:00').d
            return t1 == nil and #e1 > 0 and t2 == nil and #e2 > 0 and not ok
                and d == '1979-05-27T07:32:00'
        "#,
        )
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}
