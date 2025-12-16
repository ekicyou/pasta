# Research & Design Decisions: pasta-word-definition-dsl

---

## Summary
- **Feature**: pasta-word-definition-dsl
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - 既存のStatement enumにWordDef variantを追加するパターンが確立されている
  - trie-rs 0.4.2はpredictive_search()で前方一致検索をサポート、日本語（UTF-8）対応済み
  - SpeechPart::VarRefをRuntime解決に拡張することで最小限の変更で実装可能

---

## Research Log

### trie-rs前方一致検索の調査

- **Context**: 単語定義の前方一致検索（例：`＠場所`が`場所`, `場所＿日本`にマッチ）に最適なライブラリを調査
- **Sources Consulted**: 
  - https://docs.rs/trie-rs/latest/trie_rs/
  - https://crates.io/crates/trie-rs
- **Findings**:
  - `trie-rs` 0.4.2がLOUDSベースのメモリ効率的なTrie実装を提供
  - `predictive_search()`メソッドで前方一致検索をサポート
  - 日本語（UTF-8文字列）に対応済み（例：`"すし"`, `"すしや"`等）
  - `map::Trie`を使用すると各単語にValue（単語リスト）を関連付け可能
  - イテレータベースの遅延評価でメモリ効率が良い
  - Rust 1.75.0以降対応、MIT/Apache-2.0デュアルライセンス
- **Implications**:
  - `predictive_search(key)`で前方一致するすべての単語名を取得可能
  - 単語辞書構築時にTrieを事前構築し、検索時に高速参照
  - 依存関係: `trie-rs = "0.4.2"`

### 既存ASTパターンの調査

- **Context**: 新しいStatement::WordDef variantの追加方法を確認
- **Sources Consulted**: 
  - `crates/pasta/src/parser/ast.rs`
  - `crates/pasta/src/transpiler/mod.rs`
- **Findings**:
  - `Statement` enumは現在5つのvariant: `Speech`, `Call`, `Jump`, `VarAssign`, `RuneBlock`
  - 各variantは`span: Span`フィールドを含む
  - Transpilerの`transpile_statement()`でmatchによる振り分け
  - 新variant追加のパターンは`RuneBlock`が最近の例として確立
- **Implications**:
  - `Statement::WordDef`追加は既存パターンに沿った自然な拡張
  - パーサー・トランスパイラの変更は最小限で済む

### SpeechPartフォールバック検索の調査

- **Context**: 会話行内の`＠名前`参照をどのように拡張するか
- **Sources Consulted**: 
  - `crates/pasta/src/parser/ast.rs` (SpeechPart定義)
  - `crates/pasta/src/transpiler/mod.rs` (transpile_speech_part)
- **Findings**:
  - `SpeechPart::VarRef(String)`: 変数参照として実装済み
  - `SpeechPart::FuncCall`: 関数呼び出しとして実装済み
  - 現在の`VarRef`はget_variable()呼び出しに変換される
  - `FuncCall`はcontext.resolve_function()でローカル→グローバル検索
- **Implications**:
  - VarRefの意味を拡張し、Runtime側でフォールバック検索を実装
  - Transpiler生成コードで`resolve_ref()`関数を呼び出すよう変更
  - AST構造変更なし、後方互換性維持

### pest文法拡張の調査

- **Context**: 単語定義構文`＠単語名：単語1　単語2`のパースルール追加方法
- **Sources Consulted**: 
  - `crates/pasta/src/parser/pasta.pest`
- **Findings**:
  - `at_marker`, `colon`, `ident`などの基礎トークンは既存
  - `file`ルールに`global_label`が配置、同様に`word_def_stmt`追加可能
  - `global_label`内に`attribute_line`, `rune_block`, `statement`が配置される構造
  - `indent`で宣言部/実行部を区別可能
- **Implications**:
  - `word_def_stmt`ルールを追加し、`file`と`global_label`に配置
  - 配置制約はパーサーレベルで検証（宣言部/実行部の境界）

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 既存コンポーネント拡張 | Statement enumに新variant追加、既存フローに統合 | 最小限の変更、既存パターン踏襲、学習コスト低 | Transpiler前処理が複雑化 | **採用** |
| Option B: 新規コンポーネント作成 | words/モジュール新設、専用パーサー・リゾルバ | 責務分離明確、テスト容易 | ファイル数増加、初期overhead | 将来リファクタリング候補 |
| Option C: ハイブリッド | Phase 1でOption A、Phase 2で分離 | 段階的改善可能 | 計画複雑性 | Phase 1スコープに集中 |

---

## Design Decisions

### Decision: VarRef拡張によるフォールバック検索

- **Context**: 会話行内の`＠名前`参照で単語辞書・ラベルを検索する方法
- **Alternatives Considered**:
  1. SpeechPart::WordRef新設 — 単語参照専用variant追加
  2. VarRef拡張 — 既存VarRefをRuntime側で多義的に解決
- **Selected Approach**: Option 2（VarRef拡張）
- **Rationale**: 
  - AST構造変更なし、パーサー変更最小限
  - ユーザーには透過的（`＠名前`と書くだけ）
  - 要件のフォールバック検索を自然に実現
- **Trade-offs**: 
  - ✅ 既存コードへの影響最小
  - ✅ 後方互換性維持
  - ❌ VarRefの意味が拡張されるためドキュメント必須
- **Follow-up**: ドキュメントでVarRefの意味拡張を明記

### Decision: Transpiler内前処理でWordDefマージ

- **Context**: 同名単語定義のマージ処理をどこで実行するか
- **Alternatives Considered**:
  1. Transpiler内実装 — transpile_file()冒頭で前処理
  2. Parser内実装 — パース時にマージ
  3. 専用Preprocessor — preprocessor.rs新設
- **Selected Approach**: Option 1（Transpiler内実装）
- **Rationale**: 
  - 既存フローに自然に統合
  - AST構造を変更せずにマージ可能
  - 将来的にpreprocessor.rsへ分離可能
- **Trade-offs**: 
  - ✅ 初期実装がシンプル
  - ❌ Transpilerの責務が増加
- **Follow-up**: 複雑化した際にpreprocessor.rsへリファクタリング

### Decision: trie-rs採用による前方一致検索

- **Context**: 単語辞書の前方一致検索に使用するデータ構造
- **Alternatives Considered**:
  1. HashMap + 線形検索 — シンプルだが効率悪い
  2. trie-rs — LOUDSベースの効率的なTrie
  3. 自作Trie — カスタマイズ可能だが開発コスト大
- **Selected Approach**: Option 2（trie-rs）
- **Rationale**: 
  - predictive_search()で前方一致検索を直接サポート
  - 日本語UTF-8対応済み
  - メモリ効率が良い（LOUDSベース）
  - 安定したライブラリ（0.4.2）
- **Trade-offs**: 
  - ✅ 開発コスト低、信頼性高
  - ❌ 外部依存追加
- **Follow-up**: Cargo.tomlに`trie-rs = "0.4.2"`追加

### Decision: シャッフルキャッシュ機構の実装

- **Context**: 単語のランダム選択で重複を回避するキャッシュ戦略
- **Alternatives Considered**:
  1. 単純ランダム選択 — 毎回ランダム、重複あり
  2. シャッフルキャッシュ — シャッフル後順次消費、枯渇時再シャッフル
- **Selected Approach**: Option 2（シャッフルキャッシュ）
- **Rationale**: 
  - 同じ単語が連続で選ばれることを防止
  - 全単語を一巡してから再シャッフル
  - 要件の「ランダム選択」を高品質に実現
- **Trade-offs**: 
  - ✅ ユーザー体験向上（単調さ回避）
  - ❌ キャッシュ管理のオーバーヘッド
- **Follow-up**: 
  - ローカルキャッシュ: グローバルラベル終了時にクリア
  - グローバルキャッシュ: セッション永続、手動クリア関数提供

---

## Risks & Mitigations

- **Transpiler前処理の複雑化** — Phase 1で最小限の実装に留め、必要に応じてリファクタリング
- **trie-rs依存追加** — 安定版0.4.2を使用、ライセンス（MIT/Apache-2.0）はプロジェクトと互換
- **VarRef意味拡張による混乱** — ドキュメントで明確に説明、エラーメッセージで検索順序を明示
- **配置制約の複雑性** — パーサーレベルで明確なエラーメッセージを出力

---

## References

- [trie-rs公式ドキュメント](https://docs.rs/trie-rs/latest/trie_rs/) — 前方一致検索API、日本語対応
- [pest PEGパーサー](https://pest.rs/) — 文法定義の参考
- gap-analysis.md — 決定事項の詳細記録
