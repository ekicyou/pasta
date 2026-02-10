# Gap Analysis: release-workflow-dll-zip-fix

## 1. 現状調査

### 対象アセット

| アセット | 現状 | 本仕様での役割 |
|----------|------|----------------|
| `.kiro/specs/release-workflow/requirements.md` | ✅ 存在（approved） | Req 4, Req 6 の修正対象 |
| `.kiro/specs/release-workflow/design.md` | ✅ 存在（approved） | Phase 4, Phase 6 の修正対象 |
| `.kiro/specs/release-workflow/tasks.md` | ✅ 存在（approved） | タスク7, タスク10 の修正対象 |
| `target/i686-pc-windows-msvc/release/pasta.dll` | ✅ 存在（2.7MB） | zip 圧縮元ファイル |
| GitHub Release v0.1.5 | ✅ 公開済み | アセット差し替え対象 |
| `gh` CLI | ✅ 認証済み（ekicyou） | delete-asset / upload 実行 |
| PowerShell `Compress-Archive` | ✅ 利用可能 | zip 圧縮に使用 |

### v0.1.5 リリースの現在のアセット状況

| アセット名 | サイズ | content-type |
|-----------|--------|--------------|
| `hello-pasta.nar` | 1,363,761 bytes | application/octet-stream |
| `pasta-vscode-0.1.5.vsix` | 384,442 bytes | application/vsix |
| `pasta.dll` | 2,739,200 bytes | application/x-msdownload |

**発見**: v0.1.5 には VSIX ファイルも含まれている。要件の Req 3.5 では「`pasta.dll.zip` と `hello-pasta.nar` の2つ」とあるが、実際には VSIX も含まれるため、確認対象を修正すべき。

### 既存仕様の関連箇所

**requirements.md** で変更が必要な箇所:
- **Req 4** (AC 4.3): `pasta.dll` の存在確認 → zip 圧縮ステップの追加
- **Req 6** (AC 6.7): アセット添付リストに `pasta.dll` → `pasta.dll.zip`

**tasks.md** で変更が必要な箇所:
- **タスク7** (Phase 4): `pasta.dll` の存在確認後に zip 圧縮を追加
- **タスク10** (Phase 6): `$assets` 配列内の `pasta.dll` → `pasta.dll.zip`

**design.md** で変更が必要な箇所:
- Phase 4 フロー内に zip 圧縮ステップを追加
- Components and Interfaces テーブルの Phase 4 記述更新
- Requirements Traceability の Req 4.3 記述更新

---

## 2. 要件実現可能性分析

### Requirement 1: zip 圧縮工程の追加

| 技術要素 | 状況 | 対応 |
|----------|------|------|
| `Compress-Archive` コマンドレット | ✅ PowerShell 5.1+ に標準搭載 | 追加インストール不要 |
| zip 出力パス | ❓ 未定義 | `target/i686-pc-windows-msvc/release/pasta.dll.zip` を推奨 |
| zip 圧縮の冪等性 | ⚠️ 既存ファイルがあると上書きエラー | `-Force` フラグで対応可能 |

**PowerShell コマンド例**:
```powershell
Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" `
  -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" `
  -Force
```

### Requirement 2: 仕様ドキュメントの更新

| 対象ファイル | 変更量 | 複雑度 |
|-------------|--------|--------|
| `requirements.md` | 小（AC 2行追加 + 1行変更） | Low |
| `tasks.md` | 小（タスク7に3行追加 + タスク10で1行変更） | Low |
| `design.md` | 中（フロー図、トレーサビリティ表、コンポーネント表の更新） | Low-Medium |

### Requirement 3: v0.1.5 アセット差し替え

| 操作 | コマンド | リスク |
|------|---------|--------|
| アセット削除 | `gh release delete-asset v0.1.5 pasta.dll -y` | Low（-y で確認スキップ） |
| zip 圧縮 | `Compress-Archive` | Low |
| アップロード | `gh release upload v0.1.5 pasta.dll.zip` | Low |

**制約**: `pasta.dll` が `target/i686-pc-windows-msvc/release/` に存在する必要がある → **確認済み（存在する）**

---

## 3. 実装アプローチの選択肢

### Option A: 既存コンポーネントの拡張（推奨 ✅）

**理由**: 本仕様は既存の `release-workflow` 仕様ドキュメント群にピンポイントで修正を加えるものであり、新しいコンポーネントの作成は不要。

**変更ファイル一覧**:
1. `release-workflow/requirements.md` — Req 4 に AC 追加、Req 6 の AC 6.7 変更
2. `release-workflow/tasks.md` — タスク7 に zip 圧縮ステップ追加、タスク10 のアセットリスト変更
3. `release-workflow/design.md` — フロー図・テーブルの更新

**v0.1.5 差し替え**: `gh` CLI コマンドを3つ実行するだけ（delete-asset → compress → upload）

**Trade-offs**:
- ✅ 最小限の変更で完結
- ✅ 既存の仕様構造を維持
- ✅ 全操作が確立されたツールで実行可能
- ❌ 特になし（変更量が少ないため）

### Option B: 新規コンポーネント作成

**不採用理由**: zip 圧縮は PowerShell のワンライナーで完結するため、スクリプトやモジュールの新規作成は過剰。

### Option C: ハイブリッド

**不採用理由**: 変更量が小さく、段階的実装の必要性がない。

---

## 4. 実装複雑度とリスク

### 工数: **S（1日以内）**
- 仕様ドキュメントの修正は限定的（3ファイル、各数行の変更）
- v0.1.5 差し替えは `gh` CLI コマンド3つで完結
- 全ツールが確認済み・利用可能

### リスク: **Low**
- 使用するツール（`Compress-Archive`, `gh release delete-asset`, `gh release upload`）はすべて標準的
- 既存の DLL ビルド成果物は存在確認済み
- ロールバック: v0.1.5 のアセット削除後に失敗した場合、ローカルの `pasta.dll` を手動でアップロード可能

---

## 5. 要件へのフィードバック

### 修正提案

1. **Req 3, AC 5**: 「`pasta.dll.zip` と `hello-pasta.nar` の2つのアセット」→ 実際には VSIX も含まれるため、「少なくとも `pasta.dll.zip` と `hello-pasta.nar` が存在する」に修正すべき
2. **Req 1, AC 1**: zip 出力先パスを明示的に定義すべき（`target/i686-pc-windows-msvc/release/pasta.dll.zip` を推奨）
3. **Req 1, AC 5**: `Compress-Archive` に `-Force` フラグを指定し、既存ファイルの上書きを保証すべき

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（既存コンポーネントの拡張）

### 主要な決定事項
1. zip 出力先パス: `target/i686-pc-windows-msvc/release/pasta.dll.zip`（DLL と同じディレクトリ）
2. zip 圧縮タイミング: Phase 4（ゴーストビルド）内、DLL 存在確認の直後
3. 冪等性確保: `Compress-Archive -Force` を使用

### Research Needed
- なし（全ツール確認済み）
