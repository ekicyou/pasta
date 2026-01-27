# Gap Analysis: shiori-entry

## 概要

**機能名**: `shiori-entry` - SHIORI/3.0 プロトコル Lua エントリーポイント

**分析スコープ**: 既存の `pasta.shiori.main` を置き換える新しいエントリーポイントモジュール `pasta.shiori.entry` の実装ギャップを分析する。

**主な発見**:
- `pasta.shiori.main` は現在ミニマル実装（load: 常にtrue、request: 204 No Content）
- `pasta.shiori.event` モジュールは完全実装済み（EVENT.fire）
- Rust側は `scripts/pasta/shiori/main.lua` をハードコード読み込み
- テストフィクスチャが3箇所（pasta_shiori/tests）に存在
- main.lua の require 変更が不要な可能性が高い（グローバルSHIORIテーブルベース）

---

## 1. Current State Investigation

### 1.1 既存アセット

#### `pasta.shiori.main` モジュール（minimal実装）

**場所**: `crates/pasta_lua/scripts/pasta/shiori/main.lua`

**現在の実装**:
```lua
SHIORI = SHIORI or {}

function SHIORI.load(hinst, load_dir)
    -- Minimal implementation: always succeed
    return true
end

function SHIORI.request(request_text)
    -- Minimal implementation: return 204 No Content
    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Sender: Pasta\r\n" ..
        "\r\n"
end

return SHIORI
```

**特徴**:
- グローバル `SHIORI` テーブルを初期化
- `SHIORI.load` は常に `true` を返す（何もしない）
- `SHIORI.request` は固定の 204 レスポンスを返す（**EVENT.fire を呼んでいない**）
- `SHIORI.unload` は未実装

**問題点**:
- イベント振り分けが機能していない（EVENT モジュールを利用していない）
- 要件定義に記載された「EVENT.fire への委譲」が実装されていない

#### `pasta.shiori.event` モジュール（完成済み）

**場所**: `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`

**実装状況**:
- ✅ `EVENT.fire(req)` - イベント振り分け実装済み
- ✅ `EVENT.no_entry(req)` - デフォルトハンドラ（204 No Content）
- ✅ エラーハンドリング（xpcall）
- ✅ REG モジュール統合（`pasta.shiori.event.register`）
- ✅ デフォルトイベントハンドラ（`boot.lua`）

**シグネチャ**:
```lua
function EVENT.fire(req) -> string
```

**reqテーブル構造**（Rust側 `lua_request.rs` でパース済み）:
```lua
{
    id = "OnBoot",
    method = "get",
    version = 30,
    charset = "UTF-8",
    sender = "SSP",
    reference = { [0] = "...", [1] = "..." },
    dic = { ["ID"] = "OnBoot", ... },
}
```

**依存関係**:
- `pasta.shiori.event.register` - ハンドラレジストリ
- `pasta.shiori.res` - レスポンス構築

#### `pasta.shiori.res` モジュール（完成済み）

**場所**: `crates/pasta_lua/scripts/pasta/shiori/res.lua`

**提供関数**:
- `RES.ok(script)` - 200 OK レスポンス
- `RES.no_content()` - 204 No Content レスポンス
- `RES.err(message)` - 500 Internal Server Error レスポンス

### 1.2 Rust側の統合ポイント

#### `PastaLuaRuntime::from_loader_with_scene_dic`

**場所**: `crates/pasta_lua/src/runtime/mod.rs:338-400`

**main.lua 読み込み処理**:
```rust
let main_lua_path = loader_context
    .base_dir
    .join("scripts/pasta/shiori/main.lua");
if main_lua_path.exists() {
    match std::fs::read_to_string(&main_lua_path) {
        Ok(script) => {
            runtime.lua.load(&script).set_name("main.lua").exec()?;
            tracing::debug!("Loaded main.lua");
        }
        // ...
    }
}
```

**重要な発見**:
- ✅ **ファイル名がハードコード**: `scripts/pasta/shiori/main.lua`
- ✅ 読み込み失敗時は warn ログのみ（継続可能）
- ✅ グローバル `SHIORI` テーブルの存在チェックなし（単純なrequire）

**変更の必要性**: 
- `main.lua` → `entry.lua` にファイル名変更が**必須**
- または既存の `main.lua` を `entry.lua` の薄いラッパーに変更

#### `PastaShiori::cache_lua_functions`

**場所**: `crates/pasta_shiori/src/shiori.rs:160-210`

**SHIORI関数キャッシュロジック**:
```rust
let shiori_table: Result<Table, _> = globals.get("SHIORI");
match shiori_table {
    Ok(table) => {
        self.load_fn = table.get::<Function>("load").ok();
        self.request_fn = table.get::<Function>("request").ok();
        self.unload_fn = table.get::<Function>("unload").ok();
    }
}
```

**重要な発見**:
- ✅ グローバル `SHIORI` テーブルから関数を取得
- ✅ モジュール名には依存しない（グローバル変数ベース）
- ✅ `SHIORI.unload` は optional（None でも動作可能）

**変更の必要性**: 
- **不要** - グローバル `SHIORI` テーブルが正しく初期化されれば動作する

### 1.3 テストフィクスチャ

#### 3箇所のテストフィクスチャ

1. **`crates/pasta_shiori/tests/support/scripts/pasta/shiori/main.lua`**
   - テストヘルパー用の main.lua（グローバル設定値を記録）
   
2. **`crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua`**
   - ライフサイクルテスト用フィクスチャ（request_count カウンタ実装）

3. **`crates/pasta_lua/scripts/pasta/shiori/main.lua`**
   - 実際の本番用コード（現在はミニマル実装）

**変更の必要性**:
- フィクスチャはテスト固有の実装なので、**本番コードの変更後も維持**
- テストが `SHIORI` グローバル変数ベースなら、main.lua → entry.lua 変更の影響なし

### 1.4 コーディング規約

**lua-coding.md から抽出**:
- モジュールテーブル: `UPPER_CASE`（例: `local ENTRY = {}`）
- モジュール構造: require → テーブル宣言 → ローカル関数 → 公開関数 → return
- グローバル変数: `SHIORI` テーブルは例外的に許可される（Rust側が参照するため）
- LuaDoc: `--- @module`, `--- @param`, `--- @return` 形式

---

## 2. Requirements Feasibility Analysis

### 2.1 技術的要件マッピング

| 要件 | 必要なアセット | 現在の状態 | ギャップ |
|------|---------------|-----------|---------|
| Req 1: グローバル SHIORI テーブル初期化 | entry.lua, グローバル変数 | main.lua で実装済み | **移植が必要** |
| Req 2: SHIORI.load 実装 | entry.lua, SHIORI.load 関数 | main.lua で minimal 実装 | **拡張ポイントコメント追加** |
| Req 3: SHIORI.request 実装 | entry.lua, EVENT.fire 呼び出し | **未実装**（204固定） | **新規実装が必要** |
| Req 4: SHIORI.unload 実装 | entry.lua, SHIORI.unload 関数 | **未実装** | **新規実装が必要** |
| Req 5: 既存モジュール移行 | Rust側変更, main.lua 削除 | main.lua がハードコード | **Rust側変更が必要** |
| Req 6: Rust側整合性 | グローバル SHIORI テーブル | 互換性あり | **ギャップなし** |
| Req 7: テスト要件 | 単体テスト, 結合テスト | 結合テストのみ存在 | **単体テスト新規作成** |

### 2.2 ギャップ詳細

#### Missing: SHIORI.request の EVENT.fire 呼び出し

**現状**: `main.lua` は固定の 204 レスポンスを返す

**要件**: `EVENT.fire(req)` を呼び出してイベント振り分けを実行

**実装内容**:
```lua
function SHIORI.request(req)
    return EVENT.fire(req)
end
```

**依存**:
- `pasta.shiori.event` モジュール（完成済み）
- Rust側から `req` テーブルを受け取る（`lua_request.rs` で実装済み）

#### Missing: SHIORI.unload 実装

**現状**: 未実装

**要件**: クリーンアップ処理の拡張ポイントを提供

**実装内容**:
```lua
function SHIORI.unload()
    -- 将来の拡張ポイント
end
```

#### Missing: Rust側のファイル名変更

**現状**: `scripts/pasta/shiori/main.lua` をハードコード

**要件**: `scripts/pasta/shiori/entry.lua` に変更

**変更箇所**:
- `crates/pasta_lua/src/runtime/mod.rs:309` (from_loader)
- `crates/pasta_lua/src/runtime/mod.rs:378` (from_loader_with_scene_dic)

#### Missing: 単体テスト

**現状**: 結合テストのみ（`shiori_lifecycle_test.rs`, `shiori_event_test.rs`）

**要件**: entry モジュール単独でのテスト

**テストケース**:
- `require("pasta.shiori.entry")` でグローバル `SHIORI` が作成される
- `SHIORI.load(0, "/path")` が `true` を返す
- `SHIORI.request(req)` が `EVENT.fire` を呼び出す
- `SHIORI.unload()` がエラーなく完了する

### 2.3 制約・前提

#### 制約

1. **Rust側のファイル名依存**: `main.lua` が2箇所でハードコードされている
2. **グローバル変数契約**: Rust側が `SHIORI` グローバルテーブルを期待している
3. **テストフィクスチャの互換性**: 既存テストが動作し続ける必要がある

#### 前提

1. **EVENT モジュールは完成済み**: `EVENT.fire` が正常に動作する
2. **RES モジュールは完成済み**: エラーハンドリングが可能
3. **Rust側のリクエストパース**: `lua_request.rs` が req テーブルを生成済み

### 2.4 複雑度シグナル

| 観点 | 評価 | 理由 |
|------|------|------|
| ビジネスロジック | 単純 | イベント振り分けへの委譲のみ |
| データモデル | 単純 | reqテーブルはRust側で構築済み |
| 外部統合 | 低 | EVENT/RES モジュールのみ（既存） |
| アルゴリズム | なし | 単純な関数呼び出し |

---

## 3. Implementation Approach Options

### Option A: Extend Existing (main.lua を修正)

**該当**: 既存の `main.lua` を修正して EVENT.fire を呼び出す

#### 修正対象ファイル

- `crates/pasta_lua/scripts/pasta/shiori/main.lua`

#### 変更内容

1. **SHIORI.request の修正**:
   ```lua
   local EVENT = require("pasta.shiori.event")
   
   function SHIORI.request(req)
       return EVENT.fire(req)
   end
   ```

2. **SHIORI.unload の追加**:
   ```lua
   function SHIORI.unload()
       -- 将来の拡張ポイント
   end
   ```

#### 互換性評価

- ✅ グローバル `SHIORI` テーブル維持
- ✅ Rust側の変更不要（ファイル名そのまま）
- ✅ テストフィクスチャへの影響なし

#### 複雑度・保守性

- ✅ 変更量が最小（10行程度）
- ✅ 認知負荷が低い（既存ファイルの修正のみ）
- ❌ **要件との不一致**: 要件は「entry.lua を新規作成」を明示

#### Trade-offs

- ✅ 最速の実装（Rust側変更なし）
- ✅ テスト影響ゼロ
- ✅ リスク最小
- ❌ **仕様書との乖離**: 要件5「既存モジュール移行」に反する
- ❌ 命名が不明瞭（"main" は責務を表現していない）

---

### Option B: Create New Component (entry.lua を新規作成)

**該当**: 新しい `entry.lua` を作成し、main.lua を削除

#### 新規ファイル

- `crates/pasta_lua/scripts/pasta/shiori/entry.lua`

#### 統合ポイント

**Rust側の変更**:
- `crates/pasta_lua/src/runtime/mod.rs:309`
  ```rust
  .join("scripts/pasta/shiori/entry.lua");  // main.lua → entry.lua
  ```
- `crates/pasta_lua/src/runtime/mod.rs:378`
  ```rust
  .join("scripts/pasta/shiori/entry.lua");  // main.lua → entry.lua
  ```

**main.lua の処理**:
- オプション1: 完全削除
- オプション2: 空ファイル化（後方互換性維持）
- オプション3: entry.lua への薄いラッパー
  ```lua
  return require("pasta.shiori.entry")
  ```

#### 責務境界

**entry.lua の責務**:
- グローバル `SHIORI` テーブルの初期化
- `SHIORI.load/request/unload` の実装
- EVENT モジュールへの委譲

**依存モジュール**:
- `pasta.shiori.event` - イベント振り分け
- `pasta.shiori.res` - 将来のエラーレスポンス生成（オプション）

#### Trade-offs

- ✅ 要件との完全一致（entry.lua 新規作成）
- ✅ 責務が明確（"entry" は役割を表現）
- ✅ 将来の拡張に備えた設計
- ❌ Rust側の変更が必要（2箇所）
- ❌ テストフィクスチャの考慮が必要

---

### Option C: Hybrid Approach (段階的移行)

**該当**: main.lua を entry.lua のラッパーとして保持

#### フェーズ1: entry.lua 新規作成

1. `entry.lua` を新規作成（完全実装）
2. `main.lua` を薄いラッパーに変更:
   ```lua
   --- @module pasta.shiori.main
   --- Deprecated: Use pasta.shiori.entry instead
   return require("pasta.shiori.entry")
   ```

#### フェーズ2: Rust側変更（オプション）

- Rust側を `entry.lua` に変更（main.lua は後方互換のため残す）
- テストフィクスチャはそのまま維持

#### フェーズ3: main.lua 削除（将来）

- 十分な移行期間後に main.lua を削除

#### リスク緩和

- ✅ 既存テストがそのまま動作（main.lua がラッパーとして機能）
- ✅ Rust側の変更を段階的に実施可能
- ✅ ロールバック容易

#### Trade-offs

- ✅ 段階的な移行が可能
- ✅ 後方互換性維持
- ✅ リスク最小
- ❌ 一時的な冗長性（main.lua と entry.lua が共存）
- ❌ 完全移行まで時間がかかる

---

## 4. Research Needed

以下の項目は設計フェーズで調査が必要：

1. **テストフィクスチャの扱い**
   - `pasta_shiori/tests/fixtures/` の main.lua をどう扱うか
   - テスト専用の実装を維持すべきか、entry.lua に統一すべきか

2. **SHIORI.load の拡張ポイント設計**
   - 設定ファイル読み込み（config.lua）の仕様
   - セーブデータ復元の設計（要件には「将来の拡張」とのみ記載）

3. **SHIORI.unload の拡張ポイント設計**
   - セーブデータ永続化のタイミング
   - リソース解放の必要性

4. **エラーハンドリング強化**
   - EVENT.fire がエラーを投げた場合の処理（要件7.3）
   - デバッグ情報（X-Error-Reason ヘッダー）の仕様

---

## 5. Implementation Complexity & Risk

### Effort Estimation

**サイズ**: **S (1-3日)**

**理由**:
- 既存パターンに従った単純な関数委譲実装
- EVENT/RES モジュールは完成済み
- Rust側の変更は2箇所のファイル名変更のみ

**内訳**:
- entry.lua 作成: 0.5日
- Rust側変更: 0.5日
- 単体テスト作成: 1日
- 結合テスト確認: 0.5日
- ドキュメント更新: 0.5日

### Risk Assessment

**リスク**: **Low**

**理由**:
- 既存パターン（EVENT.fire）を利用
- 技術スタックは既知（Lua, mlua）
- 統合ポイントは明確（グローバル SHIORI テーブル）
- パフォーマンス要件なし
- セキュリティ影響なし（内部モジュール）

**潜在的リスク**:
- テストフィクスチャの破壊（軽減策: main.lua をラッパーとして残す）
- Rust側のファイル名変更漏れ（軽減策: grep で徹底検索）

---

## 6. Recommendations for Design Phase

### 推奨アプローチ

**Option B (Complete Replacement)** を採用（開発者判断: 2026-01-27）

**理由**:
1. **要件との整合性**: entry.lua 新規作成を満たす
2. **テストは既に対応済み**: フィクスチャは reqテーブルを受け取っている
3. **クリーンな移行**: 古い main.lua を残さずシンプルに
4. **今後のエントリーポイント**: entry.lua に統一

### 設計フェーズで決定すべき事項

1. **main.lua の扱い**
   - 完全削除 vs ラッパー維持 vs 空ファイル化
   - 推奨: ラッパー維持（後方互換性）

2. **Rust側の変更タイミング**
   - entry.lua 作成と同時 vs 後回し
   - 推奨: 同時変更（シンプル化）

3. **テストフィクスチャ**
   - そのまま維持 vs entry.lua に統一
   - 推奨: そのまま維持（テスト固有の実装）

4. **拡張ポイントの仕様**
   - SHIORI.load/unload のフック設計
   - 推奨: コメントのみ（実装は将来）

### 重点調査項目

1. ✅ **Rust側のファイル名ハードコード箇所の網羅的検索**
   - `grep -r "main.lua" crates/` で追加箇所がないか確認

2. ✅ **テストフィクスチャの影響範囲**
   - `pasta_shiori/tests/fixtures/` のテストが main.lua に依存しているか確認

3. **エラーハンドリングの設計**
   - EVENT.fire がエラーを投げた場合の 500 レスポンス生成
   - 要件7.3「エラーハンドリングの強化」の詳細仕様を確定

---

## 7. Summary

### 主要な発見

1. **既存 main.lua はミニマル実装**: EVENT.fire を呼んでいないため、イベント振り分けが機能していない
2. **EVENT モジュールは完成済み**: 新規実装不要、呼び出すだけで動作
3. **Rust側はファイル名をハードコード**: 2箇所の変更が必須
4. **グローバル SHIORI テーブルベース**: モジュール名変更の影響は限定的

### 推奨戦略

**Hybrid Approach (Option C)** で段階的移行:
1. entry.lua を新規作成（完全実装）
2. main.lua を entry.lua のラッパーに変更（後方互換性維持）
3. Rust側を entry.lua に変更
4. テスト実行・検証
5. （将来）main.lua を削除

### 次のステップ

設計フェーズで以下を確定：
- entry.lua の詳細設計（LuaDoc, エラーハンドリング）
- Rust側の変更箇所リスト
- テスト戦略（単体テスト、結合テスト、E2Eテスト）
- ドキュメント更新計画（README, steering）
