# Gap Analysis: pasta-scripts-self-deploy

> ブラウンフィールド統合分析。既存コードの調査に基づき、実装戦略・選択肢・リスク・設計フェーズへ持ち越す研究項目を整理する。コード調査結果は実ファイルパス・関数名に基づく。

## 1. 分析サマリー
- **統合の主戦場は2つ**：①ビルド時（`crates/pasta_lua/` に **build.rs を新規追加** し pasta_scripts を zip 化＋MD5 算出して埋め込む）、②起動時（`PastaLoader::load()` に自己展開ステップを挿入）。Lua ローダ本体（`package.path` 解決）は無変更で要件を満たせる。
- **再利用できる既存資産が豊富**：`zip 8.6`（pasta_check で使用＝workspace 依存済み）、`flate2`（pasta_lua 使用済み）、`fs::remove_dir_all`（全消しパターン既存）、`.cache_version` バージョンマーカー前例（`env!("CARGO_PKG_VERSION")`）、`LoaderError`/`tracing` 規約。
- **新規が必要なのは僅か**：build.rs（pasta_lua）、起動時展開モジュール、zip の**決定論的生成**設定。**ハッシュは既存 `md5` crate を流用**（ドリフト検知に暗号強度不要・依存追加ゼロ、要件ディスカッション C2 で決定）。
- **主要リスクは2点**：(a) zip のバイト決定性（mtime/順序/圧縮レベル固定）、(b) 展開失敗を**非致命**にする制御（現状の `PastaLoader::load` はエラーを伝播し得る）。
- **推奨アプローチ**：Option A（既存ローダ拡張＋build.rs 追加）。影響範囲が要件の「ローダ無変更」制約と整合し、既存パターンに乗れる。

## 2. Requirement → Asset マップ（ギャップ付き）

| Req | 必要能力 | 既存資産 / 統合先 | ギャップ |
|---|---|---|---|
| 1 起動時検出・展開判定 | base_dir/pasta_scripts の `.md5` 読取と基準値比較 | `PastaLoader::load()`（`loader/mod.rs` Phase 2 付近）、`.cache_version` 前例（`loader/cache.rs`） | **Missing**: `.md5` 読取・比較ロジック、基準ダイジェスト定数 |
| 2 全消し→再展開・境界 | `pasta_scripts/` のみ全消し＋zip 展開＋`.md5` 書込 | `fs::remove_dir_all`（`copy.rs`/`cache.rs`）、`zip::ZipArchive` 展開 | **Missing**: zip 解凍→ディスク書出、`.md5` 書込。`scripts/` は別ディレクトリで自然に不可侵 |
| 3 失敗時フォールバック | 書込失敗で ERROR ログ＋起動継続 | `tracing::error!` 規約、`LoaderError` | **Constraint**: load シーケンスのエラー伝播を断ち、非致命化する制御が必要 |
| 4 zip embed＋決定論＋MD5 | build.rs で zip 生成・MD5 算出・埋め込み | `zip`（workspace 依存）、既存 `md5` crate、`include_str!` 前例（テスト）、`pasta_sample_ghost/build.rs`（埋め込みなし） | **Missing**: pasta_lua の build.rs、`include_bytes!`、`EXPECTED_MD5` 埋め込み、**決定論的 zip 設定** |
| 5 release 同梱整合 | ゴーストへ pasta_scripts(+.md5) 同梱、古くても自己修復 | `release.rs`（copy_dir_recursive で同梱）、ソース正本 `crates/pasta_lua/pasta_scripts/` | **Unknown**: 同梱する `.md5` の用意方法（初回再展開許容 vs pasta_check で生成） |
| 6 非回帰 | package.path/searchers 無変更、scripts/ 優先維持、SHIORI 不変 | `module_registry.rs::setup_package_path`、`default_lua_search_paths`（scripts/ が上位） | **Low**: 展開を読込前の前段ステップに限定すれば自然に満たせる |

## 3. 既存コードの要点（実コード根拠）

- **起動シーケンス**: `PastaLoader::load(base_dir)` → Phase1 config → **Phase2 prepare_cache_dir** → Phase3 discover_files → … → Phase6 Runtime 構築。`setup_package_path(lua, loader_context)` が `package.path` を設定（`module_registry.rs:36-52`）。
- **ゴーストルート解決**: pasta_shiori が SHIORI.load の `load_dir`（= `ghost/master/`）を `PastaLoader::load` へ渡す。`LoaderContext` が `base_dir = canonicalize(load_dir)`（Windows は `\\?\` 除去）。`pasta_scripts` ディスク位置 = `{base_dir}/pasta_scripts/`（`loader/context.rs:80-137`, `config.rs:155-163`）。
- **build.rs 現状**: `pasta_lua`・`pasta_shiori` に **build.rs なし**。`pasta_sample_ghost/build.rs` は存在するが埋め込み未使用。
- **zip/圧縮/ハッシュ**: `pasta_check/src/nar.rs` が `ZipWriter`＋`CompressionMethod::Deflated` で NAR 生成。`pasta_lua` は `flate2`（永続化 gzip）。ハッシュは `pasta_check` の `md5`（updates.txt 用、SHA 系は未使用）。
- **バージョンマーカー前例**: `loader/cache.rs` の `.cache_version`（`CURRENT_VERSION = env!("CARGO_PKG_VERSION")`）。`.md5` マーカーはこの前例に倣える。
- **全消し/エラー/ログ**: `prepare_release_dir`/`cache.rs` に `fs::remove_dir_all` パターン。`LoaderError::io/cache_directory`、`error!(path=%.., error=%e, "...")` 規約。

## 4. 実装アプローチ選択肢

### Option A: 既存ローダ拡張 ＋ build.rs 追加（推奨）
- **構成**: ①`crates/pasta_lua/build.rs`（新規）で `pasta_scripts/` を決定論的 zip 化→`OUT_DIR/pasta_scripts.zip`、既存 `md5` で MD5 算出→`EXPECTED_MD5` を `cargo:rustc-env` か生成 `.rs` で埋め込み。②`loader/extract.rs`（新規・小モジュール）で `.md5` 比較・全消し・zip 展開・`.md5` 書込。③`PastaLoader::load` の Phase2 直後に呼び出し（非致命）。
- **Trade-offs**: ✅ 既存パターン（`.cache_version`/`remove_dir_all`/`zip`）に乗れる ✅「ローダ無変更」制約と整合（前段ステップのみ） ✅ 影響局所 ❌ build.rs を初導入する学習コスト（pasta_lua）
- **適合度**: 高。要件の境界制約に最も素直。

### Option B: 専用クレート新設（`pasta_assets`）
- **構成**: 埋め込み＋展開を独立クレート化し pasta_lua が依存。
- **Trade-offs**: ✅ 関心の分離が明快、単体テスト容易 ❌ 単一アセット群のためにクレート増設は過剰 ❌ ワークスペース複雑化。
- **適合度**: 中（将来 embed 対象が増えるなら再評価）。

### Option C: ハイブリッド／段階導入
- **構成**: MVP=build.rs zip＋起動時展開（同梱 `.md5` を用意せず**初回1回だけ再展開**を許容）。後続=pasta_check が `.md5` を生成して同梱し初回も高速パス化。
- **Trade-offs**: ✅ Req5 の `.md5` 用意問題を後回しにでき初期実装が軽い ✅ 段階検証可能 ❌ 初回起動で必ず1回展開が走る（害は軽微） ❌ 2 段階の調整。
- **適合度**: 中〜高（Req5 の未確定点を吸収する現実解）。

## 5. 複雑度・リスク
- **Effort: M（3〜7日）** — 既存パターン豊富だが、build.rs 初導入・決定論 zip・非致命フォールバック配線で中程度。
- **Risk: Medium** — 技術は既知（zip/flate2/fs/tracing 既存）。不確実性は (a) zip 決定性の実現可否、(b) load シーケンスの非致命化、(c) Req5 の `.md5` 同梱方法に集中。いずれも局所で回避策あり。

## 6. 設計フェーズへの推奨と研究項目（Research Needed）
- **推奨アプローチ**: Option A をベースに、Req5 の同梱 `.md5` は Option C 的に「初回再展開許容」を MVP とし、必要なら pasta_check 生成へ拡張。
- **Research Needed**:
  1. **zip 決定性**: `zip` crate で last_modified 固定・エントリ名ソート・圧縮レベル固定によりバイト同一を保証できるか検証（保証できなければ手書き deflate or 別手段）。
  2. **ハッシュ方式（要件ディスカッション C2 で解決）**: 既存 `md5` crate を流用（マーカー名 `.md5`）。ドリフト検知に暗号強度は不要で、依存追加ゼロ・既存ハッシュ利用と統一。build.rs（build-dependency）と runtime の双方から利用。新規 `sha1` 追加は不採用。
  3. **EXPECTED_MD5 埋め込み手段**: `cargo:rustc-env` か `OUT_DIR` 生成 `.rs` の `include!`。
  4. **非致命化**: `PastaLoader::load` の Phase 戻り値設計を壊さず、展開失敗を `error!` ログ＋継続にする制御（`Result` を握り潰す境界の置き場所）。
  5. **Req5 `.md5` 同梱**: 初回再展開許容（同梱なし）か、pasta_check が built dll の `EXPECTED_MD5` を取得して `.md5` を生成・同梱するか。
  6. **runtime 展開依存**: `zip` を pasta_lua の通常（非 build）依存へ追加（解凍に必要）。`flate2` のみで済むか（純 deflate）も比較。
  7. **base_dir/対象パス**: `lua_search_paths` で `pasta_scripts` が既定以外に変更された場合の展開対象の確定（既定 `{base_dir}/pasta_scripts/` 前提で良いか）。
  8. **アトミック展開（要件化済み・Req2）＋同時展開の競合（B2）**: 要件ディスカッション2巡目で**アトミック展開を要件化**（一時領域へ展開→成功確認→アトミック入れ替え、失敗時は直前版を保全）。設計で実現手段を確定する：一時ディレクトリへ全展開→`fs::rename` による原子的差し替え（旧を退避ディレクトリへ rename してから新を rename in、成功後に旧を削除）。Windows の `rename` はディレクトリ入れ替えで制約があるため、退避→差し込み→旧削除の順や、同一ボリューム前提を要検討。これは同時展開競合（複数インスタンス）の緩和も兼ねる。`.md5` は入れ替え成功後に最後に書く（途中失敗時は旧 `.md5`／欠落のまま→次回再展開）。

---

## 7. 要件ディスカッションで判明した重要事項（追補）

### 7.1 自己展開先を `profile/` 配下へ移す（ネットワーク更新非干渉）
- **コード根拠**: `crates/pasta_check/src/update_files.rs:11` `const EXCLUDED_DIRS: &[&str] = &["profile", "var"];`、および `nar.rs:50` で `profile/` を NAR から除外。→ **`profile/`（と `var/`）配下は updates.txt の MD5 計算・`.nar` パッケージの双方から除外される**。
- **決定**: フレームワークスクリプトの自己展開先を `ghost/master/pasta_scripts/` から **`ghost/master/profile/pasta/pasta_scripts/`（base_dir 相対 `profile/pasta/pasta_scripts/`）** へ移す。`.md5` マーカーは同ディレクトリ直下。既存の `profile/pasta/{save,cache}` と並ぶ兄弟。`profile/pasta/cache/` は `CacheManager` が `remove_dir_all` 管理するため**避ける**（巻き添え消去の危険）。これにより SSP ネットワーク更新と dll 自己展開の衝突を構造的に回避する。
- **除外の確証**: `update_files.rs:142-146` は再帰の各階層で `EXCLUDED_DIRS = ["profile","var"]` 一致ディレクトリをサブツリーごとスキップ。`ghost/master/profile/...` は確実に `updates.txt` 対象外。
- **副次効果**: `profile/` は NAR から除外されるため、配布パッケージへ自然に封入されなくなり、「master 同梱の廃止（選択肢B）」が自動的に成立する。可視性は `profile/` 配下の実体ファイルとして維持。

### 7.2 hello-pasta 同梱（`release.ps1`）の廃止
- **コード根拠**: `crates/pasta_sample_ghost/release.ps1:124-147` が `crates/pasta_lua/pasta_scripts` → `{MasterDir}/pasta_scripts` を robocopy。リポジトリには `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta_scripts/` がコミット済み。
- **決定**: 自己展開へ一本化するため、(a) `release.ps1` の pasta_scripts コピー手順を削除、(b) コミット済み `master/pasta_scripts/` を撤去。これらが「リポジトリ内のドリフト源」だったため、撤去で根絶。
- **Research Needed（移行）**: 統合テスト `crates/pasta_sample_ghost/tests/common/mod.rs`（pasta_scripts を master へコピーしている）と `tests/integration_test.rs:121-123`（`"pasta_scripts"` が検索パスに含まれることを検証）への影響。テストを新しい自己展開／profile パス前提へ更新する必要がある。

### 7.4 設計 synthesis（design フェーズ確定事項）
- **挿入点**: `PastaLoader::load_with_config`（`loader/mod.rs:92-182`）の Phase 2（cache 準備, 118行）と Phase 3（discover, 120行）の間に **Phase 2.5** として `extract::sync_pasta_scripts(base_dir)` を非致命呼び出し。base_dir はこの時点で確定。
- **依存（build-vs-adopt: adopt）**: 新規 external crate は不要。`zip 8.6`（deflate）・`md5 0.8` は workspace 定義済み。pasta_lua の `[dependencies]` と `[build-dependencies]` 双方に追加するのみ。runtime の zip 読み込み（`ZipArchive`）は本体初の利用（前例は nar.rs のテストのみ）。
- **埋め込み手段**: build.rs が `OUT_DIR/pasta_scripts.zip` を生成→runtime は `include_bytes!(concat!(env!("OUT_DIR"), "/pasta_scripts.zip"))`。MD5 は `cargo:rustc-env=PASTA_SCRIPTS_MD5` →`env!("PASTA_SCRIPTS_MD5")`。runtime 側で MD5 計算は不要（マーカー文字列比較のみ）。`md5` の runtime 利用は実質なし（build のみ）→ 設計簡素化。
- **アトミック展開（実現手段）**: 同一ボリュームの一時ディレクトリへ全展開→スワップ（旧退避→新差し込み→旧削除）。Windows のディレクトリ rename 制約に対応。`.md5` はスワップ成功後に最後に書く。途中クラッシュは次回 `.md5` 不整合で自己修復。
- **エラー型**: `LoaderError` に `SelfDeploy { path, source }` を追加。呼出し側は `mod.rs:435-437` の非致命前例に倣い、ただしログは ERROR（Req3.1）。
- **検索パス更新**: `default_lua_search_paths()`（config.rs:155-163）と hello-pasta `pasta.toml`（20-26行）の `"pasta_scripts"` を `"profile/pasta/pasta_scripts"` へ置換。`scripts` は依然上位、`generate_package_path` 機構は不変。

### 7.3 旧 `ghost/master/pasta_scripts/` の検索パス移行
- **コード根拠**: `pasta.toml:23` が `lua_search_paths` に `"pasta_scripts"` を列挙。`loader/config.rs:155-163` の既定にも `"pasta_scripts"` あり。
- **決定/リスク**: 自己展開先を profile 配下へ移すなら、`lua_search_paths` の既定値と hello-pasta の `pasta.toml` から旧 `"pasta_scripts"`（master 直下）を**除去または置換**しないと、ステールな旧版が profile 版より先に解決されて**今回のバグが再発**しうる。これは `default_lua_search_paths()` の変更を伴うが、`package.path`/`searchers` の**解決ロジック自体**は不変。
- **移行スコープ（要件ディスカッション2巡目で確定）**: 外部に既存ユーザーが存在しないため、**リリース済み外部ゴースト・既存インストールのランタイム移行は非対象**。dll による旧ファイル能動削除や検索パス強制注入は行わない。やるべきは「hello-pasta の `pasta.toml`・同梱・テストを新方式へ整合させる」一回限りの実装のみ。`default_lua_search_paths()` も合わせて更新し将来ゴーストの既定を正す。
