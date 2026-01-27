# Gap Analysis: shiori-res-module

## 分析サマリー

- **スコープ**: SHIORI/3.0レスポンス組み立てユーティリティ（`pasta.shiori.res`）を新規作成
- **主な課題**: 既存の `pasta.shiori` は空実装であり、レスポンス構築機能なし。`main.lua` にハードコードされたレスポンス生成ロジックを抽出・汎用化する必要あり
- **推奨アプローチ**: Option B（新規モジュール作成）— 明確な責務分離と既存コードの保守性向上
- **実装工数**: S（1-3日）— 単純なstring組み立てロジック、外部依存なし、テスト容易

---

## 1. 現状調査

### 1.1 関連アセット

| ファイル | 役割 | 現状 |
|---------|------|------|
| `crates/pasta_lua/scripts/pasta/shiori/init.lua` | `pasta.shiori` モジュール | 空実装（TODOコメントのみ） |
| `crates/pasta_lua/scripts/pasta/shiori/main.lua` | SHIORI/3.0エントリーポイント | ハードコードされた204レスポンス生成 |
| `.kiro/specs/shiori-res-module/response.lua` | 旧参考実装 | `insert_wait`, `time_ok`, `talk` 等を含む |

**既存の `main.lua` レスポンス生成**:
```lua
function SHIORI.request(request_text)
    -- ハードコードされた204レスポンス
    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Sender: Pasta\r\n" ..
        "\r\n"
end
```

### 1.2 既存パターン・規約

**モジュール構造パターン**（`word.lua`, `actor.lua`, `scene.lua` 等）:
```lua
--- @module pasta.<module>
--- モジュール説明

local DEPENDENCY = require("pasta.dependency")  -- 依存モジュール

local MODULE = {}  -- UPPER_CASE

-- ローカル関数

-- 公開API

return MODULE
```

**命名規約**:
- モジュールテーブル: `UPPER_CASE`（例: `WORD`, `ACTOR`, `SCENE`）
- 公開関数: `snake_case`
- プライベート関数: `local function`

**依存関係ルール**:
- `pasta.store` は他モジュールを `require` しない（循環参照回避）
- 他モジュールは `pasta.store` を `require` 可能
- `pasta.shiori.res` → 依存なし（ユーティリティ）

### 1.3 統合ポイント

**将来の統合先**:
- `pasta.shiori.main.SHIORI.request()` で `RES.no_content()` を使用
- ユーザースクリプト内で `local RES = require("pasta.shiori.res")` で直接利用

**データフロー**:
```
SHIORI.request(request_text)
  ↓
[リクエスト処理ロジック（将来実装）]
  ↓
RES.ok(value, dic) / RES.no_content(dic) / RES.err(reason)
  ↓
SHIORI/3.0レスポンス文字列
```

---

## 2. 要件実現性分析

### 2.1 技術要件マッピング

| 要件 | 必要機能 | 既存アセット | ギャップ |
|------|---------|------------|---------|
| Req 1: モジュール構造 | Lua module, LuaDoc | 既存パターン（word.lua等） | **Missing** — 新規ファイル作成 |
| Req 2: 環境設定テーブル | `RES.env` table | なし | **Missing** — テーブル定義 |
| Req 3: 汎用ビルダー | `RES.build(code, dic)` | `main.lua` ハードコード | **Missing** — 関数実装 |
| Req 4-8: ステータス別関数 | `ok`, `no_content`, `err` 等 | `response.lua` 参考実装あり | **Missing** — 関数実装（typo修正） |

### 2.2 制約・課題

| 制約 | 内容 | 対応策 |
|------|------|-------|
| 循環参照回避 | `pasta.store` を require しない設計 | ✅ `RES` は依存なし（ユーティリティ） |
| CRLF制御 | Windows改行コード `\r\n` | ✅ Luaエスケープシーケンス使用 |
| 文字列連結パフォーマンス | レスポンス生成の効率 | ✅ 小規模文字列、最適化不要 |

### 2.3 複雑度シグナル

- **単純なロジック**: String concatenation + table iteration
- **外部依存なし**: Pure Lua実装
- **テスト容易**: 入力（code, dic）→ 出力（string）の純粋関数

---

## 3. 実装アプローチ選択肢

### Option A: `pasta.shiori.init.lua` を拡張

**概要**: 既存の `pasta.shiori` モジュールにレスポンス構築関数を追加

**実装詳細**:
```lua
-- pasta/shiori/init.lua
local SHIORI = {}

SHIORI.env = { charset = "UTF-8", sender = "Pasta", security_level = "local" }

function SHIORI.build(code, dic) ... end
function SHIORI.ok(value, dic) ... end
-- ...

return SHIORI
```

**統合ポイント**:
- `main.lua` で `local SHIORI = require("pasta.shiori")` → `SHIORI.no_content()`

**トレードオフ**:
- ✅ ファイル数削減（1ファイルで完結）
- ✅ `pasta.shiori` 名前空間統一
- ❌ 単一ファイルが肥大化（将来のリクエスト解析機能追加時）
- ❌ `pasta.shiori.res` という明示的なモジュール名が使えない

---

### Option B: `pasta.shiori.res` 新規作成 ⭐ **推奨**

**概要**: `crates/pasta_lua/scripts/pasta/shiori/res.lua` を新規作成

**実装詳細**:
```lua
-- pasta/shiori/res.lua
--- @module pasta.shiori.res
local RES = {}

RES.env = {
    charset = "UTF-8",
    sender = "Pasta",
    security_level = "local",
}

function RES.build(code, dic) ... end
function RES.ok(value, dic) ... end
function RES.no_content(dic) ... end
function RES.not_enough(dic) ... end
function RES.advice(dic) ... end
function RES.bad_request(dic) ... end
function RES.err(reason, dic) ... end
function RES.warn(reason, dic) ... end

return RES
```

**統合ポイント**:
- `main.lua` で `local RES = require("pasta.shiori.res")` → `RES.no_content()`
- ユーザースクリプトで直接 `require("pasta.shiori.res")`

**トレードオフ**:
- ✅ **明確な責務分離**（レスポンス構築専用モジュール）
- ✅ 既存の `pasta.shiori` を汚染しない
- ✅ 将来の拡張が容易（`pasta.shiori.req` リクエスト解析モジュール等）
- ✅ テスト独立性（`res.lua` 単体テスト可能）
- ❌ ファイル数増加（微小）

---

### Option C: ハイブリッドアプローチ

**概要**: `pasta.shiori.res` を新規作成し、`pasta.shiori.init` から再エクスポート

**実装詳細**:
```lua
-- pasta/shiori/res.lua
local RES = {}
-- ... (Option B と同じ)
return RES

-- pasta/shiori/init.lua
local SHIORI = {}
SHIORI.res = require("pasta.shiori.res")
return SHIORI
```

**使用例**:
```lua
local SHIORI = require("pasta.shiori")
return SHIORI.res.ok("Hello")  -- pasta.shiori経由

-- または
local RES = require("pasta.shiori.res")
return RES.ok("Hello")  -- 直接アクセス
```

**トレードオフ**:
- ✅ 柔軟なアクセス方法（両方サポート）
- ✅ `pasta.shiori` 名前空間の整理
- ❌ 複雑度増加（2つのアクセスパス）
- ❌ 現時点では過剰設計

---

## 4. 実装工数・リスク評価

### 工数見積もり: **S（1-3日）**

| フェーズ | 工数 | 根拠 |
|---------|------|------|
| ファイル作成 | 0.5日 | 150行程度（参考実装から単純化） |
| ユニットテスト | 1日 | 8関数 × テストケース（正常系・異常系） |
| 統合テスト | 0.5日 | `main.lua` での動作確認 |
| ドキュメント | 0.5日 | LuaDoc、使用例 |

### リスク評価: **Low**

| リスク要因 | 評価 | 対策 |
|----------|------|------|
| 技術的複雑度 | Low | 単純なstring操作、外部依存なし |
| 統合複雑度 | Low | `main.lua` 1箇所の変更のみ |
| パフォーマンス | Low | レスポンス生成は非頻繁処理 |
| 互換性 | None | 新規モジュール、既存コードへの影響なし |

---

## 5. 設計フェーズへの推奨事項

### 5.1 推奨アプローチ

**Option B（新規モジュール作成）** を推奨いたしますわ。

**理由**:
1. **責務の明確性**: レスポンス構築機能を独立したモジュールに分離
2. **将来の拡張性**: `pasta.shiori.req`（リクエスト解析）等の追加が容易
3. **テスト容易性**: 単体テスト・統合テストが明確に分離
4. **既存コードへの影響最小**: 新規ファイル作成のみ、リグレッションリスクなし

### 5.2 設計フェーズで決定すべき項目

| 項目 | 選択肢 | 推奨 |
|------|-------|------|
| ファイル配置 | `pasta/shiori/res.lua` | ✅ |
| モジュールテーブル名 | `RES` / `MOD` | ✅ `RES`（明示的） |
| `env` 変更API | 直接アクセス / setter関数 | ✅ 直接アクセス（`RES.env.charset = "..."`) |
| エラーハンドリング | `assert()` / `error()` / silent | ✅ silent（`dic = dic or {}`) |

### 5.3 リサーチ不要項目

- ✅ SHIORI/3.0仕様は明確（既存の `response.lua` で実証済み）
- ✅ Lua string操作は標準機能で実現可能
- ✅ モジュール構造パターンは既存コードで確立済み

---

## 6. 要件-アセットマップ

| 要件 | 既存アセット | ギャップ | 実装方法 |
|------|------------|---------|---------|
| **Req 1**: モジュール構造 | `word.lua` パターン | **Missing** | 新規 `res.lua` 作成、LuaDoc追加 |
| **Req 2**: `RES.env` | なし | **Missing** | テーブル定義（3フィールド） |
| **Req 3**: `RES.build()` | `main.lua` ハードコード | **Missing** | 関数実装（string連結 + iteration） |
| **Req 4**: `RES.ok()` | `response.lua` 参考実装 | **Missing** | `dic["Value"] = value` + `build()` |
| **Req 5**: `RES.no_content()` | `main.lua` 参考 | **Missing** | `build("204 No Content", dic)` |
| **Req 6**: TEACH関数 | `response.lua` 参考実装 | **Missing** | `build("311/312", dic)` |
| **Req 7**: エラー関数 | `response.lua` 参考実装 | **Missing** | `dic["X-Error-Reason"]` + `build()` |
| **Req 8**: `RES.warn()` | `response.lua` 参考実装 | **Missing** | `dic["X-Warn-Reason"]` + `no_content()` |

**凡例**:
- **Missing**: 実装なし
- **OK**: 既存実装あり
- **Partial**: 部分的実装あり

---

## 7. 既存コードベースへの影響評価

### 7.1 変更が必要なファイル

| ファイル | 変更内容 | 影響度 |
|---------|---------|-------|
| `pasta/shiori/main.lua` | `RES.no_content()` 使用 | **Low** — 1箇所のみ |
| `pasta/shiori/res.lua` | **新規作成** | None — 既存コードに影響なし |

### 7.2 後方互換性

- ✅ **完全互換**: 既存コードへの変更なし（`main.lua` の変更はオプショナル）
- ✅ 新規モジュール追加のみ、既存機能に影響なし

---

## 8. 参考実装分析（response.lua）

### 8.1 採用する機能

- ✅ `build(code, dic)` — 汎用ビルダー
- ✅ `ok(value, dic)` — 200 OK
- ✅ `no_content(dic)` — 204 No Content
- ✅ `not_enough(dic)`, `advice(dic)` — TEACH関連
- ✅ `bad_request(dic)`, `err(reason, dic)` — エラーレスポンス
- ✅ `warn(reason, dic)` — ワーニング付き204

### 8.2 除外する機能（スコープ外）

| 機能 | 除外理由 |
|------|---------|
| `insert_wait` | 会話処理モジュールで対応 |
| `time_ok` | ステートレス設計のため除外 |
| `talk()` | ウェイト処理依存、別モジュールで対応 |
| `join()` | 用途不明、旧実装の残骸 |
| `set_char_set()` 等 | `RES.env` 直接アクセスで代替 |

### 8.3 修正が必要な箇所

- ❌ `X-Error-Resion` → ✅ `X-Error-Reason`（typo修正）
- ❌ `X-Warn-Resion` → ✅ `X-Warn-Reason`（typo修正）

---

## まとめ

### スコープ
SHIORI/3.0レスポンス組み立てユーティリティ（`pasta.shiori.res`）を**新規作成**

### 主な課題
- 既存の `pasta.shiori` は空実装、レスポンス構築機能なし
- `main.lua` のハードコードされたレスポンス生成を汎用化

### 推奨アプローチ
**Option B（新規モジュール作成）** — 明確な責務分離、将来の拡張性

### 実装工数
**S（1-3日）** — 単純なstring操作、外部依存なし、テスト容易

### リスク
**Low** — 新規モジュール、既存コードへの影響最小、技術的複雑度低
