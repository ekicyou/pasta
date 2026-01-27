# Requirements Document: shiori-entry

## Introduction

本仕様は、Rust側の SHIORI.DLL 実装（pasta_shioriクレート）から呼び出される Lua エントリーポイントモジュール `pasta.shiori.entry` を定義する。

**背景**:
- 「伺か」ゴースト基盤として、SHIORI/3.0 プロトコルを Lua 側で処理する必要がある
- 現行の `pasta.shiori.main` を置き換え、責務を明確化した新エントリーポイントを提供
- イベント処理は `pasta.shiori.event` モジュールに委譲し、単一責任原則を遵守

**スコープ**:
- `entry.lua` の新規作成
- グローバル `SHIORI` テーブルの初期化
- `SHIORI.load` / `SHIORI.request` / `SHIORI.unload` の実装
- Rust側の require 対象変更（必要な場合）
- 既存 `pasta.shiori.main` の削除または空ファイル化

---

## Requirements

### Requirement 1: グローバル SHIORI テーブルの初期化

**Objective:** ゴースト開発者として、SHIORI/3.0 プロトコルハンドラが自動的に初期化されることで、追加設定なしでイベント処理を開始できる。

#### Acceptance Criteria

1. When `require("pasta.shiori.entry")` が実行された場合、the entry module shall グローバルテーブル `SHIORI` を初期化する。
2. If グローバル `SHIORI` がすでに存在する場合、then the entry module shall 既存テーブルを上書きせずに関数を追加する。
3. The entry module shall `SHIORI.load`、`SHIORI.request`、`SHIORI.unload` の3関数を `SHIORI` テーブルに登録する。
4. The entry module shall `pasta.shiori.event` モジュールを require して `EVENT` テーブルを取得する。

---

### Requirement 2: SHIORI.load によるDLLロード処理

**Objective:** Rust ランタイムとして、DLLロード時に Lua 側の初期化処理を呼び出し、初期化成否を確認できる。

#### Acceptance Criteria

1. When Rust側から `SHIORI.load(hinst, load_dir)` が呼び出された場合、the entry module shall 初期化処理を実行する。
2. The `SHIORI.load` function shall 引数として `hinst`（integer: DLLインスタンスハンドル）と `load_dir`（string: ゴーストの master/ ディレクトリパス）を受け取る。
3. When 初期化が正常に完了した場合、the `SHIORI.load` function shall `true`（boolean）を返却する。
4. If 初期化中にエラーが発生した場合、then the `SHIORI.load` function shall `false`（boolean）を返却する。
5. The `SHIORI.load` function shall 将来の拡張に備えて、設定ファイル読み込み・セーブデータ復元の拡張ポイントを提供する（現時点では未実装）。

---

### Requirement 3: SHIORI.request によるリクエスト処理

**Objective:** Rust ランタイムとして、SHIORI リクエストを Lua 側に渡し、SHIORI/3.0 形式のレスポンス文字列を受け取れる。

**注**: 既存の `main.lua` は `request_text: string` を受け取る古い実装だが、Rust側（`pasta_shiori::shiori.rs:277`）は `lua_request::parse_request()` で**reqテーブル**を生成し、Luaに渡している。本要件はRust側の現在の実装に合わせている。

#### Acceptance Criteria

1. When Rust側から `SHIORI.request(req)` が呼び出された場合、the entry module shall `EVENT.fire(req)` を呼び出す。
2. The `SHIORI.request` function shall 引数 `req` として以下の構造を持つテーブルを受け取る（Rust側 `lua_request.rs` で生成）:
   - `method`: string ("get" または "notify")
   - `version`: integer (30 = SHIORI/3.0)
   - `id`: string (イベントID)
   - `charset`: string (文字セット)
   - `sender`: string (ベースウェア名)
   - `reference`: table (0-indexed の Reference ヘッダー配列)
   - `dic`: table (全ヘッダーの辞書)
3. When `EVENT.fire(req)` がレスポンス文字列を返却した場合、the `SHIORI.request` function shall そのレスポンス文字列をそのまま返却する。
4. The `SHIORI.request` function shall SHIORI/3.0 形式のレスポンス文字列（例: `SHIORI/3.0 200 OK\r\n...`）を返却する。

---

### Requirement 4: SHIORI.unload によるDLLアンロード処理

**Objective:** Rust ランタイムとして、DLLアンロード時に Lua 側のクリーンアップ処理を呼び出せる。

#### Acceptance Criteria

1. When Rust側から `SHIORI.unload()` が呼び出された場合、the entry module shall クリーンアップ処理を実行する。
2. The `SHIORI.unload` function shall 引数を受け取らない。
3. The `SHIORI.unload` function shall 戻り値を返却しない（void）。
4. The `SHIORI.unload` function shall 将来の拡張に備えて、セーブデータ保存・リソース解放の拡張ポイントを提供する（現時点では未実装）。

---

### Requirement 5: 既存モジュールの移行

**Objective:** プロジェクトメンテナーとして、既存の `pasta.shiori.main` から新しい `entry.lua` への移行を完了し、重複コードを排除できる。

#### Acceptance Criteria

1. The entry module shall `crates/pasta_lua/scripts/pasta/shiori/entry.lua` に配置される。
2. When 移行が完了した場合、the project shall `pasta/shiori/main.lua` を削除または空ファイル化する。
3. If Rust側で `require("pasta.shiori.main")` がハードコードされている場合、then the project shall `require("pasta.shiori.entry")` に変更する。
4. The entry module shall `lua-coding.md` のモジュール構造規約（標準モジュール構造、UPPER_CASE テーブル）に従う。

---

### Requirement 6: Rust側との整合性

**Objective:** Rust ランタイムとして、Lua エントリーポイントの変更後も既存の SHIORI 呼び出しフローが正常に動作することを保証できる。

#### Acceptance Criteria

1. The entry module shall Rust側の `pasta_shiori` クレートが期待する `SHIORI.load`、`SHIORI.request`、`SHIORI.unload` のシグネチャと互換性を保つ。
2. When Rust側から SHIORI 関数が呼び出された場合、the entry module shall 既存のテストケース（`shiori_event_test.rs` 等）が通過する動作を維持する。
3. If Rust側でエントリーポイントモジュール名がハードコードされている場合、then the implementation shall その箇所を特定し変更する。

---

### Requirement 7: テスト要件

**Objective:** 開発者として、entry モジュールの動作を自動テストで検証でき、リグレッションを防止できる。

#### Acceptance Criteria

1. The project shall 以下の単体テストを提供する:
   - `require("pasta.shiori.entry")` でグローバル `SHIORI` テーブルが作成される
   - `SHIORI.load(0, "/path/to/ghost")` が `true` を返す
   - `SHIORI.request(valid_req)` が `EVENT.fire` を呼び出しレスポンス文字列を返す
   - `SHIORI.unload()` がエラーなく完了する
2. The project shall 以下の結合テストを提供する:
   - Rust側から `SHIORI.load` 呼び出し → Lua側の `SHIORI.load` が実行される
   - Rust側から `SHIORI.request` 呼び出し → Lua側でレスポンスが生成される
3. When すべてのテストが実行された場合、the project shall `cargo test --all` が成功することを保証する。

---

## Original Project Description

<!-- 以下は初期化フェーズで入力されたプロジェクト説明（参照用） -->

<details>
<summary>元の要件定義書（入力）</summary>

### 目的
Rust側の SHIORI.DLL 実装から呼び出される Lua 側のエントリーポイントを提供する。
グローバル変数 `SHIORI` を初期化し、SHIORI/3.0 プロトコルの load / request / unload を処理する。

### 責務
- グローバルテーブル `SHIORI` の初期化
- `SHIORI.load(hinst, load_dir)` の実装
- `SHIORI.request(req)` の実装（EVENT.fire への委譲）
- `SHIORI.unload()` の実装

### ファイル配置
```
crates/pasta_lua/scripts/pasta/shiori/entry.lua
```

### 既存ファイルとの関係
- `pasta.shiori.main` → **削除または空ファイル化**（entry.lua に置き換え）
- `pasta.shiori.res` → entry.lua から利用（エラーレスポンス生成等）
- `pasta.shiori.event` → entry.lua から利用（EVENT.fire 呼び出し）

### インターフェース仕様

#### SHIORI.load(hinst, load_dir) → boolean
DLLロード時の初期化処理。

#### SHIORI.request(req) → string
リクエスト処理。`EVENT.fire(req)` に委譲。

#### SHIORI.unload() → void
DLLアンロード時のクリーンアップ処理。

</details>
