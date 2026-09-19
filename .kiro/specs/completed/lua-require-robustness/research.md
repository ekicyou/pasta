# Research & Gap Analysis: lua-require-robustness

> **生成フェーズ**: `/kiro-validate-gap`（要件生成直後）＋ `/kiro-spec-design`（設計フェーズの実測・統合・設計判断を追記）
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

### Decision: 標準 searcher フォールバックの去就と `package.path` の ANSI 変換

- **由来**: Open Question 4（要件ディスカッションで「設計判断」に分類）。
- **Alternatives**:
  1. **標準 searcher を残す**: Rust searcher を前置し、標準 searcher はそのまま後段に残す。ただし `setup_package_path` の ANSI 変換は非 ANSI パスで `InvalidInput` を返すため、Requirement 2 を満たすには「変換不能エントリのみスキップ」「lossy 変換」「`package.path` を空にする」のいずれかへ変更が必要。
  2. **標準 searcher を撤去し `package.path` も設定しない**: 解決経路が Rust searcher 1 本に収束し、失敗時のエラー文言も一元化できる。ただしゴースト作者が `package.path` を自前で追記して使っている場合に影響が出る（現状そのような用例は未確認）。
- **申し送り**: 設計フェーズで (a) LuaJIT の searcher テーブル実測（Research Needed 1）、(b) `require` のエラー集約挙動（Research Needed 4）、(c) `package.path` 追記の実用例調査 の 3 点を確認してから決定する。Requirement 2 の達成には、少なくとも「非 ANSI パスで `setup_package_path` がロード全体を落とさない」ことが必須条件である。

### 要件ディスカッションの決定と設計への申し送り

要件ディスカッションで確定した内容と、それに伴う設計上の着眼点。**本節以前の記述中の Requirement 4.x の番号は再編前のもの**である（旧 4.2→新 4.3、旧 4.3→新 4.5、旧 4.4→新 4.7、旧 4.5→新 4.8、旧 4.6→新 4.9。新 4.2 / 4.4 / 4.6 / 4.10 は追加）。

- **#1 500 応答経路の復旧（Requirement 4）**: 本仕様に含める。対象はリクエスト処理エラー全般。
  - 実装点は `crates/pasta_shiori/src/actor/thread.rs:184-196` の `Err(_)` 分岐 1 箇所。reply を drop する代わりに `Reply::Value(e.to_shiori_response())` を送れば、`last_load_error` → `MyError::Load` → 500 の既存配管がそのまま本番へ届く。`marshaling.rs` の drop／Timeout／try_send 失敗 → 204 は安全網として無変更（`pasta-actor-runtime` R5.3 / R5.6 / R5.7 と非干渉）。
  - **新規発見**: `to_shiori_response()` はエラーメッセージを無加工で `X-ERROR-REASON` へ埋め込む。LuaJIT の `module 'X' not found:` は**複数行**（候補パスごとに `\n\tno file ...`）のため、そのままでは SHIORI 応答のヘッダ構造が壊れる。単一行化（Requirement 4.6）が必須。ログ側は複数行のまま欠落なく残す（Requirement 4.7）。
  - NOTIFY は応答経路を持たず即 204（プロトコル上不可避）。可視化は GET で行う（Requirement 4.3 / 4.8 は GET 限定）。
- **#2 `main` は致命（Requirement 5.2）**: `crates/pasta_lua/pasta_scripts/main.lua`（何もしない既定実装）が常に自己展開されるため「不在」は正常状態ではない。`factory.rs:197-201` を `?` 伝搬へ。
- **#3 旧経路 `from_loader`（Requirement 5.6）**: 是正する。利用箇所は `tests/runtime/encoding_test.rs` と `tests/runtime/runtime_api_test.rs` の計 9 箇所のみ。旧経路は `scripts/pasta/shiori/entry.lua` をファイル直読みするため、致命化するとフィクスチャに entry が無いテストが落ちる可能性がある。**修正 vs 撤去（テストを `from_loader_with_scene_dic` へ移行）は設計で決定**。
- **#4 パス長上限なし（Requirement 1.6）**。
- **#5 チャンク識別子はバイト単位で同一（Requirement 3.4 / 3.7 / 3.8）**: 現行の識別子は「`generate_package_path` が作るテンプレート（区切り `/`）の `?` へ、`.` を `\` に置換したモジュール名を代入したもの」。Rust searcher が同じテンプレート文字列から同じ代入で候補を作れば、探索順序（Requirement 3.1〜3.3）と識別子の両方が自動的に一致する。読み込み用パス（必要なら verbatim）と命名用パスは分離する。「Decision: チャンク名の生成方式」の選択肢は **(1) バイト単位再現で確定**。
- **#6 テストは常時実行・ロケール非依存（Requirement 6.6 / 6.7）**。
- **#7 ドキュメントは `book/`（Requirement 5.4 / 6.5 / 6.8）**: ページ構成は設計で決定。
- **#8 x86 テストを CI で実行（Requirement 7.3）**: `.github/workflows/build.yml:48` の `cargo test --all` はターゲット指定が無く、x86 ジョブでもホスト（x64）のテストが走っている。x86 ジョブを `--target i686-pc-windows-msvc` 付きへ改める。CI 時間の増加と、本仕様と無関係な x86 固有の失敗が表面化する可能性に注意（多数出た場合は報告のうえ縮退を判断）。
- **スコープ外の観測（別タスク候補）**: `actor_teardown_test.rs:241` の `repeated_reload_tears_down_and_does_not_leak` が PR #32（ドキュメントのみの変更）の CI で失敗し、直後の main では成功している（flaky の疑い）。また CI 失敗でもマージ可能な状態にある。本仕様では扱わないが、x86 テスト追加時に同テストが再発する可能性がある。

## Risks & Mitigations

- **【高】500 経路の欠損**: entry を `?` にしても利用者から見た症状（無言）が変わらない。→ Open Question 1 を要件ディスカッションの最優先で決着させる。決着前に実装へ進むと「修正したのに直っていない」状態になる。
- **【中】Rust std も 260 で落ちる可能性**: `std::path::absolute` は verbatim を付けない。ホストが longPathAware でない場合、Rust 側の `File::open` も失敗し得る。→ 設計フェーズで実機実測。verbatim 付与が必要なら、読み込み用と命名用のパスを分離する。**→ 設計フェーズで解消（M2）**: std が内部で verbatim を自動付与するため落ちない。分離も不要。
- **【中・設計フェーズで追加】DLL 境界の ANSI パス**: SHIORI `load` はディレクトリパスを ANSI で受け取るため、非 ANSI 設置パスは pasta_lua に届く前に欠損し得る。→ `loadu`（UTF-8）対応の要否を設計ディスカッションで決定する（design.md Open Questions 1）。
- **【中】長パステストが CI でのみ壊れる**: CI ランナーの `%TEMP%` は 8.3 短縮名（`RUNNER~1`）。前例あり（`context.rs:42-66`）。→ テスト用ディレクトリの組み立てで `canonicalize` を使わず `std::path::absolute` に揃える。クリーンアップ自体が長パスを踏む点にも注意。
- **【中】非 ANSI 環境が CI に無い**: 変換不能文字はロケール依存（日本語ロケールでは日本語が通ってしまう）。→ ロケール非依存に失敗する文字集合（例: 日本語 CP932 環境での한글・キリル・絵文字）を選ぶ。
- **【低】チャンク名回帰**: → `chunk_name_validation_test.rs` と `byte_invariant_test.rs` を実装前に走らせ、ベースラインを固定してから着手。
- **【低】新規テストの `PASTA_DEBUG` 汚染**: 症状がモジュール未検出として現れ、本件の症状と紛らわしい。→ 新規テストファイルに `#[ctor]` ガードを必ず置く（Requirement 6.4）。

---

## Research Needed（設計フェーズで解消）

> **状態**: 全 6 項目を設計フェーズで解消済み。結果は次節「設計フェーズの実測」を参照。

1. ~~LuaJIT 2.1 + mlua 0.11 `luajit52` の searcher テーブル名~~ → **解消（M1）**: `package.loaders` と `package.searchers` は同一テーブル。
2. ~~`std::fs::File::open` が 260 超の非 verbatim パスを開けるか・verbatim 付与方式・UNC~~ → **解消（M2）**: 開ける。std が内部で `\\?\`（UNC は `\\?\UNC\`）を自動付与するため明示付与は不要。
3. ~~`package.path` の ANSI 変換を残す場合の変換不能時の扱い~~ → **解消（設計判断）**: ANSI 変換自体を撤去し UTF-8 で設定する。
4. ~~標準 searcher を残した場合のエラー集約挙動~~ → **解消（M4）**: 成功時は表に出ないが、未検出時は `no file` 行が二重になる。置換を採用。
5. ~~長パス一時ディレクトリの構築方式と削除の長パス耐性~~ → **解消（M2 / M5）**: 深いネスト方式。`remove_dir_all` は非 verbatim の長パスで成功。
6. ~~非 ASCII・ANSI 表現可能パスでの現行チャンク識別子のバイト表現~~ → **解消（M6）**: ANSI バイト列（不正な UTF-8）であり、ソースマップ側キー（UTF-8）と一致しない。

---

## 設計フェーズの実測

> **生成フェーズ**: `/kiro-spec-design`（2026-09-18）
> **環境**: Windows 11 Pro 10.0.26200 / rustc 1.98.1 / mlua 0.11.6（`luajit52`, `vendored`）/ luajit-src 210.6.6 / システム ACP = 932 / `LongPathsEnabled` = 1（ただし実験 exe は longPathAware マニフェストを持たない）
> **方法**: リポジトリ外のスクラッチ cargo プロジェクト（mlua のみ依存）で計測。x64 と i686 の両ターゲットで同一結果。実験コードはリポジトリに残していない。

### M1. searcher テーブル

- `type(package.loaders)` = `table`、`type(package.searchers)` = `table`、`rawequal(package.loaders, package.searchers)` = **true**、`#package.loaders` = **4**（preload / Lua / C / C-root）。
- ソース裏付け: `luajit2/src/lib_package.c` の `luaopen_package` は loaders テーブルを作成後、`#if LJ_52` で同じテーブルを `searchers` にも set する。`lj_cf_package_require` は `LUA_ENVIRONINDEX`（= package テーブル）の `"loaders"` フィールドを require のたびに参照する。
- **Implication**: テーブルの**インプレース書き換え**（`loaders[2] = f`）なら両名称から見える。テーブル自体の差し替えは不要。

### M2. 長パスと Rust std

- 332 文字のディレクトリを非 verbatim パスで `create_dir_all` → 成功。345 文字のファイルへ `write` → 成功。
- 区切り混在形（`C:/…/deep/mod_long.lua`、`C:/…/deep/sub\mod_x.lua`）で `File::open` → 成功。
- **同一プロセス・同一パス**で LuaJIT 標準 `require` は `module 'mod_long' not found`（欠陥 A の再現）。すなわちプロセスは長パス非対応であり、std 側の成功は OS 設定ではなく std の機構による。
- ソース裏付け: `library/std/src/sys/path/windows.rs` の `maybe_verbatim` → `get_long_path`。248 文字（`LEGACY_MAX_PATH`）以上、またはドライブ絶対でないパスは `GetFullPathNameW` で絶対化・正規化（`/` → `\`）したうえで `\\?\`（`\\server\share` は `\\?\UNC\`）を前置する。呼び出し側へ verbatim 形は返らない。
- **Implication**: 読み込み用パスと命名用パスの分離は不要。Requirement 3.7 は構造的に満たされる。ネットワークパス（UNC）も std が処理する。

### M3. チャンク識別子のバイト同一性（ASCII 短パス）

- 同一の `package.path`（`<root>/?.lua;<root>/?/init.lua`）で `pasta.shiori.m` と `pasta`（`init.lua`）を、標準 searcher と Rust searcher プロトタイプ（テンプレートの `?` を、モジュール名の `.` を `\` に置換した文字列で置き換え、`lua.load(bytes).set_name("@" + candidate)`）でロード。
- `debug.getinfo(1,'S').source`・`short_src`・チャンクへの引数（`...` = モジュール名、個数 1）が**完全一致**。形は `@C:/…/short/pasta\shiori\m.lua`（前置部 `/`・展開部 `\` の混在）。
- ソース裏付け: `lj_load.c` `luaL_loadfilex` は `lua_pushfstring(L, "@%s", filename)` をチャンク名にする。`require` は loader を `name` 1 引数で呼ぶ。

### M4. `require` のエラー集約とエラー値の型

- 未検出時のメッセージは `module 'X' not found:` + 各 searcher が返した**文字列**の連結（文字列なら連結、それ以外は捨てる）。
- 構成別の未検出メッセージ:
  - 標準のみ: `no field package.preload[…]` + `no file` × 2（+ cpath 行）。
  - Rust 前置＋標準残置: `no file` 行が **4 行（二重）**。
  - Rust で `loaders[2]` を置換: 標準のみと**バイト同一**。
- ロード失敗（構文エラー）を Rust コールバックの `Err(mlua::Error::RuntimeError)` で返すと、Lua の `pcall(require, "bad")` が受け取る値は **userdata**（mlua のエラーオブジェクト）になり、文言にも `runtime error:` / `syntax error:` 接頭辞と traceback が付く。標準は **string**。
- Rust 関数が `(nil, msg, true)` を返し、Lua 側ラッパが `error(msg, 0)` する構成では、未検出・構文エラー・実行時エラーの 3 種とも `type(e) .. "::" .. tostring(e)` が標準と**バイト同一**（構文エラー本文は `mlua::Error::SyntaxError { message }` の `message` をそのまま使う）。
- shebang（`#!…`）行と UTF-8 BOM を先頭に持つモジュールは、標準・Rust searcher とも同じくロードできる（LuaJIT はレキサ初期化で両者をスキップするため、バッファ経由のロードでも同じ）。

### M5. 後始末

- 300 文字超のツリーと非 ANSI 名ツリーを含むルートへの `std::fs::remove_dir_all`（非 verbatim）→ 成功。`tempfile::TempDir` の drop と同経路。

### M6. 非 ASCII パスの現行チャンク識別子

- `…/日本語フォルダ/?.lua` を (a) UTF-8 のまま、(b) CP932 へ変換して `package.path` に設定し、標準 `require` でロード。
  - (a) は `module 'm' not found`（narrow `fopen` が UTF-8 バイト列を CP932 として解釈するため）。
  - (b) は成功するが、`source` は `@` + **CP932 バイト列**（`93 fa 96 7b 8c ea …`）。UTF-8 として不正で、`@` + UTF-8 パスと一致しない。
- デバッグフックは `source` を lossy な文字列として受け取るため、(b) の識別子は U+FFFD を含む文字列になり、Rust 文字列（UTF-8）由来のソースマップキーと一致しない。**現行は日本語パス上でブレークポイントが効いていない可能性が高い**（エンドツーエンドの実機確認まではしていない）。
- Rust searcher + UTF-8 の `package.path` では `source` = `@` + UTF-8 パス（一致）。
- 複数文字体系混在名（`日本語_한글_Кириллица_ελληνικά_😀`）は CP932 へ変換不能。この名前の 300 文字超ツリーで Rust searcher は成功し、`source` に `\\?\` は含まれず、未検出メッセージは当該名を欠落なく含む。

### M7. x86

- i686-pc-windows-msvc でビルドした同じ実験で M1〜M6 の結果が x64 と一致。

### M8. x86 ターゲットでの既存テストのベースライン

- 変更前のワークツリーで `cargo test --all --target i686-pc-windows-msvc --no-fail-fast` を実行（開発機・ACP 932）。**90 テストバイナリすべて成功（passed 2127 / failed 0 / ignored 11）**。
- **Implication**: Requirement 7.3 の「無関係な x86 固有の失敗が多数表面化する」リスクは、少なくとも開発機では顕在化しない。CI ランナー固有の差（ロケール・8.3 短縮名の `%TEMP%`）は残るため、CI 変更は実装の早い段階で入れて確認するのが安全。
- 副作用の注意: テスト実行が `crates/pasta_lua/tests/fixtures/sample.generated.lua` を改行コードのみ異なる内容で書き戻す（内容差分なし）。計測後に復元済み。

### 付随する発見

- **SHIORI `load` のパスは ANSI**: `crates/pasta_shiori/src/windows.rs` の `load` は `hdir` を ANSI としてデコードする。UKADOC「DLL共通仕様」には UTF-8 でパスを渡す `loadu`（SSP 2.6.92・2025-01-16 以降、`load` より優先して呼ばれる）があるが、pasta.dll は未エクスポート。非 ANSI 設置パスは DLL 境界で欠損する可能性が高く、Requirement 2 のエンドツーエンド達成には `loadu` 対応が要る見込み（design.md Open Questions 1）。
- **スクラッチビルドが長パスで失敗**: セッションのスクラッチパッド（約 170 文字）配下で i686 をビルドすると、LuaJIT の `msvcbuild.bat` がパス長で失敗した。ターゲットディレクトリを短い場所へ逃がして回避。開発ツールチェーン自体が本件と同種の制約を持つ実例である。CI はリポジトリ直下の `target/` を使うため同じ問題が起きる可能性は低いが、x86 テスト有効化時に留意する。
- **非 ASCII の `.pasta` ファイル名**: モジュール名（UTF-8）と `package.path`（ANSI）の混在により、現行では `pasta.scene.<日本語名>` の解決が失敗している可能性がある（未実測）。UTF-8 統一で解消される見込み。

---

## Design Synthesis（設計フェーズ）

### Generalization

- Requirement 1（長パス）と Requirement 2（非 ANSI）は「narrow `fopen` への依存」という同一問題の 2 つの現れであり、「Lua ファイル searcher のファイルオープンを Rust std へ置き換える」1 つの機構で同時に解消する。
- Requirement 4（entry 失敗の無言化）・5.2（main）・5.6（旧経路）・4.4（リクエスト処理エラー）は「`Err` を 204／warn に読み替える経路」という同一問題であり、(a) 起動モジュールの一律致命化、(b) アクターの `Err` 分岐 1 箇所、の 2 点で解消する。

### Build vs. Adopt

- **Adopt**: 長パス対応は Rust std の内部 verbatim 付与をそのまま採用（自前の `\\?\` 付与・Windows API 直叩きは不要と実測で確定）。エラー文脈は mlua の `ErrorContext`（`WithContext`）を採用。検索規則は LuaJIT `searchpath` の仕様をそのまま踏襲。
- **Build**: Lua ファイル searcher 本体のみ（mlua / LuaJIT に wide-API 版の searcher は存在しない）。
- **Rejected**: LuaJIT へのパッチ（`fopen` → `_wfopen`）— vendored ビルドの保守負担が大きく、`package.path` の文字コード問題も別途残る。

### Simplification

- 読み込み用／命名用パスの分離（当初想定）→ **不要**（M2）。
- `LoaderContext` を searcher へ渡す設計 → **不要**。`package.path` を実行時に読めば状態ゼロで両起動経路に効き、標準の意味論も保てる。
- 標準 searcher のフォールバック残置 → **不要**（M4）。
- 致命／継続を表す enum やテーブル → **不要**。起動モジュール 3 種がすべて致命のため、ヘルパ関数 1 個とログフィールド `fatal` で足りる。
- `from_loader` の撤去 → 見送り。是正（`?` 伝搬 2 箇所）の方が差分が小さい。

---

## 設計判断（設計フェーズ）

### Decision: 標準 Lua ファイル searcher を置換する

- **Context**: brief は「前置＋標準をフォールバックとして残す」。本書の申し送りでは設計判断。
- **Alternatives**: (1) 前置＋残置、(2) `loaders[2]` を置換、(3) 標準を残し `package.path` を空にする。
- **Selected**: (2)。
- **Rationale**: M4 のとおり (1) は未検出メッセージが二重化し、非 ASCII パスでは後段が無意味な候補を出す。Rust searcher が開けないファイルを narrow `fopen` が開ける場面は無い。(3) は `package.path` という単一情報源を失う。(2) は ASCII パスで未検出メッセージまでバイト同一。
- **Trade-offs**: brief の記述からの逸脱（design.md Open Questions 2 で確認）。
- **Follow-up**: `module_searcher_test.rs` の標準版 VM とのバイト比較を恒久ゲートにする。

### Decision: `package.path` を UTF-8 で設定し ANSI 変換を撤去する

- **Selected**: `generate_package_path()` の文字列を無変換で設定。`generate_package_path_bytes` は撤去。
- **Rationale**: 非 ANSI パスで `setup_package_path` がロードを落とす経路を消す（Requirement 2 の必須条件）。チャンク識別子が UTF-8 になり、ソースマップキーと一致する（M6・Requirement 3.8）。ASCII パスではバイト同一（Requirement 3.4）。
- **Trade-offs**: ANSI バイト列を `package.path` に自前追記する作者コードは解決不能になる（用例未確認・design.md Open Questions 3）。`package.searchpath` を `package.path` に対して呼ぶ作者コードは、非 ASCII パスで従来（ANSI）と結果が変わる（対象外の narrow API）。

### Decision: ロード失敗は Lua 文字列エラーとして送出する

- **Rationale**: M4。`pcall(require, …)` の戻り値を文字列として扱う Lua コードの前提を保つ。

### Decision: 旧経路 `from_loader` は是正して残す

- **Alternatives**: (1) 撤去してテスト 9 箇所を本番経路へ移行、(2) 失敗の `?` 伝搬へ是正。
- **Selected**: (2)。`entry.lua` の**不在**はスキップのまま（仮定）。
- **Rationale**: 9 箇所中 7 箇所は実在しない `/test/path` 等を base にした軽量ランタイム用途で、本番経路（`main` / `entry` / `scene_dic` が致命）へは移行できない。撤去は代替コンストラクタの新設を伴い差分が増える。
- **Follow-up**: design.md Open Questions 4。

### Decision: ドキュメントは `book/src/reference/startup.md` の 1 ページに集約

- **Rationale**: 検索パス・起動シーケンス・既知の制限・切り分け手順は相互参照が密で、既存の `debug/troubleshooting.md`（デバッガ接続専用）とは読者の入口が異なる。`debug/troubleshooting.md` からは誘導リンクのみ置く。

---

## 設計ディスカッションの決定

- **#1 `loadu` を本仕様へ含める**: 従来の `load` は設置パスを ANSI で受けるため、ANSI 外の文字はホスト側で欠落し、本番では Requirement 2 が成立しない。DLL 共通仕様（UKADOC）の `loadu`（UTF-8・SSP 2.6.92 以降・`load` より優先）を追加する。既存の `ShioriString::to_utf8_str`（`util/hglobal/mod.rs:165`）と `lifecycle::spawn_actor` の再利用で足り、エクスポートは `load` と同じ `no_mangle`（`.def` なし）。要件 2.4〜2.6 / 6.9 / 6.10 を追加。
- **#2 searcher は置換・設置時にレイアウト検証**: `package.loaders` が 4 要素でなければ `Err`（起動失敗として可視化）。冪等の契約は「VM 構築時に 1 回だけ呼ぶ」へ弱めた。
- **#3 `X-ERROR-REASON`**: `stack traceback:` 以降を落とす。`no file` の候補パス列は根本原因なので残す。長さ上限なし。要件 4.11 を追加。
- **#4 `request` の UTF-8 デコード失敗 → 204 は無変更**: 要件 4.10 の安全網に明示。ホストが通常運転中に非 UTF-8 を送る場面の有無が未確認で、変更は 3.5（バイト不変）を破るおそれがある。
- **#5 `from_loader` の `entry.lua` 不在はスキップ**: 旧経路は SHIORI 応答モジュールを任意とする軽量構築経路。存在して失敗した場合のみ致命。
- **#6〜#9**: `package.path` の UTF-8 解釈は `book/` に明記／`package.cpath` の候補行は対象外／`load` 失敗後のホスト挙動は実機確認して `book/` へ反映／非 ASCII の `.pasta` ファイル名をテストに含める。

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

## Open Questions

設計ディスカッションで全項目を解決済み。決定は上記「設計ディスカッションの決定」および `design.md` の「Open Questions」節の表を参照。

---

## References

- GitHub issue [#29](https://github.com/ekicyou/pasta/issues/29)（長パスで require 失敗）・[#30](https://github.com/ekicyou/pasta/issues/30)（ロード失敗の無言化）
- `.kiro/specs/lua-require-robustness/brief.md` — discovery 決定事項・却下した代替案
- `.kiro/specs/completed/load-error-logging/requirements.md` — 500 + `X-ERROR-REASON` 機構の元仕様
- `.kiro/specs/completed/lua-module-path-resolution/requirements.md` — 検索パス順序・`require` 統一・起動順序の元仕様
- `.kiro/specs/completed/pasta-source-map/` — チャンク名依存の元仕様
- `crates/pasta_lua/src/loader/context.rs:42-66` — `canonicalize` を避ける理由（8.3 短縮名・CI 実績）
- `crates/pasta_shiori/src/windows.rs:178-183` — 本番経路が 500 でなく 204 を返す契約の記述
