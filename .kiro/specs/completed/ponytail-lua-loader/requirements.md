# Requirements Document

## Project Description (Input)
ゴーストの作者が、辞書の読み込みの端で困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。古い日付の .pasta に戻しても変換のキャッシュが古い結果を動かし続け、同じモジュール名になる 2 つの辞書の片方が黙って消え、finalize 前の `@pasta_search` の結果が起動ごとに変わり、`;` を含むフォルダでは本当の原因が隠れたまま起動しない。加えて、使われない抽象（`presentation/`・`RendererInjection` など約 900 行）とテストだけが使う古い構築の道が、読み手の手間を増やしている。`crates/pasta_lua/src` の `loader/`・`runtime/`・`presentation/`・`encoding/` と `lib.rs` の再輸出の監査の指摘を残らず直し、辞書の日付が戻っても変換し直され、衝突は 2 つのファイル名つきの読み込みエラーになり（`pasta_check` と同じ規則）、finalize 前の検索はキャッシュの状態によらず同じになり、`;`・`?` を含むパスははっきりした読み込みエラーになり、使われない抽象と古い構築の道が無くなるようにする。読めないセーブの退避（監査の 1 番）は `toml-serialization` へ移ったので範囲に入れない。

## Introduction

この spec は、2026-10-19 の監査が `pasta_lua` の読み込みまわり（`loader/`・`runtime/`・`presentation/`・`encoding/` と `lib.rs` の再輸出）で見つけた指摘のうち、2〜8 番と、ギャップ分析で同じ領域に増えた指摘を残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：作者の手元で起きる 4 つの困りごと（古い日付の辞書・同じモジュール名・finalize 前の検索・パスの `;`）。
- **挙動を変えない撤去**：使われない抽象・テストだけが使う構築の道・小物。

要件は「その領域の指摘が残らず直っている」ことと、挙動を変える直しの決めごとである。どう直すか（関数・ファイルの形）は設計で決める。

> brief.md だけでは決めきれなかった 9 つの論点は、2026-11-01 の要件ディスカッションで、すべて推奨案のとおりに確定した（research.md の「要件ディスカッションへの持ち越し」）。

## Boundary Context

- **In scope**
  - 監査の 2〜8 番（キャッシュの日付・同じモジュール名・finalize 前の検索・パスの `;`・使われない抽象・古い構築の道・小物）。
  - ギャップ分析で増えた、同じ領域の指摘（要件 1.6・1.7・2.7・4.2・5.5・5.6）。
  - 大文字と小文字だけ違うモジュール名の衝突（要件 2.7・2.8）。症状が監査の 3 番と同じなので、この spec で直す。`pasta_check validate` の衝突の比べ方も、ローダーと揃えて直す。
  - 挙動を変える直しと撤去に合わせた、マニュアルの頁と `.kiro/steering/structure.md` の直し。
- **Out of scope**
  - 読めないセーブの退避（監査の 1 番）。2026-10-24 に `toml-serialization` へ移り、PR #200 で入った。`runtime/persistence.rs`・`runtime/lifecycle.rs`・`pasta-persistence.md` には触らない。
  - `ponytail-lua-transpiler` が持つもの：`src/code_gen/`・`src/normalize.rs`・`src/config.rs`・`src/context.rs`（`TranspileContext`）・`src/transpiler.rs`。`loader/context.rs`（`LoaderContext`）は、この spec が持つ。
  - `src/debug/`（`ponytail-lua-debug`）、`pasta_scripts/`（`ponytail-lua-scripts`）、`search/context.rs`（`ponytail-core-registry`）、`sakura_script/`（`talk-grapheme-clusters` で完了。要件 5.3 のとおり、出力をバイト単位で保つ相手）、`pasta_shiori` の `enc.rs`（`ponytail-shiori`）。
  - `runtime/searcher.rs` の FFI の道（`lua-cmodule-robustness` で完了）。
  - `pasta_check` のファイルのパスからモジュール名を決める規則は変えない（ローダーが合わせる相手）。変えるのは、決めた名前どうしを比べるときに大文字と小文字を区別しないことだけである（要件 2.8）。
- **Adjacent expectations**
  - `pasta_shiori` は、ローダーの読み込みエラーを今の経路（ログと `X-ERROR-REASON: Load error: …`）でそのまま作者へ届ける。この spec は、その経路を変えない。
  - `ponytail-lua-transpiler` とは `lib.rs` の再輸出を共有する。行が別なので、後から入る側が合わせる。
  - `toml-serialization` が整えた `runtime/module_registry.rs` の形（`toml_to_lua` など）を保つ。
  - `scene-play-event` の申し送り：デバッグの「設定の有効」と「待ち受けの有無」の 2 つの条件を混ぜない。

## Requirements

### Requirement 1: 辞書の日付が戻っても変換し直す

**Objective:** As a ゴーストの作者, I want 辞書を古い日付のものに戻しても、次の起動で戻した内容のとおりに動いてほしい, so that バックアップからの復元・Explorer でのコピー・NAR の展開のあとに、古い変換結果が黙って動き続けない。

#### Acceptance Criteria

1. When 起動の時点で、ある `.pasta` の更新時刻が、その変換結果を作ったときの `.pasta` の更新時刻と異なる（新しい・古いを問わない）, the Pasta Loader shall その `.pasta` を変換し直し、新しい変換結果で動く。
2. When 起動の時点で、辞書のフォルダの `.lua` の更新時刻が、前回キャッシュへ写したときの更新時刻と異なる, the Pasta Loader shall その `.lua` を写し直す。
3. When `.pasta` または辞書の `.lua` の更新時刻が、前回の変換または写しのときと同じである, the Pasta Loader shall 変換または写しを省き、キャッシュの結果を使う。
4. If 辞書のファイルかキャッシュの更新時刻を取得できない, then the Pasta Loader shall そのファイルを変換し直す（今と同じ）。
5. When 作者がバックアップから前日の日付の `.pasta` を上書きで戻してゴーストを起動し直した, the Pasta Loader shall 戻した `.pasta` の内容でトークする。
6. If `.pasta` の変換結果をキャッシュへ書けなかった, then the Pasta Loader shall そのファイルの名前つきの読み込みエラーにし、前の変換結果で動き続けない（今は警告だけで続け、古いキャッシュがあればそれが動く。辞書の `.lua` の写しの失敗は、今も読み込みエラーである）。
7. If キャッシュの更新時刻を、変換元の辞書の更新時刻にそろえられなかった, then the Pasta Loader shall 警告をログに書いて読み込みを続ける（そろわなかったキャッシュは、次の起動で更新時刻が違うので必ず変換し直される）。

### Requirement 2: 同じモジュール名になる辞書は読み込みエラーにする

**Objective:** As a ゴーストの作者, I want 同じモジュール名になる 2 つの辞書を置いたら、ゴーストの読み込みがはっきり失敗してほしい, so that 片方の辞書の内容が黙って消えず、どのファイルを直せばよいかが分かる。

#### Acceptance Criteria

1. If 2 つ以上の `.pasta` が同じモジュール名になる（例：`dic/会話-朝.pasta` と `dic/会話_朝.pasta` は、どちらも `pasta.scene.会話_朝`）, then the Pasta Loader shall 辞書の変換と Lua の実行より前に読み込みエラーにし、エラーの理由にモジュール名と、関わるすべてのファイルの名前（ゴーストのフォルダから見た形）を含める。
2. If 辞書のフォルダの 2 つ以上の `.lua` が同じモジュール名になる, then the Pasta Loader shall 1 と同じく読み込みエラーにし、モジュール名と関わるすべてのファイルの名前をエラーの理由に含める。
3. The Pasta Loader shall ファイルのパスからモジュール名を決める規則を、`pasta_check validate` と同じにする。`pasta_check` が `module-pasta-collision`・`module-lua-collision` のエラーにする組み合わせと、ローダーが読み込みエラーにする組み合わせは一致する。
4. While `.pasta` と辞書の `.lua` が同じモジュール名（大文字と小文字の違いを除く）になる, the Pasta Loader shall 今と同じく `.pasta` を採り、その `.lua` を警告つきで読み込まない（`pasta_check` の `module-lua-shadowed` が警告であるのと揃う）。
5. When 衝突で読み込みエラーになった, the Pasta Loader shall そのエラーを、ほかの読み込みエラーと同じ経路（ログと、`pasta_shiori` が返す `X-ERROR-REASON`）で作者へ届ける。
6. The Pasta Loader shall 同じモジュール名にならない辞書の組み合わせ（日本語の名前・サブフォルダを含む）を、今と同じく読み込む。
7. If 2 つ以上の辞書のモジュール名が、大文字と小文字の違いを除いて同じになる（例：`dic/A-b.pasta` の `pasta.scene.A_b` と `dic/a_b.pasta` の `pasta.scene.a_b`。Windows ではキャッシュの 1 つのファイルを奪い合う）, then the Pasta Loader shall 1 と同じく読み込みエラーにし、モジュール名と関わるすべてのファイルの名前をエラーの理由に含める。この扱いは OS によらない。
8. The `pasta_check validate` shall 7 の組み合わせを、`module-pasta-collision`・`module-lua-collision` のエラーにする（3 の「ローダーと `pasta_check` は同じ組み合わせをエラーにする」を保つ）。

### Requirement 3: finalize 前の検索を、起動ごとに同じにする

**Objective:** As a ゴーストの作者, I want `main.lua` などで finalize の前にシーンを検索したときの結果が、起動のたびに同じであってほしい, so that 「編集した直後の起動でだけ見つかり、次の起動からは見つからない」という再現しにくい違いに悩まない。

#### Acceptance Criteria

1. While シーン辞書の finalize の前である（`main.lua` の実行中・シーン辞書の読み込み中の Lua ブロックのトップレベル）, the `@pasta_search` module shall キャッシュの状態や直前に編集した辞書によらず、同じ結果を返す。
2. While シーン辞書の finalize の前である, the `@pasta_search` module shall 正しい引数で行ったどのシーン・単語の検索にも「見つからない」を返す（エラーにはしない。今の「結果は使えない」というマニュアルの断りを、決まった結果に変える。空のシーン名などの引数の誤りは、finalize の後と同じくエラーのままとする）。
3. When `main.lua` に `SEARCH:search_scene("OnBoot")` と書いたゴーストを、辞書を編集した直後と、編集せずにもう一度起動した, the `@pasta_search` module shall どちらの起動でも同じ結果（見つからない）を返す。
4. When シーン辞書の finalize が終わった, the `@pasta_search` module shall 今と同じく、すべての辞書のシーンと単語を検索の対象にする。

### Requirement 4: 検索パスの区切りになる文字を含むフォルダを、はっきり断る

**Objective:** As a ゴーストの作者, I want ゴーストのフォルダのパスに Lua の検索パスを壊す文字が入っているとき、その理由が分かる読み込みエラーになってほしい, so that `require "pasta"` の module not found のような、本当の原因を隠したエラーに迷わない。

#### Acceptance Criteria

1. If ゴーストのフォルダ（`ghost/master`）の絶対パスに `;` か `?` が含まれる（例：`C:\SSP\ghost\a;b\ghost\master`）, then the Pasta Loader shall 辞書の変換と Lua の実行より前に読み込みエラーにし、エラーの理由に問題の文字とパスを含める。
2. If `pasta.toml` の `[loader]` の `lua_search_paths` の項目に `;` か `?` が含まれる, then the Pasta Loader shall 1 と同じく、問題の文字とその項目を理由に含めた読み込みエラーにする。
3. The Pasta Loader shall `;` と `?` を含まないパス（日本語・空白・長いパス・8.3 形式の短い名前を含む）を、今と同じく読み込む。
4. When 1 または 2 の読み込みエラーになった, the Pasta Loader shall そのエラーを、ほかの読み込みエラーと同じ経路（ログと `X-ERROR-REASON`）で作者へ届ける。

### Requirement 5: 使われない抽象を無くす

**Objective:** As a pasta の開発者・読み手, I want 本番から呼ばれない抽象と、実装が 1 つしかない差し替えの継ぎ目が無くなってほしい, so that 読み込みまわりのコードを読む手間が減り、使われない道の保守がいらなくなる。

#### Acceptance Criteria

1. The pasta_lua crate shall `presentation` モジュール（`PresentationMarker`・`RenderBoundary` など）を提供しない。
2. The pasta_lua crate shall `RendererInjection`・`SakuraRenderBoundary`・`default_sakura_renderer` を提供しない。
3. When ランタイムが `@pasta_sakura_script` を登録する, the Pasta Lua Runtime shall `talk_to_script` の出力を、撤去の前とバイト単位で同じにする。
4. The pasta_lua crate shall `Encoding::OEM` と、実装が 1 つだけの `Encoder` の抽象を提供しない。そのうえで、`@enc` の UTF-8 と ANSI の変換と、`ffi.load` の名前の扱いの結果は変えない。
5. The pasta_lua crate shall 本番で作られない読み込みエラーの種類（`LoaderError::Parse` とその作り方、`LoaderError::Transpile`）を提供しない（ギャップ分析で増えた指摘）。
6. The Pasta Loader shall 古い版のキャッシュの置き場所（`scene_dic.lua` の旧位置）を消す処理を持たない。版が変わるとキャッシュは丸ごと消えるので、この処理は働かない（ギャップ分析で増えた指摘）。

### Requirement 6: テストだけが使う古い構築の道を無くす

**Objective:** As a pasta の開発者, I want ランタイムを組み立てる道が本番の 1 本になってほしい, so that 本番と違う道を通るテストが、本番の不具合を見逃さない。

#### Acceptance Criteria

1. The pasta_lua crate shall 変換済みのコードを直接読む古い構築の道（`from_loader`）、`scene_dic` を後から読む道（`load_scene_dic`）、それらだけが使う `TranspileResult` を提供しない。
2. The Pasta Lua Runtime shall 本番の構築の道で、使わない引数（今の `_scene_dic_path`）を受け取らない。
3. When 古い道を使っていたテスト（`tests/runtime/encoding_test.rs`・`tests/runtime/runtime_api_test.rs`）を移した, the pasta_lua test suite shall 同じ観点（`@enc`・`package.path`・実行の API）を、`PastaLoader::load` と一時ゴーストを通して確かめる。
4. The Pasta Lua Runtime shall 本番の構築の道の起動の順（`main` → `pasta.shiori.entry` → `pasta.scene_dic`）と、起動モジュールの失敗を致命として文脈つきで返す扱いを、今と同じに保つ。

### Requirement 7: 孤立したキャッシュの警告を 1 回にする

**Objective:** As a ゴーストの作者, I want 元の辞書が無くなったキャッシュの警告が、1 つのファイルにつき 1 回だけ出てほしい, so that ログが同じ警告の重複で読みにくくならない。

#### Acceptance Criteria

1. When 元の辞書が無いキャッシュのファイルが見つかった, the Pasta Loader shall そのファイルごとに警告を 1 回だけログへ書く。
2. The Pasta Loader shall 1 の警告を、`[loader] debug_mode` の値によらず出す（今、常に出ている方を残し、`debug_mode` のときだけ重ねて出す方を無くす）。
3. The Pasta Loader shall 孤立したキャッシュのファイルを読み込まず、消しもしない（今と同じ）。

### Requirement 8: マニュアルと steering を、直した挙動に合わせる

**Objective:** As a ゴーストの作者・pasta の開発者, I want マニュアルと steering が、直したあとの挙動と構成のとおりに書かれていてほしい, so that 書いてあることと動きが食い違わない。

#### Acceptance Criteria

1. When 要件 1〜4 の挙動を変える直しが入る, the pasta manual shall 同じ変更で、その挙動を書いた頁を直す。少なくとも次の頁である。
   - `internals/transpiler.md`（キャッシュの判定）
   - `internals/loader.md`（同じモジュール名の衝突・`package.path`）
   - `reference/startup.md`（`;`・`?` を含むパス）
   - `lua/modules/pasta-search.md`・`lua/modules/index.md`・`internals/internal-modules.md`（finalize 前の検索）
   - `debug/validate.md`（`pasta_check` の衝突が、大文字と小文字の違いを除いて比べること。ローダーも同じ組み合わせを読み込みエラーにすること）
2. When 要件 5〜6 の撤去が入る, the pasta manual shall 撤去したものの記述を消すか、今の構成に書き直す。少なくとも次の頁である。
   - `internals/shiori.md`（presentation とレンダラの注入の節・ファイルの表・継ぎ目の表・ソースの一覧）
   - `internals/talk-output.md`（その節への参照）
   - `internals/execution-model.md`（`factory.rs`・`renderer_injection.rs` の行）
   - `internals/logging-encoding.md`（`Encoder`・`Encoding::OEM`）
   - `index.md`（章の見張るパス）
3. When 要件 5〜6 の撤去が入る, the `.kiro/steering/structure.md` shall `presentation/` と `renderer_injection.rs` の行を消し、今の構成を書く。
4. When `lua/modules/` などスキルの参照の元になる頁を直した, the pasta manual shall `node book/tools/gen-skill-refs.mjs` を走らせ、生成物を同じ変更に入れる（`--check` で差分が無い）。
5. If 実装の時点で、README と `tech.md` に「公開クレートの pub API を予告なく消すことがある」断り書きがまだ無い, then the change shall その断り書きを 1 回だけ書く（ponytail の spec のうち、先に main へ入った 1 本が書く）。

### Requirement 9: 直していない挙動を保ち、品質の関門を通す

**Objective:** As a pasta の開発者, I want 直しのほかの読み込みの挙動が変わらず、決まった関門を通ってほしい, so that 整理のついでに別の不具合が入らない。

#### Acceptance Criteria

1. The Pasta Loader shall 要件 1〜7 のほかの起動の挙動を変えない。対象は、辞書の発見と読み込みの順、`init.pasta`・`init.lua` を断る扱い、版と別名表の指紋の変化でキャッシュを丸ごと消す扱い、読み込みエラーの経路である。
2. When 挙動を変える直し（要件 1〜4）を入れる, the change shall 直す前の誤りを再現するテストを先に置き、直したあとにそのテストが通る。
3. The change shall `cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lua -p pasta_shiori -p pasta_check` を通る。
4. If 新しいテストが環境変数 `PASTA_DEBUG` を読む道を通る, then the test shall その値を中和する守りを持ち、開発者の環境によって結果が変わらない。
