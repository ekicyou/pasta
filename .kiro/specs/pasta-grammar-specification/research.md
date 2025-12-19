# Research & Design Decisions

## Summary
- **Feature**: `pasta-grammar-specification`
- **Discovery Scope**: Full（既存パーサー・トランスパイラーの破壊的変更を伴う拡張 + 外部仕様調査）
- **Authoritative Specification**: [grammar-specification.md](grammar-specification.md)（正規仕様）
- **Key Findings**:
  1. Sakura スクリプトは「字句のみ認識、非解釈」で確定。半角 `\` + ASCII トークン + 非ネスト `[...]`（`\]` エスケープ許容）— 仕様 7.2-7.4
  2. Jump マーカー（`？`）は廃止、Call（`＞`）へ統一 — 仕様 2.4
  3. 全角文字（`＼` `［］`）は pest 定義から完全削除 — 仕様 7.4
  4. **ukadoc 公式仕様検証完了**: エスケープ規則・コマンド体系の正確な仕様を確認
  5. **text_part バグ発見**: `＄` が除外されておらず変数参照が吸収される — 仕様 6.3

---

## Research Log

### 1. ukadoc さくらスクリプト仕様（フルリサーチ）

#### 1.1 エスケープ規則
- **Source**: https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#notes_escape
- **Official Specification**:
  > - 「\」をさくらスクリプトの開始記号でなく単に表示したい場合は「\\」とする
  > - 同様に環境変数埋め込みタグの「%」は「\%」と書く
  > - スクウェアブラケット内に引数を持つタグ（\q[ラベル名,ID]など）内でのみ、「]」は「\]」と書ける
  > - 複数引数を持つタグの第２引数以降で「,」を含めたい場合、全体を""で囲む
  > - 同様に「"」を含めるには""を二重にする

- **Design Implications**:
  - `\\` はバックスラッシュリテラル出力（Pasta では `\` は Sakura エスケープ開始なので透過）
  - `\]` は角括弧内のみ有効 → Pasta パーサーは `[...]` 内容を非解釈で透過し、`\]` もそのまま透過
  - `"..."` や `""` の引用は Sakura 側の処理 → Pasta パーサーは関与しない

#### 1.2 コマンド体系（網羅的分類）
- **Source**: https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html

| カテゴリ | 代表例 | パターン |
|---------|--------|----------|
| スコープ | `\0`, `\1`, `\h`, `\u`, `\p[ID]` | 単一文字 / 文字+数字 / 文字+`[...]` |
| サーフェス | `\s0`, `\s[ID]`, `\i[ID,wait]` | 文字+数字 / 文字+`[...]` |
| バルーン | `\n`, `\n[half]`, `\c`, `\_l[x,y]` | 単一文字 / アンダースコア+文字+`[...]` |
| ウェイト | `\w9`, `\_w[時間]`, `\x` | 文字+数字 / アンダースコア+文字+`[...]` |
| 選択肢 | `\q[...]`, `\_a[...]`, `\__q[...]` | 文字+`[...]` |
| イベント | `\e`, `\![raise,...]`, `\![embed,...]` | 単一文字 / `!`+`[...]` |
| 未分類 | `\_u[0x0000]`, `\_m[0x00]`, `\&[ID]` | アンダースコア+文字+`[...]` |

- **Key Pattern Observations**:
  1. **全コマンドは半角 ASCII のみ** — 全角定義なし
  2. パターン: `\` + (文字|数字|`_`|`!`) + オプション数字 + オプション`[...]`
  3. ブラケット `[...]` は非ネスト（ネストはさくらスクリプト側で解釈）

#### 1.3 ランタイム寿命（Sakura Script Lifetime）
- **Source**: ukadoc
- **Specification**: 「基本的に、スクリプト終了（\eの実行時）まで」
- **Implication**: Pasta Runtime が Sakura スクリプトを透過する設計と整合

### 2. 現在の pest 実装の詳細分析

#### 2.1 pasta.pest 構造（329行）
- **現状の sakura_command ルール**:
```pest
sakura_command = @{
    // Pattern 1: Underscore + letters + optional brackets: \_w[50]
    sakura_underscore ~ sakura_letter+ ~ (sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close)?  |
    
    // Pattern 2: ! or letter(s) + brackets: \s[0], \custom[arg], \![event]
    ("!" | "！" | sakura_letter+) ~ sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close  |
    
    // Pattern 3: Single letter + digits: \w8
    sakura_letter ~ sakura_digit+ ~ !sakura_letter  |
    
    // Pattern 4: Single letter: \n
    sakura_letter  |
    
    // Pattern 5: Digits only: \0, \1
    sakura_digit+
}
```

- **問題点**:
  1. `sakura_escape = { "\\" | "＼" }` — 全角バックスラッシュを許容（ukadoc に定義なし）
  2. `sakura_bracket_open/close` — 全角括弧を許容（ukadoc に定義なし）
  3. `sakura_letter` に全角英字を含む（ukadoc では ASCII のみ）
  4. **`\]` エスケープ未対応** — ブラケット内で `\]` を許容していない
  5. 5パターンは複雑すぎる — 「非解釈」仕様なら簡素化可能

#### 2.2 AST 型（ast.rs 301行）
- **現状の Statement enum**:
```rust
pub enum Statement {
    Speech { speaker, content, span },
    Call { target, filters, args, span },
    Jump { target, filters, span },  // ← 削除対象
    VarAssign { name, scope, value, span },
    RuneBlock { content, span },
}
```

- **JumpTarget enum**: Call/Jump 共用 → Jump 削除後も Call で使用可能

#### 2.3 Transpiler（mod.rs 905行）
- **Statement::Jump 分岐**（現状）:
```rust
Statement::Jump { target, filters, span: _ } => {
    let search_key = Self::transpile_jump_target_to_search_key(target);
    let filters_str = Self::transpile_attributes_to_map(filters);
    writeln!(writer, "for a in crate::pasta::jump(ctx, \"{}\", {}, []) {{ yield a; }}", search_key, filters_str)?;
}
```

- **pasta::jump() 関数**: 実質的に `pasta::call()` と同一実装
- **削除の影響**: Transpiler の match 分岐削除のみ（型変更波及は minimal）

### 3. 既存テスト構造分析

#### 3.1 テストファイル分布（38ファイル）
| カテゴリ | ファイル数 | 代表例 |
|---------|-----------|--------|
| Parser 系 | 12 | `parser_tests.rs`, `pest_sakura_test.rs` |
| Transpiler 系 | 8 | `two_pass_transpiler_test.rs`, `transpile_comprehensive_test.rs` |
| Engine/Runtime 系 | 10 | `engine_integration_test.rs`, `persistence_test.rs` |
| その他 | 8 | デバッグ、診断用 |

#### 3.2 Jump 使用箇所の調査
- **grep "？" tests/**: 複数フィクスチャで使用
- **grep "Jump" tests/**: 約 15 箇所（テストケース、コメント）
- **影響範囲**: Phase 3 で一括置換・削除可能

#### 3.3 全角 Sakura 使用箇所
- **grep "＼" tests/**: 使用箇所なし（既に半角統一済み）
- **grep "［" tests/**: 使用箇所なし
- **結論**: Phase 3 のテスト修正は最小限

### 4. Jump と Call のセマンティクス差異
- **Context**: Jump マーカー（`？`）を廃止すべきか、維持すべきか
- **Sources Consulted**: 
  - grammar-specification.md 4章
  - gap-analysis-2025-12-19.md
  - transpiler/mod.rs（既存実装）
- **Findings**:
  - Jump と Call にセマンティクス上の差異なし（同一動作）
  - DSL レベルで区別する必要性がない
  - トランスパイラでは両方とも同じ Rune コード生成
  - `pasta::jump()` と `pasta::call()` は実質同一実装
- **Implications**: Jump 廃止は DSL 整理として妥当。破壊的変更だが MVP 前段階なので許容範囲

### 5. 全角文字サポートの範囲
- **Context**: Sakura エスケープで全角 `＼` `［］` をサポートすべきか
- **Sources Consulted**: 
  - ukadoc（さくらスクリプトエスケープ）
  - grammar-specification.md 11.16
  - pasta.pest（現行実装）
- **Findings**:
  - **ukadoc では全角バックスラッシュのエスケープ定義なし**
  - さくらスクリプトは ASCII コマンド前提
  - 全角括弧 `［］` もコマンド構文として定義なし
  - 現行 pasta.pest では `＼` `［` `］` を定義しているが ukadoc に根拠なし
- **Implications**: 全角文字は pest 定義から削除（Case A 決定）

### 6. テスト層別化と Cargo 互換性
- **Context**: テストファイルリネームが Cargo テスト実行に影響するか
- **Sources Consulted**: 
  - Cargo ドキュメント（tests/ ディレクトリ自動検出）
- **Findings**:
  - Cargo は `tests/**/*.rs` を自動検出・テスト対象化
  - 命名規則（`pasta_parser_*_test.rs` 等）は検出を阻害しない
  - tests/common モジュール参照も影響なし
- **Implications**: Phase 0 テスト層別化は安全に実行可能

### 7. text_part における変数参照の除外漏れ（バグ発見）
- **Context**: アクション行内のインライン要素が正しくパースされるか
- **Sources Consulted**: 
  - grammar-specification.md 6.3（インライン要素）
  - pasta.pest（現行実装）
- **Findings**:
  - 仕様 6.3: インライン要素に `＄var_name`（変数参照）が含まれる
  - 現行 pest: `text_part = @{ (!(at_marker | sakura_escape | NEWLINE) ~ ANY)+ }`
  - `dollar_marker` が除外されていないため、`＄` が text_part に吸収される
- **Implications**: Phase 1 で `dollar_marker` を除外対象に追加（C1）

### 8. GRAMMAR.md と grammar-specification.md の乖離
- **Context**: 2つのドキュメントの関係性と正規性
- **Sources Consulted**: 
  - GRAMMAR.md（ユーザー向け）
  - grammar-specification.md（正規仕様）
- **Findings**:
  - GRAMMAR.md には `//` コメント、`/* */` ブロックコメント、同期セクションの記載あり
  - grammar-specification.md にはこれらの記載なし（コメントは `#` / `＃` のみ、同期セクション未定義）
  - grammar-specification.md が正規仕様、GRAMMAR.md は Phase 3 で同期セクション等を削除
- **Implications**: GRAMMAR.md の記載は正規仕様外の「未実装/廃止機能」として Phase 3 で修正

---

## Architecture Pattern Evaluation

### 段階化戦略比較

| Option | Description | Strengths | Risks / Limitations | Verdict |
|--------|-------------|-----------|---------------------|---------|
| **層単位段階化** | Parser → Transpiler → Runtime/Tests の順で修正 | リグレッション発生箇所特定容易、各層で Green テスト維持可能 | 各フェーズ間で依存関係あり | ✅ **採用** |
| 一括修正 | 全層同時修正 | 修正完了が早い可能性 | リグレッション発生時の原因特定困難 | ❌ 却下 |
| 機能単位段階化 | Sakura → Jump → 全角の順で修正 | 機能ごとに完結 | 同一ファイルを複数回修正、混乱の元 | ❌ 却下 |

### Phase 0-3 アーキテクチャ

```
Phase 0: Test Preparation (Pre-Implementation)
├── test-baseline.log capture (1 task)
├── test file rename to layer convention (1 task)
└── test baseline comparison (1 task)

Phase 1: Parser Layer (pest + AST)
├── pasta.pest modifications
│   ├── Sakura: simplify to non-interpreting pattern
│   ├── Sakura: half-width only (remove ＼ ［ ］)
│   ├── Sakura: add \] escape in brackets
│   └── Jump: remove jump_marker and jump_content rules
└── ast.rs modifications
    └── Remove Statement::Jump variant

Phase 2: Transpiler Layer
├── transpiler/mod.rs modifications
│   ├── Remove Statement::Jump match branch
│   ├── Remove pasta::jump() wrapper generation
│   └── Update TranspileContext if needed
└── Intermediate verification

Phase 3: Runtime/Tests/Documentation
├── tests/ modifications
│   ├── Remove Jump-related test cases
│   └── Update Sakura test cases
├── GRAMMAR.md revision
└── Final verification
```

---

## Design Decisions

### Decision 1: Sakura コマンド簡素化（案 B）
- **Context**: pest 定義の詳細5パターンを維持するか簡素化するか
- **Alternatives Considered**:
  1. 案 A — 詳細5パターン維持、半角・`\]` 対応のみ追加
  2. 案 B — 完全簡素化（ASCIIトークン + 非ネスト `[...]` + `\]` 許容）
- **Selected Approach**: ✅ **案 B（完全簡素化）**
- **Rationale**: 
  - 仕様「Sakura は字句のみ認識、非解釈」に忠実
  - 詳細パターン区別は実装上不要（Rune VM が解釈）
  - 未知トークンを通すことで将来の Sakura Script 拡張に対応
  - ukadoc のコマンド体系は多様（5パターンで網羅困難）
- **Trade-offs**: 既存の詳細パターンテストは削除が必要
- **Follow-up**: Phase 1 で pest ルール修正
- **ukadoc 裏付け**: コマンド数 100+ 、パターンは継続的に拡張される可能性

### Decision 2: Jump マーカー廃止（案 A）
- **Context**: Jump（`？`）を維持するか廃止するか
- **Alternatives Considered**:
  1. 案 A — Jump 廃止、Call（`＞`）へ統一
  2. 案 B — Jump 維持（下位互換性優先）
- **Selected Approach**: ✅ **案 A（Jump 廃止）**
- **Rationale**: 
  - MVP 未達段階での積極的な破壊的変更を許容
  - セマンティクス上の差異なし（同一動作）
  - DSL 整理・保守性向上
  - `pasta::jump()` と `pasta::call()` の実装重複解消
- **Trade-offs**: Jump 依存テスト全削除が必要（影響範囲: 約15箇所）
- **Follow-up**: Phase 1-3 で Jump 関連コード・テスト削除

### Decision 3: 全角文字完全削除（Case A）
- **Context**: Sakura エスケープで全角 `＼` `［］` をサポートするか
- **Alternatives Considered**:
  1. Case A — 全角完全削除（pest で reject）
  2. Case B — 全角許容・非推奨（grammar では半角例のみ）
- **Selected Approach**: ✅ **Case A（全角完全削除）**
- **Rationale**: 
  - **ukadoc に全角エスケープ定義なし**（公式仕様外）
  - さくらスクリプトは ASCII コマンド前提
  - 全角括弧 `［］` もコマンド構文として定義なし
  - テストで既に全角不使用を確認済み
- **Trade-offs**: 既存全角スクリプトは半角への移行必須（影響最小限）
- **Follow-up**: Phase 1 で pest ルール修正
- **ukadoc 裏付け**: "「\」をさくらスクリプトの開始記号でなく..." — 半角のみ記載

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Phase 1 ast.rs 型変更の波及 | 中 | 高 | Transpiler で compiler error として即座に検出 |
| Jump 削除漏れ | 低 | 中 | `grep -r "Jump\|？" src/ tests/` で検索、チェックリスト活用 |
| テスト置換ミス | 低 | 中 | Phase 0 の test-baseline.log と比較 |
| 全角テスト削除漏れ | 極低 | 低 | `grep -r "＼\|［\|］" tests/` で確認（現状使用なし） |
| pest ルール変更による既存パース失敗 | 中 | 高 | Phase 1 完了時に全テスト実行、差分レビュー |
| Sakura 簡素化による未知パターン取りこぼし | 低 | 中 | 「非解釈」設計により未知も透過、リスク最小化 |

---

## Validation Checklist

### Full Discovery 完了確認項目
- [x] 要件文書（requirements.md）レビュー完了
- [x] 既存実装（pasta.pest, ast.rs, transpiler/mod.rs）分析完了
- [x] 外部仕様（ukadoc）調査完了
- [x] 技術的整合性検証完了（Pest 2.8, Rune 0.14 互換性確認）
- [x] アーキテクチャパターン評価完了（層単位段階化採用）
- [x] リスク分析完了（6項目識別、軽減策策定）
- [x] 設計決定3件の根拠明確化

### ukadoc クロスリファレンス
| 設計決定 | ukadoc 根拠 |
|---------|------------|
| 半角 `\` のみ | "「\」をさくらスクリプトの開始記号でなく..." — 全角言及なし |
| `\]` 許容 | "スクウェアブラケット内に引数を持つタグ内でのみ、「]」は「\]」と書ける" |
| コマンド非解釈 | コマンド数 100+ 、詳細解釈は Rune VM 側で実施 |

### grammar-specification.md ギャップ一覧（Phase 1 対象）
| ID | 仕様箇所 | 現状 pest | 問題 | 対応 |
|----|---------|-----------|------|------|
| A1 | 7.2 | `sakura_escape = { "\\" \| "＼" }` | 全角許容 | 半角のみ |
| A2 | 7.3 | 5パターン複雑 | 過剰詳細 | 簡素化 |
| A3 | 7.3 | `(!] ~ ANY)*` | `\]` 非許容 | `\]` 許容 |
| A4 | 7.4 | 全角 `［` `］` 許容 | 仕様外 | 半角のみ |
| A5-A7 | 7.4 | `sakura_letter/digit/underscore` 全角含む | 不要 | 削除 |
| B1-B4 | 2.4 | `jump_marker`, `jump_content` | 仕様外 | 削除 |
| C1 | 6.3 | `text_part` に `＄` 未除外 | バグ | 修正 |

---

## References
- [grammar-specification.md](grammar-specification.md)（**正規仕様**）
- [ukadoc - さくらスクリプトリスト（公式仕様）](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html)
- [ukadoc - さくらスクリプトのエスケープ](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#notes_escape)
- gap-analysis-2025-12-19.md（層別ギャップ分析）
- test-hierarchy-plan.md（テスト層別化計画）

