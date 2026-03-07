# 継続情報: dola-cue-pasta-dsl-extension 要件レビュー

> 作成日: 2026-03-02
> 前提: requirements.md v2, gap-analysis.md v2 を基に要件レビューを実施中

---

## 完了済みアクション

### 自明な修正（コミット済み）

| 内容 | コミット |
|------|----------|
| BarrierKind 名称を実コードに整合（All/Any/Explicit → WaitForInput/WaitForChoice/Timeout） | `d04a934` |

### ディスカッション済み議題

| # | 議題 | 結論 | コミット |
|---|------|------|----------|
| Q1 | 暗黙キーフレームの「所要時間」 | **外部注入アプローチ**: Duration Resolver トレイトを定義し CueSheet ビルダーに注入。パーサーは行の出現順序と構造のみ出力。dola 内で所要時間は確定しない。Req 2 AC 5-6 追加、gap-analysis R-1 解決済み | Q1コミット |
| Q2 | 未定義 `@command` の Emote フォールバック根拠 | **最頻出用途が Emote だから**。Req 4 設計注記に根拠追記 | `5446200` |

### 設計判断（design.md で詳細化する事項 — 変更不要）

| ID | 項目 |
|----|------|
| D-1 | `!` コマンド行の具体的 PEG 文法 (gap-analysis R-4) |
| D-2 | `@alias = Command(args)` の PEG 文法 (gap-analysis R-2) |
| D-3 | 実装アプローチ選択 A/B/C (gap-analysis R-5) |
| D-4 | CueCommand 記法の EN/JA 対応表 |
| D-5 | MVP フェーズ分割計画 |

---

## 未完了ディスカッション議題（ここから再開）

### Q3: 継続行（`:content`）の CueCommand::Text 挙動（Req 5.4, gap-analysis R-10）✅

**結論**: **A — 同一 `Cue` の `Text` に `\n` 結合**

- 継続行は `\n` を区切りとして直前 Cue の Text に結合する
- 継続行内の `@command` は不許可（パースエラー）
- タイムライン上は前行の暗黙キーフレームに続くルールを適用（B でも同様のため A を選択）
- タイプライター側で改行処理が必要なのは A / B 共通のため、シンプルな A を採用

---

### Q4: `%` 行不在時のデフォルトスロット（Req 6.6, gap-analysis R-7）✅

**結論**: **未割り当てアクターがいた場合のみ、空きスロットの 0 番から順に割り当て。スロット割り当ては最後の状態を継続**

- スロット割り当ては**セッションをまたいで永続**する（最終シーンの配置を継続）
- `%` 行が存在する場合はその指定を使用（明示優先）
- `%` 行がなく未割り当てのアクターが出現した場合、現在未使用の最小スロット番号（0 から）を割り当てる
- 既に割り当て済みのアクターは再割り当てしない
- **ランタイム API 要件**: DSL 実行層は「現在のスロット割り当て状態を取得する」メソッドを提供する必要がある（例: `get_slot_assignment(actor) -> Option<SlotId>`）

---

### Q5: `CueCommand::Clear` 生成ポリシー（Req 5.5, gap-analysis R-9）✅

**結論**: **A — `!clear` 明示コマンドのみ**

- DSL としては `!clear` を明示的に記述した場合のみ `CueCommand::Clear` を生成する
- シーン遷移時の自動 Clear はアプリ（wintf 等）の責務とし、DSL・dola は関知しない
- 責務分離: DSL = 宣言、クリアタイミング制御 = アプリ層

---

### Q6: `RouteRemove` 発行条件（gap-analysis R-8）✅

**結論**: **明示 `!` コマンドのみ。シーン終了時の自動生成なし**

- スロット割り当ては永続（Q4）のため、シーン終了時に RouteRemove を自動挿入しない
- `RouteAdd` は「未割り当てアクターが初出現した場合のみ」生成される
- `RouteRemove` はスクリプト作者が明示的に `!route_remove` と書いた場合のみ生成される
- Q5（Clear）と同じ「DSL は宣言のみ、ライフサイクルはアプリ層」の原則に従う
- **ランタイム API 要件**: Q4 の `get_slot_assignment()` により RouteAdd 生成要否の判定が可能

---

### Q7: 1行内の複数 `@command` 処理（Req 5.6, gap-analysis R-6）✅

**結論**: **A — 全て適用（出現順に CueCommand を生成）**

- `さくら：＠笑顔　ふふーんいいでしょ。＠驚き　あ、ちょっとそれは持っていかないで！` のように、セリフの途中で表情が変化するケースがあるため
- `@command` は出現順に Cue として生成される（テキスト断片と交互に並ぶイメージ）
- 同一アクター・同一行内で `Text → Emote → Text → Emote` の順に複数 Cue が生成される
- 各 Cue の start_time 計算は Duration Resolver の責務

---

## 全議題終了後の次ステップ

### ✅ 完了: requirements.md v3 全面更新（2026-03-03）

Q1〜Q7 の全決定事項を requirements.md に反映済み。主な変更点:
- 要件 3: Clear の明示のみ生成（AC 11 追加）
- 要件 5: 継続行 `\n` 結合 + `@command` 不許可（AC 4-6 更新）
- 要件 6: スロット永続 + RouteRemove 明示のみ + `get_slot_assignment()` API（AC 6-8 更新）
- 要件 9: Duration Resolver / get_slot_assignment API 仕様の design.md 記載要件追加（AC 7-9 追加）

### 次アクション

```
/kiro-spec-design dola-cue-pasta-dsl-extension
```

design.md v2 書き直し（旧 `\cue_*` + `[timestamp]` 形式を完全廃棄）では以下を含める:
- `!` コマンド行の PEG/EBNF 文法（キーフレーム宣言・指定・Barrier・Clear・route_remove）
- エイリアス定義行の PEG 文法（`@alias = Command(args)` の `=` セパレータ）
- Duration Resolver トレイトの型定義
- CueCommand 記法対応表（EN/JA）
- `get_slot_assignment()` API 仕様
- RouteAdd/Switch/Remove 判定ロジック
- 実装フェーズ計画（MVP → Full）
- `cue.pasta` の v2 全面書き換え
