# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka スクリプトエンジン 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおけるスクリプトエンジンの要件を定義する。キャラクターとの自然な会話を実現し、人格と魅力を表現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 4.1 | 里々にインスパイアされた対話記述DSLを解釈・実行できる |
| 4.3 | ユーザーがキャラクターをダブルクリックした時、対話イベントを発火する |
| 4.4 | 変数を保持し、スクリプト内で参照・更新できる |
| 4.5 | 条件分岐・ループ等の制御構文をサポートする |
| 4.6 | 複数キャラクター間での会話（掛け合い、漫才的やりとり）をスクリプトで記述できる |
| 4.7 | 発言者の切り替え、割り込み、同時発言などの会話制御ができる |
| 4.8 | 会話を中断・再開できる（Generator機能） |
| 4.9 | チェイントーク（連続会話）を実装できる |
| 29.6 | さくらスクリプトの基本コマンドを出力できる |
| 29.7 | 独自拡張コマンドを定義できる |
| 29.8 | スクリプト出力とMCPコマンドを組み合わせられる |

### スコープ

**アーキテクチャ方針**: サブクレート`pasta`として独立実装し、SHIORI.DLL形式への分離を容易にする

**含まれるもの:**
- **サブクレート`pasta`の実装**（スクリプトエンジン本体）
  - 対話記述DSL（里々インスパイア）のパーサーと実行エンジン
  - **スクリプト言語の設計**（コマンド構文、ウェイト記法等）
  - Runeスクリプト実行エンジンの統合
  - **Generators（ジェネレータ）ベースの状態マシン実装**
    - 会話の中断・再開機能
    - チェイントーク（連続会話）サポート
    - yield式による段階的IR生成
  - 変数管理（グローバル/ローカル）
  - 制御構文（条件分岐、ループ、関数）
  - 複数キャラクター会話制御
- **中間表現（IR）の出力**（wintf-P0-typewriter への入力形式）
- さくらスクリプト互換コマンドの出力
- `wintf`クレートとの統合インターフェース

**含まれないもの:**
- LLM連携（areka-P2-llm-integration の責務）
- ゴーストパッケージ管理（areka-P0-package-manager の責務）
- タイプライター表示アニメーション（wintf-P0-typewriter の責務）
- DirectComposition/Direct2D等のグラフィックス機能（wintfの責務）

---

## DSL Syntax Specification

### 設計方針

本DSLは以下の方針で設計される：

1. **IME最適化**: JISキーボードでのIMEローマ字入力を前提とし、頻出構文はワンキー入力可能な全角記号を使用
2. **里々互換**: 基本構造は里々の辞書形式を踏襲
3. **Ren'Py模倣**: ラベル・call・jumpの概念はRen'Pyを参考
4. **Rust互換識別子**: ラベル名・変数名はRustの識別子規則に準拠（Unicode対応）
5. **Runeトランスコンパイル**: DSLはRuneスクリプトに変換されて実行される
6. **国際化対応**: キーワード文字は全角・半角両方をサポート（日本語圏・英語圏に配慮）

### アーキテクチャ

```
┌──────────────────────────────────────────────────────┐
│ サブクレート: pasta                                   │
│ ┌─────────────┐                                      │
│ │ DSL Script  │  会話記述特化（シンプルな構文）       │
│ └──────┬──────┘                                      │
│        │ parse (パーサー実装)                         │
│        ↓                                              │
│ ┌─────────────┐                                      │
│ │ Pasta AST   │  抽象構文木                           │
│ └──────┬──────┘                                      │
│        │ transpile                                    │
│        ↓                                              │
│ ┌─────────────┐                                      │
│ │ Rune Script │  汎用スクリプト実行エンジン           │
│ └──────┬──────┘                                      │
│        │ execute (rune VM + Generators)               │
│        ↓                                              │
│ ┌─────────────┐                                      │
│ │ State Mgr   │  状態マシン（Generator制御）          │
│ │ (Generator) │  - 中断/再開                         │
│ └──────┬──────┘  - チェイントーク                    │
│        │ yield IR tokens                              │
│        ↓                                              │
│ ┌─────────────┐                                      │
│ │     IR      │  中間表現（Vec<TypewriterToken>）    │
│ └──────┬──────┘                                      │
└────────┼─────────────────────────────────────────────┘
         │ 公開API (pasta::execute_script)
         ↓
┌──────────────────────────────────────────────────────┐
│ クレート: wintf                                       │
│ ┌──────────────────────────────────────────────┐    │
│ │ wintf-P0-typewriter                          │    │
│ │ タイプライター表示アニメーション              │    │
│ └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

**責務分担**:
- **pasta クレート**: スクリプトエンジン全体（DSLパーサー、Rune統合、IR生成）
  - **DSL層**: 会話フロー制御（ラベル、ジャンプ、発言、ウェイト）に特化
  - **Rune層**: 複雑なロジック（演算、関数、MCP連携）を担当
  - **トランスコンパイラ**: DSL構文をRuneコードに変換
  - **IR生成**: TypewriterToken列の生成
- **wintf クレート**: UIレンダリングとアニメーション
  - **Typewriter**: IR（TypewriterToken）を受け取り、タイピング表示を実行

**クレート分離の利点**:
1. **SHIORI.DLL化が容易**: `pasta`をC FFI経由で公開可能
2. **責務の明確化**: スクリプトロジックとUIレンダリングを完全分離
3. **再利用性**: 他のプロジェクトでも`pasta`を利用可能
4. **並行開発**: `pasta`と`wintf`を独立して開発可能

### キーワード文字対応表

すべてのキーワード文字は全角・半角両方をサポートする。

| 機能 | 全角（日本語IME） | 半角（英語圏） |
|------|-----------------|---------------|
| グローバルラベル | `＊` | `*` |
| ローカルラベル | `ー` | `-` |
| call | `＞` | `>` |
| jump | `？` | `?` |
| 式展開/属性 | `＠` | `@` |
| 属性区切り | `：` | `:` |
| 発言者区切り | `：` | `:` |
| 引数括弧 | `（）` | `()` |
| 文字列リテラル | `「」` | `""` |
| コメント | `＃` | `#` |
| 変数代入 | `＄` | `$` |
| 代入演算子 | `＝` | `=` |

**利点**:
- 日本語圏: ワンキー入力で効率的
- 英語圏: 標準的なASCII記号で入力可能
- 混在可能: 同一ファイル内で全角・半角を自由に組み合わせ可能

### 設計イシュー（Design Phase検討事項）

#### イシュー1: DSL層のIR生成責務

**現状の整理**:
- **DSLが直接IR生成するもの**: ウェイト変換（`\w[n]`, `\_w[n]` → `Wait(duration)` IRトークン）のみ
- **関数呼び出し経由**: その他の特殊機能（`＠笑顔`, `＠同時発言開始` など）はすべてRune関数として実装

**設計オプション**:

1. **Option A（現状）**: ウェイトのみDSL層で直接IR生成
   - ✅ さくらスクリプト互換（`\w[n]` 構文）を保持
   - ⚠️ DSL層とRune層の責務が分散

2. **Option B**: ウェイトも関数化（`＠ウェイト(0.5)`）
   - ✅ 完全に統一された設計（すべて関数呼び出し）
   - ✅ DSL層の責務を最小化（純粋なパーサー + トランスパイラ）
   - ⚠️ さくらスクリプト互換性の喪失
   - 💡 移行パス: DSL層で`\w[n]` → `＠ウェイト(n/1000.0)` に変換可能

**推奨**: Design Phaseで決定（さくらスクリプト互換性の優先度による）

### コメント

**コメント構文**:

```
＊挨拶                     # グローバルラベルの定義
　　＠時間帯：朝           # 朝の時間帯属性
　　＞＊他のラベル         # 別のラベルを呼び出し
　　さくら：今日は＃曇り  # 発言（＃は発言内容の一部）
　　＃ この行全体がコメント
```

**コメント規則**:

| 行の種類 | `＃` または `#` の扱い |
|---------|----------------------|
| **ラベル定義** (`＊`, `ー`) | 行末コメント可 |
| **属性定義** (`＠key：`) | 行末コメント可 |
| **制御フロー** (`＞`, `？`) | 行末コメント可 |
| **Runeブロック** (` ```内) | Rune構文に従う（`//`, `/* */`） |
| **発言行** (`:` 含む) | `＃`は発言の一部（コメントではない） |
| **継続行** (発言の続き) | `＃`は発言の一部（コメントではない） |
| **空行・その他** | 行頭`＃`のみコメント |

**設計意図**:
- 構文行: 注釈が書きやすい（メンテナンス性）
- 会話文: `＃`を自由に使える（表現の自由度）
- パーサー: 既存の行種別判定を活用

### 基本構文

#### ラベル定義

**グローバルラベル**（ファイル全体スコープ）:
```
＊ラベル名
  処理内容...
```
- **記号**: `＊` (全角アスタリスク、Shift + `:` キー)
- **位置**: 行頭（インデントなし）
- **命名規則**: Rust識別子規則（Unicode XID_Start + XID_Continue）

**ローカルラベル**（グローバルラベル内スコープ）:
```
＊親ラベル
  処理...
  ーローカル名
    ローカル処理...
```
- **記号**: `ー` (全角ハイフン、`-` キー、ワンキー)
- **位置**: インデント + 行頭
- **スコープ**: 直前のグローバルラベル内でのみ有効

**識別子命名規則**:

ラベル名・変数名は以下の制約に従う：

- **基本**: Rust識別子規則（Unicode XID_Start + XID_Continue）
- **冒頭文字禁止**: `＊`, `ー`, `＞`, `？`, `＠`, `＄`
- **全体禁止**: `：`, `＝` (コロン、イコール)

**理由**:
- DSL構文キーワードとの曖昧性を回避
- 属性定義・式展開・発言者区切り・代入演算子との明確な区別

**OK例**: `挨拶morning`, `挨拶_朝`, `あいさつ`, `greeting１`  
**NG例**: `＠挨拶`, `＞next`, `時間：朝`, `ーlocal`, `＄変数`, `値＝１`

#### 制御フロー

**call**（サブルーチン呼び出し、戻り先を記憶）:
```
＞ローカル名           # ローカルcall（現在のグローバルラベル内）
＞＊グローバル名       # グローバルcall（ファイル全体）
＞＊グローバルーローカル # ロングジャンプcall（グローバル配下のローカル）
＞＠変数名             # 動的call（変数の値をラベル名として解決）
```
- **記号**: `＞` (全角右角括弧、Shift + `.` キー)
- **動作**: 指定ラベルを実行後、呼び出し元に戻る
- **デフォルト**: ローカルスコープ（安全性優先）

**jump**（無条件ジャンプ、戻らない）:
```
？ローカル名           # ローカルjump（現在のグローバルラベル内）
？＊グローバル名       # グローバルjump（ファイル全体）
？＊グローバルーローカル # ロングジャンプ（グローバル配下のローカル）
？＠変数名             # 動的jump（変数の値をラベル名として解決）
```
- **記号**: `？` (全角疑問符、Shift + `/` キー)
- **動作**: 指定ラベルにジャンプし、呼び出し元には戻らない
- **デフォルト**: ローカルスコープ（安全性優先）

#### ランダム選択とラベル解決

**同一ラベル名の複数定義**:

DSLは同一名のラベルを複数定義できる（里々互換）。

```
＊挨拶
  さくら：おはよう！

＊挨拶
  さくら：こんにちは！

＊挨拶
  さくら：こんばんは！
```

**内部処理**:
- エンジンは自動的に連番を付与: `挨拶_1`, `挨拶_2`, `挨拶_3`
- 作者は意識する必要なし

**前方一致ランダム選択**:

call/jumpは前方一致するすべてのラベルから1つをランダム選択する。

```
？＊挨拶              # 「挨拶」で始まるすべてのグローバルラベル
？＊挨拶ー朝          # 「挨拶」グローバル配下の「朝」ローカルラベル
```

**ロングジャンプの解決（フラット化方式）**:

```
？＊挨拶ー朝
```

1. 前方一致でグローバルとローカルの組み合わせをすべて列挙:
   - `挨拶_1ー朝_1`
   - `挨拶_1ー朝_2`
   - `挨拶_2ー朝_1`
   - `挨拶イベントー朝`
2. フラットリストとして扱い、1つをランダム選択
3. 選択されたラベルにジャンプ

**選択肢キャッシュ**:

エンジンは選択キーワード（例: `＊挨拶ー朝`）ごとにキャッシュを保持する。

**キャッシュ動作**:
1. 初回呼び出し時:
   - 前方一致でフラットリストを構築
   - リストをシャッフル
   - キャッシュに保存
2. 2回目以降:
   - キャッシュからリストを取得
   - 先頭要素を消費（未消化順に使用）
3. リスト消費完了:
   - キャッシュをクリア
   - 次回は再構築（全候補が再び利用可能）

**利点**:
- 会話バリエーションを順に消化
- 重複を避けつつ、全候補を均等に使用
- 里々の辞書エントリ的な挙動

#### 変数と関数呼び出し

**式展開**（統一構文）:
```
＠識別子
＠識別子（引数）
```

**基本動作**:
- **記号**: `＠` または `@` (全角・半角両対応)
- **評価**: Rune変数または関数呼び出しとして評価
- **展開**: 戻り値の文字列をその場に埋め込む
- **出力**: IR形式のトークン列に変換

**キャラクターオブジェクト**:
- **第一引数**: すべての組み込み関数の第一引数はキャラクターオブジェクト
- **型**: Rune Object型 `#{ name: String, id: u32, surfaces: #{...}, state: #{...} }`
- **暗黙的渡し**: 発言者コンテキスト内では、グローバル変数として宣言されたキャラクターオブジェクトが自動的に第一引数として渡される
- **命名制約**: キャラクター名は有効な変数名でなければならない（英数字、アンダースコア、Unicode識別子）
  - ✅ OK: `さくら`, `うにゅう`, `sakura`, `char_01`, `キャラ１`
  - ❌ NG: `さくら（本体）`, `うにゅう-1`, `キャラ＃１` (スペース、括弧、記号を含む)
- **利点**: 関数内で発言者の状態・属性にアクセス可能、DSL記述が簡潔

**引数構文**:

```
＠関数名（引数１　引数２　引数３）              # 位置引数
＠関数名（name：太郎　age：20）               # 名前付き引数
＠関数名（太郎　age：20　city：「東京」）      # 混在可能
```

**引数の種類**:
1. **位置引数**: 順序で識別
2. **名前付き引数**: `名前：値` 形式
3. **値の型**:
   - 数値: `123`, `3.14`
   - 文字列: `「文字列」` または `"文字列"`
   - 変数参照: `＠変数名`
   - 関数呼び出し: `＠関数（引数）` (ネスト可能)

**文字列リテラル**:
- **全角鉤括弧**: `「こんにちは　世界」` (JISワンキー、スペース含む)
- **半角ダブルクォート**: `"Hello World"` (英語圏標準)
- 両方をサポート、混在可能

**引数区切り**:
- **区切り文字**: 1つ以上のスペース（全角・半角両可）
- **注意**: スペースが意味を持つため、文字列内スペースは `「」` または `""` で囲む

**使用例**:
```
さくら：今日は＠場所　に行こう。                     # 変数参照
さくら：こんにちは＠W（300）お元気？                # 関数呼び出し
さくら：＠format（「おはよう」　＠user_name）       # 文字列 + 変数
さくら：＠greet（time：＠current_time　name：太郎） # 名前付き引数
さくら：＠nested（＠func1（"hello"）　＠func2（））  # ネスト
```

**式の制限**:
- 会話文内では式展開を**サポートしない**（セキュリティリスク回避）
- `＠変数名` または `＠関数名（引数）` のみ許可
- 算術演算・比較演算は変数代入文または関数内で実行

#### ウェイトとタイミング制御

**ウェイトシステム**:

DSLは自動的にテキストにウェイトを挿入し、自然なタイピング表現を実現する。

**ウェイト種別**（キャラクター毎のシステムパラメーター）:
1. **通常ウェイト**: 通常の文字間隔
2. **濁点ウェイト**: 濁点（゛）で終わる文字の後
3. **半濁点ウェイト**: 半濁点（゜）で終わる文字の後
4. **点々ウェイト**: `‥`, `…` 等の連続記号1文字毎

**明示的ウェイト挿入**:
```
さくら：こんにちは＠W（300）お元気ですか？
```
- `＠W（ミリ秒）`: 指定時間のウェイトをIRトークンとして挿入
- 関数呼び出しの一種として実装（IRウェイトトークンを返す）

**禁則処理**:

DSLは自動的に日本語禁則処理を行い、適切な位置にウェイトを挿入する。

**禁則文字定義**:
- **行頭禁則**: `、。，．・？！゛゜ヽヾゝゞ々ー）］｝」』!),.:;?]}｡｣､･ｰﾞﾟ`
- **行末禁則**: `（［｛「『([{｢`
- **ぶらさげ**: `、。，．,.`

**ウェイト挿入ルール**:
1. **基本**: 文字毎に通常ウェイトを挿入
2. **禁則文字**: 行頭禁則・ぶらさげ文字が続く間は通常ウェイトで連続処理
3. **区切り位置**: 禁則文字列の終端で濁点/半濁点ウェイトを挿入
   - 直前の文字が濁点 → 濁点ウェイト
   - 直前の文字が半濁点 → 半濁点ウェイト
   - それ以外 → 通常ウェイト
4. **点々記号**: `‥`, `…` の各文字に点々ウェイトを挿入

**処理例**:
```
入力: 「こんにちは、元気？」
出力: 「こ」[通常W]「ん」[通常W]「に」[通常W]「ち」[通常W]「は」[通常W]「、」[通常W]「元」[通常W]「気」[通常W]「？」[通常W]

入力: 「がんばって‥‥ね」
出力: 「が」[濁点W]「ん」[通常W]「ば」[濁点W]「っ」[通常W]「て」[通常W]「‥」[点々W]「‥」[点々W]「ね」[通常W]
```

**さくらスクリプトエスケープ**:

DSLはさくらスクリプトコマンド自体を解釈しないが、エスケープ処理は透過的に行う。

```
さくら：こんにちは\n今日はいい天気だね
```
- `\n`, `\w[n]`, `\s[n]` 等はそのままテキストトークンとして出力
- IR層またはTypewriter層で解釈
- **互換性**: SHIORI.DLL形式への転用が可能

#### 発言

**発言者指定**:
```
<空白><発言者名><空白>：<発言内容>
```

**構文規則**:
- **発言者名**: Unicode空白と `：` 以外のすべての文字
  - ひらがな、カタカナ、漢字、ASCII、数値、絵文字等すべて可
  - 例: `さくら`, `うにゅう`, `０`, `ナレーション`, `🌸`
- **コロン前の空白**: 見た目を揃えるため空白を許容
- **インデント**: Unicode空白カテゴリすべて（全角・半角混在可）

**複数行発言**（継続行）:
```
　さくら　：こんにちは。
　　　　　　今日はいい天気だね。
　　　　　　どこか行こうよ。
　うにゅう：せやね。
```
- **継続条件**: コロンなし + インデント増加
- **終了条件**: インデント減少 or 新しい発言者

**パーサー処理**:
1. 行に `：` が含まれる → 新しい発言開始
   - コロン前（空白除去後）= 発言者名
   - コロン後 = 発言内容
2. 行に `：` がなく、インデント増加 → 前の発言に継続
3. インデント減少 → 発言終了、次の処理へ

#### ラベル属性

**属性定義**（メタデータ付与）:

```
＊ラベル名
　　＠属性名：値
　　＠属性名：値
　処理内容...
```

**構文規則**:
- **記号**: `＠key：value` (ワンキー入力可能)
- **位置**: ラベル定義行の直後、インデント必須
- **継続**: `＠`で始まる行が続く限り属性ブロック
- **終了**: `＠`で始まらない行で属性ブロック終了
- **値の式展開**: `＠属性：＠変数名` で動的な値を指定可能

**使用例**:
```
＊挨拶
　　＠時間帯：朝
　　＠重み：５
　さくら：おはようございます！

＊挨拶
　　＠時間帯：昼
　　＠重み：３
　さくら：こんにちは！

＊挨拶イベント
　　＠シーン：起床
　ー元気
　　　＠時間帯：朝
　　　＠テンション：高
　　　さくら：おはよう！！元気だね！
```

**属性フィルタリング**:

call/jump時に属性でラベルを絞り込める。

```
＞＊挨拶　＠時間帯：朝                    # 朝の挨拶のみ選択
＞＊挨拶　＠時間帯：＠現在時刻            # 動的フィルタ（変数展開）
？＊挨拶イベントー元気　＠テンション：高  # 複合フィルタ
```

**フィルタ動作**:
1. 前方一致でラベル候補を列挙
2. 指定された属性条件に一致するラベルのみ残す
3. 残った候補からランダム選択（キャッシュ適用）

**属性と引数の併用**:
```
＞＊挨拶　＠時間帯：朝　（＠ユーザー名）  # フィルタ + 引数渡し
```
- **属性**: ラベル選択の絞り込み条件
- **引数**: 選択後のラベルに渡すパラメータ

#### ローカル関数定義

**Runeコードブロック**:

ラベル内でローカル関数を定義できる（スコープはラベル内のみ）。

````
＊ラベル名
　　```rune
　　fn 関数名(引数) {
　　　　// Rune構文
　　}
　　```
　　処理内容...
````

**構文規則**:
- **開始**: インデント + ` ```rune`
- **終了**: インデント + ` ``` `
- **内容**: Rune構文そのまま（DSL解釈しない）
- **スコープ**: 定義したラベル内のみ有効
- **複数関数**: 1ブロックに複数の関数定義可能

**使用例**:
````
＊挨拶システム
　　```rune
　　fn format_greeting(name, time) {
　　　　time + "、" + name + "さん"
　　}
　　
　　fn get_emoji(emotion) {
　　　　if emotion == "happy" { "😊" }
　　　　else if emotion == "sad" { "😢" }
　　　　else { "😐" }
　　}
　　```
　　
　　さくら：＠format_greeting（＠user_name、＠current_time）
　　さくら：今日は良い天気ですね＠get_emoji（＠mood）
````

**設計意図**:
- 会話文中に式を埋め込まない（可読性・セキュリティ）
- その場限りの変換はローカル関数で対応
- 複雑なロジックは別ファイルのRune関数（グローバル）で記述

#### 変数代入

**代入構文**:

```
＄変数名＝式                    # ローカル変数（ラベル内スコープ）
＄＊変数名＝式                  # グローバル変数（プログラム全体）
```

**記号**:
- `＄` または `$`: 変数代入キーワード（全角・半角両対応）
- `＊` または `*`: グローバル指定（他の構文と一貫）
- `＝` または `=`: 代入演算子（全角・半角両対応）

**スコープ**:

| 記号 | スコープ | 寿命 |
|------|---------|-----|
| `＄` | ローカル | ラベル実行中のみ |
| `＄＊` | グローバル | プログラム全体（永続） |

**式の許可範囲**:

代入文の右辺では制限付き式をサポートする。

**許可される要素**:
1. **リテラル**:
   - 数値: `123`, `１２３`, `3.14`, `３．１４` (全角数字もUnicode変換)
   - 文字列: `「文字列」`, `"文字列"`
2. **変数参照**: 
   - `＠変数名`, `＠＊グローバル変数`
3. **関数呼び出し**: 
   - `＠関数名（引数）`
4. **四則演算**: 
   - `+`, `-`, `*`, `/`, `%` (加減乗除、剰余)
5. **括弧**: 
   - `（式）`, `(式)` (優先順位制御)

**許可されない要素**:
- 比較演算子 (`==`, `!=`, `<`, `>`, etc.)
- 論理演算子 (`and`, `or`, `not`)
- 複雑な制御構文
- → これらはローカル関数またはRune関数で記述

**使用例**:

```
＊計算例
　　＄年齢＝１５                          # リテラル代入
　　＄名前＝「さくら」                    # 文字列リテラル
　　＄来年＝＠年齢＋１                    # 算術式
　　＄合計＝（＠価格＊＠数量）＋＠送料    # 括弧と演算
　　＄結果＝＠計算関数（＠年齢　５）      # 関数呼び出し
　　＄＊好感度＝＠＊好感度＋１０          # グローバル変数更新
　　
　　さくら：私は＠年齢　歳です。
　　さくら：来年は＠来年　歳になります。

＊初期化
　　＄＊天気＝「晴れ」                    # グローバル変数設定
　　＄＊カウンター＝０
　　＄＊好感度＝５０
```

**変数参照の解決順序**:
1. ローカル変数を検索
2. 見つからなければグローバル変数を検索
3. 見つからなければローカル関数を検索
4. 見つからなければグローバル関数を検索
5. 見つからなければラベルを検索（callとして実行）
6. 見つからなければエラー

**名前空間の重複チェック**:

厳格な名前空間管理により、以下の重複を**禁止**する：

| スコープ | 禁止される重複 |
|---------|--------------|
| **ローカル** | ローカル変数 ⇔ ローカルラベル<br>ローカル変数 ⇔ ローカル関数<br>ローカルラベル ⇔ ローカル関数 |
| **グローバル** | グローバル変数 ⇔ グローバルラベル<br>グローバル変数 ⇔ グローバル関数<br>グローバルラベル ⇔ グローバル関数 |

**許可される唯一の「重複」**:
- 同名グローバルラベルの複数定義のみ許可（ランダム選択用）
- 内部では自動的に連番付与（`挨拶_1`, `挨拶_2`, ...）されるため、実際は一意

**エラー例**:
```
＊計算
  ＄結果＝５           # ローカル変数「結果」定義
  ー結果              # エラー: 「結果」は既にローカル変数として定義済み
  
＄＊天気＝「晴れ」     # グローバル変数「天気」定義
＊天気               # エラー: 「天気」は既にグローバル変数として定義済み
```

**設計意図**:
- 名前の衝突を完全に防止（予測可能な動作）
- リファクタリングの安全性確保
- IDE・デバッガのサポートを容易に
- 早期エラー検出（コンパイル時）

**全角数字の扱い**:
- Unicode数字カテゴリ（`\p{Nd}`）を自動的に半角に変換
- `０１２３４５６７８９` → `0123456789`
- 日本語入力の流れで記述可能

**変数代入の設計意図**:
- 簡単な計算はDSL内で完結（利便性）
- 複雑なロジックはRune関数へ（セキュリティ・保守性）
- グローバル指定は`＊`で統一（一貫性）
- 厳格な名前空間管理（安全性）

#### ファイル構成とロード規則

**プロジェクト構造**:

```
project/
├── main.rune              # エントリーポイント（Runeスクリプト）
├── helpers.rune           # グローバル関数（推奨配置）
├── game_logic.rune        # ゲームロジック（推奨配置）
└── dic/                   # 辞書フォルダ（会話データ）
    ├── greetings.pasta    # 挨拶パターン
    ├── events.pasta       # イベント会話
    ├── daily.pasta        # 日常会話
    └── special/           # サブフォルダ可能
        └── holiday.pasta  # 特別イベント
```

**ロード規則**:

1. **Pasta DSL**: `./dic/**/*.pasta` を全て自動読み込み
   - 再帰的にサブフォルダも探索
   - 全ラベルをグローバルジャンプテーブルに統合
   - 同名ラベルは自動的に連番付与（ランダム選択用）
   - ファイル間のラベル参照は自由（全ファイル読み込み後に解決）

2. **Rune Script**: `main.rune` から `mod`/`use` で明示的インポート
   - ルート階層推奨（`mod helpers;` でインポート）
   - `mod dic;` は技術的に可能だが**非推奨**（作法違反）

**設計意図**:

| 配置 | 役割 | ロード方式 | 性質 |
|------|------|-----------|------|
| `dic/` | 宣言的な会話データ | 自動読み込み | グローバルジャンプテーブル |
| ルート | 手続き的なロジック | 明示的インポート | モジュールシステム |

**利点**:
- ✅ Convention over Configuration（設定ファイル不要）
- ✅ ファイル分割が自由（保守性向上）
- ✅ ラベル名前空間はグローバル（DSLの性質に合致）
- ✅ Runeスクリプトは標準的なモジュールシステム

**エンジン初期化フロー**:
```rust
// 1. Pasta DSLの読み込み
let pasta_files = glob("./dic/**/*.pasta")?;
let label_table = parse_all_pasta(pasta_files)?;

// 2. Runeスクリプトの初期化
let rune_vm = init_rune("main.rune")?;

// 3. 統合（ラベルとRune関数の接続）
let engine = ScriptEngine::new(label_table, rune_vm)?;
```

#### 使用例

```
＃ 初期化ラベル - 変数の初期値設定
＊初期化
  ＄＊場所＝「ニューヨーク」      # グローバル変数: 場所
  ＄＊次の行動＝「会話」          # グローバル変数: 次の行動
  ＄＊現在時刻＝「朝」            # グローバル変数: 現在時刻
  ＄＊好感度＝５０                # グローバル変数: 好感度（数値）

＃ 表情制御の例
＊感情表現
  さくら    ：＠笑顔　やった！
  　　　　　　＠ウェイト（０．５）  # 0.5秒待機
  うにゅう  ：＠微笑み　おめでとう。
  さくら    ：＠サーフェス（３）　ありがとう！  # サーフェスID=3に変更
  
＊メインメニュー
  ナレーション：どうする？
  ？＠次の行動        # 動的ジャンプ（変数の値で分岐）

＃ 挨拶バリエーション - 時間帯別の挨拶パターン
＊挨拶
　　＠時間帯：朝
　　＠重み：５      # 選択確率の重み付け
　さくら    ：おはよう！

＊挨拶
　　＠時間帯：昼
　　＠重み：３
　さくら    ：こんにちは！

＊挨拶イベント
　　＠シーン：起床
　ー朝                  # ローカルラベル: 朝の挨拶
　　　＠テンション：通常
　　　さくら    ：おはようございます。
　　　　　　　　　今日もがんばろうね！
　　　うにゅう  ：おはよう。
　ー朝
　　　＠テンション：高  # テンション高めバージョン
　　　さくら    ：朝だよ！起きて！
　ー夜
　　　さくら    ：こんばんは。
　　　うにゅう  ：おやすみ。

＊会話
　　```rune
　　// ローカル関数: 場所名をフォーマット
　　fn format_location(loc) {
　　　　"「" + loc + "」"
　　}
　　// ローカル関数: 感情に応じた絵文字生成
　　fn make_emoji(emotion, intensity) {
　　　　if emotion == "happy" {
　　　　　　if intensity > 5 { "😆" } else { "😊" }
　　　　} else { "😐" }
　　}
　　```
　　
　　＄テンション＝＠＊好感度／１０          # ローカル変数: 好感度から計算
　　
  さくら    ：今日は＠format_location（＠＊場所）に行こう！
  うにゅう  ：いいね＠make_emoji（"happy"　＠テンション）
  さくら    ：天気も良いし＃最高だね  # 会話内の＃は発言の一部
  
  ＄＊好感度＝＠＊好感度＋５              # グローバル変数を更新
  
  ＞＊挨拶　＠時間帯：＠＊現在時刻         # 属性フィルタで時間帯に合った挨拶
  ＞＊挨拶イベントー朝　＠テンション：高  # 属性付きロングジャンプ
  さくら    ：楽しかったね。
  ？＊終了           # グローバルjump

＊別れ
  さくら    ：またね！
  うにゅう  ：ばいばい。

＊終了
  ナレーション：さようなら。
```

---

## Requirements

### Requirement 1: 対話記述DSL

**Objective:** ゴースト制作者として、自然な会話を簡潔に記述したい。それにより効率的にゴーストを制作できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 里々にインスパイアされた対話記述DSLを解釈・実行できる
2. **The** Script Engine **shall** トーク（発言ブロック）を定義できる
3. **The** Script Engine **shall** トーク内でテキストと制御コマンドを混在できる
4. **The** Script Engine **shall** トークの呼び出し（関数呼び出し）をサポートする
5. **The** Script Engine **shall** UTF-8エンコーディングのスクリプトファイルを読み込める

---

### Requirement 2: 中間表現（IR）出力

**Objective:** 描画エンジンとして、パース済みの構造化データを受け取りたい。それによりTypewriterは表示アニメーションに専念できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** スクリプトを中間表現（IR）に変換して出力できる
2. **The** Script Engine **shall** テキストトークン（表示文字列）をIRに含められる
3. **The** Script Engine **shall** ウェイトトークン（待機時間）をIRに含められる
4. **The** Script Engine **shall** サーフェス切り替えトークン（キャラクター指定、サーフェスID）をIRに含められる
5. **The** Script Engine **shall** 発言者切り替えトークンをIRに含められる
6. **The** Script Engine **shall** すべてのIRトークンにキャラクターコンテキストを含められる
6. **The** Script Engine **shall** 将来の拡張トークン（速度変更、ポーズ等）を追加可能な設計とする
7. **The** Script Engine **shall** IRの型定義を `wintf-P0-typewriter` と共有する

---

### Requirement 3: さくらスクリプト互換出力

**Objective:** ゴースト制作者として、既存のさくらスクリプト知識を活用したい。それにより学習コストを削減できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** さくらスクリプトの基本コマンドをIRに変換できる
2. **The** Script Engine **shall** サーフェス切り替えコマンド（`\s[n]`）を解釈できる
3. **The** Script Engine **shall** ウェイトコマンド（`\w[n]}`, `\_w[n]`）を解釈できる
4. **The** Script Engine **shall** 発言者切り替えコマンド（`\0`, `\1`等）を解釈できる
5. **The** Script Engine **shall** 改行コマンド（`\n`）を解釈できる
6. **The** Script Engine **shall** 独自拡張コマンドを定義できる

---

### Requirement 4: 変数管理

**Objective:** ゴースト制作者として、キャラクターの状態を変数で管理したい。それにより動的な会話を実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** グローバル変数を保持・参照・更新できる
2. **The** Script Engine **shall** ローカル変数（トーク内スコープ）をサポートする
3. **The** Script Engine **shall** 変数の型（文字列、数値、真偽値、Object）をサポートする
4. **The** Script Engine **shall** 変数を文字列展開（`＠変数名`）できる
5. **The** Script Engine **shall** システム変数（日時、カウンター等）を提供する
6. **The** Script Engine **shall** 変数永続化をRuneスクリプトの責務とする
   - Runeの任意の変数をTOMLに書き出し可能
   - 永続化はRuneスクリプトのmainルーチンが実装
   - DSL層は永続化機能を持たない（責務外）

**変数永続化設計**:

```rune
// Runeスクリプト側での永続化実装例
pub async fn main() {
    // 起動時: TOMLから変数復元
    let state = load_state_from_toml("save.toml");
    
    // グローバル変数に設定
    set_global("好感度", state.好感度);
    set_global("場所", state.場所);
    set_global("さくら", state.さくら);
    set_global("うにゅう", state.うにゅう);
    
    // メインループ
    loop {
        // スクリプト実行...
        
        // 定期保存: グローバル変数をTOMLに書き出し
        if should_save() {
            let state = #{
                好感度: get_global("好感度"),
                場所: get_global("場所"),
                さくら: get_global("さくら"),
                うにゅう: get_global("うにゅう"),
            };
            save_state_to_toml("save.toml", state);
        }
    }
}
```

**責務分担**:
- **DSL層**: 変数の宣言・参照・更新のみ
- **Rune層**: 永続化ロジック（TOML読み書き）
- **pasta API**: `get_global()`, `set_global()` のみ提供

---

### Requirement 5: 制御構文

**Objective:** ゴースト制作者として、条件分岐やループで複雑なロジックを記述したい。それにより多様な会話パターンを実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 条件分岐（if/else）をサポートする
2. **The** Script Engine **shall** ループ（while/for/repeat）をサポートする
3. **The** Script Engine **shall** 比較演算子（==, !=, <, >, <=, >=）をサポートする
4. **The** Script Engine **shall** 論理演算子（and, or, not）をサポートする
5. **The** Script Engine **shall** 算術演算子（+, -, *, /, %）をサポートする
6. **The** Script Engine **shall** ランダム選択（複数候補から1つを選択）をサポートする

---

### Requirement 6: 複数キャラクター会話制御

**Objective:** ゴースト制作者として、複数キャラクター間の掛け合いを記述したい。それにより漫才的なやりとりを実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 発言者（キャラクター）を切り替えられる
2. **The** Script Engine **shall** 複数キャラクターが同時に発言できる（同期発言）
3. **The** Script Engine **shall** キャラクターが他のキャラクターの発言に割り込めむ
4. **The** Script Engine **shall** キャラクター間でスコープ（変数、状態）を共有できる
5. **The** Script Engine **shall** 2体以上のキャラクター会話をサポートする
6. **The** Script Engine **shall** シンクロナイズドセクション（同時発言）を実装できる
   - 関数呼び出しによる同期制御（`＠同時発言開始`, `＠同期`, `＠同時発言終了`）
   - 複数キャラクターのIRトークンを同一タイミングで出力
   - バルーン/シェルの複数表示を同期制御
7. **The** Script Engine **shall** キャラクター名を有効な変数識別子として扱う
   - キャラクター名はグローバル変数名として使用される
   - 英数字、アンダースコア、Unicode識別子をサポート
   - スペース、括弧、記号を含む名前は非サポート

#### Technical Details: シンクロナイズドセクション

**IR設計**:

```rust
// TypewriterTokenに同期マーカーを追加
pub enum TypewriterToken {
    Text(String),
    Wait(f64),
    ChangeSpeaker(String),
    
    // 同時発言制御
    BeginSync { sync_id: String },  // 同期セクション開始
    EndSync { sync_id: String },    // 同期セクション終了
    SyncPoint { sync_id: String },  // 同期ポイント（待ち合わせ）
    
    FireEvent { target: Entity, event: TypewriterEventKind },
}
```

**設計原則**: 
1. 特殊なことをしたければ関数を呼べ
2. 関数の第一引数はキャラクターオブジェクト（Rune Object型）
3. 発言者コンテキスト内では第一引数は暗黙的に渡される

**DSL構文例**:

```
＊同時発言例
　　さくら：＠同時発言開始　せーの
　　　　　　＠同期
　　うにゅう：＠同時発言開始　せーの
　　　　　　＠同期
　　　　　　＠同時発言終了
　　さくら：＠同時発言終了　（笑）
　　　　　　＠笑顔
　　うにゅう：＠微笑み　息がぴったりやね。
```

**暗黙的引数渡し**:
- `さくら：＠笑顔` → `笑顔(さくら)` に展開（グローバル変数 `さくら` を第一引数として渡す）
- `うにゅう：＠微笑み` → `微笑み(うにゅう)` に展開（グローバル変数 `うにゅう` を第一引数として渡す）
- トランスパイラが発言者コンテキストを追跡し、自動的に第一引数を補完

**組み込み関数** (pasta標準ライブラリで提供):

```rune
// 第一引数: キャラクターオブジェクト（Rune Object型）
// 実体: グローバル変数として宣言されたRune Object
// 例: let さくら = #{ name: "さくら", id: 0, surfaces: #{...}, state: #{...} };

// 同期制御
pub fn 同時発言開始(character) {
    yield BeginSync { 
        sync_id: generate_sync_id(),
        character_name: character.name 
    };
}

pub fn 同期(character) {
    yield SyncPoint { 
        sync_id: current_sync_id(),
        character_name: character.name 
    };
}

pub fn 同時発言終了(character) {
    yield EndSync { 
        sync_id: current_sync_id(),
        character_name: character.name 
    };
}

// 表情・サーフェス制御
pub fn 笑顔(character) {
    // キャラクターの笑顔サーフェスIDを取得
    let surface_id = character.surfaces.笑顔;
    yield ChangeSurface { 
        character_name: character.name,
        surface_id 
    };
}

pub fn 微笑み(character) {
    let surface_id = character.surfaces.微笑み;
    yield ChangeSurface { 
        character_name: character.name,
        surface_id 
    };
}

pub fn サーフェス(character, surface_id) {
    yield ChangeSurface { 
        character_name: character.name,
        surface_id 
    };
}

// ウェイト
pub fn ウェイト(duration) {
    // 第一引数は不要（現在の発言者コンテキストから取得）
    yield Wait { duration };
}
```

**暗黙的引数渡しの仕組み**:
```rune
// DSL: さくら：＠笑顔　こんにちは
// トランスパイル後:
yield change_speaker("さくら");
笑顔(さくら);  // グローバル変数 さくら を第一引数として渡す
yield emit_text("こんにちは");
```

**トランスコンパイル例**:

```rune
pub fn 同時発言例() {
    // さくら：＠同時発言開始　せーの
    yield change_speaker("さくら");
    同時発言開始(さくら);  // グローバル変数 さくら を暗黙的に渡す
    yield emit_text("せーの");
    
    // 　　　　＠同期
    同期(さくら);
    
    // うにゅう：＠同時発言開始　せーの
    yield change_speaker("うにゅう");
    同時発言開始(うにゅう);  // グローバル変数 うにゅう を暗黙的に渡す
    yield emit_text("せーの");
    
    // 　　　　＠同期
    同期(うにゅう);
    
    // 　　　　＠同時発言終了
    同時発言終了(うにゅう);
    
    // さくら：＠同時発言終了　（笑）
    yield change_speaker("さくら");
    同時発言終了(さくら);
    yield emit_text("（笑）");
    
    // 　　　　＠笑顔
    笑顔(さくら);
    
    // うにゅう：＠微笑み　息がぴったりやね。
    yield change_speaker("うにゅう");
    微笑み(うにゅう);
    yield emit_text("息がぴったりやね。");
}

**グローバル変数宣言** (初期化時):
```rune
// キャラクターオブジェクトをグローバル変数として宣言
let さくら = #{
    name: "さくら",
    id: 0,
    surfaces: #{
        笑顔: 1,
        微笑み: 2,
        通常: 0,
    },
    state: #{},
};

let うにゅう = #{
    name: "うにゅう",
    id: 1,
    surfaces: #{
        笑顔: 10,
        微笑み: 11,
        通常: 10,
    },
    state: #{},
};
```

**wintf側の処理**:

```rust
// crates/wintf/src/systems/typewriter_sync.rs

pub struct SyncSection {
    sync_id: String,
    characters: HashMap<String, Vec<TypewriterToken>>,
    sync_points: Vec<usize>,
}

impl TypewriterSystem {
    fn process_sync_section(&mut self, tokens: Vec<TypewriterToken>) {
        let mut current_speaker = None;
        let mut sync_buffers = HashMap::new();
        
        for token in tokens {
            match token {
                TypewriterToken::BeginSync { sync_id } => {
                    // 同期セクション開始
                    self.active_sync = Some(sync_id);
                }
                TypewriterToken::ChangeSpeaker(name) => {
                    current_speaker = Some(name);
                }
                TypewriterToken::Text(text) => {
                    if let Some(speaker) = &current_speaker {
                        sync_buffers.entry(speaker.clone())
                            .or_insert_with(Vec::new)
                            .push(text);
                    }
                }
                TypewriterToken::SyncPoint { .. } => {
                    // 同期ポイント：複数バルーンを同時表示
                    self.display_all_balloons(&sync_buffers);
                    sync_buffers.clear();
                }
                TypewriterToken::EndSync { .. } => {
                    // 同期セクション終了
                    self.active_sync = None;
                }
                _ => {}
            }
        }
    }
    
    fn display_all_balloons(&mut self, buffers: &HashMap<String, Vec<String>>) {
        // 複数キャラクターのバルーンを同時に表示
        for (speaker, texts) in buffers {
            let entity = self.get_character_entity(speaker);
            let balloon_text = texts.join("");
            
            // バルーンコンポーネントにテキストを設定
            self.world.entity_mut(entity)
                .get_mut::<BalloonText>()
                .unwrap()
                .set_text(balloon_text);
        }
        
        // すべてのバルーンを同時に表示開始
        self.world.send_message(ShowAllBalloons);
    }
}
```

**使用例**:

1. **漫才のツッコミ**: 「せーの！」の同時発言
2. **合唱**: 複数キャラが同じセリフを同時に
3. **驚きの同期**: 「えっ！？」を複数キャラが一齐に
4. **リアクション**: イベントに対する複数キャラの同時反応

---

### Requirement 7: イベントハンドリング

**Objective:** ゴースト制作者として、ユーザーの操作に応じた会話を記述したい。それによりインタラクティブな体験を実現できる。

#### Acceptance Criteria

1. **When** ユーザーがキャラクターをクリックした時, **the** Script Engine **shall** 対応するイベントハンドラを呼び出す
2. **When** ユーザーがキャラクターをダブルクリックした時, **the** Script Engine **shall** 対話イベントを発火する
3. **The** Script Engine **shall** イベント名でハンドラを定義できる
4. **The** Script Engine **shall** イベント引数（クリック位置、キャラクターID等）をハンドラに渡す
5. **The** Script Engine **shall** 未定義イベントのデフォルトハンドラを設定できる

---

### Requirement 8: Generatorsベース状態マシン

**Objective:** スクリプト開発者として、会話を中断・再開したい。それによりチェイントークや動的な会話フローを実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** Rune Generatorsを使用して実装される
2. **The** Script Engine **shall** 会話の実行を任意の時点で中断（suspend）できる
3. **The** Script Engine **shall** 中断された会話を後で再開（resume）できる
4. **The** Script Engine **shall** 中断時の実行コンテキスト（変数、スタック）を保持する
5. **The** Script Engine **shall** yield式でIRトークンを段階的に生成できる
6. **The** Script Engine **shall** チェイントーク（連続会話）を実装できる
   - 1つの会話が完了した後、次の会話を自動起動
   - 会話間で状態を引き継ぎ可能
7. **The** Script Engine **shall** 会話の実行状態を問い合わせできる
   - 実行中（Running）
   - 中断中（Suspended）
   - 完了（Completed）
8. **The** Script Engine **shall** シンクロナイズドセクションとGeneratorを統合できる
   - 同期制御関数（`同時発言開始`, `同期`, `同時発言終了`）がIRトークンをyield
   - wintf側で複数バルーンの同時表示を制御
   - 原則: 特殊機能は関数呼び出しで実現

#### Technical Details

**yield戦略の設計判断**:

| アプローチ | yield単位 | メリット | デメリット | 推奨度 |
|-----------|----------|---------|-----------|--------|
| **A: トーク単位** | 会話全体を1 yield | ・エンジン側がシンプル<br>・トーク境界が明確 | ・細かい制御不可<br>・長い会話で応答性低下 | ⭐️⭐️⭐️ |
| **B: IR単位（推奨）** | IRトークン毎にyield | ・柔軟な中断ポイント<br>・呼び出し側で分割判断<br>・応答性向上 | ・エンジン側が複雑 | ⭐️⭐️⭐️⭐️⭐️ |
| **C: 文字単位** | 1文字毎にyield | ・最細粒度制御 | ・オーバーヘッド大<br>・実用的でない | ⭐️ |

**採用戦略: IR単位でyield（オプションB）**

理由:
1. **柔軟性**: 呼び出し側が中断タイミングを制御可能
2. **応答性**: 長い会話でもIRトークン毎に制御を返せる
3. **責務分離**: トーク分割判断はエンジン呼び出し側（wintf）の責務
4. **将来性**: ユーザー入力待ち、アニメーション同期等に対応しやすい

**Rune Generatorsの活用**:

```rune
// Pastaスクリプトからトランスコンパイル
pub fn 挨拶() {
    // 各IR操作でyield → 呼び出し側に制御を返す
    yield emit_text("さくら：こんにちは");  // IRトークン1個生成
    yield wait(0.5);                        // IRトークン1個生成
    yield emit_text("元気ですか？");        // IRトークン1個生成
    
    // チェイントーク判定
    if should_chain_talk() {
        yield call("挨拶_続き");  // Generator連鎖
    }
}

pub fn 挨拶_続き() {
    yield emit_text("さくら：今日は良い天気ですね");
}
```

**状態マシン管理**:

```rust
// crates/pasta/src/generator.rs
pub struct ScriptGenerator {
    generator: rune::runtime::Generator,
    state: GeneratorState,
}

pub enum GeneratorState {
    Running,
    Suspended { context: SavedContext },
    Completed,
}

impl ScriptGenerator {
    /// 次のIRトークンを生成（yieldまで実行）
    pub fn resume(&mut self) -> Result<Option<TypewriterToken>, PastaError> {
        match self.generator.resume(())? {
            rune::runtime::GeneratorState::Yielded(value) => {
                self.state = GeneratorState::Suspended { /* ... */ };
                Ok(Some(value.into_any()?))
            }
            rune::runtime::GeneratorState::Complete(_) => {
                self.state = GeneratorState::Completed;
                Ok(None)
            }
        }
    }
    
    /// すべてのIRトークンを生成（完了まで実行）
    pub fn resume_all(&mut self) -> Result<Vec<TypewriterToken>, PastaError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.resume()? {
            tokens.push(token);
        }
        Ok(tokens)
    }
}
```

**チェイントーク実装例**:

```
＊挨拶
　　さくら：おはよう！
　　＠チェイン確率：＠calc_chain_prob（＠＊好感度）
　　？＊挨拶_続き　＠チェイン：有効

＊挨拶_続き
　　＠チェイン：有効
　　さくら：今日も元気だね！
```

**呼び出し側での制御例**:

```rust
// crates/wintf/src/systems/script_system.rs

/// シナリオ1: 一括生成（通常の会話）
fn execute_normal_talk(generator: &mut ScriptGenerator) -> Vec<TypewriterToken> {
    generator.resume_all().unwrap()  // すべてのIRを一括取得
}

/// シナリオ2: フレーム分割（長い会話）
fn execute_with_frame_budget(
    generator: &mut ScriptGenerator,
    max_tokens_per_frame: usize,
) -> Vec<TypewriterToken> {
    let mut tokens = Vec::new();
    for _ in 0..max_tokens_per_frame {
        match generator.resume() {
            Ok(Some(token)) => tokens.push(token),
            Ok(None) => break,  // 完了
            Err(e) => {
                tracing::error!("Generator error: {:?}", e);
                break;
            }
        }
    }
    tokens  // 次フレームで続きをresume()
}

/// シナリオ3: ユーザー入力待ち
fn execute_with_user_input(
    generator: &mut ScriptGenerator,
) -> Vec<TypewriterToken> {
    let mut tokens = Vec::new();
    
    // 入力プロンプトまで実行
    while let Ok(Some(token)) = generator.resume() {
        tokens.push(token.clone());
        
        // 選択肢イベント検出
        if matches!(token, TypewriterToken::FireEvent { 
            event: TypewriterEventKind::UserInput, .. 
        }) {
            break;  // ここで中断、入力後に再度resume()
        }
    }
    
    tokens
}

/// シナリオ4: チェイントーク自動実行
fn execute_chain_talk(
    engine: &mut PastaEngine,
    initial_label: &str,
) -> Vec<TypewriterToken> {
    let mut all_tokens = Vec::new();
    let mut current_label = Some(initial_label.to_string());
    
    while let Some(label) = current_label {
        let mut generator = engine.start_generator(&label).unwrap();
        
        // この会話を完了まで実行
        let tokens = generator.resume_all().unwrap();
        all_tokens.extend(tokens);
        
        // チェイントーク判定
        current_label = engine.check_chain_talk(&label);
    }
    
    all_tokens
}
```

**使用シナリオ**:

1. **通常の会話**: `resume_all()`で一括生成
2. **フレーム分割**: `resume()`を予算分だけ呼び出し、次フレームで続行
3. **ユーザー入力待ち**: IRトークンを監視し、入力イベント検出で中断
4. **チェイントーク**: 会話終了後、条件判定してGeneratorを連鎖起動
5. **アニメーション同期**: TypewriterのIR消化速度に合わせて`resume()`呼び出し

---

### Requirement 9: 関数スコープ解決

**Objective:** ゴースト制作者として、関数呼び出し時にローカル関数とグローバル関数を自動的に検索したい。それにより`＠＊関数名`と明示的に書かなくても、自然にグローバル関数を利用できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 関数呼び出し時に以下の順序でスコープ解決を行う
   - まずローカルスコープ（現在のラベル内定義関数）を検索
   - ローカルに見つからない場合、グローバルスコープ（ファイル全体の関数）を検索
2. **The** Script Engine **shall** `＠関数名`構文でローカル→グローバルの自動検索をサポートする
3. **The** Script Engine **shall** `＠＊関数名`構文で明示的にグローバルスコープのみを検索する（既存機能維持）
4. **The** Script Engine **shall** 関数が見つからない場合、適切なエラーメッセージを提供する
5. **The** Script Engine **shall** ローカル関数がグローバル関数より優先される（シャドーイング）

#### 設計方針

**スコープ解決アルゴリズム**:

```
＠関数名（引数） の場合:
  1. 現在のラベル内のローカル関数を検索
  2. 見つかった → そのローカル関数を呼び出し
  3. 見つからない → グローバル関数を検索
  4. 見つかった → そのグローバル関数を呼び出し
  5. 見つからない → PastaError::FunctionNotFound

＠＊関数名（引数） の場合:
  1. グローバル関数のみを検索
  2. 見つかった → そのグローバル関数を呼び出し
  3. 見つからない → PastaError::FunctionNotFound
```

**使用例**:

```
＊会話
　　```rune
　　// ローカル関数定義
　　fn format_location(loc) {
　　　　"「" + loc + "」"
　　}
　　```
　　
　　さくら：今日は＠format_location（＠＊場所）に行こう！
　　　　　　　　　　　　　　└─ ローカル関数が優先
　　
　　さくら：＠笑顔　楽しみだね！
　　　　　　└─ ローカルになければグローバル（標準ライブラリ）を検索
　　
　　さくら：＠＊グローバル関数（引数）
　　　　　　└─ 明示的にグローバルのみ検索（既存機能）
```

**利点**:
- 記述が簡潔（`＠＊`を毎回書く必要なし）
- ローカル関数でグローバル関数をオーバーライド可能
- 既存の`＠＊`構文も維持（後方互換性）

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. スクリプト解析は起動時に完了すること
2. イベントハンドラの実行は10ms以内に開始すること
3. メモリ使用量は妥当な範囲に抑えること（スクリプトサイズに比例）

### NFR-2: エラーハンドリング

**設計方針**:
- **静的エラー（パース時）**: Rust `Result<T, E>` + `thiserror` による構造化エラー
- **動的エラー（実行時）**: Rune関数による動的な `yield` でエラーIRトークンを返す

#### Acceptance Criteria

1. **The** Script Engine **shall** パース時の構文エラーを `Result<T, PastaError>` で返す
2. **The** Script Engine **shall** エラー位置（ファイル名、行番号、列番号）を含むエラー情報を提供する
3. **The** Script Engine **shall** 実行時エラーを Rune 関数が `yield Error(message)` IRトークンで返す
4. **The** Script Engine **shall** エラーメッセージは制作者が理解しやすいものであること
5. **The** Script Engine **shall** `thiserror` クレートを使用してエラー型を定義する

**エラー型設計例**:

```rust
// crates/pasta/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PastaError {
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    
    #[error("Rune runtime error: {0}")]
    RuneError(#[from] rune::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

**実行時エラー（Rune関数）**:

```rune
// Rune関数内でエラーハンドリング
pub fn call_with_error_handling(label) {
    if !label_exists(label) {
        // エラーIRトークンをyield
        yield Error { message: `ラベル「${label}」が見つかりません` };
        return;
    }
    // 正常処理
    yield call(label);
}
```

### NFR-3: 拡張性

1. 新しいコマンドを追加可能な設計とすること
2. 外部モジュール（MCP等）との連携が可能な設計とすること

---

## Dependencies

### クレート間依存関係

```
pasta (新規サブクレート)
  ├─ rune (外部クレート: Generators, TOML永続化)
  ├─ thiserror (外部クレート: エラー型定義)
  ├─ [パーサー実装] (設計時決定: nom/pest/手書き)
  └─ wintf (共通型定義のみ参照)
       └─ TypewriterToken, TypewriterEventKind

wintf (既存クレート)
  └─ pasta (実行時依存)
       └─ pasta::execute_script()
```

### 外部クレート依存

| クレート | 用途 | 備考 |
|---------|------|------|
| `rune` | スクリプト実行エンジン | Generators, TOML永続化機能 |
| `thiserror` | エラー型定義 | 構造化エラーハンドリング |
| `[parser]` | DSLパーサー | nom/pest/手書き（設計時決定） |

### 依存する仕様

| 仕様 | 依存内容 | クレート |
|------|----------|----------|
| `wintf-P0-typewriter` | **IR型定義の共有**：TypewriterToken, TypewriterEventKind | `wintf` |
| `wintf-P0-animation-system` | サーフェス切り替えの実行（IR経由） | `wintf` |
| `wintf-P0-balloon-system` | テキスト表示の実行（IR経由） | `wintf` |

### 依存される仕様

| 仕様 | 依存内容 | クレート |
|------|----------|----------|
| `areka-P0-reference-ghost` | スクリプト実行 | `pasta` API |
| `areka-P1-devtools` | デバッグ機能 | `pasta` API |
| `areka-P2-llm-integration` | LLM応答との統合 | `pasta` API |

### 公開API

`pasta`クレートは以下のAPIを公開する：

```rust
// エラー型（thiserrorベース）
#[derive(Error, Debug)]
pub enum PastaError {
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    #[error("Rune runtime error: {0}")]
    RuneError(#[from] rune::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// メイン実行関数（Result返却）
pub fn execute_script(
    label: &str,
    args: Vec<RuneValue>,
    filters: HashMap<String, String>,
) -> Result<Vec<TypewriterToken>, PastaError>;

// スクリプトロード（Result返却）
pub fn load_scripts(dic_path: &Path) -> Result<(), PastaError>;

// 変数操作（グローバル変数のみ）
pub fn get_global(name: &str) -> Option<RuneValue>;
pub fn set_global(name: &str, value: RuneValue);

// イベント登録（Result返却）
pub fn register_event(event_name: &str, handler_label: &str) -> Result<(), PastaError>;

// Generator制御（Result返却）
pub struct ScriptGenerator;
impl ScriptGenerator {
    pub fn new(label: &str) -> Result<Self, PastaError>;
    pub fn resume(&mut self) -> Result<Option<TypewriterToken>, PastaError>;
    pub fn resume_all(&mut self) -> Result<Vec<TypewriterToken>, PastaError>;
}
```

**Rust的な設計原則**:
- すべての公開関数は `Result<T, PastaError>` を返す
- エラーは `thiserror` で構造化
- 静的エラー（パース時）は即座に `Err` 返却
- 動的エラー（実行時）は Rune 関数が `yield Error(...)` IRトークン生成

---

## Glossary

| 用語 | 定義 |
|------|------|
| **DSL** | Domain Specific Language。特定用途向けの言語 |
| **里々** | 伺かゴースト制作用の対話記述言語 |
| **さくらスクリプト** | 伺かの標準スクリプト形式 |
| **トーク** | 1つの発言ブロック（会話単位） |
| **サーフェス** | キャラクターの表情・ポーズ画像 |
| **イベントハンドラ** | 特定のイベントに応じて実行される処理 |
