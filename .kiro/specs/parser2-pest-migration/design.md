# Design Document: Parser2 Migration

## 設計概要

本文書は、pasta2.pest文法に基づく新パーサー層（`src/parser2/`）の詳細設計を定義します。Gap analysisで特定された5つのDesign Itemsを詳細化し、実装フェーズへの橋渡しを行います。

**設計原則**:
- pasta2.pestは**検証済み・不変の憲法**として扱う（一切の文法変更を認めない）
- Legacy parser（`src/parser/`）との完全な独立性を保証
- 全202行の文法規則に対して完全なAST型定義を提供

## Design Item 1: AST型の完全設計

### 1.1 型階層の全体像

pasta2.pestの文法構造に対応する23種類のAST型を定義します：

```
FileNode                              # file規則
├── FileScope                         # file_scope規則
│   ├── FileAttrLine                  # file_attr_line規則
│   └── FileWordLine                  # file_word_line規則
└── GlobalSceneScope                  # global_scene_scope規則
    ├── GlobalSceneStart              # global_scene_start規則
    │   ├── GlobalSceneLine           # global_scene_line規則
    │   └── GlobalSceneContinueLine   # global_scene_continue_line規則
    ├── GlobalSceneInit               # global_scene_init規則
    │   ├── GlobalSceneAttrLine       # global_scene_attr_line規則
    │   └── GlobalSceneWordLine       # global_scene_word_line規則
    └── LocalSceneScope               # local_scene_scope規則
        ├── LocalSceneLine            # local_scene_line規則
        └── LocalSceneItem            # local_scene_item規則
            ├── VarSetLine            # var_set_line規則
            ├── CallSceneLine         # call_scene_line規則
            ├── ActionLine            # action_line規則
            └── ContinueActionLine    # continue_action_line規則
```

### 1.2 コア型定義

#### 1.2.1 トップレベル型

```rust
/// pasta2.pest file規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct FileNode {
    pub file_scope: Option<FileScope>,
    pub global_scenes: Vec<GlobalSceneScope>,
    pub span: Span,
}

/// pasta2.pest file_scope規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct FileScope {
    pub attrs: Vec<AttrLine>,
    pub words: Vec<WordDefLine>,
    pub span: Span,
}
```

#### 1.2.2 グローバルシーン型

```rust
/// pasta2.pest global_scene_scope規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSceneScope {
    pub start: GlobalSceneStart,
    pub init: Vec<GlobalSceneInit>,
    pub code_blocks: Vec<CodeBlock>,
    pub first_local: LocalStartSceneScope,
    pub other_locals: Vec<LocalSceneScope>,
    pub span: Span,
}

/// pasta2.pest global_scene_start規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum GlobalSceneStart {
    Line(GlobalSceneLine),
    Continue(GlobalSceneContinueLine),
}

/// pasta2.pest global_scene_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSceneLine {
    pub marker: GlobalMarker,  // ＊ or *
    pub scene_id: Identifier,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

/// pasta2.pest global_scene_continue_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSceneContinueLine {
    pub marker: GlobalMarker,  // ＊ or *
    pub span: Span,
}

/// pasta2.pest global_scene_init規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum GlobalSceneInit {
    Attr(GlobalSceneAttrLine),
    Word(GlobalSceneWordLine),
}
```

#### 1.2.3 ローカルシーン型

```rust
/// pasta2.pest local_scene_scope規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct LocalSceneScope {
    pub start: LocalSceneLine,
    pub items: Vec<LocalSceneItem>,
    pub code_blocks: Vec<CodeBlock>,
    pub span: Span,
}

/// pasta2.pest local_start_scene_scope規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct LocalStartSceneScope {
    pub items: Vec<LocalSceneItem>,
    pub code_blocks: Vec<CodeBlock>,
    pub span: Span,
}

/// pasta2.pest local_scene_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct LocalSceneLine {
    pub marker: LocalMarker,  // ・ or -
    pub scene_id: Identifier,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

/// pasta2.pest local_scene_item規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum LocalSceneItem {
    VarSet(VarSetLine),
    CallScene(CallSceneLine),
    Action(ActionLine),
    ContinueAction(ContinueActionLine),
}
```

#### 1.2.4 アクション型

```rust
/// pasta2.pest action_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct ActionLine {
    pub speaker: Identifier,
    pub actions: Vec<Action>,
    pub span: Span,
}

/// pasta2.pest continue_action_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueActionLine {
    pub actions: Vec<Action>,
    pub span: Span,
}

/// pasta2.pest action規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FnCall(FnCall),
    WordRef(WordRef),
    VarRef(VarRef),
    SakuraScript(SakuraScript),
    Talk(Talk),
    AtEscape,
    DollarEscape,
    SakuraEscape,
}

/// pasta2.pest talk規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct Talk {
    pub content: String,
    pub span: Span,
}
```

### 1.3 式・リテラル型

#### 1.3.1 文字列リテラル（4階層PUSH/POP対応）

```rust
/// pasta2.pest string_literal規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum StringLiteral {
    Blank,                          // "" or 「」
    Fenced(FencedString),           // 4階層括弧対応
}

/// PUSH/POPスタック機構で解析された文字列
#[derive(Debug, Clone, PartialEq)]
pub struct FencedString {
    pub content: String,
    pub fence_type: FenceType,
    pub span: Span,
}

/// 文字列フェンスの種類（4階層日本語 + 英語）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FenceType {
    Japanese1,  // 「...」
    Japanese2,  // 「「...」」
    Japanese3,  // 「「「...」」」
    Japanese4,  // 「「「「...」」」」
    English,    // "..." (PUSHで任意長対応)
}
```

#### 1.3.2 数値リテラル

```rust
/// pasta2.pest number_literal規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    pub value: f64,
    pub raw: String,  // 全角数字対応のため元文字列を保持
    pub span: Span,
}
```

#### 1.3.3 式・演算子

```rust
/// pasta2.pest expr規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub term: Term,
    pub operations: Vec<BinOp>,
    pub span: Span,
}

/// pasta2.pest bin規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct BinOp {
    pub operator: BinOperator,
    pub right: Term,
    pub span: Span,
}

/// pasta2.pest bin_op規則に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOperator {
    Add,     // + or ＋
    Sub,     // - or －
    Mul,     // * or ＊ or ×
    Div,     // / or ／ or ÷
    Modulo,  // % or ％
}

/// pasta2.pest term規則に対応
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    ParenExpr(Box<Expr>),
    FnCall(FnCall),
    VarRef(VarRef),
    Number(NumberLiteral),
    String(StringLiteral),
}
```

### 1.4 識別子・マーカー型

#### 1.4.1 識別子（予約ID検証対応）

```rust
/// pasta2.pest id規則に対応（reserved_id拒否済み）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    /// __name__形式の予約IDをチェック（AST構築段階での追加検証）
    pub fn validate_not_reserved(&self) -> Result<(), String> {
        if self.name.starts_with("__") && self.name.ends_with("__") {
            Err(format!("Reserved identifier pattern: {}", self.name))
        } else {
            Ok(())
        }
    }
}
```

#### 1.4.2 マーカー型（全角・半角等価性）

```rust
/// pasta2.pest global_marker規則に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlobalMarker {
    FullWidth,  // ＊
    HalfWidth,  // *
}

/// pasta2.pest local_marker規則に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LocalMarker {
    FullWidth,  // ・
    HalfWidth,  // -
}

/// 全角・半角の等価性を保証
impl PartialEq<LocalMarker> for GlobalMarker {
    fn eq(&self, _other: &LocalMarker) -> bool {
        false  // 異なるマーカー種別は等価でない
    }
}
```

### 1.5 コードブロック型

```rust
/// pasta2.pest code_block規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub language: Option<Identifier>,  // 言語識別子（例: rune, rust）
    pub content: String,
    pub fence_length: usize,  // PUSHで記録されたバッククォート数
    pub span: Span,
}
```

### 1.6 補助型

```rust
/// Sakuraスクリプト（\s0等）
#[derive(Debug, Clone, PartialEq)]
pub struct SakuraScript {
    pub id: String,
    pub args: Option<SakuraArgs>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SakuraArgs {
    pub body: String,  // PUSH/POP解析済み内容
    pub strings: Vec<SakuraString>,  // 内部の文字列リテラル
    pub span: Span,
}

/// ソース位置情報
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

## Design Item 2: PUSH/POPスタックのAST構築設計

### 2.1 Pestスタック機構の理解

pasta2.pestでは3箇所でPUSH/POP機構を使用：

1. **4階層文字列リテラル**: `slfence_ja1`〜`slfence_ja4`, `slfence_en`
2. **コードブロック**: `code_open` → `code_contents` → `code_close`
3. **Sakuraスクリプト引数**: `sakura_open` → `sakura_body` → `sakura_close`
   - 内部文字列: `sakura_str_open` → `sakura_str_body` → `sakura_str_close`

### 2.2 文字列リテラルの構築アルゴリズム

```rust
/// pest::iterators::Pair<Rule> からStringLiteralを構築
fn build_string_literal(pair: Pair<Rule>) -> Result<StringLiteral, ParseError> {
    match pair.as_rule() {
        Rule::string_blank => Ok(StringLiteral::Blank),
        Rule::string_fenced => {
            let mut inner = pair.into_inner();
            
            // PUSH規則で決定されたフェンスタイプを取得
            let fence_pair = inner.next().unwrap();
            let fence_type = match fence_pair.as_rule() {
                Rule::slfence_ja1 => FenceType::Japanese1,
                Rule::slfence_ja2 => FenceType::Japanese2,
                Rule::slfence_ja3 => FenceType::Japanese3,
                Rule::slfence_ja4 => FenceType::Japanese4,
                Rule::slfence_en => FenceType::English,
                _ => unreachable!(),
            };
            
            // string_contents（PEEK照合で解析済み）
            let content_pair = inner.next().unwrap();
            let content = content_pair.as_str().to_string();
            
            // strclose（POP自動照合）は消費されるが検証不要
            
            Ok(StringLiteral::Fenced(FencedString {
                content,
                fence_type,
                span: Span::from_pair(&fence_pair),
            }))
        }
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}
```

### 2.3 コードブロックの構築アルゴリズム

```rust
/// pest::iterators::Pair<Rule> からCodeBlockを構築
fn build_code_block(pair: Pair<Rule>) -> Result<CodeBlock, ParseError> {
    let span = Span::from_pair(&pair);
    let mut inner = pair.into_inner();
    
    // code_open（PUSH規則で開始）
    let open_pair = inner.next().unwrap();
    let mut open_inner = open_pair.into_inner();
    
    // PUSHで記録されたバッククォート列を取得
    let fence_str = open_inner.next().unwrap().as_str();
    let fence_length = fence_str.len();
    
    // 言語識別子（オプション）
    let language = open_inner.next().map(|p| Identifier {
        name: p.as_str().to_string(),
        span: Span::from_pair(&p),
    });
    
    // code_contents（PEEKで終了マーカー検出まで消費）
    let content_pair = inner.next().unwrap();
    let content = content_pair.as_str().to_string();
    
    // code_close（POP自動照合）は消費されるが検証不要
    
    Ok(CodeBlock {
        language,
        content,
        fence_length,
        span,
    })
}
```

### 2.4 Sakuraスクリプト引数の構築アルゴリズム

```rust
/// Sakuraスクリプト引数（ネストした文字列リテラル対応）
fn build_sakura_args(pair: Pair<Rule>) -> Result<SakuraArgs, ParseError> {
    let span = Span::from_pair(&pair);
    let mut inner = pair.into_inner();
    
    // sakura_open（PUSH_LITERAL("]")）
    inner.next().unwrap();  // 消費のみ
    
    // sakura_body（sakura_str混在の可能性あり）
    let body_pair = inner.next().unwrap();
    let body = body_pair.as_str().to_string();
    
    // sakura_str（内部文字列）を抽出
    let mut strings = Vec::new();
    for str_pair in body_pair.into_inner() {
        if str_pair.as_rule() == Rule::sakura_str {
            let mut str_inner = str_pair.into_inner();
            
            // sakura_str_open（PUSH("\"")）
            str_inner.next().unwrap();
            
            // sakura_str_body（PEEK{2}でエスケープ対応）
            let str_body = str_inner.next().unwrap().as_str().to_string();
            
            // sakura_str_close（POP）
            strings.push(SakuraString {
                content: str_body,
                span: Span::from_pair(&str_pair),
            });
        }
    }
    
    // sakura_close（POP）は消費されるが検証不要
    
    Ok(SakuraArgs { body, strings, span })
}
```

## Design Item 3: スコープ階層解析アルゴリズム

### 3.1 3層スコープ構造の定義

pasta2.pestは以下の厳格な階層構造を定義：

```
file規則
├── file_scope（0または1個）        # FileScope型
│   └── file_scppe_item+             # ファイル属性・ワード定義
└── global_scene_scope*（0個以上）   # GlobalSceneScope型
    ├── global_scene_start           # グローバルシーン開始
    ├── global_scene_init*           # グローバルシーン属性・ワード定義
    ├── code_scope*                  # コードブロック（全スコープで配置可能）
    ├── local_start_scene_scope      # 最初のローカルシーン（必須）
    │   └── local_scene_item+
    └── local_scene_scope*           # 2番目以降のローカルシーン
        ├── local_scene_start        # ローカルシーン開始
        └── local_scene_item+
```

### 3.2 FileNode構築アルゴリズム

```rust
/// pest::iterators::Pair<Rule::file> からFileNodeを構築
fn build_file_node(file_pair: Pair<Rule>) -> Result<FileNode, ParseError> {
    let span = Span::from_pair(&file_pair);
    let mut file_scope = None;
    let mut global_scenes = Vec::new();
    
    for scope_pair in file_pair.into_inner() {
        match scope_pair.as_rule() {
            Rule::file_scope => {
                // file_scopeは最大1個のみ存在
                if file_scope.is_some() {
                    return Err(ParseError::DuplicateFileScope);
                }
                file_scope = Some(build_file_scope(scope_pair)?);
            }
            Rule::global_scene_scope => {
                global_scenes.push(build_global_scene_scope(scope_pair)?);
            }
            Rule::EOI => {} // 終端マーカー
            _ => return Err(ParseError::UnexpectedRule(scope_pair.as_rule())),
        }
    }
    
    Ok(FileNode {
        file_scope,
        global_scenes,
        span,
    })
}
```

### 3.3 GlobalSceneScope構築アルゴリズム

```rust
/// pest::iterators::Pair<Rule::global_scene_scope> からGlobalSceneScopeを構築
fn build_global_scene_scope(pair: Pair<Rule>) -> Result<GlobalSceneScope, ParseError> {
    let span = Span::from_pair(&pair);
    let mut inner = pair.into_inner();
    
    // 1. global_scene_start（必須）
    let start_pair = inner.next().ok_or(ParseError::MissingGlobalSceneStart)?;
    let start = match start_pair.as_rule() {
        Rule::global_scene_line => GlobalSceneStart::Line(build_global_scene_line(start_pair)?),
        Rule::global_scene_continue_line => GlobalSceneStart::Continue(build_global_scene_continue_line(start_pair)?),
        _ => return Err(ParseError::UnexpectedRule(start_pair.as_rule())),
    };
    
    // 2. global_scene_init*（0個以上）
    let mut init = Vec::new();
    let mut code_blocks = Vec::new();
    let mut first_local = None;
    let mut other_locals = Vec::new();
    
    for item_pair in inner {
        match item_pair.as_rule() {
            Rule::global_scene_attr_line => {
                init.push(GlobalSceneInit::Attr(build_global_scene_attr_line(item_pair)?));
            }
            Rule::global_scene_word_line => {
                init.push(GlobalSceneInit::Word(build_global_scene_word_line(item_pair)?));
            }
            Rule::code_block => {
                code_blocks.push(build_code_block(item_pair)?);
            }
            Rule::local_start_scene_scope => {
                // 最初のlocal_start_scene_scope（必須）
                if first_local.is_some() {
                    return Err(ParseError::DuplicateLocalStartScope);
                }
                first_local = Some(build_local_start_scene_scope(item_pair)?);
            }
            Rule::local_scene_scope => {
                // 2番目以降のlocal_scene_scope
                other_locals.push(build_local_scene_scope(item_pair)?);
            }
            _ => return Err(ParseError::UnexpectedRule(item_pair.as_rule())),
        }
    }
    
    // local_start_scene_scopeは必須
    let first_local = first_local.ok_or(ParseError::MissingLocalStartScope)?;
    
    Ok(GlobalSceneScope {
        start,
        init,
        code_blocks,
        first_local,
        other_locals,
        span,
    })
}
```

### 3.4 LocalSceneScope構築アルゴリズム

```rust
/// pest::iterators::Pair<Rule::local_scene_scope> からLocalSceneScopeを構築
fn build_local_scene_scope(pair: Pair<Rule>) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from_pair(&pair);
    let mut inner = pair.into_inner();
    
    // 1. local_scene_start（必須）
    let start_pair = inner.next().ok_or(ParseError::MissingLocalSceneStart)?;
    let start = build_local_scene_line(start_pair)?;
    
    // 2. local_scene_item+（1個以上）
    let mut items = Vec::new();
    let mut code_blocks = Vec::new();
    
    for item_pair in inner {
        match item_pair.as_rule() {
            Rule::var_set_line => {
                items.push(LocalSceneItem::VarSet(build_var_set_line(item_pair)?));
            }
            Rule::call_scene_line => {
                items.push(LocalSceneItem::CallScene(build_call_scene_line(item_pair)?));
            }
            Rule::action_line => {
                items.push(LocalSceneItem::Action(build_action_line(item_pair)?));
            }
            Rule::continue_action_line => {
                items.push(LocalSceneItem::ContinueAction(build_continue_action_line(item_pair)?));
            }
            Rule::code_block => {
                code_blocks.push(build_code_block(item_pair)?);
            }
            Rule::blank_line => {} // 空白行は無視
            _ => return Err(ParseError::UnexpectedRule(item_pair.as_rule())),
        }
    }
    
    if items.is_empty() {
        return Err(ParseError::EmptyLocalScene);
    }
    
    Ok(LocalSceneScope {
        start,
        items,
        code_blocks,
        span,
    })
}
```

## Design Item 4: reserved ID検証方法の決定

### 4.1 検証戦略の比較

pasta2.pestでは以下の方法でreserved ID（`__name__`）を拒否：

```pest
reserved_id = _{ dunder ~ idn2* ~ dunder }
id          = @{ !(reserved_id) ~ identifier }
```

**方法A: Pest negative lookahead（現在の実装）**
- **利点**: パース段階で即座に拒否、エラー位置が正確
- **欠点**: エラーメッセージがPest標準形式（"expected id, found __"）

**方法B: AST構築段階でのRust検証**
- **利点**: カスタムエラーメッセージ可能（"Reserved identifier pattern: __name__"）
- **欠点**: パース成功後のAST構築で失敗、二段階エラー処理

### 4.2 採用方針: ハイブリッドアプローチ

**決定**: Pest negative lookaheadを**主検証**、Rust検証を**補助検証**として併用

```rust
impl Identifier {
    /// AST構築段階での追加検証（Pestをすり抜けた場合の防御）
    pub fn validate_not_reserved(&self) -> Result<(), ParseError> {
        if self.name.starts_with("__") && self.name.ends_with("__") && self.name.len() > 4 {
            Err(ParseError::ReservedIdentifier {
                name: self.name.clone(),
                suggestion: format!("Identifiers starting and ending with '__' are reserved. Consider using '{}_' or '{}' instead.", 
                    &self.name[2..], &self.name[2..self.name.len()-2]),
            })
        } else {
            Ok(())
        }
    }
}

/// 全Identifier構築時に自動検証
fn build_identifier(pair: Pair<Rule>) -> Result<Identifier, ParseError> {
    let name = pair.as_str().to_string();
    let span = Span::from_pair(&pair);
    
    let id = Identifier { name, span };
    id.validate_not_reserved()?;  // 追加検証
    Ok(id)
}
```

### 4.3 エラーメッセージ設計

```rust
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Pest解析エラー（標準メッセージ）
    PestError(Box<pest::error::Error<Rule>>),
    
    /// 予約ID使用エラー（カスタムメッセージ）
    ReservedIdentifier {
        name: String,
        suggestion: String,
    },
    
    // その他のエラー型...
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::ReservedIdentifier { name, suggestion } => {
                write!(f, "Reserved identifier '{}': {}", name, suggestion)
            }
            ParseError::PestError(e) => write!(f, "{}", e),
            // ...
        }
    }
}
```

## Design Item 5: 継続行のAST設計

### 5.1 継続行の文法構造

pasta2.pestにおける継続行の定義：

```pest
action_line          = { pad ~ id ~ s ~ kv_marker ~ s ~ actions ~ eol }
continue_action_line = { pad ~ kv_marker ~ s ~ actions ~ eol }
```

**特徴**:
- `action_line`: 話者ID + `:` + アクション
- `continue_action_line`: `:` + アクション（話者ID省略）

### 5.2 AST型設計

```rust
/// pasta2.pest action_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct ActionLine {
    pub speaker: Identifier,      // 話者ID
    pub actions: Vec<Action>,     // アクション列
    pub span: Span,
}

/// pasta2.pest continue_action_line規則に対応
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueActionLine {
    pub actions: Vec<Action>,     // アクション列（話者IDなし）
    pub span: Span,
}

/// LocalSceneItemに統合
#[derive(Debug, Clone, PartialEq)]
pub enum LocalSceneItem {
    VarSet(VarSetLine),
    CallScene(CallSceneLine),
    Action(ActionLine),
    ContinueAction(ContinueActionLine),  // 継続行専用型
}
```

### 5.3 意味解析での処理方針

継続行は**前のActionLineと結合**する必要があります：

```rust
/// LocalSceneScope解析後の後処理で継続行を統合
fn merge_continue_actions(items: Vec<LocalSceneItem>) -> Vec<LocalSceneItem> {
    let mut merged = Vec::new();
    let mut pending_speaker: Option<Identifier> = None;
    
    for item in items {
        match item {
            LocalSceneItem::Action(mut action) => {
                // 通常のアクション行（話者ID付き）
                pending_speaker = Some(action.speaker.clone());
                merged.push(LocalSceneItem::Action(action));
            }
            LocalSceneItem::ContinueAction(continue_action) => {
                // 継続行を直前のActionLineに統合
                if let Some(LocalSceneItem::Action(ref mut last_action)) = merged.last_mut() {
                    last_action.actions.extend(continue_action.actions);
                } else {
                    // エラー: 継続行の前にActionLineが存在しない
                    return Err(ParseError::OrphanContinueAction {
                        span: continue_action.span,
                    });
                }
            }
            other => {
                merged.push(other);
            }
        }
    }
    
    Ok(merged)
}
```

### 5.4 例: 継続行の解析

```pasta
＊グローバルシーン
　ー開始
　　話者：こんにちは
　　　：これは継続行です
　　　：さらに続きます
```

**AST構造**:
```rust
LocalSceneScope {
    start: LocalSceneLine { scene_id: "開始", ... },
    items: vec![
        LocalSceneItem::Action(ActionLine {
            speaker: Identifier { name: "話者" },
            actions: vec![
                Action::Talk(Talk { content: "こんにちは" }),
                Action::Talk(Talk { content: "これは継続行です" }),  // 統合後
                Action::Talk(Talk { content: "さらに続きます" }),    // 統合後
            ],
        }),
    ],
}
```

## 次のステップ

Design phaseが完了したら、以下を実施：

1. **spec.json更新**: `phase: "design-generated"`に変更
2. **Tasks phase開始**: 実装タスクの詳細分解
   - Task 1: grammar.pest移動（git mv）
   - Task 2: `src/parser2/mod.rs` 骨格作成
   - Task 3: `src/parser2/ast.rs` 型定義（23型）
   - Task 4: AST構築関数実装（build_*系）
   - Task 5: エラー型定義とFrom trait実装
   - Task 6: `parse_file` / `parse_str` API実装
   - Task 7: `lib.rs` への統合
   - Task 8: テストファイル作成（8項目カバレッジ）
3. **Implementation phase**: タスク実行・検証
4. **Validation**: 全要件のAcceptance Criteria確認

---

**設計承認**: Design Document完成後、人間による承認を経て実装フェーズに移行します。
