# Requirements Document: pasta-transpiler-actor-variables

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta トランスパイラー アクター変数参照修正 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-14 |
| **Priority** | P1 (Correctness) |
| **Status** | Requirements Generated |

---

## Introduction

本要件定義書は、Pasta DSLトランスパイラーがアクター代入時に文字列リテラルではなく、変数参照を生成するように修正する。現在の実装では `ctx.actor = "さくら"` のように文字列として出力しているが、正しくは `ctx.actor = さくら` のようにアクター変数（オブジェクト）を参照すべきである。

### Background

**現在の実装（誤り）:**

```rune
pub fn __start__(ctx, args) {
    ctx.actor = "さくら";        // 文字列リテラル（誤り）
    yield Actor("さくら");       // 文字列リテラル（誤り） - Actor関数自体は正しい
    yield Talk("こんにちは");
}
```

**参照実装（正しい）:**

```rune
// main.rn で定義
pub mod actors {
    pub const さくら = #{
        name: "さくら",
        id: "sakura",
    };

    pub const うにゅう = #{
        name: "うにゅう",
        id: "unyuu",
    };
}

// トランスパイル出力
pub mod メイン_1 {
    use pasta::*;           // pasta関数（jump, call）を短縮形で使用
    use pasta_stdlib::*;    // pasta_stdlib関数（word等）を短縮形で使用
    use crate::actors::*;   // アクター定義をインポート
    
    pub fn __start__(ctx, args) {
        ctx.actor = さくら;          // 変数参照（オブジェクト）
        yield Actor(ctx.actor.name); // アクター名フィールドを使用
        yield Talk("こんにちは");   // Talk関数呼び出し
    }
}
```

### Problem Statement

**課題1: 型の不一致とアクセス方法**

- **現在**: 文字列 `"さくら"` を代入・参照
- **期待**: オブジェクト `#{ name: "さくら", id: "sakura" }` を代入し、`ctx.actor.name` で名前を参照

**課題2: 拡張性の欠如**

文字列では以下が不可能：
- アクターID（内部識別子）の管理
- アクター属性（表示名、画像パス、声優情報など）の拡張
- 実行時のアクター情報参照

**課題3: 設計仕様との乖離**

参照実装（`comprehensive_control_flow.rn`）では変数参照が使用されており、トランスパイラー出力が設計意図に反する。

**課題4: 発見の経緯**

`pasta-transpiler-pass2-output` 仕様の検証中、Rune VMコンパイル検証で発見。Pass 2出力は正常だが、Pass 1（アクター代入）に問題があることが判明。

### Scope

**含まれるもの：**

1. **Pass 1 の Statement::Speech 処理修正**
   - `ctx.actor = "さくら"` → `ctx.actor = さくら` に変更
   - `yield Actor("さくら")` → `yield Actor(さくら)` または `yield #{ type: "Actor", actor: ctx.actor }` に変更

2. **モジュールレベルの use 文生成**
   - `use super::{さくら, うにゅう};` などのインポート文を生成
   - スクリプト内で使用される全アクターを自動検出してインポート

3. **テスト更新**
   - トランスパイラー出力の検証パターン更新
   - Rune VMコンパイル検証（アクター変数の解決確認）

**含まれないもの：**

- `main.rn` のアクター定義生成（手動またはプロジェクト初期化で対応）
- アクター情報の動的登録機能
- Pass 2 の修正（`pasta-transpiler-pass2-output` で完了済み）

---

## Requirements

### Requirement 1: アクター変数参照の生成

**Objective:** トランスパイラー開発者として、アクター代入時に文字列ではなく変数を参照することで、型安全性と拡張性を確保する。

#### Acceptance Criteria

1. When 会話文（`Statement::Speech`）をトランスパイルする, the Pasta Transpiler shall `ctx.actor = さくら;` のように変数名（識別子）を出力する
2. When アクター名に日本語が含まれる, the Pasta Transpiler shall そのまま識別子として使用する（`さくら`, `うにゅう`, `ななこ` など）
3. When アクター名に記号が含まれる, the Pasta Transpiler shall サニタイズせず、そのまま出力する（Rune識別子として有効な場合）
4. When 文字列リテラルを出力しない, the Pasta Transpiler shall ダブルクォートで囲まない
5. When トランスパイラーがエラーを検出しない, the Pasta Transpiler shall アクターが `main.rn` で定義されていることは検証しない（Rune VMコンパイル時に検証）

#### 現在のコード（修正対象）

**場所**: `crates/pasta/src/transpiler/mod.rs:353`

```rust
// Generate speaker change (store as string)
writeln!(writer, "        ctx.actor = \"{}\";", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

#### 修正後のコード

```rust
// Generate speaker change (store as actor variable)
writeln!(writer, "        ctx.actor = {};", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

---

### Requirement 2: Actor イベント生成の修正

**Objective:** スクリプト実行者として、アクター変更イベントを `Actor` 関数呼び出しで生成し、アクター変数の `name` フィールドを引数として渡すことを保証する。

#### Acceptance Criteria

1. When アクター変更を出力する, the Pasta Transpiler shall `yield Actor(ctx.actor.name);` の形式で出力する
2. When アクター情報を参照する, the Pasta Transpiler shall `ctx.actor.name` から現在のアクター名を取得する
3. When 文字列リテラルを使用しない, the Pasta Transpiler shall `yield Actor("さくら")` の形式を使用しない
4. When アクター変数の構造を期待する, the Pasta Transpiler shall アクター変数に `name` フィールドが存在することを前提とする
5. When Actor 関数を呼び出す, the Pasta Transpiler shall `pasta_stdlib::Actor` または短縮形 `Actor` を使用する

#### 現在のコード（修正対象）

**場所**: `crates/pasta/src/transpiler/mod.rs:355`

```rust
writeln!(writer, "        yield Actor(\"{}\");", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

#### 修正後のコード

```rust
writeln!(writer, "        yield Actor(ctx.actor.name);")
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

**注記**: 
- トランスパイラーはアクター変数に `name` フィールドが存在することを期待してよい（`main.rn` で保証）
- `Actor` 関数は `pasta_stdlib::actor_event(name: String)` として定義され、`ScriptEvent::ChangeSpeaker { name }` を返す
- Rust側の実装では文字列（name）のみを必要とするため、アクターオブジェクト全体ではなく `name` フィールドのみを渡す

---

### Requirement 3: モジュールレベル use 文の生成

**Objective:** 開発者として、生成されたモジュールが必要なアクター変数をインポートし、Rune VMコンパイルが成功することを保証する。

#### Acceptance Criteria

1. When モジュール（`pub mod ラベル名_N`）を生成する, the Pasta Transpiler shall モジュール冒頭に以下の3つのuse文を順に出力する
   - `use pasta::*;` - pasta関数（jump, call）を短縮形で使用
   - `use pasta_stdlib::*;` - pasta_stdlib関数（word, select_label_to_id等）を短縮形で使用
   - `use crate::actors::*;` - アクター定義をインポート
2. When ワイルドカードインポートを使用する, the Pasta Transpiler shall アクターの存在判定・個別収集・ソート処理は不要である
3. When 短縮形を有効にする, the Pasta Transpiler shall `crate::pasta::jump` の代わりに `jump` を使用できるようにする
4. When 短縮形を有効にする, the Pasta Transpiler shall `pasta_stdlib::word` の代わりに `word` を使用できるようにする

#### 生成例

```rune
pub mod メイン_1 {
    use pasta::*;           // jump, call を短縮形で
    use pasta_stdlib::*;    // word, Actor, Talk を短縮形で
    use crate::actors::*;   // さくら, うにゅう をインポート

    pub fn __start__(ctx, args) {
        ctx.actor = さくら;           // actors モジュールから自動的に解決
        yield Actor(ctx.actor.name);  // アクター名フィールドを使用
        
        // 短縮形で呼び出し可能
        for a in call(ctx, "自己紹介", #{}, []) { yield a; }  // crate::pasta::call → call
        yield word(ctx, "挨拶", []);  // pasta_stdlib::word → word
    }
}
```

**注記**: 
- 決定事項（2025-12-14）により、アクター定義は `pub mod actors { ... }` 内に配置
- `use pasta::*;` と `use pasta_stdlib::*;` により、関数呼び出しが簡潔化
- `use crate::actors::*;` により、アクターの個別列挙が不要
- ワイルドカードインポートを使用する理由は、各モジュールに集約したため名前空間の衝突リスクが低いため
- アクターの存在判定は不要（アクターが存在しないスクリプトは想定しない）

---

### Requirement 4: pasta関数の短縮形呼び出し

**Objective:** 開発者として、トランスパイラーが生成するコードで pasta 関数を短縮形で呼び出し、可読性を向上させる。

#### Acceptance Criteria

1. When ラベル呼び出し（call）を生成する, the Pasta Transpiler shall `for a in call(ctx, "ラベル名", #{}, []) { yield a; }` の形式で出力する
2. When ラベルジャンプ（jump）を生成する, the Pasta Transpiler shall `for a in jump(ctx, "ラベル名", #{}, []) { yield a; }` の形式で出力する
3. When パス修飾を使用しない, the Pasta Transpiler shall `crate::pasta::call` や `crate::pasta::jump` を出力しない
4. When 短縮形が使用可能, the Pasta Transpiler shall モジュール冒頭の `use pasta::*;` により短縮形が有効であることを前提とする

#### 現在のコード（修正対象）

**現在の出力**:
```rune
for a in crate::pasta::call(ctx, "自己紹介", #{}, []) { yield a; }
for a in crate::pasta::jump(ctx, "会話分岐", #{}, []) { yield a; }
```

#### 修正後のコード

**新しい出力**:
```rune
for a in call(ctx, "自己紹介", #{}, []) { yield a; }
for a in jump(ctx, "会話分岐", #{}, []) { yield a; }
```

**注記**: 
- `use pasta::*;` により `call` と `jump` が直接使用可能
- トランスパイラーはパス修飾を省略してシンプルな関数呼び出しを生成
- 可読性が向上し、生成コードがより自然な Rune コードに近づく

---

### Requirement 5: テスト出力の検証

**Objective:** 開発者として、トランスパイラーが正しいコードを生成し、Rune VMでコンパイルが成功することを確認する。

#### Acceptance Criteria

1. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `ctx.actor = さくら;` が含まれることを検証する（文字列リテラルでないこと）
2. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `yield Actor(ctx.actor.name);` が含まれることを検証する
3. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `use pasta::*;` が含まれることを検証する
4. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `call(ctx, ...)` や `jump(ctx, ...)` が含まれることを検証する（`crate::pasta::` プレフィックスなし）
5. When Rune VMコンパイルテストを実行する, the Test Suite shall `main.rn` と統合してコンパイルが成功することを確認する
6. When 生成コードを検証する, the Test Suite shall ダブルクォートで囲まれたアクター名（`"さくら"`）が存在しないことを確認する
7. When 全テストを実行する, the Test Suite shall 既存の単体・統合テストが全てパスすることを確認する

---

## Technical Context

### 現在の実装

**ファイル**: `crates/pasta/src/transpiler/mod.rs`

**関数**: `transpile_statement_to_writer()` (334-424行目)

**修正箇所**: Statement::Speech の処理（346-362行目）

```rust
Statement::Speech {
    speaker,
    content,
    span: _,
} => {
    // Generate speaker change (store as string)
    writeln!(writer, "        ctx.actor = \"{}\";", speaker)  // ← 修正1
        .map_err(|e| PastaError::io_error(e.to_string()))?;
    writeln!(writer, "        yield Actor(\"{}\");", speaker)  // ← 修正2
        .map_err(|e| PastaError::io_error(e.to_string()))?;

    // Generate talk content
    for part in content {
        Self::transpile_speech_part_to_writer(writer, part, &context)?;
    }
}
```

**追加箇所**: `transpile_global_label()` (276行目直後)

グローバルラベル生成時に3つの use 文を出力する。ローカルラベルは同じモジュール内の関数なので、use 文は不要（モジュールレベルのuse文を継承）。

### 修正戦略

**新しい形式への即座の移行**

本仕様では、文字列リテラルによるアクター代入を完全に廃止し、変数参照形式のみをサポートします。

**採用理由**:
- 文字列形式は設計ミスであり、維持する理由がない
- 型安全性と拡張性を確保するために必須の変更
- 早期に修正することで技術的負債を回避
- 参照実装との完全な一致
- テスト更新のコストは限定的

**移行方針**:
- 古い形式を前提とした全てのテストを新形式に書き換え
- フィクスチャファイルを新形式で再生成
- 検証パターンを新しい出力に合わせて修正
- 後方互換性のためのフラグや条件分岐は一切追加しない

### 影響範囲

| コンポーネント | 影響 | 対応 |
|--------------|------|------|
| `transpile_statement_to_writer()` | 修正必要 | Statement::Speech の処理を変更 |
| `transpile_global_label()` | 修正必要 | 3つの use 文の生成を追加（276行目直後） |
| `transpile_local_label()` | 影響なし | モジュール内関数のためuse文不要 |
| `main.rn` (テストフィクスチャ) | 更新必要 | アクター定義を `pub mod actors { ... }` に移動 |
| `comprehensive_control_flow.rn` | 更新必要 | アクター定義を `pub mod actors { ... }` に移動 |
| `comprehensive_control_flow.transpiled.rn` | 更新必要 | トランスパイラーで再生成 |
| テスト検証パターン | 更新必要 | 文字列リテラル検証を変数参照検証に変更 |
| Pass 2 出力 | 影響なし | `__pasta_trans2__`, `pasta` は不変 |

---

## Testing Strategy

### Unit Tests

| テストケース | 入力 | 期待される出力 |
|-------------|------|----------------|
| **アクター変数参照** | `さくら：こんにちは` | `ctx.actor = さくら;` |
| **Actorイベント生成** | 同上 | `yield Actor(ctx.actor.name);` |
| **use文生成** | 任意のラベル | `use pasta::*;`, `use pasta_stdlib::*;`, `use crate::actors::*;` |
| **call短縮形** | ラベル呼び出し | `call(ctx, "ラベル", #{}, [])` （`crate::pasta::` なし）|
| **jump短縮形** | ラベルジャンプ | `jump(ctx, "ラベル", #{}, [])` （`crate::pasta::` なし）|

### Integration Tests

1. **Rune VMコンパイル検証**:
   - `comprehensive_control_flow.pasta` のトランスパイル
   - `main.rn` と統合してRune VMコンパイル
   - アクター変数の解決確認

2. **参照実装との一致**:
   - 生成コードと `comprehensive_control_flow.rn` の構造比較
   - アクター代入形式の一致確認

3. **既存テストの継続**:
   - 268テスト全てがパス
   - Pass 2 出力の不変性確認

---

## Implementation Notes

### 実装対象

本仕様では新しい形式のみに対応します。古い形式（文字列リテラルでのアクター代入）を前提とした既存テストは全て書き換え対象です。

1. **Statement::Speech 処理の修正**
   - 文字列リテラルから変数参照に変更
   - Actor イベント生成の修正

2. **use 文生成の追加**
   - グローバルラベルモジュール冒頭への3つの use 文出力（`use pasta::*;`, `use pasta_stdlib::*;`, `use crate::actors::*;`）
   - ローカルラベル関数は use 文不要（モジュールレベルのuse文を継承）

3. **pasta関数のパス修飾削除**
   - `crate::pasta::call` → `call` に短縮
   - `crate::pasta::jump` → `jump` に短縮

4. **テストおよびフィクスチャの更新**
   - `comprehensive_control_flow.transpiled.rn` を新形式で再生成
   - `main.rn` のアクター定義を `pub mod actors { ... }` 構造に変更
   - 検証パターンを新しい出力に合わせて修正
   - Rune VMコンパイル検証の追加

### コード変更見積もり

- **修正箇所**: 5箇所
  - Statement::Speech 処理（353, 355行目）: 2行修正
  - Statement::Call 処理（375行目）: 1行修正
  - Statement::Jump 処理（390行目）: 1行修正
  - transpile_global_label（276行目直後）: 3行追加
- **追加行数**: 3行（use 文生成）
- **修正行数**: 4行（文字列リテラル削除、パス修飾削除）
- **テスト更新**: 約10テストケース

---

## Dependencies

| 依存仕様/コンポーネント | 理由 | 状態 |
|---------------------|------|------|
| `pasta-transpiler-pass2-output` | Pass 2 出力は変更しない | ✅ Completed |
| `main.rn` (手動作成) | アクター定義が必要 | 既存 |
| `LabelRegistry` | ラベル情報管理 | 既存、変更不要 |

---

## Namespace Organization: 決定事項（2025-12-14）

**名前空間の整理方針**:

#### 全体の名前空間構造
```rune
pub mod __pasta_trans1__;  // トランスパイラー出力（1pass目のラベル情報）
pub mod __pasta_trans2__;  // トランスパイラー出力（2pass目のラベルセレクター）
pub mod pasta;             // トランスパイラーが呼び出すランタイム関数
pub mod pasta_stdlib;      // Pastaエンジンが公開するRust関数
pub mod actors;            // main.rnで定義されるアクター情報
```

#### アクター定義の配置（main.rn）
```rune
pub mod actors {
    pub const さくら = #{
        name: "さくら",
        id: "sakura",
    };

    pub const うにゅう = #{
        name: "うにゅう",
        id: "unyuu",
    };

    pub const ななこ = #{
        name: "ななこ",
        id: "nanako",
    };
}

pub fn main() {
    // Entry point (not used in tests)
}
```

### 本仕様での対応範囲

本仕様で実装する項目:
1. アクター変数参照の生成（`ctx.actor = さくら;`）
2. Actor イベント生成（`yield Actor(ctx.actor.name);`）
3. 3つのuse文の生成（各モジュール冒頭）
4. pasta関数の短縮形呼び出し（パス修飾削除）
5. テストフィクスチャの更新

### 採用理由:
- `use crate::actors::*;` により、アクターの個別列挙・存在判定が不要
- `use pasta::*;` と `use pasta_stdlib::*;` により、関数呼び出しが簡潔化
- 生成コードの可読性が向上し、自然な Rune コードに近づく
- 将来的な拡張（アクター属性追加など）が容易

---

## Future Work

- **アクター定義の自動生成**: スクリプト解析からアクターリストを抽出し、`main.rn` のテンプレートを生成
- **アクター属性の拡張**: 画像パス、声優情報、表示名（ローカライズ）などの追加
- **動的アクター登録**: 実行時のアクター追加機能
- **名前空間の再設計**: アクターモジュール化、pasta関数の標準ライブラリ統合

---

## References

- **発見元**: `.kiro/specs/pasta-transpiler-pass2-output/KNOWN-ISSUES.md`
- **参照実装**: `crates/pasta/tests/fixtures/comprehensive_control_flow.rn` (1-12行目: アクター定義、14-36行目: 使用例)
- **現在の実装**: `crates/pasta/src/transpiler/mod.rs` (346-362行目: Statement::Speech 処理)
- **関連仕様**: `pasta-transpiler-pass2-output` (Pass 2 出力修正、完了済み)
