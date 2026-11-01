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

（執筆中。spec を 1 本流しながら追記する）

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
