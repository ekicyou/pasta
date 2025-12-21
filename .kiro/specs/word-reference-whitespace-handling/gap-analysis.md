# ギャップ分析: 単語参照後の空白処理

## 分析概要

### スコープ
Pastaパーサーにおけるインライン参照（`word_expansion`単語参照および`var_ref`変数参照）直後の空白文字処理に関する不具合修正。現在、`＠単語名　テキスト`または`＄変数名　テキスト`のように参照直後に空白が続く場合、空白が`text_part`として分離され、Runeコード生成時に不要な`yield Talk("　...")`が生成される。

**対象要素**:
- `＠単語名`（WordExpansion）: 単語参照
- `＄変数名`（VarRef）: 変数参照

**対象外**:
- さくらスクリプト（`\n`, `\w8`など）: 意味を持つエスケープシーケンスであり、空白処理の対象外

### 課題の核心
- **Pest文法**: `WHITESPACE`が暗黙的（`_`プレフィックス）に定義されており、トークン間で自動的に消費される
- **speech_contentルール**: `text_part | func_call | word_expansion | var_ref | sakura_script`の順序で選択され、`text_part`が貪欲マッチする
- **text_partルール**: `@{ (!(at_marker | dollar_marker | sakura_escape | NEWLINE) ~ ANY)+ }`として定義され、空白文字を含む全文字を捕捉する
- **問題**: `word_expansion`または`var_ref`の後、Pestは暗黙WHITESPACEを消費するが、その後の`text_part`が残りの空白から始まるテキストを再度捕捉してしまう
- **重要**: さくらスクリプト（`sakura_script`）はこの問題の対象外であり、直後の空白を除去してはならない

### 推奨アプローチ
**Option B: Rustパーサー層での後処理** - 開発者確認によりVarRefも対象に含めることが決定したため、最もシンプルなOption Bを採用

---

## 1. 現状調査

### 既存アセットマップ

| アセット | ファイルパス | 役割 | 現状評価 |
|---------|------------|------|---------|
| Pest文法 | `src/parser/pasta.pest` | DSL文法定義 | ✅ 既存 |
| `speech_content`ルール | `pasta.pest:134` | 発言内容のパース規則 | ⚠️ 修正必要 |
| `text_part`ルール | `pasta.pest:141` | テキストパート定義 | ⚠️ 修正必要 |
| `word_expansion`ルール | `pasta.pest:266-268` | 単語参照パース | ✅ 問題なし |
| `WHITESPACE`定義 | `pasta.pest:11` | 暗黙空白定義 | ✅ 問題なし |
| Rustパーサー | `src/parser/mod.rs` | ASTへの変換 | ⚠️ 後処理追加検討 |
| `parse_speech_content()` | `mod.rs:557-597` | SpeechPart配列生成 | ⚠️ 後処理追加検討 |
| トランスパイラ | `src/transpiler/mod.rs` | AST→Rune変換 | ✅ 修正不要（ASTが正しければ） |
| テストフィクスチャ | `tests/fixtures/comprehensive_control_flow.pasta` | 問題再現ケース | ✅ 既存 |
| 単語定義テスト | `tests/pasta_transpiler_word_code_gen_test.rs` | 単語参照コード生成テスト | ⚠️ 新規テストケース追加 |

### 既存パターンと制約

**Pest文法パターン**:
- `WHITESPACE = _{ ... }`: アンダースコアプレフィックスにより暗黙的（自動消費）
- `text_part = @{ ... }`: アットマークプレフィックスにより原子的（内部ルール非展開）
- `speech_content = { (text_part | func_call | word_expansion | ...)* }`: 選択肢の順序でマッチング優先度決定

**Rust ASTパターン**:
- `SpeechPart`列挙型: `Text`, `VarRef`, `FuncCall`, `WordExpansion`, `SakuraScript`
- `parse_speech_content()`: Pestの`Rule`を`SpeechPart`配列に変換

**トランスパイラパターン**:
- `transpile_speech_part_to_writer()`: 各`SpeechPart`を個別の`yield Talk()`に変換
- WordExpansion → `yield Talk(pasta_stdlib::word("", "name", []))`

**制約**:
- Pest PEGパーサーの仕様上、選択肢は左から右に評価され、最初にマッチした選択肢が採用される
- 暗黙WHITESPACEはトークン境界で自動消費されるが、`@{}`原子ルール内では無視されない
- `text_part`は貪欲マッチであり、次のマーカー（`@`, `$`, `\`）または改行まですべてを捕捉する

---

## 2. 要件実現性分析

### 技術要素マップ

| 要件 | 必要技術 | 既存実装 | ギャップ |
|------|---------|---------|---------|
| Req 1.1: インライン参照直後空白スキップ | Rust後処理 | ❌ なし | **Gap**: パーサー層での空白除去ロジック不在（WordExpansionとVarRef両方） |
| Req 1.2: 複数連続空白スキップ | 同上 | ❌ なし | **Gap**: 同上 |
| Req 1.3: 空テキストパート非生成 | Rust後処理 | ⚠️ 部分的（空チェックなし） | **Gap**: 空文字列SpeechPart::Textのフィルタリング不在 |
| Req 1.4: 行末空白+改行の認識 | Pest文法 | ✅ 既存（NEWLINE認識） | ✅ OK（改行認識は正常） |
| Req 1.5: インライン参照前の空白保持 | 既存text_part動作 | ✅ 既存 | ✅ OK |
| Req 1.6: さくらスクリプト除外 | 既存動作 | ✅ 既存 | ✅ OK（さくらスクリプトは対象外） |
| Req 2: トランスパイラ正確性 | AST正確性依存 | ✅ 既存 | ✅ OK（パーサー修正後） |
| Req 3: テスト検証 | 新規テストケース | ⚠️ 部分的 | **Gap**: VarRef空白除去の明示的テストケース不在 |
| Req 4: 文法仕様整合性 | SPECIFICATION.md参照 | ✅ 既存 | ✅ OK（仕様範囲内修正） |

### ギャップと制約

**Missing: Rustパーサー層でのインライン参照直後空白除去ロジック**
- 現在の`speech_content`ルールは`word_expansion`および`var_ref`後の空白を`text_part`に含めてしまう
- `parse_speech_content()`でWordExpansionおよびVarRef直後のText先頭空白をトリムする必要がある
- さくらスクリプト（`sakura_script`）は対象外であり、直後の空白を除去してはならない

**Missing: Rustパーサー層での空SpeechPart除去**
- 現在の`parse_speech_content()`は生成されたすべての`SpeechPart`をそのまま配列に追加
- 空文字列の`SpeechPart::Text("")`が存在してもフィルタリングされない
- WordExpansion/VarRef直後の空白トリム後、空文字列になったTextパートを除外する必要がある

**Constraint: Pest暗黙WHITESPACEの挙動**
- `WHITESPACE = _{...}`により、ルール境界で自動消費されるが、原子ルール`@{}`内では無視されない
- `text_part = @{...}`は原子ルールのため、内部で空白を含む文字をすべて捕捉する

**Research Needed: Pest前読みパターンの詳細**
- `&rule`（ポジティブ前読み）や`!rule`（ネガティブ前読み）を活用した空白除去パターンの検証
- 他のPestプロジェクトでの類似パターン調査（特にマークアップ言語パーサー）

---

## 3. 実装アプローチオプション

### Option A: Pest文法のみで解決（word_expansion拡張）

**戦略**: `word_expansion`ルールを修正し、直後の空白を明示的に消費する

**修正箇所**:
- `src/parser/pasta.pest:266-268`

**具体案**:
```pest
// 修正前
word_expansion = {
    at_marker ~ word_name
}

// 修正案1: 直後の空白を明示的に消費（ただしtext_partに影響）
word_expansion = {
    at_marker ~ word_name ~ WHITESPACE*
}

// 修正案2: speech_contentルール側で制御
speech_content = { 
    (text_part | func_call | (word_expansion ~ WHITESPACE*) | var_ref | sakura_script)* 
}
```

**互換性評価**:
- ✅ `word_expansion`は括弧なし、`func_call`は括弧ありで明確に区別されているため、構文衝突なし
- ⚠️ Pest PEGの評価順序により、`word_expansion ~ WHITESPACE*`をグループ化しても、次の`text_part`が残り空白を捕捉する可能性

**複雑性**:
- ✅ 文法レベルでの解決は理解しやすい
- ❌ Pestの暗黙WHITESPACEとの相互作用が複雑

**Trade-offs**:
- ✅ Rustコード変更なし、文法のみで完結
- ✅ パーサーロジックへの影響最小化
- ❌ Pest PEGの仕様上、期待通り動作しない可能性が高い
- ❌ `text_part`の貪欲マッチを制御できない

**推奨度**: ⚠️ 低 - Pest PEGの特性上、この方法では解決困難

---

### Option B: Rustパーサー層での後処理（parse_speech_content修正）

**戦略**: `parse_speech_content()`関数内で、`WordExpansion`および`VarRef`直後の`Text`パートから先頭空白をトリムする

**修正箇所**:
- `src/parser/mod.rs:557-597` (`parse_speech_content()`)

**具体案**:
```rust
fn parse_speech_content(pair: Pair<Rule>) -> Result<Vec<SpeechPart>, PastaError> {
    let mut parts = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_part => {
                let text = inner_pair.as_str().to_string();
                
                // WordExpansionまたはVarRef直後の空白除去
                let trimmed_text = if let Some(last_part) = parts.last() {
                    match last_part {
                        SpeechPart::WordExpansion { .. } | SpeechPart::VarRef { .. } => {
                            text.trim_start().to_string()
                        }
                        _ => text,
                    }
                } else {
                    text
                };
                
                // 空文字列は追加しない
                if !trimmed_text.is_empty() {
                    parts.push(SpeechPart::Text(trimmed_text));
                }
            }
            Rule::word_expansion => {
                let word_name = inner_pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::word_name)
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default();
                parts.push(SpeechPart::WordExpansion { name: word_name });
            }
            Rule::var_ref => {
                let (var_name, var_scope) = parse_var_ref_with_scope(inner_pair)?;
                parts.push(SpeechPart::VarRef {
                    name: var_name,
                    scope: var_scope,
                });
            }
            // ... 他のルール（さくらスクリプトは除外）
        }
    }

    Ok(parts)
}
```

**互換性評価**:
- ✅ 既存の文法定義を変更しないため、他のパース処理への影響なし
- ✅ さくらスクリプト（`SakuraScript`）直後の空白には影響しない（開発者要件により除外）
- ✅ `FuncCall`直後の空白にも影響しない（括弧があるため空白が続くケースは稀）

**複雑性**:
- ✅ ロジックがシンプルで理解しやすい
- ✅ 既存のRustコードパターンに沿った実装
- ✅ 開発者要件（VarRefとWordExpansionのみ対象）に完全準拠

**Trade-offs**:
- ✅ Pest文法を変更しないため、文法の複雑性増加なし
- ✅ Rustレベルでの制御により、柔軟な後処理が可能
- ✅ テストが容易（Rustユニットテストで検証可能）
- ❌ パーサー層に「空白除去」というドメインロジックが混入（やや責務範囲外）
- ⚠️ `text_part`がパースした文字列を後から変更するため、元のソースとの対応が不明瞭になる可能性

**推奨度**: ✅ 高 - 実装が容易で副作用が少ない

---

### Option C: ハイブリッドアプローチ（文法微修正 + Rust後処理）

**戦略**: Pest文法で`word_expansion`直後の空白を明示的にスキップ可能にし、Rust層で確実に後処理を実施

**修正箇所**:
- `src/parser/pasta.pest:134` (`speech_content`ルール)
- `src/parser/mod.rs:557-597` (`parse_speech_content()`)

**具体案**:

**Pest文法側（微調整）**:
```pest
// 修正前
speech_content = { (text_part | func_call | word_expansion | var_ref | sakura_script)* }

// 修正後: word_expansionを優先評価し、直後の明示的空白消費を試みる
speech_content = { 
    (func_call | word_expansion_with_ws | var_ref | sakura_script | text_part)* 
}

word_expansion_with_ws = {
    word_expansion ~ ws?
}

// ws = 明示的空白（非暗黙）
ws = _{ (" " | "\t" | "\u{3000}")+ }
```

**Rust後処理側**:
```rust
// parse_speech_content()内で、WordExpansion直後のText先頭空白をトリム
// + 空文字列Textパートを除外
```

**互換性評価**:
- ✅ `word_expansion`を`word_expansion_with_ws`に置き換え、直後空白を明示消費
- ✅ 評価順序を変更し、`word_expansion_with_ws`を`text_part`より先に評価

**複雑性**:
- ⚠️ 文法とRustの両方に変更が必要
- ✅ 各層の責務が明確（文法=構文認識、Rust=意味処理）

**Trade-offs**:
- ✅ 文法レベルでの最適化とRust層での安全網の二重保護
- ✅ 将来的に他のマーカー（`$変数　`など）でも同様パターンを適用可能
- ❌ 変更箇所が多く、リグレッションテストの範囲が広い
- ❌ Pest文法の理解が必要

**推奨度**: ✅ 中〜高 - 最も堅牢だが実装コストがやや高い

---

## 4. 実装複雑性とリスク評価

### 工数見積もり

| オプション | 工数 | 根拠 |
|-----------|------|------|
| Option A: Pest文法のみ | **S (1-2日)** | 文法修正は小規模だが、Pest PEG仕様の調査と動作検証に時間がかかる。実現性が不確実。 |
| Option B: Rust後処理 | **S (1-2日)** | `parse_speech_content()`の修正のみ。既存パターンに沿った実装で、テスト追加も容易。 |
| Option C: ハイブリッド | **M (3-4日)** | 文法とRust両方の修正が必要。テスト範囲が広く、リグレッション検証に時間を要する。 |

### リスク評価

| オプション | リスク | 根拠 |
|-----------|-------|------|
| Option A: Pest文法のみ | **High** | Pest PEGの仕様上、`text_part`の貪欲マッチを制御できない可能性が高い。期待通り動作しない場合、文法の大幅な再設計が必要。 |
| Option B: Rust後処理 | **Low** | 既存パターンに沿った実装で、影響範囲が限定的。空白トリムロジックは単純で、バグ混入リスクが低い。 |
| Option C: ハイブリッド | **Medium** | 文法変更によるリグレッションリスクがあるが、Rust層の後処理が安全網となる。テストカバレッジを高めれば低リスク化可能。 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**第一候補: Option B（Rustパーサー層での後処理）**

**理由**:
- 実装が最もシンプルで、工数とリスクが最小
- 既存のPest文法を変更せず、Rustコードのみで完結
- テストが容易で、リグレッションリスクが低い
- 他のマーカー（`$変数`、さくらスクリプト`\n`など）への拡張も同様パターンで対応可能

**第二候補: Option C（ハイブリッドアプローチ）**

**条件付き推奨**:
- Option Bで解決後、パフォーマンスや保守性の観点から最適化が必要な場合
- または、将来的に他のマーカーでも同様の空白処理が必要になることが明確な場合

### 主要な設計判断事項

1. **空白除去の適用範囲**: WordExpansionのみか、VarRef/FuncCall/SakuraScriptにも適用するか
   - 推奨: まずWordExpansionのみに絞り、他は要件に応じて拡張
   
2. **全角/半角/タブの扱い**: 全種類の空白を除去するか、特定の空白のみか
   - 推奨: `WHITESPACE`定義に含まれるすべての空白を対象（全角・半角・タブ）

3. **空文字列SpeechPartの扱い**: 生成時に除外するか、トランスパイラ層でフィルタするか
   - 推奨: パーサー層で除外（ASTの品質向上、トランスパイラの簡素化）

4. **エラー処理**: 空白除去ロジックでエラーが発生した場合の挙動
   - 推奨: 空白除去は常に成功（trim_start()は失敗しない）

### 設計フェーズで必要なリサーチ

1. **Pest PEG前読みパターン調査** (低優先度)
   - Option Cを採用する場合のみ必要
   - 他のPestプロジェクトでの空白処理パターン調査
   
2. **既存テストケースの網羅性確認** (高優先度)
   - comprehensive_control_flow.pasta以外に、単語参照を含むテストケースを洗い出し
   - 回帰テストの範囲を明確化

3. **SPECIFICATION.md整合性確認** (中優先度)
   - 単語参照の文法仕様に「直後の空白を無視する」旨の記載があるか確認
   - 必要に応じて仕様書を更新

### 実装順序の推奨

**Phase 1: Option B実装（1-2日）**
1. `parse_speech_content()`にWordExpansion直後の空白トリムロジック追加
2. 空文字列TextパートのフィルタリングNEW: comprehensive_control_flow.pastaを含む単語参照テストケース追加
4. 既存テストスイート（`cargo test --all`）の実行とリグレッション確認

**Phase 2: Option C検討（必要に応じて）**
1. Phase 1でパフォーマンスや保守性の課題が判明した場合のみ
2. Pest文法修正の詳細設計
3. 段階的ロールアウト（文法変更 → Rustロジック調整）

---

## 6. まとめ

### ギャップ分析サマリー

- **既存実装**: Pest文法とRustパーサーは正常に動作しているが、`word_expansion`直後の空白が`text_part`に含まれる
- **主要ギャップ**: パーサー層での空白除去ロジック不在、空SpeechPartのフィルタリング不在
- **推奨アプローチ**: Option B（Rustパーサー層での後処理）を第一候補とし、必要に応じてOption C（ハイブリッド）を検討
- **工数**: S（1-2日）、リスク: Low

### 次のステップ

1. `/kiro-spec-design word-reference-whitespace-handling`を実行し、技術設計を生成
2. 設計フェーズでOption Bの詳細実装方針を確定
3. タスク分解後、実装フェーズへ移行
