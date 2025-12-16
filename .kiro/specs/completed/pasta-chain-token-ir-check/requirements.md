# Requirements Document

## Project Description (Input)
pastaエンジンに関する仕様。チェイントーク（いったん会話を終了して次のトークに移る）ためのIRは定義されているか。例えば「＠チェイン」または「＞チェイン」などとして関数を呼び出して、チェイントーク挿入が実現可能か。なお、すでに実現可能だと要件定義作成時調査で確認できた場合、本仕様は確認のみでクローズとする。要件定義指示の時に調査せよ。

## 調査結果サマリー

pastaエンジンのコードベース調査により、**チェイントーク機能はすでに実装済み**であることが確認できました。

### 実装確認事項

1. **IR定義**: `ScriptEvent::FireEvent`が存在し、イベント通知機能を提供
2. **チェインAPI**: `PastaEngine::execute_label_chain()`メソッドが実装済み
3. **属性ベースチェイン**: `@chain`属性によるラベル間連鎖機能をサポート
4. **統合テスト**: `test_chain_talk_manual()`および`test_chain_talk_with_api()`で動作検証済み

### 実装されている機能

- **イベント発火**: `ScriptEvent::FireEvent`を使用したイベント通知
- **チェイン実行**: `execute_label_chain()`による自動連鎖実行（最大深度制限付き）
- **マニュアルチェイン**: 個別の`execute_label()`呼び出しによる手動連鎖
- **チェイン検出**: `event_name.starts_with("chain:")`によるチェインイベント判定

### 実装箇所

- **IR定義**: `crates/pasta/src/ir/mod.rs` - `ScriptEvent`列挙型
- **エンジンAPI**: `crates/pasta/src/engine.rs` - `execute_label_chain()`メソッド
- **テストコード**: `crates/pasta/tests/engine_integration_test.rs` - 統合テスト

## Introduction

本要件は、Pasta DSLエンジンにおけるチェイントーク機能の確認を目的としています。チェイントークとは、現在の会話を終了し、次の会話ラベルへ自動的に移行する機能です。

調査の結果、pastaエンジンには既に以下の2つの方法でチェイントーク機能が実装されていることが確認できました：

1. **属性ベースチェイン**: `@chain`属性によるラベル間の連鎖指定
2. **イベントベースチェイン**: `FireEvent`イベントによる動的なチェイン制御

本仕様は、既存実装の確認および、DSL構文レベルでのチェイントーク挿入方法（`＠チェイン`や`＞チェイン`など）の実現可能性を検証することを目的とします。

## Requirements

### Requirement 1: IRレベルでのチェイントーク対応確認

**Objective:** As a Pasta Engine開発者, I want IRレベルでチェイントーク機能が定義されていることを確認する, so that チェイントークの基盤が既に存在することを検証できる

#### Acceptance Criteria

1. The Pasta IR shall `ScriptEvent::FireEvent`列挙型バリアントを定義し、イベント名とパラメータを保持する
2. When `ScriptEvent::FireEvent`が`event_name: "chain:ラベル名"`形式で発行された場合, the Pasta Engine shall それをチェインイベントとして認識する
3. The Pasta Engine shall `execute_label_chain()`メソッドを提供し、初期ラベルと最大連鎖深度を引数として受け取る
4. When `execute_label_chain()`が実行された場合, the Pasta Engine shall 連鎖深度制限に達するまで、または次のラベルが指定されなくなるまで、ラベルを順次実行する
5. The Pasta Engine shall すべての連鎖実行で生成された`ScriptEvent`を単一のベクタとして返す

### Requirement 2: 属性ベースチェイン機能の確認

**Objective:** As a スクリプト作成者, I want ラベル定義に`@chain`属性を使用してチェイン先を指定できる, so that 静的なチェイントーク連鎖を宣言的に定義できる

#### Acceptance Criteria

1. When ラベル定義に`＠属性名：chain`形式の属性が付与された場合, the Pasta Parser shall それをチェイン属性として解析する
2. The Pasta Engine shall ラベル属性を`LabelInfo`構造体の`attributes: HashMap<String, String>`フィールドに保存する
3. When `@chain`属性を持つラベルが実行された場合, the Pasta Runtime shall ラベル実行終了時にチェインイベントを自動的に発行する
4. The Pasta Runtime shall チェインイベントを`ScriptEvent::FireEvent { event_name: "chain:次のラベル名", params: vec![] }`形式で生成する
5. When `execute_label_chain()`がチェインイベントを検出した場合, the Pasta Engine shall 次のラベルを自動的に実行する

### Requirement 3: マニュアルチェイン実行の確認

**Objective:** As a アプリケーション開発者, I want 複数のラベルを順次実行してチェイントークを実現できる, so that プログラム側から手動でチェインを制御できる

#### Acceptance Criteria

1. The Pasta Engine shall `execute_label(label_name: &str)`メソッドを提供し、指定されたラベルを1回実行する
2. When `execute_label()`が複数回順次呼び出された場合, the Pasta Engine shall 各ラベルを独立して実行し、それぞれの`ScriptEvent`ベクタを返す
3. The Pasta Engine shall ラベル間の状態（グローバル変数など）を保持し、連続実行時に状態が継続する
4. When 統合テスト`test_chain_talk_manual()`が実行された場合, the Pasta Test Suite shall 2つのラベルを順次実行してチェイントークを模擬できることを検証する
5. The Pasta Engine shall マニュアルチェイン実行時にエラーハンドリングを提供し、途中でエラーが発生した場合は`Result::Err`を返す

### Requirement 4: DSL構文レベルでのチェイントーク挿入可能性調査

**Objective:** As a スクリプト作成者, I want 会話行またはステートメント内で`＠チェイン`や`＞チェイン`のような構文を使用してチェイントークを挿入できる, so that DSLレベルで直接チェインを制御できる

**調査結果**: 現在のpasta DSLでは、`＠チェイン`構文は**未実装**です。ただし、以下の代替手段が実装済みです：
- Rune関数呼び出し: `＠関数名()`で関数を呼び出し、関数内で`fire_event("chain:次のラベル", [])`を実行
- 属性ベースチェイン: ラベル定義に`＠chain：次のラベル`を付与

#### Acceptance Criteria

1. When 会話行内に`＠チェイン(次のラベル)`形式の構文が記述された場合, the Pasta Parser shall それを関数呼び出しとして解析する（`SpeechPart::FuncCall`）
2. When `＠チェイン`関数が実行された場合, the Pasta Runtime shall Rune標準ライブラリの`fire_event()`関数を使用して`ScriptEvent::FireEvent`を発行する
3. The Pasta Standard Library shall `chain(label_name: String)`関数を提供し、`fire_event(format!("chain:{}", label_name), [])`を実行する
4. When `＞チェイン`構文（call文）が記述された場合, the Pasta Parser shall 通常のcall文として解析し、チェインイベントを発行しない
5. The Pasta Documentation shall `＠チェイン()`関数の使用方法と、属性ベースチェインとの違いを説明する

### Requirement 5: チェイン深度制限と無限ループ防止

**Objective:** As a Pasta Engine, I want チェイン実行に深度制限を設けることで無限ループを防止する, so that スクリプトエラーによるシステムハングを回避できる

#### Acceptance Criteria

1. The Pasta Engine shall `execute_label_chain()`メソッドで`max_chain_depth: usize`パラメータを必須とする
2. When チェイン実行深度が`max_chain_depth`に達した場合, the Pasta Engine shall チェイン実行を停止し、それまでに収集したすべての`ScriptEvent`を返す
3. The Pasta Engine shall デフォルトの最大チェイン深度として推奨値（例：10）をドキュメントに記載する
4. When 無限チェインループが検出された場合, the Pasta Engine shall エラーログを出力せず、単に深度制限で実行を停止する
5. The Pasta Engine shall チェイン深度カウンタをラベル実行ごとにインクリメントし、`depth < max_chain_depth`の間のみループを継続する

### Requirement 6: チェイントークテストカバレッジの確認

**Objective:** As a Pasta Engine開発者, I want チェイントーク機能の動作が統合テストで検証されていることを確認する, so that 機能の安定性と信頼性を保証できる

#### Acceptance Criteria

1. When 統合テスト`test_chain_talk_manual()`が実行された場合, the Test Suite shall 2つのラベルを順次実行し、合計4つ以上のイベントが生成されることを検証する
2. When 統合テスト`test_chain_talk_with_api()`が実行された場合, the Test Suite shall 3つのラベルを独立して実行し、各ラベルが2つ以上のイベントを生成することを検証する
3. The Test Suite shall チェイン実行のエッジケース（最大深度到達、チェインなしラベル）をカバーする
4. The Test Suite shall チェインイベント検出ロジック（`event_name.starts_with("chain:")`）をテストする
5. The Test Suite shall マニュアルチェイン実行とAPI経由チェイン実行の両方をテストする

### Requirement 7: ドキュメント更新

**Objective:** As a Pasta DSL ユーザー, I want チェイントーク機能の使い方を理解できる, so that 効果的に会話の連鎖を実装できる

#### Acceptance Criteria

1. The Pasta Documentation shall GRAMMAR.mdにチェイントークの説明セクションを追加する
2. The Pasta Documentation shall 属性ベースチェインの使用方法を説明する（`＠chain：次のラベル`構文を含む）
3. The Pasta Documentation shall Rune関数経由チェインの使用方法を説明する（`＠チェイン(ラベル名)`または`fire_event("chain:ラベル名", [])`を含む）
4. The Pasta Documentation shall マニュアルチェイン実行（`execute_label()`の連続呼び出し）の使用例を提供する
5. The Pasta Documentation shall `execute_label_chain()` APIの使用方法と最大深度パラメータの推奨値を説明する
6. The Pasta Documentation shall 最低2つの実用的なサンプルスクリプト（属性ベース、関数ベース）を提供する

---

## 結論

**本仕様はクローズとします。**

調査の結果、pastaエンジンには既にチェイントーク機能が実装されており、以下の方法で利用可能です：

1. **属性ベースチェイン**: ラベル定義に`＠chain：次のラベル`を付与
2. **イベントベースチェイン**: Rune関数から`fire_event("chain:次のラベル", [])`を呼び出し
3. **マニュアルチェイン**: `execute_label()`を連続して呼び出し
4. **自動チェイン実行**: `execute_label_chain()`メソッドを使用

DSL構文レベルでの`＠チェイン`関数は未実装ですが、Rune標準ライブラリに`chain(label_name: String)`関数を追加することで簡単に実装可能です（Requirement 4参照）。

本要件定義は、既存実装の確認と、DSL構文拡張の設計指針を提供します。実装が必要な場合は、Requirement 4を基に設計フェーズへ進んでください。