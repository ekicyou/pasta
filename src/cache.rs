//! Parse result caching for performance optimization.
//!
//! This module provides caching of parsed AST and transpiled Rune code
//! to avoid re-parsing the same script multiple times.

use crate::PastaFile;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;
    use crate::transpiler::Transpiler;

    #[test]
    fn test_cache_empty() {
        let cache = ParseCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ParseCache::new();
        let script = r#"
＊test
    さくら：こんにちは
"#;

        // Parse and transpile
        let ast = parse_str(script, "<test>").unwrap();
        let rune_source = Transpiler::transpile(&ast).unwrap();

        // Insert into cache
        cache.insert(script, ast.clone(), rune_source.clone());

        // Retrieve from cache
        let cached = cache.get(script);
        assert!(cached.is_some());

        let (cached_ast, cached_rune) = cached.unwrap();
        assert_eq!(cached_ast.scenes.len(), ast.scenes.len());
        assert_eq!(cached_rune, rune_source);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ParseCache::new();
        let script1 = r#"
＊test1
    さくら：こんにちは
"#;
        let script2 = r#"
＊test2
    さくら：こんばんは
"#;

        let ast1 = parse_str(script1, "<test1>").unwrap();
        let rune_source1 = Transpiler::transpile(&ast1).unwrap();

        cache.insert(script1, ast1, rune_source1);

        // Try to get a different script
        let cached = cache.get(script2);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ParseCache::new();
        let script = r#"
＊test
    さくら：こんにちは
"#;

        let ast = parse_str(script, "<test>").unwrap();
        let rune_source = Transpiler::transpile(&ast).unwrap();

        cache.insert(script, ast, rune_source);
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_multiple_entries() {
        let mut cache = ParseCache::new();
        let scripts = vec![
            r#"
＊test1
    さくら：こんにちは
"#,
            r#"
＊test2
    さくら：こんばんは
"#,
            r#"
＊test3
    さくら：おはよう
"#,
        ];

        for script in &scripts {
            let ast = parse_str(script, "<test>").unwrap();
            let rune_source = Transpiler::transpile(&ast).unwrap();
            cache.insert(script, ast, rune_source);
        }

        assert_eq!(cache.len(), 3);

        // Verify all entries can be retrieved
        for script in &scripts {
            let cached = cache.get(script);
            assert!(cached.is_some());
        }
    }

    #[test]
    fn test_hash_consistency() {
        let script = r#"
＊test
    さくら：こんにちは
"#;
        let hash1 = ParseCache::hash_script(script);
        let hash2 = ParseCache::hash_script(script);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_difference() {
        let script1 = r#"
＊test1
    さくら：こんにちは
"#;
        let script2 = r#"
＊test2
    さくら：こんばんは
"#;
        let hash1 = ParseCache::hash_script(script1);
        let hash2 = ParseCache::hash_script(script2);
        assert_ne!(hash1, hash2);
    }
}
