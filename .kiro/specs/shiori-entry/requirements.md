# Requirements Document

## Project Description (Input)
# pasta.shiori.entry 要件定義書

## 1. 概要

### 1.1 目的
Rust側の SHIORI.DLL 実装から呼び出される Lua 側のエントリーポイントを提供する。
グローバル変数 `SHIORI` を初期化し、SHIORI/3.0 プロトコルの load / request / unload を処理する。

### 1.2 責務
- グローバルテーブル `SHIORI` の初期化
- `SHIORI.load(hinst, load_dir)` の実装
- `SHIORI.request(req)` の実装（EVENT.fire への委譲）
- `SHIORI.unload()` の実装

### 1.3 ファイル配置
```
crates/pasta_lua/scripts/pasta/shiori/entry.lua
```

### 1.4 既存ファイルとの関係
- `pasta.shiori.main` → **削除または空ファイル化**（entry.lua に置き換え）
- `pasta.shiori.res` → entry.lua から利用（エラーレスポンス生成等）
- `pasta.shiori.event` → entry.lua から利用（EVENT.fire 呼び出し）

---

## 2. インターフェース仕様

### 2.1 グローバル変数

| 変数名 | 型 | 説明 |
|--------|------|------|
| `SHIORI` | table | SHIORI/3.0 プロトコルハンドラを格納するグローバルテーブル |

### 2.2 SHIORI.load(hinst, load_dir)

Rust側から DLL ロード時に呼び出される。

**シグネチャ:**
```lua
function SHIORI.load(hinst, load_dir) -> boolean
```

**引数:**
| 名前 | 型 | 説明 |
|------|------|------|
| `hinst` | integer | DLL インスタンスハンドル |
| `load_dir` | string | ゴーストの master/ ディレクトリパス |

**戻り値:**
| 型 | 説明 |
|------|------|
| `boolean` | 初期化成功時 `true`、失敗時 `false` |

**処理内容:**
1. 必要に応じて初期化処理を行う（将来の拡張ポイント）
2. `true` を返す

### 2.3 SHIORI.request(req)

Rust側から SHIORI リクエスト受信時に呼び出される。

**シグネチャ:**
```lua
function SHIORI.request(req) -> string
```

**引数:**
| 名前 | 型 | 説明 |
|------|------|------|
| `req` | table | パース済み SHIORI リクエスト |

**req テーブル構造:**
```lua
req = {
    method = "get" | "notify",  -- リクエストメソッド
    version = 30,               -- SHIORIバージョン (30 = SHIORI/3.0)
    id = "OnBoot",              -- イベントID
    charset = "UTF-8",          -- 文字セット
    sender = "SSP",             -- 送信元（ベースウェア名）
    reference = {               -- Reference ヘッダー（0-indexed）
        [0] = "value0",
        [1] = "value1",
        -- ...
    },
    dic = {                     -- 全ヘッダーの辞書
        ["ID"] = "OnBoot",
        ["Charset"] = "UTF-8",
        -- ...
    },
    -- 以下はオプション（存在する場合のみ）
    base_id = "...",            -- Base ID
    status = "...",             -- Status
    security_level = "...",     -- Security Level
}
```

**戻り値:**
| 型 | 説明 |
|------|------|
| `string` | SHIORI/3.0 レスポンス文字列 |

**処理内容:**
1. `EVENT.fire(req)` を呼び出す
2. 戻り値（レスポンス文字列）をそのまま返す

### 2.4 SHIORI.unload()

Rust側から DLL アンロード時に呼び出される。

**シグネチャ:**
```lua
function SHIORI.unload() -> void
```

**引数:** なし

**戻り値:** なし

**処理内容:**
1. 必要に応じてクリーンアップ処理を行う（将来の拡張ポイント）

---

## 3. 依存モジュール

| モジュール | 用途 |
|------------|------|
| `pasta.shiori.event` | `EVENT.fire(req)` の呼び出し |
| `pasta.shiori.res` | エラーレスポンス生成（将来の拡張用） |

---

## 4. 実装例
```lua
--- @module pasta.shiori.entry
--- SHIORI/3.0 プロトコル エントリーポイント
---
--- Rust側の PastaShiori から呼び出されるグローバル SHIORI テーブルを初期化する。
--- load / request / unload の各関数を提供し、イベント処理は EVENT モジュールに委譲する。

-- 1. 依存モジュールの読み込み
local event = require("pasta.shiori.event")
-- local res = require("pasta.shiori.res")  -- 将来の拡張用

-- 2. グローバル SHIORI テーブルの初期化
SHIORI = SHIORI or {}

-- 3. SHIORI.load
--- DLLロード時の初期化処理
--- @param hinst integer DLLインスタンスハンドル
--- @param load_dir string ゴーストのmaster/ディレクトリパス
--- @return boolean 初期化成功時true
function SHIORI.load(hinst, load_dir)
    -- 将来の拡張ポイント:
    -- - 設定ファイルの読み込み
    -- - セーブデータの復元
    -- - アクター定義の読み込み
    return true
end

-- 4. SHIORI.request
--- SHIORIリクエストの処理
--- @param req table パース済みSHIORIリクエスト
--- @return string SHIORI/3.0レスポンス文字列
function SHIORI.request(req)
    return event.fire(req)
end

-- 5. SHIORI.unload
--- DLLアンロード時のクリーンアップ処理
function SHIORI.unload()
    -- 将来の拡張ポイント:
    -- - セーブデータの保存
    -- - リソースの解放
end

-- 6. モジュールとしても返す（オプション）
return SHIORI
```

---

## 5. Rust側の変更点

### 5.1 PastaLoader での読み込み対象変更

現在 `pasta.shiori.main` を読み込んでいる箇所を `pasta.shiori.entry` に変更する。

**確認事項:**
- Rust側でモジュール名をハードコードしている箇所があるか？
- それとも `SHIORI` グローバル変数の存在だけをチェックしているか？

### 5.2 想定される変更なし（確認待ち）

Rust側は `SHIORI.load` / `SHIORI.request` / `SHIORI.unload` の存在をチェックしてキャッシュしているため、Lua側のモジュール名が変わっても影響しない可能性が高い。

ただし、**どこで `require("pasta.shiori.main")` を呼んでいるか**を確認する必要がある。

---

## 6. テスト観点

### 6.1 単体テスト

| テストケース | 期待結果 |
|--------------|----------|
| `require("pasta.shiori.entry")` | グローバル `SHIORI` テーブルが作成される |
| `SHIORI.load(0, "/path/to/ghost")` | `true` が返る |
| `SHIORI.request(valid_req)` | `EVENT.fire` が呼ばれ、レスポンス文字列が返る |
| `SHIORI.unload()` | エラーなく完了する |

### 6.2 結合テスト

| テストケース | 期待結果 |
|--------------|----------|
| Rust側から SHIORI.load 呼び出し | Lua側の SHIORI.load が実行される |
| Rust側から SHIORI.request 呼び出し | Lua側の SHIORI.request が実行され、レスポンスが返る |
| OnBoot イベント発火 | `EVENT.fire` → ハンドラ実行 → `RES.ok(...)` 形式のレスポンス |

### 6.3 E2Eテスト（伺か上での動作確認）

| テストケース | 期待結果 |
|--------------|----------|
| ゴースト起動 | 「こんにちは」と喋る |

---

## 7. 今後の拡張ポイント

### 7.1 SHIORI.load での初期化処理
- 設定ファイル（config.lua 等）の読み込み
- pasta スクリプト（*.pasta）のトランスパイル結果読み込み
- セーブデータの復元

### 7.2 SHIORI.unload でのクリーンアップ処理
- セーブデータの永続化
- ログのフラッシュ

### 7.3 エラーハンドリングの強化
- `EVENT.fire` がエラーを投げた場合の 500 レスポンス生成
- デバッグ情報の X-Error-Reason ヘッダー付与

---

## 8. チェックリスト

- [ ] `pasta/shiori/entry.lua` を新規作成
- [ ] `pasta.shiori.event` の実装完了を確認（AIさん担当）
- [ ] Rust側の require 対象を確認・必要なら変更
- [ ] `pasta/shiori/main.lua` を削除または空ファイル化
- [ ] ユニットテスト作成
- [ ] 伺かでの動作確認

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
