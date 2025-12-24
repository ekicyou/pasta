# Technical Design

---
**Purpose**: `src/engine.rs`をparser2/transpiler2スタックに移行し、旧スタックへの参照を完全に排除する

---

## Overview
このドキュメントは、PastaEngineのコア実装である`engine.rs`を、新しいパーサー（parser2）とトランスパイラー（transpiler2）スタックに移行するための技術設計を定義する。

### Scope
- `src/engine.rs`のインポート文更新
- パース処理のparser2 API呼び出しへの置き換え
- ASTマージロジックのitems構造対応
- トランスパイル処理の2-pass API呼び出しへの移行
- エラーハンドリングの統合

### Classification
- **Type**: Extension（既存システムの内部実装変更）
- **Complexity**: Medium
- **Impact**: Core（エンジン中核部分）

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PastaEngine                                   │
│  ┌──────────────┐    ┌──────────────────────────────────────────┐  │
│  │    Loader    │───▶│           engine.rs (修正対象)            │  │
│  │ (変更なし)   │    │  ┌────────────────────────────────────┐  │  │
│  └──────────────┘    │  │  Parse Loop                         │  │  │
│                      │  │  - parser2::parse_file() 呼び出し   │  │  │
│                      │  │  - PastaError::ParseError変換       │  │  │
│                      │  └────────────────────────────────────┘  │  │
│                      │              ▼                            │  │
│                      │  ┌────────────────────────────────────┐  │  │
│                      │  │  AST Merge                          │  │  │
│                      │  │  - items構造でextend                │  │  │
│                      │  │  - 単一PastaFile構築                │  │  │
│                      │  └────────────────────────────────────┘  │  │
│                      │              ▼                            │  │
│                      │  ┌────────────────────────────────────┐  │  │
│                      │  │  Transpile (2-pass)                 │  │  │
│                      │  │  - Pass1: Registry登録+モジュール   │  │  │
│                      │  │  - Pass2: Selector生成              │  │  │
│                      │  └────────────────────────────────────┘  │  │
│                      │              ▼                            │  │
│                      │  ┌────────────────────────────────────┐  │  │
│  ┌──────────────┐    │  │  Runtime Integration (変更なし)     │  │  │
│  │   Runtime    │◀───│  │  - SceneTable構築                  │  │  │
│  │ (変更なし)   │    │  │  - WordTable構築                   │  │  │
│  └──────────────┘    │  └────────────────────────────────────┘  │  │
│                      └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Changes

| Component           | Current State              | Target State                      | Change Type |
| ------------------- | -------------------------- | --------------------------------- | ----------- |
| engine.rs imports   | parser, transpiler         | parser2, transpiler2, error types | Replace     |
| Parse loop          | `parser::parse_file()`     | `parser2::parse_file()`           | Replace     |
| AST merge           | scenes/global_words extend | items extend                      | Modify      |
| Transpile           | Single-pass call           | 2-pass sequence                   | Replace     |
| Error handling      | 旧TranspileError           | PastaError変換                    | Enhance     |
| Runtime integration | (unchanged)                | (unchanged)                       | None        |

---

## Detailed Design

### 1. Import Statement Changes

**Location**: [src/engine.rs](../../../src/engine.rs#L6-L13)

**Current State**:
```rust
use crate::loader::{DirectoryLoader, PastaLoader};
use crate::parser::{self, PastaFile};
use crate::runtime::evaluator::Evaluator;
use crate::runtime::label_table::{SceneTable, WordTable};
use crate::runtime::session::Session;
use crate::transpiler::Transpiler;
use crate::PastaError;
```

**Target State**:
```rust
use crate::error::Transpiler2Pass;
use crate::loader::{DirectoryLoader, PastaLoader};
use crate::parser2::{self, ast::PastaFile};
use crate::registry::{SceneRegistry, WordDefRegistry};
use crate::runtime::evaluator::Evaluator;
use crate::runtime::label_table::{SceneTable, WordTable};
use crate::runtime::session::Session;
use crate::transpiler2::Transpiler2;
use crate::PastaError;
```

**Design Rationale**:
- `parser` → `parser2`: 新パーサーモジュール
- `parser::PastaFile` → `parser2::ast::PastaFile`: 新AST構造
- `Transpiler` → `Transpiler2`: 2-passトランスパイラー
- `Transpiler2Pass`: エラー変換用の列挙型
- `SceneRegistry`, `WordDefRegistry`: 事前作成が必要

---

### 2. Parse Loop Modification

**Location**: [src/engine.rs](../../../src/engine.rs#L127-L148)

**Current Logic**:
```rust
for (path, content) in files {
    let file = parser::parse_file(&path, &content).map_err(|e| ...)?;
    parsed_files.push(file);
}
```

**New Logic**:
```rust
for (path, content) in files {
    let file = parser2::parse_file(&path, &content)
        .map_err(|e| PastaError::ParseError {
            file: path.clone(),
            line: e.line_col().0,
            column: e.line_col().1,
            message: e.to_string(),
        })?;
    parsed_files.push(file);
}
```

**Note**: parser2::parse_file()は既に内部でPest errorをPastaError::ParseErrorに変換済み（本仕様で実装済み）。よってそのまま`?`演算子で伝播可能。

**Actual Implementation** (parser2で変換済みのため):
```rust
for (path, content) in files {
    let file = parser2::parse_file(&path, &content)?;
    parsed_files.push(file);
}
```

---

### 3. AST Merge Logic

**Location**: [src/engine.rs](../../../src/engine.rs#L149-L157)

**Current Logic**:
```rust
// Merge all parsed files into a single unit
let mut all_scenes = Vec::new();
let mut all_global_words = Vec::new();
for file in &parsed_files {
    all_scenes.extend(file.scenes.iter().cloned());
    all_global_words.extend(file.global_words.iter().cloned());
}
```

**New Logic**:
```rust
use crate::parser2::ast::FileItem;

// Merge all parsed files into items-based structure
let mut all_items: Vec<FileItem> = Vec::new();
for file in &parsed_files {
    all_items.extend(file.items.iter().cloned());
}

// Create merged PastaFile
let merged_file = PastaFile {
    path: "merged".to_string(),
    items: all_items,
    span: None, // Merged file has no single span
};
```

**Design Rationale**:
- items構造でextendするだけで順序保持
- path="merged"は複数ファイルの統合を表現
- spanはNoneで明示的に「マージ後」を表現

---

### 4. Transpile 2-Pass Sequence

**Location**: [src/engine.rs](../../../src/engine.rs#L158-L174)

**API Verification** (実装確認済み):
- `Transpiler2::transpile_pass1<W: std::io::Write>(file, scene_registry, word_registry, writer)`
- `Transpiler2::transpile_pass2<W: std::io::Write>(scene_registry, writer)`
- `Vec<u8>` は `std::io::Write` を実装済み、バッファとして適切
- WordDefRegistry API: `register_global(&str, Vec<String>)` - GlobalWord型と完全一致

**Current Logic**:
```rust
let (rune_code, scene_registry, word_def_registry) =
    Transpiler::transpile_with_registry(&merged_file)?;
```

**New Logic**:
```rust
// Prepare registries and buffer
let mut scene_registry = SceneRegistry::new();
let mut word_def_registry = WordDefRegistry::new();
let mut buffer: Vec<u8> = Vec::new();

// Pass 1: Register definitions and generate modules
Transpiler2::transpile_pass1(&merged_file, &mut scene_registry, &mut word_def_registry, &mut buffer)
    .map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass1))?;

// Pass 2: Generate selectors
Transpiler2::transpile_pass2(&scene_registry, &mut buffer)
    .map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass2))?;

// Convert buffer to String
let rune_code = String::from_utf8(buffer)
    .map_err(|e| PastaError::TranspileError(format!("UTF-8 conversion error: {}", e)))?;
```

**Design Rationale**:
- 2-passで段階的に処理を分離
- Pass1: 定義登録とモジュール生成（前方参照解決）
- Pass2: Selector生成（全定義が揃った状態で）
- エラーにPass段階情報を付与して追跡性向上

**Prerequisites** (実装確認済み):
- `TranspileError::into_pasta_error(Transpiler2Pass)` 実装済み ([src/transpiler2/error.rs](../../../src/transpiler2/error.rs#L74-L79))
- `PastaError::Transpiler2Error` バリアント追加済み
- `Transpiler2Pass` 列挙型定義済み

---

### 5. Error Flow Integration

**Sequence Diagram**:
```
┌─────────────┐  ┌──────────┐  ┌─────────────┐  ┌──────────────┐
│  engine.rs  │  │ parser2  │  │ transpiler2 │  │ PastaError   │
└──────┬──────┘  └────┬─────┘  └──────┬──────┘  └──────┬───────┘
       │              │               │                │
       │ parse_file() │               │                │
       │─────────────▶│               │                │
       │              │               │                │
       │    Ok(file)  │               │                │
       │◀─────────────│               │                │
       │   or Err ────┼───────────────┼───────────────▶│ ParseError
       │              │               │                │
       │              │ pass1()       │                │
       │──────────────┼──────────────▶│                │
       │              │   Ok/Err      │                │
       │◀─────────────┼───────────────│                │
       │              │   Err ────────┼───────────────▶│ Transpiler2Error(Pass1)
       │              │               │                │
       │              │ pass2()       │                │
       │──────────────┼──────────────▶│                │
       │              │   Ok/Err      │                │
       │◀─────────────┼───────────────│                │
       │              │   Err ────────┼───────────────▶│ Transpiler2Error(Pass2)
       │              │               │                │
```

---

## Data Models

### PastaFile (parser2::ast)
```rust
pub struct PastaFile {
    pub path: String,
    pub items: Vec<FileItem>,  // 旧: scenes + global_words 分離構造
    pub span: Option<Span>,
}

pub enum FileItem {
    FileAttr(FileAttr),
    GlobalWord(GlobalWord),
    GlobalSceneScope(GlobalSceneScope),
}

// KeyWords型（GlobalWordのエイリアス）
pub struct KeyWords {
    pub name: String,
    pub words: Vec<String>,
    pub span: Span,
}
// 注: GlobalWord = KeyWords (同一型)
```

### Registry Types (shared)
```rust
// src/registry/scene_registry.rs
pub struct SceneRegistry { /* ... */ }

// src/registry/word_registry.rs (注: word_def_registryではない)
pub struct WordDefRegistry {
    entries: Vec<WordEntry>,
}

impl WordDefRegistry {
    pub fn register_global(&mut self, name: &str, values: Vec<String>) -> usize;
    pub fn register_local(&mut self, module_name: &str, name: &str, values: Vec<String>) -> usize;
    pub fn all_entries(&self) -> &[WordEntry];
}

// 型変換不要: KeyWords.words は既に Vec<String>
```

### Error Types
```rust
// src/error.rs
pub enum PastaError {
    ParseError { file: String, line: usize, column: usize, message: String },
    Transpiler2Error { pass: Transpiler2Pass, message: String },
    TranspileError(String),  // Legacy (旧transpiler用、残存)
    // ... other variants
}

pub enum Transpiler2Pass {
    Pass1,
    Pass2,
}
```

---

## Implementation Plan

### Phase 1: Import更新
1. engine.rsのuse文を新スタックに変更
2. コンパイルエラーを確認（後続フェーズで解消）

### Phase 2: Parse Loop変更
1. parser::parse_file → parser2::parse_file
2. エラー型は既にPastaError変換済みのため追加作業なし
3. 単体テストで動作確認

### Phase 3: AST Merge変更
1. scenes/global_words分離 → items統合に変更
2. merged_file構築ロジック実装
3. 結合テストで動作確認

### Phase 4: Transpile 2-Pass変更
1. Registry事前作成
2. pass1/pass2順次呼び出し
3. バッファ→String変換
4. 全テスト実行で検証

---

## Test Strategy

### Existing Test Coverage
- 611テスト成功（3 ignored）— 変更によるリグレッション検出
- parser2: 94テスト — パース機能カバー
- transpiler2: 専用テスト — 2-pass動作カバー

### Verification Approach
1. **Phase単位検証**: 各Phase完了時に`cargo test`実行
2. **リグレッションチェック**: 全611テスト成功維持
3. **特定テスト重点確認**:
   - `pasta_engine_rune_*` テスト群 — エンジン統合
   - `pasta_integration_*` テスト群 — E2E動作

---

## Requirements Traceability

| Requirement         | Design Section      | Verification                    |
| ------------------- | ------------------- | ------------------------------- |
| REQ-PARSER2-001     | 2. Parse Loop       | cargo test (parser integration) |
| REQ-AST-002         | 3. AST Merge        | cargo test (merge behavior)     |
| REQ-TRANSPILER2-003 | 4. Transpile 2-Pass | cargo test (transpile output)   |
| REQ-REGISTRY-004    | 4. Transpile 2-Pass | レジストリ参照の一貫性確認      |
| REQ-COMPAT-005      | All sections        | 611テスト成功維持               |
| REQ-TEST-006        | Test Strategy       | CI/CD通過                       |
| REQ-DOC-007         | Implementation Plan | CHANGELOG更新                   |

---

## Appendix

### References
- [requirements.md](requirements.md) — 機能要件一覧
- [gap-analysis.md](gap-analysis.md) — 現状分析とギャップ
- [research.md](research.md) — 調査結果と設計判断

### Change Log
| Date       | Version | Author | Changes                 |
| ---------- | ------- | ------ | ----------------------- |
| 2025-01-XX | 1.0     | AI     | Initial design document |
