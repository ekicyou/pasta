# 状況: manual-reader-first

マニュアルの改訂の進み具合・決定・未決の議題を残す。セッションが切れても、ここから続けられるようにする。方針は [brief.md](brief.md)。

最終更新: 2026-11-19

## 進め方

- 要件 → 設計 → タスクのフェーズは取らない。対話で決めて、サブエージェントで並行して直す。
- 波ごとに開発者がプレビューで確かめ、議題が閉じてから PR にする（議論の途中で PR を出さない）。
- ブランチ: `claude/kiro-discovery-book-revision-c2de3a`

### プレビュー

- `http://localhost:3939`（`.claude/launch.json` の `pasta-manual`。`book/book/` を配る。`launch.json` はコミットしない）
- 章を直したら、リポジトリのルートで次の 4 つを順に走らせて作り直す（`mdbook serve` では台詞の吹き出しと着色が出ない）。

```sh
mdbook build book
node book/tools/highlight/highlight-html.mjs book/book
node book/tools/talk/talk-html.mjs book/book
node book/tools/bigram-index/build-index.mjs book/book
```

- `book/node_modules` は `npm ci` 済み（このワークツリー）。

### 直した後の検査

```sh
node book/tools/gen-skill-refs.mjs          # 生成元の章を直したら作り直す
node book/tools/verify-content.mjs
node book/tools/link-check.mjs
node book/tools/tutorial-check.mjs          # 入門と辞書を直したとき
node book/tools/verify-static.mjs --no-build
node book/tools/verify-search.mjs --no-build
```

## 波と進み具合

| 波 | 対象 | 状態 |
| -- | ---- | ---- |
| 1 | リファレンス 3 章（`startup`・`pasta-toml`・`runtime-failures`） | 作業中（2026-11-19 着手。分身 3 名） |
| 1 | 内部設計 10 章 | 作業中（2026-11-19 着手。分身 2 名。監査のとおりの軽い手入れ） |
| 1 | デバッグの部（6 章） | 議題 5 の承認待ち |
| 1 | シェルの章の絵 | 議題 8 の承認待ち |
| 2 | 文法の部（10 章） | 議題 4 の承認待ち |
| 3 | 入門ガイド（17 章）・hello-pasta の辞書・絵 16 枚 | 議題 1〜3 の承認待ち |
| — | 規約 `book/AUTHORING.md` と検査の直し | 議題 1 の承認待ち。波ごとに同じ変更で直す |

波 1 の後始末（親がやる）:

- `node book/tools/gen-skill-refs.mjs` でスキルの参照ファイルを作り直す。
- `pasta-toml.md` のテンプレートのコメントから「既定」を消すので、`crates/pasta_lua/tests/loader/config_defaults_test.rs` の `config_reference_doc_matches_ssot` を、テンプレートのフェンスの中の行を見る形に直す（値のずれの検出を残すため）。
- `book/AUTHORING.md` 第 4 節の「検査対象の 6 キー」は実際は 7 キー（`line_newlines` が抜けている）。直す。
- プレビューを作り直す。

## 決定の記録

| 日付 | 決定 | 理由 |
| ---- | ---- | ---- |
| 2026-11-19 | spec のフェーズを取らず、brief と状況ファイルだけで進める | 開発者の指示 |
| 2026-11-19 | 雑記は直さない。Lua の部は、ほかの直しに伴うリンクと語の直しだけ | 開発者の指示（雑記）。Lua の部は指摘に無い |
| 2026-11-19 | リファレンス 3 章と内部設計は、議題を待たずに着手する | 指示がはっきりしていて、ほかの議題の答えで変わらない |

## 未決の議題（2026-11-19 に提示）

推奨案は各議題の先頭。開発者の答えが出たら「決定の記録」へ移す。

1. **規約を読む人の側へ改める** — 推奨: 改める。入門の「台詞に書いた事実を、台詞以外にも必ず書く」をやめ、事実は 1 回だけ書く。全章に「今の動きだけを書く・Windows 11 で自明なことを書かない・抽象の規則には実物の例を 1 つ」を足す。章の並びを縛る検査は新しい構成に合わせて直す。
2. **入門の構成** — 推奨: 「まず会ってみる（hello-pasta を入れて、1 行書き換えて ▶ で鳴らす）」を先頭に足し、前提環境と準備の解説は付録へ下げる。自分のゴーストの器は、入門用の nar（スターター）を SSP に入れて始める。確かめ方は「SSP の再起動」から、エディタの「▶ シーンを実行」に替える。
3. **見本ゴーストの台詞** — 推奨: 「パスタ屋の見習い 2 人」の設定（README にあるが台詞に出ていない）を笑いの足場にして、13 枚を書き直す。書く前に emo2 開発のセッションへ掛け合いのコツを相談する。絵 16 枚は撮り直す。
4. **文法の部** — 推奨: 概要を「Pasta DSL は台本である」の全体像（通しの作例 1 本と、柱・ト書き・台詞の対応の表）に書き直し、目次を 概要 → マーカー → 行とブロック（柱）→ アクション行（台詞）→ アクター辞書（登場人物）→ さくらスクリプト → 単語 → 変数 → リテラル → Call の順に並べ替える。語「台本」は辞書を指す語にし、今「台本」と呼んでいる出力は「スクリプト」に言い換える（エディタの出力の文言 `台本:` も同じ変更で直す）。
5. **デバッグの部** — 推奨: 部の題を「書いて試す」に、章を「pasta-recipe で書く」「VS Code で書く」に改める（pasta-recipe を先に）。WebView2・ポート番号・前の版からの移行・設定の置き場のパスは消す。本体にも Claudia を出し、入門と同じ気軽さで書く。ファイル名は変えない（URL を保つ）。
6. **リファレンス 3 章** — 着手済み。プレビューで確かめてもらう。
7. **内部設計** — 着手済み（監査のとおり。構成は変えない）。
8. **シェルの章の絵** — 推奨: 表情の一覧に顔の切り抜き（2 人 × 9 表情）を並べ、当たり判定の枠を重ねた図を 1 枚足す。

## 調査の要点（2026-11-19。サブエージェント 4 名）

### 入門ガイド

- 最初の台詞が出るまでに、入口・前提環境・最小一式の 3 章（約 400 行）を読む。nar を入れる手順は、最小一式の章の 6 番目の節に埋まっている。
- `hello-pasta.nar` は 13 段ぶんの辞書を全部含む。段ごとの形は無い。ネットワーク更新に対応していて、手で入れた変更は更新で上書きされる。
- 入門に、VS Code の拡張と pasta-recipe は出てこない（VS Code は「エディタの例」の 1 か所だけ）。確かめ方は「SSP を終了して、もう一度起動する」に統一されている。エディタの ▶ には `pasta.toml` の `[debug] enabled = true` が要るが、配布版には無い。
- 台詞は「害のない見本文」。いちばん乾いているのは 1 段（定型の挨拶 1 行）・5 段・7 段・10 段。README の設定（給仕見習いと見習い料理人）を使った台詞は 0 本。
- 固い制約（道具か規約を同じ PR で直さないと CI が落ちる）:
  - 目次の並び `index → prerequisites → setup → NN-*`（`verify-content.mjs` の C-order）
  - 段の章の H2 5 種の文言と順・800 字以上（C-sections・C-steps）
  - 地の文は「1 行 1 文の指示」だけ。生 HTML の行は不可（C-prose）
  - 入門フォルダの中に `UTF-8`・`Shift_JIS`・`Windows`・`SSP` の語（`Shift_JIS` は `prerequisites.md` にしか無い）
  - 辞書 `X.pasta` と章 `X.md` の名前と逐語の一致（`tutorial-check.mjs`）
  - 章の数の決め打ち 14・17（`verify-scripts-test.mjs`）
  - `STAGES.md` は 1〜14 段（`tutorial_stages_test.rs`）
  - 台詞を持つテスト: `pasta_sample_ghost/tests/touch_shortcuts_test.rs`（8・9 段）、`pasta_shiori/tests/byte_invariant_test.rs`（1 段の OnBoot の応答の全文）
  - pasta-recipe の説明動画は `10-save.pasta` の台詞に依存する
  - 新しい配布物へのリンクは `release.yml` の添付の行と照合される（`link-check.mjs` の `release-asset`）
- 絵 16 枚は実機の SSP で手で撮る（`book/SCREENSHOTS.md`）。写っている台詞を変えた絵は撮り直す。

### 文法の部

- マニュアルの本文に、台本・脚本・柱・ト書きの説明は 1 行も無い。プロジェクトの中では roadmap と `cue-command-runtime` の brief が「柱＝`＊` と `＆属性`、ト書き＝`！` 行、台詞＝アクション行」と言っている。
- 通しの作例（アクター辞書＋シーン＋`％` 行＋表情の単語）が最初に出るのは、最後の章 `actor-dictionary.md`。概要の唯一の作例は算術式。
- `？` 単独のジャンプは無い。章題「Call / Jump」の Jump は本文に無い。
- `！` 行で今働くのは `!select` だけ。
- 語「台本」は、デバッグと Lua の章で「ゴーストが返したさくらスクリプト」の意味に使われている（5 章・25 か所）。エディタの出力にも `台本:` がある（`editors/shared/`）。
- スキルは目次の順を見ない（`SKILL.md` が自前のマーカー表を持つ）。目次の並べ替えはスキルに響かない。目次の順を縛る検査は無い。
- `block-structure.md` の半分（314 行）は属性（候補の選び方）。`call-jump.md` の後半 350 行は特殊な呼び出し。

### リファレンス 3 章

- 実物のログが手元にある（リポジトリの外。`fatal=true` の行・パースエラーの行）。ログ 1 行の形は `<UTC 時刻>Z <レベル> <target>: <メッセージ> <key=value …>`。
- `startup.md` の入口の行の例は target が古い（`pasta::windows` → 今は `pasta::load_entry`）。パースエラーの形も実物と違う。
- `pasta-toml.md` の例「表示速度を速くする `script_wait_normal = 30`」は、既定の 50 と同じ出力になる（効かない例）。
- `config_reference_doc_matches_ssot` が見るのは 7 キーで、テンプレートは「既定」を含む行だけを見る。

### 内部設計

- 自明な説明は少ない（全体で 15 か所ほど）。目が滑る原因は、実装の逐語の書き起こし・章をまたぐ重複（15 件）・入力 → 出力の例の欠落・経緯の語り。
- 手入れの量は約 490 行（全体の 12%）。重い順に `talk-output`・`shiori`・`registry-search`・`internal-modules`。書き直しが要る章は無い。
