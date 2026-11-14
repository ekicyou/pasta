# ギャップ分析: date-var-calendar

- 実施日: 2026-11-14
- 対象: `requirements.md`（requirements-generated・未承認）と、ブランチ `claude/kiro-start-date-var-calendar-6b9bd0`（base `f40e01f5`）の今のコード
- 方法: Grep／Glob／Read によるコードの調べ。外部の調べは data.go.jp のページの取得を 1 回試みたが、証明書の期限切れで取れなかった（「要調査」に回す）

## 1. 分析の要約

- **転記の芯はそのまま使える**: `＄節句` は `act.lua` の `transfer_date_to_var` の中の 1 行（`SEKKU_JA[month*100+day] or "なし"`）で、日付は `CLOCK.date(self.req)`（仮想時刻・`X-Pasta-Time` 込み）から取っている。`＄祝日`・`＄行事` はここに 2 行足せば、イベントのシーンを選ぶ前の自動転記（`event/init.lua` の `create_act`・`virtual_dispatcher.lua`）にも何もせずに乗る（Req 1.3・1.4・3.6・3.7・7.2）。
- **祝日の規則は新しく書く純粋な関数**: 今のコードに祝日の計算は無い。年・月・日（と曜日）から名前を返す関数を 1 つ足す。置き場所は Lua（`pasta_scripts`）か Rust（`pasta_core`）の 2 案。曜日は今日の `date.wday` しか無いので、前後の日の曜日は自分で数える（Sakamoto の式などの算術。`os.time` は Windows で 1970 年より前に失敗する）。
- **`calendar.csv` は Rust で読み、規則を `pasta_core` に置くのが今の型に最も近い**: `[scene.alias]`（`SceneAliasTable`）と `[actor].font`（`ActorFont`）は、規則を `pasta_core` に置き、pasta.dll（`pasta_lua` のローダ）と `pasta_check`（ファイルシステム非依存・WASM でも動く核）の両方が同じ規則を呼んでいる。`pasta_check` は `pasta_lua` に依存しない（dev-dependency だけ）ので、表を Lua で読むと Req 6.4（判定の食い違いなし）は二重実装＋一致テストでしか守れない。
- **pasta_check 側は殻をほぼ触らずに済む**: CLI（`ghost_files::collect`）・VSCode（`ghostValidator.ts`）・pasta-recipe（`check.rs`）の 3 つの殻は、どれも核の `select_dictionary_files` が返したパスだけを読み、`FileText::{Text, NotUtf8, Unreadable}` で渡す。核がここに `calendar.csv` を加えれば、3 つの殻とも中身を読んで渡す（Req 5.6・5.7 の区別もこの型にもうある）。
- **要調査**: 内閣府の CSV のライセンス表示の正確な文言（data.go.jp が取れなかった）、春分・秋分の式の 2028 年以降の正しさ、テスト用データを crates.io のパッケージに入れるか、`.claude/skills/*/references/` の再生成が分類器に止められないか。

## 2. 今のコードの調べ

### 2.1 日時変数の転記（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`）

- `SHIORI_ACT_IMPL.transfer_date_to_var`（155〜195 行）: `self.req.date` が無ければ何もせずに返す → `CLOCK.date(self.req)` → 英語の数値の変数・日本語の変数・`曜日`・`時１２`・`季節`・`節句`。
- `＄節句` の作り（120〜121・186〜192 行）: モジュールの local の表 `SEKKU_JA = { [107]="人日", ... }` を `date.month * 100 + date.day` で引き、無ければ `"なし"`。`＄祝日`・`＄行事` も同じく「当たらなければ `なし`」の形に揃えられる。
- `CLOCK.date` が返す表（`clock.lua` の `PastaDate`）: `year`・`month`・`day`・`hour`・`min`・`sec`・`yday`・`wday`（日曜 0）など。祝日の計算に要るのは年・月・日と曜日。
- 呼ばれる場所: `event/init.lua` 106 行（`create_act`。全イベントのシーンの選ぶ前）、`virtual_dispatcher.lua` 182〜183 行（時報・ランダムトーク）、`＞transfer_date_to_var`（作者の Call）。**OnSecondChange のたびにも走る**ので、祝日の計算は日付ごとに覚えておく（メモ化）か、軽い計算にしておく（要設計）。

### 2.2 読み込みのエラーと `[scene.alias]` の作り

- 規則: `pasta_core::registry::scene_alias`（`SceneAliasTable::with_written`）・誤りの型 `pasta_core::error::SceneAliasError`（英語のメッセージ、`thiserror`）。
- ランタイム側: `pasta_lua/src/loader/config/mod.rs` の `parse_scene_aliases` が `toml::Spanned` の位置から行番号を数え（`content[..pos].matches('\n').count() + 1`）、`"{e}; at [scene.alias] {target} = \"…\" (line N)"` の形で `toml::de::Error` に包む → `LoaderError::Config(path, e)`（`"Failed to parse configuration file: '{path}': {e}"`）。
- 伝わり方: `PastaLoader::load_with_config` の `Err` → `pasta_shiori/src/shiori.rs` の `load` が `last_load_error` に覚え、`load` は `false`、その後の要求は 500 で理由を返す（`X-ERROR-REASON` は `error.rs` の `single_line` で 1 行化）。ログにも `PastaShiori load failed` が出る。バルーンには出ない（`book/src/reference/runtime-failures.md` 15 行）。
- 同じ型の前例 2 つ目: `[actor."名前"].font` は規則を `pasta_core/src/actor_font.rs`、行番号付けを `pasta_lua/src/loader/config/actor_font.rs`、検査を `pasta_check/src/validate/config.rs`（`FindingKind::ConfigActorFont`）に分けている。
- **`calendar.csv` 用には `LoaderError` に新しい種類が要る**（今の `Config` は `toml::de::Error` を持つので、CSV の誤りを入れるには `serde::de::Error::custom` で包むか、新しい種類 `Calendar { path, line, reason }` などを足す）。ファイル名と行番号を Display に入れれば、既存の経路でそのまま見える。

### 2.3 ゴーストのファイルを読み込みのときに読む場所

- Rust: `PastaLoader::load_with_config`（`pasta_lua/src/loader/mod.rs`）の Phase 1 で `pasta.toml` を `fs::read_to_string` で読む。ゴーストのファイルを読み込みのときに読むのは今は Rust だけ（`pasta.toml`・辞書・キャッシュ・セーブデータ `@pasta_persistence`）。
- Lua: `pasta_scripts` の中に `io.open` は 1 つも無い（`scriptlibs/luacheck` だけ）。`SHIORI.load(hinst, load_dir)`（`entry.lua`）は `true` を返すだけで、ここでファイルを読む前例は無い。Windows の Lua の `io.open` は ANSI のパスを取るので、日本語のパスには `@enc` の変換が要る（`pasta_lua/src/encoding`）。
- Lua へ値を渡す口: `factory.rs` の `from_loader_with_scene_dic` が `@pasta_config`・`@enc`・`@pasta_persistence`・`@pasta_log`・`@pasta_sakura_script`・`@pasta_debug` を `package.loaded` に登録してから `main` → `pasta.shiori.entry` → `pasta.scene_dic` を `require` する。`@pasta_config` は `custom_fields`（pasta.toml の `[loader]` 以外）の表で、`config.lua` は `pcall(require, "@pasta_config")` で無いときにも動く。
- **Lua のテストの実行器**（`tests/lua_unittest_runner.rs`）は `@pasta_persistence`・`@pasta_search`・`@pasta_sakura_script`・`@pasta_log` だけを登録する。`act.lua` が新しい Rust のモジュール（例 `@pasta_calendar`）を `require` すると、`act.lua` を読む多数のテストが落ちる。`config.lua` と同じ `pcall(require, …)`＋無ければ空の表、か、実行器での登録が要る。

### 2.4 `pasta_check validate` と共有の可能性

- 核（`pasta_check/src/validate/`）は `std::fs` に触れず、`GhostInput { pasta_toml: TomlSource, entries, dirs }` とファイルの中身の表 `BTreeMap<String, FileText>` を受け取る。依存は `pasta_dsl`・`pasta_core`・`toml`・`serde`・`glob`。
- `FileText` は `Text { text }`（BOM は残したまま）・`NotUtf8`・`Unreadable { reason }` の 3 つで、Req 5.6・5.7 の区別とそのまま対応する。ファイルが無いことは `entries` に無いことで分かる（Req 6.3）。
- 殻の 3 つ（CLI `ghost_files.rs`／VSCode `editors/vscode/src/ghostValidator.ts`／pasta-recipe `editors/recipe/src-tauri/src/check.rs`）は、いずれも `select_dictionary_files`（WASM では `wasm_select_ghost_files`）が返したパスだけを読む。核がここに `calendar.csv` を返せば殻は変えずに読む。ただし:
  - 関数名と `Stats.dictionary_files` の数え方が「辞書ファイル」前提。`calendar.csv` を同じ口で返すなら、数に入れない工夫か、別の関数（例 `select_ghost_files`）が要る。
  - `report::skip_reasons` は `FileNotUtf8`・`FileUnreadable` を「読めない辞書ファイル」として**飛び先と条件の検査を丸ごと省く**理由にする。`calendar.csv` の文字コードの誤りで Call の検査まで省かれないよう、別の `FindingKind`（例 `calendar-invalid`・`calendar-not-utf8`）にするか、パスで分ける必要がある（前例: `ConfigActorFont` は省く理由に入れていない）。
  - VSCode の検査のきっかけは `.pasta` と master の直下の `pasta.toml` の保存、配下の `.pasta`・`.lua` の作成・削除（`ghostValidator.ts` 83〜129 行）。`calendar.csv` の保存では検査し直さない（Req 6 は `pasta_check validate` だけを言うので、エディタは範囲外にするか要相談）。
- 新しい `FindingKind` を足すと、マニュアル `book/src/debug/validate.md` の表（116〜117 行に `config-scene-alias`・`config-actor-font`）と、省く理由の説明（77 行）にも行が要る。
- 一致テスト: `pasta_check/tests/validate_parity_test.rs` の `load_failure_matches_errors` が「pasta.dll が読み込みを失敗する ⇔ 検査が誤りを出す」を比べている。`calendar.csv` の事例をここに足せば Req 6.4 を確かめられる。
- `pasta_check release` は検査を関門にしていて（`release.rs`）、`ghost/master` を `profile/` 以外まるごと写すので `calendar.csv` は `.nar` に入る。検査が誤りを出せば release も止まる（Req 6.2 は追加の作業なしで満たせる）。

### 2.5 ファイル名の大文字・小文字

- pasta.dll は `base_dir.join("calendar.csv")` のように OS に任せると、Windows では `Calendar.CSV` も読む。pasta_check の核は `entries` の文字列で探すので、そのままでは完全一致になり、Windows で食い違う（Req 6.4 に関わる）。VSCode の殻は `pasta.toml` を小文字にして比べている前例がある。**どちらに揃えるか要設計**（例: 核も大文字・小文字を区別せずに探し、両方に `Calendar.csv` の事例のテストを置く）。

### 2.6 Lua のテストとフィクスチャ

- Lua のテスト: `crates/pasta_lua/tests/lua_specs/*_test.lua`（`init.lua` の `specs` に登録）。`＄節句` は `shiori_act_test.lua` 710〜733 行（五節句の日と前後の日）・797〜812 行（仮想時刻のずれで決まる）。`lua_unittest_runner.rs` は `Lua::new()`（`io` あり）で、package.path に `lua_specs` を足している。テストからファイルを読む前例は無い（フィクスチャのパスを Lua に渡す口が要る）。
- Rust の結合テスト: `pasta_lua/tests/*_test.rs`（例 `scene_condition_e2e_test.rs`）、`pasta_lua/tests/fixtures/`（`loader/` など。CSV は今リポジトリに 1 つも無い）。
- SHIORI の結合テスト: `pasta_shiori/tests/virtual_clock_e2e_test.rs` と `fixtures/virtual_clock/dic/clock.pasta`（`＊振り分け` が `＞「季節_」＆＄季節`・`＞「節句_」＆＄節句`）。Req 8.6 の「仮想時刻・`X-Pasta-Time` でずらす」はここに祝日・行事の振り分けを足すのが近い（`calendar.csv` もこのフィクスチャに置ける）。
- 読み込みのエラーのテストの前例: `pasta_lua/tests/loader/`（`scene_alias_cache_test.rs`・`config_test.rs`・`startup_test.rs` の actor_font の事例）、`pasta_shiori/tests/scene_alias_ontalk_e2e_test.rs`。

### 2.7 答え合わせのデータ（内閣府の CSV）と出典の表示

- リポジトリの今の扱い:
  - ルート `LICENSE`（MIT）。NOTICE・THIRD-PARTY の類のファイルは無い。
  - 取り込んだ素材の隣に出典の文書を置く形が 3 つある: `book/src/img/claudia/LICENSE.txt`（出典リポジトリ・取り込んだコミットと日付・ライセンス・縮小の手順）、`book/tools/highlight/grammars/LICENSE.lua-grammar.md`（SPDX・著作権者・取得元 URL・取得日・改変の方針・「公開成果物に含めない」）、`crates/pasta_sample_ghost/art/README.md`（生成元・手順・配布物に入らないこと）。**「データの隣に、出典・取得日・ライセンス・手を加えたこと・配布物に入るか」を書いた文書を置く**のが今の型。
  - 配布するバイナリの第三者ライセンスは `cargo-about`（`about.toml`・`about.hbs`）が作る。テスト用のデータはこれに乗らない。
- 気をつけること:
  - `pasta_lua`・`pasta_core`・`pasta_shiori` は `publish = true` で、`pasta_lua` と `pasta_core` は `include`／`exclude` を持たない。`crates/pasta_lua/tests/` に置いたデータは **crates.io のパッケージにそのまま入る**（Req 8.2 の「pasta の配布物には含めない」との関係を決める必要がある。`exclude` を足すか、パッケージに入ることを許して出典の文書も一緒に入れるか）。
  - CI は Windows に加えて Ubuntu・macOS でも `cargo test --workspace` を回す（`build.yml` の `build-unix`）。`pasta_lua` の `encoding` は Windows でだけ ANSI を変換し、Unix ではそのまま通す。**Shift_JIS のまま置くと、Unix のテストで名前を読み解く手段が無い**（Shift_JIS を読むクレートは依存に無い）。元のまま置くなら、テストで名前を比べず日付だけ比べるか、依存を足すことになる。
  - `.gitattributes` が無く、作業ツリーは CRLF（`autocrlf`）。Shift_JIS のファイルを git の改行の変換にかけると、照合用の元のファイルのバイトがずれる恐れがある（UTF-8 にして LF／CRLF どちらでも読めるようにするか、`.gitattributes` で `-text` を付けるか）。
- **要調査**: data.go.jp のページ（https://www.data.go.jp/data/dataset/cao_20190522_0002）の取得は証明書の期限切れで失敗した。ライセンスの欄が CC-BY 4.0 か、公共データ利用規約（PDL1.0。CC-BY 4.0 と互換）か、出典の書き方の例（「出典: 内閣府ホームページ（URL）」・「編集・加工した」旨の表示）を、設計のときに確かめる。

### 2.8 マニュアルとスキルの資料

- `book/src/grammar/variables.md`: 「エンジンが値を入れる変数」（340 行〜）の注意文（350 行。`＄季節`・`＄節句` の名前の衝突）、日時変数の表（376〜387 行。`＄節句` の行が 383 行）、振り分けの説明と例（391〜412 行。`＊節句_なし`）。`＄祝日`・`＄行事` はここに同じ形で足せる（Req 9.1・9.2・9.5）。
- `calendar.csv` の書き方の節（Req 9.3・9.6）の置き場所は未定。新しい章にすると、`book/tools/gen-skill-refs.mjs` の `GENERATION_MAP`（24 エントリ・順序固定）と `gen-skill-refs-test.mjs` の「24 エントリ」「24 章」の確かめ（105・265 行）、`book/src/SUMMARY.md` にも手が入る。`variables.md` の中の節にすれば、生成の対応表は変わらない。
- 読み込みのエラーの説明: `book/src/internals/loader.md` の「起動の段階構成」（61 行〜。段階表）に、`calendar.csv` を読む段を足すことになる。`book/src/reference/startup.md` の「ゴーストが起動しない・喋らないとき」の案内にも 1 行要るかもしれない。`pasta_check` の誤りのコードは `book/src/debug/validate.md`。
- スキルの資料: `gen-skill-refs.mjs` は `.claude/skills/<skill>/references/` に書き出す（184・275 行）。`variables.md` は `pasta-ghost-authoring` へ、`debug/validate.md` は対応表に無い（要確認）。**`.claude/skills` の書き換えは分類器に止められることがある**（開発の記録）ので、生成のコミットの手順を事前に確かめておく。
- `pasta-ghost-authoring` の SKILL.md の description には「時報変数, 日時変数, date variables」がある。`＄祝日`・`＄行事` のキーワードを足すかは任意（SKILL.md の編集は上と同じ止められる恐れ）。

## 3. 要件と今の資産の対応

| 要件 | 今の資産 | ギャップ |
| ---- | -------- | -------- |
| Req 1（`＄祝日` を入れる） | `transfer_date_to_var`・`CLOCK.date`・`なし` の作り（`＄節句`） | **Missing**: 祝日の関数。転記は 1 行で足りる |
| Req 2.1・2.2（固定の日・ハッピーマンデー） | 無し | **Missing**: 規則の表。第 n 月曜は「日が 7n−6〜7n で月曜」で決まる |
| Req 2.3（春分・秋分） | 無し | **Missing**・**Unknown**: 式（よく使われる 1980〜2099 年向けの近似式）と範囲外の年の扱い（未決 4） |
| Req 2.4・2.5（振替休日・国民の休日） | 無し | **Missing**: 前後の日の判定。月をまたぐ（例 5/6、9 月の連休）ので日の足し引きと曜日の計算が要る |
| Req 2.6・2.8（名前・2022〜CSV の最後の年で一致） | 無し | **Constraint**: 答え合わせのデータの形（未決 9）と Unix の CI |
| Req 3（`＄行事`） | 転記の口・`なし` の作り | **Missing**: 表の照合（書いた順の先勝ち・3 つの書き方・年またぎ・2/29） |
| Req 4（表の書き方） | `pasta.toml` を Rust で読む前例・`FileText` の BOM の扱い | **Missing**: 1 行ずつの読み取り・BOM・CRLF・注釈・空白の取り除き |
| Req 5（検査とエラー） | `[scene.alias]`・`[actor].font` の読み込みエラーの型と伝わり方 | **Missing**: 誤りの型と `LoaderError` の種類。**Constraint**: 無い／読めない（権限）の区別（`fs::read` の `NotFound` と他を分ける。`ghost_files.rs` の `pasta.toml` に前例） |
| Req 5.9・5.10（読み込みのときに 1 回・読み込み直しで反映） | ローダは `load` ごとに VM を作り直す（`shiori.rs` の `release_runtime`） | 読み込みのときに読めば自然に満たせる |
| Req 6（`pasta_check validate`） | 核の入力の型・殻の 3 つ・一致テスト・release の関門 | **Missing**: 核の検査と `FindingKind`。**Constraint**: 省く理由に入れない・`dictionary_files` の数・大文字小文字 |
| Req 7（既存を変えない） | 既存のテスト（`shiori_act_test.lua` ほか） | 既存の変数に触れない足し算なら低リスク。`act.lua` が新しいモジュールを要るなら Lua のテストの実行器に影響 |
| Req 8（テスト） | Lua／Rust／SHIORI の 3 層のテストの置き場 | **Missing**: 答え合わせのデータ・出典の文書・データをテストへ渡す口 |
| Req 9（マニュアル・スキル） | `variables.md` の表と例・`gen-skill-refs.mjs` | **Missing**: 表の書き方の節（置き場所は未定）・`validate.md`・`internals/loader.md` |

## 4. 実装の選択肢

### 案 A: すべて Lua に足す（`act.lua` を広げる）

- 祝日の関数と `＄行事` の照合を `act.lua`（または新しい `pasta/shiori/calendar.lua`）に書き、`calendar.csv` は `SHIORI.load(hinst, load_dir)` か最初の転記で Lua の `io.open` で読む。
- ✅ Lua だけで閉じ、`＄節句` と同じ場所に並ぶ。Lua のテストで祝日の規則を全部の日について回せる。
- ❌ 読み込みのエラーを「ファイル名と行番号付きで読み込みを失敗させる」には、`SHIORI.load` の `false`（理由は `last_load_error` に乗らない）か `main` 前後の `require` の失敗に頼ることになり、`[scene.alias]` と同じ見え方（Req 5.8）に揃えにくい。
- ❌ Windows の日本語のパスで `io.open` に `@enc` の変換が要る。UTF-8 の検査も Lua で自前に書く。
- ❌ `pasta_check` は Lua を持たないので、検査（Req 6）は Rust で二重に書き、一致テストで食い違いを見張ることになる。

### 案 B: 表の読み取りを `pasta_core` に置き、pasta.dll と pasta_check で共有する（Rust 中心）

- `pasta_core` に `calendar` の規則（1 行の読み取り・日付の 3 つの書き方の検査・照合）を置く（`SceneAliasTable`・`ActorFont` と同じ置き方）。
- pasta.dll: ローダの段（Phase 1 の後。例「1.6」）で `ghost/master/calendar.csv` を `fs::read` → `String::from_utf8`（失敗は UTF-8 で保存し直す旨のエラー）→ `pasta_core` で検査 → 誤りは `LoaderError` の新しい種類（ファイル名・行番号・原因）。表は `@pasta_calendar`（仮）などで Lua に渡し、`act.lua` が日付で引く（引くのは Rust の関数でも Lua でもよい）。
- pasta_check: 核が `calendar.csv` を読むべきパスとして返し、`pasta_core` の同じ規則で検査、新しい `FindingKind` で報告する。殻の 3 つは変えない（VSCode の保存のきっかけだけは別）。
- 祝日の規則は Lua（`＄節句` の隣）でも Rust（`pasta_core`）でもよい。祝日は `pasta_check` に要らないので、共有の理由は無い。
- ✅ Req 5.8（`[scene.alias]` と同じ扱い）と Req 6.4（判定の一致）を、同じコードを呼ぶことで構造的に満たせる。今の 2 つの前例と同じ層の分け方。
- ✅ UTF-8・権限・無いの区別を Rust の `io::ErrorKind` と `String::from_utf8` で素直に書ける（`ghost_files.rs` の `pasta.toml` に同じ形がある）。
- ❌ 触るクレートが 3 つ（`pasta_core`・`pasta_lua`・`pasta_check`）に広がる。Lua へ表を渡す口を新しく作る（Lua のテストの実行器も合わせる）。

### 案 C: 混ぜる（表は案 B、祝日は Lua）

- `calendar.csv` は案 B のとおり Rust（`pasta_core`＋ローダ＋核）。`＄祝日` の規則は Lua の純粋な関数（例 `pasta/shiori/holiday.lua`。`clock.lua` と同じく循環参照を避けて `act.lua` から `require`）。
- 祝日の答え合わせは、Lua のテストでデータを読むか、Rust の結合テストで Lua の関数を全部の日について呼び、Rust 側でデータを読んで比べる。
- ✅ 祝日は `＄節句` と同じ Lua の層にあり、読みやすい。検査の共有が要る所だけ Rust。
- ❌ 規則が 2 つの言語に分かれる（ただし互いに独立）。

### 比べ

| | 案 A | 案 B | 案 C |
| - | - | - | - |
| Req 5.8 の見え方を `[scene.alias]` に揃える | 難しい | そのまま | そのまま |
| Req 6.4（判定の一致） | 二重実装＋一致テスト | 同じコード | 同じコード |
| 新しく作る口 | Lua の読み取り・`@enc` | Lua へ表を渡す口 | Lua へ表を渡す口 |
| 祝日のテスト | Lua で全部の日 | Rust で全部の日 | Lua か Rust の結合テスト |

## 5. 手間とリスク

- **手間: M（3〜7 日）**。祝日の規則・表の読み取りと検査・3 層のテスト・pasta_check の検査・マニュアル 3〜4 か所と、個々は小さいが触る所が多い。既存の型（`[scene.alias]`・`[actor].font`・`＄節句`）に沿えるので L にはならない。
- **リスク: 中**。規則と照合は純粋な関数で、テストの答え合わせもある。リスクは周りに集まる: 答え合わせのデータのライセンスと置き方（crates.io のパッケージ・Unix の CI・文字コード）、春分・秋分の式の将来の正しさ、pasta_check と pasta.dll の大文字小文字・省く理由の食い違い、`.claude/skills` の再生成の手順。

## 6. 設計フェーズへの申し送り

### 推す向き（決定ではない）

- 層の分け方は案 B か案 C（表の規則は `pasta_core` に置いて pasta.dll と pasta_check で共有）。今の 2 つの前例（`SceneAliasTable`・`ActorFont`）と同じで、Req 5.8・6.4 を構造で満たせる。
- 転記は `act.lua` の `＄節句` の隣に 2 行。当たらない日は `なし`。

### 要調査

1. data.go.jp の `cao_20190522_0002` のライセンスの欄と、出典の表示の書き方（CC-BY 4.0 か PDL1.0 か、「加工した」旨の書き方）。証明書の期限切れで今回は取れなかった。
2. 春分・秋分の式（例 `floor(20.8431 + 0.242194*(y-1980) - floor((y-1980)/4))`・秋分は 23.2488）の有効な年の範囲と、国立天文台の暦要項（毎年 2 月に翌年分）との突き合わせ。
3. テスト用データを `pasta_lua`（または置く crate）の crates.io のパッケージに入れるか（`exclude` を足すか、出典の文書ごと入れるか）。
4. `gen-skill-refs.mjs` の書き出し（`.claude/skills/pasta-ghost-authoring/references/variables.md`）が、このリポジトリの手順で止められずにコミットできるか。
5. OnSecondChange ごとの転記で祝日の計算を毎回するか、日付ごとに覚えるか（計算の重さは小さいが、毎秒走る）。

## 7. 要件ディスカッションに出す論点（ギャップ分析から）

requirements.md の「未決の論点」1〜9 に加え、コードを調べて見えたものを挙げる。

- **論点 9 に足す事実**: CI は Ubuntu・macOS でもテストを回し、Shift_JIS を読む手段が依存に無い。元のまま（Shift_JIS）コミットすると、Unix では名前を比べられない（日付だけ比べるか、依存を足すか、UTF-8 に変えてコミットするか）。`.gitattributes` が無いので、元のままなら改行の変換を止める設定も要る。
- **論点 6 に足す事実**: pasta_check の殻の 3 つ（CLI・VSCode・pasta-recipe）は核が選んだファイルを読むので、核が `calendar.csv` を選べば VSCode と pasta-recipe の診断にも自然に出る。ただし VSCode は `calendar.csv` の保存では検査し直さない。エディタでの即時の検査まで範囲に入れるか。
- **新しい論点 A（テスト用データと配布物）**: crates.io のパッケージを Req 8.2 の「pasta の配布物」に数えるか。数えるなら `exclude` を足す、数えないなら出典の文書も同じ場所に置いてパッケージに入れる。
- **新しい論点 B（ファイル名の大文字・小文字）**: Windows の pasta.dll は `Calendar.CSV` も読むが、pasta_check の核は一覧の文字列で探す。大文字・小文字を区別せずに探すことに揃えるか、`calendar.csv` の完全一致だけにするか（完全一致なら pasta.dll 側も一覧を見て探す必要がある）。
- **新しい論点 C（pasta_check の誤りが他の検査を止めるか）**: `calendar.csv` が UTF-8 でない・行が不正なとき、今の「読めない辞書ファイル」と同じに Call・条件の検査まで省くか、省かずに続けるか（`config-actor-font` は省かない前例）。
- **新しい論点 D（マニュアルの置き場所）**: `calendar.csv` の書き方を `variables.md` の中の節にするか、新しい章（`reference/calendar-csv.md` など）にするか。新しい章は `GENERATION_MAP`（24 エントリ）とその自己テスト・`SUMMARY.md` に手が入る。
- **新しい論点 E（`YYYYMMDD` の年の範囲）**: `00000101` や 1970 年より前の年を受け付けるか（暦の検査だけなら 0001〜9999 年を受け付けられる。Req 5.2 は月日の検査だけを言う）。

## 8. 次の段

- 要件ディスカッション（`/kiro-requirements-discussion date-var-calendar`）で、未決の論点 1〜9 と上の論点 A〜E を決める。
- 要件が決まったら `/kiro-design date-var-calendar` で設計へ進む。
