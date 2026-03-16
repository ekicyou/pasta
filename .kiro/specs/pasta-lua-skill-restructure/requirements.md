# Requirements Document

## Introduction

pasta-lua-codingスキルの構造改善仕様。現在のSKILL.md（588行）を500行未満にコンパクト化し、詳細なAPIリファレンスを `references/` サブディレクトリに分離する。AIエージェントがスキル本体を高速にロードしつつ、必要に応じてリッチなリファレンスを参照できる構成を実現する。

### 現状分析

| 項目 | 現状 |
|------|------|
| SKILL.md | 588行（7セクション構成） |
| references/ | 未作成 |
| 情報ソース | steering/lua-coding.md（695行）、LUA_API.md（1160行） |
| SKILL.mdの主要セクション | §1 Purpose（30行）、§2 Quick Ref（50行）、§3 Conventions（95行）、§4 Runtime API（120行）、§5 Internal Modules（115行）、§6 SHIORI Handlers（90行）、§7 Testing（60行） |

## Requirements

### Requirement 1: SKILL.md行数制約

**Objective:** As a AIエージェント, I want スキル本体を軽量にロードしたい, so that コンテキストウィンドウを効率的に使用できる

#### Acceptance Criteria

1. The SKILL.md shall have fewer than 500 lines in total（YAML frontmatter含む）
2. The SKILL.md shall retain the YAML frontmatter（name, description, metadata）without modification to `name` or `description` fields

### Requirement 2: referencesディレクトリ構成

**Objective:** As a AIエージェント, I want 詳細リファレンスをドメイン別に分割して参照したい, so that 必要な情報だけを選択的にロードできる

#### Acceptance Criteria

1. The references directory shall be located at `.agents/skills/pasta-lua-coding/references/`
2. The references directory shall contain domain-separated Markdown files for each major topic area
3. The SKILL.md shall contain a references index section listing all reference files with one-line descriptions
4. When AIエージェントがリファレンスを参照する場合, the reference files shall be self-contained and independently readable without requiring SKILL.md context

### Requirement 3: リファレンスファイルのドメイン分割

**Objective:** As a AIエージェント, I want ドメインごとに適切な粒度でリファレンスを取得したい, so that 無関係な情報のロードを避けられる

#### Acceptance Criteria

1. The references shall include a file for Runtime API（`@pasta_search`, `@pasta_persistence`, `@pasta_config`, `@pasta_sakura_script`, `@enc`, mlua-stdlib）
2. The references shall include a file for Internal Modules（STORE, ACT, SCENE, WORD, GLOBAL, SAVE, finalize_scene）
3. The references shall include a file for SHIORI Handlers（REG, RES, イベント一覧, フォールバック, 仮想ディスパッチャ）
4. The references shall include a file for Coding Conventions（命名規約, モジュール構造, クラス設計パターン, 型注釈, エラーハンドリング）
5. The references shall include a file for Testing & Lint（lua_test, テストファイル規約, 決定論的テスト, luacheck）

### Requirement 4: リファレンス内容のリッチ化

**Objective:** As a ゴースト開発者（AI/人間）, I want 現在のSKILL.mdよりも詳細で実用的なリファレンスを得たい, so that 実装時に情報ソース（LUA_API.md, steering/lua-coding.md）を直接参照する必要がなくなる

#### Acceptance Criteria

1. The reference files shall include complete API signatures with all parameters and return types
2. The reference files shall include practical usage examples（現行SKILL.mdの例に加え、情報ソースから追加例を含める）
3. The reference files shall include cross-reference links to related sections in other reference files（例: ACTオブジェクトの説明からSHIORI Handlersへのリンク）
4. The reference files shall document edge cases and caveats（例: `@pasta_config` のpcall必須理由、`@enc` のプラットフォーム依存性）
5. The reference files shall note their information source at the bottom of each file for traceability

### Requirement 5: SKILL.md本体の構成

**Objective:** As a AIエージェント, I want SKILL.md単体でスキルの全体像と使い方の概要を把握したい, so that リファレンスをロードするかどうかを判断できる

#### Acceptance Criteria

1. The SKILL.md shall retain the §1 Purpose & Prerequisites section（DSL vs Lua判断基準を含む）
2. The SKILL.md shall retain the §2 Quick Reference section（モジュール一覧テーブル、DSL→Luaブリッジ基本形）
3. The SKILL.md shall include a condensed summary（各セクション3-5行程度）for §3-§7 with reference links
4. The SKILL.md shall include a "References" section listing all reference files with file paths and one-line descriptions
5. If リファレンスファイルが存在しない場合, the SKILL.md shall still function as a standalone reference（degraded but usable）
