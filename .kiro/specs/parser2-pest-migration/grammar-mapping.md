# Grammar Rule → AST Type Mapping

**目的**: pasta2.pestの全規則とAST型の対応関係を明示し、transpiler2実装時の参照を容易にする

**分類基準**:
- **normal規則**: トランスパイラーが認識する必要がある（AST型に対応）
- **atomic規則** (`@{ }`): テキストとして取得、AST内でString型として保持
- **silent規則** (`_{ }`): 中間処理のみ、AST型なし（パーサー内部で消費）

---

## Normal規則（AST型対応）

| 規則名 | AST型 | 備考 |
|--------|------|------|
| `file_scope` | `FileScope` | ファイルレベル属性・単語定義 |
| `global_scene_scope` | `GlobalSceneScope` | グローバルシーン全体 |
| `global_scene_start` | `GlobalSceneScope.name, is_continuation` | シーン開始行の解析 |
| `global_scene_line` | `GlobalSceneScope.name` | 名前付きグローバルシーン |
| `global_scene_continue_line` | `GlobalSceneScope.is_continuation=true` | 未名シーン（継承） |
| `global_scene_attr_line` | `GlobalSceneScope.attrs` | グローバルシーン属性 |
| `global_scene_word_line` | `GlobalSceneScope.words` | グローバルシーン単語定義 |
| `local_scene_scope` | `LocalSceneScope` | ローカルシーン全体 |
| `local_start_scene_scope` | `LocalSceneScope(name=None)` | __start__シーン |
| `local_scene_line` | `LocalSceneScope.name` | 名前付きローカルシーン |
| `file_attr_line` | `FileScope.attrs` | ファイルレベル属性 |
| `file_word_line` | `FileScope.words` | ファイルレベル単語定義 |
| `code_block` | `CodeBlock` | 言語識別子付きコード |
| `action_line` | `ActionLine` | 話者名付きアクション行 |
| `continue_action_line` | `ContinueAction` | 継続行（`:` 開始） |
| `call_scene` | `CallScene` | シーン呼び出し |
| `var_set_global` | `VarSet(scope=Global)` | グローバル変数代入 |
| `var_set_local` | `VarSet(scope=Local)` | ローカル変数代入 |
| `var_ref_global` | `VarRef(scope=Global)` | グローバル変数参照 |
| `var_ref_local` | `VarRef(scope=Local)` | ローカル変数参照 |
| `fn_call_global` | `FnCall(scope=Global)` | グローバル関数呼び出し |
| `fn_call_local` | `FnCall(scope=Local)` | ローカル関数呼び出し |
| `word_ref` | `Action::WordRef` | 単語参照 |
| `actions` | `Vec<Action>` | アクション列 |
| `attr` | `Attr` | 属性（キー：値） |
| `key_words` | `KeyWords` | 単語定義 |
| `key_literal` | `KeyWords` | リテラル値の単語定義 |
| `key_expr` | `KeyWords` | 式の単語定義 |
| `key_attr` | `Attr` | 属性（式値） |
| `args` | `Args` | 引数リスト |
| `positional_arg` | `Arg::Positional` | 位置引数 |
| `key_arg` | `Arg::Keyword` | キーワード引数 |
| `words` | `Vec<String>` | 単語値リスト |
| `paren_expr` | `Expr::Paren` | 括弧式 |
| `digit` | - | 数値リテラル内で使用（正規化処理） |
| `sakura_args` | `String` (in SakuraScript) | さくらスクリプト引数 |
| `sakura_str` | `String` (in sakura_args) | さくらスクリプト内文字列 |

---

## Atomic規則（テキスト取得）

| 規則名 | AST内での扱い | 備考 |
|--------|-------------|------|
| `id` | `String` | 識別子（reserved_id除外済み） |
| `xid1` | - | id内で使用（XID_START） |
| `xidn` | - | id内で使用（XID_CONTINUE） |
| `number_literal` | `Expr::Integer\|Float` | 全角→半角正規化後パース |
| `string_blank` | `Expr::BlankString` | 空文字列リテラル |
| `string_contents` | `Expr::String` | 文字列内容 |
| `string_nofenced` | `Expr::String` | 括弧なし文字列 |
| `word_nofenced` | `Vec<String>` | 括弧なし単語列 |
| `code_contents` | `CodeBlock.content` | コードブロック内容 |
| `sakura_script` | `Action::SakuraScript` | さくらスクリプト全体 |
| `sakura_id` | `String` (in SakuraScript) | さくらスクリプトID |
| `sakura_body` | `String` (in sakura_args) | さくらスクリプト引数本体 |
| `sakura_str_body` | `String` (in sakura_str) | さくらスクリプト内文字列本体 |
| `talk` | `Action::Talk` | プレーンテキスト |
| `at_escape` | `Action::Escape("＠＠"\|"@@")` | エスケープ（全角/半角保持） |
| `dollar_escape` | `Action::Escape("＄＄"\|"$$")` | エスケープ（全角/半角保持） |
| `sakura_escape` | `Action::Escape("\\\\")` | さくらエスケープ |
| `add_op` | `BinOp::Add` | 加算演算子 |
| `sub_op` | `BinOp::Sub` | 減算演算子 |
| `mul_op` | `BinOp::Mul` | 乗算演算子 |
| `div_op` | `BinOp::Div` | 除算演算子 |
| `modulo_op` | `BinOp::Mod` | 剰余演算子 |
| `ws` | - | 空白文字列（解析で使用、AST不要） |
| `no_ws` | - | 非空白文字（解析で使用、AST不要） |

---

## Silent規則（AST型なし）

**中間規則**: パーサー内部で使用、transpiler2は認識不要

### トップレベル構造

| 規則名 | 用途 |
|--------|------|
| `file` | エントリーポイント、file_scopeとglobal_scene_scopeを統合 |
| `file_scppe_item` | ファイルレベルアイテム（file_attr_line\|file_word_line\|blank_line） |
| `global_scene_init` | グローバルシーン初期化部（attr/word/blank） |
| `code_scope` | コードブロックグループ |
| `local_scene_item` | ローカルシーンアイテム（var_set/call_scene/action_line/continue） |
| `local_scene_start` | ローカルシーン開始（local_scene_line） |
| `blank_line` | 空白行（無視） |

### マーカー定義

| 規則名 | 用途 |
|--------|------|
| `hash`, `at`, `amp`, `ast`, `dollar` | 全角/半角統一（`＃\|#`, `＠\|@`, etc.） |
| `add`, `sub`, `mul`, `div`, `modulo` | 演算子マーカー |
| `lparen`, `rparen`, `gt`, `lt`, `pipe` | 括弧・記号 |
| `comma`, `dot`, `colon`, `semi`, `equals` | 区切り記号 |
| `comment_marker`, `attr_marker`, `global_marker`, etc. | 行頭マーカー |

### 式構造

| 規則名 | 用途 |
|--------|------|
| `expr` | 式のトップレベル（term + bin*） |
| `term` | 項（number_literal\|string_literal\|var_ref\|fn_call\|paren_expr） |
| `bin` | 二項演算（op + term） |
| `bin_op` | 二項演算子選択 |
| `set` | 代入構造 |

### 文字列関連

| 規則名 | 用途 |
|--------|------|
| `string_literal` | 文字列リテラル統合 |
| `string_fenced` | 括弧付き文字列 |
| `strfence` | 文字列括弧選択 |
| `slfence_ja1-4` | 日本語括弧PUSH（1-4階層） |
| `slfence_en` | 英語引用符PUSH |
| `strclose` | 文字列終端POP |

### アクション関連

| 規則名 | 用途 |
|--------|------|
| `action` | アクション選択（escape\|fn_call\|word_ref\|var_ref\|sakura_script\|talk） |
| `talk_word` | talkテキスト1文字 |

### その他

| 規則名 | 用途 |
|--------|------|
| `space_chars` | 14種類Unicode空白定義 |
| `s`, `pad` | 空白パターン |
| `eol`, `or_comment_eol` | 改行・コメント処理 |
| `identifier`, `id1`, `idn`, `idn2` | 識別子構成 |
| `dunder`, `reserved_id` | 予約ID検証（`__name__`拒否） |
| `scene` | シーン識別子（id + attrs?） |
| `var_ref`, `fn_call`, `var_set` | 参照/呼び出し統合 |
| `call_scene_line` | シーン呼び出し行 |
| `attrs` | 属性列 |
| `attr_value` | 属性値選択 |
| `word` | 単語選択 |
| `arg` | 引数選択 |
| `comma_sep` | カンマ区切り |
| `sakura_marker` | さくらマーカー（`\`） |
| `sakura_open`, `sakura_close` | さくらスクリプト括弧PUSH/POP |
| `sakura_str_open`, `sakura_str_close` | さくらスクリプト内文字列PUSH/POP |
| `code_open`, `code_close` | コードブロック括弧PUSH/POP |

---

## 実装ガイドライン

### パーサー実装

1. **Normal規則**: 対応するAST型を構築
2. **Atomic規則**: `.as_str()`でテキスト取得、必要に応じて正規化
3. **Silent規則**: Pairs<Rule>イテレーション時に分岐処理のみ

### トランスパイラー実装

1. **Normal規則のAST型**: パターンマッチで処理
2. **Atomic規則の結果**: String型フィールドとして取得済み
3. **Silent規則**: 無視（AST構築時に既に処理済み）

### テストカバレッジ

- **Normal規則**: 各規則に対応するfixture作成（必須）
- **Atomic規則**: 全角/半角、エッジケースのテスト
- **Silent規則**: 間接的にカバー（normal規則テストで網羅）

---

## 規則カウント

- **Total**: 140規則（silent含む全定義）
- **Normal**: 39規則（AST型対応）
- **Atomic**: 26規則（テキスト取得）
- **Silent**: 75規則（中間処理）
