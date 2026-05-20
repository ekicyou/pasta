# Research & Design Decisions

## Summary
- **Feature**: property-write-helpers
- **Discovery Scope**: Extension（既存actトークンバッファシステムへのメソッド追加）
- **Key Findings**:
  1. `raw_script` トークンはアクターグループ不在時にドロップされるため、単独 `set_property` 呼び出しに使えない
  2. SSPさくらスクリプトのエスケープ規則は5種: `\`→`\\`, `%`→`\%`, `]`→`\]`, `,`→`""` 囲み, `"`→`""`
  3. 既存のエスケープユーティリティは存在しない（raw_scriptは意図的にエスケープなしでパススルーする設計）

## Research Log

### raw_script トークンのアクターグループ依存性
- **Context**: AC 1-4（set_propertyのみ・talkなしでの単独出力）の実現可能性調査
- **Sources**: `pasta_scripts/pasta/act.lua` L42-56 `group_by_actor()` 実装
- **Findings**:
  - `raw_script` は `talk`/`sakura_script` と異なりアクターフィールドを持たない
  - `group_by_actor` は `talk`/`sakura_script` でアクターグループを開始し、それ以外のトークンは現在のアクターグループに追加する
  - `current_actor_token` が nil の場合（＝まだ talk/sakura_script が来ていない場合）、raw_script を含む非アクタートークンは**無視される**
  - コメントに「現在の設計ではこの状況は発生しない」と明記
- **Implications**: `raw_script` を使う設計ではAC 1-4を満たせない。専用トークン型が必要。

### SSPさくらスクリプトエスケープ規則
- **Context**: AC 2-5（tostring後の特殊文字エスケープ）の設計根拠
- **Sources**: https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html「さくらスクリプトのエスケープ」セクション
- **Findings**:
  - `\` → `\\`（さくらスクリプト開始記号のエスケープ）
  - `%` → `\%`（環境変数埋め込みタグのエスケープ）
  - `]` → `\]`（スクウェアブラケット内のタグ引数でのみ）
  - `,` → 引数全体を `""` で囲む（第2引数以降で引数内容にカンマを含む場合）
  - `"` → `""` 二重化（`""` 囲み内部での引用符のエスケープ）
- **Implications**: `\![set,property,<name>,<value>]` タグでは name と value の両方が `[]` 内の引数。エスケープ対象は `\`, `%`, `]`, および `,`/`"` を含む場合のクォーティング。

### 既存トークン型の独立出力パターン
- **Context**: set_property トークンの group_by_actor での扱い設計
- **Sources**: `pasta_scripts/pasta/act.lua` `group_by_actor()` 実装
- **Findings**:
  - `spot` と `clear_spot` は アクターグループとは独立して result テーブルに直接追加される
  - `sakura_builder.build()` ではこれらを最上位トークンとして処理
  - このパターンをそのまま `set_property` に適用可能
- **Implications**: 新トークン型 `set_property` を spot/clear_spot と同じカテゴリで扱えば、アクターグループ不在でも出力される。

## Design Decisions

### Decision: 専用トークン型を使わず raw_script として蓄積
- **Context**: set_property の出力を専用トークン型で管理するか、raw_script トークンとして扱うか
- **Alternatives Considered**:
  1. 専用 `set_property` トークン型 → sakura_builder にエスケープ処理と変換ロジックが必要、ビルダーが知る必要のない似た処理の追加
  2. `raw_script` トークンとして蓄積 → set_property メソッドがエスケープ・タグ組み立てまで完了、ビルダーは既存の raw_script 処理で対応
- **Selected Approach**: 選択肢2 — raw_script トークン方式
- **Rationale**: ビルダーの責務を不必要に拡張しない。raw_script は「生のさくらスクリプトをそのまま出力する」という意図であり、組み立て済みタグ文字列の出力にそのまま合致。raw_script のバグ修正により単独出力も可能に。
- **Trade-offs**: エスケープ処理が act.lua に配置されるが、タグ組み立てとエスケープは同一責務なので自然な配置
- **Follow-up**: 将来的に他のSSPタグ生成メソッドが追加された場合、escape_tag_arg の共有化を検討

### Decision: raw_script バグ修正のスコープ包含
- **Context**: raw_script トークンがアクターグループ不在時にドロップされるバグ
- **Selected Approach**: 本specのスコープに含める
- **Rationale**: set_property が raw_script として蓄積する設計のため、このバグの修正は前提条件。また close_ghost 等の既存機能の信頼性も向上。
- **Modification**: group_by_actor でのハイブリッド分岐（アクターグループ存在時はグループ内、不在時は独立出力）+ sakura_builder での最上位 raw_script ハンドリング追加

### Decision: エスケープ処理の責務配置（改訂）
- **Context**: name/value 引数に含まれる SSP 特殊文字のエスケープ
- **Previous Decision**: sakura_builder 内でエスケープ → **撤回**
- **Selected Approach**: act.lua 内の escape_tag_arg ローカル関数
- **Rationale**: 専用トークン型を使わないため、ビルダーにエスケープ責務を持たせる理由がない。タグ文字列の組み立てとエスケープは同一の関心事であり、set_property メソッドの近傍に配置するのが自然。

### Synthesis: 一般化・簡素化の判断
- **Generalization**: set_property は「アクター非依存のSSPコマンド出力」の一事例。raw_script トークンとしての蓄積で、トークン型の一般化は不要。raw_script のバグ修正でアクター非依存出力の基盤を整備。
- **Build vs. Adopt**: 全コンポーネントは内部 Lua スクリプト。外部依存なし。
- **Simplification**: エスケープ関数は act.lua 内のローカル関数として実装。独立モジュール化は不要。ビルダーに新しいトークン型の知識を持たせず、既存の raw_script パイプラインを活用。
