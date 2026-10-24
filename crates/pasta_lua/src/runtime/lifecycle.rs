//! Drop-time lifecycle and persistence save for PastaLuaRuntime.
//!
//! This file contains the `impl PastaLuaRuntime` persistence-save method and
//! the `impl Drop` that invokes it on teardown (split impl pattern).

use super::PastaLuaRuntime;
use super::persistence;
use mlua::Table;
use std::path::Path;

impl PastaLuaRuntime {
    /// Save persistence data from the `pasta.save` table (`require("pasta.save")`).
    ///
    /// Called automatically on Drop to save any modified persistent data.
    fn save_persistence_data(&self) -> Result<(), persistence::PersistenceError> {
        // Get persistence config
        let persistence_config = self
            .config
            .as_ref()
            .and_then(|c| c.persistence())
            .unwrap_or_default();

        let base_dir = self.base_dir.as_deref().unwrap_or(Path::new("."));
        let file_path = base_dir.join(persistence_config.effective_file_path());

        // Try to get save from Lua
        // SAFETY(injection): Module name is a compile-time string literal.
        // The match handles the error case gracefully (early return Ok).
        let save_table: Table = match self.lua.load(r#"require("pasta.save")"#).eval() {
            Ok(t) => t,
            Err(e) => {
                // save might not exist if runtime wasn't fully initialized
                tracing::debug!(error = %e, "Could not access pasta.save, skipping persistence save");
                return Ok(());
            }
        };

        // save(data) と同じ共通の道で保存する
        persistence::save_table(
            &self.lua,
            &save_table,
            &file_path,
            persistence_config.obfuscate,
        )?;

        if persistence_config.debug_mode {
            tracing::debug!(path = %file_path.display(), "Saved persistence data on drop");
        }

        Ok(())
    }
}

impl Drop for PastaLuaRuntime {
    fn drop(&mut self) {
        // Save persistence data (errors are logged, not propagated)
        if let Err(e) = self.save_persistence_data() {
            tracing::error!(error = %e, "Failed to save persistence data on drop");
        }
    }
}
