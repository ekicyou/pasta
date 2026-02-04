# Requirements Document

## Project Description (Input)
ACT:build() / SHIORI_ACT:build()の早期打ち切りパターン。ACT:build()にて、最初に撮影トークンの蓄積を確認し、０件だったらnilを返してください。SHIORI_ACT:build()は、ACT:build()がnilを返したなら、スクリプト作成処理に入らずにnilをリターンしてください。これは、会話が作成されなかったことをnil応答で検出するための仕様です。

## Requirements

### Context

現状の実装では、ACT:build()は撮影トークンが0件でも空の配列を返し、SHIORI_ACT:build()は必ずスクリプト生成処理に入ります。これにより、会話未作成の状態を検出する手段がなく、不要な処理が実行される可能性があります。

この機能により、撮影トークンが0件の場合にnilを返すことで、呼び出し元が会話未作成を検出し、早期リターンによる処理の最適化が可能になります。

**関連ファイル:**
- `crates/pasta_lua/scripts/pasta/act.lua` - ACT_IMPL.build()実装
- `crates/pasta_lua/scripts/pasta/shiori/act.lua` - SHIORI_ACT_IMPL.build()実装

**依存関係:**
- ACT:build()の戻り値が`table[]`から`table[]|nil`に変更される
- SHIORI_ACT:build()の戻り値が`string`から`string|nil`に変更される

### Functional Requirements

#### R1: ACT:build()の早期リターン（撮影トークン0件時）
**When** ACT:build()が呼び出され、撮影トークン（self.token）が0件である場合、**the** ACT:build() **shall** トークンのグループ化処理を実行せず、nilを返す。

**詳細仕様:**
- 撮影トークン数の判定条件: `#self.token == 0`
- nilリターン後も`self.token`は空テーブル`{}`にリセットする（状態の一貫性維持）
- 1件以上のトークンが存在する場合は、現状通りグループ化・統合処理を実行する

#### R2: SHIORI_ACT:build()の早期リターン（ACT:build()がnil時）
**When** SHIORI_ACT:build()が呼び出され、親のACT.IMPL.build(self)がnilを返した場合、**the** SHIORI_ACT:build() **shall** さくらスクリプト生成処理（BUILDER.build()）を実行せず、nilを返す。

**詳細仕様:**
- ACT.IMPL.build(self)の戻り値がnilであることを明示的に検証する（`== nil`）
- nilリターン前に追加の処理（ログ出力等）は行わない
- tokenがnilでない場合は、現状通りBUILDER.build()で変換処理を実行する

#### R3: 会話未作成の検出可能性
**The** シーン関数内部 **shall** SHIORI_ACT:build()がnilを返した場合に、会話が未作成であることを検出できる。

**詳細仕様:**
- 戻り値の型: `string|nil`（型アノテーションを更新）
- シーン関数は必要に応じてnil検証を行う（`if script == nil`）
- nil検出後の処理はシーン作者の責任（本機能のスコープ外）
- **注記:** OnTalk/OnHourイベントハンドラはコルーチン（thread）を返すだけであり、act:build()は呼ばない

### Non-Functional Requirements

#### NFR1: パフォーマンス最適化
- 撮影トークン0件時に、不要なグループ化処理（group_by_actor, merge_consecutive_talks）をスキップすることで、CPU時間を削減する
- さくらスクリプト生成処理（BUILDER.build()）のスキップにより、メモリ割り当てを削減する

#### NFR2: テストリグレッション対応
- 既存テストケース（act_test.lua: 20件以上、shiori_act_test.lua: 22件）が非nil前提で記述されている
- 既存テストは引き続きパスすることを確認（トークンありケース）
- 新規テストケース（トークン0件時のnilリターン）を追加
- 型アノテーション（@return）を更新：`table[]|nil`, `string|nil`

### Constraints & Assumptions

**制約:**
- 既存テストケースが非nil前提で記述されているため、リグレッション確認が必要
- シーン関数内でSHIORI_ACT:build()の戻り値がnil可能性を考慮する必要がある（将来的な設計ガイドライン）

**前提:**
- 撮影トークン0件 = 会話未作成と定義する
- nilリターンは正常系の一部であり、エラー状態ではない
- 既存のエラーハンドリング（例: scene_result.funcが関数でない場合のnil）とは異なる用途でnilを使用する

### Acceptance Criteria

#### AC1: ACT:build()の早期リターン動作確認
- ✅ 撮影トークンが0件の場合、nilを返す
- ✅ 撮影トークンが1件以上の場合、現状通りグループ化された配列を返す
- ✅ nilリターン後も`self.token`が空テーブル`{}`にリセットされている

#### AC2: SHIORI_ACT:build()の早期リターン動作確認
- ✅ ACT.IMPL.build(self)がnilを返した場合、nilを返す
- ✅ ACT.IMPL.build(self)が配列を返した場合、現状通りさくらスクリプト文字列を返す
- ✅ BUILDER.build()がnilリターン時に呼び出されない（パフォーマンステストで確認）

#### AC3: 型アノテーション更新
- ✅ ACT_IMPL.build()の@returnが`table[]|nil`に更新されている
- ✅ SHIORI_ACT_IMPL.build()の@returnが`string|nil`に更新されている

#### AC4: 既存テストの互換性確認
- ✅ 既存のACT:build()テストケースが引き続きパスする
- ✅ 既存のSHIORI_ACT:build()テストケースが引き続きパスする
- ✅ 新規テストケース（トークン0件時のnilリターン）が追加されている

#### AC5: ドキュメント・型アノテーション整合性
- ✅ init.lua:40のドキュメント例が、build()の戻り値がnilとなる可能性を記載
- ✅ 既存シーン関数がnilを考慮しない場合でもクラッシュしない（nil連結等のエラーはシーン作者の責任）

