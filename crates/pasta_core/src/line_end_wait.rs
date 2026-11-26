//! 行末の待ちのキー 5 つの名前の表。

/// 行末の待ちのキー（字の種類の順: ふつうの字・句点・読点・強調・リーダー）
pub const LINE_END_WAIT_KEYS: [&str; 5] = [
    "script_wait_normal_eol",
    "script_wait_period_eol",
    "script_wait_comma_eol",
    "script_wait_strong_eol",
    "script_wait_leader_eol",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_follow_script_wait_kind_order() {
        let expected = ["normal", "period", "comma", "strong", "leader"]
            .map(|kind| format!("script_wait_{kind}_eol"));
        assert_eq!(LINE_END_WAIT_KEYS.to_vec(), expected.to_vec());
    }
}
