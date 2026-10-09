# ギャップ分析: hello-pasta-tutorial-stages

作成: 2026-10-08（`/kiro-validate-gap`）。要件 `requirements.md` と現行コードベースの差を洗い、設計フェーズへ選択肢を申し送る。決定は設計で行う。

## 1. 現状調査

### 1.1 対象資産

| 資産 | 場所 | 現状 |
| ---- | ---- | ---- |
| hello-pasta 配布辞書（SSOT） | `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/{actors,boot,talk,click,choice}.pasta` | 5 ファイル・187 行。コメントはテスト向け。`OnBoot` 単一シーン、`OnTalk` 7 個（うち 1 個が `＄％currentghost.name`）、`OnMouseDoubleClick` 7 個 + choice.pasta に 1 個、`時報12`・`時報その他`×3、`OnClose`×2 |
| 設定・配布ファイル | 同 `master/pasta.toml`・`descript.txt`・`install.txt`・`scripts/README.md`、`shell/master/*` | `pasta_patterns = ["dic/*.pasta"]`、`[actor."女の子"] spot=0` / `[actor."男の子"] spot=1`、talk 間隔 180–300 秒。`scripts/` は README のみ |
| 実ローダー検証 | `crates/pasta_sample_ghost/tests/self_deploy_integration_test.rs` | `fixture_master_dir()` を tempdir へコピー（`profile/` を除外）→ `PastaLoader::load(base)`。`#[ctor]` で `PASTA_DEBUG` を中和。読み込み成功と自己展開のみ検証、イベントは送らない |
| 構造テスト（文字列照合） | `crates/pasta_sample_ghost/tests/integration_test.rs` `test_pasta_scripts`・`test_random_talk_patterns`・`test_hour_chime_patterns`、`src/scripts.rs` のユニットテスト | `＊OnBoot`/`＊OnFirstBoot`/`＊OnClose` の存在、`OnTalk` 5〜10 個、`OnMouseDoubleClick` 7 個以上、`＊時報12`・`＊時報その他`・`＄時１２` の存在、イベント辞書に `％女の子`/`％男の子` のグローバル定義が無いこと |
| 必須ファイル | `tests/dist_src_validation_test.rs` | `dic/{actors,boot,talk,click}.pasta` など 8 ファイルの存在（`choice.pasta` は未掲載） |
| SHIORI ゴールデン | `crates/pasta_shiori/tests/byte_invariant_test.rs`・`kick_unused_byte_invariant_test.rs` | `OnBoot` の完全応答バイト列を固定: `\p[0]\s[1]起動したよ～。\_w[950]\p[1]\s[11]さあ、\_w[450]始めようか。\_w[950]\e`。`shiori_sample_ghost_test.rs` も `起動したよ` を含むことを assert |
| SHIORI e2e | `crates/pasta_shiori/tests/scene_kick_{,gate_,multibeat_,preempt_}e2e_test.rs`、`tests/common/mod.rs` `copy_sample_ghost_to_temp()` | hello-pasta を tempdir へコピーし、独自のシーンを `dic/` に追記、`pasta.toml` の talk 間隔を上書き。辞書の中身には依存しない（シーン名の衝突だけ注意） |
| マニュアル CI | `.github/workflows/manual.yml`（`tutorial-check` → `cargo test -p pasta_sample_ghost`）、`book/tools/tutorial-check.mjs`（+ `tutorial-check-test.mjs`）、`book/tools/verify-content.mjs` | `first-ghost.md` の ```` ```pasta ```` ブロック群と各 `dic/*.pasta` の**逐語一致**（改行・末尾空白を正規化）。1 つでも不一致なら exit 1 で公開中断。`verify-content.mjs` も tutorial-check の成立を受入基準に含む |
| イベント送信ハーネス | `crates/pasta_shiori/tests/common/test_env.rs` `TestEnv::new(fixture)` / `request(text)` | SHIORI/3.0 の生リクエスト文字列を送って `ShioriResponse` を得る。fixture 名ベース（pasta_shiori の `tests/fixtures/`） |
| フィクスチャ慣習 | `crates/pasta_lua/tests/fixtures/*.pasta`、`crates/pasta_lua/tests/fixtures/{e2e,loader}/`、`crates/pasta_shiori/tests/fixtures/` | 単体 `.pasta` と、ゴースト構成のディレクトリ両方の慣習がある。「複数ディレクトリをループで読み込む」テストは見当たらない（Research Needed: `pasta_lua/tests/fixtures/e2e` の走査方法） |
| 文法の権威 | `book/src/grammar/*.md`・`book/src/lua/shiori-events.md`・`variables.md`・`call-jump.md`・`reference/pasta-toml.md` | 段階表に要る表現はすべて記載あり（§2.1） |

### 1.2 慣習

- 辞書は `dic/*.pasta` に役割別に分け、アクター辞書は `actors.pasta` に集約（イベント辞書にグローバル定義を置かない、とテストが固定）。
- テストはコミット済みフィクスチャを**その場でロードせず** tempdir へコピーする（自己展開の汚染ガード）。`PASTA_DEBUG` 中和の `#[ctor]` 併設が必須（メモリ: `pasta-debug-env-breaks-tests`）。
- 利用者向け情報の権威はマニュアル。辞書の作例もマニュアル記載の文法だけで書く。
- `pasta_lua` のテストが `sample.generated.lua` の改行だけ書き換えることがある（メモリ）。本 spec の検証でも生成物をリポジトリ内に落とさない。

## 2. 要件の実現可能性

### 2.1 段階表の各段階が現行文法で書けるか（Requirement 1.3）

| 段階 | 必要な表現 | マニュアルの根拠 | 判定 |
| ---- | ---------- | ---------------- | ---- |
| 1 しゃべらせたい | `＊OnBoot` + アクター行 | `lua/shiori-events.md` OnBoot（同名シーン実行） | 可 |
| 2 掛け合い（＋ランダムトーク） | `＊会話`（scene-name-alias で OnTalk の別名）、2 アクター、`％` アクター辞書、`：` の位置合わせ、`pasta.toml` の `talk_interval_*` | `grammar/actor-dictionary.md`・`action-line.md`・`shiori-events.md` 仮想ディスパッチャ・`reference/pasta-toml.md`（別名の記述は scene-name-alias の PR で追加される） | 可（議題 6 で 2 段目へ繰り上げ）。`＊会話` の動作は上流 `scene-name-alias` のマージ後（G16） |
| 3 表情 | `＠表情名：\s[n]`（アクター辞書）、台詞頭 `＠表情` | `actor-dictionary.md`・`words.md` | 可 |
| 4 毎回ちがう | 同名グローバルシーン複数（`＊会話` の繰り返し・単独 `＊`） | `block-structure.md`（同名シーンの抽選・単独 `＊`） | 可。別名は完全一致なので `＊会話朝` は OnTalk にならない点を段階表に注意として書く |
| 5 単語でちょこっと変える | `＠単語：a、b、c`、`＠単語` 参照 | `words.md` | 可（brief の 4 段目から独立） |
| 6 時報 | `＊時報12`・`＊時報その他`、`＄時１２` | `shiori-events.md` OnHour フォールバック・`variables.md` 日時変数 | 可 |
| 7 挨拶（イベント） | `＊OnGhostChanged`・`＊OnGhostChanging`、`＞transfer_req_to_var`、`＄ｒ０`、`＄％baseware.name`（プロパティ読み取り。`＊会話` 1 つ） | `shiori-events.md` OnGhostChanged（Reference0＝直前ゴーストの本体側の名前）・`variables.md` リクエスト変数・プロパティ変数・UKADOC OnGhostChanging / baseware.name | 可。`OnGhostChanging` は UKADOC 側のイベントで、同名シーンで応答できる（「ここに無いイベントも同名のシーンで応答できる」）。議題 7・8 で確定 |
| 8 触ったら反応 | `＊OnMouseDoubleClick` + `＄ｒ４` | `shiori-events.md` OnMouseDoubleClick（Reference4＝当たり判定）・`variables.md` の作例そのもの | 可 |
| 9 選択肢 | `＠？ジャンプ先「表示」`、`!select(秒)`、ローカル/グローバルシーンへのルーティング | `block-structure.md` 選択肢行・`shiori-events.md` OnChoiceSelectEx | 可 |
| 10 覚えていてほしい | `＄＊回数＝＄＊回数＋１`、参照（自分の回数を言う `＊会話` 1 つ） | `variables.md` グローバル変数（JSON 保存） | 可（議題 9 で確定）。**Research Needed**: 未代入の `＄＊回数` に `＋１` したときの挙動（`dsl-codegen-runtime-safety` 後の数値化ヘルパーが nil をどう扱うか）。初期値の代入が要るなら作例にその行を足す |
| 11 続き・分岐 | `＞シーン`（Call）、同名シーン複数＋前方一致の候補からのランダム選択（＝pasta の「分岐」）、ローカルシーン `・`、`＞チェイントーク`、ローカル／グローバルのジャンプの違い（ローカルが先・ローカル候補があればグローバルは候補外） | `call-jump.md`「前方一致によるターゲット解決」「スコープ解決アルゴリズム」「候補の選択」（候補をシャッフルし一巡するまで同じシーンを繰り返さない）・`block-structure.md` 同名グローバルシーン | 可。pasta の分岐は IF ではなくランダムジャンプが基本（議題 2 で確認）。条件分岐は Lua 拡張で、作例では深入りしない |
| 12 Lua への入り口 | 「少し紹介する程度」の最小作例: シーン内 Lua ブロックに小さな関数を 1 つ書き `＞＠関数（）` で呼ぶ。`scripts/` は使わない | `call-jump.md` 条件分岐の実現・`block-structure.md` Lua ブロック・`lua/patterns.md` | 可（議題 3 で確定） |
| 13 配布 | SSP の nar 作成機能（開発者用機能を有効化 → 「ディレクトリをドロップした際に更新ファイルや NAR を作成」ON → ゴーストフォルダを SSP にドロップ） | UKADOC SSP ヘルプ `ssphelp/dev.html`・`ssphelp/config-dev.html`（`OnNarCreating`/`OnNarCreated` が発生） | 辞書差分なし。12 段目と同じ中身を `.nar` にする。`pasta_check release`/`release.ps1` は内製ツールのため案内しない（議題 1 で決定） |

### 2.2 ギャップ（Missing / Unknown / Constraint）

| # | 項目 | 種別 | 内容 |
| - | ---- | ---- | ---- |
| G1 | 段階辞書の置き場所と形 | Resolved | 議題 5 で「配布辞書 `dic/` そのものが正本、章 N の辞書 ＝ 章 1〜N で追加されたファイルの集まり」に確定（Option D）。別置きの段階ディレクトリは持たない。ファイル名は段階番号を接頭にしたテーマ別（例: `01-boot.pasta`）。現行の 5 ファイル構成（`actors/boot/talk/click/choice`）は章ごとの 12 ファイル前後に組み替わる |
| G2 | 全段階を実ローダーで読み込むテスト | Missing | hello-pasta の `master/` を tempdir へコピーし、`dic/` を「段階 1〜N のファイルだけ」に絞って `PastaLoader::load`（＋`OnBoot` 疎通）を N = 1〜12 でループ。既存の `self_deploy_integration_test.rs` の tempdir/ctor パターンを流用できる。段階表（ファイル→段階の対応）をテストが読める形で持つ必要がある（G13） |
| G3 | `OnBoot` 疎通の確認手段（Requirement 4.2） | Resolved | `pasta_sample_ghost` の dev-deps は `pasta_lua` のみだが、`PastaLoader::load` が返すランタイムの `exec(&str)`（`pasta_lua/src/runtime/exec.rs:31`）で `require "pasta.shiori.entry"` → `SHIORI.request({id="OnBoot", method="get", version=30})` を実行すると SHIORI 応答文字列が返る（`pasta_lua/tests/shiori/event_handler_test.rs:170-186` の方式）。dev-dep 追加なしで C2 が成立する。`pasta_shiori` 経由（`PastaShiori::load`/`request`、`shiori_sample_ghost_test.rs:37-54`）は C3 の予備 |
| G4 | 最終段階＝配布辞書の一致検証 | Resolved | Option D により構成で満たされる（最終段階＝全ファイル＝配布辞書）。代わりに「段階表が列挙するファイル集合 ＝ `dic/` の実ファイル集合」の一致検査が要る（Requirement 4.3） |
| G5 | `OnBoot` ゴールデン | Constraint → 更新 | `pasta_shiori` の 3 テストが `OnBoot` 単一シーン・固定文を固定。ゴールデンは `actors.pasta` の `＠通常`→`\s[1]`/`\s[11]`、`pasta.toml` の `[talk]` 待ち時間と `[actor]` の spot にも依存する（Requirement 6.4 で不変）。議題 5 の「ファイル追加のみ」により `OnBoot` は 1 段目の形（女の子の一言）で固定され、配布辞書の `OnBoot` が変わる → 3 テストのゴールデン文字列・assert 条件を更新する（Requirement 5.1）。`OnBoot` を複数定義しない（単一シーン・決定性は保つ）。1 段目はアクター辞書の表情を使わないので、`OnBoot` の出力に `\s[n]` が含まれるかは 1 段目の作例の書き方次第（設計で決める。アクター宣言だけの辞書で `\p[0]` が出るかを確認） |
| G6 | 構造テストの固定値 | Constraint | `OnTalk` 5〜10（`talk.pasta`）、`OnMouseDoubleClick` 7 以上（`click.pasta`。`choice.pasta` の 1 個を含めると現状 8 定義）、`時報12`・`時報その他`・`＄時１２`、`src/scripts.rs` のユニットテストが「シーン内で使う `＠表情` はすべて `actors.pasta` に定義あり」を検査、`choice.pasta` は dist_src の必須一覧に無い。`ontalk_probe_test.rs` が `OnTalk` シーンの存在を要求。教材化で件数を減らすならテスト更新（Requirement 5.2） |
| G7 | `tutorial-check.mjs` の逐語一致 | Constraint → 追従 | `tutorial-check.mjs` は `DIC_FILES` に 5 ファイル名を固定で列挙（`:40-47`）しており、章別ファイルへの組み替えでファイル名の時点で `missing-source` になる。本文も辞書の行を引用している（`:157`・`:182` が `起動したよ～`、`:235-236`・`:271` が `＄ゴースト名`、`:7`・`:435` と `getting-started/index.md:11` が「hello-pasta と同じになる」と明言）。議題 11: 本 spec が機械的に追従する（`DIC_FILES` を `dic/` の実ファイルから導く、`first-ghost.md` の pasta ブロック差し替えと引用行・見出しの修正、`tutorial-check-test.mjs` の前提合わせ）。照合方式の拡張（段階ごと）と全面書き直しは下流 spec の持ち場 |
| G8 | `OnGhostChanged` 作例の「起動挨拶が二重にならない」注意 | Constraint | 作例は `OnGhostChanged` で挨拶し `OnBoot` は来ない前提。読者向けコメントで説明（辞書コメント）。Lua 側で 204 を返す道もあるが入門では使わない |
| G9 | シーン名の前方一致衝突 | Constraint | 新シーン名（例: `挨拶` は choice.pasta に既存。`時報`・`OnTalk`・`OnMouse…` で始まる名前は候補に混ざる）。e2e テストが追加するシーン名との衝突も確認（`scene_kick_*` が追記するシーン名を設計で列挙） |
| G10 | emo2 側の名前 | Constraint | `OnGhostChanged` の Reference0 は `むらさき`（配布版・DEBUG 版で共通）、Reference2 は `えも？？`/`えも2DEBUG`。作例は `＄ｒ０` を使う |
| G11 | プロパティ読み取りの作例 | Resolved | get_property は非同期コールバック（`\![get,property,OnPastaCallBack{N}…]`）を挟むが読者からは 1 行で動く。議題 8: `＄％currentghost.name`（自分の名前を名乗る）は削り、7 段目のファイルに `＊会話` を 1 つ足して `＄％baseware.name` を読む（UKADOC で `baseware.name`・`baseware.version`・`ghostlist.count` の実在を確認。`username` プロパティは同時起動中の相手ゴーストの呼ばれ方で、自分のユーザー名ではない）。`byte_invariant_test` は OnTalk を登録しない fixture を使っており、この作例に依存しない |
| G12 | 配布物の変化の周知 | Resolved | `crates/pasta_sample_ghost/RELEASE.md` はリリース**手順書**であり利用者向けの変更履歴ではない。リリースノートは `release-workflow`/`release-ci` が git log を Conventional Commits の種類で分類して生成する。したがって周知の経路は、本 spec のマージコミット（PR タイトル）を `feat(pasta_sample_ghost): …` の利用者向けの言葉で書くこと。あわせて `crates/pasta_sample_ghost/README.md` の辞書構成の節（`:125-146` のツリー）を追従させる。`.nar` 同梱の文書は増やさない（`release-ci` が成果物の追跡をやめる予定） |
| G13 | 段階表の置き場所 | Missing | 下流が逐語参照する正本。候補: `crates/pasta_sample_ghost/ghosts/hello-pasta-stages/README.md`（段階辞書と同居）か `crates/pasta_sample_ghost/README.md` の節 |
| G14 | 段階辞書の置き場所の制約 | Constraint | `release.ps1` は `ghosts\hello-pasta` を固定で対象にし、`pasta_check release` は対象ディレクトリ全体を無選別にコピーする（`pasta_check/src/release.rs:19`）→ **`ghosts/hello-pasta/` の下に段階辞書を置くと `.nar` に混入する**。`pasta_shiori` の e2e も `ghost/master` 全体を tempdir へコピーする。`pasta.toml` の `pasta_patterns = ["dic/*.pasta"]` は再帰しないので `dic/` の下にサブフォルダを置いても読み込まれない。`ghosts/` 直下の兄弟ディレクトリ（例: `ghosts/hello-pasta-stages/`）なら配布・e2e のどちらにも拾われない |
| G16 | `＊会話` → `OnTalk` の別名（上流 spec `scene-name-alias`） | Blocking（実装） | 2026-10-08 確認。`pasta.toml` にシーン名別名表（既定「会話 → OnTalk」1 件、表を書けば既定を丸ごと置換）。置換は `SceneRegistry::sanitize_name` の前段で登録・検索の両方に効き、宣言・Call/Jump・選択肢・`SCENE.co_exec` すべてが対象。**完全一致のみ**（`＊会話・朝`・`＊会話朝` は別シーン）。仮想ディスパッチャは不変（候補名 "OnTalk" のみ）。`＊OnTalk` は後方互換で両方書けば同じ候補群。時報側は不変。マニュアル（block-structure・call-jump・shiori-events・pasta-toml）とスキル references は同 PR で更新。マニュアルが `＊会話` を汎用例題名として 21 か所で使っている件は向こうの要件で整理。brief のみ・PR 未着手。ロードマップ上は Phase 11 Wave 6 だがユーザーが最優先で PR を出させる方針。本 spec: 設計は待たず、**実装の着手をマージ後にゲート**（議題 6）。辞書ディスパッチャ案（`＊OnTalk` → `＞会話`）は不要になった |
| G15 | `manual.yml` のトリガー | Constraint | `manual.yml` の `paths:` に `crates/pasta_sample_ghost/**` が無い。辞書だけを変える PR では tutorial-check が走らず（`build.yml` の `cargo test --all` だけ走る）、逐語一致の崩れが main への push まで見つからない。段階辞書の検証（Requirement 4.7）も同じ理由で `manual.yml` 側では PR 時に走らない。議題 11: 本 spec が `paths` に hello-pasta の `dic/**` を加える（Requirement 5.4b） |

### 2.3 複雑さの信号

- アルゴリズムは無し。**作例の執筆**（教材化・emo2 の制約・前方一致の回避）と、**フィクスチャをループするテスト**が主。
- 外部連携は emo2（名前と台詞の制約のみ。コードの結合は無い）。
- 既存テストとの整合（G5〜G7）が最大のリスク。辞書の変更と `first-ghost.md` 差し替えとテスト更新を 1 PR で揃える必要がある。

## 3. 実装アプローチの選択肢

### Option D（採用。議題 5）: 配布辞書そのものを章ごとのファイルに分け、段階 N ＝ 先頭 N ファイル

- `ghosts/hello-pasta/ghost/master/dic/` のファイルを章ごとに分割し、段階番号を接頭にしたテーマ別の名前にする（例: `01-boot.pasta`・`02-…pasta`）。段階 N の辞書は段階 1〜N で追加されたファイルの集まり。別置きの段階ディレクトリも最終段階の複製も持たない。
- 検証は hello-pasta の `master/` を tempdir にコピーし、`dic/` を段階 1〜N のファイルに絞って読み込み＋`OnBoot` 疎通を N = 1〜12 でループ。段階表（ファイル→段階）とディレクトリの実ファイルの一致も検査。
- 前提: すべての段階が「ファイルを足すだけ」で成立する（Requirement 1.6）。`OnBoot` は 1 段目の形で固定、ゴールデンは更新（G5）。
- トレードオフ: ✅ 正本が 1 つ、二重管理なし、下流の tutorial-check が「章 N のコードブロック ＝ ファイル N」で 1 対 1 に照合できる、読者も「ファイルを足すだけ」。❌ 序盤の作例を「`OnBoot` を育てない」形に組み替える必要がある。配布辞書のファイル数が 5 → 12 前後に増える（`dist_src_validation_test.rs`・`integration_test.rs` の file 単位の検査は更新）。
- 設計で確認: アクターの宣言（1〜2 段目）と表情の定義（3 段目）を別ファイルに分けて同じアクターに合流できるか（アクター辞書 `％女の子` を 2 ファイルで定義したときの挙動）。できなければ、2 段目で宣言と表情を一度に書くか、1〜2 段目をアクター辞書なしで成立させる。

以下 A〜C は議題 5 以前の候補（記録として残す）。

### Option A: 既存クレートの拡張（段階辞書を `ghosts/` 配下へ、既存テストを拡張）

- `crates/pasta_sample_ghost/ghosts/hello-pasta-stages/01-…/dic/` のように段階ディレクトリを並べ、最終段階は配布辞書と同一内容の複製（G4 の一致テストで守る）。
- `self_deploy_integration_test.rs` に「段階ディレクトリを列挙 → hello-pasta の設定・シェルと合成して tempdir へ → `PastaLoader::load`」のループテストを追加。
- トレードオフ: ✅ 既存の tempdir/ctor パターンをそのまま使える、下流が参照するパスが配布物のすぐ隣で分かりやすい。`ghosts/hello-pasta/` の**外**（兄弟ディレクトリ）に置けば `release.ps1`/`pasta_check`/`pasta_shiori` e2e のどれにも拾われない（G14 で確認済み）。❌ 最終段階の複製が二重管理になる（一致テストで補う）。

### Option B: 新規コンポーネント（テスト専用フィクスチャ ＋ 新テストファイル）

- `crates/pasta_sample_ghost/tests/fixtures/stages/NN/dic/` と `tests/tutorial_stages_test.rs` を新設。配布物とテストの境界が明確。
- トレードオフ: ✅ 配布パイプラインに触れない、責務が分かれる。❌ 下流（マニュアル執筆）が `tests/fixtures` を正本として逐語引用するのは置き場所として不自然。段階表 README も tests 配下に置くことになる。

### Option C: ハイブリッド（最終段階＝配布辞書そのもの、途中段階だけ別置き）

- 途中段階（1〜11）は段階ディレクトリに置き、最終段階は `ghosts/hello-pasta/ghost/master/dic/` を直接指す（複製を作らない）。段階表で「12 段目＝配布辞書」と明記。
- トレードオフ: ✅ 二重管理が消え、Requirement 2.3 が構成で満たされる。❌ 下流が「段階 12 のディレクトリ」を同じ規則で引けない（例外規則が 1 つ要る）。

### 検証手段の選択肢（G3）

- **C1**: `PastaLoader::load` のみ（読み込み・トランスパイル・Lua 起動まで）。最小。`OnBoot` 疎通（Requirement 4.2）は満たさない。
- **C2**: `pasta_lua` のランタイム API から `OnBoot` を発火（`runtime.exec` で `SHIORI.request({id="OnBoot", …})`。G3 で成立を確認済み）。dev-dep 追加なし。議題 10 により、実イベントを初めて扱う段階では同じ経路で `OnGhostChanged`・`OnMouseDoubleClick`・`OnChoiceSelectEx` などを Reference 付きで 1 回送る（`SHIORI.request` の `reference` の渡し方は `pasta_lua/tests/shiori/` のテストを手本に設計で確認）。
- **C3**: `pasta_shiori` を dev-dep に足し `TestEnv` 相当で SHIORI/3.0 リクエストを送る。実配布に最も近い。❌ dev-dep が増え、`pasta_shiori` → `pasta_sample_ghost` → … の循環は無いか確認（現在は `pasta_shiori/tests` が `pasta_sample_ghost/ghosts` をパスで参照しているだけで Cargo 依存は無い）。

## 4. 工数とリスク

| 項目 | 見積 | 根拠 |
| ---- | ---- | ---- |
| 工数 | **M（3〜7 日）** | 作例 12 段の執筆と教材化、ループテスト 1 本、既存テスト・`first-ghost.md`・README の追従。新しい技術要素は無い |
| リスク | **Medium** | 技術は既知だが、ゴールデン（G5）・構造テスト（G6）・tutorial-check（G7）の 3 系統を同時に揃える必要があり、emo2 の台詞制約と前方一致（G9）で作例の自由度が狭い |

## 5. 設計フェーズへの推奨と申し送り

- **採用**: Option D（配布辞書を章ごとのファイルに分け、段階 N ＝ 先頭 N ファイル。議題 5）＋ 検証は C2（G3 で成立確認済み）。A〜C は不採用。
- **キーとなる決定**: ファイル名の規則（番号の桁数・区切り・テーマ名）、段階表の正本の置き場所とテストから読める形（G13。例: 段階表 Markdown の表をテストがパースする／ファイル名の番号を段階とみなす）、ループテストの形（既存テストの拡張か新規ファイルか。最も近い手本は `pasta_lua/tests/transpiler/final_regression_test.rs` の `broad_fixtures()`）、`OnBoot` の新しい固定文、`manual.yml` のトリガー追加の要否（G15）。
- **実装前の前提（ゲート）**: 上流 `scene-name-alias` の PR が main に入っていること（G16）。設計は `＊会話` 前提で先に進める。実装着手時に本 spec のブランチへ main を取り込み、`＊会話` の動作と `pasta.toml` の別名表の記述（マニュアル）を確認してから辞書を書く。段階表の 2 段目のマニュアル根拠（別名の節）は、その PR で追加されるページを指す。
- **Research Needed（設計で調べる）**:
  1. `＄＊回数＝＄＊回数＋１` の初回（nil）挙動（G2.1 段階 10）。
  2. `scene_kick_*_e2e_test.rs` が追記するシーン名の一覧（G9）。
  3. アクター辞書 `％女の子` を 2 つのファイルに分けて定義したときの挙動（宣言と表情の合流。Option D の前提）。
  4. アクター辞書なし（または表情なし）のアクター行が `\p[0]` を出力するか（1 段目の `OnBoot` と新ゴールデンの形）。
- **要件ディスカッション（2026-10-08 完了）**: [OPEN-1]〜[OPEN-11] はすべて解消（議題 1〜11 と G12）。決定は `requirements.md` 本文に反映済み。未解決の論点は残っていない。

---

# 設計フェーズの調査と決定（2026-10-08 `/kiro-spec-design -y`）

## Summary
- **Feature**: `hello-pasta-tutorial-stages`
- **Discovery Scope**: Extension（既存の配布辞書・テスト・マニュアル CI の組み替え。新しい外部依存なし）→ light discovery ＋ コード調査
- **Key Findings**:
  - 未代入の `＄＊回数＋１` は値なし（`act.lua` `arith_operand` が nil を返し警告）で、回数が永久に始まらない。DSL には比較・条件分岐が無く（`markers.md`「演算子」）、Lua 抜きでは初回の初期化を書けない → 要件 3.6 の作例は現行実装のままでは成立しない（設計 Q1）。
  - hello-pasta の `surfaces.txt` には当たり判定（collision）が無いため、ダブルクリックの `Reference4`（当たり判定の識別子。UKADOC）は空になる → 8 段目の `＄ｒ４` の作例は実機で空文字を言う（設計 Q2。シェルは `hello-pasta-shell-art` の持ち場）。
  - `tutorial-check.mjs` の抽出正規表現は最初の ```` ``` ```` でブロックを閉じるため、```` ```lua ```` を内側に持つ辞書（12 段目）は途中で切れて逐語一致しない → 抽出を CommonMark のフェンス長規則に合わせる修正が要る（設計 Q5）。

## Research Log

### R1: `＄＊回数＝＄＊回数＋１` の初回（nil）挙動
- **Sources**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` `arith_operand`・`ACT_IMPL.arith`、`book/src/grammar/variables.md`「算術の評価」（`＄x＝＄未代入＋1` の例）、`markers.md`「演算子」、`pasta_lua/pasta_scripts/pasta/save.lua`、`book/src/reference/pasta-toml.md` `[persistence]`
- **Findings**: 被演算子が nil なら警告 `act:arith - operand is not a number` を出して結果は nil。代入先 `save.回数` は nil のまま。`＆` 連結も nil で値なし。既定値を与える構文・設定（pasta.toml）・標準の GLOBAL 関数は無い。`OnFirstBoot` は SSP では初回だけ（204 なら OnBoot へ続く。UKADOC）で、読者は 1 段目で初回起動を済ませている。
- **Implications**: 初期化には (a) Lua 関数、(b) 前段のファイル（`OnFirstBoot` 等）への初期化行の追加、(c) ランタイムの機能追加 のいずれかが要る。(b) は要件 1.6（前のファイルを書き換えない）と読者の起動順で破綻、(c) は本 spec の範囲外。設計は (a) を仮定 A1 として草案を書き、要件 3.6・3.7 の調整を設計ディスカッションへ送る。

### R2: `scene_kick_*_e2e_test.rs` が追加するシーン名
- **Sources**: `crates/pasta_shiori/tests/scene_kick_e2e_test.rs`・`scene_kick_gate_e2e_test.rs`・`scene_kick_multibeat_e2e_test.rs`・`scene_kick_preempt_e2e_test.rs`
- **Findings**: 追加するシーンは `KickE2EProbe`・`GatePrevScene`・`GateUnresolvedSceneDoesNotExist`・`GateDropSceneAfterTeardown`・`KickMultiBeat`・`KickPreemptPrev`・`KickPreemptNew`・`KickForceOneShot`・`KickLastWinsA`・`KickLastWinsB`。追加ファイルは `dic/kick_e2e.pasta`・`kick_gate.pasta`・`kick_multibeat.pasta`・`kick_preempt.pasta`。`pasta.toml` の `talk_interval_min = 180  # …`・`talk_interval_max = 300  # …` の行を文字列置換する（`pasta.toml` は本 spec で変えないので影響なし）。
- **Implications**: 新しいシーン名を `Kick`・`Gate` で始めない（テストで検査）。ファイル名は `NN-` 接頭なので衝突しない。e2e は talk 間隔を 10 秒にするため、11 段目のチェイントークが `＊会話` に入ると進行中会話が生じうる（設計 Q6。実装時に e2e を流して確認）。

### R3: 同じアクター辞書を 2 ファイルで宣言したとき
- **Sources**: `book/src/grammar/actor-dictionary.md`「グローバルアクター辞書定義」、`words.md`
- **Findings**: 同名の `％アクター名` は別ファイルでも 1 つのアクターにまとまり単語は合算される。グローバル単語も同様。アクター名は pasta.toml の `[actor."名前"]` だけでもアクション行に使える。
- **Implications**: アクター辞書は 3 段目のファイルに 1 回だけ書けば十分で、1〜2 段目はアクター辞書なしで成立する。分割宣言は不要なので使わない（単純さ優先）。

### R4: 表情なしのアクター行と `\p[0]`
- **Sources**: `pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` `emit_actor_switch`・`spot_to_tag`、`actor-dictionary.md`「バルーン連携」、`pasta_shiori/tests/byte_invariant_test.rs` の現行ゴールデン
- **Findings**: 話者が切り替わるたびに `\p[spot]` を出す。spot は pasta.toml の `spot`（`％` 行が無ければそのまま）。表情の有無には依存しない。表情が無ければ `\s[n]` は出ない。句読点のウェイト `\_w[...]` は `[talk]` 設定で付く。
- **Implications**: 新しい OnBoot（女の子の一言・表情なし）の応答は `\p[0]〈台詞＋ウェイト〉\e` の形になる見込み。正確なバイト列は実装時に特性化採取する。`shiori_sample_ghost_test.rs` の `\s[` の assert は外す。

### R5: 段階の SHIORI 疎通の経路
- **Sources**: `pasta_lua/pasta_scripts/pasta/shiori/entry.lua` `SHIORI.request`、`event/init.lua` `EVENT.fire`、`event/choice_select.lua`、`pasta_shiori/src/lua_request.rs` `parse_request`
- **Findings**: `SHIORI.request(req)` は `EVENT.fire` の結果を 200/204、例外を 500 の応答文字列にする。`req` は `id`・`method`・`version`・`charset`・`sender`・`reference`（0 始まり）・`dic`・`date`。`OnChoiceSelectEx` は Reference1 を選択 ID、Reference2 をスコープとして前方一致検索し、見つからなければ 204。
- **Implications**: `pasta_lua` だけで（`pasta_shiori` への依存なしに）疎通できる。`date` は OnHour 系でのみ使うので、OnBoot・実イベントの検証では省ける見込み（仮定 A6。実装時に確認）。

### R6: ローダーの読み込み順・フェンス・その他
- **Findings**:
  - `pasta_lua/src/loader/discovery.rs` は `glob` で列挙し、辞書順に読む → `NN-` 接頭で段階順と一致。
  - `OnHour` の候補は `時報%02d` → `OnHour%02d` → `時報その他` → `OnHourOther`（`virtual_dispatcher.lua`）。`時報12` と `時報その他` は互いの前方一致にならない。
  - `OnGhostChanging` の Reference0 は切り替わる先のゴーストの本体側の名前、Reference1 は `manual`/`automatic`、Reference2 は名前（UKADOC）。204 なら続けて OnClose。
  - `pasta.dll`（約 3.8 MB）は `master/` に追跡されているが、ローダーには不要 → 段階ごとのコピーから外す。
  - `book/tools/tutorial-check.mjs` の `/```pasta[^\S\r\n]*\r?\n([\s\S]*?)```/g` は ````` ````pasta ````` の 2 文字目から一致し、内側の ```` ```lua ```` で閉じてしまう。
  - スキル `pasta-ghost-authoring` の `actors.pasta`・`talk.pasta` 等は汎用の分割例で、hello-pasta を指していない → 追従不要。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 段階表を Markdown 1 本（人とテストが共有） | `STAGES.md` の GFM 表をテストが行単位でパース | 正本が 1 つ。新規依存なし。下流が逐語参照しやすい | 見出し名の改名でテストが壊れる | **採用** |
| Markdown（人向け）＋ TOML（テスト向け） | 2 ファイル | パースが堅い | 二重管理。`toml` の dev-dep 追加 | 不採用 |
| ファイル名の番号だけを正本にする | 表はテストが読まない | 最小 | 要件 1.1・4.3（表と実ファイルの一致・検証イベントの保持）を満たさない | 不採用 |
| 疎通を `pasta_shiori` 経由（C3） | dev-dep に `pasta_shiori` | 実配布に最も近い | 依存追加 | 不採用（C2 で足りる） |

## Design Decisions

### Decision: 1 段 1 ファイル・`NN-name.pasta`（ASCII）
- **Context**: 要件 2.3（番号の桁数と区切りは設計で決める）。
- **Alternatives**: 1 桁番号／日本語のテーマ名／1 段に複数ファイル。
- **Selected**: 2 桁ゼロ埋め・半角ハイフン・ASCII 小文字の英単語。1 段 1 ファイル。
- **Rationale**: ローダーの辞書順で段階順になる。ASCII は `.nar`・URL・読者環境でのファイル名の扱いが単純。1 段 1 ファイルなら下流が「章 N ＝ ファイル N」で照合できる。
- **Trade-offs**: 1 段に作例を分けたくなっても 1 ファイルに収める。

### Decision: 段階表は `crates/pasta_sample_ghost/STAGES.md`（`ghosts/` の外）
- **Context**: 要件 1.1・2.7・6.3、G13・G14。
- **Selected**: クレート直下の Markdown。`ghosts/hello-pasta/` の下に置かない（`.nar` に入るため）。
- **Follow-up**: 見出し `## 段階表`・`## 検証イベント表` と列見出しを固定し、改名は Revalidation Trigger とする。

### Decision: 検証は新規 `tests/tutorial_stages_test.rs`（C2 経路）
- **Context**: 要件 4。既存 `self_deploy_integration_test.rs` は自己展開の検証が責務。
- **Selected**: 専用ファイルに段階の組み立て・ロード・`SHIORI.request` 疎通・一致・前方一致衝突をまとめる。コピー関数はファイル内に持つ（`tests/common` に移すと他バイナリで dead_code 警告）。
- **Trade-offs**: 15 行ほどのコピー関数が 2 か所に並ぶ。

### Decision: 辞書の文字列構造の検査は `src/scripts.rs` に一本化
- **Context**: 要件 5.2。`integration_test.rs` と `src/scripts.rs` が同じ検査を重複して持つ。
- **Selected**: `src/scripts.rs` を新ファイル構成に付け替え、`integration_test.rs` の重複 3 本は削除する。
- **Rationale**: 更新箇所を 1 つにする（意図は保つ）。

### Decision: シーンの `％女の子、男の子` 行を書かない
- **Context**: 現行は各シーンに `％` 行を書いている。立ち位置は pasta.toml の spot で決まる（R4）。
- **Selected**: 作例では書かない。1〜2 段目をアクター辞書・`％` 行なしで成立させ、作例を短くする。

### Synthesis（一般化・採用か自作か・単純化）
- **一般化**: 「段階 N の辞書」は「ファイル集合の前方 N 個」という 1 つの規則に落ちる。段階ごとのディレクトリや設定差し替えは持たない。
- **採用か自作か**: 疎通は既存の `SHIORI.request`（Lua）を使い、`pasta_shiori` を足さない。Markdown パーサのクレートは足さず行単位で読む（表の形を固定するので十分）。`tutorial-check` のフェンス抽出は CommonMark の規則に合わせる（新しい規則を作らない）。
- **単純化**: 前方一致の衝突検査は「全グローバルシーン名の対で前方一致しない」1 本に絞る。段階ごとの出力内容の照合はしない（要件 4.5）。`profile/` の使い回しなどの高速化は入れない。

## Risks & Mitigations
- 10 段目の作例が要件どおりには成立しない（R1）— 仮定 A1（Lua 関数）で草案を書き、設計ディスカッションで要件を調整する。
- 8 段目の `＄ｒ４` が実機で空（R6・UKADOC）— `hello-pasta-shell-art` へ当たり判定の追加を申し送る案を設計ディスカッションで決める。
- ロードだけでは `＊会話` の中身（10・11・12 段目の Call・Lua）が実行されない — 実装の最後に SSP/areka で目視確認する。
- 11 段目のチェイントークと `pasta_shiori` e2e の干渉（R2）— 実装時に e2e を流して確認。
- 上流 `scene-name-alias` が未マージ — 実装の着手をゲートする。`ontalk_probe_test.rs` も別名が前提。

## References
- UKADOC SHIORI Event: OnFirstBoot・OnGhostChanging・OnMouseDoubleClick（<https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html>）— 204 時の後続イベントと Reference の内容
- マニュアル `book/src/grammar/variables.md`・`markers.md`・`actor-dictionary.md`・`call-jump.md`・`block-structure.md`、`book/src/lua/shiori-events.md`
