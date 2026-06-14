//! Loader-side source-map build orchestration.
//!
//! Sibling submodule of [`super`] holding the multi-chunk `.pasta`↔`.lua`
//! [`SourceMap`] build responsibility split out of `loader/mod.rs` (C3).

use super::{CacheManager, PastaLoader};
use crate::debug::source_map::{MapBuilderSink, SourceMap};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::sync::Arc;
use tracing::{debug, warn};

impl PastaLoader {
    /// Build and aggregate the multi-chunk `.pasta`↔`.lua` [`SourceMap`] for the
    /// discovered `.pasta` files (task 4.3 — `SourceMapBuilder`).
    ///
    /// This is the loader-side source-map build orchestration (design "Flow 1",
    /// File Structure Plan `loader/mod.rs` line 166). For EACH `.pasta` file it:
    ///
    /// 1. Re-parses the source and creates a [`MapBuilderSink::new(pasta_file,
    ///    chunk_name)`], where `chunk_name` is derived from the SAME loader cache
    ///    path construction the runtime require/hook path uses
    ///    ([`CacheManager::source_to_cache_path`] — task 1.1's confirmed naming
    ///    strategy; `set_name` is NOT needed because the hook source and this key
    ///    match after [`crate::debug::source_map::canonicalize_chunk_name`]).
    /// 2. Transpiles with the sink attached via
    ///    [`LuaTranspiler::transpile_with_source_map`], which records pre-normalize
    ///    `(.lua line -> .pasta span)` correspondences (Requirement 1.1) and returns
    ///    the normalize [`crate::normalize::LineShift`].
    /// 3. Calls `sink.finish(&shift)` to rebase the records onto the final `.lua`
    ///    line numbers (Requirement 2.1), yielding a per-chunk
    ///    [`crate::debug::source_map::ChunkSourceMap`].
    /// 4. Inserts the chunk into the aggregate [`SourceMap`] keyed by chunk name
    ///    ([`SourceMap::insert_chunk`] canonicalizes the key internally — task 3.4).
    ///
    /// The result is held as an `Arc<SourceMap>` (immutable shared reference, design
    /// "Architecture" `Arc<SourceMap>` 不変共有 — Requirement 3.1, in-memory, no
    /// mandatory intermediate file). Task 4.4 threads this `Arc` into
    /// [`crate::debug::enable`].
    ///
    /// # Debug gating (Requirements 3.1, 7.1)
    ///
    /// This is invoked by the loader ONLY when debugging is enabled. On the disabled
    /// (default) path the loader does NOT call this, so no sink is attached during
    /// transpilation, no [`SourceMap`] is allocated, and the generated `.lua` bytes
    /// stay byte-invariant (the production transpile in
    /// [`process_incremental`](Self::process_incremental) uses the sink-less
    /// [`LuaTranspiler::transpile`]).
    ///
    /// Files that fail to read/parse/transpile are skipped with a warning (the map
    /// is best-effort and non-fatal; the primary transpile path in
    /// `process_incremental` is the one that surfaces hard failures).
    ///
    /// # Optional disk sidecar (task 6.1 — Requirement 3.2)
    ///
    /// When `sidecar` is `true` (the resolved `source_map_sidecar` flag — task 4.1,
    /// `env > file > 既定 false`), each per-chunk [`ChunkSourceMap`] is ALSO written
    /// to a `<generated.lua>.map` sidecar next to its generated `.lua`
    /// ([`crate::debug::source_map::write_sidecar`]). The sidecar write is
    /// NON-FATAL (design Error 611, 616 / Requirement 3.1): an I/O failure is logged
    /// via `tracing::warn!` and the in-memory `SourceMap` build continues unaffected.
    /// When `sidecar` is `false` (default) no `.map` file is written and the
    /// in-memory map is the sole path.
    pub fn build_source_map(
        pasta_files: &[std::path::PathBuf],
        cache_manager: &CacheManager,
        sidecar: bool,
    ) -> Arc<SourceMap> {
        Arc::new(build_source_map_inner(pasta_files, cache_manager, sidecar))
    }
}

/// Core multi-chunk source-map build.
///
/// See [`PastaLoader::build_source_map`] for the contract. Kept as a private free
/// function so the build logic stays separate from the `Arc` wrapping; tests
/// exercise it through [`PastaLoader::build_source_map`].
fn build_source_map_inner(
    pasta_files: &[std::path::PathBuf],
    cache_manager: &CacheManager,
    sidecar: bool,
) -> SourceMap {
    let transpiler = LuaTranspiler::default();
    let mut source_map = SourceMap::new();
    let mut chunk_count = 0usize;

    for file_path in pasta_files {
        // Read the .pasta source.
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (read error)");
                continue;
            }
        };

        let filename = file_path.to_string_lossy().to_string();
        let pasta_file = match pasta_dsl::parse_str(&content, &filename) {
            Ok(pf) => pf,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (parse error)");
                continue;
            }
        };

        // Chunk name = SAME loader cache path construction the runtime require/hook
        // path reports (task 1.1). `insert_chunk` canonicalizes this internally so
        // it matches the `@<absolute .lua path>` hook source after normalization.
        let chunk_name = cache_manager
            .source_to_cache_path(file_path)
            .to_string_lossy()
            .to_string();

        // Attach the recording sink and transpile, capturing the normalize shift.
        let mut sink = MapBuilderSink::new(filename.clone(), chunk_name.clone());
        let mut output = Vec::new();
        let shift = match transpiler.transpile_with_source_map(
            &pasta_file,
            &mut output,
            Some(&mut sink),
        ) {
            Ok((_ctx, shift)) => shift,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (transpile error)");
                continue;
            }
        };

        // Rebase pre-normalize records onto final `.lua` lines (2.1) and aggregate
        // the per-chunk map by chunk name (3.4).
        let chunk_map = sink.finish(&shift);

        // Optional disk sidecar (task 6.1 / Requirement 3.2): when enabled, write a
        // `<generated.lua>.map` next to the generated `.lua` (the same cache path the
        // chunk name was derived from). NON-FATAL (design Error 611, 616 / 3.1): an
        // I/O failure is logged and the in-memory map build continues unchanged.
        if sidecar {
            let lua_path = cache_manager.source_to_cache_path(file_path);
            if let Err(e) =
                crate::debug::source_map::write_sidecar(&lua_path, &filename, &chunk_map)
            {
                warn!(
                    file = %lua_path.display(),
                    error = %e,
                    "source-map: sidecar write failed; continuing with in-memory map (non-fatal, 3.2/3.1)"
                );
            } else {
                debug!(file = %lua_path.display(), "source-map: wrote sidecar (.lua.map)");
            }
        }

        source_map.insert_chunk(chunk_name, filename, chunk_map);
        chunk_count += 1;
    }

    debug!(
        chunks = chunk_count,
        files = pasta_files.len(),
        "source-map: built aggregate map"
    );
    source_map
}
