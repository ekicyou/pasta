# Requirements Document

## Project Description (Input)
ゴースト作者（pasta DSL でゴーストを書く利用者）が、最初に書く `pasta.toml` の記述量が多すぎて負担になっている。`[package]` が必須のように見えるが Rust では消費されておらず、SHIORI 動作に必要な設定と将来のノベルゲームエンジン用途の設定が混在し、多くのフィールドはデフォルトが一意に決まるのに必須に見えて不安を与える。

本仕様では、`pasta.toml` のロード後に**単一のデフォルト適用ステップ**を通し、省略された値を**デフォルト表（SSOT）**から補完する設計モデルを採用する。デフォルト表は用途ごとに **SHIORI プロファイル**と**将来エンジンプロファイル**の2系統を概念として持ち、本仕様では SHIORI プロファイルのデフォルト値のみを確定する（エンジンプロファイルの値確定は将来仕様へ予約）。各セクション・各フィールドは「SHIORI プロファイルにデフォルトを持つ（省略可）」「デフォルトを持たない（必須）」「エンジンプロファイル専用（SHIORI では適用不要）」のいずれかに一意分類する。

「最小限の必須セクション」は**少なくとも1つの `[actor]` 定義のみ**とする（アクター名は `descript.txt` と一致が必要で、`spot` はゴースト固有のためデフォルト化できない唯一の residual）。`[actor]` 不在時は起動を妨げず軽量な警告で判別可能にする。`[package]` はエンジンプロファイル専用として位置づけ、デフォルトテンプレート・サンプルから除去する。最小テンプレートとフルリファレンステンプレートの2層を SSOT から提供する。既存のフル記述 `pasta.toml`（hello-pasta 等）は完全後方互換で従来どおり動作させる。

セクション名のリネームや名前空間による物理的再グルーピング（アプローチ B）、将来エンジンプロファイルのデフォルト値の具体確定・実装、設定解釈以外のランタイム挙動の変更は対象外とする。

## Boundary Context

- **In scope（本仕様が責務を持つ振る舞い）**:
  - `pasta.toml` の各セクション・各フィールドを「SHIORI プロファイルにデフォルトを持つ（省略可）/ デフォルトを持たない（必須）/ エンジンプロファイル専用（SHIORI では適用不要）」のいずれかに一意分類する
  - SHIORI プロファイルのデフォルト値を**単一のデフォルト表（SSOT）**として定義し、ロード後にそれを適用する**単一の補完ステップ**を確立する（明示値は上書きしない）
  - 「最小限の必須セクション」を少なくとも1つの `[actor]` 定義に限定し、それ以外の全セクションを省略可能・デフォルト補完対象として明確化する
  - `[actor]` 不在を起動を妨げない軽量な警告（ログまたは通知）で判別可能にする
  - 最小構成（`[actor]` のみ）でも慣例的な dic 配置の辞書が読み込まれること（pasta_patterns の SHIORI デフォルトの整合）の保証
  - `[package]` をエンジンプロファイル専用として位置づけ、デフォルトテンプレート・サンプルゴースト（hello-pasta）から除去する
  - 最小テンプレートとフルリファレンステンプレートの2層提供、およびデフォルト値の SSOT 化
  - 既存のフル記述 `pasta.toml` の完全後方互換の保証（回帰確認を含む）
  - 利用者向けドキュメント（マニュアル／サンプル）へのプロファイルモデル・分類の反映
- **Out of scope（本仕様が扱わない振る舞い）**:
  - セクション名のリネームや名前空間による物理的再グルーピング（アプローチ B）
  - 将来エンジンプロファイルのデフォルト値の具体確定・実装（概念・予約の明示に留める）
  - SSP プロパティ／永続化フォーマット等、設定解釈以外のランタイム挙動の変更
  - 設定値のバリデーション強化（型安全ラッパー等）。`[actor]` 不在の軽量警告を超える検証は行わない
  - `pasta.toml` ファイル不在を許容すること（明示的な必須は維持し、最小化の対象はあくまで記述量である）
- **Adjacent expectations（隣接仕様・系への期待）**:
  - デフォルト適用は Rust 側（`pasta_lua` ローダ）と Lua 側（`pasta.config` / `STORE.actors`）の両経路で一貫して同一の値を提供することを前提とする
  - 利用者マニュアル（pasta-user-manual）が設定ドキュメントの権威であり、プロファイルモデルはそこへ反映される
  - 将来エンジン（areka 等）のパッケージ概念仕様が、本仕様で予約した `[package]`（エンジンプロファイル）を将来の消費者として受け取り得る

## Requirements

### Requirement 1: デフォルトプロファイルモデルの確立
**Objective:** ゴースト作者として、各セクション・各フィールドが「省略してよい（デフォルトが自動適用される）」のか「自分で必ず書く必要がある」のかを明確に知りたい。各項目を「本当に要るのか」という不安なく取捨選択するため。

#### Acceptance Criteria
1. The pasta-config 仕様 shall `pasta.toml` の全セクション（`[loader]`、`[logging]`、`[persistence]`、`[lua]`、`[talk]`、`[ghost]`、`[actor]`、`[debug]`、`[package]`）およびそのフィールドを「SHIORI プロファイルにデフォルトを持つ（省略可）/ デフォルトを持たない（必須）/ エンジンプロファイル専用（SHIORI では適用不要）」のいずれか1つに分類する。
2. The pasta-config 仕様 shall SHIORI プロファイルのデフォルト値を単一のデフォルト表（SSOT）として定義する。
3. Where あるセクションまたはフィールドが「SHIORI プロファイルにデフォルトを持つ」に分類される場合、the pasta-config 仕様 shall その SHIORI デフォルト値を SSOT 上に明示する。
4. Where あるセクションまたはフィールドが「エンジンプロファイル専用」に分類される場合、the pasta-config 仕様 shall SHIORI 用途では適用・記述が不要であることを明示し、エンジンプロファイルのデフォルト値の確定を将来仕様へ予約する。
5. The pasta-config 仕様 shall 同一のセクション・フィールドが複数の分類に重複して属さないこと（分類が一意であること）を保証する。

### Requirement 2: 最小限の必須セクションの定義（`[actor]` のみ）
**Objective:** ゴースト作者として、最小限の必須セクションだけを書いた簡易な `pasta.toml` でゴーストを起動したい。初期の記述負担を最小化するため。

#### Acceptance Criteria
1. The pasta-config 仕様 shall 「デフォルトを持たない必須」を少なくとも1つの `[actor]` 定義に限定し、それ以外の全セクションを省略可能とする。
2. When 少なくとも1つの `[actor]` 定義を含む最小構成が与えられたとき、the SHIORI（pasta.dll）shall 他のセクションを省略していてもデフォルト補完によりゴーストとして起動する。
3. If `pasta.toml` に `[actor]` 定義が1つも含まれないとき、then the SHIORI（pasta.dll）shall 起動を停止せず、その不足を利用者が判別できる軽量な警告（ログまたは通知）を発する。
4. The pasta-config 仕様 shall `[ghost]`（`talk_interval_min` / `talk_interval_max` / `hour_margin` / `spot_newlines`）を含む `[actor]` 以外の全セクション（`[loader]`、`[logging]`、`[persistence]`、`[lua]`、`[talk]`、`[ghost]`、`[debug]`）を省略可能（SHIORI デフォルト有）として定義する。
5. When 最小構成（必須の `[actor]` のみ）が慣例的な dic 配置（dic 直下の `*.pasta` を含む）の辞書とともに与えられたとき、the SHIORI（pasta.dll）shall `pasta_patterns` の SHIORI デフォルトによりそれらの辞書を読み込んで起動する。

### Requirement 3: デフォルト適用ステップと省略時の補完
**Objective:** ゴースト作者として、任意セクションを省略したときに無難なデフォルトが自動適用され、かつ自分が明示した値は尊重されてほしい。省略しても従来と同じ挙動になる安心感を得るため。

#### Acceptance Criteria
1. The SHIORI（pasta.dll）shall `pasta.toml` のロード後に、SHIORI プロファイルのデフォルト表（SSOT）に基づき省略された項目を補完する単一の適用ステップを通す。
2. When 任意セクションまたは任意フィールドが省略されているとき、the SHIORI（pasta.dll）shall 当該項目へ SSOT の SHIORI デフォルト値（フル記述時と同一の値）を補完する。
3. While 最小構成（必須の `[actor]` のみ）で起動した状態、the SHIORI（pasta.dll）shall Rust 側ローダの解釈と Lua 側（`pasta.config` / `STORE.actors`）の解釈で一貫した同一のデフォルト値を提供する。
4. If ある項目が `pasta.toml` に明示的に記述されているとき、then the SHIORI（pasta.dll）shall デフォルト補完によってその明示値を上書きしない。
5. The pasta-config 仕様 shall ファイル不在を許容しない（`pasta.toml` の存在自体は引き続き必須とする）。

### Requirement 4: `[package]` のエンジンプロファイル化と除去
**Objective:** ゴースト作者として、SHIORI 用途では `[package]` を書かなくてよいと明確に知りたい。必須に見える冗長な記述を避けるため。

#### Acceptance Criteria
1. The pasta-config 仕様 shall `[package]`（`name` / `version` / `edition`）を「エンジンプロファイル専用」に分類し、SHIORI 用途では適用・記述が不要であることを明示する。
2. The pasta-config 仕様 shall エンジンプロファイルにおける `[package]` のデフォルト値の確定を将来仕様へ予約する（本仕様では実装しない）。
3. The デフォルトテンプレート（最小テンプレート） shall `[package]` セクションを含まない。
4. The サンプルゴースト（hello-pasta） shall `[package]` セクションを含まない（冒頭コメントの「必須項目は `[package]` と `[loader]` のみ」という記述の是正を含む）。
5. Where `pasta.toml` に `[package]` セクションが記述されている場合（既存ゴースト）、the SHIORI（pasta.dll）shall それを無視して従来どおり起動する（エラーや警告で起動を妨げない）。

### Requirement 5: テンプレートの2層提供とデフォルト値の SSOT 化
**Objective:** ゴースト作者として、最小テンプレートとフルリファレンステンプレートを使い分けたい。最初は最小から始め、必要に応じて全項目を参照できるようにするため。

#### Acceptance Criteria
1. The pasta-config 仕様 shall 「最小テンプレート」（必須の `[actor]` のみ）と「フルリファレンステンプレート」（全セクション・全フィールドを分類・デフォルト注記付きで網羅）の2層を提供する。
2. The 最小テンプレート shall Requirement 2 が定義する必須セクション（`[actor]`）のみを含む。
3. The フルリファレンステンプレート shall 各セクション・各フィールドについて、その分類（SHIORI デフォルト有 / 必須 / エンジンプロファイル専用）と SHIORI デフォルト値を判別できる形で記述する。
4. The 両テンプレートおよびドキュメントに現れる SHIORI デフォルト値 shall 単一のデフォルト表（SSOT）から導かれ、SSOT との間で値が矛盾しない。
5. If SSOT 上の SHIORI デフォルト値が更新されたとき、then the テンプレート・ドキュメントに反映されるデフォルト値 shall SSOT と一致した状態を保つ（テンプレートが独立に乖離しない）。

### Requirement 6: 既存フル記述の完全後方互換
**Objective:** 既存ゴーストの作者として、これまで書いてきたフル記述の `pasta.toml` をそのまま動かし続けたい。今回の再整理で既存ゴーストが壊れないことを保証するため。

#### Acceptance Criteria
1. When 既存のフル記述 `pasta.toml`（hello-pasta 等の従来形式）が与えられたとき、the SHIORI（pasta.dll）shall 本仕様適用前と同一の解釈・挙動でゴーストを起動する。
2. While 既存のフル記述で `[package]` を含む `pasta.toml` を読み込んでいる状態、the SHIORI（pasta.dll）shall `[package]` を含むいずれのセクションについてもエラーや挙動変化を生じさせない。
3. The pasta-config 仕様 shall 既存テスト（loader/config、startup、virtual_event_config、CONFIG.actor→STORE.actors 初期化等）の意図する挙動を破壊しない。
4. The pasta-config 仕様 shall 後方互換を検証する回帰確認を、既存のローダ／統合テスト水準で（フル記述・最小構成の双方について、起動および辞書読み込みの確認を含めて）伴う。

### Requirement 7: 利用者ドキュメントへのプロファイルモデル反映
**Objective:** ゴースト作者として、利用者マニュアルとサンプルでプロファイルモデルと最小構成を学びたい。フル記述をコピーして冗長化することなく、最小から始められるようにするため。

#### Acceptance Criteria
1. The 利用者向けドキュメント（マニュアル／README） shall 各セクション・各フィールドの分類（SHIORI デフォルト有 / 必須 / エンジンプロファイル専用）を反映する。
2. The 利用者向けドキュメント shall 最小構成の `pasta.toml` 例（必須の `[actor]` のみ）を提示する。
3. The 利用者向けドキュメント shall `[package]` が SHIORI 用途では不要で、エンジンプロファイル専用の予約であることを明示する。
4. When ゴースト作者がドキュメントに従って最小構成を作成したとき、the 作成された `pasta.toml` shall 本仕様の最小テンプレートと整合する（フル記述のコピーを強制されない）。
