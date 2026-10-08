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
| 7 挨拶（イベント） | `＊OnGhostChanged`、`＞transfer_req_to_var`、`＄ｒ０` | `shiori-events.md` OnGhostChanged（Reference0＝直前ゴーストの本体側の名前）・`variables.md` リクエスト変数 | 可。`OnGhostChanging` は UKADOC 側のイベントで、同名シーンで応答できる（「ここに無いイベントも同名のシーンで応答できる」） |
| 8 触ったら反応 | `＊OnMouseDoubleClick` + `＄ｒ４` | `shiori-events.md` OnMouseDoubleClick（Reference4＝当たり判定）・`variables.md` の作例そのもの | 可 |
| 9 選択肢 | `＠？ジャンプ先「表示」`、`!select(秒)`、ローカル/グローバルシーンへのルーティング | `block-structure.md` 選択肢行・`shiori-events.md` OnChoiceSelectEx | 可 |
| 10 覚えていてほしい | `＄＊名前＝…`、参照 | `variables.md` グローバル変数（JSON 保存） | 可。**Research Needed**: 未代入の `＄＊回数` に `＋１` したときの挙動（`dsl-codegen-runtime-safety` 後の数値化ヘルパーが nil をどう扱うか）。初期値の代入が要るなら作例の形が変わる |
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
| G7 | `tutorial-check.mjs` の逐語一致 | Constraint | 辞書を変えた瞬間に `manual.yml` が赤くなる。`first-ghost.md` のコードブロック差し替え（[OPEN-9]）が同じ PR に要る。さらに本文も辞書の行を引用している（`:157`・`:182` が `起動したよ～`、`:235-236`・`:271` が `＄ゴースト名`、`:7`・`:435` と `getting-started/index.md:11` が「hello-pasta と同じになる」と明言）ため、コードブロックだけ差し替えると本文が食い違う。照合方式の拡張（段階ごと）は下流 spec の持ち場 |
| G8 | `OnGhostChanged` 作例の「起動挨拶が二重にならない」注意 | Constraint | 作例は `OnGhostChanged` で挨拶し `OnBoot` は来ない前提。読者向けコメントで説明（辞書コメント）。Lua 側で 204 を返す道もあるが入門では使わない |
| G9 | シーン名の前方一致衝突 | Constraint | 新シーン名（例: `挨拶` は choice.pasta に既存。`時報`・`OnTalk`・`OnMouse…` で始まる名前は候補に混ざる）。e2e テストが追加するシーン名との衝突も確認（`scene_kick_*` が追記するシーン名を設計で列挙） |
| G10 | emo2 側の名前 | Constraint | `OnGhostChanged` の Reference0 は `むらさき`（配布版・DEBUG 版で共通）、Reference2 は `えも？？`/`えも2DEBUG`。作例は `＄ｒ０` を使う |
| G11 | `＄％currentghost.name` 作例 | Unknown | get_property は非同期コールバック（`\![get,property,OnPastaCallBack{N}…]`）を挟む。入門の段階表に置くか削るか（[OPEN-6]）。`byte_invariant_test` は OnTalk を登録しない fixture を使っており、この作例に依存しない |
| G12 | 配布物の変化の周知 | Resolved | `crates/pasta_sample_ghost/RELEASE.md` はリリース**手順書**であり利用者向けの変更履歴ではない。リリースノートは `release-workflow`/`release-ci` が git log を Conventional Commits の種類で分類して生成する。したがって周知の経路は、本 spec のマージコミット（PR タイトル）を `feat(pasta_sample_ghost): …` の利用者向けの言葉で書くこと。あわせて `crates/pasta_sample_ghost/README.md` の辞書構成の節（`:125-146` のツリー）を追従させる。`.nar` 同梱の文書は増やさない（`release-ci` が成果物の追跡をやめる予定） |
| G13 | 段階表の置き場所 | Missing | 下流が逐語参照する正本。候補: `crates/pasta_sample_ghost/ghosts/hello-pasta-stages/README.md`（段階辞書と同居）か `crates/pasta_sample_ghost/README.md` の節 |
| G14 | 段階辞書の置き場所の制約 | Constraint | `release.ps1` は `ghosts\hello-pasta` を固定で対象にし、`pasta_check release` は対象ディレクトリ全体を無選別にコピーする（`pasta_check/src/release.rs:19`）→ **`ghosts/hello-pasta/` の下に段階辞書を置くと `.nar` に混入する**。`pasta_shiori` の e2e も `ghost/master` 全体を tempdir へコピーする。`pasta.toml` の `pasta_patterns = ["dic/*.pasta"]` は再帰しないので `dic/` の下にサブフォルダを置いても読み込まれない。`ghosts/` 直下の兄弟ディレクトリ（例: `ghosts/hello-pasta-stages/`）なら配布・e2e のどちらにも拾われない |
| G16 | `＊会話` → `OnTalk` の別名（上流 spec `scene-name-alias`） | Blocking（実装） | 2026-10-08 確認。`pasta.toml` にシーン名別名表（既定「会話 → OnTalk」1 件、表を書けば既定を丸ごと置換）。置換は `SceneRegistry::sanitize_name` の前段で登録・検索の両方に効き、宣言・Call/Jump・選択肢・`SCENE.co_exec` すべてが対象。**完全一致のみ**（`＊会話・朝`・`＊会話朝` は別シーン）。仮想ディスパッチャは不変（候補名 "OnTalk" のみ）。`＊OnTalk` は後方互換で両方書けば同じ候補群。時報側は不変。マニュアル（block-structure・call-jump・shiori-events・pasta-toml）とスキル references は同 PR で更新。マニュアルが `＊会話` を汎用例題名として 21 か所で使っている件は向こうの要件で整理。brief のみ・PR 未着手。ロードマップ上は Phase 11 Wave 6 だがユーザーが最優先で PR を出させる方針。本 spec: 設計は待たず、**実装の着手をマージ後にゲート**（議題 6）。辞書ディスパッチャ案（`＊OnTalk` → `＞会話`）は不要になった |
| G15 | `manual.yml` のトリガー | Constraint | `manual.yml` の `paths:` に `crates/pasta_sample_ghost/**` が無い。辞書だけを変える PR では tutorial-check が走らず（`build.yml` の `cargo test --all` だけ走る）、逐語一致の崩れが main への push まで見つからない。段階辞書の検証（Requirement 4.7）も同じ理由で `manual.yml` 側では PR 時に走らない |

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
- **C2**: `pasta_lua` のランタイム API から `OnBoot` を発火（`runtime.exec` で `SHIORI.request({id="OnBoot", …})`。G3 で成立を確認済み）。dev-dep 追加なし。
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
- **要件ディスカッションへ**: `requirements.md` の [OPEN-3]・[OPEN-4]・[OPEN-6]・[OPEN-7]・[OPEN-9]（[OPEN-1] は議題 1、[OPEN-11] は議題 2、[OPEN-5] は議題 3、[OPEN-2] は議題 4、[OPEN-8] は議題 5、[OPEN-10] は G12 で解消）。特に [OPEN-9]（`first-ghost.md` のコードブロック差し替えを本 spec が持つか）は、本 spec の完了条件（Requirement 5.5）に直結する。
