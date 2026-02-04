# Implementation Plan

## Task Overview

sakura-script-wait機能の実装タスク一覧。pasta_luaクレートに新規モジュール`sakura_script/`を追加し、既存の`@pasta_*`パターンに従ってLua API経由でウェイト付きさくらスクリプトを生成する。

**実装スコープ**: 
- TalkConfig設定管理（`loader/config.rs`）
- `@pasta_sakura_script`モジュール公開（`sakura_script/`）
- トークン分解とウェイト挿入ロジック
- Runtime統合と単体/統合テスト

---

## Implementation Tasks

### 1. 依存関係とプロジェクト構成の準備

- [ ] 1.1 (P) Cargo.tomlにregex依存を追加
  - `crates/pasta_lua/Cargo.toml`のdependenciesセクションに`regex = "1"`を追加
  - バージョン指定は`tech.md`の方針に従い1.x系を使用
  - _Requirements: 4.1_

- [ ] 1.2 (P) sakura_script/ディレクトリ構造作成
  - `crates/pasta_lua/src/sakura_script/`ディレクトリを作成
  - `mod.rs`、`tokenizer.rs`、`wait_inserter.rs`の3ファイルを配置
  - `lib.rs`に`pub mod sakura_script;`を追加
  - _Requirements: 1.1_

### 2. 設定管理の実装

- [ ] 2.1 (P) TalkConfig構造体の定義
  - `crates/pasta_lua/src/loader/config.rs`に`TalkConfig`構造体を追加
  - フィールド: ウェイト値5種（script_wait_normal等）、文字セット6種（chars_period等）
  - `Default`トレイト実装でハードコードデフォルト値を設定
  - `#[serde(default)]`属性でtoml読み込み時の部分設定対応
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 2.2 PastaConfigへのTalkConfig統合
  - `PastaConfig`に`talk: Option<TalkConfig>`フィールド追加（`custom_fields`パターン）
  - `talk()`アクセサメソッドを実装（`Option<&TalkConfig>`を返す）
  - 2.1完了後に実装（TalkConfig定義に依存）
  - _Requirements: 3.1_

- [ ] 2.3 (P) 不正設定値の警告ログ出力
  - tomlデシリアライズ失敗時にtracing::warnでログ出力
  - デフォルト値へのフォールバック動作を実装
  - `#[serde(default)]`により自動的にフォールバックされることをテストで確認
  - _Requirements: 3.4_

### 3. トークナイザーの実装

- [ ] 3.1 (P) CharSets構造体とToken型の定義
  - `CharSets`: 6種類のHashSet<char>を保持
  - `CharSets::from_config()`メソッドで文字列からHashSetへ変換
  - `TokenKind` enum: 8種類の文字種別（SakuraScript, Period, Comma等）
  - `Token`構造体: kind + text
  - _Requirements: 4.1_

- [ ] 3.2 Tokenizer構造体の実装
  - フィールド: `sakura_tag_regex: Regex`, `char_sets: CharSets`
  - `new(config: &TalkConfig) -> Result<Self, regex::Error>`メソッド（Regexコンパイル）
  - 正規表現パターン: `\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?`
  - 3.1完了後に実装（CharSets/Token型定義に依存）
  - _Requirements: 4.1, 4.3_

- [ ] 3.3 トークン分解ロジックの実装
  - `tokenize(&self, input: &str) -> Vec<Token>`メソッド
  - 先頭`\`チェック→正規表現マッチ→SakuraScriptトークン優先処理
  - `chars()`イテレータでUnicode文字を順次処理
  - CharSetsによる文字種別判定とトークン生成
  - 3.2完了後に実装（Tokenizer構造体に依存）
  - _Requirements: 4.1, 4.2, 4.3_

### 4. ウェイト挿入ロジックの実装

- [ ] 4.1 (P) WaitValues構造体の定義
  - フィールド: normal, period, comma, strong, leaderの5つのi64値
  - TalkConfigから変換するヘルパー関数を用意
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 4.2 ウェイト挿入関数の実装
  - `insert_waits(tokens: &[Token], wait_values: &WaitValues) -> String`
  - ルール1: SakuraScript/LineEndProhibitedトークンはスキップ
  - ルール2: Generalトークンは各文字後に`(normal - 50)`ミリ秒挿入
  - ルール3: Leaderトークンは各文字後に`(leader - 50)`ミリ秒挿入
  - ルール4-5: Period/Comma/Strong/LineStartProhibitedの連続処理（最大値追跡）
  - ルール6: ウェイト形式`\_w[ms]`生成
  - 4.1完了後に実装（WaitValues定義に依存）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

### 5. Lua API公開

- [ ] 5.1 WaitValueResolver関数の実装
  - `resolve_wait_values(actor: Option<&Table>, config: &TalkConfig) -> LuaResult<WaitValues>`
  - 3段階フォールバック: actor → config → hardcoded
  - actorテーブルから`script_wait_*`キー読み取り
  - 数値以外の値はデフォルトにフォールバック
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 5.2 talk_to_script関数の実装
  - Lua公開関数として`fn(actor: Option<Table>, talk: String) -> LuaResult<String>`
  - nil/空文字列バリデーション（空文字列を返す）
  - Tokenizer呼び出し → WaitInserter呼び出し
  - エラー時は元のtalk文字列を返し警告ログ出力
  - 5.1, 3.3, 4.2完了後に実装（全コンポーネントに依存）
  - _Requirements: 1.2, 1.3, 6.1, 6.2_

- [ ] 5.3 register関数の実装
  - `register(lua: &Lua, config: Option<&TalkConfig>) -> LuaResult<Table>`
  - TalkConfigからTokenizerとWaitInserterを初期化（Regexコンパイル）
  - `talk_to_script`関数をクロージャに格納しLuaテーブルに追加
  - `_VERSION`と`_DESCRIPTION`メタデータ追加
  - 5.2完了後に実装（talk_to_script定義に依存）
  - _Requirements: 1.1_

- [ ] 5.4 Runtime統合
  - `crates/pasta_lua/src/runtime/mod.rs`の`PastaRuntime::from_loader()`内で初期化
  - `let talk_config = self.config().and_then(|c| c.talk());`
  - `let sakura_module = sakura_script::register(&self.lua, talk_config.as_ref())?;`
  - `self.register_module("@pasta_sakura_script", sakura_module)?;`
  - 5.3, 2.2完了後に実装（register関数とTalkConfigアクセサに依存）
  - _Requirements: 1.1_

### 6. テスト実装

- [ ] 6.1 (P) Tokenizer単体テスト
  - さくらスクリプトタグ検出テスト（`\h\s[0]こんにちは`）
  - Unicode文字処理テスト（濁点・半濁点の個別char処理確認）
  - 文字種別判定テスト（Period, Comma, Strong等）
  - トークン順序保持確認
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 6.2 (P) WaitInserter単体テスト
  - 通常文字ウェイト挿入テスト（`こんにちは` → 各文字に`\_w[50]`）
  - 連続ぶら下げ文字処理テスト（`」」」！？。、` → 最大値950ミリ秒）
  - さくらスクリプトタグ保護テスト（タグにウェイト挿入しない）
  - ゼロ以下ウェイトのスキップテスト
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 6.3 WaitValueResolver単体テスト
  - actorテーブルからのウェイト値読み取りテスト
  - TalkConfigへのフォールバックテスト
  - ハードコードデフォルトへのフォールバックテスト
  - 数値以外の型のフォールバックテスト
  - 6.1, 6.2完了後に実装（関連コンポーネントに依存）
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 6.4 (P) TalkConfig統合テスト
  - pasta.toml [talk]セクション読み込みテスト
  - デフォルト値適用テスト（セクション不在時）
  - 部分設定のマージテスト
  - 不正型値の警告ログとフォールバックテスト
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 6.5 Lua API統合テスト
  - `require "@pasta_sakura_script"`モジュール読み込みテスト
  - `talk_to_script(actor, talk)`基本動作テスト（要件7.1-7.3の出力例）
  - nil actor時のデフォルト設定使用テスト
  - 空文字列入力時の空文字列返却テスト
  - エラー時の元文字列返却テスト
  - 6.3, 6.4完了後に実装（全コンポーネント統合に依存）
  - _Requirements: 1.1, 1.2, 1.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3_

### 7. ドキュメント整合性の確認と更新

- [ ] 7.1 SOUL.md整合性確認
  - コアバリュー（日本語フレンドリー、UNICODE識別子）との整合性確認
  - 設計原則（UI独立性、宣言的フロー）への影響評価
  - ビジョン（ゴースト会話の記述基盤）への貢献確認
  - 6.5完了後に実施（全機能完成後の評価）
  - _Requirements: all_

- [ ] 7.2 (P) クレートREADME更新
  - `crates/pasta_lua/README.md`に`@pasta_sakura_script`モジュールのセクション追加
  - API仕様とLua使用例を記載
  - pasta.toml設定例を追加
  - _Requirements: all_

- [ ] 7.3 (P) TEST_COVERAGE.md更新
  - 新規テストファイルのマッピング追加
  - カバレッジ指標の更新
  - _Requirements: all_

- [ ] 7.4 (P) steering/tech.md更新
  - pasta_lua依存関係セクションにregex 1.xを追加
  - _Requirements: 4.1_

---

## Task Summary

- **総タスク数**: 7メジャータスク、24サブタスク
- **要件カバレッジ**: 全7要件（1.1-7.3）を網羅
- **並列実行可能タスク**: 10サブタスク（(P)マーク付き）
- **平均タスク工数**: 1-3時間/サブタスク（設計詳細度に基づく見積もり）

---

## Dependencies & Critical Path

### Critical Path（順序依存）
1. `1.1, 1.2` → `2.1` → `2.2` → `3.1` → `3.2` → `3.3`
2. `4.1` → `4.2`
3. `5.1`, `3.3`, `4.2` → `5.2` → `5.3` → `5.4`
4. テスト: `6.3`, `6.4` → `6.5`
5. ドキュメント: `6.5` → `7.1`

### Parallel Opportunities
- `1.1` ∥ `1.2`（環境準備）
- `2.1` ∥ `2.3`（設定関連）
- `3.1` ∥ `4.1`（データ型定義）
- `6.1` ∥ `6.2` ∥ `6.4`（独立単体テスト）
- `7.2` ∥ `7.3` ∥ `7.4`（ドキュメント更新）

---

## Next Steps

1. タスク内容を確認し、不明点があれば設計書（design.md）参照
2. 承認後、`/kiro-spec-impl sakura-script-wait 1.1`で実装開始
3. 各タスク完了時にチェックボックスを更新
4. 全タスク完了後、`/kiro-spec-status sakura-script-wait`で最終確認
