# Parser2 AST → Requirements マッピング表

## 概要
parser2 が出力するすべての AST 要素が requirements.md で議論・定義されているか、体系的に確認するための対応表。

## AST 要素別マッピング

### レベル 1: ファイルレベル

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **PastaFile** | struct | ファイル全体の最上位AST | Req 2 AC1 | ✅ | 3層スコープの最上位 |
| `path` | PathBuf | ファイルパス | (implicit) | ✅ | メタデータ |
| `file_scope` | FileScope | ファイルスコープ | Req 2, Req 12, Req 15 | ✅ | 属性と単語定義を含む |
| `global_scenes` | Vec<GlobalSceneScope> | グローバルシーン集合 | Req 2 AC2, Req 3 | ✅ | ファイル最上位シーン |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 2a: FileScope（ファイル属性・単語定義）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **FileScope** | struct | ファイルレベルメタデータ | Req 2, Req 4, Req 12 | ✅ | 新規AST（parser1になし） |
| `attrs` | Vec<Attr> | ファイル属性 | Req 12 (AC1-AC6) | ✅ | すべてのシーンに継承 |
| `words` | Vec<KeyWords> | ファイル単語定義 | Req 15 (AC1-AC5) | ✅ | 新規要件（parser1では未処理） |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 2b: GlobalSceneScope（グローバルシーン）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **GlobalSceneScope** | struct | グローバルシーン定義 | Req 2 AC2, Req 3-4 | ✅ | Runeグローバル関数化 |
| `name` | String | シーン名 | Req 4 AC2 | ✅ | シンボル登録キー |
| `attrs` | Vec<Attr> | シーン属性 | Req 12 (AC1-AC6) | ✅ | FileScope属性とmerge |
| `words` | Vec<KeyWords> | シーン単語定義 | (Req 15は FileScope のみ) | ⚠️ | グローバルシーン内の単語処理？ |
| `local_scenes` | Vec<LocalSceneScope> | ローカルシーン集合 | Req 2 AC3, Req 4 AC3 | ✅ | ネストされたシーン |
| `code_blocks` | Vec<CodeBlock> | Runeコードブロック | Req 13 (AC1-AC6) | ✅ | mod末尾に一括出力 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 3: LocalSceneScope（ローカルシーン）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **LocalSceneScope** | struct | ローカルシーン定義 | Req 2 AC3, Req 4 AC3 | ✅ | Rune関数内のネストシーン |
| `name` | String | シーン名 | Req 3-4 | ✅ | ローカルスコープのキー |
| `attrs` | Vec<Attr> | ローカル属性 | Req 12 (AC1-AC6) | ✅ | シーンレベル属性 |
| `words` | Vec<KeyWords> | ローカル単語定義 | (要確認) | ⚠️ | ローカルスコープの単語処理？ |
| `items` | Vec<LocalSceneItem> | シーン本体 | Req 2, Req 7, Req 14-15 | ✅ | ActionLine/ContinueAction等 |
| `code_blocks` | Vec<CodeBlock> | Runeコードブロック | Req 13 (AC2-AC6) | ✅ | 関数内にインライン出力 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 4: LocalSceneItem（シーン本体の各項目）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **LocalSceneItem** enum | - | ローカルシーン本体 | Req 2-7, Req 14-15 | ✅ | 3 variants |
| `ActionLine` | variant | 会話行 | Req 2 AC4, Req 7 | ✅ | yield文出力 |
| `ContinueAction` | variant | 継続行 | Req 14 (AC1-AC5) | ✅ | 明示的`:`prefix |
| `CallScene` | variant | シーン呼び出し | Req 3 (AC1-AC5) | ✅ | 関数呼び出し化 |

### レベル 5a: ActionLine（会話行）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **ActionLine** | struct | 会話行（キャラ + アクション列） | Req 2 AC4, Req 7 | ✅ | yield文に変換 |
| `speaker` | Option<String> | 話者名 | Req 2 AC4 | ✅ | メタデータ |
| `actions` | Vec<Action> | アクション列 | Req 7 (AC1-AC7) | ✅ | Talk/VarRef等 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 5b: ContinueAction（継続行）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **ContinueAction** | struct | 継続行（前行との連結） | Req 14 (AC1-AC5) | ✅ | 明示的prefix `：` or `:` |
| `actions` | Vec<Action> | 追加アクション列 | Req 14 AC2 | ✅ | 直前ActionLineに連結 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 5c: CallScene（シーン呼び出し）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **CallScene** | struct | 呼び出し文（`＞scene`） | Req 3 (AC1-AC5) | ✅ | 関数呼び出しに変換 |
| `name` | String | 呼び出しシーン名 | Req 3 AC1 | ✅ | シーン解決の対象 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 6: Action（アクション型）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **Action** enum | - | アクション要素 | Req 7 (AC1-AC7) | ✅ | 6 variants |
| `Talk(String)` | variant | 会話テキスト | Req 7 AC1 | ✅ | yield Talk() |
| `VarRef { name, scope }` | variant | 変数参照 | Req 5, Req 7 AC2 | ✅ | ctx.local/global |
| `WordRef(String)` | variant | 単語参照 | Req 7 AC3 | ✅ | 単語辞書呼び出し |
| `FnCall { name, args, scope }` | variant | 関数呼び出し | Req 6 AC5, Req 7 AC4 | ✅ | func(ctx, args, ...) |
| `SakuraScript(String)` | variant | Sakuraスクリプト | Req 7 AC5 | ✅ | emit_sakura_script() |
| `Escape(String)` | variant | エスケープシーケンス | Req 7 AC6-AC7 | ✅ | 2文字目抽出 |

### レベル 6b: VarRef（変数参照のスコープ）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **VarScope** enum | - | 変数スコープ | Req 5 | ✅ | 3 scopes |
| `Local` | variant | ローカル変数 | Req 5 AC1 | ✅ | `$var` |
| `Global` | variant | グローバル変数 | Req 5 AC2 | ✅ | `$*var` |
| `System` | variant | システム変数 | Req 5 AC3 | ✅ | `$**var` |

### レベル 6c: FnCall（関数呼び出しのスコープ）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **FnScope** enum | - | 関数スコープ | Req 6 AC5 | ✅ | 2 scopes |
| `Local` | variant | ローカル関数 | Req 6 AC5a | ✅ | `@func` |
| `Global` | variant | グローバル関数 | Req 6 AC5b | ✅ | `@*func` |

### レベル 7: Expr（式）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **Expr** enum | - | 式要素 | Req 6 (AC1-AC6) | ✅ | 9 variants |
| `Integer(i64)` | variant | 整数リテラル | Req 6 AC1 | ✅ | 全角・半角対応 |
| `Float(f64)` | variant | 浮動小数点 | Req 6 AC2 | ✅ | 小数点含む |
| `String(String)` | variant | 文字列リテラル | Req 6 AC3 | ✅ | 括弧括り「」 |
| `BlankString` | variant | 空文字列 | Req 6 AC3 | ✅ | 「「」」 |
| `VarRef { name, scope }` | variant | 変数参照（式内） | Req 5-6 | ✅ | 同じVarScope利用 |
| `FnCall { name, args, scope }` | variant | 関数呼び出し（式内） | Req 6 AC5 | ✅ | 同じFnScope利用 |
| `Paren(Box<Expr>)` | variant | 括弧式 | (implicit) | ✅ | 優先度制御 |
| `Binary { left, op, right }` | variant | 二項演算 | Req 6 AC4 | ✅ | +, -, *, /, %, ==等 |

### レベル 7b: BinOp（二項演算子）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **BinOp** enum | - | 演算子 | Req 6 AC4 | ✅ | 10 operators |
| `Plus`, `Minus`, `Mult`, `Div`, `Mod` | variants | 算術演算 | Req 6 AC4 | ✅ | +, -, *, /, % |
| `Eq`, `Ne`, `Lt`, `Gt`, `Le`, `Ge` | variants | 比較演算 | Req 6 AC4 | ✅ | ==, !=, <, >, <=, >= |

### レベル 8: VarSet（変数代入）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **VarSet** | struct | 変数代入文 | Req 5 (AC4-AC5) | ✅ | LocalSceneItem 候補？ |
| `name` | String | 変数名 | Req 5 AC4 | ✅ | 代入対象 |
| `scope` | VarScope | スコープ | Req 5 AC1-AC3 | ✅ | ローカル・グローバル・システム |
| `value` | Expr | 代入値 | Req 5 AC4 | ✅ | RHS式として評価 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 9: CodeBlock（Runeコードブロック）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **CodeBlock** | struct | Rune埋め込みコード | Req 2 AC8, Req 13 | ✅ | グローバル/ローカル対応 |
| `language` | String | 言語（"rune"） | Req 13 | ✅ | 拡張性用 |
| `content` | String | コード内容 | Req 13 AC3 | ✅ | 未加工で埋め込み |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 10: KeyWords（単語定義）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **KeyWords** | struct | 単語定義 | Req 4 AC4, Req 15 | ✅ | ファイル・グローバル・ローカル対応 |
| `name` | String | 単語名 | Req 15 AC1 | ✅ | 単語キー |
| `words` | Vec<String> | 単語リスト | Req 15 AC1 | ✅ | 選択肢 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 11: Attr（属性）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **Attr** | struct | 属性定義 | Req 4 AC1, Req 12-13 | ✅ | ファイル・シーン対応 |
| `key` | String | 属性キー | Req 12 AC1 | ✅ | HashMap キー |
| `value` | AttrValue | 属性値 | Req 12 AC3 | ✅ | 文字列・エスケープ処理 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 11b: AttrValue（属性値）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **AttrValue** enum | - | 属性値型 | Req 12 AC1 | ✅ | 2 variants |
| `String(String)` | variant | 文字列値 | Req 12 AC1 | ✅ | 括弧括り処理済み |
| `Expr(Expr)` | variant | 式値 | Req 12 AC1 | ⚠️ | parser2で出力される？ |

### レベル 12: Args（関数引数）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **Args** | struct | 引数リスト | Req 6 AC5 | ✅ | 関数呼び出し用 |
| `args` | Vec<Arg> | 引数列 | Req 6 AC5 | ✅ | 位置・名前付き対応 |
| `span` | Span | ソース位置 | Req 8 AC5 | ✅ | エラー報告用 |

### レベル 12b: Arg（個別引数）

| AST 要素 | 型 | 説明 | 対応 Requirements | カバレッジ | 備考 |
|---------|-----|------|-----------------|----------|------|
| **Arg** enum | - | 個別引数 | Req 6 AC5 | ✅ | 2 variants |
| `Positional(Expr)` | variant | 位置引数 | Req 6 AC5 | ✅ | 式値 |
| `Named { key, value }` | variant | 名前付き引数 | (Req 6?) | ⚠️ | parser2で出力される？ |

---

### ⚠️ 部分的・不明確な要素
1. **GlobalSceneScope.words** - グローバルシーン内の単語定義はどう処理される？
   - **CLARIFIED** (Req 4 AC4-AC5 更新): FileScope.words → GlobalSceneScope.words → LocalSceneScope.words の順序で処理
   - スコープチェーン検索ルール実装：LocalScope → GlobalScope → FileScope

2. **LocalSceneScope.words** - ローカルシーン内の単語定義の処理
   - **CLARIFIED** (Req 4 AC4-AC5 更新): LocalSceneScope.words は最も内側のスコープとして処理
   - 同一スコープ内重複：Warning（エラーではない）

3. **AttrValue::Expr** - 属性値が式型の場合の処理
   - Req 12 では文字列値のみ言及
   - parser2 で実際に出力される可能性は低い（属性は文字列値が主）
   - **P1 設計段階での検証対象**

4. **Arg::Named** - 名前付き引数の処理
   - Req 6 AC5 では言及なし
   - parser2 で実際に出力される可能性は低い（位置引数が主）
   - **P1 設計段階での検証対象**

---

## 確認結果

### ✅ カバレッジ確認完了（最新）

**AST → Requirements マッピング状況：**
- 全 20+ AST 要素中、**16/16 要件で完全/部分的にカバー**
- 主要要素（PastaFile, FileScope, GlobalSceneScope, LocalSceneScope, Action, Expr, VarSet, CodeBlock, KeyWords, Attr）：**100% カバー**
- 拡張要素（AttrValue::Expr, Arg::Named）：**P1 検証対象**

**明確化された不確実性：**
1. **単語スコープ継承ルール** - Req 4 AC4-AC5 で明確化完了
   - スコープチェーン：LocalScope > GlobalScope > FileScope
   - 登録順序：FileScope.words → GlobalScene.words → LocalScene.words

2. **重複定義の処理** - Req 4 AC4 で明確化完了
   - 同一キーの属性・単語定義：Warning（エラーではない）
   - シーンレベル属性優先（Req 12 AC3）

3. **コードブロック出力戦略** - Req 13 で明確化完了
   - Global：mod末尾に一括出力
   - Local：関数内にインライン出力

---

## 次のアクション

### ✅ 要件定義フェーズ完了
- 16 要件すべて定義完了
- AST 全要素カバレッジ確認完了
- 主要な不確実性について明確化完了

### ⏭️ 設計フェーズへの遷移準備
- **コマンド:** `/kiro-spec-design transpiler2-layer-implementation [-y]`
- **入力:** 本マッピングドキュメント + 更新された requirements.md
- **出力:** 詳細設計ドキュメント（design.md）+ アーキテクチャ図

### 残存する P1 検証項目
- AttrValue::Expr の parser2 出現パターン確認
- Arg::Named の parser2 出現パターン確認
- （これらは設計フェーズで詳細化される予定）
