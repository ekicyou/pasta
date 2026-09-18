# Research & Gap Analysis: lua-require-robustness

> **生成フェーズ**: `/kiro-validate-gap`（要件生成直後）
> **対象**: `.kiro/specs/lua-require-robustness/requirements.md`（Requirement 1〜7）
> **調査基準**: worktree `C:/home/maz/git/pasta/.claude/worktrees/lua-require-robustness-8ae027`（v0.3.4 時点）

## Summary

- **Feature**: `lua-require-robustness`
- **Discovery Scope**: Complex Integration（既存の起動シーケンス・モジュール解決・SHIORI 応答経路の 3 層にまたがる）
- **Key Findings**:
  1. **ブリーフの前提が 1 つ崩れている**。欠陥 B の受け皿として想定していた「既存の 500 + `X-ERROR-REASON` 経路」は、アクターランタイム移行後の本番リクエスト処理から**呼び出されなくなっている**。`MyError::to_shiori_response()` の本番呼び出し元はゼロで、リクエスト処理のエラーは応答チャネルの drop 経由で一律 204 に落ちる。entry のロード失敗を `Err` 伝搬させるだけでは、204 の出所が変わるだけで利用者からは依然として無言のままになる。
  2. **挿入点はきれいに空いている**。`package.loaders` / `package.searchers` / `package.preload` を触る本番コードは皆無で、Rust モジュールはすべて `package.loaded` へ直接登録されている。`require` は `package.loaded` を searcher より先に見るため、searcher 前置で `@pasta_*` 群を隠す事故は起きない。
  3. **チャンク名の回帰リスクは想定より低い**。ソースマップ／ブレークポイントは `canonicalize_chunk_name`（`@` 除去・`\`→`/`・Windows は小文字化）を通してから照合する。現行の本番チャンク名は Windows で**区切りが混在**しており、正規化はその吸収が目的。既存の `chunk_name_validation_test.rs` がそのまま回帰ゲートになる。
  4. **非 ANSI パスは現状「無言」ではなく「ハード失敗」**。`generate_package_path_bytes()` の ANSI 変換は変換不能文字で `Err` を返し、`setup_package_path` の `?` でロード全体が失敗する。欠陥 A の 2 症状（長パス＝無言 204／非 ANSI＝ロード失敗）は現れ方が異なる。
  5. **長パス・非 ANSI の実ディレクトリを作るテストは 1 件も存在しない**。非 ASCII は合成文字列としてしか登場せず、実際にファイルを開くテストはない。Requirement 6 はテスト基盤の新規構築を伴う。

---

## Research Log

### 1. 起動シーケンスと失敗の扱い

- **Context**: 欠陥 B（entry ロード失敗の握りつぶし）の正確な位置と、Err の伝搬先を確定する。
- **Sources**: `crates/pasta_lua/src/runtime/factory.rs`, `crates/pasta_lua/src/runtime/runtime_config.rs`, `crates/pasta_lua/src/loader/mod.rs`, `crates/pasta_lua/src/loader/error.rs`, `crates/pasta_shiori/src/error.rs`
- **Findings**:
  - 本番の起動関数は `PastaLuaRuntime::from_loader_with_scene_dic`（`factory.rs:124-249`、戻り値 `LuaResult<Self>` = `mlua::Result`）。
  - `factory.rs:197-201` `require("main")` → warn・継続。
  - `factory.rs:203-208` `require("pasta.shiori.entry")` → warn・継続（**欠陥 B**）。
  - `factory.rs:210-213` `require("pasta.scene_dic")` → `?` で伝搬（致命）。
  - `lua_require`（`runtime_config.rs:352-355`）は Lua 標準 `require` の素の呼び出し。未検出時は LuaJIT 標準の `module 'X' not found: ... no file '<候補>'` 文字列がそのまま `mlua::Error::RuntimeError` として上がる（実機ログの文面と一致）。
  - Err の経路: `factory` → `loader/mod.rs:213-221` の `?` → `LoaderError::Runtime(#[from] mlua::Error)`（`loader/error.rs:47-49`）→ `MyError::Load(...)`（`pasta_shiori/src/error.rs:47-51`）。**entry を `?` に変えるだけで、この既存の Err 配管に乗る**。
  - **旧経路 `from_loader`（`factory.rs:35-93`）にも同型の無言化がある**。`scripts/pasta/shiori/entry.lua` を `std::fs::read_to_string` + `lua.load(..).set_name("entry.lua").exec()` で読み、読み取り失敗・実行失敗のいずれも warn のみ（`factory.rs:74-90`）。現状は `crates/pasta_lua/tests/runtime/{encoding_test,runtime_api_test}.rs` からのみ使用。
- **Implications**: Requirement 4.1 / 5.1 / 5.2 の実装点は `factory.rs` の 3 行に閉じる。Requirement 4.1 の変更自体は S 規模。ただし 4.2〜4.5（応答面）は下記 §2 の問題に直撃する。

### 2. ロード失敗が利用者に届く経路（最重要）

- **Context**: ブリーフは「既存の `last_load_error` → 500 + `X-ERROR-REASON` 機構へ流すだけ」を前提にしていた。その機構が生きているかを確認する。
- **Sources**: `crates/pasta_shiori/src/shiori.rs`, `crates/pasta_shiori/src/error.rs`, `crates/pasta_shiori/src/windows.rs`, `crates/pasta_shiori/src/actor/thread.rs`, `crates/pasta_shiori/src/actor/marshaling.rs`
- **Findings**:
  - `last_load_error` の格納（`shiori.rs:147`）と、後続 request での `MyError::Load(msg)` 化（`shiori.rs:153-161`）は**生きている**。
  - しかし 500 応答を組み立てる `MyError::to_shiori_response()`（`error.rs:79-87`）の**本番呼び出し元はゼロ**。リポジトリ全体で定義（`error.rs:79`）と自身の単体テスト（`error.rs:144-151`）のみ。
  - 本番 FFI 経路は `windows.rs:151-188` → `lifecycle::marshal_request` → `actor/thread.rs:184-196`。ここで `shiori.request(..)` が `Err` の場合、**reply を送らず drop** する。受信側 `actor/marshaling.rs:180-184` が `Disconnected` を観測して `default_204()` を返す。
  - `windows.rs:178-183` のコメントがこの設計変更を明記している（「旧実装は 500 を返したが、本番アクター経路は…必ず文字列を返す契約に統一する・R5.6」）。
  - **Rust 側 204 フォールバック（`shiori.rs:260-271` `default_204_response()`）は事実上デッド**。到達するのは「entry の require が warn で握りつぶされ、他は成功した」場合のみ、すなわちまさに本件の障害モード。意図的に利用しているフィクスチャ・ゴースト・プロファイルは存在しない（`pasta_lua/tests/fixtures/loader/*` は `scripts/` を持たないが、自己展開された `profile/pasta/pasta_scripts/pasta/shiori/entry.lua` が既定検索パス上にあるため `SHIORI` は常に存在する）。
- **Implications**:
  - **Requirement 4.2 / 4.3 / 4.5 は「既存機構への接続」では満たせない**。entry を `?` にすると、無言 204 の出所が「`SHIORI.request` 不在」から「アクターの reply drop」へ移るだけで、利用者体験は変わらない。
  - 少なくとも「ロード失敗状態の request はエラー文字列応答（500）を返す」という契約をアクター境界に追加する必要がある。これは `pasta-actor-runtime` が確立した「必ず文字列を返す」契約と矛盾しない（500 も文字列である）。ブリーフの「`pasta_shiori` は原則無変更」という境界想定は成立しない。
  - 一方で **Requirement 4.5（204 を返さない）は副作用が小さい**。デッド判定により、区別すべき正当な利用者は見つかっていない。

### 3. モジュール検索パスと ANSI 変換

- **Context**: 欠陥 A の根の位置と、searcher 前置時に踏襲すべき解決規則を確定する。
- **Sources**: `crates/pasta_lua/src/loader/context.rs`, `crates/pasta_lua/src/loader/config/mod.rs`, `crates/pasta_lua/src/runtime/module_registry.rs`, `crates/pasta_lua/src/encoding/{mod.rs,windows.rs}`
- **Findings**:
  - 検索パス既定値（`loader/config/mod.rs:235-243`）: `profile/pasta/save/lua` → `scripts` → `profile/pasta/pasta_scripts` → `profile/pasta/cache/lua` → `scriptlibs`。
  - `generate_package_path`（`context.rs:95-110`）は各検索パスにつき `<abs>/?.lua`・`<abs>/?/init.lua` の順で 2 エントリを出し `;` 連結。**バックスラッシュは全て `/` へ置換**。重複排除も存在確認もしない。計 10 エントリ。
  - `setup_package_path`（`module_registry.rs:37-53`）は `package.path` を**上書き**（既定値を残さない）。`package.cpath` は不変。
  - ANSI 変換は外部クレートではなく自前実装（`encoding/windows.rs`、`MultiByteToWideChar` / `WideCharToMultiByte`、`CP_ACP`）。**変換不能文字は `ErrorKind::InvalidInput` のハードエラー**（`encoding/windows.rs:104-113`）。したがって非 ANSI パスでは `setup_package_path` の `?`（`factory.rs:158`）でロードが失敗する。
  - base_dir の絶対化は `std::path::absolute`（`context.rs:68`）で、`canonicalize` は**意図的に避けている**（8.3 短縮名・symlink でチャンク名がキャッシュ／ソースマップ側と乖離し、CI でブレークポイントが壊れた実績。`context.rs:42-66` に詳細）。同コメントは「`\\?\` 拡張長プレフィックスを出さないので除去も不要」とも明記している。
  - `package.loaders` / `package.searchers` / `package.preload` を触る本番コードは**皆無**。ヒットはベンダ同梱 luacheck（`scriptlibs/luacheck/config.lua:130,139,141`）とテストのスタブのみ。
  - Rust 製モジュール（`@pasta_config` / `@enc` / `@pasta_persistence` / `@pasta_log` / `@pasta_sakura_script` / `@pasta_search`）は `package.loaded` へ直接 set（`module_registry.rs:21-27`）。`require` は `package.loaded` を searcher より先に見るため、**searcher 前置でこれらを隠す事故は構造的に起きない**。
- **Implications**:
  - Requirement 3.1〜3.3 は `LoaderContext::absolute_search_paths()` を searcher 側で再利用すれば自然に満たせる（`package.path` 文字列の再パースは不要）。
  - `std::path::absolute` が `\\?\` を出さない以上、**長パス対応には searcher 側で verbatim プレフィックスを明示的に付ける必要がある可能性が高い**（設計フェーズで実測要）。ただしチャンク名には verbatim 形を**出してはならない**（`context.rs:42-66` の乖離問題の再発）。読み込み用パスと命名用パスを分離する設計が要る。
  - `package.path` の ANSI 変換をフォールバック用に残すと、非 ANSI パスでは変換段階で落ちるため、Requirement 2 を満たすには残し方を変える（lossy 化・エラー無視・当該エントリのスキップ等）か撤去する判断が要る（Open Question 4）。

### 4. チャンク名の生成と消費

- **Context**: Requirement 3.4 / 3.6（ソースマップ・デバッガ無回帰）の実際の厳しさを測る。
- **Sources**: `crates/pasta_lua/src/debug/source_map/mod.rs`, `crates/pasta_lua/src/debug/breakpoints.rs`, `crates/pasta_lua/src/debug/dap/resolver.rs`, `crates/pasta_lua/src/loader/cache.rs`, `crates/pasta_lua/tests/chunk_name_validation_test.rs`
- **Findings**:
  - 本番のシーンチャンク名は**標準 `require` が付けるもの**で、Rust 側の `set_name` は通っていない。形式は `@<絶対 .lua パス>`、Windows では**区切りが混在**（`package.path` 前置部は `/`、`?` 展開部は `\`）。
  - `canonicalize_chunk_name`（`source_map/mod.rs:316-330`）: `@` 除去 → `\`→`/` → Windows は小文字化。FS 問い合わせなし・8.3 解決なし・絶対化なし。区切り統一は上記の混在を吸収するために**必須**と実測コメントに明記（`mod.rs:296-315`）。
  - 格納／照合の両側で正規化（`SourceMap::add_chunk` `mod.rs:452`、`resolve_lua_to_pasta` `mod.rs:540`）。ブレークポイント照合も同様（`breakpoints.rs:120-133`）。DAP 側は二重正規化を避けて生のフック source を渡す（`dap/resolver.rs:78-84`）。
  - キャッシュ側のキー生成は `CacheManager::source_to_cache_path`（`cache.rs:229-240`）、モジュール名は `source_to_module_name`（`cache.rs:210-223`、`pasta.scene.<dotted>`・`-`→`_`）。
  - `chunk_name_validation_test.rs` がトランスパイル → `require` → ラインフックの往復を実測で固定している（`@` 始まり・末尾一致・正規化後の等価性を assert）。ヘッダ（24-36 行）は「構築時一致＋正規化で十分、`set_name` 明示命名は不要」という決定を記録。
- **Implications**:
  - Rust searcher が `lua.load(..).set_name(..)` で名前を付ける場合、**正規化後に同一のキーへ落ちれば**ソースマップ／ブレークポイントは無回帰。区切りをすべて `/` に揃えた `@<絶対パス>` で要件を満たす見込みが高い（大小文字と実体パスが一致していること、verbatim プレフィックスを含めないことが条件）。
  - ただし**エラーメッセージとスタックトレースには生の形が出る**ため、「同一形式」をどこまで厳密に求めるかは要判断（Open Question 7）。
  - `chunk_name_validation_test.rs` がそのまま Requirement 3.4 / 3.6 の回帰ゲートになる。

### 5. テスト基盤

- **Context**: Requirement 6 のテストを既存基盤の上に載せられるか。
- **Sources**: `crates/pasta_lua/tests/common/mod.rs`, `crates/pasta_shiori/tests/common/{mod.rs,test_env.rs}`, `crates/pasta_lua/tests/loader/*`, `crates/pasta_shiori/tests/byte_invariant_test.rs`
- **Findings**:
  - `PASTA_DEBUG` 中和の `#[ctor::ctor]` ガードは 4 箇所に同一実装で存在（`pasta_lua/tests/common/mod.rs:13-32`、`pasta_lua/tests/scene_identity_index_test.rs:22-38`、`pasta_shiori/tests/common/mod.rs:11-29`、`pasta_shiori/src/shiori_request_tests.rs:5-22`）。**新規テストファイルを追加する場合は同ガードが必要**（Requirement 6.4）。既存コメントは、汚染時の症状が `module 'i18n' not found` という**無関係に見えるモジュール未検出**として現れると記録しており、本件の調査時に混同しやすい。
  - 一時ゴースト構築ヘルパ: `copy_fixture_to_temp`（`pasta_lua/tests/common/mod.rs:135-158`、`pasta_shiori/tests/common/mod.rs:44-61`）、`create_temp_with_pasta`（161-190）、`ShioriTestEnv::new`（`tests/common/test_env.rs:59-71`）。いずれも `tempfile::TempDir::new()`（ASCII・短パス）ベース。
  - Requirement 3.5（バイト不変）の既存ゲート: `crates/pasta_shiori/tests/byte_invariant_test.rs`（ゴールデン応答＋改竄検知）、`kick_unused_byte_invariant_test.rs`、`ffi_extern_session_e2e_test.rs`。
  - **長パス・非 ASCII の実ディレクトリを作るテストは存在しない**。非 ASCII は `context.rs:320-335` と `tests/runtime/encoding_test.rs:294-316` に**合成文字列**として現れるだけで、ファイルは一切開かれない（前者はバイト列が UTF-8 と異なることだけ、後者は `package.path` が空でないことだけを assert）。
  - パス形式の回帰テストは 8.3 短縮名関連のみ（`context.rs:278-318`）。
- **Implications**: Requirement 6.1 / 6.2 は**新しいテストハーネス（深い階層または長い名前の一時ディレクトリ、非 ANSI 名の一時ディレクトリ）の構築**を伴う。`TempDir` 直下にネストを掘る方式が素直だが、CI ランナーの `%TEMP%` が 8.3 短縮名（`RUNNER~1`）であること、テスト自身のクリーンアップも長パスを踏むことに注意が要る。

### 6. ドキュメント

- **Context**: Requirement 5.4 / 6.5 の更新先を特定する。
- **Findings**:
  - `docs/` は存在しない。`book/`（mdBook・GitHub Pages 公開）と `doc/spec/`（DSL 仕様）。
  - `book/src/lua/modules.md` は公開モジュール API（`@pasta_*`）を記述するが、**`package.path`・検索パス順序・起動シーケンスには触れていない**。
  - **設置パス長・非 ASCII パスの制限を記した利用者向けドキュメントはリポジトリ全体に存在しない**。
  - 起動シーケンスの実質的な記述はコードコメント（`loader/mod.rs:50-59`、`factory.rs:101-107`）と完了済み spec アーカイブのみ。
- **Implications**: Requirement 5.4 / 6.5 は既存節の改訂ではなく**新規記述**になる。`book/src/lua/modules.md` への節追加、または `book/src/debug/troubleshooting.md` への症状ベースの記述が候補。

---

## Requirement-to-Asset Map

| 要件 | 既存資産 | ギャップ |
|------|----------|----------|
| 1. 長パスでのモジュール解決 | `LoaderContext::absolute_search_paths()`、`package.loaded` 方式の既存モジュール登録 | **Missing**: Rust 実装 searcher そのもの。**Unknown**: `std::path::absolute` は verbatim を出さないため、Rust std 側でも 260 超で失敗し得る。verbatim 付与の要否は実測要 |
| 2. 非 ANSI パス | `encoding::to_ansi_bytes`（問題の当事者） | **Missing**: 非 ANSI 経路。**Constraint**: ANSI 変換は変換不能文字でハードエラー。フォールバックを残すなら扱いの決定が必要 |
| 3. 解決結果の不変性 | `default_lua_search_paths`、`canonicalize_chunk_name`、`chunk_name_validation_test.rs`、`byte_invariant_test.rs` | **Constraint**: 優先順位・2 パターン・チャンク名を厳密踏襲。既存テストが回帰ゲートとして流用可能。追加資産はほぼ不要 |
| 4. entry ロード失敗の可視化 | `last_load_error`（`shiori.rs:147,158`）、`LoaderError`→`MyError::Load` の Err 配管 | **Missing（重大）**: 500 応答の本番出口。`to_shiori_response()` は呼び出し元ゼロ、アクター境界で 204 に潰れる。ブリーフの境界想定（`pasta_shiori` 無変更）が崩れる |
| 5. 致命／非致命の明確化 | `factory.rs:197-213` の 3 行 | **Missing**: `main` の方針決定（Open Question 2）、旧経路 `from_loader` の扱い（Open Question 3）、判別可能なログ構造 |
| 6. 検証・既知の制限 | `#[ctor]` ガード、`copy_fixture_to_temp`、`ShioriTestEnv` | **Missing**: 長パス／非 ANSI の一時ディレクトリ構築ハーネス（前例ゼロ）、パス長制限のドキュメント（前例ゼロ） |
| 7. x86/x64 同一動作 | 既存 CI の両ターゲットビルド | **Constraint**: Windows API 呼び出しを増やす場合はポインタ幅非依存に |

---

## Architecture Pattern Evaluation

| 選択肢 | 概要 | 強み | リスク・限界 |
|--------|------|------|--------------|
| Rust searcher 前置（ブリーフ採用案） | `package.loaders` の先頭に Rust クロージャを挿入。モジュール名→候補パス解決・読み込み・命名をすべて Rust std（wide API）で行う | 長パスと非 ANSI を一度に解消。`package.loaded` 優先のため既存 `@` モジュールに無影響。挿入点が完全に空いている | チャンク名の再現が要件。LuaJIT での table 名（`package.loaders` / `package.searchers`）の実測確認が必要 |
| `package.path` の verbatim 化 | `\\?\` を `package.path` に前置 | 変更が最小 | **無効**。ANSI API は `\\?\` を受け付けない（ブリーフで却下済み） |
| 8.3 短縮名化 | `GetShortPathNameW` でパスを短縮 | 標準 searcher をそのまま使える | **不可**。8.3 生成が無効な環境あり。CI で短縮名起因の不具合実績あり（`context.rs:42-66`）。非 ANSI も解決しない |
| 内蔵スクリプトのみ zip から preload | `package.preload` にメモリ上のチャンクを登録 | 自己展開自体が不要になる | **部分解**。`scripts/` 上書き層と `cache/lua` のシーンモジュールを救えない（ブリーフで却下済み） |

---

## Design Decisions（設計フェーズへの申し送り）

### Decision: 実装配置（Option A / B / C）

- **Option A — 既存コンポーネント拡張**: `module_registry.rs` に searcher 登録関数を追加し、`factory.rs` の warn を `?` に替える。
  - ✅ 新規ファイルゼロ、最短距離。`setup_package_path` の隣という自然な位置。
  - ❌ `module_registry.rs` はモジュール登録の責務であり、パス解決＋ファイル読み込み＋命名という別責務が混入する。`oversized-file-decomposition` の「全ファイル < 600 行」方針への圧力。
- **Option B — 新規コンポーネント**: `crates/pasta_lua/src/runtime/searcher.rs`（または `loader/searcher.rs`）に、モジュール名→候補パス→読み込み→チャンク命名を完結させる。`module_registry.rs` からは登録のみ呼ぶ。
  - ✅ 責務が明確で単体テストしやすい（パス解決だけを長パス無しで検証できる）。`LoaderContext` を入力に取る純粋関数に寄せられる。
  - ❌ ファイルが 1 つ増える。`LoaderContext` との依存方向を切る設計判断が要る。
- **Option C — ハイブリッド（推奨）**: searcher 本体は Option B の新規モジュール、`factory.rs` の Err 伝搬と `module_registry.rs` からの登録は Option A の最小変更。応答面（Requirement 4.2/4.3）は Open Question 1 の結論次第で `pasta_shiori` のアクター境界に別タスクとして切る。
  - ✅ ブリーフのタスク順（B → A）と整合。B が先に入れば A の実装中の失敗も可視化された状態で作業できる。
  - ❌ Open Question 1 の結論が出るまで Requirement 4 の実装範囲が確定しない。

### Decision: チャンク名の生成方式

- **Alternatives**: (1) 現行の混在区切りをバイト単位で再現する、(2) 区切りを `/` に統一した `@<絶対パス>` を付ける（正規化後に一致）。
- **申し送り**: (2) で `canonicalize_chunk_name` を通る全消費者（ソースマップ・ブレークポイント・DAP）は無回帰になる見込み。差が出るのはエラーメッセージとスタックトレースの生表示のみ。`chunk_name_validation_test.rs` を実測ゲートとして設計フェーズで判定すること。**verbatim プレフィックスはチャンク名に含めない**（読み込み用パスと命名用パスを分離する）。

---

## Risks & Mitigations

- **【高】500 経路の欠損**: entry を `?` にしても利用者から見た症状（無言）が変わらない。→ Open Question 1 を要件ディスカッションの最優先で決着させる。決着前に実装へ進むと「修正したのに直っていない」状態になる。
- **【中】Rust std も 260 で落ちる可能性**: `std::path::absolute` は verbatim を付けない。ホストが longPathAware でない場合、Rust 側の `File::open` も失敗し得る。→ 設計フェーズで実機実測。verbatim 付与が必要なら、読み込み用と命名用のパスを分離する。
- **【中】長パステストが CI でのみ壊れる**: CI ランナーの `%TEMP%` は 8.3 短縮名（`RUNNER~1`）。前例あり（`context.rs:42-66`）。→ テスト用ディレクトリの組み立てで `canonicalize` を使わず `std::path::absolute` に揃える。クリーンアップ自体が長パスを踏む点にも注意。
- **【中】非 ANSI 環境が CI に無い**: 変換不能文字はロケール依存（日本語ロケールでは日本語が通ってしまう）。→ ロケール非依存に失敗する文字集合（例: 日本語 CP932 環境での한글・キリル・絵文字）を選ぶ。
- **【低】チャンク名回帰**: → `chunk_name_validation_test.rs` と `byte_invariant_test.rs` を実装前に走らせ、ベースラインを固定してから着手。
- **【低】新規テストの `PASTA_DEBUG` 汚染**: 症状がモジュール未検出として現れ、本件の症状と紛らわしい。→ 新規テストファイルに `#[ctor]` ガードを必ず置く（Requirement 6.4）。

---

## Research Needed（設計フェーズで解消）

1. LuaJIT 2.1 + mlua 0.11 `luajit52` において、searcher テーブルが `package.loaders` か `package.searchers` か（両方が同一 table を指すか）を実機確認する。
2. 対象環境で `std::fs::File::open` が 260 超の非 verbatim パスを開けるか。開けない場合の verbatim 付与方式と、ネットワークパス（UNC）の扱い。
3. `package.path` の ANSI 変換を残す場合、変換不能時にエントリ単位でスキップするのか全体を lossy にするのか（Open Question 4 と連動）。
4. 標準 searcher をフォールバックに残した場合、Rust searcher が先に成功する限り標準 searcher のエラー文言は表に出ないか（`require` のエラー集約挙動の実測）。
5. 長パス一時ディレクトリの構築方式（深いネスト vs 長い単一名）と、Windows のディレクトリ削除 API の長パス耐性。

---

## Effort & Risk

| 範囲 | Effort | Risk | 根拠 |
|------|--------|------|------|
| 欠陥 B: entry の Err 伝搬（Requirement 4.1 / 5.x） | **S** | Low | `factory.rs` の 3 行。既存の Err 配管に乗るだけ |
| 500 応答経路の復旧（Requirement 4.2 / 4.3 / 4.5） | **M** | **High** | アクター境界の契約変更。`pasta-actor-runtime` が確立した設計に触れる。スコープ判断が未決 |
| 欠陥 A: Rust searcher（Requirement 1 / 2 / 3） | **M** | Medium | 挿入点は空いているが、チャンク名一致・verbatim・LuaJIT 実測の 3 つの未知がある |
| テスト基盤（Requirement 6.1 / 6.2） | **M** | Medium | 前例ゼロ。CI のパス形式依存で壊れやすい |
| ドキュメント（Requirement 5.4 / 6.5） | **S** | Low | 新規記述だが分量は小さい |
| **合計** | **L（1〜2 週）** | **Medium〜High** | Open Question 1 の結論で上下する |

---

## Open Questions（`requirements.md` と同一・ディスカッションで確定）

本ギャップ分析で新たに判明・更新した項目を含む。詳細は `requirements.md` の「Open Questions」節を参照。

1. **【重大】** 500 + `X-ERROR-REASON` 応答経路の復旧を本仕様に含めるか、別 spec へ切り出すか。
2. `main` のロード失敗を致命扱いにするか。
3. 旧経路 `from_loader` の同型の無言化を同時に是正するか。
4. 標準 searcher をフォールバックとして残すか撤去するか（`package.path` の ANSI 変換の扱いを含む）。
5. サポートするパス長の上限を要件として明示するか。
6. 長パス・非 ANSI テストの CI 実行範囲。
7. チャンク識別子の「同一形式」の厳密さ（生表示の差を許容するか）。
8. ドキュメント更新の対象範囲。

---

## References

- GitHub issue [#29](https://github.com/ekicyou/pasta/issues/29)（長パスで require 失敗）・[#30](https://github.com/ekicyou/pasta/issues/30)（ロード失敗の無言化）
- `.kiro/specs/lua-require-robustness/brief.md` — discovery 決定事項・却下した代替案
- `.kiro/specs/completed/load-error-logging/requirements.md` — 500 + `X-ERROR-REASON` 機構の元仕様
- `.kiro/specs/completed/lua-module-path-resolution/requirements.md` — 検索パス順序・`require` 統一・起動順序の元仕様
- `.kiro/specs/completed/pasta-source-map/` — チャンク名依存の元仕様
- `crates/pasta_lua/src/loader/context.rs:42-66` — `canonicalize` を避ける理由（8.3 短縮名・CI 実績）
- `crates/pasta_shiori/src/windows.rs:178-183` — 本番経路が 500 でなく 204 を返す契約の記述
