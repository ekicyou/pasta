# Requirements Document

## Introduction
pasta.dll（SHIORI ランタイム）を新しい LuaJIT ビルドへ差し替えても、ゴースト側ディスクの `pasta_scripts/`（標準ランタイム Lua 一式）が古いまま取り残され、バージョンドリフトによる実行時エラー（例：LuaJIT に存在しない `coroutine.close` を踏んでの 500 Internal Server Error）が発生する。コード側のバグが修正済みでも、手動コピー忘れが構造的にバグの温床となっている。

本機能は、pasta.dll をフレームワークスクリプト（標準ランタイム Lua 一式）の「所有者」と位置づけ、起動時にディスク上の実体を dll 内蔵の正本へ自動同期する自己展開（self-deploy）方式を導入する。自己展開先は SSP のネットワーク更新計算（`updates.txt` の MD5 計算・`.nar` パッケージ）から除外される `profile/` 配下のディレクトリ——**確定パス `ghost/master/profile/pasta/pasta_scripts/`（base_dir 相対 `profile/pasta/pasta_scripts/`）**、以下「**自己展開先**」——とし、`.md5` マーカーは同ディレクトリ直下（`profile/pasta/pasta_scripts/.md5`）に置く。`ghost/master/pasta_scripts/` への同梱は廃止する。これによりバージョンドリフトと、ネットワーク更新と自己展開の干渉を構造的に根絶し、同時にスクリプトを常にディスク上の解凍済み実体として可視化し、ユーザーのカスタムは優先度が上の `scripts/`（`ghost/master/` 直下）へ逃がす。

> **ネットワーク更新除外の根拠**: `crates/pasta_check/src/update_files.rs:142-146` は再帰の各階層で `EXCLUDED_DIRS = ["profile", "var"]` に一致するディレクトリをサブツリーごと除外する。よって `ghost/master/profile/...` は `updates.txt` の対象外。`.nar` には profile/ がランタイム生成のためビルド時に存在せず封入されない。

詳細な背景・設計判断は `.kiro/specs/pasta-scripts-self-deploy/brief.md` を参照。

## Boundary Context
- **In scope**:
  - 起動時のバージョン検出（`.md5` マーカー比較）と、不一致時の自己展開先への自己展開
  - 展開方式（一時領域へ展開→成功確認→アトミック入れ替え）と書き込み失敗時のフォールバック挙動（失敗時は直前版を保全）
  - dll 内蔵の保持形式（zip 圧縮 blob）と、ビルド時の決定論的アーカイブ生成・基準ダイジェスト埋め込み
  - 自己展開先（`profile/` 配下、ネットワーク更新除外領域）への解凍済み実体としての配置・可視化
  - フレームワークスクリプトの `ghost/master/` 同梱の廃止と、dll 自己展開への一本化
  - SSP ネットワーク更新（`updates.txt` / `.nar`）との非干渉
  - hello-pasta の構成（`pasta.toml` の検索パス・同梱）を自己展開方式へ整合（旧 `ghost/master/pasta_scripts/` の検索パスエントリ除去・コミット済み同梱撤去）
- **Out of scope**:
  - 抽出後ファイルのファイル単位改竄検知（ツリー再ハッシュ）。本機能は dll⇄ディスクのバージョンドリフト検知に限定する
  - `coroutine.close` バグ自体の修正（luajit-migration で完了済み）
  - Lua ローダの `package.path` / `package.searchers` の**解決機構**の変更（検索パスの既定値変更は含むが、解決ロジック自体は不変）
  - `scripts/`（ユーザーカスタム層）の挙動・優先順位の変更
  - 既にリリース済みの外部ゴースト・既存インストールのランタイム移行（現時点で対象ユーザーが存在しないため非対象。dll による旧ファイル能動削除や検索パスの強制注入は行わない）
- **Adjacent expectations**:
  - `release-workflow` / hello-pasta 配布手順（`release.ps1`）：`ghost/master/pasta_scripts/` のコピー手順を廃止し、自己展開へ移行すること
  - `luajit-migration`：LuaJIT 2.1（Lua 5.1 相当）互換が前提であること

## Requirements

### Requirement 1: 起動時バージョン検出と自己展開判定
**Objective:** ゴースト配布者・dll を更新したユーザーとして、pasta.dll の起動時に自己展開先のフレームワークスクリプトが自動で正本へ整合されてほしい。そうすれば手動コピー忘れによるバージョンドリフトを意識せずに済む。

#### Acceptance Criteria
1. When pasta.dll が起動し Lua スクリプト読み込み前の初期化が実行される, the pasta.dll shall 自己展開先の `.md5` マーカーを読み取り、内蔵の基準ダイジェストと比較する。
2. When `.md5` マーカーの値が基準ダイジェストと一致する, the pasta.dll shall 自己展開先へ一切書き込みを行わず、そのまま起動を継続する。
3. If `.md5` マーカーが存在しない（自己展開先が未生成の場合を含む）, then the pasta.dll shall 自己展開先への再展開（自己展開）を実行する。
4. If `.md5` マーカーの値が基準ダイジェストと一致しない, then the pasta.dll shall 自己展開先への再展開（自己展開）を実行する。
5. While 一致による高速パスを通る, the pasta.dll shall ディスク上ファイルの再ハッシュ計算を行わず、マーカー文字列の比較のみで判定する。
6. While 高速パス（一致）で起動する, the pasta.dll shall 使用中のフレームワークスクリプト版（基準ダイジェスト）を DEBUG レベルでログに記録し、現在地を診断可能にする。

### Requirement 2: 展開方式と所有権境界（アトミック展開）
**Objective:** ゴースト配布者として、再展開が常に正本どおりの完全な状態を生み、ユーザー資産を巻き込まず、かつ展開が失敗しても直前の動作版を壊さないでほしい。そうすれば旧版の残骸（orphan）による幽霊バグを防ぎつつ、失敗時のフォールバックも成立する。

#### Acceptance Criteria
1. When 再展開を実行する, the pasta.dll shall 内蔵正本を一時領域へ展開し、全ファイルの展開成功を確認してから自己展開先へアトミックに反映（入れ替え）する。
2. When アトミックな入れ替えが完了する, the pasta.dll shall 自己展開先の内容を内蔵正本と完全一致させ、旧版で削除されたファイル（orphan）を残さない。
3. If 一時領域への展開または入れ替えが失敗する, then the pasta.dll shall 自己展開先の直前の状態（旧スクリプトがあればそれ）を破壊せず保全する。
4. When アトミックな入れ替えが成功して再展開が完了する, the pasta.dll shall 最後に `.md5` マーカーへ基準ダイジェストを書き込む（入れ替え前に中断・クラッシュした場合は `.md5` が旧値のまま／欠落となり、次回起動で再展開される）。
5. The pasta.dll shall 同期処理の対象を自己展開先ディレクトリ（および同一領域内の一時領域）のみに限定し、`scripts/`（ユーザーカスタム層）および他のゴーストファイルには一切変更を加えない。
6. The pasta.dll shall 展開後のフレームワークスクリプトをディスク上に解凍済みの生ファイル（テキスト）として配置し、目視・grep・読み取りで内容を確認可能にする。
7. When 再展開を実行する, the pasta.dll shall 同期を実施した旨と更新後の版を識別できる情報をログに記録する。

### Requirement 3: 書き込み失敗時のフォールバック
**Objective:** ユーザーとして、読み取り専用ディレクトリやファイルロックなどで同期に失敗しても、ゴーストが起動不能にならず、かつ問題が診断可能であってほしい。

#### Acceptance Criteria
1. If 再展開中にディスクへの書き込みが失敗する, then the pasta.dll shall ERROR レベルのログを出力する。
2. If 書き込み失敗が発生する, then the pasta.dll shall 起動を中断せず、原子性保証（Requirement 2.3）により保全された直前の自己展開先スクリプト（初回展開失敗時は欠落）で実行を継続する。
3. The ERROR ログ shall 同期失敗の事実・対象パス・ドリフトが未解消である旨を含み、原因の特定を可能にする。

### Requirement 4: 内蔵保持形式とビルド時の決定論的生成
**Objective:** メンテナとして、dll 内のスクリプト正本がソースと常に一致し、配布物が肥大化せず、バージョン比較が安定して機能してほしい。

#### Acceptance Criteria
1. When pasta.dll をビルドする, the Pasta ビルドシステム shall ソースの `pasta_scripts/` ツリー全体（同梱 Lua ライブラリ socket/mime 等を含む。`scriptlibs/` 等の別検索パスは対象外）から zip 圧縮アーカイブを生成し、dll 成果物へ埋め込む。
2. When zip アーカイブを生成する, the Pasta ビルドシステム shall その zip blob のダイジェスト（MD5）を算出し、基準ダイジェストとして dll 成果物へ埋め込む。
3. The Pasta ビルドシステム shall 同一ソースから常にバイト同一の zip アーカイブを生成する（エントリ順序の固定・タイムスタンプの固定値化・圧縮レベルの固定により、ビルド時刻等の非決定要素を排除する）。
4. While ソースの `pasta_scripts/` 内容が不変である, the Pasta ビルドシステム shall 同一の基準ダイジェストを生成し、ビルドのたびの変化を起こさない。
5. When ソースの `pasta_scripts/` 内容が変化する, the Pasta ビルドシステム shall 変化を反映した基準ダイジェストへ更新する。
6. The Pasta ビルドシステム shall 埋め込み正本をビルドのたびにソースから生成し、手動でのアーカイブ同期を不要にする。

### Requirement 5: リリース同梱の廃止とネットワーク更新非干渉
**Objective:** ゴースト配布者として、フレームワークスクリプトを配布パッケージへ同梱せず dll 自己展開に一本化し、SSP のネットワーク更新がエンジンスクリプトを管理・削除して自己展開と衝突しないようにしてほしい。権威ある基準ダイジェストは pasta.dll が内蔵する（Requirement 4）。

#### Acceptance Criteria
1. The リリースパッケージング shall フレームワークスクリプトを `ghost/master/` 配下へ同梱せず、配布物への封入を dll 内蔵 zip のみとする（`release.ps1` の `pasta_scripts` コピー手順および当該コミット済みコピーは廃止する）。
2. The 自己展開先 shall `profile/pasta/pasta_scripts/`（base_dir 相対）に置かれ、`updates.txt`（MD5 計算）および `.nar` パッケージから除外される。
3. While SSP のネットワーク更新が実行される, the ネットワーク更新 shall 自己展開先のフレームワークスクリプトを管理・削除対象とせず、自己展開と干渉しない。
4. The `.md5` マーカーの生成・書き込み shall pasta.dll が所有し、リリースパッケージングはその生成・整合に関与しない。
5. When フレッシュなインストール後に初回起動が行われる, the pasta.dll shall 自己展開先を生成してフレームワークスクリプトを展開する。

### Requirement 6: 既存挙動の非回帰と hello-pasta 整合
**Objective:** メンテナとして、本機能の追加が Lua スクリプトの読み込み解決やユーザー上書き、SHIORI イベント処理の既存挙動を壊さず、かつ hello-pasta の構成が自己展開方式と整合して旧 `pasta_scripts/` のステール版が解決されないことを保証してほしい。

#### Acceptance Criteria
1. The pasta.dll shall Lua の `package.path` / `package.searchers` の**解決機構**を変更せず、同期を読み込み前の前段ステップとして実施する。
2. The pasta.dll shall `scripts/`（`ghost/master/` 直下）がフレームワークスクリプトより優先される既存の検索順位を維持し、ユーザーが `scripts/` に配置した上書きを引き続き有効にする。
3. The 既定 `lua_search_paths` および hello-pasta の `pasta.toml` shall フレームワークスクリプトの検索パスを自己展開先（`profile/pasta/pasta_scripts`）へ更新し、旧 `pasta_scripts`（`ghost/master/` 直下）エントリを除去する。
4. The hello-pasta 配布構成 shall コミット済み `ghost/master/pasta_scripts/` および `release.ps1` のコピー手順を撤去し、自己展開に一本化する（Requirement 5.1 と整合）。
5. While 同期完了後に通常の起動シーケンスが進行する, the pasta.dll shall 既存の SHIORI イベント処理の挙動を変更しない。
