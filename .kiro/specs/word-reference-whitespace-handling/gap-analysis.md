# ギャップ分析レポート（更新版）

## 分析対象
- **仕様名**: word-reference-whitespace-handling
- **分析日**: 2025-12-24
- **分析種別**: 再調査（パーサー変更後）

---

## エグゼクティブサマリー

**結論: 本仕様で報告されたバグは既に解決済みです。**

- パーサー文法（`grammar.pest`）の`word_ref`と`var_ref`ルールには既に空白スキップ（`~ s`）が組み込まれています
- 実測テストにより、単語参照（`@場所　`）および変数参照（`$名前　`）直後の空白が正しく除去されていることを確認
- トランスパイル結果も要件書の「正しい出力」と完全に一致
- **追加の実装作業は不要**

### 主要発見事項
1. 文法ルール`word_ref = { word_marker ~ id ~ s}`の末尾`~ s`が空白を自動消費
2. 同様に`var_ref_local`と`var_ref_global`も末尾`~ s`で空白処理済み
3. `s = _{ space_chars* }`はサイレントルールとして空白をキャプチャせずに消費

---

## 1. 現状調査結果

### 1.1 パーサー文法（grammar.pest）の変更点

#### 現在の単語参照（word_ref）定義 - Line 154
```pest
word_ref = { word_marker ~ id ~ s}
```

#### 現在の変数参照（var_ref）定義 - Lines 79-81
```pest
var_ref        =_{ var_ref_global | var_ref_local }
var_ref_local  = { var_marker                 ~ id ~ s }
var_ref_global = { var_marker ~ global_marker ~ id ~ s }
```

#### 空白定義 - Lines 1-6
```pest
space_chars = _{ " " | "\t" | "\u{3000}" | "\u{00A0}" | ... }  // 半角・全角・タブ等
ws          = @{ space_chars+ }
s           = _{ space_chars* }  // 0個以上の空白（サイレントルール）
```

**重要**: `~ s` はPestの「サイレントルール（`_`接頭辞）」で定義された空白マッチャーです。これにより：
- `word_ref`と`var_ref`は参照名の後の空白を**自動的に消費**
- 消費された空白はASTに含まれない（`_`接頭辞によりトークンは非キャプチャ）

### 1.2 以前の問題点との比較

**以前のgap-analysis.mdでの認識**:
- `speech_content`ルールで`text_part`が貪欲マッチし空白を含む
- Option B（Rustパーサー層での後処理）を推奨

**現在の実態**:
- `grammar.pest`が更新され、`word_ref`と`var_ref`の末尾に`~ s`が追加済み
- パーサー層での後処理は**不要**

### 1.3 実測テスト結果

#### テスト1: 単語参照後の全角空白

**入力**:
```pasta
*天気
  @場所：東京　大阪
  さくら：@場所　の天気は@天気　だね。
```

**パース結果**（`cargo run --example test_whitespace_parsing`）:
```rust
ActionLine { 
    actor: "さくら", 
    actions: [
        WordRef("場所"), 
        Talk("の天気は"),   // ← 空白なし ✅
        WordRef("天気"), 
        Talk("だね。")      // ← 空白なし ✅
    ]
}
```

**トランスパイル結果**:
```rune
yield Talk(pasta_stdlib::word("", "場所", []));
yield Talk("の天気は");      // ← 要件書の「正しい」出力と一致 ✅
yield Talk(pasta_stdlib::word("", "天気", []));
yield Talk("だね。");        // ← 要件書の「正しい」出力と一致 ✅
```

#### テスト2: 変数参照後の全角空白

**入力**:
```pasta
*挨拶
  太郎：＄名前　さんこんにちは
```

**パース結果**:
```rust
ActionLine { 
    actor: "太郎", 
    actions: [
        VarRef { name: "名前", scope: Local }, 
        Talk("さんこんにちは")  // ← 空白なし ✅
    ]
}
```

### 1.4 テストスイート結果

```bash
$ cargo test --all
# 結果: 133+ tests passed, 0 failed
```

---

## 2. 要件との照合

| 要件ID   | 要件内容                           | 現状                       | 判定   |
| -------- | ---------------------------------- | -------------------------- | ------ |
| R1-AC1   | 単語参照直後の空白をスキップ       | `word_ref ~ s`で対応済み   | ✅ 満足 |
| R1-AC2   | 複数空白の連続スキップ             | `s = space_chars*`で対応   | ✅ 満足 |
| R1-AC3   | 空白除去後の空テキストパート不生成 | Pestパーサーの自然な動作   | ✅ 満足 |
| R1-AC4   | 行末空白の正しい処理               | `~ s`が処理                | ✅ 満足 |
| R1-AC5   | 参照前の空白は保持                 | `talk`ルールで別途処理     | ✅ 満足 |
| R1-AC6   | さくらスクリプト後の空白は保持     | `sakura_script`に`~ s`なし | ✅ 満足 |
| R2-AC1-4 | トランスパイラの正確性             | 上記テスト結果             | ✅ 満足 |
| R3-AC1-4 | テスト検証                         | 既存テストスイート全パス   | ✅ 満足 |
| R4-AC1-4 | 文法仕様整合性                     | SPECIFICATION.mdと一致     | ✅ 満足 |

---

## 3. 実装アプローチ分析

### Option A: 何もしない（現状維持） ✅ **推奨**

**根拠**:
- バグは既にパーサー文法レベルで解決済み
- 全テストがパス（`cargo test --all` = 133+ tests passed）
- 追加実装は不要

**トレードオフ**:
- ✅ リスクゼロ
- ✅ コスト0
- ❌ 仕様書作成の工数が無駄になった可能性

### Option B: ドキュメント整備のみ（オプション）

**内容**:
- 要件書を「解決済み」としてアーカイブ
- GRAMMAR.mdまたはSPECIFICATION.mdに空白処理の明示的記述を追加

**トレードオフ**:
- ✅ 将来の開発者への明確なドキュメンテーション
- ❌ 小規模の工数

### Option C: 回帰テスト追加（オプション）

**内容**:
- 空白処理に特化したテストケースを`tests/parser2_integration_test.rs`に追加
- 今後の文法変更による回帰を防止

**サンプルテスト**:
```rust
#[test]
fn test_word_ref_whitespace_handling() {
    let source = "＊テスト\n  さくら：@単語　の後\n";
    let file = parse_str(source, "test.pasta").unwrap();
    // ... Talk("の後") に空白が含まれないことを検証
}
```

**トレードオフ**:
- ✅ 回帰防止
- ❌ 既存動作のテストなので価値は限定的

---

## 4. 推定工数とリスク

| オプション          | 工数  | リスク | 推奨度 |
| ------------------- | ----- | ------ | ------ |
| A: 現状維持         | 0日   | なし   | ⭐⭐⭐    |
| B: ドキュメント整備 | 0.5日 | なし   | ⭐⭐     |
| C: 回帰テスト追加   | 0.5日 | なし   | ⭐      |

---

## 5. 結論と推奨アクション

### 推奨: Option A（現状維持）

**理由**:
1. 報告されたバグは**過去の文法更新で既に解決済み**
2. 文法定義の`~ s`パターンにより、単語参照・変数参照後の空白は自動的に消費される
3. 全テストスイートがパスしており、既存機能に問題なし

### 次のステップ

1. **この仕様をクローズ**: `.kiro/specs/word-reference-whitespace-handling/`を`completed/`に移動
2. **spec.jsonを更新**: `"phase": "resolved-no-action-needed"`に変更
3. **オプション**: 将来の参考のためにこのgap-analysis.mdを保持

---

## 付録: 調査したファイル

| ファイル                                          | 行番号   | 内容                                 |
| ------------------------------------------------- | -------- | ------------------------------------ |
| `src/parser/grammar.pest`                         | L1-6     | 空白定義（`s`, `ws`, `space_chars`） |
| `src/parser/grammar.pest`                         | L79-81   | `var_ref`定義（`~ s`あり）           |
| `src/parser/grammar.pest`                         | L154     | `word_ref`定義（`~ s`あり）          |
| `src/parser/grammar.pest`                         | L151-160 | `action`および`talk`ルール           |
| `src/parser/mod.rs`                               | L600-670 | `parse_actions`関数                  |
| `src/transpiler/code_generator.rs`                | L240-260 | Action処理                           |
| `tests/fixtures/comprehensive_control_flow.pasta` | L48      | 問題ケース（空白なしで記載）         |

---

## 変更履歴

| 日付       | 内容                                                 |
| ---------- | ---------------------------------------------------- |
| 2025-12-21 | 初回gap-analysis作成（Option B推奨）                 |
| 2025-12-24 | 再調査によりバグ解決済みを確認（Option A推奨に変更） |
