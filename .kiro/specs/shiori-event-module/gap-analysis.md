# Implementation Gap Analysis: shiori-event-module

## 分析概要

本分析では、`pasta.shiori.event` モジュールの要件と既存コードベースのギャップを評価する。

**スコープ**: SHIORI イベント振り分けとハンドラ登録機構の実装
**既存資産**: `pasta.shiori.res`（完成済み）、`pasta.shiori.main`（ミニマル実装）、Rust側リクエストパース機構
**実装方針**: 新規サブモジュール作成（Option B）を推奨

---

## 1. 現状調査（Current State Investigation）

### 1.1 既存資産の確認

#### ディレクトリ構造

```
crates/pasta_lua/scripts/pasta/shiori/
├── init.lua        # 空のSHIORIモジュール（TODOコメントのみ）
├── main.lua        # グローバルSHIORIテーブル（minimal実装）
└── res.lua         # レスポンス組み立てモジュール（完成済み）
```

#### `pasta.shiori.res` モジュール（完成済み）

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/res.lua`

**提供機能**:
- `RES.build(code, dic)` — 汎用ビルダー
- `RES.ok(value, dic)` — 200 OK
- `RES.no_content(dic)` — 204 No Content
- `RES.err(reason, dic)` — 500 Internal Server Error
- `RES.warn(reason, dic)` — 警告付き204
- その他: `not_enough`, `advice`, `bad_request`

**環境設定**: `RES.env.charset`, `RES.env.sender`, `RES.env.security_level`

**テスト**: `crates/pasta_lua/tests/shiori_res_test.rs` で15テストケース実装済み

**評価**: ✅ 要件との依存関係が明確。そのまま利用可能。

---

#### `pasta.shiori.main` モジュール（minimal実装）

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/main.lua`

**現状**:
```lua
SHIORI = SHIORI or {}

function SHIORI.load(hinst, load_dir)
    return true  -- 常にtrue
end

function SHIORI.request(request_text)
    -- ハードコードされた204レスポンス
    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Sender: Pasta\r\n" ..
        "\r\n"
end
```

**評価**: 🔶 `SHIORI.request()` を `EVENT.fire(req)` 呼び出しに置き換える必要あり（既存関数の拡張）

---

#### Rust側リクエストパース機構

**ファイル**: `crates/pasta_shiori/src/lua_request.rs`

**関数**: `parse_request(lua: &Lua, text: &str) -> MyResult<Table>`

**生成するLuaテーブル構造**:
```lua
req = {
    method = "get",        -- "get" / "notify"
    version = 30,          -- SHIORI/3.0 → 30
    id = "OnBoot",         -- イベント名（キー要素）
    charset = "UTF-8",
    sender = "SSP",
    base_id = "OnBoot",
    status = "starting",
    security_level = "local",
    reference = { [0] = "shell", [1] = "first", ... },  -- 0始まり配列
    dic = { ID = "OnBoot", Charset = "UTF-8", ... },    -- 全ヘッダー辞書
}
```

**テスト**: `crates/pasta_shiori/tests/lua_request_test.rs` で包括的テスト済み

**評価**: ✅ `req.id` によるイベント振り分けに必要な構造が完備

---

### 1.2 アーキテクチャパターンと規約

#### モジュール命名規約（lua-coding.md）

| 対象 | スタイル | 例 |
|------|---------|-----|
| モジュールテーブル | UPPER_CASE | `local RES = {}`, `local EVENT = {}` |
| ローカル変数 | snake_case | `local handler = ...` |
| 関数 | snake_case | `function EVENT.fire(req)` |

#### モジュール構造テンプレート

```lua
--- @module pasta.shiori.event
--- 説明

-- 1. require文
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

-- 2. モジュールテーブル宣言
local EVENT = {}

-- 3. ローカル関数・定数

-- 4. 公開関数
function EVENT.fire(req) ... end
function EVENT.no_entry(req) ... end

-- 5. 末尾でreturn
return EVENT
```

#### テストパターン（shiori_res_test.rs）

```rust
fn create_runtime_with_pasta_path() -> PastaLuaRuntime {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime.exec(&format!(
        r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
    )).expect("Failed to configure package.path");
    runtime
}

#[test]
fn test_event_fire_dispatches_registered_handler() {
    let runtime = create_runtime_with_pasta_path();
    runtime.exec(r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        
        REG.OnBoot = function(req)
            return "custom response"
        end
        
        local req = { id = "OnBoot" }
        local response = EVENT.fire(req)
        assert(response == "custom response")
    "#).unwrap();
}
```

**評価**: ✅ 明確な規約とテストパターンが確立済み

---

### 1.3 循環参照回避パターン（store.lua参考）

`pasta.store` モジュールは他モジュールをrequireせず、共有データのみ提供する設計。

**本モジュールへの適用**:
- `register.lua` は依存ゼロ（空テーブルのみ返却）
- `init.lua` は `register.lua` と `res.lua` を require
- 循環参照リスクなし

---

## 2. 要件実現性分析（Requirements Feasibility Analysis）

### 2.1 技術要件の抽出

| 要件ID | 技術要件 | 既存資産 | ギャップ |
|--------|---------|---------|---------|
| Req 1 | ハンドラ登録テーブル（register.lua） | なし | **作成必要** |
| Req 2 | イベント振り分けモジュール構造（init.lua） | なし | **作成必要** |
| Req 3 | デフォルトハンドラ（no_entry） | `RES.no_content()` 利用可能 | **実装必要** |
| Req 4 | イベント振り分け（fire） | `req.id` 構造確認済み | **実装必要** |
| Req 5 | xpcallエラーハンドリング | `RES.err()` 利用可能 | **実装必要** |
| Req 6-7 | リクエスト構造理解 | Rust側で完備 | なし |
| Req 8 | 公開API定義 | モジュールパターン確立済み | 適用のみ |
| Req 9 | 使用例ドキュメント | なし | **Missing** | ドキュメント追加 |

### 2.2 ギャップと制約

#### 欠落機能（Missing Capabilities）

1. **新規ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`
2. **新規ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/register.lua`
3. **新規テスト**: `crates/pasta_lua/tests/shiori_event_test.rs`

**スコープ外**:
- `main.lua` の修正（Rust側統合で対応）
- Rust側の統合コード（別タスク）

#### 技術的制約（Constraints）

- **Lua VM制約**: xpcallのパフォーマンスオーバーヘッド（微小、許容範囲）
- **テーブル参照**: `REG[req.id]` は動的キー参照（Lua標準、問題なし）
- **統合制約**: `pasta.shiori.main` がグローバルSHIORIテーブルを定義しているため、requireパスの調整が必要

#### 未確認事項（Research Needed）

1. **pasta.shiori.main の require 順序**: グローバルSHIORIテーブルとモジュールrequireの相互作用
   - → 設計フェーズで検証（実装時に確認可能）

2. **req.id が nil の振る舞い**: Rust側パーサーが必ず id を設定するか？
   - → lua_request_test.rs で確認可能（既存テストに id=nil ケースなし）
   - → 防御的プログラミングで `req.id or "NIL"` を推奨

---

## 3. 実装アプローチオプション（Implementation Approach Options）

### Option A: 既存コンポーネント拡張 ❌

**該当**: `pasta.shiori.main.lua` を拡張してイベント振り分けも内包

**理由**:
- main.lua の責務がエントリーポイント定義から逸脱
- 単一ファイルに振り分けロジックとハンドラ管理を詰め込むと肥大化
- テスタビリティ低下（main.lua がグローバルSHIORI依存）

**Trade-offs**:
- ✅ ファイル数は最小
- ❌ 責任分離原則違反
- ❌ テスト困難

**評価**: 非推奨

---

### Option B: 新規サブモジュール作成 ✅ 推奨

**構成**:
```
crates/pasta_lua/scripts/pasta/shiori/
├── event/
│   ├── init.lua       # EVENT.fire, EVENT.no_entry
│   └── register.lua   # REG = {}
├── init.lua
├── main.lua           # SHIORI.request() を修正
└── res.lua
```

**rationale**:
- **責任分離**: イベント振り分けを独立モジュール化
- **再利用性**: 将来の拡張（イベントフック、ミドルウェア）に対応可能
- **テスタビリティ**: `pasta.shiori.event` を独立してテスト可能
- **既存パターン踏襲**: `pasta.shiori.res` と同様のサブモジュール構造

**integration points**:
- `pasta.shiori.main.SHIORI.request()` から `EVENT.fire(req)` を呼び出し
- Rust側 `lua_request::parse_request()` でreqテーブル生成済み
- `pasta.shiori.res` を内部で利用

**responsibility boundaries**:
- `register.lua`: ハンドラ登録テーブルの提供のみ
- `init.lua`: イベント振り分け、エラーハンドリング、デフォルト処理
- `main.lua`: Rustエントリーポイント、reqパース結果の受け渡し

**Trade-offs**:
- ✅ 責任分離明確
- ✅ テスト容易
- ✅ 拡張性高い
- ❌ ファイル数+2（微小なコスト）

**評価**: ✅ 強く推奨

---

### Option C: Hybrid Approach（段階的導入） 🔶

**Phase 1**: register.lua + minimal event.fire のみ実装
**Phase 2**: エラーハンドリング拡充
**Phase 3**: main.lua統合

**rationale**:
- 段階的なテスト・検証が可能
- 初期リスク低減

**Trade-offs**:
- ✅ リスク分散
- ❌ 複数PRが必要（オーバーヘッド）
- ❌ 中途半端な状態でのマージリスク

**評価**: シンプルな機能のため不要（Option Bで一括実装推奨）

---

## 4. 実装複雑度とリスク評価

### Effort（工数）

**S (1-3日)** ✅

**理由**:
- 新規ファイル2つ（合計 ~80行）
- main.lua修正（5行程度）
- テストファイル1つ（150行程度）
- 既存パターン踏襲で実装明確
- 外部ライブラリ依存なし

---

### Risk（リスク）

**Low（低リスク）** ✅

**理由**:
- **確立されたパターン**: lua-coding規約とres.luaの前例あり
- **明確な仕様**: 要件が具体的（EARS形式）
- **既存インフラ**: Rust側リクエストパース完備
- **テスト容易**: モジュール独立性高く、ユニットテスト可能
- **影響範囲限定**: 新規サブモジュールのため既存機能への影響微小

**潜在リスク**:
1. **req.id が nil の場合の振る舞い** → 防御的プログラミングで対応（`req.id or "NIL"`）
2. **main.lua のグローバルSHIORI依存** → 設計フェーズで require 順序を明確化

---

## 5. 設計フェーズへの推奨事項

### 5.1 推奨アプローチ

**Option B（新規サブモジュール作成）** を採用

**理由**:
- 責任分離原則に合致
- テスタビリティ高い
- 既存パターン（pasta.shiori.res）との一貫性
- 拡張性確保

---

### 5.2 設計判断事項（Design Decisions）

以下の3つの設計判断が必要です：

#### 議題1: main.lua統合方法 ✅ 決定

**決定**: main.lua統合は本仕様のスコープ外

**理由**:
- EVENT.fireは独立したイベントハンドリングツールとして実装
- 実際の統合はRust側（pasta_shiori）で行う
- Rust側で `SHIORI.request(req)` としてreqテーブルを渡す形式
- main.luaは現状のまま維持（minimal実装）

**影響**:
- 本仕様で実装: `pasta.shiori.event.register`, `pasta.shiori.event.init`
- 本仕様で実装しない: main.luaの修正、Rust側の統合コード
- テスト: EVENT.fire単体でテスト可能（reqテーブルをLua側で構築）

---

#### 議題2: req.idがnilの防御的プログラミング方針 ✅ 決定

**決定**: 選択肢A（Lua標準の挙動に任せる）

**実装方針**:
```lua
function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry
    -- req.id が nil なら REG[nil] → nil → EVENT.no_entry
    
    local ok, result = xpcall(function()
        return handler(req)
    end, debug.traceback)
    
    if ok then
        return result
    else
        return RES.err(result)
    end
end
```

**理由**:
- **シンプル**: コードが簡潔で読みやすい
- **一貫性**: `pasta.shiori.res` の `dic = dic or {}` パターンと同様
- **Luaらしさ**: 短絡評価と `nil` の扱いを活用
- Luaの仕様: `REG[nil]` は常に `nil` を返すため、自然に `EVENT.no_entry` へフォールバック

**前提条件**:
- Rust側が `req` テーブルを必ず渡す（`nil` を渡さない）
- `req.id` が `nil` でも `EVENT.no_entry(req)` で適切に処理される（204 No Content）

---

#### 議題3: エラーレスポンスのtraceback詳細度

**背景**: `xpcall` でキャッチしたエラーを `RES.err(traceback)` で返すが、本番環境でスタックトレースを露出するかの判断が必要。

**選択肢**:

**A. 常にtraceback全体を返す（シンプル、推奨）**
```lua
if ok then
    return result
else
    return RES.err(result)  -- result = debug.traceback()
end
```
- 利点: シンプル、デバッグ容易、初期実装に最適
- 欠点: 本番環境で内部情報が露出（ただしローカル環境のみで使用想定）

**B. debug_mode フラグで制御（将来拡張）**
```lua
local traceback = result
if not RES.env.debug_mode then
    traceback = "Internal Server Error (details hidden)"
end
return RES.err(traceback)
```
- 利点: 本番環境での情報制御が可能
- 欠点: 初期実装としては過剰設計、RES.envへの拡張が必要

**C. エラーメッセージのみ抽出**
```lua
local error_msg = result:match("^[^\n]+") or "Unknown error"
return RES.err(error_msg)
```
- 利点: 情報露出を最小化
- 欠点: デバッグ困難、開発体験悪化

**推奨**: Option A（常にtraceback全体を返す）
- 理由: SHIORIはローカル環境で動作、デバッグ容易性を優先
- 将来必要ならOption Bへ拡張可能

**決定**: （開発者確認必要）

---

### 5.3 研究項目（削除済み）

設計判断事項（5.2）に統合しました。以下の項目は議題として扱います：
- main.lua統合方法 → 議題1
- req.id防御方針 → 議題2
- エラーレスポンス詳細度 → 議題3

---

## 6. 要件→資産マッピング（Requirement-to-Asset Map）

| 要件ID | 要件内容 | 既存資産 | ギャップ | 対応 |
|--------|---------|---------|---------|------|
| Req 1 | register.lua構造 | なし | **Missing** | 新規作成 |
| Req 2 | init.lua構造 | なし | **Missing** | 新規作成 |
| Req 3 | no_entry実装 | RES.no_content | - | init.lua内で実装 |
| Req 4 | fire実装 | req.id構造確認済み | **Missing** | init.lua内で実装 |
| Req 5 | xpcallエラー処理 | RES.err | - | init.lua内で実装 |
| Req 6 | ハンドラシグネチャ | なし | **Missing** | ドキュメント追加 |
| Req 7 | reqテーブル構造 | lua_request.rs | ✅ | 完備 |
| Req 8 | 公開API | モジュールパターン | - | 規約適用 |
| Req 9 | main.lua統合 | SHIORI.request() | **Constraint** | 関数修正 |

---

## 7. 結論

### 分析サマリー

1. **既存資産**: `pasta.shiori.res`（完成）、Rust側リクエストパース（完成）
2. **ギャップ**: 新規ファイル2つ + 既存1ファイル修正 + テスト1ファイル
3. **推奨**: Option B（新規サブモジュール作成）
4. **工数**: S（1-3日）
5. **リスク**: Low（確立されたパターン、影響範囲限定）

### 次ステップ

設計フェーズで以下を決定：
- main.lua統合の詳細シーケンス
- エラーレスポンスの詳細度制御方針
- テストケース網羅性の確認

**コマンド**:
```bash
/kiro-spec-design shiori-event-module
```

---

**分析完了日**: 2026-01-27  
**分析者**: AI開発支援エージェント
