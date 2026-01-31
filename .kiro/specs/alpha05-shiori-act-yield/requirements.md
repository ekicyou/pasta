# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **pasta.shiori.act モジュールのyield制御・設定管理・総合テスト機能** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha03-shiori-act-sakura（completed）- 基本さくらスクリプト生成
- **目的**: SHIORI会話フローにおけるyield制御、スコープ切り替え時の改行設定、総合テストを実現

### 技術的背景

- **対象モジュール**: `pasta.shiori.act`（`crates/pasta_lua/scripts/pasta/shiori/act.lua`）
- **継承元**: `pasta.act`（`crates/pasta_lua/scripts/pasta/act.lua`）
- **設定ファイル**: `pasta.toml`（`[ghost]`スコープ）
- **テストファイル**: `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua`

### 既存実装状況（alpha03完了時点）

以下のメソッドは実装済み：
- `SHIORI_ACT.new(actors)` - コンストラクタ
- `act:talk(actor, text)` - スコープ切り替え + テキスト追加
- `act:surface(id)` - サーフェス変更タグ
- `act:wait(ms)` - 待機タグ
- `act:newline(n)` - 改行タグ
- `act:clear()` - クリアタグ
- `act:build()` - さくらスクリプト文字列生成（`\e`終端付与）
- `act:reset()` - バッファ・スコープリセット

継承元（`pasta.act`）から利用可能：
- `act:yield()` - トークン出力（`coroutine.yield`呼び出し）
- `act:end_action()` - アクション終了
- `act:init_scene(scene)` - シーン初期化
- `act:call(...)` - シーン呼び出し
- `act:word(name)` - 単語検索
- `act.sakura:talk(text)` - アクタープロキシ

---

## Requirements

### Requirement 1: SHIORI_ACT用yieldメソッド

**Objective:** As a ゴースト開発者, I want `act:yield()`でさくらスクリプトをビルドしてyieldしたい, so that 会話の途中で一度確定してベースウェアに送信できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:yield()` メソッドをオーバーライドする
2. When `yield()` が呼び出された場合, the メソッド shall 以下を順番に実行する:
   - `local rc = coroutine.yield(self:build())` でスクリプトをyield（build内で自動リセット）
   - `self._resume_value = rc` でレジューム値を保存
   - `return rc` でレジューム値を返す
3. The `_resume_value` フィールド shall yieldから戻った値を保持し、次回アクセス可能とする
4. If `yield()` がコルーチン外で呼び出された場合, the メソッド shall エラーを発生させる（Luaネイティブエラー）
5. The 既存の `ACT_IMPL.yield()` との互換性 shall トークンベースではなく、さくらスクリプト文字列をyieldする点が異なる

### Requirement 1.1: build()メソッドの自動リセット

**Objective:** As a 開発者, I want `act:build()`でスクリプト構築後に自動リセットしたい, so that 次のスクリプト構築がクリーンな状態で始まる

#### 設計意図（チェイントーク対応）

- **チェイントーク**: 1回の会話終了後、時間経過後に続けて続きの会話を行う制御
- 各さくらスクリプトは**完全に終了**（`\e`終端）
- 次の会話は**初めからセッティングやり直し**（スコープタグ再出力含む）
- yield後に同じアクターから継続しても、スコープタグ（`\0`等）が再出力されるのは意図通り

#### Acceptance Criteria

1. The `build()` メソッド shall さくらスクリプト文字列を構築して返す（既存動作）
2. The `build()` メソッド shall 文字列返却前に `self:reset()` を呼び出してバッファをリセットする
3. The 設計方針 shall 「build = 構築して吐き出す（副作用込み）」とする
4. The リセット後 shall `_current_scope = nil` となり、次のtalk()でスコープタグが再出力される
5. The 既存テスト shall build()後のバッファ状態検証を更新する（空になることを期待）

---

### Requirement 2: init_script メソッド

**Objective:** As a 開発者, I want `act:init_script()`でさくらスクリプト組み立て状態を初期化したい, so that コンストラクタや手動リセット時に明確に初期化できる

#### Acceptance Criteria

1. The `pasta.shiori.act` shall `act:init_script()` メソッドを提供する
2. When `init_script()` が呼び出された場合, the メソッド shall 以下を実行する:
   - `self._buffer = {}` でバッファを空にする
   - `self._current_scope = nil` でスコープをリセット
   - `self._resume_value = nil` でレジューム値をリセット
3. The メソッド shall メソッドチェーン可能（`return self`）とする
4. The `SHIORI_ACT.new()` コンストラクタ shall 内部で `init_script()` を呼び出して初期化を統一する
5. The 用途 shall コンストラクタでの初期化、および手動での完全リセット（`_resume_value`含む）

**Note:** `build()` は自動で `reset()` を呼ぶため、通常のyield後リセットは不要。`init_script()` は `_resume_value` も含む完全初期化が必要な場合に使用。

---

### Requirement 3: pasta.toml スコープ切り替え改行設定

**Objective:** As a ゴースト開発者, I want スコープ切り替え時の改行数を設定ファイルで制御したい, so that ゴーストごとに表示スタイルをカスタマイズできる

#### 段落区切り改行の発生条件

- スコープを持った「最初の」発言では改行不要
- 相手にスコープが移り、相手が発言後、スコープが戻ってきた時に段落区切りが発生
- `scope_switch_newlines` はこの段落区切り改行の数を制御

**Note:** テキスト後の改行（`newline()`メソッド呼び出し）はスクリプト作成者が明示的に制御するものであり、本設定の対象外。

#### Acceptance Criteria

1. The `pasta.toml` shall `[ghost]` セクションをサポートする
2. The `[ghost]` セクション shall `scope_switch_newlines` 設定を持つ:
   - 型: 整数
   - デフォルト: 1
   - 意味: スコープ復帰時（actor変更時、2回目以降）に挿入する段落区切り`\n`の数
3. When `scope_switch_newlines = 0` の場合, the `talk()` メソッド shall スコープ復帰時に改行を挿入しない
4. When `scope_switch_newlines = 2` の場合, the `talk()` メソッド shall スコープ復帰時に`\n\n`を挿入する
5. The 設定読み込み shall Luaモジュール `pasta.config` 経由で行う
6. The `SHIORI_ACT.new()` shall 設定を読み込み、インスタンスに保持する（`self._scope_switch_newlines`）
7. If 設定ファイルが存在しないまたは設定未定義の場合, the デフォルト値（1）を使用する

---

### Requirement 4: pasta.config モジュール

**Objective:** As a 開発者, I want pasta.toml設定をLuaから読み取りたい, so that ランタイム設定を動的に取得できる

#### Acceptance Criteria

1. The `pasta.config` モジュール shall `crates/pasta_lua/scripts/pasta/config.lua` に配置する
2. The モジュール shall `PASTA_CONFIG.get(section, key, default)` メソッドを提供する
3. When 設定値が存在する場合, the メソッド shall 設定値を返す
4. When 設定値が存在しない場合, the メソッド shall `default` 引数を返す
5. The モジュール shall pasta.toml解析済みテーブルをキャッシュする（ロード時に1回のみ解析）
6. If pasta.tomlが存在しない場合, the モジュール shall 空テーブルをキャッシュする

---

### Requirement 5: 総合フィーチャーテスト

**Objective:** As a 開発者, I want pasta.shiori.actの全機能を網羅する総合テストを実行したい, so that 実装の品質と回帰を検証できる

#### Acceptance Criteria

1. The 総合テスト shall `crates/pasta_lua/tests/lua_specs/shiori_act_integration_test.lua` に配置する
2. The テスト shall 以下のシナリオを検証する:
   - 複数アクター会話（sakura, kero, char2）のスコープ切り替え
   - 表情変更（surface）とテキストの組み合わせ
   - 待機（wait）と改行（newline）のタイミング制御
   - メソッドチェーン（`act:talk(...):surface(5):wait(500)`）
   - yield後のバッファリセットと継続
   - 設定ファイルによる改行数変更
3. The テスト shall 期待されるさくらスクリプト出力との完全一致を検証する
4. The テスト shall エラーケース（無効なactor、コルーチン外yield等）を検証する
5. The テスト shall 既存の `shiori_act_test.lua` とは別ファイルで、シナリオベースの統合テストとする

---

### Requirement 6: 既存テストの拡充

**Objective:** As a 開発者, I want 既存テストにyield・init_script・設定関連のテストを追加したい, so that 単体レベルでも品質を保証できる

#### Acceptance Criteria

1. The `shiori_act_test.lua` shall `yield()` メソッドのテストを追加する:
   - さくらスクリプト文字列がyieldされること
   - yield後にバッファがリセットされること
   - `_resume_value` が正しく設定されること
2. The テスト shall `init_script()` メソッドのテストを追加する:
   - バッファ、スコープ、レジューム値がすべてリセットされること
   - メソッドチェーンが動作すること
3. The テスト shall 設定読み込み（`pasta.config`）のテストを追加する:
   - デフォルト値の取得
   - 設定値の取得
   - 存在しないキーのデフォルト値フォールバック

---

## Out of Scope

- 高度なさくらスクリプト機能（バルーン制御、アニメーション等）
- 複数pasta.tomlファイルのマージ
- ランタイム中の設定変更（再読み込み）

---

## Context References（セッション継続用）

### ファイルパス

- **SHIORI_ACT実装**: `crates/pasta_lua/scripts/pasta/shiori/act.lua`
- **ACT（継承元）**: `crates/pasta_lua/scripts/pasta/act.lua`
- **既存テスト**: `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua`
- **設定モジュール（新規）**: `crates/pasta_lua/scripts/pasta/config.lua`
- **pasta.toml例**: `crates/pasta_lua/tests/fixtures/loader/with_config/pasta.toml`

### 既存シグネチャ

```lua
-- pasta.shiori.act
SHIORI_ACT.new(actors) → ShioriAct
SHIORI_ACT_IMPL.talk(self, actor, text) → self
SHIORI_ACT_IMPL.surface(self, id) → self
SHIORI_ACT_IMPL.wait(self, ms) → self
SHIORI_ACT_IMPL.newline(self, n) → self
SHIORI_ACT_IMPL.clear(self) → self
SHIORI_ACT_IMPL.build(self) → string  -- ★自動リセット追加（破壊的変更）
SHIORI_ACT_IMPL.reset(self) → self

-- pasta.act (継承元)
ACT_IMPL.yield(self) → nil (coroutine.yield内部)
ACT_IMPL.end_action(self) → nil
ACT_IMPL.init_scene(self, scene) → save, var
ACT_IMPL.call(self, global_scene_name, key, attrs, ...) → any
ACT_IMPL.word(self, name) → string|nil
```

### 新規追加予定

```lua
-- pasta.shiori.act 追加メソッド
SHIORI_ACT_IMPL.yield(self) → any (resume value)
SHIORI_ACT_IMPL.init_script(self) → self

-- pasta.config 新規モジュール
PASTA_CONFIG.get(section, key, default) → any
```

---

## Glossary

| 用語 | 説明 |
|------|------|
| yield | コルーチンの中断・再開ポイント |
| resume value | coroutine.resumeで渡された値（yield戻り値） |
| スコープ切り替え | sakura↔kero等のキャラクター切り替え |
| pasta.toml | Pasta設定ファイル（TOML形式） |
| init_script | さくらスクリプト組み立て状態の完全初期化 |
| 総合フィーチャーテスト | シナリオベースの統合テスト |
