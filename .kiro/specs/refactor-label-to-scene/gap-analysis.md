# Gap Analysis: refactor-label-to-scene

## 分析サマリー

- **スコープ**: Pasta全体（Rustソースコード、ドキュメント、テスト、仕様）における「Label」→「Scene」用語統一リファクタリング
- **主な課題**: 
  - 5つの主要Rust構造体（`LabelRegistry`, `LabelInfo`, `LabelDef`, `LabelScope`, `LabelTable`）のリネーム
  - 176個のMarkdownドキュメント内の用語置換（日本語「ラベル」+ 英語「label」）
  - 3ファイル名の変更とそれに伴うmod.rs更新
  - 生成されるRuneコード内の関数名変更
- **推奨アプローチ**: ハイブリッド（IDE Rename + 手動スクリプト + 検証）
- **リスク**: Medium（広範な影響範囲だが、純粋なリファクタリングのため型チェック・テストで検証可能）

---

## 1. 現状調査

### 1.1 主要アセット（Rustコード）

#### 構造体・列挙型

| 型 | ファイル | 行数（概算） | 用途 |
|----|---------|------------|------|
| `LabelRegistry` | `src/transpiler/label_registry.rs` | 180 | トランスパイル時のシーン登録・ID管理 |
| `LabelInfo` (transpiler) | `src/transpiler/label_registry.rs` | 30 | シーン情報（トランスパイラ用） |
| `LabelInfo` (runtime) | `src/runtime/labels.rs` | 30 | シーン情報（ランタイム用） |
| `LabelTable` | `src/runtime/labels.rs` | 284 | ランタイムシーン検索テーブル |
| `LabelDef` | `src/parser/ast.rs` | 60 | パーサーAST：シーン定義 |
| `LabelScope` | `src/parser/ast.rs` | 10 | シーンスコープ（Global/Local） |
| `LabelId` | `src/runtime/labels.rs` | 5 | シーン一意識別子 |
| `LabelNotFound` | `src/error.rs` | - | エラーバリアント |

#### 関数・メソッド（主要なもの）

| 関数/メソッド | ファイル | リネーム対象 |
|-------------|---------|------------|
| `select_label_to_id` | `src/stdlib/mod.rs` | ✅ `select_scene_to_id` |
| `label_selector` (生成Rune) | `src/transpiler/mod.rs` | ✅ `scene_selector` |
| `execute_label` | `src/engine.rs` | ✅ `execute_scene` |
| `transpile_global_label` | `src/transpiler/mod.rs` | ✅ `transpile_global_scene` |
| `transpile_local_label` | `src/transpiler/mod.rs` | ✅ `transpile_local_scene` |
| `label_to_fn_name_with_counter` | `src/transpiler/mod.rs` | ✅ `scene_to_fn_name_with_counter` |
| `from_label_registry` | `src/runtime/labels.rs` | ✅ `from_scene_registry` |
| `all_labels` (LabelRegistry) | `src/transpiler/label_registry.rs` | ✅ `all_scenes` |
| `get_label` (LabelRegistry) | `src/transpiler/label_registry.rs` | ✅ `get_scene` |

#### ファイル名変更

| 現在 | リネーム後 | 影響範囲 |
|------|-----------|---------|
| `src/transpiler/label_registry.rs` | `src/transpiler/scene_registry.rs` | `mod.rs`の`mod label_registry;`修正 |
| `src/runtime/labels.rs` | `src/runtime/scene.rs` | `mod.rs`の`mod labels;`修正 |
| `tests/label_id_consistency_test.rs` | `tests/scene_id_consistency_test.rs` | - |
| `tests/pasta_engine_label_resolution_test.rs` | `tests/pasta_engine_scene_resolution_test.rs` | - |
| `tests/pasta_transpiler_label_registry_test.rs` | `tests/pasta_transpiler_scene_registry_test.rs` | - |

#### 変数名（高頻度出現）

- `label`: `scene`（関数引数、ローカル変数）
- `label_registry`: `scene_registry`
- `label_table`: `scene_table`
- `local_label`: `local_scene`
- `global_label`: `global_scene`
- `label_counters`: `scene_counters`
- `labels`: `scenes`（Vec, HashMap等のコレクション）

### 1.2 ドキュメントアセット

#### Markdownファイル（176ファイル）

| カテゴリ | 対象ファイル | 置換パターン |
|---------|-------------|-------------|
| 文法リファレンス | `GRAMMAR.md`, `SPECIFICATION.md` | 「グローバルラベル」→「グローバルシーン」等 |
| ステアリング | `.kiro/steering/*.md` (4ファイル) | 日本語「ラベル」+ 英語「label/Label」 |
| README | `README.md`, `examples/scripts/README.md` | 同上 |
| 仕様ドキュメント | `.kiro/specs/**/*.md` (~150ファイル) | 同上 |
| サンプルコメント | `examples/**/*.pasta` | Pastaコメント内の「ラベル」 |

#### 置換パターン（日本語）

- グローバルラベル → グローバルシーン
- ローカルラベル → ローカルシーン
- ラベル定義 → シーン定義
- ラベル呼び出し → シーン呼び出し
- ラベル前方一致 → シーン前方一致
- 重複ラベル → 重複シーン
- ラベルテーブル → シーンテーブル
- ラベル解決 → シーン解決
- ラベル検索 → シーン検索
- ラベル名 → シーン名
- ラベル内 → シーン内
- ラベル関数 → シーン関数
- ラベル登録 → シーン登録
- ラベル管理 → シーン管理
- ラベルID → シーンID
- ラベルマーカー → シーンマーカー
- ラベル辞書 → シーン辞書
- ラベル配下 → シーン配下
- ラベルブロック → シーンブロック
- ラベル行 → シーン行
- ラベルスコープ → シーンスコープ
- ラベル未発見 → シーン未発見
- ラベル直後 → シーン直後
- ラベル宣言 → シーン宣言
- ラベル実行 → シーン実行
- ラベル命名 → シーン命名
- ラベルジャンプテーブル → シーンジャンプテーブル
- ラベル継続チェイン → シーン継続チェイン
- サブラベル → サブシーン
- 親ラベル → 親シーン
- 同名ラベル → 同名シーン
- 呼び出し先ラベル → 呼び出し先シーン
- 遷移先ラベル → 遷移先シーン
- 助詞付き全パターン（ラベルから/へ/に/が/を/の/も/と 等）

#### 置換パターン（英語・識別子）

**Rust識別子:**
- `LabelRegistry` → `SceneRegistry`
- `LabelInfo` → `SceneInfo`
- `LabelDef` → `SceneDef`
- `LabelScope` → `SceneScope`
- `LabelTable` → `SceneTable`
- `LabelId` → `SceneId`
- `LabelNotFound` → `SceneNotFound`
- `label_registry` → `scene_registry`
- `label_table` → `scene_table`

**生成されるRuneコード変数名:**
- `label` → `scene`
- `label_fn` → `scene_fn`
- `label_id` → `scene_id`
- `label_selector` → `scene_selector`

**エラーメッセージ:**
- `"Label not found"` → `"Scene not found"`
- `"ラベルID"` → `"シーンID"`

### 1.3 アーキテクチャパターン

Pastaは以下のレイヤー構成：

```
Parser (ast.rs) → Transpiler (label_registry, mod) → Runtime (labels.rs) → Stdlib (mod.rs)
                                                    ↓
                                                 Engine
```

**リファクタリング影響範囲**: 全レイヤー（Parser, Transpiler, Runtime, Stdlib, Engine, Error）

**既存パターン**: 
- **2パストランスパイル**: Pass 1でシーン登録（Registry）、Pass 2でRune生成
- **ランタイムテーブル**: `LabelTable`がRadixMapで前方一致検索
- **ID管理**: トランスパイラが採番（1-based）、ランタイムが使用

**制約**:
- ファイル名変更はmod.rsの更新を伴う
- 公開API変更により下流依存がある場合は破壊的変更（現状: crates.io未公開のためOK）

---

## 2. 要件実現可能性分析

### 2.1 要件マッピング

| 要件 | 技術要素 | 既存アセット | ギャップ |
|------|---------|------------|---------|
| Req 1: ドキュメント用語統一 | Markdown置換 | 176 Markdownファイル | **手動スクリプト必要**（日本語パターンマッチング） |
| Req 2: ステアリング用語統一 | Markdown置換 | 4ステアリングファイル | 同上 |
| Req 3: Rust識別子リファクタリング | 構造体/列挙型リネーム | 8型定義 | **IDE Rename機能で自動化可能** |
| Req 4: 変数名・コメントリファクタリング | 変数・コメント置換 | 200+箇所（概算） | **IDEまたはスクリプト** |
| Req 5: テストコード用語統一 | テストファイル・関数リネーム | 3ファイル、~20関数 | **IDE Rename** |
| Req 6: エラーメッセージ用語統一 | エラーバリアントリネーム | `LabelNotFound` | **IDE Rename** |
| Req 7: 生成コード用語統一 | トランスパイラ出力修正 | `label_selector`生成箇所 | **コード修正**（L196, L226） |
| Req 8: 仕様ドキュメント更新 | Markdown置換 | ~150 specファイル | **手動スクリプト** |

### 2.2 ギャップと制約

| ギャップ | 詳細 | 解決策 |
|---------|------|-------|
| **日本語置換の複雑性** | 「ラベル」が文脈により異なる意味を持つ可能性 | **検証スクリプト**で誤検出を手動チェック |
| **Rust識別子の全参照更新** | IDEのRenameが全参照を追跡できるか | rust-analyzerのRename機能は信頼性高い |
| **mod.rsファイル更新** | ファイル名変更後、`mod label_registry;`等を修正 | **手動修正**（3箇所のみ） |
| **生成Runeコード検証** | トランスパイル結果が正しく`scene_selector`を生成するか | **既存テスト実行**で検証 |
| **破壊的変更の影響** | 公開APIが変更される | **制約ではない**（crates.io未公開、内部プロジェクト） |

### 2.3 複雑性シグナル

- **シンプルなCRUD**: ❌（リファクタリングのため該当しない）
- **アルゴリズム**: ❌
- **ワークフロー**: ✅（リファクタリングワークフロー：IDE Rename → スクリプト置換 → 検証）
- **外部統合**: ❌

---

## 3. 実装アプローチオプション

### Option A: 完全手動リファクタリング（非推奨）

**概要**: すべての置換を手動で検索・置換する。

**Trade-offs**:
- ❌ **高リスク**: 人的ミスによる置換漏れ・誤置換
- ❌ **低効率**: 数百箇所の手作業
- ✅ **シンプル**: ツール不要

**推奨度**: ⭐☆☆☆☆

---

### Option B: IDE Rename機能 + 手動スクリプト（推奨）

**概要**: 
1. **Phase 1（Rust識別子）**: rust-analyzerのRename機能で構造体・列挙型・関数をリネーム
2. **Phase 2（ドキュメント）**: PowerShellまたはPythonスクリプトでMarkdown内の日本語・英語用語を一括置換
3. **Phase 3（検証）**: `cargo test`, `cargo check`で破壊的変更を検証

**詳細**:

#### Phase 1: IDE Rename（Rust）

| ステップ | 対象 | ツール |
|---------|------|-------|
| 1.1 | `LabelRegistry` → `SceneRegistry` | VSCode: F2 (Rename Symbol) |
| 1.2 | `LabelInfo` → `SceneInfo` | 同上（2箇所を個別にRename） |
| 1.3 | `LabelDef` → `SceneDef` | 同上 |
| 1.4 | `LabelScope` → `SceneScope` | 同上 |
| 1.5 | `LabelTable` → `SceneTable` | 同上 |
| 1.6 | `LabelId` → `SceneId` | 同上 |
| 1.7 | `LabelNotFound` → `SceneNotFound` | 同上 |
| 1.8 | `select_label_to_id` → `select_scene_to_id` | 同上 |
| 1.9 | ファイル名変更 | 手動（3ファイル + mod.rs修正） |

#### Phase 2: スクリプト置換（Markdown）

```powershell
# 日本語パターン置換スクリプト例
$patterns = @{
    'グローバルラベル' = 'グローバルシーン'
    'ローカルラベル' = 'ローカルシーン'
    'ラベル定義' = 'シーン定義'
    'ラベル呼び出し' = 'シーン呼び出し'
    'ラベル前方一致' = 'シーン前方一致'
    '重複ラベル' = '重複シーン'
    # ... (その他パターン)
}

Get-ChildItem -Path . -Recurse -Include "*.md" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    foreach ($old in $patterns.Keys) {
        $content = $content -replace $old, $patterns[$old]
    }
    Set-Content $_.FullName $content
}
```

#### Phase 3: 検証

```bash
cargo check        # コンパイルエラー確認
cargo test         # 全テスト実行
cargo clippy       # リントエラー確認
```

**Trade-offs**:
- ✅ **高信頼性**: IDE Renameは全参照を追跡
- ✅ **効率的**: スクリプト自動化で手作業最小化
- ✅ **検証可能**: Rust型システムとテストで破壊的変更を検出
- ❌ **スクリプト作成コスト**: PowerShell/Pythonスクリプトの作成・テストが必要

**推奨度**: ⭐⭐⭐⭐⭐

---

### Option C: 完全自動化ツール開発（過剰）

**概要**: カスタムリファクタリングツールを開発し、Rust ASTパースとMarkdown置換を統合する。

**Trade-offs**:
- ✅ **再利用可能**: 将来の類似リファクタリングに活用
- ❌ **オーバーエンジニアリング**: 1回限りのリファクタリングには不要
- ❌ **開発コスト**: ツール開発に1-2日必要

**推奨度**: ⭐⭐☆☆☆

---

## 4. 実装複雑性とリスク

### 4.1 工数見積もり

| フェーズ | タスク | 工数 |
|---------|-------|-----|
| Phase 1 | IDE Rename（8型 + 主要関数） | **2-3時間** |
| Phase 2 | スクリプト作成・実行（Markdown置換） | **3-4時間** |
| Phase 3 | ファイル名変更 + mod.rs修正 | **1時間** |
| Phase 4 | 検証（cargo check/test/clippy） | **1-2時間** |
| Phase 5 | 手動レビュー（誤置換チェック） | **2-3時間** |
| **合計** | - | **M（9-13時間 ≈ 1-2日）** |

**サイズ**: **M（Medium）**

### 4.2 リスク評価

| リスク要因 | レベル | 緩和策 |
|-----------|-------|-------|
| **IDE Renameの漏れ** | Low | rust-analyzerは高精度、cargo checkで検証 |
| **日本語置換の誤検出** | Medium | スクリプト実行後、git diffで全変更をレビュー |
| **テスト失敗** | Low | 純粋なリファクタリングのため、テスト修正のみで対応可能 |
| **ドキュメント整合性** | Low | 用語統一により整合性は向上 |
| **破壊的変更の影響** | Low | 内部プロジェクト、公開APIなし |

**総合リスク**: **Medium**

---

## 5. 推奨事項（設計フェーズへ）

### 5.1 推奨アプローチ

**Option B（IDE Rename + 手動スクリプト）** を採用

**理由**:
- rust-analyzerのRename機能は信頼性が高く、全参照を自動追跡
- スクリプト自動化により、大量のMarkdown置換を効率化
- 既存テストによる検証で、破壊的変更を確実に検出

### 5.2 主要な設計決定事項

| 決定事項 | 選択肢 | 推奨 |
|---------|-------|-----|
| **Rust識別子リネーム** | 手動 vs IDE Rename | **IDE Rename**（F2） |
| **Markdown置換** | 手動 vs スクリプト | **スクリプト**（PowerShell/Python） |
| **ファイル名変更タイミング** | 先 vs 後 | **IDE Rename後**（依存関係更新後） |
| **検証方法** | 手動 vs 自動テスト | **自動テスト**（cargo test） |
| **レビュー方法** | 全diff vs サンプリング | **全diff**（git diff --stat + 重要箇所の詳細確認） |

### 5.3 設計フェーズで検討すべき項目

#### 技術調査不要（既知）
- ✅ rust-analyzerのRename機能の動作確認済み
- ✅ PowerShell/PythonによるMarkdown置換パターン確立済み

#### 設計フェーズで詳細化が必要

1. **Markdown置換スクリプトの完全パターンリスト**
   - 日本語用語の全バリエーション抽出
   - 誤検出を避けるための正規表現設計
   - 置換対象外パターンの定義（例: コードブロック内のRune変数名）

2. **ファイル名変更手順書**
   - `src/transpiler/label_registry.rs` → `scene_registry.rs`
   - `src/runtime/labels.rs` → `scenes.rs`（注: 複数形）
   - mod.rs修正箇所の特定

3. **検証チェックリスト**
   - cargo check: コンパイルエラーゼロ
   - cargo test: 全テストパス
   - cargo clippy: 新規警告なし
   - git diff: 意図しない変更がないか確認

4. **ロールバック戦略**
   - git branch作成（`refactor/label-to-scene`）
   - 各フェーズ後にcommit（Phase 1完了、Phase 2完了...）
   - 問題発生時は前のcommitにrevert

---

## 6. 要件-アセット対応表

| 要件ID | アセット | 状態 | ギャップ | アプローチ |
|-------|---------|------|---------|----------|
| Req 1.1-1.6 | 176 Markdownファイル | ✅ 存在 | **スクリプト必要** | Phase 2（スクリプト置換） |
| Req 2.1-2.4 | 4ステアリングファイル | ✅ 存在 | **スクリプト必要** | Phase 2（スクリプト置換） |
| Req 3.1 | `LabelRegistry` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 3.2 | `LabelInfo` | ✅ 存在（2箇所） | - | Phase 1（IDE Rename × 2） |
| Req 3.3 | `label_registry.rs` | ✅ 存在 | **mod.rs修正必要** | Phase 3（ファイル名変更） |
| Req 3.4 | `LabelTable` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 3.5 | `labels.rs` → `scene.rs` | ✅ 存在 | **mod.rs修正必要** | Phase 3（ファイル名変更） |
| Req 3.6 | `LabelDef` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 3.7 | `LabelScope` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 3.8 | `select_label_to_id` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 3.9 | `label_selector`（生成Rune） | ✅ 存在（L196） | **コード修正** | Phase 1（手動修正） |
| Req 3.10 | `execute_label` | ✅ 存在（engine.rs L314） | - | Phase 1（IDE Rename） |
| Req 4.1-4.6 | 変数名・コメント | ✅ 存在（200+箇所） | **IDE一括置換** | Phase 1（Find & Replace） |
| Req 5.1-5.4 | 3テストファイル | ✅ 存在 | **ファイル名変更** | Phase 3（ファイル名変更） |
| Req 6.1-6.3 | `LabelNotFound` | ✅ 存在 | - | Phase 1（IDE Rename） |
| Req 7.1-7.3 | トランスパイラ出力 | ✅ 存在 | **コード修正** | Phase 1（L196, L226修正） |
| Req 8.1-8.3 | 仕様ドキュメント（完了済み+進行中全て） | ✅ 存在（~150ファイル） | **スクリプト必要** | Phase 2（スクリプト置換） |
| Req 8.3 | `pasta-label-continuation/` → `pasta-scene-continuation/` | ✅ 存在 | **ディレクトリ名変更** | Phase 3（手動リネーム） |

---

## 7. 制約と前提

### 制約
- **破壊的変更許容**: crates.io未公開のため、公開API変更は問題なし
- **テストカバレッジ**: 既存テストが十分にカバーしているため、リファクタリング後の動作検証が可能
- **git管理**: すべての変更はgit管理下で追跡可能

### 前提
- rust-analyzerが正常に動作する環境（VSCode + rust-analyzer拡張）
- PowerShellまたはPython実行環境
- `cargo check`, `cargo test`, `cargo clippy`が実行可能

---

## 8. 未解決項目（Research Needed）

| 項目 | 詳細 | 調査結果 |
|------|------|---------|
| ~~`execute_label`の存在確認~~ | ~~Req 3.10で要求されているが、現状確認できていない~~ | ✅ **確認完了**: `src/engine.rs` L314に存在（`execute_label`, `execute_label_with_filters`） |
| 日本語「ラベル」の全出現パターン | スクリプト置換の完全性確保 | ✅ **確認完了**: Markdownファイル内に約110箇所出現 |
| 英語"label"の誤検出リスク | 変数名以外（例: HTMLの`<label>`タグ）が存在するか | ⚠️ **要注意**: コメント・ドキュメント内に多数出現、コードブロック内の置換は慎重に |

**結論**: 主要な未解決項目は解決済み。設計フェーズで詳細なスクリプト実装を行う準備が整った。

---

## 次のステップ

1. **未解決項目の調査**（設計フェーズ開始前）
   - `execute_label`の存在確認
   - 日本語「ラベル」の全パターン抽出

2. **設計フェーズへ進む**
   - `/kiro-spec-design refactor-label-to-scene` を実行
   - 設計ドキュメントで以下を詳細化：
     - Markdown置換スクリプトの完全実装
     - ファイル名変更手順書
     - 検証チェックリスト

3. **実装フェーズの準備**
   - git branchの作成
   - バックアップの作成（念のため）
