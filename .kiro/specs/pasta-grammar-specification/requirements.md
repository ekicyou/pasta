# 要件定義書

## プロジェクト説明（入力）

Pasta DSLの文法仕様が、現在の実装（pest定義）と乖離しているため、文法設計を再度整理し直す必要があります。現在のpest定義から逆算して、あるべきパスタスクリプト文法を検討し、その過程で破壊的変更（DSL文法の変更）が発生します。

**重要な注記**: 本仕様では、文法仕様の再検討・整理に伴う破壊的変更に対応するテストの修正・リグレッション対応もスコープに含めます。リグレッション対応が大規模になる可能性があることに留意してください。

**成果物**:
1. 正確な文法仕様 — pest定義の詳細コメント化、または開発者向け技術リファレンス
2. GRAMMAR.md — 一般ユーザー向け文法説明ドキュメント（破壊的変更を反映）
3. テスト修正・リグレッション対応レポート — 破壊的変更に伴う既存テストの修正状況

---

## 要件

### 1. Pest文法定義と実装意図の分析

**目的**: パーサー開発者として、現在の`pasta.pest`による実装定義を完全に把握し、各ルールの意図と制約を明確にしたい。これにより、あるべき文法仕様の検討基盤を得られます。

#### 受け入れ基準
1. When analyzing `src/parser/pasta.pest`, the 開発者 shall identify all production rules including ファイル構造、ラベル、ステートメント、マーカー、式
2. The 開発者 shall document the supported character categories including Unicode空白、全角/半角文字、識別子 in the implementation context
3. The 開発者 shall map each rule's semantic purpose to its usage intent in the Pasta DSL
4. The 開発者 shall verify that pest rules correctly handle edge cases including 行継続、rune ブロック、埋め込み式
5. If contradictions exist between pest comments and implementation rules, the 開発者 shall report them as deviation candidates

### 2. 文法仕様の再検討と理想形設計

**目的**: 保守性と拡張性に優れた、あるべきパスタスクリプト文法を設計したい。現在のpest実装と理想的なDSL設計のギャップを分析し、破壊的変更の必要性と範囲を明確にします。

#### 受け入れ基準
1. The 文法仕様チーム shall document the constraint conditions extracted from the current pest definition
2. The 文法仕様チーム shall propose the ideal form of Pasta DSL from perspectives of ユーザビリティ、拡張性、保守性
3. The 文法仕様チーム shall enumerate all locations where breaking changes are necessary between current implementation and ideal form
4. The 文法仕様チーム shall document the rationale (なぜ変更が必要か) for each breaking change
5. The 文法仕様チーム shall estimate the impact range of each breaking change on 既存スクリプト、テスト、トランスパイラ

### 3. GRAMMAR.md の乖離箇所の特定と評価

**目的**: ドキュメント維持者として、GRAMMAR.mdの現在の記述と実装（pest定義）および設計（あるべき仕様）との食い違いを明確にしたい。改善対象を系統的に把握し、優先度を決定します。

#### 受け入れ基準
1. The ドキュメント分析者 shall identify productions that are missing from GRAMMAR.md but present in pest rules
2. The ドキュメント分析者 shall verify that GRAMMAR.md contains no incorrect syntax descriptions
3. The ドキュメント分析者 shall flag features in GRAMMAR.md that exist but are not reflected in pest rules as "未実装機能"
4. The ドキュメント分析者 shall enumerate features that exist in pest rules but are not documented in GRAMMAR.md as "未文書化機能"
5. The ドキュメント分析者 shall clarify implementation scope and design gaps for features that are only partially implemented in GRAMMAR.md

### 4. 破壊的変更の影響範囲分析とテスト修正計画

**目的**: テスト責任者として、文法仕様の破壊的変更に伴うリグレッション対応を明確にしたい。既存テストの修正が必要な範囲を把握し、修正戦略を立案します。

#### 受け入れ基準
1. The テスト分析者 shall traverse test code under `tests/` and enumerate all tests affected by breaking changes
2. The テスト分析者 shall determine whether each affected test requires "修正"、"削除"、or "新規作成"
3. When test modification is necessary, the テスト分析者 shall document the specific changes including 変更前後の構文、期待値 etc
4. The テスト分析者 shall apply modification to test fixtures in `tests/fixtures/*.pasta` to support breaking changes
5. The テスト分析者 shall verify that all tests pass under the new specification after modification

### 5. 実装仕様に基づく正確な文法説明の作成

**目的**: パーサー開発者として、あるべき文法仕様を完全に反映した正確な文法説明が必要です。他の開発者が仕様の意図を理解し、拡張・保守が容易になるようにします。

#### 受け入れ基準
1. The 文法リファレンス shall describe all production rules from the final pest implementation that reflects breaking changes
2. The 文法リファレンス shall clarify supported character types including Unicode categories and marker symbols for each rule
3. The 文法リファレンス shall provide concrete syntax examples and annotations for statement types including 発言、単語定義、制御フロー
4. When rules include branching or choice options, the 文法リファレンス shall clearly explain conditions and precedence
5. The 文法リファレンス shall distinguish between required and optional elements for each rule component

### 6. 一般ユーザー向け GRAMMAR.md ドキュメントの改訂

**目的**: Pastaユーザーとして、DSL文法を直感的に理解できるドキュメントが必要です。新しい文法仕様に基づき、スクリプト記述の学習曲線を下げ、バグを削減します。

#### 受け入れ基準
1. When users read the core syntax sections of GRAMMAR.md (ラベル、発言、ステートメント), the document shall explain purpose before syntax detail
2. The GRAMMAR.md shall provide 3 or more practical examples covering common use cases for each statement type including 発言、制御、変数
3. When features have multiple variations (全角/半角マーカー、optional parameters), the GRAMMAR.md shall emphasize recommended usage
4. When pest defines complex precedence or edge case handling, the GRAMMAR.md shall provide clear guidance in a "落とし穴" section
5. The GRAMMAR.md shall be organized based on user intent (e.g., "対話を作成する方法"、"フロー制御の方法"、"単語定義の方法") rather than pure syntax order

### 7. 実装と仕様の同期メカニズムの構築

**目的**: 保守性向上のため、pest定義とドキュメントの同期を支援する仕組みが必要です。将来の仕様変更時に乖離を最小化できます。

#### 受け入れ基準
1. The 同期メカニズム shall enable inline comments in `pasta.pest` to directly reference GRAMMAR.md sections or version information
2. When `pasta.pest` is modified, the メカニズム shall flag related GRAMMAR.md sections as requiring review
3. The メカニズム shall provide automated reporting when pest rules lack corresponding documentation
4. The メカニズム shall provide checklist or template to support documentation updates when grammar is extended
5. The メカニズム shall document version coupling between pest rules and GRAMMAR.md sections

### 8. 実装成果物の検証と品質確保

**目的**: 要件定義者として、改訂後のドキュメント、破壊的変更、テスト修正が正確に実装されていることを確認し、将来の機能追加や保守時の信頼性を確保します。

#### 受け入れ基準
1. When developers create new Pasta scripts using GRAMMAR.md, they shall be able to create correct scripts without referencing source code
2. All pest rules documented in GRAMMAR.md shall compile and execute without error under existing test fixtures
3. When GRAMMAR.md describes syntax variations (全角 vs 半角 etc), the document shall include examples of all documented variations
4. The ドキュメント検証プロセス shall cross-reference rule descriptions with actual pest implementation at least once
5. When validation discovers inconsistencies, the メカニズム shall provide reports identifying sections requiring correction

---

## スコープに関する特記事項

**破壊的変更への対応**:
- 本仕様の実装過程で、Pasta DSL の文法仕様に変更が加えられることがあります
- その結果、既存のテスト、テストフィクスチャ、トランスパイル結果が無効になる可能性があります
- テスト修正は本仕様のスコープに含まれ、修正後すべてのテストが成功する状態を目標とします

**リグレッション対応の複雑性**:
- テスト修正が大規模になる可能性があります（複数ファイル、多数のケース）
- 修正の優先順位付け、段階的対応を設計段階で検討することを推奨します
