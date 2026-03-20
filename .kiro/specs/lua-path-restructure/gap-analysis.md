# ギャップ分析: lua-path-restructure

## 分析概要

- **スコープ**: Lua検索パスの再定義（`user_scripts`廃止、`scripts`→ユーザー用、`pasta_scripts`→標準ランタイム）、ディレクトリリネーム、テスト・ビルド・ドキュメントの網羅修正
- **複雑度**: 変更自体は単純（文字列置換とディレクトリ移動）だが、影響範囲が広く修正漏れのリスクが高い
- **推奨アプローチ**: Option A（既存コンポーネント拡張）— 新規ファイル作成なし、全変更が既存ファイルの修正
- **リスク**: 低（確立されたパターンの文字列/パス修正のみ）
- **工数見積**: S（1-3日）

---

## 1. 現行状態の調査結果

### 1.1 デフォルト検索パス定義

**ファイル**: `crates/pasta_lua/src/loader/config.rs` L166-173

```rust
pub fn default_lua_search_paths() -> Vec<String> {
    vec![
        "profile/pasta/save/lua".to_string(),
        "user_scripts".to_string(),     // ← 削除対象
        "scripts".to_string(),          // ← "pasta_scripts" に変更
        "profile/pasta/cache/lua".to_string(),
        "scriptlibs".to_string(),
    ]
}
```

**変更後**:
```rust
pub fn default_lua_search_paths() -> Vec<String> {
    vec![
        "profile/pasta/save/lua".to_string(),
        "scripts".to_string(),          // ユーザー作成スクリプト
        "pasta_scripts".to_string(),    // pasta 標準ランタイム
        "profile/pasta/cache/lua".to_string(),
        "scriptlibs".to_string(),
    ]
}
```

### 1.2 物理ディレクトリ構造

**移動対象**: `crates/pasta_lua/scripts/` → `crates/pasta_lua/pasta_scripts/`

現行の `scripts/` 内容:
```
scripts/
├── ct.lua           # キャンセルトークン
├── hello.lua        # サンプルスクリプト
├── main.lua         # エントリーポイント（user_scripts参照あり）
├── README.md        # ドキュメント
└── pasta/           # pasta標準ランタイム
    ├── act.lua
    ├── actor.lua
    ├── config.lua
    ├── global.lua
    ├── init.lua
    ├── save.lua
    ├── scene.lua
    ├── store.lua
    ├── word.lua
    ├── areka/
    │   └── init.lua
    └── shiori/
        ├── act.lua
        ├── entry.lua
        ├── init.lua
        ├── res.lua
        ├── sakura_builder.lua
        └── event/
            ├── boot.lua
            ├── init.lua
            ├── register.lua
            ├── second_change.lua
            └── virtual_dispatcher.lua
```

---

## 2. 要件-アセットマップ（影響箇所の網羅一覧）

### Req 1: デフォルト検索パスの再定義

| 修正対象 | ファイル | 行 | ギャップ |
|---|---|---|---|
| デフォルトパス定義 | `crates/pasta_lua/src/loader/config.rs` | L166-173 | `user_scripts`削除、`scripts`→`pasta_scripts`に変更 |

### Req 2: ディレクトリ移動

| 修正対象 | 操作 | ギャップ |
|---|---|---|
| `crates/pasta_lua/scripts/` | ディレクトリ名変更 → `pasta_scripts/` | Missing |
| `scripts/main.lua` 内の `user_scripts` 参照 | コメント文修正 | `user_scripts`→`scripts`への案内に変更 |
| `scripts/README.md` | 新規作成（旧READMEは削除） | `pasta_scripts/` 用のREADME（編集禁止の案内）に差し替え |
| `scripts/` (ユーザー用) | 新規 README.md 配置 | ユーザーカスタム用フォルダーの説明 |
| `scripts/hello.lua` | 削除 | ランタイム不要のサンプルファイル |
| `tests/lua_specs/transpiler_test.lua` | 削除 | `require("hello")` のみを検証。hello.lua 削除に伴い不要 |
| `tests/lua_specs/init.lua` | `"transpiler_test"` エントリ削除 | 上記テスト削除に伴う整合 |
| `.vscode/launch.json` | `"Lua (pasta_lua scripts)"` エントリ削除 | `hello.lua` を `program` とするデバッグ設定。hello.lua 削除に伴い不要 |

### Req 3: hello-pasta サンプルゴースト

| 修正対象 | ファイル | ギャップ |
|---|---|---|
| 設定ファイル(dist-src) | `crates/pasta_sample_ghost/dist-src/ghost/master/pasta.toml` | `user_scripts`→`scripts`、`scripts`→`pasta_scripts` |
| 設定ファイル(生成済み) | `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` | 同上 |
| リリーススクリプト | `crates/pasta_sample_ghost/release.ps1` L153 | `$ScriptsDest` を `"pasta_scripts"` に変更、コピー元も `pasta_scripts` に |
| リリーススクリプト | `crates/pasta_sample_ghost/release.ps1` L129, 175 | コメント文の`scripts/`を`pasta_scripts/`に更新 |
| 生成済みLuaスクリプト群 | `ghosts/hello-pasta/ghost/master/scripts/` | ディレクトリ名を `pasta_scripts/` に変更（リリース再生成で解決） |
| main.lua(配布物内) | `ghosts/hello-pasta/ghost/master/scripts/main.lua` | `user_scripts`→`scripts`への案内コメント修正（リリース再生成で解決）|
| README.md(配布物内) | `ghosts/hello-pasta/ghost/master/scripts/README.md` | パス参照更新（リリース再生成で解決） |
| 更新ファイル | `ghosts/hello-pasta/updates2.dau` | 全行 `ghost/master/scripts/`→`ghost/master/pasta_scripts/` （リリース再生成で解決）|
| 更新ファイル | `ghosts/hello-pasta/updates.txt` | 同上（リリース再生成で解決） |
| ソースコード | `crates/pasta_sample_ghost/src/main.rs` L30, 84 | `scripts/`参照コメント更新 |
| ソースコード | `crates/pasta_sample_ghost/src/lib.rs` L95 | `scripts/`参照コメント更新 |
| ログファイル | `ghosts/hello-pasta/ghost/master/profile/pasta/logs/pasta.log` | 古いパスが記録されているが、ログなので変更不要（リリース再生成後に自然更新） |

### Req 4: テストの整合性

| 修正対象 | ファイル | 行 | ギャップ |
|---|---|---|---|
| テスト共通ヘルパー | `crates/pasta_lua/tests/common/mod.rs` | L25, 119-120, 151-152 | `.join("scripts")` → `.join("pasta_scripts")` |
| E2Eヘルパー | `crates/pasta_lua/tests/common/e2e_helpers.rs` | L46, 105 | `.join("scripts")` → `.join("pasta_scripts")` |
| ローダーstartupテスト | `crates/pasta_lua/tests/loader/startup_test.rs` | L187, 228-229 | `scripts` 参照の更新 |
| ローダーconfig設定テスト | `crates/pasta_lua/tests/loader/config_test.rs` | L20, 386, 399 | デフォルトパスのアサーション更新（`user_scripts`削除、`pasta_scripts`追加） |
| ローダーlifecycleテスト | `crates/pasta_lua/tests/loader/lifecycle_test.rs` | L70-145 | `user_scripts`→`scripts`、`scripts`→`pasta_scripts` に変更（`PastaLoader::load()` がデフォルトパスを使用するテスト） |
| ローダーluaパススルーテスト | `crates/pasta_lua/tests/loader/lua_passthrough_test.rs` | L18 | `"scripts"` の参照確認 |
| ランタイムfinalize_sceneテスト | `crates/pasta_lua/tests/runtime/finalize_scene_test.rs` | L482 | `.join("scripts")` 更新 |
| ランタイムencodingテスト | `crates/pasta_lua/tests/runtime/encoding_test.rs` | L17, 192, 226, 259 | `"scripts"` 参照の更新 |
| transpiler/fallback検索テスト | `crates/pasta_lua/tests/transpiler/fallback_search_integration_test.rs` | L196 | `.join("scripts")` 更新 |
| ソースコード内テスト | `crates/pasta_lua/src/loader/context.rs` | L147, 152, 160, 177, 217, 231, 246 | 変更不要（任意のディレクトリ名を渡す汎用テストであり、デフォルト値のテストではない） |
| pasta_sample_ghostテスト共通 | `crates/pasta_sample_ghost/tests/common/mod.rs` | L49, 62, 71 | `.join("scripts")` → `.join("pasta_scripts")`, コメント更新 |
| pasta_sample_ghost統合テスト | `crates/pasta_sample_ghost/tests/integration_test.rs` | L121-128 | `user_scripts`アサーション削除、`pasta_scripts`アサーション追加 |

### Req 5: ドキュメント・ステアリング

| 修正対象 | ファイル | ギャップ |
|---|---|---|
| プロジェクト構造 | `.kiro/steering/structure.md` | `scripts/` → `pasta_scripts/` のディレクトリ記述更新 |
| pasta_lua README | `crates/pasta_lua/README.md` | L64, 102-104, 118-122, 268-287, 401-404 のパス参照更新 |
| pasta_sample_ghost README | `crates/pasta_sample_ghost/README.md` | L75, 99, 153 の `scripts/` 参照更新 |
| テストカバレッジ | `TEST_COVERAGE.md` | L88 `user_scripts` テスト名の更新 |
| AIスキル(pasta-lua-coding) | `.agents/skills/pasta-lua-coding/SKILL.md` | L5, 7, 29, 32, 34, 36 の `scripts/` 参照更新 |
| AIスキル(pasta-lua-coding/refs) | `.agents/skills/pasta-lua-coding/references/runtime-api.md` | L4, 496 の `scripts/` 参照更新 |
| AIスキル(pasta-lua-coding/refs) | `.agents/skills/pasta-lua-coding/references/testing-lint.md` | L220, 223, 226 の luacheck パス更新 |

---

## 3. 実装アプローチの評価

### Option A: 既存コンポーネント拡張（推奨）

**理由**: 全変更が既存ファイルの修正であり、新規ファイル作成は不要

**手順**:
1. `crates/pasta_lua/scripts/` → `crates/pasta_lua/pasta_scripts/` のリネーム（git mv）
2. `default_lua_search_paths()` の修正
3. `pasta_scripts/main.lua` 内の `user_scripts` コメントを `scripts` に更新
4. `pasta_scripts/README.md` のパス記述更新
5. テストコードの修正（上記一覧の全箇所）
6. hello-pasta 設定ファイル（pasta.toml × 2箇所）の更新
7. release.ps1 のパス更新
8. ソースコードコメントの更新（main.rs, lib.rs）
9. ドキュメント・ステアリングの更新
10. hello-pasta 配布物の再生成（`release.ps1` 実行で updates2.dau, updates.txt 等が自動更新）
11. `cargo test --all` で全パス確認

**Trade-offs**:
- ✅ 新規ファイルなし、既存パターンの修正のみ
- ✅ git mv でリネーム履歴が保持される
- ❌ 修正箇所が多い（約30ファイル）ため漏れ注意

### Option B/C: 該当なし

既存コンポーネントの改名変更であり、新規コンポーネント作成やハイブリッドアプローチの選択肢は存在しない。

---

## 4. 工数・リスク評価

| 項目 | 評価 | 根拠 |
|---|---|---|
| **工数** | **S（1-3日）** | 変更は全てパス文字列の置換とディレクトリ移動。ロジック変更なし |
| **リスク** | **低** | 既存パターンの踏襲、文字列置換のみ、全テスト通過で検証可能 |

### 注意事項

- `context.rs` 内テストの `"scripts"` は任意のディレクトリ名を渡す汎用テストであり、変更不要（検証済み）
- `lifecycle_test.rs` のテストは `PastaLoader::load()` を使用しデフォルトパスに依存するため、`user_scripts`→`scripts`、`scripts`→`pasta_scripts` への更新が必須（検証済み）
- hello-pasta の `ghosts/` 配下の生成済みファイル群（配布物）はリリース再生成で自動解決

---

## 5. 持ち越し事項

全項目検証済み — 持ち越しなし。

| 項目 | 結果 | 根拠 |
|---|---|---|
| context.rs テストの扱い | **変更不要** | 汎用テストパスとして任意のディレクトリ名を渡すテスト。デフォルト値のテストではない |
| lifecycle_test.rs の扱い | **変更必須** | `PastaLoader::load()` がデフォルトパスを使用するテスト。`user_scripts`→`scripts`、`scripts`→`pasta_scripts` に更新 |
| 配布物再生成の手順 | **タスクに含める** | release.ps1 修正後の配布物再生成は実装の自然なステップ |
| .gitignore の確認 | **問題なし** | .gitignore に `scripts`/`pasta_scripts` 関連のパターンなし。確認済み |
