# ギャップ分析: manual-shell-guide

- 実施日: 2026-10-16（worktree `claude/manual-shell-guide-6a95f7`、base `3071114a`）
- 対象: `requirements.md`（要件 1〜11・未決事項 12 件）と、既存のマニュアル（`book/`）・検査ツール（`book/tools/`）・見本のシェル（`crates/pasta_sample_ghost/ghosts/hello-pasta/shell/master/`）
- 位置づけ: 判断ではなく材料。選択肢と制約を並べ、決めるのは要件ディスカッションと設計の段である。

## 1. 分析の要約

- **新しく書くのは章 1 つと導線 3 か所だけで、仕組みの追加はほぼ要らない**。目次・ビルド・台詞の検査・リンク検証・検索の検査は、`reference/` の直下に章を 1 つ置けば自動で対象になる。直すのは、章の数を決め打ちした 3 か所（60 → 61）だけである。
- **題材の事実はほぼそろっている**。見本のシェルの実物（18 枚・`Face`/`Head`/`Body`・吹き出しの 4 行）、当たり判定の重なりと部位名の実機の結果（`hello-pasta-shell-art` の Implementation Notes）、絵のライセンス（`art/README.md`）、同梱バルーンの規則（`crates/pasta_check/README.md`）がある。欠けているのは「シェルに無いサーフェス番号を指したときに SSP で何が起きるか」の実機の事実だけ（R3.6）。
- **入門の章への導線には、執筆規約の制約が 2 つかかる**。段の章の「もっと詳しく」は段階表の `使う文法要素` と同じに保つ決まりがあり（第 8 節チェックリスト）、台詞以外の段落は 1 行 1 文に限られる（`C-prose`）。リンクは本文の箇条書きに置くのが無理の無い形。
- **抜き出しの照合（R10.3）は既存の道具に無い**。`tutorial-check.mjs` は `pasta` ブロックしか見ず、マニュアルの CI はシェルのフォルダの変更では走らない。照合を入れるなら、置き場所（book の道具か、`pasta_sample_ghost` の Rust テストか）を設計で決める。
- **推奨は Option B（`reference/` に新章＋導線）**、照合を入れるなら Option C。規模は S、リスクは Low（実機の確認 1 点だけが Medium 寄り）。

## 2. 現状の調査

### 2.1 マニュアルの構成（`book/src/SUMMARY.md`）

| パート | 章の数 | 備考 |
| ------ | ------ | ---- |
| 表紙 `introduction.md` | 1 | 扉のナビ（15〜22 行）はパートごとに 1 つ。リファレンスは `reference/startup.md` を指す（新章を足しても直さなくてよい） |
| 入門 / チュートリアル | 16 | `C-order` が並びを、`C-prose` が 16 章を、`C-sections` が段の章 13 を数える |
| Pasta DSL 文法 | 10 | |
| Lua API / コーディング | 14 | |
| デバッグ | 6 | |
| リファレンス | 3 | `startup.md`・`pasta-toml.md`・`external-links.md`。索引の章は無く平たい |
| 内部設計 | 10 | |
| 合計 | 60 | `SUMMARY.md` を除く `book/src` の全 `.md` |

### 2.2 シェルについて今マニュアルにあること

| 場所 | 書いてあること | 本 spec との関係 |
| ---- | -------------- | ---------------- |
| `getting-started/setup.md` 節「シェルの中身」 | ファイルの種類の表、見本の絵 2 枚の表（`book/src/img/hello-pasta/`）、「接点はサーフェス番号だけ」「`pasta.dll` はシェルを読まない」「バルーンは入っていない」「同じ番号の画像を差し替えるところから始められる」、UKADOC の 2 ページへのリンク | 導線を足す場所（R9.1）。節を残すか薄くするかは未決 5 |
| `getting-started/03-face.md` | 表情名とサーフェス番号の表（0〜8・10〜18）、アクター辞書の書き方 | R3.3・R5.4 のリンク先。導線を足す場所（R9.2） |
| `getting-started/08-touch.md` | 当たり判定の名前が `＄ｒ４` に入る、外を触ると空。部位名は「頭の当たり判定に `Head` と付けているシェルでは」と仮定の形 | R3.4・R6.2 のリンク先。導線を足す場所（R9.3）。仮定の言い回しは未決 11 |
| `getting-started/setup.md` 節「pasta.toml を書く」 | `spot` の `0`/`1` | R3.5・R7.5 |
| `reference/pasta-toml.md` | `[actor]` の `spot` | R7.5 のリンク先 |
| `reference/external-links.md` | UKADOC のさくらスクリプト一覧など | シェルの UKADOC ページは載っていない（足すかは設計） |
| `getting-started/13-nar.md` | SSP の NAR 作成機能で `.nar` を作る | `pasta_check` には触れていない（R7.4 の読み先の問題） |

- 絵の差し替え・表情の追加・当たり判定の書き方・吹き出しの位置・バルーンの同梱は、マニュアルのどこにも無い（brief の Current State のとおり）。

### 2.3 見本のシェルの実物（題材）

- `shell/master/descript.txt`: `charset`・`type,shell`・`name,master`・`craftman`・`craftmanw`・`seriko.use_self_alpha,1`・`sakura.balloon.offsetx,100`・`sakura.balloon.offsety,-105`・`kero.balloon.offsetx,50`・`kero.balloon.offsety,120`。
- `shell/master/surfaces.txt`（146 行）: 18 ブロック。各ブロックは `element0,overlay,surfaceN.png,0,0` と `collisionex0,Face,rect,…`・`collisionex1,Head,rect,…`・`collisionex2,Body,rect,…`。
  - 女の子: Face 124,130,204,200／Head 96,84,233,201／Body 110,218,221,403。男の子: Face 126,160,208,215／Head 105,75,227,216／Body 124,233,207,394（Body はポーズで共通）。
- 立ち絵は 333×500 の透過 PNG 18 枚（`surface0`〜`8`・`surface10`〜`18`。`surface9` は無い）。
- 実機の結果（`.kiro/specs/completed/hello-pasta-shell-art/tasks.md` Implementation Notes 6.1）: 重なりは先に書いた方が勝つ（`Face` が `Head` に勝つ）。部位名は 顔の中央 = `Face`、髪・帽子 = `Head`、胴 = `Body`、靴・首の隙間・上げた手 = 空。
- 吹き出しの位置は SSP の既定バルーンの高さ（約 168 px）が前提。吹き出しが反対側へ出たときの値（SSP 専用 `offsetxr`/`offsetyr`）は置いていない（未確認のリスクとして記録）。
- 形の保証: `crates/pasta_sample_ghost/tests/shell_assets_test.rs` が、枚数・寸法・透過・大きさ・18 定義・`Face`/`Head`/`Body`・矩形の包含関係・画素の一致・本の写し 2 枚とのバイト一致（460〜478 行、`[0, 10]`）を検査する。
- 絵のライセンス: Unlicense（クレートの本体は MIT）。記録は `crates/pasta_sample_ghost/art/README.md`（`.nar` に入らない）。

### 2.4 執筆規約（`book/AUTHORING.md`）

- リファレンスのパートに固有の節は無い。新章は第 1〜5 節と第 7 節に従う（生成対象にするなら第 4 節も）。導入・締めは台詞、本体は普通の文体（第 1 節の鉄則・第 2 節の表）。
- 第 7 節「置ける場所」: 生成対象でも内部設計でもない章は、本体にも台詞を置ける（置かなくてもよい）。
- 第 8 節（入門だけ）: 「シェルのファイルの中身は書き写さない」「画像を置かない」「回避レシピを載せない」「『もっと詳しく』のリンク先は段階表の `使う文法要素` と同じ」「台詞以外の段落は 1 行・句点 1 つ（`C-prose`）」。
  - FYI: 「画像を置かない」と、`hello-pasta-shell-art` が `setup.md` に足した見本の絵の表は、規約の文面の上では食い違って見える（検査は通っている）。本 spec の持ち場ではないが、`setup.md` を触るときに気づく点として記録する。
- 第 8 節「章を足す・名前を変えるとき」の表は入門の章向けだが、直す場所（`SUMMARY.md`・`verify-scripts-test.mjs`・`talk-test.mjs`・`gen-skill-refs-test.mjs`）は新章にもそのまま当てはまる。

### 2.5 検査ツールと、章を足すと動く数

| 検査 | 新章（`reference/` 直下）への効き方 | 直す所 |
| ---- | ----------------------------------- | ------ |
| `book/tools/verify-scripts-test.mjs` 81 行 | `T-syntax`・`T-intro`・`T-outro` の PASS 件数 60 | 61 に（78 行のコメント「全 60 章」も） |
| `book/tools/talk/talk-test.mjs` 453〜454 行（J-9） | `talk.mjs --stats` の章の行数 60 | 61 に |
| `book/tools/gen-skill-refs-test.mjs` 599 行（K-10） | `book/src` の章の数 60。続く 600〜606 行で全章の `chapterRegions`（H1 で始まり、フェンス外の `---` が 2 本以上）を確かめる | 61 に。新章は H1／導入／`---`／本体／`---`／締めの形にする |
| `verify-content.mjs` D-voice（319〜364 行） | `VOICE_DIRS` は再帰しない。`reference/` 直下なら、フェンス外の散文に Claudia の口調マーカーが要る（導入・締めの台詞で満たせる）。D-codevoice はフェンス内の口調を禁じる | 無し（`reference/shell/x.md` のように下のフォルダへ置くと D・T の検査から外れ、数の検査だけが増えるので避ける） |
| `verify-content.mjs` T-syntax/T-intro/T-outro | 導入・締めは台詞だけ、両方の話し手 | 無し |
| `verify-content.mjs` T-assets | `book/src/img/claudia/` は表情 16 枚＋LICENSE だけ | 新しい画像をそこへ置かない |
| `book/tools/link-check.mjs` | 相対 `.md` リンク・`#anchor`（`manual-link-anchor-check` 以降）・GitHub の blob/tree URL の実在・README と skill のリンク | 無し（`art/README.md` を GitHub の URL で指すなら実在検査の対象になる） |
| `verify-static.mjs` | 目次の全章に HTML と前後のナビがあり、画像などの参照が解決する | 無し |
| `verify-search.mjs` | 最初の台詞の語で、その章が検索に出る | 最初の台詞を章に固有の語で始める |
| `tutorial-check.mjs` | `getting-started` の `pasta` ブロックだけ。`text`・`toml` は見ない | 無し（シェルの抜き出しは照合されない） |
| `gen-skill-refs.mjs` | `GENERATION_MAP`（23 件・固定）に載せなければ生成されない | 生成対象にしないなら無し。する場合は `EXPECTED_MAP` と「23」の全箇所・SKILL.md からのリンク・生成物のコミット |
| `cargo test -p pasta_sample_ghost` | 本の写し 2 枚とシェルのバイト一致 | 新章で別の絵の写しを置くなら、検査の番号の列 `[0, 10]` を伸ばすかどうか |
| `.github/workflows/manual.yml` | `paths` は `book/**`・2 つのスキル・`crates/*/README.md`・hello-pasta の `dic/**` | `shell/**` は含まない（シェルを変えても manual CI は走らない） |

### 2.6 隣の spec とファイルの重なり

| spec | 重なる所 | 取り決め |
| ---- | -------- | -------- |
| `touch-poke-shortcuts` | `08-touch.md`（本文を書き換える） | 後から入る側が rebase で合わせる（roadmap 159 行）。本 spec が足すのはリンクだけ |
| `pasta-check-dic-validate` | 章を足す場合、`SUMMARY.md` と 3 つの数 | 後から入る側が数を合わせる（roadmap 160 行） |
| `manual-novel-guide`（0.5.0） | 章の数、シェルの説明との分担（brief 議題「シェルの説明との分担」） | 本 spec が先に入る見込み。ノベルの画面のシェルは本章の範囲外 |
| `shell-element-click`（0.5.0） | エレメント単位のクリック（`surfaces.txt` の新しい書き方） | 本 spec の範囲外。当たり判定の説明は今の `collisionex` に限る |
| `getting-started-screenshots` | 入門の章と見本のシェルの絵 | スクリーンショットは向こうの持ち場（未決 12） |
| `manual-print-media-refs` | `verify-static.mjs` | 動画・画像の参照の検査。新章が動画を持たなければ重ならない |

## 3. 要件と既存資産の対応表

凡例: **Missing** = 新しく作る／**Unknown** = 調べる必要がある／**Constraint** = 既存の規約・検査が縛る／**Exists** = 既にある

| 要件 | 既存資産 | ギャップ |
| ---- | -------- | -------- |
| R1 置き場所と目次 | `SUMMARY.md` のリファレンス（平たい 3 章） | **Missing**: 新章。**Constraint**: `reference/` 直下に置かないと D-voice・T-* から外れる。最初の台詞の語が検索の検査に使われる |
| R2 書き方と出どころ | AUTHORING 第 1〜5・7 節 | **Constraint**: リファレンス用の規約が無い（第 8 節の「シェルの中身を書き写さない」は入門だけ）。新章の抜き出しの規則を AUTHORING に書くかは **Unknown**（設計で決める） |
| R3 接点のまとめ | `setup.md`・`03-face.md`・`08-touch.md`・shell-art の実機ノート | **Exists**（事実）。**Unknown**: R3.6 の「無いサーフェス番号を指したとき」の SSP の挙動（Research Needed） |
| R4 差し替え | 実物の 333×500・`shell_assets_test.rs`・`art/README.md` | **Exists**。**Constraint**: `art/README.md` は `.nar` に入らないので、リポジトリの GitHub URL で指す（`link-check` の実在検査がかかる） |
| R5 表情の追加 | `03-face.md`・`grammar/actor-dictionary.md` | **Missing**: 本文。**Unknown**: サーフェス番号の決まり（既定の 0／10、番号の範囲）を UKADOC のどの項で示すか |
| R6 当たり判定 | 実物の `collisionex`・実機ノート・UKADOC `collisionex*`・`collision-sort` | **Missing**: 本文。**Exists**: 事実と UKADOC の項 |
| R7 バルーン | 実物の 4 行・実機ノート・`crates/pasta_check/README.md`「同梱バルーン」（77〜84 行）・UKADOC のインストール／配布用ファイルの作成 | **Constraint**: マニュアルに `pasta_check` の章が無い（13 段目は SSP の NAR 作成）。読み先を外部 URL にする（未決 10） |
| R8 範囲外の機能 | UKADOC | **Missing**: 1〜2 行の案内 |
| R9 導線 | `setup.md` 節「シェルの中身」・`03-face.md`・`08-touch.md` | **Constraint**: 「もっと詳しく」＝ `使う文法要素`、`C-prose`（1 行 1 文）、`C-sections`（段の章の 5 つの H2）。リンクは本文の箇条書きに置く。`08-touch.md` は `touch-poke-shortcuts` と重なる |
| R10 実物との一致 | `tutorial-check.mjs`（`pasta` だけ）・`shell_assets_test.rs`（形の検査） | **Missing**: `text` の抜き出しの照合。**Constraint**: manual CI は `shell/**` で走らない |
| R11 検査と公開 | 3 つの数・`link-check`・`verify-static`・`verify-search`・mdBook | **Missing**: 数を 60 → 61。生成対象にしない前提なら他は無し |

## 4. 実装の選択肢

### Option A: 既存の章を伸ばす（入門の準備の章か、入門の付録）

- `setup.md` の節「シェルの中身」を厚くする、または `getting-started/` に付録の章を足す。
- ✅ 新しいパートや章の置き場所を考えなくてよい。
- ❌ 第 8 節が「シェルのファイルの中身を書き写さない」「画像を置かない」「台詞以外は 1 行 1 文」を課すので、`surfaces.txt` の行を示す説明が書けない。
- ❌ 付録を `getting-started/` に置くと `C-prose`（16）・`C-order`・入口の章の「章の一覧」の表・`SUMMARY.md` の入門の並びまで動き、入門の段の流れを絵の話で途切れさせない（brief の Approach）という狙いに反する。
- 評価: brief の Approach と第 8 節の両方に逆らう。採る理由は薄い。

### Option B: リファレンスのパートに新章を置き、入門からリンクする（brief の候補）

- `book/src/reference/` の直下に章を 1 つ（例: `reference/shell.md`）。`SUMMARY.md` のリファレンスに 1 行。`setup.md`・`03-face.md`・`08-touch.md` の本文の箇条書きにリンクを 1 行ずつ。3 つの数を 61 に。
- ✅ 第 8 節に縛られず、通常の章の規約（導入・締めだけ台詞）で書ける。抜き出しも示せる。
- ✅ 目次・ビルド・台詞・リンク・検索の検査が自動でかかる。仕組みの追加が要らない。
- ✅ 入門の段の流れを変えない。
- ❌ 抜き出しと実物のずれは、書き手の確認頼み（R10.3 を満たさない）。
- ❌ リファレンスは「仕様の一覧」の色が濃く、手順を含む章として少し毛色が違う（題の付け方で吸収できる）。

### Option C: Option B ＋ 抜き出しの照合（R10.3 の前提を採る場合）

- B に加えて、新章の `text` ブロックの抜き出しが見本のシェルの連続した行と逐語一致することを機械で確かめる。置き場所の候補は 2 つ。
  - C-1: `book/tools/` に照合を足す（`tutorial-check.mjs` を広げるか、小さな新しい道具）。manual CI の `paths` に `crates/pasta_sample_ghost/ghosts/hello-pasta/shell/**` を足さないと、シェルの変更で落ちない（過去に `dic/**` で同じ問題があり、paths に足して解いた前例がある）。
  - C-2: `crates/pasta_sample_ghost/tests/shell_assets_test.rs` に、章を読んで照合するテストを足す。本の写し 2 枚のバイト一致と同じ場所・同じ流儀で、main の CI（`cargo test`）と manual CI（`cargo test -p pasta_sample_ghost`）の両方で走る。
- ✅ シェルが変わったときに章のずれが公開前に落ちる。
- ❌ 道具かテストが 1 つ増える（照合の規則＝どのブロックを照合対象にするかの印の決め方が要る）。
- ❌ brief の Out「シェルを検査する道具」と紛らわしい（あちらは利用者向けの道具で、こちらはマニュアルの整合の検査。区別を要件か設計に一言書く）。

## 5. 規模とリスク

- **規模: S（1〜3 日）** — 文章の章 1 つ、導線 3 行、数の更新 3 か所。Option C でも照合は小さな関数 1 つ。brief の見積もり（5〜8 タスク）と合う。
- **リスク: Low** — 既存の型に乗るだけで、検査の流儀も決まっている。R3.6（無いサーフェス番号の挙動）だけ SSP の実機確認が要り、確かめられないときの扱いを決めておく必要がある（Medium 寄りの 1 点）。

## 6. 設計の段への申し送り

### 推奨と決めること

- 推奨は **Option B**。R10.3（照合）を採るなら **Option C-2**（既存の `shell_assets_test.rs` の流儀に乗り、CI の `paths` を触らずに両方の CI で走る）。
- 設計で決めること:
  1. 章のファイル名と題（`reference/` 直下。H1 と `SUMMARY.md` の題を一致させる）。
  2. 章の H2 の構成（接点のまとめ → 差し替え → 表情 → 当たり判定 → バルーン → 範囲外、など）。
  3. 抜き出しの範囲（どの行を示すか）と、照合を入れる場合の対象の印。
  4. 導線の文言と位置（3 章とも本文の箇条書きか、台詞の後の箇条書きか。`C-prose` に通る形）。
  5. AUTHORING に新章用の一言（抜き出しの規則・第 8 節との違い）を足すか。
  6. `reference/external-links.md` にシェルの UKADOC ページを足すか。
  7. 当たり判定の位置を図で示すか（示すなら画像の置き場所と `shell_assets_test.rs` の番号の列）。

### Research Needed

- **R3.6**: アクター辞書の表情名が、シェルに無いサーフェス番号（例: `\s[9]`）を指したときの SSP の表示（キャラクターが消えるのか、前の絵のままか）。UKADOC に明記が見当たらない。SSP の実機で確かめる（`ssp-ghost-test-launch` の手順）。
- **R5.2**: サーフェス番号の決まりを UKADOC のどの項で示すか（既定のサーフェス、`\s[-1]`、番号の範囲）。
- **R7.4**: 同梱バルーンの読み先の URL（UKADOC の「インストール」の `balloon.directory` の項、「配布用ファイルの作成」、`pasta_check` の README の GitHub URL とアンカー）。
- **R7.2**: 吹き出しの位置の UKADOC の項（シェルの `descript.txt` の `sakura.balloon.offsetx` 等）のアンカー。
- **R6.3・R6.5**: UKADOC の `collisionex*` と `collision-sort` の項のアンカー（重なったときの既定）。
- 0.4.0 の範囲（「入門の完成」）に収まるかの確認: 本 spec はゴーストを完成させるための「入門の次の一歩」で、ゴールの内側にある（roadmap のウェーブ 2026-10-16 に入っている）。

---

# 設計の段の調査と判断（2026-10-16・kiro-spec-design）

## Summary

- **Feature**: `manual-shell-guide`
- **Discovery Scope**: Extension（既存のマニュアルと検査の型に章を 1 つ足す。light discovery）
- **Key Findings**:
  - 章を `book/src/reference/` の直下に置けば、口調・台詞・リンク・静的性・検索の検査が自動でかかる。章の数の決め打ちは 3 つの自己テストと `verify-content.mjs` の冒頭コメント（数字だけ）。`link-check-test.mjs` I-2 の「60 章」は見本のデータで、直さない（ギャップ分析に無かった確認）。
  - 照合テストを `shell_assets_test.rs` に置くと、`build.yml`（パスの絞り込みなし）の `cargo test --all` で、シェルだけを変えたときも落ちる。`manual.yml` は `book/**` の変更で `cargo test -p pasta_sample_ghost` を走らせるので、章だけを変えたときも落ちる。CI の `paths` を触らなくてよい。
  - R3.6（シェルに無い番号を指したときの SSP の画面）は確かめられなかった。SSP の MCP は「Server not available」（SSP が起動していない）。UKADOC の `\s[ID]` の項は「`\s[-1]` で非表示」までで、定義の無い番号の扱いは書かれていない。

## Research Log

### 章の置き場所と検査のかかり方
- **Sources**: `book/tools/verify-content.mjs`（`VOICE_DIRS`・T 系）、`verify-scripts-test.mjs` 78〜84 行、`talk/talk-test.mjs` 453〜454 行、`gen-skill-refs-test.mjs` 592〜606 行、`link-check-test.mjs` 1190〜1230 行
- **Findings**: `VOICE_DIRS` に `reference` があり非再帰。K-10 は全章で `chapterRegions` と旧来の判定の一致を見るため、本体に `---` を置かない形が安全。`link-check-test.mjs` I-2 の 60 は `judgeHeadingIds` に渡す架空のレポートの値。
- **Implications**: 直す数は 3 つ（＋コメント 2 か所）。章は H1／導入／`---`／本体／`---`／締め。

### 照合テストの置き場所と改行
- **Sources**: `crates/pasta_sample_ghost/tests/shell_assets_test.rs`（`book_sample_images_are_byte_identical_to_shell`・`parse_surfaces_txt`）、`.github/workflows/build.yml`・`manual.yml`、`git ls-files --eol`
- **Findings**: シェルの 2 ファイルはリポジトリ内 LF・作業ツリー CRLF。既存のパーサは `str::lines` で読んでいる。`common::workspace_root()` で `book/` を読む前例がある。
- **Implications**: 照合も `str::lines` で行単位に比べる。新しい依存は要らない。

### UKADOC の読み先（ukagaka docs MCP で確認）
- **Findings**:
  - `descript_shell.html` の `sakura.balloon.offsetx` の項に「X座標の＋はどの場合もシェルに近づく向き」「Y座標の＋は常に下方向」とある（R7.2 の向きの正本）。アンカーは `#_1_sakura.balloon.offsetx%2C%E5%BA%A7%E6%A8%99`（`descript_shell_surfaces.html` にも同名の項があり、そちらは `_2_` 付き。シェル全体の値は `_1_` の方）。
  - `descript_shell_surfaces.html` の `collision-sort` の項に「none（IDによらず先に書かれている方が手前）」とある（R6.3）。`collisionex*` の項に形の一覧（rect・ellipse・circle・polygon・region）がある（R6.5。章には写さない）。`element*` の項に「element0 が定義されていると元の surface*.png の内容が破棄されて element0 で置き換えられる」とある（R4.6 の突き合わせの根拠）。
  - `dev_shell.html`（シェルの作成）に「surface○○.png（0以上の数字）」「surface0.png と surface10.png は基本的に必須」「SSP は開発用パレットの『当たり判定を表示』で確認できる」「seriko.use_self_alpha,1 でアルファチャンネルを透明度に使う」とある（R4.4・R5.2・R6.6）。
  - `manual_install.html` に「ゴーストにバルーンなどを同梱する場合」の構成がある（R7.4。ページ内のアンカーは無いのでページへリンク）。`dev_bind.html` が着せ替え・アニメーションの読み先（R8.1）。
  - `list_sakura_script.html` の `\s[ID番号]` の項は、定義の無い番号の挙動を書いていない（R3.6 は実機が要る）。
- **Implications**: 設計書の UKADOC リンク表に URL を固定した。UKADOC のアンカーはリンク検証の対象外なので、実装のときに人が開いて確かめる。

### 実機の事実（hello-pasta-shell-art の Implementation Notes 6.1）
- 重なりは先に書いた定義が勝つ（`Face` が `Head` の手前）。部位名は女の子: 顔の中央 `Face`・頬の横の髪 `Head`・胴 `Body`・靴／首の隙間／上げた手は空。男の子: コック帽 `Head`・顔 `Face`・胴 `Body`・横の髪 `Head`・靴／首の隙間は空。
- 注意: 画面の写しで裏が取れたのは女の子の 4 か所と男の子の帽子で、残りは実装者の書き起こし（`dblclick-log.txt`）が根拠と記録されている。R2.7（実機で確かめた事実だけ）との関係は、設計ディスカッションで確認する（OQ-5）。
- 吹き出しの値は SSP 既定のバルーンの高さ（約 168 px）が前提。反対側に出たときの値（`offsetxr`/`offsetyr`）は置いていない。

### R3.6 の実機確認（2026-10-16・確認済み）
- 設計の生成のときは SSP が起動しておらず確かめられなかった。設計ディスカッションの前に、SSP を起動して確かめた。
- **環境**: SSP/2.9.11（20261010-6; Windows NT 10.0.26300）。起動していたゴーストは「えも2DEBUG」（シェル `ghost_dev/project/emo2/ghost/emo2/shell/master/`）。このシェルにはサーフェス 9・19・7777 の定義も画像も無い（`surfaces*.txt` と `surface*.png` を確かめた）。無い番号を指したときの扱いは SSP 本体の振る舞いでシェルに依らないため、hello-pasta の複製は起動しなかった。pasta は表情名の値の `\s[9]` をそのまま出すので、さくらスクリプトを直接送った結果と同じになる。
- **手順と結果**:
  1. 送る前の画面: 本体側 `surface 1000`、相方側 `surface 2206`（`mcp__ssp__dump_surface`）。
  2. `\0\s[9]\1\s[19]\e` を `strict: true` で送った。送った後も本体側は `1000`、相方側は `2206` のまま。立ち絵は消えず、前の絵が残った。エラーログに `surface not found (9) at position 2`・`surface not found (19) at position 9` が出た。2 つ目の誤りも記録されたので、スクリプトは最初の誤りで止まらず最後まで進んだ。
  3. `\0\s[7777]\e` を `strict` なしで送った（作者のゴーストが普段出すのと同じ送り方）。画面は `1000` のまま。エラーログには何も残らなかった（スクリプトのログには、送ったスクリプトが残る）。
- **Implications**: 章の 2.4 節 (a) に書く事実は次のとおり。「シェルに無い番号を指しても、SSP は前の絵を出したまま台詞を続ける。立ち絵は消えず、ふだんの設定ではエラーも出ない。気づく手がかりは、表情が替わらないことだけ」。`strict` は SSP の MCP・SSTP の検査のための指定で、作者がふだん使うものではないため、章には書かない。

## Design Decisions

### Decision: 章の置き場所と名前（D1）
- **Context**: R1.1（`reference/` の直下）
- **Alternatives**: `reference/shell.md`／`reference/shell/index.md`（下のフォルダは `VOICE_DIRS` が非再帰のため口調・台詞の検査から外れる）
- **Selected**: `book/src/reference/shell.md`。題は「シェルを自分の絵にする」（R1.3 の目的のうち最初の 1 つを題にし、残り 2 つは導入で示す。OQ-3）

### Decision: 照合の置き場所（D2）
- **Alternatives**: (1) `book/tools/` に新しい道具か `tutorial-check.mjs` の拡張（`manual.yml` の `paths` に `shell/**` を足す必要がある）／(2) `shell_assets_test.rs` の Rust テスト
- **Selected**: (2)。既存の「本の写しとシェルの照合」と同じ場所・流儀で、main とマニュアルの両方の CI で走る
- **Trade-offs**: マニュアルの道具の自己テスト群とは別の場所になる

### Decision: 照合の印（D3）
- **Alternatives**: (1) フェンスの前に HTML コメントの印を置き、印の付いたブロックだけ照合／(2) 情報文字列に元のファイル名を書く（mdBook と着色の道具への影響が読めない）／(3) 章のコードブロックはすべて `text` の抜き出しとし、全ブロックを照合
- **Selected**: (3)。印の規約が要らず、照合漏れが起きない。辞書の書き方は表とインラインコードで示し、正本の章へリンクするので、`pasta` ブロックを置く必要が無い（R2.5・R5.4）
- **Trade-offs**: 将来、章に例示のブロックを置きたくなったら照合の規則を変える必要がある（そのときは仕様の写しになっていないかを先に疑う）

### Decision: 数の照合の範囲（D4）
- **Context**: R10.2・R10.3（範囲は設計で決める）
- **Selected**: 座標と吹き出しの値は抜き出しの中にだけ置き、逐語の照合で守る。本文に残る数（333×500・18 枚・0〜8・10〜18・`Face`/`Head`/`Body`）は既存の `shell_assets_test.rs` の定数と検査が守る（シェルが変われば先に落ちる）
- **Rejected**: 章の本文から数を正規表現で拾う照合（壊れやすく、本文の言い回しを縛る）
- **Follow-up**: OQ-2

### Decision: 当たり判定の図（D5）・AUTHORING（D6）・外部リンク集（D7）
- D5: 図を置かない。新しい画像の置き場所と照合の追加が要り、抜き出し E1 と「返る場所」の表で足りる（R2.9）
- D6: `book/AUTHORING.md` は変えない。規則は照合テストの doc コメントと失敗メッセージにある（OQ-4）
- D7: `reference/external-links.md` は変えない。UKADOC のシェルのページは章から直接リンクする

## Synthesis

- **一般化**: R3（接点）・R5（表情）・R6（当たり判定）・R7（吹き出し）は、どれも「辞書の側の名前・番号と、シェルの側の行の対応」という同じ形である。章は接点の表を 1 つ置き、後の節はその表の各行を深掘りする構成にした。
- **Build vs Adopt**: 照合は既存の Rust テストの流儀（`common::workspace_root()`・`str::lines`・panic で失敗）をそのまま使い、新しい道具・依存を足さない。検査ツールの判定ロジックは触らない。
- **Simplification**: 照合の印・AUTHORING の追記・外部リンク集の追記・図を省いた。足す仕組みは照合テスト 1 つだけ。

## Risks & Mitigations

- R3.6 の実機確認 — 2026-10-16 に SSP 2.9.11 で確かめ済み（「R3.6 の実機確認」）。SSP の版が上がって振る舞いが変わったら、章の 2.4 節を見直す
- UKADOC のアンカーの変更 — リンク検証の対象外。実装のときに開いて確かめ、リンク表に URL を固定した
- 入門の `08-touch.md` を `touch-poke-shortcuts` と同時に触る — 後から入る側が rebase で 1 行を保つ（roadmap の取り決め）
- 章を足す別の spec と数が衝突する — 後から入る側が数を合わせる（roadmap の取り決め）

## 参照したスキルと規約

- `kiro-spec-design`（design-principles・discovery-light・synthesis・review-gate）、`kiro-validate-design`（design-review）
- `book/AUTHORING.md` 第 1〜5 節・第 7 節・第 8 節（入門の章へ足すリンクの制約）
- ukagaka docs MCP（UKADOC の項とアンカーの確認）、SSP MCP（接続できず）

## 設計ディスカッションの確定事項（2026-10-16・全議題を推奨案で確定）

- **OQ-1（R3.6）**: SSP 2.9.11 の実機で確かめて解決した（「R3.6 の実機確認」）。設計の検証の判定は、条件付きの GO から GO になった。
- **OQ-2（R10.2・R10.3）**: D4 のとおり。定数 `CANVAS`・`SAKURA`・`KERO` の doc コメントに「変えたら book/src/reference/shell.md の本文も直す」を 1 行ずつ足す。理由: 判定に触れずに、定数を直す人に章のことが伝わる。本文から数を拾う照合は、壊れやすく言い回しを縛る。
- **OQ-3（R1.2・R1.3）**: 題は「シェルを自分の絵にする」。理由: 読者の最初の動機をそのまま題にできる。入門からのリンクは H2 のアンカーを指すので、題を変えても壊れない。
- **OQ-4（D6）**: AUTHORING.md は変えない。理由: 規則は照合テストが強制し、失敗メッセージに直し方が出る。規約とテストで二重に持たない。
- **OQ-5（R2.7・R3.4・R6.3）**: shell-art 6.1 の書き起こし（`dblclick-log.txt`）も実機の記録として認め、部位名をすべて書く（D8）。理由: SSP で実際にダブルクリックした結果を写した記録で、推測ではない。
