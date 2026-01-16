# Gap Analysis: shiori-request-minimal-implementation

## 分析サマリー

- **スコープ**: pasta_shiori から pasta_lua への SHIORI リクエスト処理委譲（Rust → Lua → Rust の往復）
- **主要チャレンジ**: 
  - PastaLuaRuntime に Lua グローバル関数を呼び出す機能が存在しない（`call_global` メソッド未実装）
  - main.lua スクリプトの自動ロード機構が存在しない
  - Lua 側で SHIORI プロトコルレスポンスを構築する標準ライブラリが不在
- **推奨アプローチ**: ハイブリッド（既存拡張 + 新規作成）
  - PastaLuaRuntime に `call_global` メソッドを追加（既存拡張）
  - scripts/pasta/shiori/main.lua を新規作成（新規）
  - PastaLoader の main.lua 自動ロード処理を追加（既存拡張）

---

## 1. Current State Investigation

### 既存アセット

#### pasta_shiori クレート構成

| ファイル | 役割 | 現状 |
|---------|------|------|
| `src/shiori.rs` | PastaShiori 実装 | load() 実装済み、request() は TODO スタブ |
| `src/error.rs` | エラー型定義 | MyError::Script, to_shiori_response() 実装済み |
| `src/logging/` | ロギング機構 | 実装済み（GlobalLoggerRegistry） |
| `src/windows.rs` | Windows DLL エクスポート | 実装済み |

**キーパターン**:
- `PastaShiori::load()` は `PastaLoader::load()` を呼び出して `PastaLuaRuntime` を初期化
- `runtime: Option<PastaLuaRuntime>` でライフサイクル管理
- エラーは `MyError` → SHIORI レスポンス変換（`to_shiori_response()`）

#### pasta_lua クレート構成

| ファイル | 役割 | 現状 |
|---------|------|------|
| `src/runtime/mod.rs` | PastaLuaRuntime | exec(), exec_file(), lua(), register_module() 実装済み |
| `src/loader/mod.rs` | PastaLoader | load() 実装済み、pasta.toml 読み込み、トランスパイル |
| `src/loader/context.rs` | LoaderContext | lua_search_paths, package.path 生成実装済み |
| `scripts/pasta/init.lua` | PASTA モジュール | アクター・シーン API 実装済み |
| `scripts/pasta/shiori/init.lua` | SHIORI モジュール | 空スタブ（TODO コメントのみ） |

**キーパターン**:
- `PastaLuaRuntime::from_loader()` で transpiled コードをロード
- package.path は `LoaderContext::generate_package_path()` で設定（scripts/ 等）
- mlua crate を使用：`lua.load(script).eval()` でコード実行

### 既存コンベンション

| 項目 | 規約 |
|------|------|
| Rust-Lua 統合 | mlua crate、`LuaResult<Value>` 型 |
| エラーハンドリング | `MyError` 型、`From<LoaderError>` 実装済み |
| モジュールロード | `@pasta_*` 形式、package.loaded に登録 |
| テスト配置 | `tests/*_test.rs`、copy_fixture_to_temp() パターン |

### インテグレーションポイント

| ポイント | 既存実装 | ギャップ |
|---------|---------|---------|
| Lua 関数呼び出し | exec() で任意コード実行可能 | グローバル関数の直接呼び出し API なし |
| main.lua ロード | package.path に scripts/ 含まれる | 自動 require/load 機構なし |
| エラー伝播 | mlua::Error → MyError::Script 変換なし | From 実装が必要 |

---

## 2. Requirements Feasibility Analysis

### 技術要件マッピング

| Requirement | 必要な機能 | 現状 | ギャップ |
|------------|----------|------|---------|
| Req 1: main.lua ロード | scripts/pasta/shiori/main.lua を自動 require | package.path に scripts/ 含む | **Missing**: 自動ロード処理 |
| Req 2: SHIORI.request 関数 | Lua グローバル SHIORI テーブル | scripts/pasta/shiori/init.lua 空 | **Missing**: 関数定義 |
| Req 3: Rust 呼び出し | PastaShiori::request → Lua 関数（リクエスト渡し） | exec() で任意コード可能 | **Missing**: 関数参照保持と呼び出し |
| Req 4: 関数参照保持 | Load時に SHIORI.request 参照取得 | なし | **Missing**: 関数参照取得・保持機構 |
| Req 5: レスポンスフォーマット | 文字列結合のみ | Lua 標準機能で実現可能 | なし |
| Req 6: テスト | pasta_shiori テスト | 既存テストあり | **Extend**: 新テスト追加 |

### ギャップと制約

#### Missing Capabilities

1. **SHIORI.request 関数参照の保持機構**
   - 場所: PastaShiori 構造体
   - 機能: Load時に `lua.globals().get::<_, Table>("SHIORI")?.get::<_, Function>("request")?` で取得
   - 保持: `Option<mlua::Function>` フィールド
   - 呼び出し: `function.call::<_, String>((request_text,))?`

2. **main.lua 自動ロード機構**
   - 場所: PastaLoader または PastaLuaRuntime::from_loader
   - アプローチ: `runtime.exec("require('pasta.shiori.main')")` を初期化時に実行
   - エラーハンドリング: main.lua 不在時はワーニングログのみ（エラー不要）

3. **scripts/pasta/shiori/main.lua 実装**
   - 内容: SHIORI グローバルテーブルに request 関数を定義
   - 引数: リクエスト文字列（SHIORI/3.0 形式）
   - 戻り値: レスポンス文字列（204 No Content）

#### Research Needed

- **mlua::Function のライフタイム管理**: PastaShiori が Function 参照を保持する際の所有権とライフタイム設計
- **Lua エラー時の詳細メッセージ取得**: mlua::Error から message 抽出方法

#### Constraints

- **PastaLuaRuntime は mlua::Lua をラップ**: 直接 lua() メソッドでアクセス可能
- **既存 exec() は eval() ベース**: 戻り値を Value で返す
- **package.path は from_loader で設定済み**: scripts/ は既に検索対象
- **mlua::Function は Lua への参照を保持**: PastaShiori が Function を保持する場合、PastaLuaRuntime より長生きしないこと

### 複雑度シグナル

- **シンプルな API 追加**: call_global は既存 mlua パターンの薄いラッパー
- **既存アーキテクチャと整合**: PastaLoader → Runtime の初期化フローを踏襲
- **外部統合なし**: SHIORI プロトコルは文字列処理のみ

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components

**適用範囲**: PastaLuaRuntime, PastaLoader, pasta_shiori/shiori.rs

#### 拡張対象ファイル

1. **`crates/pasta_lua/src/runtime/mod.rs`**
   - 追加: `call_global(&self, name: &str, args: impl IntoLuaMulti) -> LuaResult<Value>` メソッド
   - 影響: なし（新規 pub メソッド）
   - 後方互換: 完全（既存 API に変更なし）

2. **`crates/pasta_lua/src/loader/mod.rs`**
   - 追加: main.lua 自動ロード処理（`from_loader` 内で `runtime.exec("require('pasta.shiori.main')")` 実行）
   - 影響: 初期化シーケンスに 1 ステップ追加
   - 後方互換: main.lua 不在時はワーニングのみ

3. **`crates/pasta_shiori/src/shiori.rs`**
   - 変更: `request()` メソッド実装（TODO 削除）
   - 影響: なし（スタブからの実装）

4. **`crates/pasta_shiori/src/error.rs`**
   - 追加: `From<mlua::Error> for MyError` 実装
   - 影響: なし（新規 trait 実装）

#### 複雑度とメンテナビリティ

- ✅ **認知負荷低**: 各ファイルに 1 メソッド/処理追加のみ
- ✅ **単一責任維持**: call_global は Runtime の責務、main.lua ロードは Loader の責務
- ✅ **ファイルサイズ**: runtime/mod.rs は 375 行、20 行程度の追加で許容範囲内

#### Trade-offs

- ✅ 最小限のファイル変更（3-4 ファイル）
- ✅ 既存パターン踏襲（mlua ラッパー、Loader 初期化フロー）
- ❌ runtime/mod.rs の機能が増える（ただし関連性あり）

### Option B: Create New Components

**適用範囲**: SHIORI プロトコルハンドラーを独立モジュール化

#### 新規作成理由

- SHIORI プロトコル処理は将来的に拡張可能（リクエストパース、レスポンスビルダー）
- pasta_shiori/shiori.rs から分離することで、プロトコル層とブリッジ層を分離

#### 新規コンポーネント

1. **`crates/pasta_lua/src/shiori/protocol.rs`**
   - 責務: SHIORI レスポンス構築ユーティリティ
   - API: `build_204_response() -> String`
   - 統合: pasta_shiori から呼び出し

2. **`crates/pasta_shiori/src/lua_bridge.rs`**
   - 責務: Rust-Lua ブリッジ専用
   - API: `call_shiori_request(runtime: &PastaLuaRuntime, request: &str) -> MyResult<String>`
   - 統合: shiori.rs の request() から呼び出し

#### Trade-offs

- ✅ 関心の分離（プロトコル層、ブリッジ層）
- ✅ 将来の SHIORI 拡張に対応しやすい
- ❌ ファイル数増加（+2 ファイル）
- ❌ 現時点では過剰設計（204 No Content のみ）

### Option C: Hybrid Approach ⭐ **推奨**

**戦略**: 最小限の拡張 + Lua 側新規作成

#### Phase 1: Minimal Viable Implementation

##### Rust 側拡張（既存コンポーネント）

1. **PastaShiori に SHIORI.request 関数参照を保持**
   - ファイル: `crates/pasta_shiori/src/shiori.rs`
   - 実装: `shiori_request_fn: Option<mlua::Function>` フィールド追加（5 行）
   - Load時: Runtime から関数取得・保持（10-15 行）

2. **PastaLoader に main.lua 自動ロード追加**
   - ファイル: `crates/pasta_lua/src/loader/mod.rs` (from_loader 内)
   - 実装: `runtime.exec("require('pasta.shiori.main')")` + エラーハンドリング（10 行）

3. **PastaShiori::request 実装**
   - ファイル: `crates/pasta_shiori/src/shiori.rs`
   - 実装: 保持した関数を `function.call::<_, String>((req,))?` で呼び出し（15-20 行）

4. **エラー変換追加**
   - ファイル: `crates/pasta_shiori/src/error.rs`
   - 実装: `From<mlua::Error>` trait（5-10 行）

##### Lua 側新規作成

5. **scripts/pasta/shiori/main.lua 作成**
   - 内容: SHIORI グローバルテーブル定義、request(request_text) 関数実装
   - 引数: リクエスト文字列（SHIORI/3.0 形式）
   - 戻り値: 204 No Content レスポンス文字列
   - サイズ: 20-30 行

#### Phase 2: Future Enhancements（スコープ外）

- SHIORI リクエストパーサー（scripts/pasta/shiori/parser.lua）
- レスポンスビルダー（scripts/pasta/shiori/response.lua）
- イベントディスパッチャー（scripts/pasta/shiori/dispatcher.lua）

#### リスク軽減

- **段階的ロールアウト**: Phase 1 で最小実装、動作確認後に拡張
- **フィーチャーフラグ不要**: main.lua 不在時は従来通り動作（後方互換）
- **ロールバック戦略**: main.lua ロード失敗時はワーニングログのみ

#### Trade-offs

- ✅ 最小コスト（約 60-80 行の Rust コード + 30 行の Lua コード）
- ✅ 段階的拡張可能（Phase 2 で独立モジュール化）
- ✅ 既存パターンと整合
- ❌ Phase 2 への移行時にリファクタリング必要（ただし影響範囲限定的）

---

## 4. Implementation Complexity & Risk

### Effort Estimation

**サイズ: S (1-3 days)**

#### 理由

- 既存パターンの踏襲（mlua ラッパー、Loader 初期化フロー）
- 外部依存なし（mlua, pasta_lua 既存）
- SHIORI プロトコルはシンプル（文字列処理のみ）
- 変更ファイル数: 5 ファイル（Rust 4 + Lua 1）
- 総実装行数: 約 100 行

#### 内訳

| タスク | 見積もり | 根拠 |
|-------|---------|------|
| 関数参照保持機構実装 | 0.5 日 | mlua::Function 取得・保持、テスト含む |
| main.lua 自動ロード | 0.3 日 | 既存 from_loader 拡張のみ |
| PastaShiori::request 実装 | 0.5 日 | 関数 call のみ、テスト含む |
| エラー変換実装 | 0.2 日 | From trait 実装のみ |
| main.lua 作成 | 0.3 日 | レスポンス文字列構築のみ |
| 統合テスト | 0.7 日 | E2E テスト、エラーケース検証 |
| **合計** | **2.5 日** | |

### Risk Assessment

**リスク: Low**

#### 理由

| リスク要因 | レベル | 根拠 |
|----------|--------|------|
| 技術習熟度 | Low | mlua パターン既存コードで確認済み |
| 統合複雑度 | Low | 既存 Loader/Runtime 初期化フロー踏襲 |
| パフォーマンス | Low | 関数呼び出しオーバーヘッドのみ（文字列処理） |
| セキュリティ | Low | Lua VM サンドボックス内で実行 |
| 後方互換性 | Low | main.lua 不在時は既存動作維持 |
| アーキテクチャ変更 | None | 新規機能追加のみ |

#### リスク軽減策

- **mlua ドキュメント参照**: Function::call() パターン確認
- **既存テストパターン踏襲**: loader_integration_test.rs, shiori.rs テスト構造を模倣
- **段階的実装**: call_global → main.lua ロード → request 実装の順で進める

---

## 5. Recommendations for Design Phase

### 推奨アプローチ

**Option C: Hybrid Approach（最小限拡張 + Lua 側新規作成）**

#### 理由

- コスト最小（S サイズ、2.5 日）
- リスク最小（Low、既存パターン踏襲）
- 将来拡張性確保（Phase 2 でモジュール化可能）
- 既存コードへの影響最小（後方互換維持）

### 設計フェーズで決定すべき重要事項

1. **SHIORI.request 関数参照のライフタイム管理**
   - PastaShiori が mlua::Function を保持する際の所有権設計
   - PastaLuaRuntime との生存期間の関係
   - Option<mlua::Function> vs. 関数名保持 + 都度取得

2. **main.lua ロードタイミング**
   - from_loader の最後 vs. transpiled コードロード後
   - エラーレベル: warn vs. debug（main.lua 不在時）

3. **SHIORI.request Lua 関数シグネチャ**
   - グローバルテーブル構造: `SHIORI.request` 確定
   - 引数: リクエスト文字列のみ
   - 戻り値: レスポンス文字列のみ

4. **テスト戦略**
   - Fixture 追加: minimal に main.lua 含める vs. 専用 fixture 作成
   - エラーケース: Lua エラー、関数不在、戻り値型不正

### 研究項目（設計フェーズで調査）

1. **mlua::Function のライフタイム制約**
   - 文献: [mlua ドキュメント - Function](https://docs.rs/mlua/latest/mlua/struct.Function.html)
   - 確認事項: Function が参照する Lua インスタンスのライフタイム、所有権移動の可否

2. **Lua エラーメッセージ伝播**
   - 文献: mlua::Error 型定義
   - 確認事項: RuntimeError から message 抽出方法

3. **SHIORI/3.0 プロトコル詳細**
   - 文献: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html
   - 確認事項: ヘッダー必須性、CRLF 形式、エラーレスポンス仕様

---

## Appendix: Code Examples

### PastaShiori 構造体拡張例（参考）

```rust
pub(crate) struct PastaShiori {
    hinst: isize,
    load_dir: Option<PathBuf>,
    runtime: Option<PastaLuaRuntime>,
    
    /// SHIORI.request Lua 関数への参照（Load時に取得）
    shiori_request_fn: Option<mlua::Function>,
}

impl Shiori for PastaShiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool> {
        // ... existing load logic ...
        
        // SHIORI.request 関数を取得・保持
        if let Some(ref runtime) = self.runtime {
            match runtime.lua().globals().get::<_, mlua::Table>("SHIORI") {
                Ok(shiori_table) => {
                    match shiori_table.get::<_, mlua::Function>("request") {
                        Ok(func) => {
                            self.shiori_request_fn = Some(func);
                            debug!("SHIORI.request function loaded");
                        }
                        Err(e) => {
                            warn!("SHIORI.request function not found: {}", e);
                            self.shiori_request_fn = None;
                        }
                    }
                }
                Err(e) => {
                    warn!("SHIORI table not found: {}", e);
                    self.shiori_request_fn = None;
                }
            }
        }
        
        Ok(true)
    }
}
```

### main.lua 実装例（参考）

```lua
-- scripts/pasta/shiori/main.lua
-- SHIORI/3.0 Protocol Entry Point

local shiori = require("pasta.shiori")

-- Global SHIORI table
SHIORI = {}

--- Handle SHIORI/3.0 request
--- @param request_text string Raw SHIORI request (e.g., "GET SHIORI/3.0\r\n...")
--- @return string SHIORI response (204 No Content)
function SHIORI.request(request_text)
    -- Minimal implementation: return 204 No Content
    return "SHIORI/3.0 204 No Content\r\n" ..
           "Charset: UTF-8\r\n" ..
           "Sender: Pasta\r\n" ..
           "\r\n"
end

return SHIORI
```

### PastaShiori::request 実装例（参考）

```rust
fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
    let runtime = self.runtime.as_ref().ok_or(MyError::NotInitialized)?;
    let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));
    
    let req = req.as_ref();
    debug!(request_len = req.len(), "Processing SHIORI request");
    
    // Call SHIORI.request function if available
    if let Some(ref func) = self.shiori_request_fn {
        let response: String = func
            .call((req,))
            .map_err(|e| MyError::Script { 
                message: format!("SHIORI.request failed: {}", e) 
            })?;
        Ok(response)
    } else {
        // Fallback: return default 204 response
        warn!("SHIORI.request function not available, returning default 204");
        Ok("SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n".to_string())
    }
}
```
