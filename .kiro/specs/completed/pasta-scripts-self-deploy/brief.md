# Brief: pasta-scripts-self-deploy

## Problem
LuaJIT へ切り替えた後、SSP に導入済みのゴーストを `shiori.dll`（pasta.dll）として起動すると、`OnClose` イベント処理で 500 Internal Server Error が発生した。原因は `pasta_scripts/pasta/shiori/event/init.lua` の `coroutine.close`（Lua 5.4+ の機能で LuaJIT=Lua 5.1 には存在しない）の無防備な呼び出しだった。

コード側はすでに `if coroutine.close then` ガードで修正済み（コミット `70c583a` luajit-migration）だが、エラーは「コードのバグ」ではなく「**デプロイのドリフト**」が真因だった。pasta.dll を新しい LuaJIT ビルドに差し替えても、ゴースト側ディスクの `pasta_scripts/` は修正前の古い Lua スクリプトのまま取り残され、古い `init.lua` が読み込まれて再発した。

つまり「dll は新しいのに、ゴーストの `pasta_scripts/` が古い」というバージョンドリフトを、現状は構造的に防ぐ仕組みがない。手動コピー忘れが常にバグの温床になる。

## Current State
- `pasta_scripts/`（標準ランタイム Lua 一式：main.lua, pasta/ 等）はゴーストのディスク上にファイルとして配置され、Lua の `package.path` 経由で `require` される。dll には一切埋め込まれていない（完全な外部ファイル方式）。
- 検索パス優先順位（`default_lua_search_paths()`、`crates/pasta_lua/src/loader/config.rs:155`）はすでに役割分離されている：
  - `scripts/` = ユーザーカスタム（優先度：高）
  - `pasta_scripts/` = 標準ランタイム（優先度：低）
- `pasta_check release` はゴーストへ `pasta_scripts/` を同梱してビルドする。
- ドリフト検知・自己修復の仕組みは存在しない。dll とディスクの `pasta_scripts/` のバージョン一致は人間の運用規律に依存している。

## Desired Outcome
- pasta.dll が `pasta_scripts/`（フレームワーク層）の「所有者」となり、起動時にディスク上の実体を dll 内蔵の正本へ自動同期する。
- バージョンドリフトが構造的に発生しない。dll を差し替えれば、次回起動でゴーストの `pasta_scripts/` が自動的に正本へ更新される（今回のような手動コピー忘れバグが根絶される）。
- `pasta_scripts/` は常にディスク上に解凍済みの実体として見える（grep・目視・デバッグで読める）。
- 編集権は dll が握り、ユーザーが `pasta_scripts/` を直接弄っても次回起動でごりっと上書きされる。ユーザーがカスタムしたい場合は優先度が上の `scripts/` に書く。

## Approach
**自己展開（self-deploy）方式**：dll 内に `pasta_scripts/` を zip 圧縮 blob として埋め込み、起動時にバージョンダイジェスト（SHA-1）を比較して、不一致なら `pasta_scripts/` を全消し→解凍展開する。Lua ローダ（`package.path`）には一切手を入れず、「読み込み前にディスクへ同期する 1 ステップ」だけを追加する。

確定した設計判断：

| # | 項目 | 決定 |
|---|---|---|
| 中核 | 方式 | dll が `pasta_scripts/` を所有。起動時にディスク同期（ローダ無変更） |
| (1) | 検出 | `.sha1` ファイル方式。ディスクの `pasta_scripts/.sha1` 文字列と埋め込み `EXPECTED_SHA1` 定数を比較 |
| (2) | 展開 | 全消し→再展開（`pasta_scripts/` のみ。`scripts/` は不可侵） |
| (3) | 失敗時 | 古いまま続行 ＋ ERROR レベルで大声ログ警告（ローダ無変更を維持） |
| (4) | release | `pasta_scripts/` の同梱を継続（インストール直後から可視、古くても自己修復） |
| (5) | 保持形式 | zip 圧縮 blob を dll へ embed。`build.rs` が ①zip 生成 ②SHA-1 計算→`EXPECTED_SHA1` |

起動時ロジック（超軽量・起動時ハッシュ計算ゼロ）：
```
1. ディスクの pasta_scripts/.sha1 を読む
2. EXPECTED_SHA1 と文字列比較
3. 一致 → 何もしない（高速パス）
4. 不一致 or .sha1 欠落 → pasta_scripts/ を全消し → 内蔵 zip を解凍展開 → .sha1 に EXPECTED_SHA1 を書く
   （書き込み失敗時は ERROR ログ ＋ 古いまま続行）
```

build.rs の責務：
1. `crates/pasta_lua/pasta_scripts/` を zip 化 → `OUT_DIR/pasta_scripts.zip`（ソースとembedが常に同期し、手動zip同期忘れを構造的に排除）
2. その zip blob の SHA-1 を計算 → `EXPECTED_SHA1` 定数として埋め込み（生成 `.rs` か `cargo:rustc-env` 経由）

dll 側は `include_bytes!` で zip blob を、生成定数で `EXPECTED_SHA1` を保持する。

## Scope
- **In**:
  - `build.rs` での `pasta_scripts.zip` 生成と SHA-1 ダイジェスト算出・埋め込み
  - dll への zip blob 埋め込み（`include_bytes!`）
  - 起動時の同期ステップ（`.sha1` 比較 → 全消し→解凍展開 → `.sha1` 書き込み）
  - 書き込み失敗時の ERROR ログ＋続行ハンドリング
  - 同期ステップを runtime 初期化（`package.path` セットアップ直前、`crates/pasta_lua/src/runtime/mod.rs` 周辺）へ組み込む
  - zip 解凍・SHA-1 計算の依存追加（`zip` / `sha1` crate、ライセンス確認込み）
- **Out**:
  - Lua ローダ（`package.path` / `package.searchers`）の変更
  - `scripts/`（ユーザーカスタム層）に関する一切の挙動変更
  - ファイル単位の改竄検知（抽出後ツリーの再ハッシュ）—— `.sha1` マーカー比較のみで、バージョンドリフト検知に限定
  - `coroutine.close` バグ自体の修正（すでに完了済み）

## Boundary Candidates
- ビルド時アーティファクト生成（build.rs：zip 化＋SHA-1）
- 埋め込みアセット保持（zip blob ＋ `EXPECTED_SHA1` 定数）
- 起動時同期ステップ（検出→展開→マーカー書き込み→失敗ハンドリング）
- release ワークフローとの同梱整合

## Out of Boundary
- Lua スクリプトの読み込み・解決ロジック（`package.path` 方式は現状維持）
- `scripts/` 層の所有権・優先順位（既存のまま尊重）
- ゴーストのユーザー辞書（`dic/*.pasta` 等）

## Upstream / Downstream
- **Upstream**:
  - `crates/pasta_lua`（runtime 初期化、loader/config、package.path セットアップ）
  - luajit-migration spec（`coroutine.close` ガード等、LuaJIT 互換の前提）
- **Downstream**:
  - `release-workflow` spec（同梱継続の整合確認。dll が自己修復するため同梱が古くても害はないが、ビルド手順の前提が変わる）
  - `pasta_check`（release サブコマンドのゴーストビルド）

## Existing Spec Touchpoints
- **Extends**: なし（新規の単一スコープ feature）
- **Adjacent**:
  - `release-workflow`（`pasta_scripts/` 同梱の整合。同梱継続の方針を確認）
  - luajit-migration（互換修正の前提を共有。本 spec はその再発防止のデプロイ層対策）

## Constraints
- **ローダ無変更**：Lua の `package.path` / `package.searchers` には手を入れない。影響範囲を「起動時同期 1 ステップ」に限定する。
- **`scripts/` 不可侵**：同期処理は `pasta_scripts/` のみを対象とし、ユーザー所有の `scripts/` には絶対に触れない。
- **起動時の軽量性**：高速パス（バージョン一致時）ではハッシュ計算を行わず、`.sha1` 文字列比較のみで判定する。
- **ライセンス安全**：追加依存（`zip`, `sha1` 等）は MIT/Apache 系を選び、`deny.toml` のポリシーに適合させる（GPL 汚染なし）。
- **LuaJIT 前提**：mlua の LuaJIT feature を前提とする（Lua 5.1 相当）。
- **可視性の維持**：ディスク上の `pasta_scripts/` は常に解凍済みの生ファイルとして存在させる（圧縮形式はあくまで dll 内部保持のみ）。
- **決定論的 zip 生成（reproducible build）**：`.sha1` 比較が正しく機能する前提として、同一ソースからは常にバイト同一の `pasta_scripts.zip` が生成されること。build.rs はエントリ順序の固定（ソート）、タイムスタンプの固定値化、圧縮レベルの固定を行い、ビルド時刻等の非決定要素を zip に混入させない。これを怠ると `EXPECTED_SHA1` がビルド毎に変動し、起動の度に不要な再展開が走る。
