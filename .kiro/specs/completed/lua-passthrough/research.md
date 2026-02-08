# Research & Design Decisions

## Summary
- **Feature**: `lua-passthrough`
- **Discovery Scope**: Extension（既存Loaderシステムへの機能追加）
- **Key Findings**:
  - `CacheManager` の主要メソッド群（`source_to_module_name`, `source_to_cache_path`, `needs_transpile`, `generate_scene_dic`, `find_orphaned_caches`）は拡張子に依存しない汎用設計であり、`.lua` パススルーに対してそのまま再利用可能
  - `discovery::discover_files` は `pasta_patterns` 設定ベースの glob パターンで動作するため、`.lua` パターンの追加検出が必要
  - `PastaLoader::transpile_incremental` がファイル種別を判定せず全ファイルをパース/トランスパイルに送るため、`.lua` ファイルの分岐処理が主要変更点

## Research Log

### CacheManager の拡張子非依存性

- **Context**: `.lua` ファイルのキャッシュコピーに `CacheManager` 既存メソッドが再利用可能か調査
- **Sources Consulted**: `crates/pasta_lua/src/loader/cache.rs` 全体（702行）
- **Findings**:
  - `source_to_module_name`: `Path::with_extension("")` で拡張子除去 → `.lua` でも `.pasta` でも同じモジュール名生成
  - `source_to_cache_path`: `Path::with_extension("lua")` でキャッシュパス生成 → `.lua` ソースの場合もキャッシュは `.lua`
  - `needs_transpile`: ソース vs キャッシュのタイムスタンプ比較のみ → ファイル種別無関係
  - `save_cache`: UTF-8文字列をキャッシュパスに書き込む → コピー用途にも利用可能
  - `generate_scene_dic`: `module_names: &[String]` を受け取りソート・出力 → 由来を問わない
  - `find_orphaned_caches`: `source_paths` → `source_to_cache_path` マッピングで判定 → `.lua` ソースも含めれば動作
- **Implications**: CacheManager 自体の変更は不要。呼び出し側（`PastaLoader`）での分岐のみ

### discovery モジュールの検出パターン

- **Context**: `.lua` ファイル検出をどこに組み込むか調査
- **Sources Consulted**: `crates/pasta_lua/src/loader/discovery.rs`、`config.rs`
- **Findings**:
  - `discover_files(base_dir, patterns)` は汎用 glob 関数。`.lua` パターンを渡せばそのまま動作
  - `LoaderConfig::pasta_patterns` はデフォルト `["dic/*/*.pasta"]`。設定ファイルでカスタマイズ可能
  - `profile/` 除外ロジックは `is_in_profile_dir` で実装済み → `.lua` にも適用される
- **Implications**: `discover_files` に `.lua` パターンを追加渡しするか、別途呼び出しで `.lua` ファイルリストを取得

### 同名衝突のモジュール名衝突

- **Context**: `helper.pasta` と `helper.lua` が同じディレクトリに存在した場合の振る舞い
- **Sources Consulted**: `cache.rs` の `source_to_module_name` / `source_to_cache_path`
- **Findings**:
  - 両方とも `pasta.scene.<dir>.helper` を生成し、キャッシュパスも同一
  - 後から処理されたファイルが上書きする（非決定的）
- **Implications**: `.pasta` 優先ポリシーを採用。衝突検出→`.lua` スキップ＋警告

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存拡張 | `transpile_incremental` に分岐追加 | 最小変更、CacheManager再利用 | 単一関数の責務拡大 | 推奨 |
| B: 新コンポーネント | `lua_passthrough.rs` を新設 | 責務分離 | CacheManager重複、統合複雑化 | 過剰 |
| C: ハイブリッド | 分岐＋ヘルパー関数 | バランス | Option Aとほぼ同等 | 実質A |

## Design Decisions

### Decision: 既存コンポーネント拡張（Option A）を採用

- **Context**: `.lua` パススルーは既存Loaderパイプラインの小規模拡張
- **Alternatives Considered**:
  1. Option A — `transpile_incremental` に拡張子判定分岐を追加
  2. Option B — 新モジュール `lua_passthrough.rs` を作成
- **Selected Approach**: Option A（既存拡張）
- **Rationale**: CacheManager の既存メソッドがすべて拡張子非依存であり、変更箇所は `transpile_incremental` のループ内分岐と `discover_files` の `.lua` パターン追加のみ
- **Trade-offs**: `transpile_incremental` の責務が「トランスパイル＋コピー」に拡大するが、処理フローは線形かつ単純
- **Follow-up**: 関数名を `process_incremental` 等に改名するかは実装時に判断

### Decision: TranspileStats に `copied` フィールドを新設

- **Context**: 統計情報で `.lua` コピーを区別するか
- **Alternatives Considered**:
  1. `copied` フィールド新設
  2. `skipped` に含める
- **Selected Approach**: `copied` フィールド新設
- **Rationale**: `.lua` コピーは「スキップ」ではなく「処理した」結果。ログに `copied=3` と表示されれば開発者が `.lua` パススルーの動作を確認できる
- **Trade-offs**: 構造体に1フィールド追加のみ

### Decision: .lua パターンは pasta_patterns と別管理

- **Context**: `.lua` 検出パターンを既存の `pasta_patterns` に混ぜるか独立させるか
- **Alternatives Considered**:
  1. `pasta_patterns` に `"dic/*/*.lua"` を追加
  2. 別途 Loader 内でハードコード
- **Selected Approach**: Loader 内で `dic/*/*.lua` パターンをハードコード（`pasta_patterns` の設定値を元に `.pasta` → `.lua` に変換）
- **Rationale**: `pasta_patterns` はユーザーが `.pasta` 配置場所をカスタマイズするための設定。`.lua` は `.pasta` と同じディレクトリに置く「おまけ」機能であり、設定項目を増やす必要はない。`pasta_patterns` の各パターンから拡張子を `.lua` に変換して並行検出すれば、ユーザーのカスタムパターンにも自動追従する
- **Trade-offs**: 設定の柔軟性は若干低い（`.lua` だけ別ディレクトリに置けない）が、ユースケース的に不要

## Risks & Mitigations
- **衝突見逃し**: `.pasta` と `.lua` の衝突検出はファイル名ベースの単純比較で十分。パフォーマンスリスクなし
- **キャッシュ整合性**: `save_cache` を使うため、既存のタイムスタンプ・バージョン管理が自動的に適用される
- **テスト不足**: 既存テストパターンに `.lua` パススルーテストを追加する必要あり

## References
- `crates/pasta_lua/src/loader/mod.rs` — Loader メインエントリ
- `crates/pasta_lua/src/loader/cache.rs` — CacheManager 実装
- `crates/pasta_lua/src/loader/discovery.rs` — ファイル検出
- `crates/pasta_lua/src/loader/config.rs` — 設定管理
- `crates/pasta_lua/src/loader/error.rs` — エラー型
