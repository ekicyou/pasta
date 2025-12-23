# Research Log - parser2-pest-migration

## Summary

**調査目的**: pasta2.pestを権威的文法とする新パーサーモジュール（parser2）の設計のための技術調査

**主要発見事項**:
1. **pasta2.pest文法は検証済み** - 一切の変更不要、AST構築実装のみ
2. **3層スコープ階層** - FileScope ⊃ GlobalSceneScope ⊃ LocalSceneScope（レガシーとの最大の差分）
3. **PUSH/POPスタック** - Pest組み込み機能、4階層文字列リテラルに使用
4. **継続行の仕様変更** - `：`または`:`で始まる行（pasta.pestとの違い）
5. **reserved ID検証** - `__name__`パターンをPest negative lookaheadで拒否

**推奨アーキテクチャ**: 完全独立モジュール（Option B）

---

## Research Log

### 調査1: pasta2.pest文法の完全分析

**日付**: 2025-01-XX

**対象**: `src/parser/pasta2.pest` (223行、約60規則)

#### 文法構造の階層

```
file
├── file_scope (0..*)
│   ├── file_attr_line
│   ├── file_word_line
│   └── blank_line
└── global_scene_scope (0..*)
    ├── global_scene_start
    │   ├── global_scene_line (名前付き)
    │   └── global_scene_continue_line (継承)
    ├── global_scene_init (0..*)
    │   ├── global_scene_attr_line
    │   ├── global_scene_word_line
    │   └── blank_line
    ├── code_scope (0..*)
    └── local_scene_scope (1..*)
        ├── local_start_scene_scope (最初のローカルシーン、local_scene_lineなし)
        │   ├── local_scene_item (1..*)
        │   └── code_scope (0..*)
        └── local_scene_scope (追加のローカルシーン)
            ├── local_scene_start (local_scene_line)
            ├── local_scene_item (1..*)
            └── code_scope (0..*)
```

#### 重要な文法規則

| 規則 | 行 | 用途 | AST対応 |
|------|---|------|---------|
| `file` | 202 | エントリーポイント | `PastaFile2` |
| `file_scope` | 179 | ファイルレベル属性/単語 | `FileScope` |
| `global_scene_scope` | 182 | グローバルシーン全体 | `GlobalSceneScope` |
| `local_scene_scope` | 188-189 | ローカルシーン全体 | `LocalSceneScope` |
| `code_block` | 170-173 | 言語識別子付きコード | `CodeBlock` |
| `continue_action_line` | 195 | 継続行（`:` 開始） | `ContinueAction` |
| `string_fenced` | 91-107 | PUSH/POP文字列 | `StringLiteral` |
| `reserved_id` | 20 | `__name__`拒否 | 検証ロジック |

#### PUSH/POPスタック分析

pasta2.pestでのPUSH/POP使用箇所：

1. **文字列リテラル** (lines 96-107):
   ```pest
   slfence_ja1 = _{ "「"{1} ~ PUSH_LITERAL("」") }
   slfence_ja2 = _{ "「"{2} ~ PUSH_LITERAL("」」") }
   slfence_ja3 = _{ "「"{3} ~ PUSH_LITERAL("」」」") }
   slfence_ja4 = _{ "「"{4} ~ PUSH_LITERAL("」」」」") }
   slfence_en = _{ PUSH("\""+) }
   strclose = _{ POP }
   ```
   - `PUSH_LITERAL`: 固定文字列をスタックにプッシュ
   - `PUSH`: パターンマッチ結果をプッシュ（英語引用符の可変長対応）
   - `POP`: スタックトップと一致する場合のみ成功

2. **さくらスクリプト** (lines 147-152):
   ```pest
   sakura_open = _{ "[" ~ PUSH_LITERAL("]") }
   sakura_close = _{ POP }
   ```

3. **コードブロック** (lines 170-173):
   ```pest
   code_open = _{ PUSH("`"{3,}) ~ id? ~ eol }
   code_close = _{ POP ~ or_comment_eol }
   ```

**結論**: PUSH/POPはPestが自動処理し、AST構築時にはマッチ結果のみを使用

---

### 調査6: エスケープシーケンスとテキスト保全

**日付**: 2025-12-23

**調査対象**: pasta2.pestのエスケープシーケンス規則とテキスト情報保全

#### pasta2.pestでの定義

```pest
// 基本マーカー定義（全角/半角両対応）
at     = _{ "＠" | "@" }
dollar = _{ "＄" | "$" }

// エスケープシーケンス（アトミック規則）
at_escape     = @{ at{2} }       // ＠＠ または @@
dollar_escape = @{ dollar{2} }   // ＄＄ または $$
sakura_escape = @{ sakura_marker{2} }  // \\
```

#### 重要な発見

**問題**: 当初のAST設計では`EscapeType`列挙型を使用
```rust
pub enum EscapeType {
    At,      // 情報損失: 全角か半角か不明
    Dollar,  // 情報損失: 全角か半角か不明
    Sakura,  // \\
}
```

**解決策**: Pestのアトミック規則`@{...}`はマッチしたテキストをそのまま取得できる
- `at_escape = @{ at{2} }` → 「＠＠」または「@@」がテキストとして取得可能
- `dollar_escape = @{ dollar{2} }` → 「＄＄」または「$$」が取得可能
- `sakura_escape = @{ sakura_marker{2} }` → 「\\\\」が取得可能

**修正後のAST設計**:
```rust
pub enum Action {
    // ...
    /// エスケープシーケンス
    /// pasta2.pestのアトミック規則により、元のテキストがそのまま保持される
    Escape(String),  // 「＠＠」「@@」「＄＄」「$$」「\\」
}
```

#### トレーサビリティ

- **要件**: Requirement 3 (pasta2.pest文法に基づくAST型定義)
- **設計原則**: 文法との完全な一貫性（文法で取得可能な情報はすべて保持）
- **影響範囲**: `Action`列挙型、エスケープ処理ロジック
- **テスト観点**: 全角/半角エスケープの区別がテスト可能であることを確認

---

### 調査7: 数値リテラルの型判定

**日付**: 2025-12-23

**調査対象**: pasta2.pestの数値リテラル規則と型システム

#### pasta2.pestでの定義

```pest
number_literal = @{ sub? ~ digit+ ~ (dot ~ digit+)? }
digit          =  { ASCII_DIGIT | '０'..'９' }
```

- `sub?`: 符号（オプション）
- `digit+`: 整数部（必須）
- `(dot ~ digit+)?`: 小数部（オプション）

#### 設計決定

**問題**: 当初のAST設計ではすべての数値を`f64`として扱っていた
```rust
pub enum AttrValue {
    Number(f64),  // 整数も浮動小数点として扱う → 型情報損失
    // ...
}

pub enum Expr {
    Number(f64),  // 同様の問題
    // ...
}
```

**解決策**: パーサー層で小数点の有無により型を判定
- **小数点なし** → `Integer(i64)`
- **小数点あり** → `Float(f64)`

**修正後のAST設計**:
```rust
pub enum AttrValue {
    Integer(i64),  // 例: 123, -456
    Float(f64),    // 例: 3.14, -0.5
    // ...
}

pub enum Expr {
    Integer(i64),
    Float(f64),
    // ...
}
```

#### 実装ガイドライン

**全角数字変換関数**:
```rust
/// 全角数字・記号を半角に変換
/// pasta2.pestのdigit規則: ASCII_DIGIT | '０'..'９'
/// 符号: '－' | '-'
/// 小数点: '．' | '.'
fn normalize_number(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            // 全角数字 → 半角数字
            '０' => '0', '１' => '1', '２' => '2', '３' => '3', '４' => '4',
            '５' => '5', '６' => '6', '７' => '7', '８' => '8', '９' => '9',
            // 全角記号 → 半角記号
            '－' => '-',  // マイナス記号
            '．' => '.',  // 小数点
            // 半角はそのまま
            c => c,
        })
        .collect()
}
```

**パーサー実装時の判定ロジック**:
```rust
fn parse_number_literal(text: &str) -> Result<Expr, PastaError> {
    // 1. 全角を半角に正規化
    let normalized = normalize_number(text);
    
    // 2. 小数点の有無で型を判定
    if normalized.contains('.') {
        // 浮動小数点数
        let value = normalized.parse::<f64>()
            .map_err(|e| PastaError::ParseError {
                message: format!("Invalid float literal '{}': {}", text, e),
            })?;
        Ok(Expr::Float(value))
    } else {
        // 整数
        let value = normalized.parse::<i64>()
            .map_err(|e| PastaError::ParseError {
                message: format!("Invalid integer literal '{}': {}", text, e),
            })?;
        Ok(Expr::Integer(value))
    }
}
```

**テストケース例**:
| 入力 | 正規化後 | 結果 |
|------|---------|------|
| `123` | `123` | `Integer(123)` |
| `１２３` | `123` | `Integer(123)` |
| `-456` | `-456` | `Integer(-456)` |
| `－４５６` | `-456` | `Integer(-456)` |
| `3.14` | `3.14` | `Float(3.14)` |
| `３．１４` | `3.14` | `Float(3.14)` |
| `－０．５` | `-0.5` | `Float(-0.5)` |

**エラーハンドリング**:
- i64オーバーフロー: `9223372036854775808` → ParseError
- 不正な形式: `.123` (先頭ピリオド) → Pestレベルで拒否済み
- 複数の小数点: `1.2.3` → Pestレベルで拒否済み

#### トレーサビリティ

- **要件**: Requirement 3 (pasta2.pest文法に基づくAST型定義)
- **設計原則**: 型安全性の最大化
- **影響範囲**: `Expr`, `AttrValue`, パーサー実装ロジック
- **テスト観点**: 整数/浮動小数点の区別、境界値（i64::MAX, i64::MIN）

---

### 調査2: レガシーパーサーとの差分分析

**対象**: `src/parser/mod.rs` (978行), `src/parser/ast.rs` (297行)

#### API比較

| 機能 | レガシーparser | parser2（設計） |
|------|---------------|-----------------|
| ファイルパース | `parse_file(path)` | `parse_file(path)` (同一シグネチャ) |
| 文字列パース | `parse_str(source, filename)` | `parse_str(source, filename)` (同一シグネチャ) |
| 戻り値型 | `Result<PastaFile, PastaError>` | `Result<PastaFile2, PastaError>` |
| エラー型 | `PastaError::PestError(String)` | 同一（再利用） |

#### AST型の差分

| レガシーAST | parser2 AST | 差分理由 |
|-------------|-------------|----------|
| `PastaFile` | `PastaFile2` | FileScope追加 |
| - | `FileScope` | 新規：ファイルレベル属性/単語 |
| `SceneDef` | `GlobalSceneScope` | 構造変更：init + local_scenes |
| - | `LocalSceneScope` | 新規：ローカルシーン専用型 |
| - | `CodeBlock` | 新規：言語ID付きコードブロック |
| `Statement::Speech` | `ActionLine` | 名前変更：action_line対応 |
| - | `ContinueAction` | 新規：継続行専用型 |

#### 未名グローバルシーンの処理

レガシーparser (mod.rs lines 35-65):
```rust
let mut last_global_scene_name: Option<String> = None;
// ...
if !explicit_name {
    if let Some(continuation_name) = last_scene_name {
        name = continuation_name;
    } else {
        return Err(PastaError::ParseError { ... });
    }
}
```

parser2でも同一パターンを適用：
- `global_scene_continue_line`（`＊` のみ）の場合、直前の名前を継承
- ファイル先頭での出現はエラー

---

### 調査3: Pest 2.8 ベストプラクティス

**ソース**: Pest公式ドキュメント、Rustコミュニティ

#### PUSH/POPの動作

```rust
// Pestが生成するPair構造
// PUSH/POPはマッチ段階で処理され、Pair結果には影響しない
// string_contentsにはフェンス内のテキストのみが含まれる

fn parse_string_literal(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::string_contents => pair.as_str().to_string(),
        Rule::string_blank => String::new(),
        _ => unreachable!(),
    }
}
```

#### negative lookahead

reserved_id検証（lines 20）:
```pest
reserved_id = _{ "__" ~ idn2* ~ "__" }
id = @{ !(reserved_id) ~ identifier }
```

- `!(...)`はnegative lookahead
- Pestが自動的に`__name__`パターンを拒否
- AST構築時の追加検証は不要

---

### 調査4: スコープ階層設計

#### 3層スコープの包含関係

```
FileScope (ファイル全体)
├── file_attrs: Vec<Attr>
├── file_words: Vec<KeyWords>
└── global_scenes: Vec<GlobalSceneScope>
    ├── name: String
    ├── is_continuation: bool
    ├── attrs: Vec<Attr>
    ├── words: Vec<KeyWords>
    ├── code_blocks: Vec<CodeBlock>
    └── local_scenes: Vec<LocalSceneScope>
        ├── name: Option<String>  // 最初のローカルシーンはnameなし
        ├── items: Vec<LocalSceneItem>
        └── code_blocks: Vec<CodeBlock>
```

#### スコープ解析アルゴリズム

```rust
fn parse_file(pairs: Pairs<Rule>) -> PastaFile2 {
    let mut file_scope = FileScope::default();
    let mut global_scenes = Vec::new();
    let mut last_global_scene_name: Option<String> = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::file_scope => {
                file_scope = parse_file_scope(pair);
            }
            Rule::global_scene_scope => {
                let scene = parse_global_scene_scope(pair, &last_global_scene_name)?;
                last_global_scene_name = Some(scene.name.clone());
                global_scenes.push(scene);
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    
    PastaFile2 { file_scope, global_scenes, ... }
}
```

---

## Architecture Pattern Evaluation

### 評価対象パターン

| パターン | 適合度 | 理由 |
|----------|--------|------|
| **レイヤードアーキテクチャ** | ✅ 最適 | 既存アーキテクチャと一致、Parser→Transpiler→Runtimeの流れ |
| モジュラーモノリス | ⚠️ 部分的 | parser2は独立モジュールだが、最終的にparserを置換 |
| マイクロカーネル | ❌ 不適 | プラグイン構造は不要 |

### 選択: レイヤードアーキテクチャ（既存踏襲）

**理由**:
1. 既存のparser→transpiler→runtimeフローと一致
2. steering/tech.mdのレイヤー構成に準拠
3. 将来のtranspiler2統合が容易

---

## Design Decisions

### DD-1: AST型の命名規則

**決定**: `PastaFile2`、`GlobalSceneScope`、`LocalSceneScope`

**理由**:
- `PastaFile2`は一時的な名前（将来の統合後にリネーム）
- Scope型は文法規則名と一致させ、設計意図を明確化
- レガシーASTとの混同を防止

**却下案**:
- `PastaFileV2`: バージョン番号は将来の混乱を招く
- 同名（`PastaFile`）: 型衝突、インポート時の曖昧性

### DD-2: 継続行のAST表現

**決定**: `ContinueAction`型を独立定義

```rust
pub struct ContinueAction {
    pub actions: Vec<Action>,
    pub span: Span,
}

pub enum LocalSceneItem {
    VarSet(VarSet),
    CallScene(CallScene),
    ActionLine(ActionLine),
    ContinueAction(ContinueAction),  // 継続行専用
}
```

**理由**:
- pasta2.pestで`continue_action_line`が独立規則
- 前行への連結はtranspiler2の責務（parser2はそのまま出力）
- pasta.pestとの仕様差分を明示

### DD-3: FileScope の省略可能性

**決定**: `FileScope`は常に存在（空でも生成）

```rust
pub struct PastaFile2 {
    pub file_scope: FileScope,  // 空でも存在
    pub global_scenes: Vec<GlobalSceneScope>,
    // ...
}

pub struct FileScope {
    pub attrs: Vec<Attr>,
    pub words: Vec<KeyWords>,
}
```

**理由**:
- Optionによる複雑性を回避
- 空のFileScopeはセマンティック的に「属性なし、単語なし」

### DD-4: LocalStartSceneScopeの扱い

**決定**: `LocalSceneScope`で統一、`name: Option<String>`で区別

```rust
pub struct LocalSceneScope {
    pub name: Option<String>,  // Noneは最初の無名ローカルシーン
    pub items: Vec<LocalSceneItem>,
    pub code_blocks: Vec<CodeBlock>,
    pub span: Span,
}
```

**理由**:
- pasta2.pestの`local_start_scene_scope`と`local_scene_scope`は構造が同一
- 差分は`local_scene_line`の有無のみ
- 単一型で表現し、処理を統一

### DD-5: reserved ID検証の実装箇所

**決定**: Pest文法層で完結（Rust追加検証なし）

**理由**:
- pasta2.pestの`id = @{ !(reserved_id) ~ identifier }`がnegative lookahead
- Pestが`__name__`パターンを自動拒否
- エラーメッセージはPestデフォルトで十分

---

## Risks & Mitigations

### Risk-1: スコープ解析の複雑性

**リスクレベル**: Medium

**内容**: 3層スコープ（file→global→local）の再帰的解析は認知負荷が高い

**軽減策**:
- 各スコープに専用パース関数を定義
- ユニットテストでスコープ境界を検証
- 図式化されたスコープ階層をドキュメント化

### Risk-2: レガシーとの互換性

**リスクレベル**: Low

**内容**: parser2はレガシーparserと完全独立だが、将来の統合時に問題が発生する可能性

**軽減策**:
- 公開APIシグネチャを一致（`parse_file`, `parse_str`）
- エラー型を共有（`PastaError::PestError`）
- 移行計画を文書化

### Risk-3: テストカバレッジ不足

**リスクレベル**: Medium

**内容**: 約60規則すべてをテストする必要があり、漏れが発生しやすい

**軽減策**:
- 文法規則チェックリストを作成
- fixtureファイルで網羅的テスト
- CIで全テスト実行

---

## References

1. **Pest公式ドキュメント**: https://pest.rs/book/
2. **Pest PUSH/POPリファレンス**: https://pest.rs/book/grammars/built-ins.html#stack
3. **既存parser実装**: `src/parser/mod.rs` (978行)
4. **pasta2.pest文法**: `src/parser/pasta2.pest` (223行)
5. **gap-analysis.md**: `.kiro/specs/parser2-pest-migration/gap-analysis.md`
