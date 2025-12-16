# Known Issues: pasta-transpiler-pass2-output

## 🐛 Issue 1: アクター代入の不一致（Pass 1の問題）

### 概要
トランスパイラーが生成するアクター代入が、参照実装と異なる。

### 現状

**参照実装（comprehensive_control_flow.rn）:**
```rune
pub const さくら = #{ name: "さくら", id: "sakura" };

pub fn __start__(ctx) {
    ctx.actor = さくら;  // 変数参照（オブジェクト）
}
```

**トランスパイラー出力（comprehensive_control_flow.transpiled.rn）:**
```rune
pub fn __start__(ctx, args) {
    ctx.actor = "さくら";  // 文字列リテラル
}
```

### 問題点

1. **型の不一致**: 
   - 参照実装: オブジェクト `#{ name: "さくら", id: "sakura" }`
   - トランスパイラー: 文字列 `"さくら"`

2. **設計意図との乖離**:
   - アクターは構造化データ（name, id, その他属性）を持つべき
   - 文字列では拡張性がない

3. **Rune VMコンパイルは成功**:
   - 両方とも文法的に有効
   - 実行時の挙動が異なる可能性

### 原因

**ソースコード**: `crates/pasta/src/transpiler/mod.rs:353`

```rust
// Generate speaker change (store as string)
writeln!(writer, "        ctx.actor = \"{}\";", speaker)
```

コメントに "store as string" と明記されており、**意図的な実装**。

### 影響範囲

- **Pass 1 の出力** (本仕様の範囲外)
- **Pass 2 の出力には影響なし** (本仕様で修正した部分は正常)

### 本仕様での対応

**対応なし（スコープ外）**

理由:
1. 本仕様は **Pass 2 出力（`__pasta_trans2__` + `pasta` モジュール）の修正**が目的
2. アクター代入は **Pass 1 の Statement::Speech 処理**で生成される
3. Pass 1 の修正は別仕様で対応すべき

### 検証結果への影響

**影響なし**

理由:
1. Rune VMコンパイルは成功している（文法的に有効）
2. 本仕様の要件（Pass 2 出力の修正）は全て達成済み
3. アクター代入は Pass 1 の責務

### 推奨される対応

**新規仕様として対応**

提案: `pasta-transpiler-actor-variables` 仕様

要件:
1. `main.rn` のアクター定義を参照するようにトランスパイラーを修正
2. `ctx.actor = "さくら"` → `ctx.actor = さくら` に変更
3. `yield Actor("さくら")` → `yield Actor(さくら)` に変更
4. Pass 1 の `Statement::Speech` 処理を修正

優先度: P2（機能拡張）

---

## 📝 記録日時

- **発見日**: 2025-12-14T06:51:57Z
- **記録日**: 2025-12-14T06:55:00Z
- **報告者**: User
- **確認者**: AI-DLC System

---

## ✅ 本仕様への影響

**影響なし - Pass 2 出力は正常**

本仕様（pasta-transpiler-pass2-output）で実装した部分:
- ✅ `pub mod __pasta_trans2__` - 正常
- ✅ `pub fn label_selector(label, filters)` - 正常
- ✅ `pub mod pasta` - 正常
- ✅ Rune VMコンパイル - 成功

アクター代入は Pass 1 で生成されるため、本仕様のスコープ外。
