# Brief: lua-require-robustness

> **由来**: areka 実機での emo2 検証（2026-09-18）で発覚。独立した 2 つの欠陥（長パスで require 失敗・ロード失敗の無言化）を 1 spec に統合。両者とも `crates/pasta_lua/src/runtime/factory.rs` の起動時モジュールロードを触るため、分割するとマージ競合する（discovery 決定）。

## Problem

ゴーストを深いフォルダへインストールした利用者が、**何も喋らず・何のエラーも出さないゴースト**を踏む。原因を示す手がかりは `pasta.log` の warn 1 行のみで、ログを見に行く発想のない利用者には診断不能。

areka 実機（pasta.dll v0.3.4）での実測:

```
Failed to load pasta.shiori.entry, continuing without SHIORI functions
  error=...second_change.lua:9: module 'pasta.shiori.event.virtual_dispatcher' not found:
    no file '...\profile\pasta\pasta_scripts\pasta\shiori\event\virtual_dispatcher.lua'  ← 実在する
→ SHIORI table not found → SHIORI.request not available, returning default 204 response（以降ずっと）
```

- 該当ファイルの絶対パスは **264 文字**（MAX_PATH=260 超過）。同じゴーストを 202 文字のパスへ置き直すと正常動作。
- 同じ実行の中で pasta_scripts の自己展開（Rust 側）は成功しており、実ファイルは存在する。

ここには**独立した 2 つの欠陥**が重なっている。

### 欠陥 A: require が narrow fopen 依存

LuaJIT のモジュール検索は narrow（ANSI）な `fopen` でファイルを開くため、**MAX_PATH=260 が絶対上限**になる。ホストプロセスの longPathAware マニフェストにも `LongPathsEnabled` レジストリにも従わない。pasta.dll は DLL なのでマニフェストで制御する余地もない。一方、自己展開は Rust std（wide API）のため 260 超でも成功する——観測された非対称はこれで説明がつく。

この前提はリポジトリ内に明記済み（`crates/pasta_lua/src/loader/context.rs` の `generate_package_path_bytes`:「Lua's file I/O functions (fopen) use ANSI encoding on Windows」）。

**同根の未発覚バグ**: `package.path` を ANSI バイト列へ変換している以上、システム ANSI コードページで表現できない文字を含むパス（例: 非日本語ロケール機上の日本語フォルダ名）でも require が失敗する。

pasta 自身が master 直下から **71 文字**を消費している（展開先 `profile/pasta/pasta_scripts` 30 字＋最深モジュール `pasta/shiori/event/virtual_dispatcher.lua` 41 字）。利用者に残る余裕は 190 文字程度。トランスパイル済みシーン（`profile/pasta/cache/lua`）も同じ経路で require されるため同じ制限を受ける。

### 欠陥 B: entry のロード失敗が警告止まり

`factory.rs` は `require("pasta.shiori.entry")` の失敗を `tracing::warn!` で握りつぶして続行する。すぐ下の `require("pasta.scene_dic")` は `?` で伝搬しているのに、entry だけが例外扱い。`entry.lua` は `SHIORI.load` / `SHIORI.request` / `SHIORI.unload` を定義する唯一の場所なので、これに失敗した時点でゴーストは SHIORI として機能しない。「continuing without SHIORI functions」は続行ではなく、沈黙したまま壊れている状態。

欠陥 A を直しても、構文エラー・ファイル破損・権限など他の原因で同じ無言化が起きる経路は残る。

## Current State

- **モジュール検索**: `module_registry.rs` の `setup_package_path` が `LoaderContext::generate_package_path_bytes()`（ANSI 変換済み）を `package.path` に設定。検索パスは `default_lua_search_paths()` の 5 本（`profile/pasta/save/lua` → `scripts` → `profile/pasta/pasta_scripts` → `profile/pasta/cache/lua` → `scriptlibs`）で、`?.lua` と `?/init.lua` の 2 パターン。実際のファイルオープンは LuaJIT 標準 searcher（narrow fopen）。
- **起動シーケンス**（`factory.rs`）: `require("main")`（失敗は warn・続行）→ `require("pasta.shiori.entry")`（失敗は warn・続行）→ `require("pasta.scene_dic")`（失敗は `?` で伝搬）。
- **既存のエラー可視化機構**: 完了済み spec `load-error-logging` により、`PastaLoader::load()` が Err を返すと `pasta_shiori` が `last_load_error` に保持し、以降の request へ **500 + `X-ERROR-REASON`** を返す（`crates/pasta_shiori/src/shiori.rs`）。**entry のロード失敗はこの機構を素通りしている**——Err にならないため。
- **204 フォールバック**: `shiori.rs` は `SHIORI` テーブル／`SHIORI.request` が無い場合に既定 204 を返す経路を持つ（最小フィクスチャ・エンジンプロファイル向けの意図的な設計の可能性あり・要確認）。
- **チャンク名の依存**: `context.rs` のコメントどおり、`package.path` 由来のチャンク名（`@<package.path の ? 展開形>`）にソースマップとデバッガのブレークポイント解決が依存している。CI の 8.3 短縮名問題を受けて base 正規化は `std::path::absolute` で行っている。

## Desired Outcome

- ゴーストの設置パスが 260 文字を超えても、また ANSI コードページ外の文字を含んでも、`require` が成功しゴーストが正常に動作する。
- `pasta.shiori.entry` のロードに失敗した場合、ゴーストは無言にならず、**既存の load 失敗経路（500 + `X-ERROR-REASON`・ログ記録）で原因が可視化される**。
- 通常パス（短い・ASCII）での外部挙動は不変。既存テスト全緑。ソースマップ／デバッガ（`.pasta` 行 BP・コールスタック）に回帰なし。

## Approach

**欠陥 B（先行・小）**: entry のロード失敗を握りつぶさず `Err` として伝搬し、既存の `last_load_error` → 500 + `X-ERROR-REASON` 機構へ流す。新しい可視化機構は作らない。`main` は利用者任意の初期化スクリプトなので現行（warn・続行）を維持するかは要件で決める。

**欠陥 A（本体）**: `package.loaders` に **Rust 実装の searcher を前置**する。モジュール名 → 検索パス解決とファイル読み込みを Rust std（wide API）で行い、`lua.load(...).set_name(...)` でチャンクを返す。長パスと非 ANSI パスを同時に解消する。標準 searcher はフォールバックとして残す。

タスク順は B → A。B が先に入れば、A の実装中に起きるロード失敗も可視化された状態で作業できる。

### 却下した代替案

- **`\?\` 前置を package.path に入れる**: ANSI API は `\?\` を受け付けない。無効。
- **8.3 短縮名（GetShortPathNameW）**: 8.3 生成が無効化された環境があり不可。CI でも短縮名起因のバグ実績あり。
- **自己展開先の入れ子を縮める**: 崖を数十文字ずらすだけで根治にならない。
- **内蔵スクリプトだけメモリ（zip）から preload**: `scripts/` 上書き層と `cache/lua` のシーンモジュールを救えない。部分解。
- **「260 文字以内に置け」とドキュメントで案内**: areka のプロファイル配下配置では通常インストールで踏む。利用者側で回避不能なケースがある。

## Scope

- **In**:
  - Rust 実装 searcher の追加と `package.loaders` への前置（`?.lua` / `?/init.lua`・既存の検索パス順序と優先度を厳密に踏襲）
  - チャンク名の現行形式との一致（ソースマップ・デバッガ無回帰）
  - entry ロード失敗の Err 伝搬と、既存 500 + `X-ERROR-REASON` 経路への接続
  - 長パス（>260）・非 ANSI パスでの require 成功を実証するテスト、entry ロード失敗が 500 になるテスト
  - 関連ドキュメントの更新（起動シーケンスの記述・既知の制限があれば明記）
- **Out**:
  - Lua 標準の `io.open` / `loadfile` / `dofile` の長パス対応（ゴースト作者コードが直接叩く narrow API。ランタイム提供の `@pasta_persistence` 等は Rust 実装で影響なし）
  - 自己展開先 `profile/pasta/pasta_scripts` の配置変更・depth 削減
  - 新しいエラー表示 UI（バルーンへのエラー表示等）。既存の 500 + `X-ERROR-REASON` で足りる
  - areka 側の既定インストールパス短縮（areka リポジトリの責務）

## Boundary Candidates

- **モジュール検索（searcher）**: `pasta_lua` の runtime 初期化層。検索パス解決＋ファイル読み込み＋チャンク名付与。`package.path` の ANSI 変換は標準 searcher フォールバック用に残すか撤去するかを設計で決める。
- **起動失敗の分類**: `factory.rs` の「どの require 失敗が致命か」の方針。entry=致命／main=要件で決定／scene_dic=致命（現行）。
- **SHIORI 応答面**: `pasta_shiori` 側は原則無変更（既存の `last_load_error` 経路に乗るだけ）。204 フォールバック経路が意図的に使われているフィクスチャ／プロファイルの有無を確認し、必要なら区別する。

## Out of Boundary

- load 失敗時のログ初期化・`X-ERROR-REASON` の書式（`load-error-logging` が確立済み・変更しない）
- pasta_scripts の自己展開機構そのもの（`pasta-scripts-self-deploy` の責務・変更しない）
- ソースマップ生成・DAP の仕様（チャンク名を一致させることで非干渉を保つ）
- budoux／Sender 判定（別件。2026-09-18 にゴースト層で対処と決定済み）

## Upstream / Downstream

- **Upstream**: `lua-module-path-resolution`（検索パスと起動シーケンスの確立）、`pasta-scripts-self-deploy`（展開先）、`load-error-logging`（500 + `X-ERROR-REASON` 機構）、`pasta-source-map`（チャンク名依存）
- **Downstream**: areka 上で動く全 pasta ゴースト（emo2・pasta-in-windows・hello-pasta）。将来の `pasta-runtime-internals-doc` は本 spec 後の起動シーケンスを記述対象とする。

## Existing Spec Touchpoints

- **Extends**: なし（上流 spec はすべて completed。本 spec は新規境界）
- **Adjacent**: `load-error-logging`（機構を再利用するのみ・仕様は変えない）、`pasta-source-map` / `pasta-vscode-lua-debug`（チャンク名不変で非干渉）、`review-improvement-loop`（進行中だが本領域に未着手）

## Constraints

- LuaJIT 2.1（mlua 0.11・`luajit52` feature）。searcher テーブル名（`package.loaders` か `package.searchers` か）は設計フェーズで実機確認すること。
- 通常パスでの外部 SHIORI 挙動はバイト不変。`cargo test --all` 全緑。
- x86（`i686-pc-windows-msvc`）・x64 両ターゲットで動作。
- 長パスのテストは CI ランナーの `%TEMP%`（8.3 短縮名 `RUNNER~1`）上で走る。パス形式依存のバグは CI でのみ再現する実績があるため、テスト用ディレクトリの組み立てに注意。ホストが longPathAware でない環境では Rust std 側も 260 超で失敗し得るため、Rust 側で verbatim（`\?\`）パスを使う必要があるかを設計で確認すること。
- `cargo test` は `PASTA_DEBUG` 環境変数に非依存であること（load を叩く新テストには既存の `#[ctor]` ガードと同等の配慮が必要）。
