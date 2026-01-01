# Implementation Gap Analysis: word-ref-ast-support

## Analysis Summary

**スコープ**: Pasta DSL文法における式内単語参照（`@word`）のAST対応

**現在地**: grammar.pestにはword_refが追加されているが、ASTと式パーサーが未対応

**主要な課題**:
1. **Phase 1（pasta_core）**: Expr enumにWordRefバリアントを追加、try_parse_expr関数を拡張
2. **Phase 2（トランスパイラー）**: pasta_rune/luaの2つのCodeGeneratorで網羅的matchに対応

**推奨**: Phase 1のみ完全実装、Phase 2は最小実装（stub）で後続フェーズへ繰越

---

## 1. Current State Investigation

### 1.1 既存アーキテクチャ

**pasta_core (言語非依存層)**
- `src/parser/ast.rs`: AST型定義（Expr enum）
- `src/parser/mod.rs`: Pestベースのパーサー実装（try_parse_expr関数）
- `src/parser/grammar.pest`: PEG文法定義

**pasta_rune (Runeバックエンド)**
- `src/transpiler/code_generator.rs`: Expr型のmatch処理（L300-348）で全バリアント処理
- 網羅的match: `Expr::Integer`, `Expr::Float`, `Expr::String`, `Expr::BlankString`, `Expr::VarRef`, `Expr::FnCall`, `Expr::Paren`, `Expr::Binary`

**pasta_lua (Luaバックエンド)**
- `src/code_generator.rs`: Expr型のmatch処理が2箇所（generate_expr関数）
- L444-498: 主要な式生成ロジック
- L513-562: 追加の式処理ロジック

### 1.2 既存パターン

**Action::WordRef（既存の単語参照）**
- アクション行（`actor：@word テキスト`）専用
- `src/parser/mod.rs` L700: Rule::word_refをActionコンテキストで処理
- 既にAction enumに `WordRef { name: String, span: Span }` が存在

**Expr型の現在のバリアント**
- Integer, Float, String, BlankString: リテラル
- VarRef: 変数参照（`$var`）
- FnCall: 関数呼び出し（`@func()`）
- Paren: 括弧式
- Binary: 二項演算

### 1.3 依存関係・制約

- grammar.pest: L72で`word_ref`が`term`に追加済み
- Pest 2.8: 自動的に`Rule::word_ref`を生成
- トランスパイラー層: 網羅的matchパターンのため、Expr追加時に全て対応必須

---

## 2. Requirements Feasibility Analysis

### 2.1 Phase 1 Technical Needs

| 要件 | 技術的実装内容 | 難易度 | 状態 |
|------|----------------|--------|------|
| Req 1: Expr::WordRef追加 | `pub enum Expr`に`WordRef { name: String }`バリアント追加 | 低 | 既存 |
| Req 2: try_parse_expr対応 | `Rule::word_ref`マッチアームを追加、idを抽出 | 低 | 既存 |
| Req 3: Action vs Expr共存 | コンテキスト分離は既に実装済み（parser L700がアクション行用） | 低 | 既存 |
| Req 5: パーステスト | pasta_core層のみのテストで完結可能 | 低 | 既存 |

**Gap**: 全て実装可能。既存パターンに従うのみ。

### 2.2 Phase 2 Technical Needs

| 要件 | 技術的実装内容 | 難易度 | 状態 |
|------|----------------|--------|------|
| Req 4.1: Rune最小実装 | `Expr::WordRef`matchアーム追加（stub: unimplemented! or todo!） | 低 | 未実装 |
| Req 4.2: Lua最小実装 | `Expr::WordRef`matchアーム追加（2箇所、stub） | 低 | 未実装 |
| 既存テスト保証 | Word_refを含まないテストは変更なしで成功 | 中 | 未検証 |

**Gap**: matchアームが不足（コンパイルエラー）。stub実装で回避可能。

### 2.3 既存テスト影響分析

- **parser2_integration_test.rs**: Expr型を使用、Matchアームが必須
- **pasta_transpiler2_*.rs**: CodeGeneratorのgenerate_expr呼び出し
- **pasta_engine_*.rs**: 統合テスト、Expr構築コード

**予想**: Expr::WordRefを追加するまでコンパイルエラーが発生。Phase 2の最小実装で解決。

---

## 3. Implementation Approach Options

### Option A: 完全実装（Phase 1+2同時）

**説明**: AST定義 + パーサー + トランスパイラーを一度に完成

**メリット**:
- ✅ 1回で完全機能完成
- ✅ 統合テスト可能
- ✅ E2Eで動作検証可能

**デメリット**:
- ❌ スコープが大きい（詳細な実装設計が必要）
- ❌ トランスパイラー2つの対応が必要

**推定工数**: 2-3日（M）

**リスク**: 中（トランスパイラー仕様の理解が必要）

---

### Option B: 段階的実装（推奨）

**Phase 1のみ実装**:
- Expr::WordRefバリアント追加
- try_parse_exprで`Rule::word_ref`処理
- パーステストの追加

**Phase 2は最小実装（スタブ）**:
- pasta_rune CodeGenerator: `Expr::WordRef => { /* stub */ }`
- pasta_lua CodeGenerator: `Expr::WordRef => { /* stub */ }` (2箇所)
- `unimplemented!()` または `todo!()` マクロで一時対応

**メリット**:
- ✅ Phase 1は高速実装（pasta_core層のみ）
- ✅ 既存テスト成功保証（スタブで網羅的match対応）
- ✅ Phase 1の成果物を明確化
- ✅ Phase 2の仕様を後で詳細化可能

**デメリット**:
- ❌ Phase 2実装までE2Eテスト不可
- ❌ スタブコードが一時的に存在

**推定工数**:
- Phase 1: 1日（S）
- Phase 2: 0.5日（S）

**リスク**: 低（パーサー層は既存パターン従順）

---

### Option C: Exprのみ対応（Transpilerは据え置き）

**説明**: Phase 1のみ。Phase 2はスキップして次の大型フェーズで対応

**メリット**:
- ✅ Phase 1最速実装
- ✅ 仕様書のみで実装完結

**デメリット**:
- ❌ grammar.pestが有効にならない（E2Eで動作未検証）
- ❌ 仕様の価値が限定的

**推定工数**: 0.5日（S）

**リスク**: 低（但し完成度低い）

---

## 4. Requirement-to-Asset Map

### Phase 1 (pasta_core)

| Requirement | 対象ファイル | 実装パターン | 難易度 | 状態 |
|-------------|------------|-----------|--------|------|
| Req 1: Expr::WordRef追加 | `ast.rs` L635 | enum variant追加 | S | Gap: 実装なし |
| Req 2: try_parse_expr対応 | `mod.rs` L925 | match arm + id抽出 | S | Gap: 実装なし |
| Req 3: Action共存 | `mod.rs` L700 | 既存のままで対応 | N/A | OK: 既実装 |
| Req 5: パーステスト | `tests/` | パーステスト追加 | S | Gap: テスト不足 |

### Phase 2 (トランスパイラー)

| Requirement | 対象ファイル | 実装パターン | 難易度 | 状態 |
|-------------|------------|-----------|--------|------|
| Req 4: Rune最小実装 | `pasta_rune/src/transpiler/code_generator.rs` L300 | match arm (stub) | S | Gap: arm不足 |
| Req 4: Lua最小実装 | `pasta_lua/src/code_generator.rs` L444, L513 | match arm (stub) x2 | S | Gap: arm不足 |
| 既存テスト保証 | `tests/` | スタブで網羅 | S | Constraint: 必須 |

---

## 5. Implementation Complexity & Risk Assessment

### Phase 1: AST + Parser

**Effort: S (1日)**
- Expr enum: 1行追加 + derive自動
- try_parse_expr: 5-10行の新規matchアーム
- テスト: 3-5テストケース追加

**Risk: 低**
- 既存パターン完全準拠（Action::WordRefが先例）
- 文法定義既に完成（grammar.pestで`word_ref`定義済み）
- 依存関係シンプル

**理由**: Pest自動生成、既知パターン

---

### Phase 2: Transpiler Stubs

**Effort: S (0.5日)**
- pasta_rune: 1 matchアーム
- pasta_lua: 2 matchアーム
- 計3つのスタブ実装

**Risk: 低**
- スタブは `unimplemented!()`/`todo!()` で十分
- 既存テストは word_ref 未使用のため影響なし
- 後続フェーズで詳細実装

**理由**: 最小変更で網羅的match対応

---

## 6. Design Phase Recommendations

### Preferred Approach: Option B（段階的実装）

**理由**:
1. Phase 1の成果物を明確化できる
2. Phase 2の詳細設計時間を確保できる
3. 既存テスト成功保証が早期に得られる
4. リスク最小化

### Phase 1 の設計焦点

1. **Expr::WordRef定義**: `{ name: String }` のみ（span不要 / 後付可能）
2. **try_parse_expr実装**: 既存VarRef/FnCallパターンを参考に`Rule::word_ref`マッチアームを追加
3. **テスト設計**:
   - パーステスト: `＄x＝＠word`、`＠func(＠arg)`、`＠word ＋ 10` など
   - 既存テスト回帰なし（word_refは新文法のため既存テストに未出現）

### Phase 2 の設計焦点（次フェーズ）

1. **Rune実装**: word定義の取得ロジック（WordDefRegistry連携）
2. **Lua実装**: 同様のword定義取得ロジック
3. **E2Eテスト**: 実際にword_refを含むPastaスクリプトを実行

### Research Needed

- **Phase 2 Rune/Lua実装**: Word定義検索API（WordDefRegistry）の使用方法確認
- **Runtime**: 単語値がexpr内で使用される際の型変換規則（数値/文字列など）

---

## 7. Conclusion

**ギャップの大きさ**: 小（新規バリアント + matchアームのみ）

**実装難易度**: 低（既存パターン従順）

**推奨路線**: **Option B（段階的実装）**
- Phase 1で pasta_core を完成
- Phase 2で stub 実装でコンパイル通過
- 次フェーズで完全実装

**次アクション**: 
1. `/kiro-spec-design word-ref-ast-support` で設計ドキュメント生成
2. Phase 1の詳細実装タスク分解へ
