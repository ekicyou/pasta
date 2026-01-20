# Implementation Gap Analysis

## 分析サマリー

### スコープ
- **要件**: `lua_request::parse_request`を使用してSHIORIリクエストを解析し、Lua側に解析済みテーブル`req`として渡す
- **影響範囲**: `PastaShiori::call_lua_request`メソッド1箇所の変更（約30行）
- **既存資産**: `parse_request`関数と包括的なテストスイートが既に実装済み

### 主な課題
- **Luaスクリプト互換性**: 現在のテストフィクスチャは生テキストを受け取る設計（`request_text`パラメータ）
- **エラーハンドリング**: パース失敗時の適切なフォールバック戦略が必要
- **後方互換性**: 既存のLuaスクリプトとの互換性維持が推奨要件に含まれる

### 推奨事項
- **Option B (新規メソッド作成)** を推奨：既存の`call_lua_request`を維持し、新規`call_lua_request_with_parsed_table`を追加
- 段階的移行により後方互換性を保ちつつ、将来的な完全移行を可能にする
- 既存テストの影響を最小化しながら、新機能の包括的なテストを追加

---

## 1. 現状調査

### 1.1 関連ファイル・モジュール

**コア実装**:
- [crates/pasta_shiori/src/shiori.rs](../../../crates/pasta_shiori/src/shiori.rs) (858行)
  - `PastaShiori` 構造体: SHIORI DLLインターフェース実装
  - `call_lua_request` メソッド (L253-276): **変更対象**
  - `cache_lua_functions` メソッド (L165-207): 関数キャッシュ機構
  
- [crates/pasta_shiori/src/lua_request.rs](../../../crates/pasta_shiori/src/lua_request.rs) (120行)
  - `parse_request` 関数 (L43-50): **再利用対象** - 既に実装済み
  - Pest文法ベースのSHIORIリクエストパーサー

**エラー型**:
- [crates/pasta_shiori/src/error.rs](../../../crates/pasta_shiori/src/error.rs)
  - `MyError` enum: 包括的なエラー型定義
  - `MyResult<T>` type alias
  - `From<mlua::Error>`, `From<ParseError>` 実装済み

**テスト資産**:
- [crates/pasta_shiori/tests/lua_request_test.rs](../../../crates/pasta_shiori/tests/lua_request_test.rs) (322行)
  - `parse_request` 関数の包括的なテスト (20+ test cases)
  - SHIORI3 GET/NOTIFYリクエストのフィクスチャ
  
- [crates/pasta_shiori/tests/shiori_lifecycle_test.rs](../../../crates/pasta_shiori/tests/shiori_lifecycle_test.rs) (250行)
  - 統合テスト: load/request/unloadライフサイクル検証
  - **影響あり**: `SHIORI.request(request_text)` の呼び出し前提

**Luaフィクスチャ**:
- [crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua](../../../crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua)
  - 現在の実装: `function SHIORI.request(request_text)` (L42)
  - **変更必要**: `function SHIORI.request(req)` に変更

### 1.2 既存のアーキテクチャパターン

**レイヤー構成**:
```
pasta_shiori (Windows DLL層)
├── shiori.rs      # SHIORI protocol interface
├── lua_request.rs # Request parsing utilities
├── error.rs       # Error type definitions
└── windows.rs     # Windows DLL exports
     ↓ depends on
pasta_lua (Lua runtime層)
├── PastaLuaRuntime
└── mlua bindings
```

**命名規約**:
- メソッド名: `snake_case` (例: `call_lua_request`, `parse_request`)
- エラー型: `MyError`, `MyResult<T>`
- Lua関数名: `SHIORI.load`, `SHIORI.request`, `SHIORI.unload`

**エラーハンドリングパターン**:
- `MyResult<T>` を使用した統一的なエラー伝播
- `From<E>` traitによる自動変換
- エラー時のロギング (`error!`, `warn!` マクロ)

**テスト戦略**:
- ユニットテスト: `lua_request_test.rs` で `parse_request` 関数を検証
- 統合テスト: `shiori_lifecycle_test.rs` でE2Eフローを検証
- フィクスチャベースのテスト: `tests/fixtures/` 配下

### 1.3 統合サーフェス

**データモデル**:
- **入力**: 生SHIORI REQUEST文字列 (SHIORI/3.0プロトコル)
- **中間形式**: Lua Table (解析済み) - `parse_request`が生成
- **出力**: SHIORI RESPONSE文字列 (Lua側で生成)

**Luaテーブル構造** (既存実装の`parse_request`出力):
```lua
{
  method = "get" | "notify",
  version = 30,
  charset = "UTF-8",
  id = "OnBoot",
  base_id = "OnBoot",
  status = "starting",
  security_level = "local",
  sender = "SSP",
  reference = {
    [0] = "shell",
    [1] = "first",
    -- ...
  },
  dic = {
    Charset = "UTF-8",
    ID = "OnBoot",
    Reference0 = "shell",
    -- 全ヘッダーの辞書
  }
}
```

**API境界**:
- Rust → Lua: `mlua::Function::call<T>(args)` メソッド
- 現在: `request_fn.call::<String>(request: &str)` (L265)
- 変更後: `request_fn.call::<String>(req_table: Table)` 

---

## 2. 要件実現性分析

### 2.1 技術的要求事項

| 要件 | 技術要素 | 現状 |
|------|---------|------|
| Req 1.1: parse_request使用 | `lua_request::parse_request(&Lua, &str)` | ✅ 実装済み (L43-50) |
| Req 1.2: reqテーブル渡し | `mlua::Function::call::<String>(Table)` | ✅ mlua API対応済み |
| Req 1.3: エラーログ出力 | `tracing::error!` マクロ | ✅ 既存実装で使用中 |
| Req 2.1-8: テーブル構造 | `parse_request`の出力構造 | ✅ テスト検証済み (lua_request_test.rs) |
| Req 3.1: 第一引数にテーブル | Lua側: `function SHIORI.request(req)` | ❌ 現在は`request_text`文字列 |
| Req 3.2: 生テキスト渡さない | 関数シグネチャ変更 | ⚠️ 互換性に影響 |
| Req 4.1: 204レスポンス維持 | `default_204_response()` (L297-302) | ✅ 既存動作維持可能 |
| Req 4.2: エラーログ記録 | `error!` + `MyError` 返却 | ✅ 既存パターン踏襲 |

### 2.2 ギャップと制約

**Missing / 実装必要**:
1. **`call_lua_request`メソッドの変更**
   - 現在: `request_fn.call::<String>(request)` (生文字列)
   - 必要: `parse_request`呼び出し → テーブル生成 → `request_fn.call::<String>(req_table)`

2. **Luaフィクスチャの更新**
   - `shiori_lifecycle/scripts/pasta/shiori/main.lua` の `SHIORI.request(request_text)` を変更
   - 既存テストが依存するため、全フィクスチャの調査が必要

3. **パースエラーハンドリング**
   - `parse_request` が失敗した場合のフォールバック戦略
   - エラーレスポンス生成（404 Bad Requestなど）

**Constraints / 制約**:
- **後方互換性**: Req 4では「影響を最小限に」とあるが、Req 3.2で「生テキストを渡さない」と明示
  - トレードオフ: 完全な新方式移行 vs 段階的移行
- **テスト影響範囲**: `shiori_lifecycle_test.rs` の4テストケースが依存
- **既存Luaスクリプト**: pasta_luaクレート内のサンプルスクリプトも確認必要

**Research Needed**:
- 既存のプロダクション環境で使用されているLuaスクリプトの有無
- 完全移行のタイムラインと段階的アプローチの妥当性

### 2.3 複雑性シグナル

- **Simple**: 既存の`parse_request`関数を呼び出すだけ
- **Moderate**: エラーハンドリングとログ出力の追加
- **Complex**: 後方互換性維持のための段階的移行戦略（オプショナル）

---

## 3. 実装アプローチオプション

### Option A: 既存メソッド直接変更（完全移行）

**変更対象**:
- `shiori.rs::call_lua_request` メソッド (L253-276)

**実装手順**:
1. `call_lua_request`内で`parse_request`を呼び出し
2. パース成功時: `request_fn.call::<String>(req_table)`
3. パース失敗時: エラーログ出力 + 400 Bad Request レスポンス返却

**コード例**:
```rust
fn call_lua_request(&self, request: &str) -> MyResult<String> {
    let request_fn = match &self.request_fn {
        Some(f) => f,
        None => {
            debug!("SHIORI.request not available, returning default 204 response");
            return Ok(Self::default_204_response());
        }
    };

    // Parse request to Lua table
    let runtime = self.runtime.as_ref().ok_or(MyError::NotInitialized)?;
    let lua = runtime.lua();
    
    let req_table = match lua_request::parse_request(lua, request) {
        Ok(table) => table,
        Err(e) => {
            error!(error = %e, "Failed to parse SHIORI request");
            return Ok(Self::error_400_response());
        }
    };

    // Call SHIORI.request(req_table)
    match request_fn.call::<String>(req_table) {
        Ok(response) => {
            debug!(response_len = response.len(), "SHIORI.request completed");
            Ok(response)
        }
        Err(e) => {
            error!(error = %e, "SHIORI.request execution failed");
            Err(MyError::from(e))
        }
    }
}
```

**Luaスクリプト変更** (`shiori/main.lua`):
```lua
-- Before:
function SHIORI.request(request_text)
    SHIORI.request_count = SHIORI.request_count + 1
    -- manual parsing needed
end

-- After:
function SHIORI.request(req)
    SHIORI.request_count = SHIORI.request_count + 1
    -- req.method, req.id, req.reference already available
    local event_id = req.id
end
```

**トレードオフ**:
- ✅ 最小限のコード変更（約20行）
- ✅ 要件に完全準拠
- ✅ Luaスクリプト側が最もシンプル
- ❌ 既存Luaスクリプトとの**破壊的変更**
- ❌ テストフィクスチャ全体の更新が必要
- ❌ 段階的移行不可（一斉変更が必要）

**影響範囲**:
- `crates/pasta_shiori/src/shiori.rs`: 1メソッド変更
- `crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua`: 関数シグネチャ変更
- `crates/pasta_shiori/tests/shiori_lifecycle_test.rs`: テスト期待値変更なし（Lua内部で吸収）

---

### Option B: 新規メソッド作成（段階的移行）

**追加メソッド**:
- `call_lua_request_with_parsed_table(&self, request: &str)` を新規作成
- 既存の `call_lua_request` は維持（deprecation警告追加）

**実装手順**:
1. 新規メソッド `call_lua_request_with_parsed_table` を実装（Option Aと同内容）
2. `Shiori::request` トレートメソッド内で新規メソッドを呼び出し
3. 既存の `call_lua_request` は `#[deprecated]` アノテーション付きで残す

**コード例**:
```rust
impl PastaShiori {
    /// Call SHIORI.request with raw text (deprecated).
    #[deprecated(since = "0.2.0", note = "Use call_lua_request_with_parsed_table instead")]
    fn call_lua_request(&self, request: &str) -> MyResult<String> {
        // 既存実装を維持
    }

    /// Call SHIORI.request with parsed table.
    fn call_lua_request_with_parsed_table(&self, request: &str) -> MyResult<String> {
        // Option A と同じ実装
    }
}

impl Shiori for PastaShiori {
    fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
        // 新規メソッドを使用
        self.call_lua_request_with_parsed_table(req.as_ref())
    }
}
```

**Lua側の互換性戦略**:
```lua
function SHIORI.request(arg)
    if type(arg) == "table" then
        -- New API: parsed table
        local event_id = arg.id
    else
        -- Old API: raw text (fallback)
        warn("Deprecated: SHIORI.request(text) is deprecated, use table API")
        -- manual parsing
    end
end
```

**トレードオフ**:
- ✅ 既存コードへの影響ゼロ（後方互換性維持）
- ✅ 段階的移行が可能（テスト・フィクスチャを順次更新）
- ✅ deprecation警告により将来の移行を促進
- ✅ 既存のLuaスクリプトが動作し続ける
- ❌ コード増加（2つのメソッドを維持）
- ❌ Lua側での型チェックロジックが必要（複雑性増加）
- ❌ 完全移行まで技術的負債が残る

**影響範囲**:
- `crates/pasta_shiori/src/shiori.rs`: 1メソッド追加 + 1トレートメソッド変更
- テストフィクスチャ: 段階的に更新可能（既存テストは継続動作）

---

### Option C: ハイブリッドアプローチ（フィーチャーフラグ）

**戦略**:
- Cargo feature flag (`parsed-request-table`) を導入
- デフォルトは旧API、フラグ有効時に新APIを使用

**実装手順**:
1. `Cargo.toml` に feature flag 定義
2. `call_lua_request` 内で `#[cfg(feature = "parsed-request-table")]` による条件分岐
3. CI/CDで両方のビルドをテスト

**コード例**:
```rust
fn call_lua_request(&self, request: &str) -> MyResult<String> {
    #[cfg(feature = "parsed-request-table")]
    {
        // Option A の実装（テーブル渡し）
    }

    #[cfg(not(feature = "parsed-request-table"))]
    {
        // 既存実装（生テキスト渡し）
    }
}
```

**トレードオフ**:
- ✅ ビルド時にAPIを切り替え可能（リリース戦略の柔軟性）
- ✅ A/Bテストによる動作検証が可能
- ❌ 条件分岐によるコード複雑性増加
- ❌ feature flagの管理コストが高い
- ❌ 最終的には一本化が必要（一時的な解決策）

**影響範囲**:
- `crates/pasta_shiori/Cargo.toml`: feature定義追加
- `crates/pasta_shiori/src/shiori.rs`: 条件分岐追加
- CI設定: 両方のビルド構成追加

---

## 4. 実装複雑性とリスク評価

### 努力見積もり: **S (1-3 days)**

**根拠**:
- コア変更は`call_lua_request`メソッド1箇所（約30行）
- `parse_request` 関数は既に実装・テスト済み
- エラーハンドリングパターンは既存コードで確立済み

**内訳**:
- コア実装: 0.5日（Option A: 20行変更、Option B: 30行追加）
- テストフィクスチャ更新: 0.5日（Luaスクリプト1ファイル）
- テストケース追加: 1日（エラーハンドリング検証）
- ドキュメント更新: 0.5日

### リスクレベル: **Low**

**根拠**:
- 確立されたパターンを使用（`parse_request`は既存関数）
- 統合ポイントが明確（`mlua::Function::call`のみ）
- 包括的なテストスイートが既存（lua_request_test.rs）

**リスク要因**:
- **Medium**: 後方互換性の影響範囲（Option Aの場合）
  - 緩和策: Option Bによる段階的移行
- **Low**: パースエラーハンドリング
  - 緩和策: 既存の`MyError`型で十分対応可能

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B (新規メソッド作成)**

**理由**:
1. **リスク最小化**: 既存テストへの影響ゼロ
2. **段階的移行**: フィクスチャとテストを順次更新可能
3. **後方互換性**: 既存Luaスクリプトが動作し続ける
4. **将来の完全移行**: deprecation警告により移行を促進

### 重要な設計決定事項

1. **エラーレスポンス仕様**
   - パース失敗時の具体的なSHIORIレスポンス形式
   - 提案: `SHIORI/3.0 400 Bad Request` + エラーメッセージ

2. **Lua側の型チェック戦略**（Option Bの場合）
   - `type(arg) == "table"` で判定
   - 旧APIサポートの期限設定

3. **テスト戦略**
   - 新規テストケース: パース失敗時の動作検証
   - 既存テスト維持: `shiori_lifecycle_test.rs`の互換性確認

### 追加調査項目

1. **pasta_luaクレート内のサンプルスクリプト調査**
   - `crates/pasta_lua/scripts/` 配下の確認
   - `scriptlibs/` 配下の依存関係確認

2. **SHIORI仕様準拠性確認**
   - SHIORI/3.0プロトコルにおけるエラーレスポンス仕様
   - 既存の伺かベースウェアとの互換性

3. **パフォーマンス影響**
   - `parse_request`のオーバーヘッド計測（ベンチマーク追加）

---

## 6. 要件-資産マッピング

| 要件ID | 資産/機能 | ギャップ | 対応アプローチ |
|--------|---------|---------|--------------|
| Req 1.1 | `lua_request::parse_request` | ✅ 実装済み | 既存関数を呼び出し |
| Req 1.2 | `mlua::Function::call<Table>` | ✅ mlua対応済み | API利用 |
| Req 1.3 | エラーログ出力 | ✅ `tracing`使用中 | `error!` マクロ |
| Req 2.1-8 | テーブル構造 | ✅ テスト検証済み | そのまま利用 |
| Req 3.1 | 第一引数テーブル | ❌ Missing | `call_lua_request`変更 |
| Req 3.2 | 生テキスト不使用 | ⚠️ Constraint | シグネチャ変更 |
| Req 3.3 | load/unload維持 | ✅ 影響なし | 変更不要 |
| Req 4.1 | 204レスポンス維持 | ✅ `default_204_response()` | 既存関数維持 |
| Req 4.2 | エラーログ+返却 | ✅ `MyError` | 既存パターン |

---

## 次のステップ

### 設計フェーズへの移行

**推奨コマンド**:
```bash
/kiro-spec-design shiori-request-parsed-table
```

**設計フェーズで決定すべき事項**:
1. **最終アプローチ選択**: Option A vs Option B vs Option C
2. **エラーレスポンス詳細仕様**: 400 Bad Request の具体的なフォーマット
3. **テスト計画**: 新規テストケースとカバレッジ目標
4. **移行戦略**: 段階的移行のタイムラインと完了基準（Option Bの場合）
5. **ドキュメント更新**: Luaスクリプト開発者向けガイド

**参照すべき追加資料**:
- SHIORI/3.0 プロトコル仕様
- mlua ライブラリドキュメント
- pasta_lua クレートの既存スクリプト実装例
