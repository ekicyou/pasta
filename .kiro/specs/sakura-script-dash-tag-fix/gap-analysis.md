# ギャップ分析: sakura-script-dash-tag-fix

## 1. 現状調査

### 1.1 問題の構造

さくらスクリプトの `\-` タグが、Pasta DSL パイプライン全体の **3つの定義箇所** で `-` が文字クラスに含まれておらず、認識不能となっている。

**パイプライン概要**:
```
Pasta DSL → [Pest Parser] → AST → [Code Generator] → Lua → [Tokenizer (regex)] → Wait Insertion → SakuraScript出力
```

### 1.2 影響を受ける箇所の詳細マップ

| # | レイヤー | ファイル | 現在の定義 | 問題 |
|---|---------|---------|-----------|------|
| **A1** | 仕様書 | `doc/spec/07-sakura-script.md` | `sakura_token ::= [!_a-zA-Z0-9]+` | `-` なし |
| **A2** | 仕様書 | `GRAMMAR.md` (L508) | `sakura_token ::= [!_a-zA-Z0-9]+` | `-` なし（07と同一内容の複製） |
| **B1** | パーサー | `grammar.pest` (L171) | `sakura_id = @{ ('a'..'z') \| ('A'..'Z') \| ('0'..'9') \| "_" \| "!" )+` | `-` なし |
| **C1** | ランタイム | `tokenizer.rs` (L113) | `r"\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?"` | `-` なし |

### 1.3 影響を受けない箇所（確認済み）

以下の箇所は **変更不要** であることを確認した：

| # | レイヤー | ファイル | 理由 |
|---|---------|---------|------|
| ✅ | AST | `parser/ast.rs` (L510) | `Action::SakuraScript { script: String }` — 文字列をそのまま保持。文字クラスに依存しない |
| ✅ | パーサー | `parser/mod.rs` (L870-874) | `inner.as_str().to_string()` — Pest出力をそのまま格納。文字クラスに依存しない |
| ✅ | コード生成 | `code_generator.rs` (L544-547) | `StringLiteralizer::literalize(script)` → Lua long string形式。`\-` は `\` を含むので `[=[...]=]` 形式が自動選択される。問題なし |
| ✅ | 文字列リテラル化 | `string_literalizer.rs` | `\` 含有で long string 使用。`-` に特別な処理なし |
| ✅ | Luaランタイム | `scripts/pasta/act.lua` (L188-190) | `sakura_script` トークンをテーブルに蓄積するだけ（パススルー） |
| ✅ | Luaビルダー | `scripts/pasta/shiori/sakura_builder.lua` (L104) | `sakura_script` タイプのトークンをバッファに追加するだけ（パススルー） |
| ✅ | ウェイト挿入 | `wait_inserter.rs` (L45) | `TokenKind::SakuraScript => None` — SakuraScriptトークンにはウェイトを挿入しない。`\-` が正しくSakuraScriptとしてトークナイズされれば自動的に透過される |
| ✅ | Rustランタイム | `runtime/mod.rs` (L717-727) | sakura_scriptモジュール登録のみ。文字クラスに依存しない |
| ✅ | 仕様書 | `doc/spec/02-markers.md` (L189-195) | エスケープ文字 `\` の定義。sakura_tokenとは独立 |

### 1.4 文法衝突リスク分析

**`local_marker` との衝突**: ❌ なし

`local_marker` は `_{ "・" | "-" }` と定義されているが、行レベルルール `local_scene_line = { pad ~ local_marker ~ scene }` でのみ使用される。`sakura_id` はアクション行内で `sakura_marker`（`\`）の直後でのみ評価されるため、構文レベルが完全に分離している。

**`sakura_escape` との衝突**: ❌ なし

`action` ルールの ordered choice で `sakura_escape`（`\\`）は `sakura_script` よりも前に試行される。`\\` は常にエスケープとして処理され、`\-` は `sakura_script` で処理される。

**現在 `\-` に遭遇した場合のパース挙動（ブラックホール問題）**:

```
入力: Alice：こんにちは\-。
1. `こんにちは` → talk にマッチ ✅
2. `\` に到達 → talk_word が sakura_marker を除外 → talk 終了
3. action の ordered choice を順に試行:
   - at_escape (`@@`) → ❌
   - dollar_escape (`$$`) → ❌
   - sakura_escape (`\\`) → 2文字目が `-` ≠ `\` → ❌
   - fn_call, word_ref, var_ref → ❌
   - sakura_script → `\` ✅ → sakura_id に `-` なし → ❌
   - talk → talk_word が `\` を除外 → ❌
4. 結果: パースエラー 🔥
```

`\` は `talk` からは除外されるが、`sakura_script` にもマッチしない「ブラックホール」状態。

## 2. 要件-資産マップ

| 要件 | 資産 | ステータス | 備考 |
|------|------|-----------|------|
| Req 1: 仕様書更新 | `doc/spec/07-sakura-script.md` | **Missing** | 文字クラスに `-` 追加必要 |
| Req 1: 仕様書更新 | `GRAMMAR.md` | **Missing** | 上記と同期更新必要（要件書で漏れていた箇所） |
| Req 2: Pest文法修正 | `grammar.pest` L171 | **Missing** | `sakura_id` に `-` 追加必要 |
| Req 3: regex修正 | `tokenizer.rs` L113 | **Missing** | `SAKURA_TAG_PATTERN` に `-` 追加必要 |
| Req 4: 一貫性保証・拡張性 | テストコード・ドキュメント | **Missing** | `\-` テストケースが存在しない・変更箇所リストのドキュメント化 |

### 要件書の追加発見事項

**GRAMMAR.md が要件書で漏れている**: 要件書では修正箇所を3箇所（仕様書・Pest・regex）としているが、`GRAMMAR.md`（人間向けリファレンス）にも同一の `sakura_token` 定義が存在する（L508）。合計 **4箇所** の修正が必要。

## 3. 実装アプローチ

### Option A: 既存コンポーネントの拡張（推奨）

本件は既存の文字クラス定義の単純な拡張であり、新コンポーネントの作成は不要。

**修正ファイル一覧（変更のみ）**:

| # | ファイル | 変更内容 |
|---|---------|---------|
| 1 | `doc/spec/07-sakura-script.md` | `[!_a-zA-Z0-9]+` → `[!\-_a-zA-Z0-9]+` |
| 2 | `GRAMMAR.md` | 同上 |
| 3 | `grammar.pest` L171 | `"_" \| "!"` → `"_" \| "!" \| "-"` |
| 4 | `tokenizer.rs` L113 | `[0-9a-zA-Z_!]+` → `[0-9a-zA-Z_!-]+` |

**追加テストファイル（新規/追記）**:

| # | ファイル | テスト内容 |
|---|---------|----------|
| 5 | `pasta_core/tests/` | `\-` のパーステスト（sakura_scriptとして認識） |
| 6 | `pasta_lua/tests/` または `tokenizer.rs` 内 | `\-` のトークナイズテスト（SakuraScriptとして分類） |

**トレードオフ**:
- ✅ 最小変更（4ファイル+テスト追加）
- ✅ 既存パターンに完全準拠
- ✅ リグレッションリスクが極めて低い
- ✅ 文字クラスの拡張のみで、ロジック変更なし
- ❌ なし（この規模の変更にデメリットはない）

### Option B / C: 該当なし

新コンポーネント作成やハイブリッドアプローチの必要性はない。本件は純粋な文字クラス拡張であり、Option A のみが合理的。

## 4. 複雑度とリスク

### 工数: **S**（1日以内）
- 4ファイルの文字クラス変更（各1行）
- テストケース追加（2-4ケース）
- パーサー再ビルド・全テスト実行

### リスク: **Low**
- 変更は文字クラスへの1文字追加のみ
- Pest PEG のマッチ範囲が広がるだけで、既存マッチに影響なし
- `local_marker` との衝突なし（構文レベルが分離）
- `sakura_escape` (`\\`) との衝突なし（ordered choice で先にマッチ）
- regex の `[0-9a-zA-Z_!-]` — ハイフンは文字クラス末尾に配置すれば安全

### 注意点
- regex内の `-` 配置: `[0-9a-zA-Z_!-]` のように**末尾に配置**すること（範囲指定と誤解されない）
- `\--` の扱い: `sakura_id` は `+`（1文字以上）なので `--` もマッチする。これが意図通りか要確認（ただし Pasta は意味を解釈しないため問題なし）

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option A（既存拡張）** を採用。変更規模が小さく、リスクも低い。

### 設計フェーズでの決定事項
1. テストケースの具体的な内容と配置（既存テストファイルへの追記 vs 新規ファイル）
2. `GRAMMAR.md` の要件書への追加（4箇所目として明記）→ ✅ 要件書反映済み

### リサーチ不要
- 外部依存ライブラリの調査は不要
- 新しいアーキテクチャパターンの導入は不要
