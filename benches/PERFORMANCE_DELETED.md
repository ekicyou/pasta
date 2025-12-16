# Performance Benchmark - Deleted

**削除日**: 2025-12-13
**削除理由**:

1. **存在しないAPI機能を使用**
   - `PastaEngine::clear_cache()` - P1機能（未実装）
   - `engine.cache_size()` - P1機能（未実装）

2. **旧API使用**
   - `PastaEngine::new(&script)` → 現在は `PastaEngine::new(&script_dir, &persistence_dir)`

3. **パフォーマンス要件の変化**
   - 2パストランスパイラー + Runeコンパイルの新アーキテクチャ
   - パフォーマンス要件の再定義が必要

4. **代替実装**
   - 核心機能テスト: test_comprehensive_control_flow_transpile
   - 実運用でのパフォーマンス測定

**将来の対応**:
- P1実装時にキャッシュ機能と共に再実装
- 新アーキテクチャに合わせたベンチマーク設計

元ファイル: performance.rs.disabled
