# cc-sdd で仕様駆動開発を回す

> 【したり顔】仕様駆動開発の道具「cc-sdd」で、pasta がどう作られているのか。今日は舞台裏をお見せしますわ。

> 【アンソニー】お嬢様、それはゴーストづくりのお役には立たないのではございませんか。

> 【目閉じ】ええ、ゴーストを作るだけなら読まなくてよろしくてよ。けれど、どんな手順で育てられてきたかを知れば、pasta の手触りの理由が少し見えてきますの。

---

## この記事について

雑記は、pasta の開発の舞台裏を書き留める読み物である。仕様の権威ではない。文法・Lua API・`pasta.toml` などの利用者向けの事実は、それぞれの章が正である。

この記事では、pasta の開発で使っている仕様駆動開発（Spec-Driven Development）のツール cc-sdd を紹介し、実際に spec を 1 本流した記録を添える。記述は 2026-11-01 時点のもので、pasta が使っている cc-sdd は v3.1.0 である。

## cc-sdd とは

[cc-sdd](https://github.com/gotalab/cc-sdd) は、AI コーディングエージェントに仕様駆動開発の手順を入れるインストーラとテンプレートの一式である。作者は gotalab（Gota）で、ライセンスは MIT。`npx cc-sdd@latest --lang ja` の 1 コマンドで、エージェント用のスキル・手引き（Claude Code なら `CLAUDE.md`）・文書のテンプレートがプロジェクトに入る。

始まりは、AWS の Kiro IDE の Spec ワークフローを Claude Code で再現する試みだった（2025-07）。[Kiro の Spec](https://kiro.dev/docs/specs/) は、機能ごとに要件（`requirements.md`）・設計（`design.md`）・タスク（`tasks.md`）の 3 文書を順に作り、フェーズの間で人が承認する。要件は EARS（Easy Approach to Requirements Syntax）の型で書く。cc-sdd はこの形と、成果物の置き場所 `.kiro/`（`.kiro/specs/`・`.kiro/steering/`）を引き継いでいる。そのうえで、入口の discovery、既存コードとの差を見る validate 系、複数の spec をまとめて作る spec-batch、サブエージェントで回す自律実装を足した。

### 版の歩み

| 版 | 時期 | 主な変化 |
| -- | ---- | -------- |
| 0.x | 2025-07〜08 | Claude Code 用。3 段の承認、EARS の要件、承認を飛ばす `-y` |
| 1.x | 2025-08〜09 | Cursor・Gemini CLI・Codex CLI などに対応。`npx` のインストーラ。validate 系 |
| 2.x | 2025-11〜2026-02 | `validate-gap`・`validate-design`・`validate-impl`、`research.md`、サブエージェント |
| 3.0 | 2026-04 | スラッシュコマンドから Agent Skills（`.claude/skills/kiro-*/SKILL.md`）へ。入口の `kiro-discovery`、`kiro-spec-batch`、自律実装の `kiro-impl`。スキルは 17 本 |
| 3.1 | 2026-09 | `kiro-impl` のレビューの切り替え（`--review required\|inline\|off`）、Devin（beta） |

2026-11-01 の時点で、最新のリリースは v3.1.0（2026-09-23）である。対応するエージェントは、Claude Code と Codex が stable、Cursor・GitHub Copilot・Devin・OpenCode・Gemini CLI・Antigravity が beta である。

### 上流が示す流れ

| 場面 | 流れ |
| ---- | ---- |
| 新しい機能 | discovery → init → requirements → design → tasks → impl |
| 既存のシステム | steering → discovery か init →（必要なら）validate-gap → design → tasks → impl |
| 大きな取り組み | discovery → spec-batch |
| 小さな変更 | discovery → そのまま実装 |

要件・設計・タスクの各フェーズで人がレビューし、`-y` は意図して早送りするときだけ使う、というのが上流の `CLAUDE.md` の決まりである。

更新用の専用コマンドは無く、同じインストールのコマンドをもう一度走らせる。既存のファイルは、上書き・追記・残すのどれかを選ぶ。

## pasta での使い方

### 置き場所

| 場所 | 役割 |
| ---- | ---- |
| `.kiro/steering/` | プロジェクト全体の前提（製品・技術・構成・ロードマップなど）。AI が毎回の作業で読む |
| `.kiro/specs/` | 機能ごとの spec（brief・要件・設計・タスク）。完了した spec は `.kiro/specs/completed/` へ移す |
| `.claude/skills/` | ワークフローの各段を実行するスキル（`kiro-*`） |

2026-11-01 時点で、完了した spec は 191 本ある。

### 上流のスキルと独自のスキル

`.claude/skills/` の `kiro-*` は 24 本ある。17 本は cc-sdd v3.1.0 が入れたもので、7 本は pasta が足した。

| 区分 | スキル |
| ---- | ------ |
| 上流：入口と文書づくり | `kiro-discovery`・`kiro-spec-init`・`kiro-spec-requirements`・`kiro-spec-design`・`kiro-spec-tasks`・`kiro-spec-batch`・`kiro-spec-quick`・`kiro-spec-status` |
| 上流：検証 | `kiro-validate-gap`・`kiro-validate-design`・`kiro-validate-impl` |
| 上流：実装と品質 | `kiro-impl`・`kiro-review`・`kiro-debug`・`kiro-verify-completion` |
| 上流：steering | `kiro-steering`・`kiro-steering-custom` |
| pasta：段のまとめ役 | `kiro-start`（要件まで）・`kiro-design`（設計）・`kiro-complete`（完了から PR のマージまで） |
| pasta：ディスカッション | `kiro-requirements-discussion`・`kiro-design-discussion` |
| pasta：棚卸と並走 | `kiro-next`（棚卸と次のウェーブの組み立て）・`kiro-wave`（ウェーブの spec を別々のセッションで始める） |

cc-sdd を更新するときは、素のまま上書きしてから、pasta が独自に足した行だけを 3 者マージで戻す。上流が書き換えた上流由来の行は、新しい文面を採る。

## 1 本流してみる

0.4.5 の ponytail 監査の直しのうち、`ponytail-lua-loader`（辞書の読み込みまわりの直しと片付け）を 1 本、最初から流した記録である。各段の見出しの下に、実際に打ったプロンプトをそのまま載せる。

### 入口：discovery

```text
/kiro-discovery 次にやるべき直近のスペックを教えて。今って属性関係のスペックはまだ着手できないのでしたっけ？
```

discovery は、アイデアを出す・イシューを登録する・次に何をすればよいかを知る、のどれにも使う入口のスキルである。上流の discovery は新しい仕事の道を決める（既存の spec を直す・新しい spec を起こす・spec は要らない、など）。pasta では、それに加えて「次は何をするか」を尋ねる窓口にもしている。

このときの discovery は、次の順に動いた。

1. リモートの `main` を取り込む（pasta が足した段。ほかのセッションが入れたものを読み落とさないため）。
2. spec の一覧と `.kiro/steering/roadmap.md` を読む。roadmap.md には、版ごとの範囲・順番・開発者の決定が日付つきでたまっている。
3. 新しい spec は要らないと判断し、次の一手を案内して止まる。答えは「属性の 0.5.0 は 0.4.5 を出した後。今のゴールは 0.4.5 の ponytail 9 本」だった。

続けて「推奨する開始コマンドを一つ出してください」と頼み、次の開始コマンドを受け取った。先頭の行は使うモデルで、後戻りしにくい土台を決める spec にだけ Fable を書く。

```text
Opus
/kiro-start ponytail-lua-loader
```

### 要件まで：kiro-start

```text
/kiro-start ponytail-lua-loader
```

`kiro-start` は pasta が足したまとめ役のスキルで、discovery が書いた `brief.md` から、要件のディスカッションまでを 1 回で進める。中身は次の段である。

| 段 | すること |
| -- | -------- |
| 0 | spec の置き場所・スキルの置き場所・リモート・既定のブランチを、決まった優先順で 1 つに決める |
| 0.5 | リモートの `main` を取り込む |
| 1 | `brief.md` があるか確かめる。無ければ止まる（discovery の後でしか動かない） |
| 2 | 今のブランチが既定のブランチなら止まる。ブランチは Claude Code のワークツリーが用意し、スキルは作らない |
| 3 | 初期化・要件・ギャップ分析を、1 つのサブエージェントにまとめて任せる |
| 4 | できたファイルと `spec.json` の状態を確かめる |
| 5 | spec をコミットする（push はしない） |
| 6 | 要件のディスカッションを、開発者との対話で行う |

段 6 までは、開発者に何も尋ねない。サブエージェントは開発者と話せないので、brief だけでは決めきれないことは最善の案で書いたうえで印（【前提】）を付け、持ち越しの問いとして持ち帰る。持ち帰った問いが、段 6 の議題になる。

段 3 と段 6 で、次のスキルが呼ばれる。上流のスキルが 3 つ、pasta のスキルが 1 つである。

| スキル | 出どころ | 役割 | できるもの |
| ------ | -------- | ---- | ---------- |
| `kiro-spec-init` | 上流 | brief をもとに spec を初期化する | `spec.json`・`requirements.md` のひな形 |
| `kiro-spec-requirements` | 上流 | steering と brief から、EARS の型で要件を書く。最後に自動のレビューの関門を通す | `requirements.md` |
| `kiro-validate-gap` | 上流 | 要件と今のコードの差を調べ、直し方の候補と危険を挙げる | `research.md` |
| `kiro-requirements-discussion` | pasta | 要件とギャップ分析を精査し、開発者と話して要件を仕上げる | `requirements.md` の手直しとコミット |

今回の段 3 は、サブエージェントの申告で約 12 分かかった（初期化 1 分・要件 8 分・ギャップ分析 4 分）。中で調べもののサブエージェントをもう 1 つ並べて走らせている。できたのは、要件 9 つ（受入基準 42、うち【前提】の印が 10）の `requirements.md` と、164 行の `research.md` である。この spec の brief は、ギャップ分析で監査をやり直すと決めていた。やり直した結果、前回の指摘は全部残っていて、新しい指摘が 6 つ増えた。

### ディスカッションの進み方

`kiro-requirements-discussion` は、まず集めた問題を 3 つに分ける。

| 分類 | 中身 | 扱い |
| ---- | ---- | ---- |
| A：自明な修正 | 誤字・EARS の型の崩れ・明らかな食い違い | 開発者に聞かずに直してコミットし、報告だけする |
| B：設計の判断 | どう作るか（how）に関わるもの | 設計の段へ回す。ディスカッションでは話さない |
| C：開発者の確認 | 何を・なぜ（what／why）が開発者の判断に頼るもの | 議題にする |

そのうえで、最初の発言で次のものを一度に出す。

1. 仕様の目的。平易な言葉で言い換え、「今はこうなる → これからはこうなる」の具体例を付ける。
2. A の修正済みの一覧と、B の設計へ回す一覧。
3. C の議題のすべて。どの議題にも、推奨案とその根拠、ほかの選択肢を選んだときにどうなるかを付ける。

今回の議題は 9 つだった。1 つはこういう形で出てくる。

```text
議題 2：.pasta のキャッシュを書けなかったら読み込みエラーにする（要件 1.6）
- 何を決めるか：今の「警告だけで続ける」をやめるか。更新時刻の設定に失敗したときの扱いも
- 具体例：キャッシュのフォルダが読み取り専用だと、今は前の変換結果が黙って動く
- 推奨案：書き込みの失敗は読み込みエラー。時刻の設定の失敗は警告だけ
          （次の起動で必ず変換し直されるので、安全な側に倒れる）
- 他の選択肢：時刻の失敗もエラー → 時刻を設定できないファイルシステムで起動できなくなる
```

開発者の答え方は 2 つある。

- 全部の推奨案でよければ「承認」と答える。全議題が推奨案で確定し、1 回のコミットで `requirements.md` に入る。
- 話したい議題があれば「議題 2 と 4 を議論したい」のように答える。指定しなかった議題は推奨案で確定してコミットし、指定した議題だけを 1 つずつ話す。1 つ決まるごとにコミットし、残りの議題の進み具合をまとめて見せる。

選択式の質問の画面は使わず、普通のチャットで答える。議題をまとめて出すのは、1 つずつ尋ねて往復を重ねるより、全体を見て判断できるからである（pasta の開発者の決定）。どの議題にも具体例を付けるのも、spec 名や要件の番号だけでは何の話か分からないことが多いからである。

すべての議題が決まると、A・B・C の件数をまとめ、次の段のコマンド（`/kiro-design ponytail-lua-loader`）と、設計の段に使うモデルの推奨を示して終わる。

今回は、9 つの議題を読んだ開発者が、一言で答えた。

```text
承認
```

これで 9 つの議題がすべて推奨案のとおりに確定し、1 回のコミットで要件に入った。【前提】の印は 0 になり、議題 4（大文字と小文字だけ違うモジュール名も衝突にする）の分として、受入基準が 2 つ増えた。brief は `pasta_check` を範囲の外にしていたが、ローダーと同じ組み合わせをエラーにするために、`pasta_check` の比べ方も直すことになった。brief の線引きより、根の同じ不具合をその場で直すほうを採る（pasta の開発者の方針）。

最後の報告は A が 1 件、B が 6 件、C が 9 件（すべて推奨案で確定）。設計の段の推奨モデルは Opus だった。設計で決めることにはどれも推奨案が立っていて、後戻りしにくい土台の決定（新しい構文・保存の形式・公開 Lua API の約束）を含まないからである。

（以降、設計の段を追記する）

## 上流から育てた部分

（執筆中）

## 振り返り

（執筆中）

## 参考

- [gotalab/cc-sdd](https://github.com/gotalab/cc-sdd)（リポジトリ・[CHANGELOG](https://github.com/gotalab/cc-sdd/blob/main/CHANGELOG.md)）
- [Kiro の Specs](https://kiro.dev/docs/specs/)・[Introducing Kiro](https://kiro.dev/blog/introducing-kiro/)
- [Kiroの仕様書駆動開発プロセスをClaude Codeで徹底的に再現した](https://zenn.dev/gotalab/articles/3db0621ce3d6d2)（作者の記事、2025-07）
- [仕様駆動開発の理想と現実、そして向き合い方](https://speakerdeck.com/gotalab555/shi-yang-qu-dong-kai-fa-noli-xiang-toxian-shi-sositexiang-kihe-ifang)（作者の登壇、2025-11）

---

> 【にっこり】仕様を先に固めて、AI と人が順に承認していく。地味ですけれど、これが pasta の足腰ですのよ。

> 【アンソニー】お嬢様の足腰も、そのくらい丈夫でございましたら。
