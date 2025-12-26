//! Parse result caching for performance optimization.
//!
//! This module provides caching of parsed AST and transpiled Rune code
//! to avoid re-parsing the same script multiple times.

use pasta_core::parser::ast::PastaFile;
use std::collections::HashMap;

/// A cache entry containing parsed AST and transpiled Rune code.
struct CacheEntry {
    /// The parsed AST.
    ast: PastaFile,
    /// The transpiled Rune source code.
    rune_source: String,
}

/// Instance-local cache for parse results.
///
/// This cache stores parsed AST and transpiled Rune code keyed by script content hash.
/// Each PastaEngine instance owns its own cache.
pub struct ParseCache {
    entries: HashMap<u64, CacheEntry>,
}

impl ParseCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Get a cached entry if it exists.
    ///
    /// # Arguments
    ///
    /// * `script` - The script source code
    ///
    /// # Returns
    ///
    /// An option containing cloned copies of the cached AST and Rune source if found.
    pub fn get(&self, script: &str) -> Option<(PastaFile, String)> {
        let hash = Self::hash_script(script);
        let entry = self.entries.get(&hash)?;
        Some((entry.ast.clone(), entry.rune_source.clone()))
    }

    /// Store a parse result in the cache.
    ///
    /// # Arguments
    ///
    /// * `script` - The script source code
    /// * `ast` - The parsed AST
    /// * `rune_source` - The transpiled Rune source code
    pub fn insert(&mut self, script: &str, ast: PastaFile, rune_source: String) {
        let hash = Self::hash_script(script);
        let entry = CacheEntry { ast, rune_source };
        self.entries.insert(hash, entry);
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Compute a hash of the script content.
    ///
    /// Uses a simple FNV-1a hash for fast hashing.
    fn hash_script(script: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in script.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new()
    }
}
