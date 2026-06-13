# Requirements Document

## Project Description (Input)
ゴーストのトーク表示では、日本語テキストを自然な分かち書き位置で改行する必要がある（`@pasta_sakura_script` の `break_lines` / `talk_to_script`）。現在この改行位置推定に `budoux 0.1.1` クレートを利用しているが、開発の主軸は後継と判断される `budouy` クレートへ移っている。古いクレートに依存し続けると、保守・機能追加・将来のバグ修正の恩恵を受けられず、技術的負債となる。本 spec では、`budoux` への依存を完全に除去し `budouy 0.2.2`（`vendored-models` feature）へ置き換え、改行挙動を従来同等に保ちつつ、ビルド・clippy・テスト・関連ドキュメントをクリーンな状態へ更新する。

## Introduction
本機能は、日本語トークの自動改行（分かち書き）に使用している分かち書きライブラリを、保守が停滞した `budoux 0.1.1` から後継の `budouy 0.2.2`（`vendored-models` feature 有効）へ移行する、依存ライブラリの差し替え作業である。`@pasta_sakura_script` の `break_lines` / `talk_to_script` という Lua 公開 API の引数・戻り値は変更せず、改行挙動も従来同等を維持する。対象は workspace の依存定義、`crates/pasta_lua` の改行実装、既存テスト、および関連ドキュメント（steering / スキル）に限定する。改行アルゴリズム（幅閾値ロジック・タグ保持処理）の挙動変更や、budouy の追加機能（多言語モデル・HTML 処理・WASM 等）の採用は本機能の対象外とする。

> **最小変更の統治原則**: 本移行はクレートを `budoux` → `budouy` へ差し替えたうえで、**その差し替えに伴いコンパイルが通らなくなった箇所だけを修正する最小対応**とする。リファクタリング・設計改善・命名変更等の付随変更は行わない。**既存の公開 API 関数（`break_lines` / `talk_to_script` 等）およびプロパティ／フィールド（`actor.budoux` 等）はすべて維持**し、新規追加・削除・改称・シグネチャ変更を行わない（模型差に起因するテスト期待値の更新のみ Req 3.3 に従い例外的に許容する）。

## Boundary Context
- **In scope**:
  - workspace の依存定義を `budoux` から `budouy`（`vendored-models` feature 指定）へ差し替える。
  - `crates/pasta_lua` の改行実装（分かち書きライブラリの保持・初期化・呼び出し）について、クレート差し替えに伴いコンパイルが通らなくなった箇所のみを新ライブラリの API へ最小修正する。
  - 既存テスト（分かち書きライブラリの取得・呼び出しに依存する箇所）を新 API へ適合させ、入力・期待値・ケース構造は流用する。
  - 関連ドキュメント（主に `.kiro/steering/tech.md` の依存一覧）内の、内部依存クレートとしての旧クレート名・バージョン記載を新ライブラリ（`budouy 0.2.2`）の記載へ同期更新する。
  - `budoux` への依存を、依存ロックファイルを含めて完全に除去する。
- **Out of scope**:
  - 改行アルゴリズム（幅閾値ロジック、タグ保持処理）の挙動変更・改善・チューニング。
  - 改行品質・分割精度の評価や、budoux/budouy 以外の改行ライブラリの比較・選定。
  - 新言語モデル（中国語・タイ語等）対応、HTML 処理、WASM など budouy の追加機能の採用。
  - `break_lines` / `talk_to_script` の Lua 公開 API（引数・戻り値）の変更。
  - pasta.toml の `actor.budoux` 設定キー名および Lua の `actor.budoux` フィールド名の改称（ゴースト作者が直接記述する公開設定キーであり、不変に維持する）。
  - 改行機構の外部呼称「budoux / BudouX」の改称、および利用者マニュアル `book/`・完了 spec 名 `budoux-line-breaker` の機構名記載の更新（外部表記名として維持する）。
  - Lua スクリプト側（`pasta_scripts` 等）の利用コードの変更。
- **Adjacent expectations**:
  - 母体 spec `ukagaka-desktop-mascot`（`budoux-line-breaker` 機能を内包）は、分かち書きによる自動改行が従来と同等に機能し続けることを前提とする。本機能はその裏側で使用するクレートを差し替えるのみで、機能境界は重複しない。
  - `@pasta_sakura_script` を利用する Lua スクリプト全般（`talk_to_script` 経由の自動改行）は、公開 API が不変であるため改修不要であることを前提とする。

## Requirements

### Requirement 1: 分かち書きライブラリ依存の差し替え
**Objective:** As a pasta workspace の保守者, I want 分かち書き依存を `budoux` から `budouy` へ差し替えたい, so that 保守が継続される後継ライブラリの恩恵（将来のバグ修正・機能追加）を受けられ、技術的負債を解消できる

#### Acceptance Criteria
1. The pasta workspace shall ワークスペースの依存定義から `budoux` クレートへの依存を含まない。
2. The pasta workspace shall ワークスペースの依存定義に `budouy` バージョン `0.2.2` への依存を含む。
3. The pasta workspace shall `budouy` 依存に対して `vendored-models` feature を有効化した状態で参照する。
4. While 依存ロックファイルが生成された状態において, the pasta workspace shall 依存ロックファイルに `budoux` のエントリを含まない。
5. Where 分かち書き模型が必要な箇所において, the pasta workspace shall 外部模型ファイルの別途配布を要さず `vendored-models` により同梱された模型を使用する。

### Requirement 2: 改行挙動の互換維持
**Objective:** As a ゴースト辞書作成者, I want 移行後も日本語トークの自動改行が従来と同等に自然な位置・結果で行われてほしい, so that 既存ゴーストのトーク表示が移行によって意図せず破綻しない

> **互換の定義（機能的同等）**: 本要件における「互換」は、分割位置の文字単位での完全一致を意味しない。「自然な分かち書き位置で改行が挿入され、平文・さくらスクリプトタグが保持される」ことを互換基準とする。budouy 既定モデルと budoux で分割境界が異なりうるため、模型差で位置が変化するテストは Req 3.3 に従い妥当性を個別判断して期待値を更新する。

#### Acceptance Criteria
1. When `break_lines` が日本語テキストと幅閾値を受け取ったとき, the 改行処理 shall 自然な分かち書き位置で改行タグ（`\n`）を挿入する（互換は機能的同等を基準とし、分割位置の文字単位一致は要さない）。
2. When `talk_to_script` がトークテキストを処理したとき, the 改行処理 shall 妥当な自動改行結果（平文・さくらスクリプトタグを保持し、自然な位置で改行された結果）を出力する。
3. The 改行処理 shall 入力テキスト中のさくらスクリプトタグを改行幅計算から除外しつつ元の相対位置に保持する。
4. The `break_lines` / `talk_to_script` shall Lua 公開 API の引数および戻り値を移行前から変更しない。
5. If 幅閾値の指定が空であるか入力が空であるとき, then the 改行処理 shall 入力テキストを変更せずに返す。
6. The 移行作業 shall 既存の公開 API 関数およびプロパティ／フィールド（`break_lines` / `talk_to_script` / `actor.budoux` 等）を維持し、新規追加・削除・改称・シグネチャ変更を行わない。
7. The 移行作業 shall コード変更を、クレート差し替えに伴いコンパイルが通らなくなる箇所の修正に限定し、付随的なリファクタリングや設計変更を含まない。

### Requirement 3: 既存テストの適合と緑化
**Objective:** As a pasta workspace の保守者, I want 既存の改行テストが新ライブラリ API でそのまま通過してほしい, so that 移行が改行挙動を壊していないことを自動で検証できる

#### Acceptance Criteria
1. When テストスイートを実行したとき, the 改行関連テスト shall すべて成功（緑）する。
2. The 改行関連テスト shall 既存テストの入力・期待値・テストケース構造を流用し、変更を分かち書きライブラリの取得・呼び出しに関する API 差分のみに限定する。
3. If 模型差により特定テストの期待する分割位置が変化する場合, then the 改行関連テスト shall 当該期待値の妥当性を個別に判断したうえで更新する。

### Requirement 4: ビルド・静的解析の健全性
**Objective:** As a pasta workspace の保守者, I want 移行後にビルド・clippy・テストがクリーンに通ってほしい, so that 移行後のコードベースを健全な状態で維持できる

#### Acceptance Criteria
1. When ワークスペースをビルドしたとき, the ビルド処理 shall エラーなく完了する。
2. When clippy を実行したとき, the 静的解析 shall 本移行に起因する新規の警告またはエラーを出さない。
3. When テストスイートを実行したとき, the テスト実行 shall エラーなく完了する。

### Requirement 5: 関連ドキュメントの同期更新
**Objective:** As a ドキュメント読者（保守者・ゴースト作成者）, I want 内部実装が依存するクレート名・バージョン記載が実装と一致してほしい, so that ドキュメントから誤った旧依存クレート情報を参照しない

> **内部クレート名と外部機構名の区別**: 改行機構の**外部呼称は「budoux / BudouX」を正式名として不変に維持**し、**内部実装が依存するクレートのみ `budouy` へ差し替える**。したがって本要件の更新対象は「内部実装の依存クレートとしての `budoux` のクレート名・バージョン記載」（例: `.kiro/steering/tech.md` の依存一覧）に限る。以下は外部表記名 budoux として**維持し更新対象外**とする: 改行機構の外部呼称「BudouX」、公開設定キー名 `actor.budoux`（pasta.toml / Lua フィールド）、利用者マニュアル `book/` 内の機構名・使用例、完了済み spec 名 `budoux-line-breaker`（`product.md` 等の履歴記載）。

#### Acceptance Criteria
1. The 関連ドキュメント shall 内部実装の依存クレートとしての旧クレート `budoux` のクレート名・バージョン記載（依存一覧等）を残さず、移行後ライブラリ（`budouy 0.2.2`）を反映した記載へ更新する。
2. The 関連ドキュメント shall 改行機構の外部呼称「BudouX」・公開設定キー名 `actor.budoux`・利用者マニュアル `book/` の機構記載・完了 spec 名 `budoux-line-breaker` を、外部表記名として更新対象とせず維持する。

### Requirement 6: ライセンスおよびツールチェーン制約の充足
**Objective:** As a pasta workspace の保守者, I want 移行が既存のライセンス方針とツールチェーン要件に適合してほしい, so that ライセンス汚染やビルド不能といった移行起因の問題を回避できる

#### Acceptance Criteria
1. The 移行後の依存構成 shall workspace のライセンス方針（`MIT OR Apache-2.0`）と互換であり、GPL 系ライセンスによる汚染を生じない。
2. Where ビルドおよび CI/開発環境において, the ツールチェーン shall `budouy` が要求する最小 Rust バージョン（1.88.0）以上である。
