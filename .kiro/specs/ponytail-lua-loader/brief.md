# Brief: ponytail-lua-loader

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域に `/ponytail:ponytail-audit` をやり直す。領域は `crates/pasta_lua/src` の `loader/`・`runtime/`・`presentation/`・`encoding/` と、それらの `lib.rs` の再輸出。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

ゴーストの作者が、読み込みとセーブの端で困る。

- 読めないセーブファイルが、空のデータで上書きされて失われる。
- 古い日付の辞書に戻しても、変換のキャッシュが古い結果を動かし続ける。
- 同じモジュール名になる 2 つの辞書の片方が、黙って消える。

加えて、使われない抽象（約 900 行）が読み手の手間を増やしている。

## Current State

2026-10-19 の監査で確かめた（cargo test と clippy は走らせていない。「使われない」の判断は、リポジトリ全体の grep による）。

1. **読めないセーブを空で上書きして失う**（Must fix）
   - 場所：`runtime/persistence.rs` L166-196（`load_impl`）、`runtime/lifecycle.rs` の Drop。
   - NotFound 以外の読み込みの誤りは、`{}` を返す。たとえば次の 2 つ。
     - 手で書き換えた save.json に、カンマが余っている。
     - OneDrive やウイルス対策ソフトのロックで、PermissionDenied になる。
   - unload の Drop → `save_to_file` が、その `{}` で原本を rename して上書きする。
   - マニュアル（`book/src/lua/modules/pasta-persistence.md` の :57・:91）は、「load は `{}` を返す」「書き込みに失敗したら原本が残る」としか書いていない。
2. **キャッシュの判定が `source_mtime > cache_mtime`**
   - 場所：`loader/cache.rs` L137-165。
   - バックアップから古い日付の .pasta を戻したとき、Explorer でコピーしたとき、NAR を展開したときに、古い変換結果が黙って動き続ける。
3. **同じモジュール名になる辞書が、後勝ちで黙って消える**
   - 場所：`loader/process.rs` L77-97・L111-274。
   - 例：`dic/会話-朝.pasta` と `dic/会話_朝.pasta` は、どちらも `pasta.scene.会話_朝` になる。同名の `.lua` も同じ。
   - `pasta_check validate` は、これを `ModulePastaCollision`・`ModuleLuaCollision` の誤りにする。同じ規則で、結果が 2 通りに分かれている。
4. **finalize 前の `@pasta_search` が、今回変換したファイルだけを見る**
   - 場所：`loader/process.rs` L111・L182、`runtime/factory.rs` L170・L213-221。
   - main.lua で `SEARCH:search_scene("OnBoot")` と書くと、編集の直後の起動でだけ見つかり、次の起動からは nil になる。
5. **パスに `;` を含むゴーストのフォルダで起動しない**
   - 場所：`loader/context.rs` L101-116、`runtime/searcher.rs` の `candidate_paths`。
   - `C:\SSP\ghost\a;b\ghost\master` のようなパスで、`package.path` が割れる。`require "pasta"` が module not found になり、本当の原因が隠れる。
6. **使われない抽象：`presentation/` と `RendererInjection`**
   - 場所：`presentation/`（`mod.rs` 292 行・`marker.rs` 126 行）、`runtime/renderer_injection.rs`（213 行）、`runtime/module_registry.rs` の `register_sakura_script_module`。
   - `RenderBoundary` は実装が 1 つで、本番から呼ばれない。`RendererInjection` は常に `default()`。
7. **テストだけが使う古い構築の道**
   - 場所：`runtime/factory.rs` の L46-105 `from_loader`・L274-287 `load_scene_dic`・L149 の未使用の `_scene_dic_path`、`loader/mod.rs` L337 の `TranspileResult`。
   - 使うのは `tests/runtime/encoding_test.rs`・`runtime_api_test.rs` だけ。
8. **小物**
   - 孤立したキャッシュの警告が 2 回出る（`cache.rs` の ~L405 と、`loader/mod.rs` L193-198 のループ）。
   - `Encoder` は実装が 1 つで、`Encoding::OEM` はテストだけが使う（`encoding/mod.rs`）。

## Desired Outcome

- 読めないセーブは原本が残り、作者に名前つきで知らされる。
- 辞書の日付が戻っても、変換し直される。
- 同じモジュール名の衝突は、2 つのファイル名つきの読み込みエラーになる。`pasta_check` と同じ規則。
- finalize 前の検索は、キャッシュの状態によらず毎回同じになる。
- `;` や `?` を含むパスは、はっきりした読み込みエラーになる。
- 使われない抽象と、古い構築の道が無くなる。

## Approach

監査の最小の直しを採る。

- 読めないセーブは、`<name>.broken` へ rename してから `{}` を返す（2026-10-19 開発者の決定）。
- キャッシュは、保存の後で `File::set_modified(source_mtime)` し、判定を `!=` にする。
- 衝突は、`discover_all_files` で `LoaderError` にする（`init.pasta` を断るのと同じ形）。
- finalize 前の検索は、merge を消して空にする。
- パスは、`load_with_config` の入口で `;`・`?` を断る。
- `presentation/`・`renderer_injection.rs` は消し、`sakura_script::register` を直接呼ぶ。
- テストは `PastaLoader::load` と一時ゴーストに移す。

## Scope

- **In**: 上の 1〜8 と、ギャップ分析で増えた同じ領域の指摘。マニュアルの次の頁も直す。
  - `pasta-persistence.md`
  - `internals/transpiler.md`（:336-337）
  - `internals/loader.md`
  - `reference/startup.md`
  - `internals/shiori.md`（L25・L87-88・L444-455）
  - `internals/talk-output.md`（L28・L341）
  - `internals/execution-model.md`（:43）
  - `.kiro/steering/structure.md`
- **Out**: 次の spec が持つもの。
  - `code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs`（`ponytail-lua-transpiler`）
  - `src/debug/`（`ponytail-lua-debug`）
  - `pasta_scripts/`（`ponytail-lua-scripts`）
  - `search/context.rs`（`ponytail-core-registry`）

## Boundary Candidates

- セーブの保護（persistence）
- キャッシュと発見（cache・process・context）
- 構築の道と抽象の撤去（factory・presentation・renderer_injection・module_registry）
- 文字コード（encoding）

## Out of Boundary

- トランスパイラの出力（`ponytail-lua-transpiler`）
- `pasta_shiori` の `enc.rs`（`ponytail-shiori` が持つ）
- `sakura_script/`（`talk-grapheme-clusters` が持つ）

## Upstream / Downstream

- **Upstream**: `pasta_core`・`pasta_dsl`
- **Downstream**:
  - `pasta_shiori`（読み込みのエラーの理由）
  - `pasta_check`（衝突の規則を揃える相手）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-transpiler`（`crates/pasta_lua/src/lib.rs` の再輸出を共有する。行が別なので、後から入る側が合わせる）
  - `talk-grapheme-clusters`（`sakura_script/` は触らない）
  - `lua-cmodule-robustness`（完了。`runtime/searcher.rs` の FFI の道は変えない）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が 1 回だけ書く。
- 挙動を変える直しは、同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` を走らせる。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lua -p pasta_shiori`。
- cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。`PASTA_DEBUG` を読む新しいテストには、それを中和する守りを付ける。
- pasta_lua のテストは `sample.generated.lua` の改行を書き換える。`git checkout --` で戻し、コミットに混ぜない。

## 申し送り（scene-play-event より、2026-10-22 完了時）

- `runtime/mod.rs`（`with_config_and_source_map`）がソースの対応表を持つ条件を、待ち受けが立ったか（`debug_handle.is_some()`）から、解決した設定のデバッグの有効（`config.debug.enabled`）へ変えた。`factory.rs` の Step 7（索引の構築）もこれで、待ち受けが無くても走る。`from_loader_with_scene_dic` は `@pasta_debug` を登録するようになった（`register_finalize_scene` の直前）。`runtime/` を整理するときは、この 2 つの条件（設定の有効・待ち受けの有無）を混ぜないこと。参照先は `.kiro/specs/completed/scene-play-event/design.md` の SourceMapRetention・DebugLuaModule。
