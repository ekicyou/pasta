# Research Log: areka-P0-script-engine

| 項目 | 内容 |
|------|------|
| **Date** | 2025-12-09 |
| **Status** | Completed |
| **Feature Classification** | New Feature (Greenfield Subcrate) |

---

## Research Scope

### Classification

**Feature Type**: New Feature（新規サブクレート `pasta` の設計）

**Discovery Depth**: Full Discovery（新規クレートのため完全な調査が必要）

### Key Questions

1. Rune Generators の API と統合パターン
2. パーサー選択（nom vs pest vs hand-written）
3. thiserror によるエラー型設計
4. TypewriterToken の拡張設計

---

## Research 1: Rune Generators

### Findings

**Rune 概要**:
- 埋め込み動的プログラミング言語（Rust製）
- バージョン: 0.14.x（現行安定版）
- 最小 Rust バージョン: 1.87+

**Generators 機能**:
```rune
fn fib() {
    let a = 0;
    let b = 1;
    loop {
        yield a;
        let c = a + b;
        a = b;
        b = c;
    }
}

let g = fib();
while let Some(n) = g.next() {
    println!("{n}");
    if n > 100 { break; }
}
```

**GeneratorState API**:
```rune
fn example() {
    let out = yield 1;
    println!("{:?}", out);
    2
}

let gen = example();
dbg!(gen.resume(()));  // Yielded(1)
dbg!(gen.resume("John"));  // Complete(2)
```

**Rust 統合パターン**:
```rust
use rune::{Context, Diagnostics, Source, Sources, Vm};
use std::sync::Arc;

let context = Context::with_default_modules()?;
let runtime = Arc::new(context.runtime()?);

let mut sources = Sources::new();
sources.insert(Source::memory("pub fn add(a, b) { a + b }")?);

let mut diagnostics = Diagnostics::new();
let result = rune::prepare(&mut sources)
    .with_context(&context)
    .with_diagnostics(&mut diagnostics)
    .build();

let unit = result?;
let mut vm = Vm::new(runtime, Arc::new(unit));
let output = vm.call(["add"], (10i64, 20i64))?;
```

### Implications

1. **Generator 制御**: `resume()` で値を送受信可能
2. **状態追跡**: `GeneratorState::Yielded` / `Complete` で判別
3. **エラーハンドリング**: 完了後の `resume()` はエラー
4. **Rust 統合**: `Context`, `Vm`, `Sources` が基本型

### Decision

✅ Rune Generators を採用（要件 Req-8 との適合性が高い）

---

## Research 2: Parser Selection

### Options Analysis

#### Option A: nom (Parser Combinator)

**特徴**:
- 関数型パーサーコンビネータ
- ゼロコピー・ストリーミング対応
- `IResult<I, O, E>` 型で結果管理
- 非常に軽量（no_std 対応）

**利点**:
- Rust ネイティブ、型安全
- 高パフォーマンス
- 柔軟なエラーハンドリング
- Unicode 対応（`nom::character`）

**欠点**:
- 学習曲線がやや急
- 文法が Rust コードに分散
- デバッグがやや困難

**サンプル**:
```rust
use nom::{IResult, Parser, bytes::complete::tag, character::complete::alpha1};

fn parse_label(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("＊")(input)?;
    alpha1(input)
}
```

#### Option B: pest (PEG Parser Generator)

**特徴**:
- Parsing Expression Grammar (PEG) ベース
- `.pest` ファイルで文法定義
- `#[derive(Parser)]` でコード生成

**利点**:
- 文法が別ファイル（可読性高い）
- 自動エラー回復
- WHITESPACE/COMMENT 自動処理
- Unicode 組み込み対応

**欠点**:
- ビルド時コード生成
- やや重い（メモリ消費）
- カスタマイズの自由度が低め

**サンプル文法**:
```pest
label = { "＊" ~ identifier }
identifier = @{ XID_START ~ XID_CONTINUE* }
WHITESPACE = _{ " " | "\t" | "　" }
```

#### Option C: Hand-written Parser

**特徴**:
- 完全なカスタム実装
- 行単位パーサー（DSL に最適化）

**利点**:
- 最高の柔軟性
- エラーメッセージ完全制御
- 依存ゼロ

**欠点**:
- 開発コスト高
- バグリスク高
- テストカバレッジ必要

### Recommendation Matrix

| 基準 | nom | pest | Hand-written |
|------|-----|------|--------------|
| 学習コスト | 中 | 低 | 高 |
| 実装コスト | 低 | 低 | 高 |
| 保守性 | 高 | 高 | 中 |
| Unicode対応 | 高 | 高 | 高 |
| エラーメッセージ | 中 | 高 | 高 |
| 柔軟性 | 高 | 中 | 最高 |
| 依存数 | 1 | 2 | 0 |

### Decision

🔶 **推奨: pest** (PEG文法の可読性とDSL記述の親和性)

**理由**:
1. Pasta DSL は行指向で PEG に適合
2. Unicode 識別子（XID_Start, XID_Continue）のネイティブサポート
3. WHITESPACE 自動処理が全角・半角対応に有利
4. `.pest` ファイルで文法の可視性・保守性向上

**代替案保留**: 実装開始後、pest の制約が問題になれば nom に切り替え可能

---

## Research 3: thiserror Error Design

### Findings

**thiserror 基本パターン**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PastaError {
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    
    #[error("Rune runtime error: {0}")]
    RuneError(#[from] rune::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

**エラーチェーン**:
- `#[from]` アトリビュートで自動変換
- `?` 演算子での伝播
- `source()` でチェーン参照

### Implications

1. 要件 NFR-2 との適合
2. 位置情報付きパースエラー
3. Rune エラーのラップ

### Decision

✅ thiserror を採用（要件仕様通り）

---

## Research 4: TypewriterToken Extension

### Current Definition

```rust
// crates/wintf/src/ecs/widget/text/typewriter_ir.rs
pub enum TypewriterToken {
    Text(String),
    Wait(f64),
    FireEvent {
        target: Entity,
        event: TypewriterEventKind,
    },
}
```

### Required Extensions for pasta

要件 Req-6（同期セクション）に基づき、以下の拡張が必要:

```rust
pub enum TypewriterToken {
    // 既存
    Text(String),
    Wait(f64),
    FireEvent { target: Entity, event: TypewriterEventKind },
    
    // 新規追加（同期セクション）
    BeginSync { sync_id: String },
    EndSync { sync_id: String },
    SyncPoint { sync_id: String },
    
    // 新規追加（発言者制御）
    ChangeSpeaker(String),
    
    // 新規追加（サーフェス制御）
    ChangeSurface { character_name: String, surface_id: u32 },
    
    // 新規追加（エラー）
    Error { message: String },
}
```

### Decision

✅ TypewriterToken を拡張（wintf-P0-typewriter との API 共有）

---

## Research 5: Subcrate Architecture

### Cargo Workspace Analysis

現在の構成:
```
dcomp_sample-rs/
├── Cargo.toml (workspace root)
└── crates/
    └── wintf/
        ├── Cargo.toml
        └── src/
```

### Proposed Structure

```
dcomp_sample-rs/
├── Cargo.toml (workspace root)
└── crates/
    ├── pasta/          # NEW
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs
    │       ├── parser/
    │       │   ├── mod.rs
    │       │   └── pasta.pest
    │       ├── ast.rs
    │       ├── transpiler.rs
    │       ├── runtime/
    │       │   ├── mod.rs
    │       │   ├── generator.rs
    │       │   └── variables.rs
    │       └── stdlib/
    │           ├── mod.rs
    │           └── functions.rune
    └── wintf/
        ├── Cargo.toml  # pasta dependency追加
        └── src/
```

### Cargo.toml Design

```toml
# crates/pasta/Cargo.toml
[package]
name = "pasta"
version = "0.1.0"
edition = "2024"

[dependencies]
rune = "0.14"
thiserror = "2"
pest = "2.8"
pest_derive = "2.8"
glob = "0.3"

[dev-dependencies]
# wintf の TypewriterToken 型定義のみ参照
wintf = { path = "../wintf", features = ["ir-types-only"] }
```

### Decision

✅ サブクレート `pasta` を `crates/pasta/` に配置

---

## Summary

| 項目 | 決定 | 根拠 |
|------|------|------|
| スクリプトエンジン | Rune Generators | 中断・再開、yield による段階的 IR 生成 |
| パーサー | pest (PEG) | Unicode 対応、文法可視性、DSL 親和性 |
| エラー型 | thiserror | 要件仕様準拠、構造化エラー |
| IR 共有 | TypewriterToken 拡張 | wintf との API 境界明確化 |
| クレート構成 | crates/pasta/ | ワークスペース構成準拠 |

---

## References

- [Rune Documentation](https://rune-rs.github.io/)
- [Rune Generators](https://rune-rs.github.io/book/generators.html)
- [pest Documentation](https://pest.rs/)
- [nom Documentation](https://docs.rs/nom/latest/nom/)
- [thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/)
