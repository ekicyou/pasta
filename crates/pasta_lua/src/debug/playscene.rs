//! 位置→シーン解決器（[`resolve_and_kick`]）— `(uri, line)` を確定シーンへ解決し、
//! 既存 [`KickSink`] へ取り次ぐ（task 3.1・requirements 2.1-2.6/7.1/7.6/8.1/8.3）。
//!
//! # 役割
//!
//! VSCode 由来の位置 `(uri, line)` を:
//!
//! 1. **正規化**: uri を `.pasta` ファイルパス文字列へ落とす（[`uri_to_pasta_path`]・
//!    `file://` スキーム除去＋パーセントデコード＋Windows ドライブ補正＋
//!    `std::path::absolute`）。`fs::canonicalize` は **使わない**（CI の 8.3 短縮名
//!    `RUNNER~1` パス不具合回避・既存メモ）。索引キー側の `\`→`/`・Windows 大小無視は
//!    [`SourceMap::scene_at`] 内の `canonicalize_pasta_file` が担うため、本関数は
//!    純粋なファイルシステムパス文字列の生成に専念する。
//! 2. **解決**: ロード済み索引（`SourceMap::scene_at`）で `(scene_id, parent)` を確定。
//!    包含（最内 local 優先）＞後方フォールバック＞未検出（[`SceneIdentityIndex`] が担う）。
//! 3. **取次**: 確定時は composite-string 方式で `KickRequest.scene` を組み（global →
//!    `会話1`、local → `:会話1:挨拶_1`）、`KickSink` を fire-and-forget で呼ぶ。
//!    `KickRequest{scene:String}` 契約は不変（pasta_shiori 無改修・design「3.1 への波及」）。
//!    未検出時は sink を呼ばず [`ResolveOutcome::NotFound`] を返す（caller=task 4.1 が
//!    理由付きエラー応答へ変換する）。
//!
//! 索引 lookup は読み取り専用の `BTreeMap` 引きのみで GET ブロックを延ばさない（8.3）。
//!
//! # 通常モード非破壊
//!
//! 本解決器・composite-string は **debug kick 経路でのみ**到達する。通常 SSP トーク
//! 再生は `:`-prefixed scene を生成せず、本モジュールにも到達しない（debug opt-in）。

// NOTE: 参照側（DAP decode → transport dispatch）への結線は task 4.1。それまで
// `resolve_and_kick`/`ResolveOutcome`/`uri_to_pasta_path` は未消費のため、本モジュール
// 内の dead_code を明示許可する（結線後に消費され警告は自然消滅する）。
#![allow(dead_code)]

use super::kick::{KickRequest, KickSink};
use super::source_map::SourceMap;

/// 位置→シーン解決の結果（[`resolve_and_kick`] の返り値）。
///
/// caller（task 4.1 transport）が成功応答／理由付きエラー応答へ変換する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveOutcome {
    /// 確定: `KickSink` を呼び済み。載せた `KickRequest.scene` 文字列を保持する
    /// （global → `会話1`、local → `:会話1:挨拶_1`）。
    Resolved(String),
    /// 未検出: sink は呼んでいない（caller がエラー応答に変換する）。
    NotFound,
}

/// 確定 `(scene_id, parent)` から `KickRequest.scene` の search-key 文字列を組む
/// （composite-string 方式・design「3.1 への波及」）。
///
/// - global（`parent = None`）→ `scene_id` をそのまま（例 `会話1`）。
/// - local（`parent = Some(p)`）→ `:{p}:{scene_id}`（例 `:会話1:挨拶_1`）。
///
/// `:` 接頭辞は kick.lua `try_dispatch` が local-composite として分解する目印。global
/// 実名は `SceneRegistry::sanitize_name` により `:` を決して含まないため衝突しない。
fn build_kick_scene(scene_id: &str, parent: Option<&str>) -> String {
    match parent {
        Some(p) => format!(":{p}:{scene_id}"),
        None => scene_id.to_string(),
    }
}

/// 位置 `(uri, line)` を解決し、確定時は `KickSink` を呼ぶ（fire-and-forget）。
///
/// `uri` を [`uri_to_pasta_path`] で `.pasta` パス文字列へ正規化し、`map.scene_at`
/// で `(scene_id, parent)` を確定する。確定時は composite-string を組んで `kick` を
/// 呼び [`ResolveOutcome::Resolved`] を、未検出時は sink を呼ばず
/// [`ResolveOutcome::NotFound`] を返す。
///
/// `line` は VSCode 由来の 1 始まり行番号（索引も 1 始まり）。
pub fn resolve_and_kick(
    map: &SourceMap,
    kick: &KickSink,
    uri: &str,
    line: u32,
) -> ResolveOutcome {
    let pasta_path = uri_to_pasta_path(uri);
    match map.scene_at(&pasta_path, line) {
        Some(identity) => {
            let scene = build_kick_scene(&identity.scene_id, identity.parent.as_deref());
            kick(KickRequest {
                scene: scene.clone(),
            });
            ResolveOutcome::Resolved(scene)
        }
        None => ResolveOutcome::NotFound,
    }
}

/// VSCode uri → `.pasta` ファイルシステムパス文字列（純粋・単体テスト対象・task 6.3）。
///
/// 変換規則（`fs::canonicalize` 非使用・`std::path::absolute` 統一）:
///
/// 1. **`file://` スキーム除去**: `file:///c:/...` / `file://host/...` の双方を許容し、
///    スキームとオーソリティ（`//[host]`）を剥がす。スキームが無い生パスはそのまま扱う。
/// 2. **パーセントデコード**: `%20`→空白・`%3A`→`:` 等を復元する（VSCode uri は
///    エンコード済みで届く）。不正な `%xx` はリテラル `%` として保持する（防御的）。
/// 3. **Windows ドライブ補正**: 先頭 `/c:/...` の余分な `/` を除き `c:/...` にする。
/// 4. **絶対化**: [`std::path::absolute`] で正規化（相対セグメント解決・区切り統一）。
///    失敗時はデコード済み文字列をフォールバックで返す（決定論性）。
///
/// 索引キー側の `\`→`/`・Windows 大小無視は呼び出し先 `SourceMap::scene_at` 内の
/// `canonicalize_pasta_file` が担う。本関数は同一の正規形へ落とせる **ファイルシステム
/// パス**を生成することに専念する（design「VSCode 側 uri→path と索引キー生成を同一規則」）。
pub fn uri_to_pasta_path(uri: &str) -> String {
    // (1) `file://` スキーム＋オーソリティを除去。
    let without_scheme = strip_file_scheme(uri);
    // (2) パーセントデコード。
    let decoded = percent_decode(&without_scheme);
    // (3) Windows ドライブ補正（先頭 `/c:` → `c:`）。
    let drive_fixed = fix_leading_drive(&decoded);
    // (4) `std::path::absolute` で正規化（canonicalize は使わない）。
    match std::path::absolute(&drive_fixed) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => drive_fixed,
    }
}

/// `file://` スキームとオーソリティ部（`//[host]`）を剥がす。
///
/// `file:///c:/a.pasta` → `/c:/a.pasta`、`file://server/share/a.pasta` →
/// `/share/a.pasta`（UNC host は捨てる・ローカル `.pasta` 前提）。スキームの無い入力は
/// そのまま返す。大小文字を問わない（`FILE://` も許容）。
fn strip_file_scheme(uri: &str) -> String {
    let lower = uri.to_ascii_lowercase();
    let rest = if let Some(stripped) = lower.strip_prefix("file://") {
        // `file://` 後ろのバイト位置で元文字列を切る（大小・非 ASCII を保つため元から取る）。
        let cut = uri.len() - stripped.len();
        &uri[cut..]
    } else {
        return uri.to_string();
    };
    // オーソリティ（host）部を剥がす: `file://` 直後に host があれば最初の `/` まで捨てる。
    // `file:///c:/...`（host 空）→ rest は `/c:/...` で先頭が `/` なのでそのまま。
    // `file://host/share/...` → host を捨て `/share/...`。
    if let Some(slash) = rest.find('/') {
        if slash == 0 {
            rest.to_string()
        } else {
            rest[slash..].to_string()
        }
    } else {
        rest.to_string()
    }
}

/// 先頭 `/c:/...`（Windows ドライブの余分な `/`）を `c:/...` へ補正する。
///
/// `file:///c:/a.pasta` 由来の `/c:/a.pasta` を `c:/a.pasta` にする。`/c:` の形
/// （`/` + 英字 + `:`）のみ対象。POSIX 絶対パス `/home/...` は変更しない。
fn fix_leading_drive(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b':'
    {
        path[1..].to_string()
    } else {
        path.to_string()
    }
}

/// パーセントデコード（`%xx` → バイト）。不正な `%xx` はリテラル `%` として保持する。
///
/// VSCode uri はパス区切り以外（空白・`:`・非 ASCII 等）をエンコードして届けるため、
/// 索引キーと突合する前に復元する。デコード後バイト列は UTF-8 lossy で文字列化する。
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val(bytes[i + 1]);
            let lo = hex_val(bytes[i + 2]);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 1 桁の 16 進文字 → 値（不正なら `None`）。
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "playscene_tests.rs"]
mod tests;
