---
name: kiro-spec-complete
description: 'Kiro仕様駆動開発のSpec完了ワークフローを実行する。DoDゲート検証→コミット→completedフォルダ移動→spec.json更新→参照パス修正→ロードマップ更新→スキルドキュメント同期→最終コミット→pushまでを中断なく完遂する。Use when: 実装完了を承認する, 承認してください, 完了を承認, spec承認, approve implementation, kiro承認完了。DO NOT USE when: 実装が完了したのみ（承認の明示がない場合）、タスクが終わっただけ'
argument-hint: <feature-name>
---

# Kiro Spec 完了ワークフロー

## 発動条件（必須）

> **⚠️ このスキルは開発者の明示的「承認」がある場合にのみ発動する。**

### ✅ 発動する（承認の明示がある）
- 「実装完了を**承認**します」「**承認**してください」
- 「このspecを**承認**する」「**approve**」「kiro **承認**完了」

### ❌ 発動しない（承認の明示がない）
- 「実装が完了した」「タスクが全部終わった」
- 「spec完了」「アーカイブしてほしい」などの曖昧な表現のみ
- AIが自律的に「完了したと判断」した場合

### 承認が不明瞭なとき
発動を迷う場合は **発動しない**。必要なら開発者に確認する:
> 「実装完了の承認をいただけますか？承認いただいた場合、完了ワークフローを実行します。」

---

## いつ使うか
- 開発者が上記「発動する」に該当する承認を**明示的に**宣言したとき
- tasks.md の全タスクが `[x]` 完了している状態で使用
- 設計文書リフレッシュが完了した後

## 権威的ソース

> **このスキルは `.kiro/steering/workflow.md` に従う。**
> 完了基準（DoD）、コミット規約、ドキュメント更新チェックリスト、スキルドキュメント更新判定はすべて workflow.md を参照すること。
> このスキルは workflow.md の「実装完了時アクション」を**自動オーケストレーション**するものであり、ルールの複製は行わない。

## 哲学
- **中断せず一連で完遂する** — 全ステップを止めずに実行
- **VSCodeの変更ファイル確定挙動を回避** — spec.json編集は移動後に行う
- **ステアリングが正** — DoD・コミット規約・禁止事項はすべて workflow.md に従う
- **繰り返し仕様は移動しない** — `release-workflow` 等は常に `.kiro/specs/` 直下に留まる

## 前提条件
- `.kiro/specs/{feature}/tasks.md` の全タスクが完了
- 設計文書と最終実装の整合確認済み

## 例外: 繰り返し仕様

`release-workflow` のような繰り返し実行型仕様は `completed/` に**移動しない**。

判定基準:
- spec.json や requirements.md に「繰り返し」「repeatable」「定期実行」等の記述がある
- `/kiro-spec-impl` のたびにタスクがリセットされる設計

繰り返し仕様の場合:
1. ステップ1（DoD検証）とステップ2（コミット）のみ実行
2. ステップ3〜5（移動・パス更新・ロードマップ）をスキップ
3. ステップ6（リモート同期）を実行
4. tasks.md のチェックボックスをリセット（全 `[x]` → `[ ]`）

---

## 手順

### ステップ1: DoD（完了基準）ゲート検証

`.kiro/steering/workflow.md` の「完了基準（DoD）」セクションを読み込み、5ゲートをすべて検証する。

1. **workflow.md を読み込む**（未読の場合）
2. **5ゲート（Spec / Test / Doc / Steering / Soul）** を順に検証
3. **Test Gate**:
   - まず `session_store_sql` でセッション記録を確認し、直近のターンで `cargo test` が実行され全テスト成功していたか判定する
     ```sql
     SELECT t.content FROM turns t
     JOIN sessions s ON t.session_id = s.id
     WHERE s.id = (SELECT id FROM sessions ORDER BY start_time DESC LIMIT 1)
       AND t.content LIKE '%cargo test%'
       AND t.content LIKE '%test result%'
     ORDER BY t.created_at DESC LIMIT 3
     ```
   - **スキップ可**: セッション記録に `test result: ok` が確認でき、その後にテスト対象コードの変更がない場合、Test Gate をスキップする。スキップ時は完了チェックリストに「(セッション記録により省略)」と注記する
   - **スキップ不可**: セッション記録が見つからない、テスト結果が不明瞭、またはテスト後にコード変更がある場合は実行する:
   ```powershell
   cargo test --all 2>&1 | Select-String "test result:|FAILED|error\["
   ```
4. **いずれかのゲートが失敗した場合**: ワークフローを中断し、開発者に報告

### ステップ2: 未コミットファイルのコミット

実装中の変更をすべてコミットする。コミットメッセージ形式は workflow.md の規約に従う。

```powershell
git add -A
git commit -m "<type>({feature-name}): 実装完了

- 変更の要約（箇条書き）"
```

### ステップ3: completedフォルダへの移動

specディレクトリをcompleted配下へ移動する。

```powershell
New-Item -ItemType Directory -Path ".kiro/specs/completed" -Force | Out-Null
Move-Item ".kiro/specs/{feature-name}" ".kiro/specs/completed/"
```

**重要**: この時点ではspec.jsonを**編集しない**。VSCodeが編集中のファイルを追跡しており、移動前に編集すると移動操作と競合してファイルが元の場所に復活する。

### ステップ4: spec.jsonのステータス更新

**移動完了後に** spec.json を更新する。以下のフィールドを変更:

```json
{
  "phase": "completed",
  "completed_at": "YYYY-MM-DDTHH:MM:SSZ"
}
```

> **注意**: `"status"` フィールドは使用しない。`"phase": "completed"` のみで完了を表す。

### ステップ5: 参照パスの更新

他のspecファイルや親仕様がこのspecを参照している場合、パスを更新する。

1. **参照箇所の検索**:
```powershell
Get-ChildItem ".kiro/specs" -Filter "*.md" -Recurse |
  Where-Object { $_.FullName -notlike "*completed*" } |
  Select-String -Pattern "{feature-name}" |
  Select-Object -ExpandProperty Path | Sort-Object -Unique
```

2. **パスの一括置換**: `.kiro/specs/{feature-name}/` → `.kiro/specs/completed/{feature-name}/`

3. **親仕様への完了マーク**: 親仕様のdesign.mdに完了ステータス（✅）を反映する（該当する場合）

### ステップ6: 追加更新チェック

workflow.md の「実装完了時アクション」セクションおよび「ドキュメント保守」セクションに従い、以下を実施する。

#### 6-1. ロードマップ更新

`.kiro/steering/roadmap.md` を確認し、完了したSpecが「Specs (dependency order)」に記載されているか判定する。

**スコープ判定（優先順位）**:
1. `requirements.md` に明示的なロードマップ項目との紐付け記述がある場合
2. 開発者が直接指示した場合
3. ロードマップの Specs 一覧にこの feature-name が含まれる場合
4. 判断に迷う場合は開発者に確認

**スコープ内の場合**:
- 対応する `- [ ] {feature-name}` を `- [x] {feature-name}` に更新

**スコープ外の場合**: スキップ

#### 6-2. スキルドキュメント更新

workflow.md の「スキルドキュメント更新検討」セクションに従い、変更領域に応じた対象スキル（`pasta-ghost-authoring` / `pasta-lua-coding`）との整合性を確認・更新する。スキップ条件も workflow.md に準拠。

#### 6-3. ステアリング・ドキュメント更新

workflow.md の「ドキュメント保守 > 更新チェックリスト」に従い、該当するドキュメントを更新する。

### ステップ7: 完了最終コミット

移動・ステータス更新・参照パス修正・追加更新をコミットする。

```powershell
git add -A
git commit -m "chore({feature-name}): spec完了・アーカイブ"
```

### ステップ8: リモート同期

確認不要。以下のコマンドを直接実行する。

```powershell
git push origin main
```

---

## 完了チェックリスト

```
- [ ] DoD 5ゲート通過（Spec/Test/Doc/Steering/Soul）
- [ ] cargo test --all 成功（またはセッション記録により省略）
- [ ] 未コミットファイルをコミット済み（ステップ2）
- [ ] completedフォルダへ移動済み（ステップ3）※繰り返し仕様はスキップ
- [ ] spec.json の phase を "completed" に更新済み（ステップ4）※繰り返し仕様はスキップ
- [ ] 参照パス更新済み（ステップ5）※繰り返し仕様はスキップ
- [ ] ロードマップ更新済み（スコープ内の場合）
- [ ] スキルドキュメント同期済み（該当する場合）
- [ ] 完了コミット済み（ステップ7）
- [ ] リモートにプッシュ済み（ステップ8）
```

---

## エラー回避

### VSCode変更確定問題
- **症状**: 移動したファイルが元の場所に復活する
- **対策**: spec.jsonは必ずステップ4（移動後）で編集。移動前に編集しない

### 参照パス更新漏れ
- **症状**: 後続specが旧パスで参照しファイルが見つからない
- **対策**: ステップ5で `Select-String` による網羅的検索を実施

### コミット漏れ
- **症状**: pushしたが変更が反映されていない
- **対策**: 各コミット前に `git status --short` で確認

### テスト失敗時
- **症状**: `cargo test --all` が失敗
- **対策**: ワークフローを中断し開発者に報告。テスト修正後に再実行
