use crate::error::*;
use crate::lua_request;
use pasta_lua::mlua::{Function, Table};
use pasta_lua::{
    ClaimToken, GlobalLoggerRegistry, LoadDirGuard, PastaLoader, PastaLuaRuntime, RuntimeConfig,
};
use std::{ffi::*, path::*};
use tracing::{debug, error, info, trace, warn};

/// PastaShiori - SHIORI implementation using pasta_lua engine.
///
/// Before loading, PastaShiori registers a default logger and initializes
/// tracing (Stage 1); PastaLoader then replaces it with the configured logger
/// (Stage 1.5). PastaShiori unregisters the logger on reload and drop.
#[derive(Default)]
pub struct PastaShiori {
    /// Base directory for ghost scripts (master/ directory)
    load_dir: Option<PathBuf>,

    runtime: Option<PastaLuaRuntime>,

    /// Error message from last failed load (for X-ERROR-REASON)
    last_load_error: Option<String>,

    load_fn: Option<Function>,

    request_fn: Option<Function>,

    unload_fn: Option<Function>,

    /// ディレクトリの占有の札（持ち主が外から渡す）。在れば VM の片付けで札で外し、無ければディレクトリで外す
    claim: Option<ClaimToken>,
}

impl PastaShiori {
    /// ディレクトリの占有の札を渡す。VM の片付けは、札の世代が今の世代のときだけ登録を外す
    /// （手放された古い VM が、占有を引き継いだ新しい持ち主の登録を消さない）。
    pub fn set_claim(&mut self, claim: Option<ClaimToken>) {
        self.claim = claim;
    }
}

// `PastaShiori` は `!Send`（mlua 由来）のままでよい。アクタースレッド（`actor::thread`）上でのみ
// 構築・所有され、スレッドを越境しない。`unsafe impl Send/Sync` を足さないこと。

impl Drop for PastaShiori {
    fn drop(&mut self) {
        self.call_lua_unload();
        self.release_runtime();
    }
}

impl PastaShiori {
    pub fn load<S: AsRef<OsStr>>(&mut self, load_dir: S) -> MyResult<bool> {
        let load_dir_path: PathBuf = load_dir.as_ref().into();

        if self.runtime.is_some() {
            info!("Releasing existing runtime for reload");
            self.release_runtime();
            self.last_load_error = None;
        }

        // 有無を確かめる前に覚える。ディレクトリが無いときも、
        // VM の片付けでそのディレクトリのログの登録・占有が外れるように。
        self.load_dir = Some(load_dir_path.clone());

        if !load_dir_path.exists() {
            error!(path = %load_dir_path.display(), "Load directory not found");
            // 以後の GET の 500 の理由に、探したパス（入口がデコードした形）を出す。
            self.last_load_error = Some(format!(
                "load directory not found: {}",
                load_dir_path.display()
            ));
            return Ok(false);
        }

        // Stage 1: a default logger so all load-phase logs are captured to file
        if let Ok(logger) = pasta_lua::PastaLogger::new(&load_dir_path, None) {
            let logger = std::sync::Arc::new(logger);
            GlobalLoggerRegistry::instance().register(load_dir_path.clone(), logger);
        }
        pasta_lua::init_tracing();

        let _guard = LoadDirGuard::new(load_dir_path.clone());

        info!(
            load_dir = %load_dir_path.display(),
            "Starting PastaShiori load"
        );

        // Stage 1.5 logger update happens inside.
        match PastaLoader::load_with_config(&load_dir_path, RuntimeConfig::new()) {
            Ok(runtime) => {
                self.cache_lua_functions(&runtime);

                self.runtime = Some(runtime);
                self.last_load_error = None;

                if !self.call_lua_load(&load_dir_path) {
                    return Ok(false);
                }

                info!(load_dir = %load_dir_path.display(), "PastaShiori load completed");
                Ok(true)
            }
            Err(e) => {
                error!(
                    load_dir = %load_dir_path.display(),
                    error = %e,
                    "PastaShiori load failed"
                );
                self.last_load_error = Some(format!("{}", e));
                Ok(false)
            }
        }
    }

    pub fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| match &self.last_load_error {
                Some(msg) => MyError::Load(msg.clone()),
                None => MyError::NotInitialized,
            })?;

        let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));

        let req = req.as_ref();
        trace!(request_len = req.len(), "Processing SHIORI request");

        self.call_lua_request(runtime, req)
    }

    /// Get a reference to the internal Lua runtime.
    /// Returns None if the runtime has not been initialized via load().
    pub fn runtime(&self) -> Option<&PastaLuaRuntime> {
        self.runtime.as_ref()
    }

    /// Cache SHIORI.load, SHIORI.request, and SHIORI.unload so each request skips the table lookup.
    fn cache_lua_functions(&mut self, runtime: &PastaLuaRuntime) {
        let lua = runtime.lua();
        let globals = lua.globals();

        let shiori_table: Result<Table, _> = globals.get("SHIORI");
        match shiori_table {
            Ok(table) => {
                self.load_fn = Self::lookup_lua_fn(&table, "load", false);
                self.request_fn = Self::lookup_lua_fn(&table, "request", false);
                self.unload_fn = Self::lookup_lua_fn(&table, "unload", true);
            }
            Err(e) => {
                warn!(error = %e, "SHIORI table not found");
                self.clear_cached_lua_functions();
            }
        }
    }

    /// Look up `SHIORI.<name>` in the table, logging cache hit/miss.
    /// `optional` functions log a miss at debug level instead of warn.
    fn lookup_lua_fn(table: &Table, name: &str, optional: bool) -> Option<Function> {
        match table.get::<Function>(name) {
            Ok(f) => {
                trace!("SHIORI.{} function cached", name);
                Some(f)
            }
            Err(_) if optional => {
                debug!("SHIORI.{} function not found (optional)", name);
                None
            }
            Err(_) => {
                warn!("SHIORI.{} function not found", name);
                None
            }
        }
    }

    /// Release the runtime, then unregister the logger (last, so every log
    /// emitted while releasing — persistence save, unregister notice — is
    /// written before the log file is closed). Shared by `Drop` and reload.
    fn release_runtime(&mut self) {
        let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));

        // Cached functions reference the VM; drop them before the runtime.
        self.clear_cached_lua_functions();

        // Runtime drop saves persistence data (logs still reach the ghost logger).
        self.runtime = None;

        if let Some(ref load_dir) = self.load_dir {
            info!(load_dir = %load_dir.display(), "Unregistering logger");
            // 札は一度きり。読み直しの後に古い世代の札で外し損ねないよう、使ったら手放す。
            match self.claim.take() {
                Some(token) => GlobalLoggerRegistry::instance().release_claim(&token),
                None => GlobalLoggerRegistry::instance().unregister(load_dir),
            }
        }
    }

    fn clear_cached_lua_functions(&mut self) {
        self.load_fn = None;
        self.request_fn = None;
        self.unload_fn = None;
    }

    /// Returns true if successful or if SHIORI.load doesn't exist (skip).
    /// Returns false if it returns false or errors.
    fn call_lua_load(&self, load_dir: &Path) -> bool {
        let load_fn = match &self.load_fn {
            Some(f) => f,
            None => {
                debug!("SHIORI.load not available, skipping");
                return true;
            }
        };

        // SHIORI の load は hinst を渡さないので、作者への約束どおり 0 を渡す
        let load_dir_str = load_dir.to_string_lossy().to_string();
        match load_fn.call::<bool>((0, load_dir_str)) {
            Ok(true) => {
                info!("SHIORI.load called successfully");
                true
            }
            Ok(false) => {
                warn!("SHIORI.load returned false");
                false
            }
            Err(e) => {
                error!(error = %e, "SHIORI.load execution failed");
                false
            }
        }
    }

    /// Returns 204 response if SHIORI.request doesn't exist.
    /// Returns 400 Bad Request if request parsing fails.
    fn call_lua_request(&self, runtime: &PastaLuaRuntime, request: &str) -> MyResult<String> {
        let request_fn = match &self.request_fn {
            Some(f) => f,
            None => {
                debug!("SHIORI.request not available, returning default 204 response");
                return Ok(crate::actor::marshaling::default_204());
            }
        };

        let lua = runtime.lua();

        let req_table = match lua_request::parse_request(lua, request) {
            Ok(table) => table,
            Err(e) => {
                error!(error = %e, "SHIORI request parsing failed");
                return Ok(e.to_shiori_400_response());
            }
        };

        match request_fn.call::<String>(req_table) {
            Ok(response) => {
                if !response.starts_with("SHIORI/3.0 204 No Content") {
                    debug!(request = %request, "### SHIORI request ###\n");
                    debug!(response = %response, "### SHIORI response###\n");
                }
                trace!(response_len = response.len(), "SHIORI.request completed");
                Ok(response)
            }
            Err(e) => {
                error!(error = %e, "SHIORI.request execution failed");
                Err(MyError::from(e))
            }
        }
    }

    /// Logs warning on error but does not propagate (safe for Drop).
    fn call_lua_unload(&self) {
        // unload_fn が在るのは VM が在るときだけ（release_runtime が VM より先に消す）
        let unload_fn = match &self.unload_fn {
            Some(f) => f,
            None => {
                debug!("SHIORI.unload not available, skipping");
                return;
            }
        };

        let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));

        if let Err(e) = unload_fn.call::<()>(()) {
            warn!(error = %e, "SHIORI.unload failed");
        } else {
            info!("SHIORI.unload called successfully");
        }
    }
}

#[cfg(test)]
#[path = "shiori_lifecycle_tests.rs"]
mod shiori_lifecycle_tests;

#[cfg(test)]
#[path = "shiori_request_tests.rs"]
mod shiori_request_tests;
