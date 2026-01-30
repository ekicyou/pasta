# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **pasta.shiori.act モジュールによるさくらスクリプト組み立て機能** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: なし（独立実装可能）
- **目的**: Pasta DSL から生成されたトーク内容を、伺かベースウェアで解釈可能なさくらスクリプト形式に変換する

### さくらスクリプトとは

伺かベースウェア（SSP等）で解釈される表示制御スクリプト。キャラクターの表情変更、テキスト表示、待機等を制御する。

主要タグ例:
- `\0` - メインキャラ（sakura）に切り替え
- `\1` - サブキャラ（kero）に切り替え
- `\s[n]` - サーフェス（表情）変更
- `\w[n]` - n ミリ秒待機
- `\e` - スクリプト終端

### 技術的背景

- **継承元**: `pasta.act` モジュール（`crates/pasta_lua/scripts/pasta/act.lua`）
- **配置場所**: `crates/pasta_lua/scripts/pasta/shiori/act.lua`
- **実装言語**: Lua 5.x（pasta_lua 規約準拠）

---

## Requirements

### Requirement 1: pasta.act 継承

**Objective:** As a 開発者, I want pasta.shiori.act が pasta.act を継承してほしい, so that 既存のactインターフェースと互換性がある

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `require("pasta.act")` により親モジュールを参照する
2. The `pasta.shiori.act` shall `setmetatable` を使用して `pasta.act` の全メソッドを継承する
3. When `pasta.shiori.act` で未定義のメソッドが呼び出された場合, the `SHIORI_ACT_IMPL.__index` shall `pasta.act` から該当メソッドを返す
4. The 継承 shall `talk`, `call`, `word`, `yield`, `end_action`, `init_scene` 等の既存メソッドを利用可能とする

---

### Requirement 2: ShioriAct クラス定義

**Objective:** As a 開発者, I want ShioriAct クラスを new で生成したい, so that 複数のactインスタンスを管理できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `SHIORI_ACT.new(ctx)` コンストラクタを提供する
2. When `new` が呼び出された場合, the `SHIORI_ACT` shall 以下を持つ新規インスタンスを返す:
   - `ctx`: 環境オブジェクト（引数から）
   - `var`: アクションローカル変数テーブル（空テーブル）
   - `token`: トークンバッファ（空テーブル）
   - `_buffer`: さくらスクリプトバッファ（空テーブル）
   - `now_actor`: 現在のアクター（nil）
   - `current_scene`: 現在のシーンテーブル（nil）
3. The インスタンス shall メタテーブル `SHIORI_ACT_IMPL` を設定する
4. The `SHIORI_ACT_IMPL.__index` shall 自身のメソッドを優先し、未定義なら `pasta.act` のメソッドを返す

---

### Requirement 3: キャラクター切り替えタグ生成

**Objective:** As a ゴースト開発者, I want キャラクターを切り替えるタグを生成したい, so that sakura/kero の発話を制御できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:sakura()` メソッドを提供する
2. When `sakura()` が呼び出された場合, the メソッド shall 内部バッファに `\0` を追加する
3. The `pasta.shiori.act` shall `act:kero()` メソッドを提供する
4. When `kero()` が呼び出された場合, the メソッド shall 内部バッファに `\1` を追加する
5. The `pasta.shiori.act` shall `act:char(n)` メソッドを提供する
6. When `char(n)` が呼び出された場合, the メソッド shall 内部バッファに `\p[n]` を追加する
7. The 各メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 4: サーフェス変更タグ生成

**Objective:** As a ゴースト開発者, I want キャラクターの表情を変更したい, so that シェルの表示を制御できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:surface(id)` メソッドを提供する
2. When `surface(id)` が呼び出された場合, the メソッド shall 内部バッファに `\s[id]` を追加する
3. If `id` が数値の場合, the メソッド shall 数値をそのまま使用する
4. If `id` が文字列の場合, the メソッド shall 文字列をそのまま使用する（エイリアス対応）
5. The メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 5: 待機・タイミング制御タグ生成

**Objective:** As a ゴースト開発者, I want テキスト表示のタイミングを制御したい, so that 自然な会話の間を演出できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:wait(ms)` メソッドを提供する
2. When `wait(ms)` が呼び出された場合, the メソッド shall 内部バッファに `\w[ms]` を追加する
3. The `pasta.shiori.act` shall `act:newline()` メソッドを提供する
4. When `newline()` が呼び出された場合, the メソッド shall 内部バッファに `\n` を追加する
5. The `pasta.shiori.act` shall `act:clear()` メソッドを提供する
6. When `clear()` が呼び出された場合, the メソッド shall 内部バッファに `\c` を追加する
7. The 各メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 6: テキスト追加

**Objective:** As a ゴースト開発者, I want テキストをさくらスクリプトに追加したい, so that 会話内容を組み立てられる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:text(str)` メソッドを提供する
2. When `text(str)` が呼び出された場合, the メソッド shall 内部バッファにテキストを追加する
3. The `text` shall バックスラッシュ `\` を `\\` にエスケープする
4. The `text` shall パーセント `%` を `%%` にエスケープする（ベースウェア依存）
5. The メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 7: 生タグ追加

**Objective:** As a 上級ゴースト開発者, I want 任意のさくらスクリプトタグを直接追加したい, so that 拡張タグを使用できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:raw(tag)` メソッドを提供する
2. When `raw(tag)` が呼び出された場合, the メソッド shall 内部バッファにタグをエスケープなしで追加する
3. The メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 8: build メソッド

**Objective:** As a 開発者, I want 組み立てたさくらスクリプトを文字列として取得したい, so that SHIORIレスポンスに含められる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:build()` メソッドを提供する
2. When `build()` が呼び出された場合, the メソッド shall 内部バッファの内容を連結した文字列を返す
3. The `build` shall 末尾に `\e`（スクリプト終端）を自動追加する
4. The `build` shall 呼び出し後も内部バッファをリセットしない（複数回呼び出し可能）
5. If 内部バッファが空の場合, the `build` shall `\e` のみを返す

---

### Requirement 9: reset メソッド

**Objective:** As a 開発者, I want 内部バッファをリセットしたい, so that 新しいスクリプトを組み立てられる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:reset()` メソッドを提供する
2. When `reset()` が呼び出された場合, the メソッド shall 内部バッファ（`_buffer`）を空テーブルにリセットする
3. The `reset` shall `token` バッファはリセットしない（pasta.act 互換性維持）
4. The メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 10: モジュール構造

**Objective:** As a 開発者, I want モジュールが pasta_lua の規約に準拠してほしい, so that 他のモジュールと整合性がある

#### Acceptance Criteria

1. The モジュールファイル shall `crates/pasta_lua/scripts/pasta/shiori/act.lua` に配置する
2. The モジュール shall `--- @module pasta.shiori.act` でドキュメント化する
3. The モジュールテーブル shall `SHIORI_ACT`（UPPER_CASE）で定義する
4. The 実装メタテーブル shall `SHIORI_ACT_IMPL`（`_IMPL` サフィックス）で定義する
5. The モジュール shall 末尾で `return SHIORI_ACT` を返す

---

### Requirement 11: テスト要件

**Objective:** As a 開発者, I want さくらスクリプト生成のテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The テストファイル shall `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua` に配置する
2. The テスト shall 各タグ生成メソッドの出力を検証する
3. The テスト shall メソッドチェーンの動作を検証する
4. The テスト shall エスケープ処理の正確性を検証する
5. The テスト shall `build()` の出力が期待どおりであることを検証する（`\e` 終端付与を含む）
6. The テスト shall `pasta.act` からの継承が正しく動作することを検証する

---

## Out of Scope

- SHIORI EVENT ハンドラ（alpha01 で実装）
- 仮想イベント発行（alpha02 で実装）
- 高度なさくらスクリプト機能（バルーン制御、アニメーション等）

---

## Glossary

| 用語 | 説明 |
|------|------|
| pasta.act | Pasta DSL 実行時の基本 act クラス |
| pasta.shiori.act | pasta.act を継承した SHIORI 専用 act クラス |
| さくらスクリプト | 伺かベースウェアで解釈される表示制御スクリプト |
| サーフェス | キャラクターの表情画像（surface0, surface1, ...） |
| スコープ | 0=メインキャラ(sakura), 1=サブキャラ(kero) |
