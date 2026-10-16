//! Shared test environment + helpers for the SHIORI async-callback integration
//! test clusters (`async_callback_simple_test.rs` / `async_callback_chain_test.rs`).
//!
//! These flat integration tests are each their own binary, so the shared
//! `AsyncCallbackEnv` harness and request helpers are kept here and `#[path]`-included
//! by each cluster file. Test-only helpers are exposed `pub(crate)`; production
//! visibility is unaffected.

#![allow(dead_code)]

use crate::common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};
use std::path::Path;
use tempfile::TempDir;

/// 非同期コールバックテスト用の環境構築
///
/// async_callback フィクスチャ（`entry.lua` の差し替えと `pasta.toml`）を一時ディレクトリへコピーする。
/// EVENT・CALLBACK・SHIORI_ACT などの本番モジュールは、`pasta.dll` が
/// `profile/pasta/pasta_scripts/` へ自己展開した標準ランタイムから読まれる。
pub(crate) struct AsyncCallbackEnv {
    shiori: PastaShiori,
    _temp_dir: TempDir,
}

impl AsyncCallbackEnv {
    pub(crate) fn new() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Copy fixture files (entry.lua override + pasta.toml).
        // scripts/ is searched before the self-deployed profile/pasta/pasta_scripts/.
        let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
        copy_dir_recursive(&fixture_src, temp_dir.path()).expect("copy fixture");

        let mut shiori = PastaShiori::default();
        let loaded = shiori
            .load(0, temp_dir.path().as_os_str())
            .expect("SHIORI load should not error");
        assert!(loaded, "SHIORI load should return true");

        Self {
            shiori,
            _temp_dir: temp_dir,
        }
    }

    pub(crate) fn request(&mut self, text: &str) -> ShioriResponse {
        let normalized = normalize_request(text);
        let raw = self
            .shiori
            .request(&normalized)
            .expect("SHIORI request should not error");
        ShioriResponse::parse(&raw).expect("Response should be parseable")
    }
}

pub(crate) fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut result = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    result.push_str("\r\n\r\n");
    result
}

pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Value 文字列から OnPastaCallBack{N} のイベントIDを抽出する
pub(crate) fn extract_callback_id(value: &str) -> String {
    let start = value
        .find("OnPastaCallBack")
        .expect("Value should contain OnPastaCallBack");
    let rest = &value[start..];
    let end = rest
        .find([',', ']'])
        .expect("OnPastaCallBack should be followed by ',' or ']'");
    rest[..end].to_string()
}
