# 実装ギャップ分析: pasta-call-resolution-priority

## 分析サマリ

- パーサは「単語定義（グローバル/ローカル）」と「会話行内の `＠単語`」を既に構文として扱えている一方、ランタイムへ辞書を渡す経路が未実装で、実行時の展開は stdlib の `word` がスタブに留まっている。
- エンジンは複数 `.pasta` をマージするが、`global_words` の統合が TODO のままで、辞書が実行時に届かない。
- トランスパイラは `SpeechPart::FuncCall` を `pasta_stdlib::word(ctx, "<name>", [args])` に変換しているが、本仕様が要求する `pasta::word(global, local)`（global label 文脈 + 単語名）とシグネチャ/意味が一致していない。
- 既存の「prefix search + フィルタ + キャッシュ + シャッフル + 枯渇」パターンはラベル解決で実装済みであり、単語辞書の検索・キャッシュにも流用できる。

## ドキュメントステータス

- `.kiro/settings/rules/gap-analysis.md` のフレームワークに従い、(1) 現状調査 → (2) 要件の技術ニーズ抽出 → (3) ギャップ/制約/不明点の明示 → (4) Option A/B/C の提示、の順で整理した。

## 1. 現状調査（Current State Investigation）

### 主要資産（domain-related assets）

- パース（AST）
  - グローバル単語定義: `PastaFile.global_words: Vec<WordDef>`
  - ローカル単語定義: `LabelDef.local_words: Vec<WordDef>`
  - 文法: `global_word_def = at_marker ~ word_name ~ colon ~ word_value_list ~ NEWLINE`
- エンジン統合
  - 複数 `.pasta` を読み込み、ラベルをマージしてからトランスパイルする
  - ただし `global_words: Vec::new() // TODO: Merge global words from all files` のまま
- トランスパイル
  - 会話行中の `SpeechPart::FuncCall` を `yield pasta_stdlib::word(ctx, "<name>", [args]);` に変換
  - `PastaFile.global_words` / `LabelDef.local_words` を参照して辞書登録する処理は見当たらない
- ランタイム/stdlib
  - stdlib に `word` は登録されているが、P0スタブとして「単語名をそのまま Talk(text)」で返す
  - ラベル解決（`LabelTable`）は RadixMap による prefix 検索 + キャッシュ + シャッフル（枯渇まで順次消費）を実装

### 命名・配置などの慣習（conventions）

- 構造は「parser → transpiler → runtime/stdlib → engine」が中心
- テストは `crates/pasta/tests/` に配置され、構文（単語定義/会話行）を含むパースのテストが存在
- ランタイム側の状態は `LabelTable` のように Engine インスタンスに紐づく所有（スレッド安全は `Mutex` 包装等）で扱う傾向

### 統合サーフェス（integration surfaces）

- `PastaEngine` が Rune `Context` に stdlib モジュールを `install` している
- stdlib 側で単語辞書を参照するには、(a) `create_module(...)` の引数で辞書を渡す、または (b) Rune 側 `ctx` に辞書を保持する等の設計が必要

## 2. 要件の技術的ニーズ（Requirements Feasibility Analysis）

EARS要件から必要になる実装要素:

- AST から辞書（`HashMap<String, Vec<String>>` 等）を構築し、複数ファイル分をマージする
- グローバル/ローカルキーの生成ルール
  - グローバル: `<単語名>:`
  - ローカル: `<グローバルラベル_解決後>:<単語名>:`
- 変換（トランスパイル）
  - 会話行内の `＠<単語名>` を `pasta::word(global, local)` 呼び出しへ置換
  - `global` 引数として「現在のグローバルラベル（解決後モジュール名）」を確定させる
  - `＠` の直後に空白が続く場合の切り分け（単語名に空白を含めない）
- ランタイムの単語解決
  - キャッシュ（検索キー単位）
  - 候補抽出（`<local>:` と `<global>:<local>:` の両方）
  - シャッフル → 1要素ずつ返す → 枯渇でキャッシュ削除
- 診断
  - 不正入力時に「登録/変換しない + 診断可能なエラー」を返す

## 3. Requirement-to-Asset Map（ギャップ明示）

凡例: **[Missing]** 未実装 / **[Constraint]** 既存設計上の制約 / **[Unknown]** 要調査

### Requirement 1: 単語辞書の登録（グローバル/ローカル）

- 既存資産
  - パーサが `global_word_def` を `PastaFile.global_words` へ取り込む（`WordDef { name, values }`）
  - パーサがローカル単語定義を `LabelDef.local_words` として保持するテストが存在
- ギャップ
  - **[Missing]** `PastaEngine` の AST マージで `global_words` を統合していない（TODO のまま）
  - **[Missing]** 収集した `global_words/local_words` を「実行時に参照可能な辞書」に変換・受け渡しする経路がない
  - **[Constraint]** 要件は「トランスパイラ1pass目で登録」と書かれているが、現状は「パーサでAST化→エンジンで統合→トランスパイル」という分割が支配的（要件の言い回しを、実装では“ASTから辞書構築”として解釈するのが自然）
  - **[Unknown]** 「ローカル単語登録行がグローバルラベル文脈なしで現れる」ケースの現状挙動（文法で排除される/別ルールに解釈される/パースエラーになる）

### Requirement 2: 会話行の単語呼び出しの変換

- 既存資産
  - `＠挨拶` のような発話行はパースできる（`test_speech_line_with_word`）
  - トランスパイラが `SpeechPart::FuncCall` を `pasta_stdlib::word(...)` に変換する経路は存在
- ギャップ
  - **[Missing]** 変換先が `pasta::word(global, local)` になっていない（現状は `word(ctx, name, args)`）
  - **[Missing]** `global`（グローバルラベル解決後モジュール名）を word 呼び出しに渡していない
  - **[Unknown]** 「インライン単語参照の直後の任意空白を区切りとして扱う」要件が、現在のパース（speech content の分割）で満たせるか

### Requirement 3: `pasta::word(global, local)` の提供

- 既存資産
  - stdlib モジュールに `word` が登録済み
- ギャップ
  - **[Missing]** シグネチャ/責務が要件と不一致（P0スタブ: 単語名をそのまま返す）
  - **[Missing]** 辞書検索・候補抽出・キャッシュ・シャッフルが無い
  - **[Constraint]** Rune から呼ぶ関数が「値（ScriptEvent）を1つ返す」のか「generator 的に複数 yield する」のか、既存イベント流（`yield Talk(...)`）との整合を設計で決める必要がある

### Requirement 4: 単語検索・抽出・キャッシュ

- 既存資産
  - `LabelTable` が「候補抽出→シャッフル→枯渇まで順次消費→キャッシュ保持」の骨格を実装している
- ギャップ
  - **[Missing]** 単語用の辞書・キャッシュ実装は存在しない
  - **[Constraint]** 乱択の注入点（テスト容易性）の方針はラベル解決の `RandomSelector` と合わせるのが自然

### Requirement 5: 入力の堅牢性

- 既存資産
  - パーサは `colon` を必須とするため、`：` が無い単語定義は構文上弾かれる可能性が高い
  - 予約ラベル名など、一部は `ParseError { line, column, message }` のように診断しやすい形で返している
- ギャップ
  - **[Missing]** 単語定義特有の診断メッセージ（「単語名が空」「値リストが空」等）を、要件どおり“診断可能”に整える処理がない
  - **[Unknown]** 「値リストが空」を現状文法で許すか（`word_value_list` が1要素必須に見えるため、入力自体がパースエラーになる可能性）

## 4. 実装アプローチの選択肢（Options）

### Option A: 既存コンポーネント拡張（Extend Existing Components）

- 変更候補
  - エンジン: 複数ファイルから `global_words` をマージ（既存 TODO 解消）
  - エンジン or stdlib: AST から `HashMap<String, Vec<String>>`（または専用構造）を構築し、`create_module(...)` に渡す
  - stdlib: `word` を「辞書検索 + キャッシュ + シャッフル + 枯渇」実装に置換
  - トランスパイラ: `word` 呼び出しを `global,label` 形式へ変更（現コンテキストで“今のグローバルラベル解決後名”を渡す）
- トレードオフ
  - ✅ 既存フロー（engine→stdlib install）に沿うので統合が素直
  - ✅ ラベル解決で確立済みのキャッシュ/シャッフル設計を流用可能
  - ❌ engine/transpiler/stdlib を同時に触るため差分が広い

### Option B: 単語辞書専用コンポーネント新設（Create New Components）

- 変更候補
  - `crates/pasta/src/runtime/words.rs` などに `WordTable`（辞書本体）と `WordResolver`（検索+キャッシュ）を新設
  - stdlib の `word` は `WordResolver` を呼ぶ薄いラッパにする
  - engine は `WordTable` を構築して stdlib へ注入
- トレードオフ
  - ✅ ラベル解決と同様に責務を分離でき、テストしやすい
  - ✅ stdlib の肥大化を抑えられる
  - ❌ 新規モジュール増加、インターフェイス設計が必要

### Option C: ハイブリッド（Hybrid）

- 方針
  - まず Option A の最小変更で要件を満たし（辞書注入・word本実装・トランスパイル修正）、後で `runtime::words` に抽出して Option B の形に整理する
- トレードオフ
  - ✅ 早期に動くものを作りつつ、整理の余地を残せる
  - ❌ 短期的に二度手間になり得る（初期実装と抽出の2段階）

## 5. 工数とリスク（Effort & Risk）

- Effort: **M (3–7 days)**
  - 根拠: engine（マージ/注入）+ transpiler（生成コード変更）+ stdlib/runtime（辞書/キャッシュ/乱択）+ テスト更新が必要
- Risk: **Medium**
  - 根拠: Rune への関数公開シグネチャ変更と、スクリプト実行時のイベント生成（yield）モデル整合が必要

## 6. Designフェーズへ持ち越す調査項目（Research Needed）

- `＠単語`（インライン参照）のパース結果が常に `SpeechPart::FuncCall` になるか、空白や記号を含むケースの挙動
- “グローバルラベル解決後モジュール名”の確定方法（トランスパイル時に `module_name` を `word(global, local)` に渡す実装方針）
- `word` が返す型/挙動（ScriptEventを1つ返すのか、generatorで複数yieldするのか）を、既存の `yield Talk(...)` 流と揃える設計
- 「値リストが空」の入力を許容するか（文法変更で許す/エラーにする/空登録にする）
- キャッシュのライフサイクル（Engineインスタンス単位、スレッド安全性、再入性）

## 次のステップ

- このギャップ分析を踏まえて `/kiro-spec-design pasta-call-resolution-priority` を実行し、(1) 辞書データ構造、(2) 注入点（engine→stdlib）、(3) 変換方針（transpiler出力）を確定するのがよい。
