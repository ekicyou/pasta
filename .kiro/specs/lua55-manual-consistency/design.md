# Design Document: lua55-manual-consistency

## Overview

**Purpose**: lua55-manualドキュメント（Lua 5.5リファレンスマニュアル日本語版）の整合性を修正し、GitHubでの閲覧体験を向上させる。

**Users**: ドキュメント閲覧者、ドキュメント管理者

**Impact**: 13ファイルのMarkdown構造を統一し、README.mdを完全な目次として再構成する。

### Goals
- 全ページで統一されたナビゲーション・メタデータ構造
- 6章のセクション重複を解消（31→11見出し）
- README.mdをGitHubデフォルトの完全目次化
- 全リンクの動作保証

### Non-Goals
- 翻訳内容の修正・改善
- 新規コンテンツの追加
- 自動化ツールの作成

## Architecture

### Architecture Pattern

**パターン**: 段階的ハイブリッドアプローチ（Phase 1-5）

各フェーズで独立した作業を行い、フェーズ間でバリデーションを実施。AIコンテキストを最小化するため、各タスクは1ファイルまたは1セクション単位で完結させる。

```
Phase 1: ナビリンク統一 (01, 05-09, GLOSSARY, LICENSE)
    ↓ [バリデーション]
Phase 2: 6章セクション重複解消 (06のみ、4分割作業)
    ↓ [バリデーション]
Phase 3: リンク検証・修正 (index.md内リンク)
    ↓ [バリデーション]
Phase 4: メタデータ統一 (05, 07, 08, 09)
    ↓ [バリデーション]
Phase 5: ファイル構成変更 (README←index, ABOUT←README)
    ↓ [最終バリデーション]
完了
```

### Technology Stack

| Layer | Choice | Role |
|-------|--------|------|
| 対象 | Markdown | ドキュメント形式 |
| プラットフォーム | GitHub | 表示・配布 |
| バージョン管理 | Git | 変更追跡・ロールバック |

## Requirements Traceability

| Req | Summary | Phase | Files |
|-----|---------|-------|-------|
| 1 | ナビリンク統一 | 1 | 01,05-09,GLOSSARY,LICENSE |
| 2 | メタデータ統一 | 4 | 05,07,08,09 |
| 3 | 6章セクション重複 | 2 | 06 |
| 4 | ファイル構成変更 | 5 | README,index,ABOUT |
| 5 | リンク検証 | 3,5 | README(旧index) |
| 6 | 付録整合性 | 1 | GLOSSARY,LICENSE |
| 7 | 段階的作業 | 全Phase | - |

## Phase Details

### Phase 1: ナビリンク統一

**対象**: 01, 05, 06, 07, 08, 09, GLOSSARY, LICENSE（8ファイル）
**工数**: S（1-2時間）

#### タスク分割

| Task | File | Action |
|------|------|--------|
| 1.1 | 01-introduction.md | ナビを詳細版に修正 |
| 1.2 | 05-auxiliary-library.md | ナビ追加 |
| 1.3 | 06-standard-libraries.md | ナビ追加 |
| 1.4 | 07-standalone.md | ナビ追加 |
| 1.5 | 08-incompatibilities.md | ナビ追加 |
| 1.6 | 09-complete-syntax.md | ナビ追加 |
| 1.7 | GLOSSARY.md | ナビ追加 |
| 1.8 | LICENSE.md | ナビ追加 |

#### ナビゲーションテンプレート

**標準形式**:
```markdown
[← 前へ: N-1 – タイトル](前.md) | [目次](./README.md) | [次へ: N+1 – タイトル →](次.md)

---
```

**章マッピング**:
| 章 | ファイル | タイトル |
|----|---------|---------|
| 1 | 01-introduction.md | はじめに |
| 2 | 02-basic-concepts.md | 基本概念 |
| 3 | 03-language.md | 言語 |
| 4 | 04-c-api.md | C API |
| 5 | 05-auxiliary-library.md | 補助ライブラリ |
| 6 | 06-standard-libraries.md | 標準ライブラリ |
| 7 | 07-standalone.md | スタンドアロン |
| 8 | 08-incompatibilities.md | 非互換性 |
| 9 | 09-complete-syntax.md | 完全な構文 |

### Phase 2: 6章セクション重複解消

**対象**: 06-standard-libraries.md（110KB）
**工数**: L（1-2日）
**リスク**: High - アンカーID変更によるリンク切れ

#### タスク分割（4分割）

| Task | Sections | 重複見出し数 |
|------|----------|-------------|
| 2.1 | 6.1-6.3 | 6.2×5回 |
| 2.2 | 6.4-6.6 | 6.4×2, 6.5×4 |
| 2.3 | 6.7-6.9 | 6.8×5, 6.9×4 |
| 2.4 | 6.10-6.11 | 6.10×2, 6.11×3 |

#### 修正パターン

**Before**:
```markdown
## 6.2 – 基本関数（パートA）
...内容A...
## 6.2 – 基本関数（パートB）
...内容B...
```

**After**:
```markdown
## 6.2 – 基本関数
...内容A...
...内容B...
```

#### アンカーID規則
- 形式: `#62-基本関数`（シングルハイフン、日本語維持）
- 最初のセクション見出しのみ残し、後続パートは見出しを削除してコンテンツを統合

### Phase 3: リンク検証・修正

**対象**: index.md内の全リンク（約200件）
**工数**: M（半日）
**依存**: Phase 2完了後

#### タスク分割

| Task | Category | 件数（概算）|
|------|----------|-----------|
| 3.1 | 目次リンク | 50件 |
| 3.2 | Lua関数索引 | 80件 |
| 3.3 | C API索引 | 50件 |
| 3.4 | 型索引 | 20件 |

#### 検証方法
1. 各リンクのアンカーIDが対象ファイルに存在することを確認
2. 6章のアンカーID変更に伴う修正を適用
3. 修正後、サンプルリンクで動作確認

### Phase 4: メタデータ統一

**対象**: 05, 07, 08, 09（4ファイル、現在blockquote形式）
**工数**: S（1-2時間）

#### タスク分割

| Task | File |
|------|------|
| 4.1 | 05-auxiliary-library.md |
| 4.2 | 07-standalone.md |
| 4.3 | 08-incompatibilities.md |
| 4.4 | 09-complete-syntax.md |

#### メタデータテンプレート

```html
<!--
  原文: https://www.lua.org/manual/5.5/manual.html#N
  参考: https://lua.dokyumento.jp/manual/5.4/manual.html#N
  翻訳日: 2026-01-29
  レビュー: AI Claude Opus 4.5
  用語対照: GLOSSARY.md参照
-->
```

### Phase 5: ファイル構成変更

**対象**: README.md, index.md, ABOUT.md（新規作成）
**工数**: M（半日）
**依存**: Phase 3完了後（リンク修正済みのindex.md使用）

#### タスク分割

| Task | Action |
|------|--------|
| 5.1 | 現README.md → ABOUT.md にリネーム・移動 |
| 5.2 | 現index.md → README.md にリネーム |
| 5.3 | 新README.md末尾にABOUT.mdリンク追加 |
| 5.4 | 新README.md冒頭にHTMLコメントメタデータ追加 |
| 5.5 | index.md削除確認 |
| 5.6 | 全ファイルのナビリンク検証（./README.md参照） |

#### 新README.md構成

```markdown
<!--
  原文: https://www.lua.org/manual/5.5/
  翻訳日: 2026-01-29
  レビュー: AI Claude Opus 4.5
-->

# Lua 5.5 リファレンスマニュアル

> リファレンスマニュアルはLua言語の公式定義です。

---

## ❖ 目次
[詳細目次（現index.mdの内容）]

## ❖ 索引
[Lua関数・C API・型（現index.mdの内容）]

---

## 翻訳について
翻訳についての詳細は[ABOUT.md](ABOUT.md)を参照してください。
```

## Risk Mitigation

| リスク | 対策 |
|--------|------|
| 6章編集ミス | 各タスク後にコミット、行数比較 |
| リンク切れ | Phase 2→3の順序厳守、検証タスク |
| ロールバック必要時 | Git履歴から復元可能 |

## Validation Checkpoints

| After Phase | Validation |
|-------------|------------|
| 1 | 各章のナビリンク動作確認 |
| 2 | 6章セクション数=11確認 |
| 3 | リンク全件検証 |
| 4 | メタデータ形式統一確認 |
| 5 | README.md表示確認、全ナビリンク動作 |

## Implementation Notes

### AIコンテキスト最小化

1. **1タスク=1ファイルまたは1セクション**: 複数ファイルを同時に扱わない
2. **テンプレート参照**: 各タスクでテンプレートを明示的に提供
3. **独立性**: 各タスクは前のタスクの結果に依存しない（Phase間依存のみ）
4. **明確な完了基準**: 各タスクに検証可能な完了条件

### コミット戦略

- 各タスク完了時にコミット
- コミットメッセージ形式: `docs(lua55-manual): [Phase.Task] 説明`
- 例: `docs(lua55-manual): [1.2] add navigation to 05-auxiliary-library`
