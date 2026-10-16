# Requirements Document

## Project Description (Input)
`pasta_shiori` の結合テストは、一時ゴーストを作るときに `crates/pasta_shiori/tests/support/scripts/` を `scripts/` へコピーする。そこにある `pasta/*.lua` は本物のランタイム（`crates/pasta_lua/pasta_scripts/pasta/`）の古い写しで、新しい関数を持たない。新しいテストは写しを避けるために、フィクスチャの `pasta.toml` で `lua_search_paths` から `scripts` を外し、その理由をコメントで書いている。写しに気づかないテストは、古いランタイムで動いた結果を確かめてしまう。

困っているのは、`pasta_shiori` の結合テストを書く人・読む人。望む姿は、結合テストが本物のランタイムだけで動き、回避用の設定とコメントが要らなくなること。写しを消して、ハーネスが本物のランタイムを使うようにする。外部の挙動は変えない（テストだけの変更）。

## Introduction

`pasta_shiori` の結合テストの共通部品は、一時ゴーストを作るたびに、標準ランタイムの古い写し（`tests/support/scripts/pasta/` の 10 ファイル。`act.lua` は本物の 726 行に対して 209 行）を `scripts/` へ置く。`scripts/` は標準ランタイムより先に読まれるので、この写しが本物を隠す。新しい結合テストは、フィクスチャの `pasta.toml` で `scripts` を検索パスから外して写しを避けている（今は 4 か所）。結合テストを足すたびに、この回避が写されて増える。

この spec では、写しを消して、結合テストが `pasta.dll` が起動時に自己展開する本物のランタイムだけで動くようにする。回避の設定とコメントを外し、効いていない別のコピー処理と、実際と合わない説明も片付ける。直すのはテストだけで、外部の挙動は変えない。

ここに書く「本物のランタイム」は、`pasta.dll` が起動時に `profile/pasta/pasta_scripts/` へ自己展開する標準ランタイムを指す。「写し」は、`crates/pasta_shiori/tests/support/` の下に置いた、標準ランタイムとテスト用ライブラリの古いコピーを指す。

## Boundary Context

- **In scope**:
  - `crates/pasta_shiori/tests/support/` の写し（`scripts/` と `scriptlibs/`）を消すこと。
  - 結合テストの共通部品（`tests/common/mod.rs`）のコピー処理と、その説明。
  - 写しを避けている 4 つのフィクスチャ（`codegen_runtime_safety`・`actor_proxy_act_delegation`・`call_execution_correctness`・`scene_alias_ontalk`）の回避の設定とコメント。
  - 写しが置かれることを前提にした確かめ（`tests/test_path_helpers_test.rs` の `scriptlibs` の確かめ）と、古いランタイムの結果を期待値にしたテスト（調べた範囲では無い）。
  - 本物のランタイムをゴーストの直下の `pasta_scripts/` へコピーしている、効いていないコピー処理（`tests/` の下の 10 ファイル）と、その説明。
  - `src/shiori_request_tests.rs` にある同じ効いていないコピー（2 か所。根が同じなのでここで片付ける。2026-10-16 確定）。
- **Out of scope**:
  - ランタイムそのもの（`crates/pasta_lua/pasta_scripts/`・ローダー・自己展開）と、`pasta_shiori` の本番のコードの変更。
  - テストの意図の変更。
  - `pasta_lua` 側のテスト基盤（`crates/pasta_lua/scriptlibs/lua_test/`・モック）。
  - テスト用の入口のスクリプト（`scripts/pasta/shiori/entry.lua`）を自分で持つ 3 つのフィクスチャ（`shiori_lifecycle`・`async_callback`・`debug_preservation`）の差し替えそのもの。本番の読み込み順を使った差し替えなので残す。
  - テストごとに重複して書かれたディレクトリのコピー関数の整理（不具合ではなく目的にも要らない。2026-10-16 確定）。
  - 写しがまた置かれないための見張りのテストや検査（写しを消し、共通部品の説明で伝えれば足りる。2026-10-16 確定）。
  - 過去の記録（`TEST_COVERAGE.md` の履歴の節、完了した spec の文書）の書き換え。
- **Adjacent expectations**:
  - 本物のランタイムは、`pasta.dll` がゴーストの読み込みのときに `profile/pasta/pasta_scripts/` へ自己展開する。この spec はその仕組みに頼り、変えない。
  - `scripts/` に置いたスクリプトが標準ランタイムより先に読まれる読み込み順は、本番の挙動として保つ。テスト用の差し替えもこの順を使う。
  - 後に続く `runtime-error-balloon` は、回避の設定を書かずに結合テストを足せることを前提に、この spec の後に入る。

## Requirements

### Requirement 1: 結合テストを本物のランタイムだけで動かす

**Objective:** As a `pasta_shiori` の結合テストを書く開発者, I want 一時ゴーストが本物のランタイムだけで動くこと, so that テストが古いランタイムで動いた結果を確かめてしまわない

#### Acceptance Criteria

1. When 結合テストがフィクスチャから一時ゴーストを作る, the 結合テストの共通部品 shall 標準ランタイムの写しを一時ゴーストの `scripts/` へ置かない。
2. When 結合テストが一時ゴーストを読み込む, the 一時ゴースト shall 標準ランタイムを、`pasta.dll` が自己展開した `profile/pasta/pasta_scripts/` からだけ読む（フィクスチャが自分で持つ差し替えのファイルを除く）。
3. The リポジトリ shall `crates/pasta_shiori/tests/support/scripts/` の標準ランタイムの写し（10 ファイル）を持たない。
4. The リポジトリ shall どのテストも使っていないテスト用ライブラリの写し `crates/pasta_shiori/tests/support/scriptlibs/lua_test/`（3 ファイル）を持たない（2026-10-16 確定）。
5. When 写しを消した後に `pasta_shiori` のテストを通しで走らせる, the `pasta_shiori` のテスト shall 要件 4 で直したものを除き、消す前と同じ確かめのまま全部通る。

### Requirement 2: 写しを避ける回避の設定とコメントを外す

**Objective:** As a 結合テストを書く開発者, I want フィクスチャに写しを避ける設定を書かずに済むこと, so that 新しい結合テストに回避が写されて増えなくなる

#### Acceptance Criteria

1. The 4 つのフィクスチャ（`codegen_runtime_safety`・`actor_proxy_act_delegation`・`call_execution_correctness`・`scene_alias_ontalk`）の `pasta.toml` shall 写しを避けるために検索パスから `scripts` を外す設定を持たない。
2. The 4 つのフィクスチャの `pasta.toml` shall 写しを避ける理由を書いたコメント（「古い runtime の写し」「理由は同じ」など）を持たない。
3. When 回避を外した 4 つのフィクスチャを使う結合テストを走らせる, the 結合テスト shall 外す前と同じ確かめのまま通る。
4. When 開発者が新しいフィクスチャを足す, the 結合テストの共通部品 shall 検索パスを書き換えなくても、その一時ゴーストを本物のランタイムで動かす。
5. The 4 つのフィクスチャの `pasta.toml` shall 回避と関係のない設定（トークの間隔・アクター・ウェイトなど、テストの確かめに使う設定と、`[loader]` の `transpiled_output_dir`・`pasta_patterns`）を変えずに保つ。

### Requirement 3: テスト用の差し替えは本番の読み込み順で最小限だけ置く

**Objective:** As a 結合テストを書く開発者, I want テストにだけ要る差し替えをフィクスチャ側に最小限だけ置けること, so that 本物のランタイムのほかの部分はそのまま確かめられる

#### Acceptance Criteria

1. Where フィクスチャがテスト用の入口のスクリプト（`scripts/pasta/shiori/entry.lua`）を要する, the そのフィクスチャ shall 差し替えるファイルだけを自分の `scripts/` に持つ。
2. The テスト用の差し替え shall 本番と同じ読み込み順（`scripts/` が標準ランタイムより先）で効く。
3. If 写しにしか無い振る舞いに頼るテストが見つかる, then the 変更 shall その振る舞いを写しごと残すのではなく、そのテストのフィクスチャに要る最小限の差し替えとして置く。
4. If 写しを残す理由が見つかる, then the 変更 shall その写しを手で保つのではなく、本物のランタイムから作る形に変える。

### Requirement 4: 写しや古いランタイムを前提にした確かめを本物に合わせる

**Objective:** As a 結合テストを読む開発者, I want テストの期待値が本物のランタイムの構成と結果に合っていること, so that 通ったテストが本番の挙動を確かめていると信じられる

#### Acceptance Criteria

1. If 写しが一時ゴーストに置かれることを確かめるテストがある（`tests/test_path_helpers_test.rs` の `scriptlibs` の確かめ）, then the そのテスト shall 写しの有無ではなく、本来の意図（深いパス・ANSI で表せないパスの下に一時ゴーストを置けて、読めること）を確かめる形に直る。
2. If 古いランタイムで動いた結果を期待値にしているテストが見つかる, then the そのテスト shall 期待値を本物のランタイムの結果に直し、テストの意図は変えない。
3. When 期待値を直したテストがある, the 変更の記録（`tasks.md` の実装メモ） shall 直したテストと、直した理由（写しに頼っていた点）を残す。

### Requirement 5: 効いていないコピー処理を片付ける

**Objective:** As a 結合テストを読む開発者, I want テストの準備が実際に効く手順だけで書かれていること, so that どのランタイムで動いているかを読み違えない

#### Acceptance Criteria

1. The 一時ゴーストを作るテストの準備 shall 本物のランタイムをゴーストの直下の `pasta_scripts/` へコピーしない（今の読み込み先ではないので効かない。`tests/` の下の 10 ファイル）。
2. When 効いていないコピーを外した後に、それを使っていたテストを走らせる, the そのテスト shall 外す前と同じ確かめのまま通る。
3. Where `src/shiori_request_tests.rs` のテスト用の準備が同じ効いていないコピーをしている, the 変更 shall そのコピーも同じく外す（2026-10-16 確定）。

### Requirement 6: テストの準備の説明を実際の動きに合わせる

**Objective:** As a 結合テストを書く開発者, I want 共通部品とテストの説明が実際の動きと一致していること, so that 新しいテストを足すときに間違った前提で書かない

#### Acceptance Criteria

1. The 結合テストの共通部品の説明 shall 標準ランタイムは `pasta.dll` が自己展開したものを使い、フィクスチャは差し替えるファイルだけを持つことを書く。
2. The 結合テストの共通部品の説明 shall 差し替えの例として、実際に差し替えている `entry.lua` を挙げる（使っていない `main.lua` を例に挙げない）。
3. The 一時ゴーストを作るテストの説明 shall 標準ランタイムの読み込み先を `profile/pasta/pasta_scripts/`（自己展開）として書き、ゴーストの直下の `pasta_scripts/` を読み込み先として書かない。
4. The 結合テストの共通部品の説明 shall 実際には呼んでいない手順（「`copy_fixture_to_temp` でサポートとフィクスチャをコピーした後」など）を書かない。

### Requirement 7: 外部の挙動を変えない

**Objective:** As a ゴースト作者, I want この変更で `pasta.dll` の動きが変わらないこと, so that 配ったゴーストに影響が出ない

#### Acceptance Criteria

1. The 変更 shall テスト以外のコード（`pasta_shiori` と `pasta_lua` の本番のコード、`crates/pasta_lua/pasta_scripts/` のランタイム）を変えない。
2. The 変更 shall `scripts/` に置いた利用者のスクリプトが標準ランタイムより先に読まれる読み込み順を変えない。
3. When 変更を終える, the ワークスペースのテスト shall 全部通り、静的検査（clippy）が警告を出さない。
