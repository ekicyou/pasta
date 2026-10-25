//! 版 3 の口（shiori4-v3-portable）: `shiori_get_api` と `PastaV3`。
//!
//! ABI の検査（`copy_from_slice` の有無・UTF-8・パニックの捕捉・応答の写し）は `shiori4_abi` が
//! 行う。ここは要求の見分けと `ActorInstance` への受け渡しだけを持つ。

use std::path::PathBuf;

use pasta_lua::LoadDirGuard;
use shiori4_abi::{
    SHIORI_204_NO_CONTENT, SHIORI_400_BAD_REQUEST, SHIORI_500_INTERNAL_SERVER_ERROR,
    SHIORI_ABI_VERSION_3, SHIORI_E_INVALID_ARG, SHIORI_E_UNSUPPORTED, Shiori, ShioriApi,
    ShioriLoadMode, ShioriResponse, ShioriStatus,
};
use tracing::{debug, info, warn};

use crate::actor::instance::ActorInstance;
use crate::actor::marshaling::{RequestKind, classify_request};

/// 版 3 のテーブル（プロセスの間ずっと同じ static・1.4）。
static API_V3: ShioriApi = ShioriApi::new::<PastaV3, { SHIORI_ABI_VERSION_3 }>();

/// shiori4 のホストが最初に呼ぶ口。版 3 だけにテーブルを返し、ほか（4 を含む）は NULL（1.1〜1.4）。
#[unsafe(no_mangle)]
pub extern "C" fn shiori_get_api(requested: u32) -> *const ShioriApi {
    if requested == SHIORI_ABI_VERSION_3 {
        &API_V3
    } else {
        std::ptr::null()
    }
}

/// 版 3 のインスタンス。`ActorInstance` を 1 つだけ持つ（unload は `ActorInstance` の Drop）。
pub struct PastaV3 {
    instance: ActorInstance,
}

impl PastaV3 {
    /// 弾いた要求を、このインスタンスのログに 1 行残す。
    fn reject(&self, kind: RequestKind, reason: &str) {
        let _log_context = LoadDirGuard::new(self.instance.load_dir().to_path_buf());
        debug!(entry = "v3", ?kind, reason, "request rejected with 400");
    }
}

impl Shiori for PastaV3 {
    const SUPPORTS_V3: bool = true;

    fn load(mode: ShioriLoadMode, dir: &str) -> Result<Self, ShioriStatus> {
        if !matches!(mode, ShioriLoadMode::V3) {
            // 版 4 のテーブルは出さないので来ない。守りだけ。
            return Err(SHIORI_E_UNSUPPORTED);
        }
        // カレントディレクトリに頼らない（2.2）。この段の NULL はログを出さない。
        let load_dir = PathBuf::from(dir);
        if !load_dir.is_absolute() || !load_dir.is_dir() {
            return Err(SHIORI_E_INVALID_ARG);
        }
        // 立ち上がれない理由のログは spawn が出す（2.4・2.5）。
        let instance = ActorInstance::spawn(0, load_dir.clone())
            .map_err(|_| SHIORI_500_INTERNAL_SERVER_ERROR)?;
        // 辞書・Lua・SHIORI.load の失敗でもインスタンスを返す（2.7）。入口の記録は旧来の入口と同じく
        // load_dir の文脈を張り直して出す（入口スレッドにはアクタースレッドの文脈が及ばない）。
        let loaded = instance.loaded();
        let _log_context = LoadDirGuard::new(load_dir);
        info!(entry = "v3", loaded, "SHIORI load entry completed");
        Ok(Self { instance })
    }

    fn request(&mut self, req: &str) -> ShioriResponse {
        let kind = classify_request(req);
        let (status, body) = match kind {
            RequestKind::Get => {
                let body = self.instance.get(req);
                // status_of の警告をこのインスタンスのログへ出すための文脈。
                let _log_context = LoadDirGuard::new(self.instance.load_dir().to_path_buf());
                (status_of(&body), body)
            }
            RequestKind::Notify => (SHIORI_204_NO_CONTENT, self.instance.notify(req)),
            RequestKind::Shiori2 | RequestKind::Unreadable => {
                let reason = reject_reason(kind);
                self.reject(kind, reason);
                (SHIORI_400_BAD_REQUEST, bad_request(reason))
            }
        };
        ShioriResponse::with_status(status, body)
    }

    fn notify(&mut self, req: &str) -> ShioriStatus {
        let kind = classify_request(req);
        if kind == RequestKind::Notify {
            self.instance.notify(req);
            return SHIORI_204_NO_CONTENT;
        }
        // GET・2.x・読めない要求はアクターへ送らない（3.4〜3.6）。
        self.reject(kind, reject_reason(kind));
        SHIORI_400_BAD_REQUEST
    }
}

/// 弾く要求の理由（`X-ERROR-REASON` とログに出す）。
fn reject_reason(kind: RequestKind) -> &'static str {
    match kind {
        RequestKind::Get => "GET is not accepted by notify",
        RequestKind::Shiori2 => "SHIORI/2.x is not supported",
        RequestKind::Notify | RequestKind::Unreadable => "unreadable request",
    }
}

/// 応答の開始行 `SHIORI/3.0 NNN` の NNN。読めなければ 500（本文は変えない・3.7）。
fn status_of(response: &str) -> ShioriStatus {
    let line = response.lines().next().unwrap_or("");
    let code = line.strip_prefix("SHIORI/3.0 ").filter(|rest| {
        let b = rest.as_bytes();
        b.len() >= 3 && b[..3].iter().all(u8::is_ascii_digit) && b.get(3).is_none_or(|&c| c == b' ')
    });
    match code.and_then(|rest| rest[..3].parse().ok()) {
        Some(status) => status,
        None => {
            warn!(
                seam = "v3.status",
                "response start line is unreadable -> status 500, body returned as is"
            );
            SHIORI_500_INTERNAL_SERVER_ERROR
        }
    }
}

/// 400 の全文（理由のヘッダ付き）。
fn bad_request(reason: &str) -> String {
    format!(
        "SHIORI/3.0 400 Bad Request\r\n\
         Charset: UTF-8\r\n\
         X-ERROR-REASON: {reason}\r\n\
         \r\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 開始行の読み取り（3.1・3.7）。
    #[test]
    fn status_of_reads_start_line() {
        assert_eq!(
            status_of("SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\n\r\n"),
            200
        );
        assert_eq!(status_of("SHIORI/3.0 204 No Content"), 204);
        assert_eq!(
            status_of("SHIORI/3.0 500 Internal Server Error\r\nX-ERROR-REASON: x\r\n\r\n"),
            500
        );
    }

    #[test]
    fn status_of_unreadable_is_500() {
        for response in [
            "garbage",
            "SHIORI/3.0 2x0",
            "SHIORI/3.0 2x0 OK\r\n\r\n",
            "SHIORI/3.0 2000 OK\r\n",
            "SHIORI/2.6 200 OK",
            "",
        ] {
            assert_eq!(status_of(response), 500, "response: {response:?}");
        }
    }

    #[test]
    fn bad_request_carries_reason_header() {
        let body = bad_request("SHIORI/2.x is not supported");
        assert_eq!(status_of(&body), 400);
        assert!(body.contains("\r\nX-ERROR-REASON: SHIORI/2.x is not supported\r\n"));
        assert!(body.ends_with("\r\n\r\n"));
    }

    /// 版 3 だけにテーブルを返し、同じポインタ（1.2〜1.4）。
    #[test]
    fn get_api_returns_v3_table_only() {
        let a = shiori_get_api(SHIORI_ABI_VERSION_3);
        assert!(!a.is_null());
        assert_eq!(a, shiori_get_api(SHIORI_ABI_VERSION_3));
        for v in [0, 2, 4, 5] {
            assert!(shiori_get_api(v).is_null(), "version {v}");
        }
    }

    /// 版 4・相対パス・ディレクトリでないは立ち上げずに弾く（2.2・2.4）。
    #[test]
    fn load_rejects_relative_and_missing_dir() {
        assert_eq!(
            PastaV3::load(ShioriLoadMode::V3, "relative/ghost").err(),
            Some(SHIORI_E_INVALID_ARG)
        );
        let missing = std::env::temp_dir().join("pasta_v3_no_such_dir_4_2");
        assert_eq!(
            PastaV3::load(ShioriLoadMode::V3, missing.to_str().unwrap()).err(),
            Some(SHIORI_E_INVALID_ARG)
        );
    }
}
