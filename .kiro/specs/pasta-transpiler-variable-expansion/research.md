# Research & Gap Analysis: pasta-transpiler-variable-expansion

## Summary
- **Feature**: pasta-transpiler-variable-expansion
- **Discovery Scope**: Extension（既存のParser/Transpiler/Runtimeへの拡張）
- **憲法レベル決定**: `ctx.local`（ローカル変数）、`ctx.global`（グローバル変数）を正式APIとして採用。現行`ctx.var.*`、`get_global()`、`set_global()`は全て変更対象。
- **Key Findings**:
  - Parserは`dollar_marker`（"＄"/"$")、`at_marker`（"＠"/"@"）、`call_marker`（"＞"/">"）を定義し、`speech_content`に`var_ref`/`func_call`を許容しているため、インライン参照の構文要素は存在する。
  - Runtimeには`VariableManager`（Local/Global/Systemスコープ）と`VariableValue`が既にあり、変数値の保持・参照の土台がある。
  - Transpilerは`SpeechPart::VarRef`をTalkに展開、`FuncCall`をword検索に展開、`Statement::Call`を`crate::pasta::call(ctx, scene, filters, args)`に展開するが、変数代入とグローバル参照の扱いは未整備箇所がある（`VarAssign::Global`が`ctx.var.<name>`へ代入する体裁／旧式では`set_global()`）。

## Requirement-to-Asset Map（Gaps tagged）
- 1. 変数スコープ管理（Local/Global）
  - Parser: `VarScope`あり、`Expr::VarRef { name, scope }`あり → 部分充足
  - Runtime: `VariableManager`あり（Local/Global/System）→ 充足
  - Transpiler: `SpeechPart::VarRef`は`ctx.var.<name>`文字列展開、`Expr::VarRef(Global)`は`get_global("name")`（旧式系）→ 不整合（Missing/Constraint）
- 2. 変数代入 `$変数: 値` / `$*変数: 値`
  - Parser: `Statement::VarAssign { name, scope, value }`あり → 充足
  - Runtime: `VariableManager::set()`あり → 充足
  - Transpiler: Localは`let name = expr`／Globalは`ctx.var.name = expr`（新式）と旧式`set_global()`の併存 → Gap: 実ランタイム側の`ctx.var`構造定義と一致させる必要（Missing）
- 3. アクション行 `$変数` → Talk展開
  - Transpiler（新式）: `yield Talk(`${{ctx.var.<name>}}`);` → 文字列式展開形式（Rune内テンプレ）
  - Runtime/stdlib: `Talk(text)`/`emit_text()`あり → 充足
  - Gap: Local変数の参照先（`ctx.local` or 直接`let`）の一貫性（Constraint）
- 4. アクション行 `@$変数` → word検索
  - Parser: `func_call`として解釈される設計（`SpeechPart::FuncCall`）／`@`は`at_marker`
  - Transpiler: `FuncCall`を`pasta_stdlib::word(module, name, [])`へ展開 → 充足
  - Gap: `@$変数`の場合の`name`が変数値である動的キーに対応する必要（Missing：現状はリテラル`name`）
- 5. コール行 `>$変数` → シーン呼び出し
  - Parser: `jump_target`に`Dynamic(var_name)`あり → 充足
  - Transpiler: `transpile_jump_target_to_search_key(Dynamic)`が`@var_name`フォーマットへ変換 → 充足（ただし`scene_selector`側で動的解決が必要）

## Gaps & Constraints（詳細）
- **✅ Resolved（憲法決定）**: `ctx.local`/`ctx.global`を正式APIとして採用。`VariableManager`を`ctx`に統合し、`ctx.local.<name>`/`ctx.global.<name>`で直接アクセスする設計とする。
- Missing: `@$変数`を「変数値をキーとするword検索」へ展開するため、`SpeechPart::FuncCall`生成時に`name`を変数参照で埋める（`word(module, get_variable(name), [])`相当）のサポート。
- **✅ Resolved（憲法決定）**: 旧式API（`get_global`, `set_global`）は廃止対象。新式は`ctx.local`/`ctx.global`テンプレ展開に統一。
- **✅ Resolved（憲法決定）**: Local変数は`ctx.local.<name>`、Global変数は`ctx.global.<name>`で参照。`let`束縛は使用せず、全て`ctx`経由とする。

## Implementation Approach Options
- Option A: 既存Transpiler拡張
  - `Statement::VarAssign(Global)`を`VariableManager`利用へリライト（`ctx.set(.., Global)`または`ctx.var.set(..)`相当）、`Expr::VarRef(Global)`も統一。
  - `SpeechPart::VarRef`は`Talk(get_variable(name))`または`ctx.var.<name>`テンプレの正式仕様化。
  - Trade-offs: ✅既存構造活用／❌併存APIの整理作業が必要。
- Option B: 新コンポーネント（VariableExpansionAdapter）
  - 変数参照と代入を一箇所でRune出力に変換するアダプタ層を追加。
  - Trade-offs: ✅責務分離／❌ファイル増、インタフェース設計が必要。
- Option C: ハイブリッド
  - Transpiler最小修正＋Adapterで`@$変数`など動的キーだけ吸収。
  - Trade-offs: ✅段階的／❌二重の経路で複雑化の懸念。

## Effort & Risk
- Effort: M（3–7日）：既存併存APIの整理とテスト追加が必要。
- Risk: Medium：テンプレ展開と`VariableManager`の統合で設計判断が絡む。

## Recommendations for Design Phase
- **✅ Decided**: Option A（Transpiler拡張）を採用。`ctx.local`/`ctx.global`を憲法レベルで確定。
- Key Decisions Remaining: `@$変数`の動的wordキー展開手順、`>$変数`の`scene_selector`動的解決契約。
- Research Needed:
  - Rune側での`${ctx.var.name}`テンプレ評価の正規手段（現在の生成文字列の扱い）。
  - `scene_selector`が`@var_name`形式をどのように解決するか（ラベルID解決フロー）。
