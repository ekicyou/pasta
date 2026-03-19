# Research & Design Decisions: local-scene-act-call

## Summary
- **Feature**: `local-scene-act-call`
- **Discovery Scope**: Extension（既存トランスパイラ・ランタイムバグ修正）
- **Key Findings**:
  - 修正対象は2関数の各1行のみ（`scope_gen.rs`、`context.rs`）
  - スナップショットテスト2ファイル（8箇所）の `__Name_N__` → `Name_N` 更新が必要
  - E2Eテストインフラ（`e2e_helpers.rs`）は既に整備済み。`create_runtime_with_finalize()` + `transpile()` パターンを再利用可能

## Research Log

### トランスパイラ出力フォーマットの調査
- **Context**: `scope_gen.rs` が生成する Lua 関数名のフォーマットを確認
- **Sources Consulted**: `crates/pasta_lua/src/code_gen/scope_gen.rs` L192-240
- **Findings**:
  - `__start__` は `scene.name == None` のときに生成。専用の条件分岐で保護済み
  - 名前ありローカルシーン（`scene.name == Some(...)`）は `format!("__{}_{}__", sanitized, counter)` で生成
  - `sanitized` は `SceneRegistry::sanitize_name()` の戻り値（Unicode対応済み）
  - `counter` は同名シーンの1始まり連番
- **Implications**: `format!("{}_{}",  sanitized, counter)` に変更するだけで、`__start__` に影響なく修正可能

### `parse_fn_name` の動作確認
- **Context**: `fn_name` → Lua関数名変換の現行ロジック確認
- **Sources Consulted**: `crates/pasta_lua/src/search/context.rs` L120-140
- **Findings**:
  - `fn_name` を `::` で分割し、`global_part` と `local_part` を抽出
  - `local_part == "__start__"` の場合はそのまま返す
  - それ以外は `format!("__{}__", local_part)` で再ラッピング
  - 修正後は `local_part` をそのまま返せばよい（Luaテーブルキーと一致するため）
- **Implications**: `__start__` の条件分岐はそのままで、else節を `local_part.to_string()` に変更

### スナップショットテスト影響範囲
- **Context**: `__Name_N__` パターンのスナップショットファイル数を確認
- **Sources Consulted**: `crates/pasta_lua/tests/transpiler/snapshots/` 全ファイル
- **Findings**:
  - **影響ファイル2つ**:
    - `transpiler__snapshot_test__tail_call_optimization.snap`: 8箇所の `SCENE.__Name_N__`
    - `transpiler__snapshot_test__scene_with_call.snap`: 1箇所の `SCENE.__サブ_1__`
  - `SCENE.__start__` は変更不要（9箇所、全て `__start__` で保護対象）
  - `SCENE.__global_name__` も変更不要（メタフィールド、ローカルシーン名ではない）
- **Implications**: `cargo test` → スナップショット失敗 → `cargo insta review` で一括更新

### E2Eテストインフラの調査
- **Context**: ローカルシーンcallのE2Eテスト追加に使えるインフラ確認
- **Sources Consulted**: `crates/pasta_lua/tests/common/e2e_helpers.rs`、`finalize_scene_test.rs`
- **Findings**:
  - `create_runtime_with_finalize()`: Lua VM + `@pasta_search`/`@pasta_persistence`/`@pasta_log` + `finalize_scene` バインディング登録済み
  - `transpile()`: Pasta DSL文字列 → Luaコード文字列変換
  - 既存テスト `test_scene_collection_local_scenes` はローカルシーン登録の確認のみ（call実行なし）
  - `act:call` のテストには `act.lua` のロードが必要だが、現在の `create_runtime_with_finalize()` は `pasta` モジュールの `require` で `act.lua` を自動ロード可能
- **Implications**: 既存インフラを拡張して `act:call` でローカルシーン解決・実行をテスト可能

### `register_global_raw` の名前依存調査
- **Context**: `register_global_raw` が `__Name_N__` に依存しているか確認
- **Sources Consulted**: `crates/pasta_core/src/registry/scene_registry.rs` L156-210
- **Findings**:
  - `local_names` 引数をそのまま `fn_name` に組み込む（`format!("{}::{}", full_name, local_name)`）
  - `__start__` のスキップには `local_name != "__start__"` を使用
  - `name` フィールドには `local_name.clone()` をそのまま格納
  - いずれも `__Name_N__` 固有の前提なし。`Name_N` が渡されれば正常に動作
- **Implications**: `register_global_raw` の修正は不要（Option E の設計通り）

## Design Decisions

### Decision: Option E — トランスパイラの `__` ラッピング除去

- **Context**: ローカルシーン名の `__` ラッピングが全経路でフォーマット不整合を起こしている
- **Alternatives Considered**:
  1. Option A — `register_global_raw` でアンマングル
  2. Option B — `fn_name_to_search_key` でアンマングル
  3. Option C — `collect_scenes`/`build_scene_registry` でアンマングル
  4. Option D — Level 1 にローカルシーン逆引き追加
  5. **Option E — トランスパイラの `__` ラッピング除去**
- **Selected Approach**: Option E
- **Rationale**: 根本原因（発生源）で直接修正。下流コンポーネントの変更不要。Level 1/Level 2 同時解決
- **Trade-offs**: スナップショットテスト更新が必要（2ファイル、`insta review` で一括処理）
- **Follow-up**: E2Eインテグレーションテストで `act:call` ローカルシーン解決を検証

## Risks & Mitigations
- **スナップショットテスト大量失敗** — `cargo insta review` で一括更新。影響は2ファイルのみ
- **既存ゴーストの再トランスパイル必要性** — pasta.dll は起動時に毎回トランスパイルするため、キャッシュ無効化のみで対応可能（キャッシュはgzip圧縮Luaコードであり、ソース変更検出で自動再生成）

## References
- gap-analysis.md — Option A〜E の詳細比較
- `crates/pasta_lua/src/code_gen/scope_gen.rs` L198-202 — 根本原因箇所
- `crates/pasta_lua/src/search/context.rs` L120-140 — `parse_fn_name` 修正箇所
- `crates/pasta_lua/tests/common/e2e_helpers.rs` — E2Eテストインフラ
