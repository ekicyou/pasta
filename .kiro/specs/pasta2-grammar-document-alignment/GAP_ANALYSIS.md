# Implementation Gap Analysis

## Overview

本ギャップ分析は、pasta2.pest文法定義と既存ドキュメント（SPECIFICATION.md、GRAMMAR.md）およびサンプルスクリプト（comprehensive_control_flow2.pasta）の間の不整合を識別するものです。

**プロジェクトの特性**：この仕様はドキュメント・サンプルスクリプトの内容更新であり、Rustプログラムコード変更を伴わない。従って、ギャップ分析は「既存ドキュメント内容」と「pasta2.pest文法定義」との乖離を明確化することが目的である。

---

## 1. 現状の調査

### 1.1 既存ドキュメント構成

| ファイル | 種別 | 行数 | 役割 |
|---------|------|------|------|
| **SPECIFICATION.md** | 仕様書 | 1,210 | 権威的文法仕様（パーサー実装用） |
| **GRAMMAR.md** | リファレンス | 625 | 学習者向け文法ガイド |
| **comprehensive_control_flow2.pasta** | サンプル | ~70 | 要件7の実装例とテストフィクスチャ |

### 1.2 pasta2.pest定義の特性

- **権威的基準**: Pest形式で厳密に定義された文法（変更不可）
- **マーカー**: 全角/半角両対応（例：`＊` vs `*`、`＠` vs `@`）
- **新規要素**: 式（expr, term, bin, bin_op）サポート、グローバル参照（`$*id`, `@*id`, `>*id`）
- **廃止要素**: Jump文（？マーカー）、演算子制約の廃止
- **さくらスクリプト**: 半角バックスラッシュ（\）のみ、全角不可

### 1.3 既存ドキュメントの現在状態

#### SPECIFICATION.md
✅ **整合**：
- マーカー定義は pasta2.pest と合致
- さくらスクリプト仕様（半角\のみ）が正確に記載
- 演算子は「将来予約」として明記
- 全角・半角混在正規化の方針あり

⚠️ **不整合・要確認**：
- 式（expr）の定義が pasta2.pest の expr, term, bin, bin_op 規則と完全には対応していない
- グローバル参照（`$*id`, `@*id`, `>*id`）の詳細説明が不足
- 属性値の型定義（attr_value）との対応が曖昧

#### GRAMMAR.md
✅ **整合**：
- マーカー一覧表が pasta2.pest の対応を正確に示す
- Jump文廃止の警告あり
- さくらスクリプトの半角\要件が記載

⚠️ **不整合・要確認**：
- 式（Expression）セクション（3.5）が pasta2.pest の expr, term, bin 規則と完全に対応していない
- グローバル参照の説明が不足（例：`@*word_ref_global`, `$*var_ref_global`, `>*call_scene_global`）
- 「式の制約」は説明されているが、実際には式がサポートされている（pasta2.pestで expr ルール定義あり）

#### comprehensive_control_flow2.pasta
⚠️ **構文エラーの可能性**：
- 行4: `＄カウンタ＝１０` → pasta2.pestでは `＝` は `set_marker` だが、実際には `:` (kv_marker) を使う必要があるか、あるいは `＝` を認めるか要確認
  - pasta2.pest の `set` ルール: `id ~ s ~ set_marker ~ s ~ expr` （`set_marker = "＝" | "="`）
  - 修正: `＄カウンタ＝１０` は実は正しい（`＝`は set_marker）
- 行25-27: `＞カウント表示（＄カウンタ）` → Call文に括弧を使用
  - pasta2.pest の `call_scene` ルール: `call_marker ~ id ~ s ~ args?`
  - args は `lparen ~ s ~ (arg ~ (comma_sep ~ arg)*)? ~ s ~ rparen`
  - 修正: 括弧の扱いが不明確（全角括弧（）vs半角括弧()）
- 行41: `＠＊天気` → グローバル単語参照
  - pasta2.pest で `word_ref_global = { word_marker ~ global_marker ~ id ~ s }` として定義
  - 現在のサンプルでは実装例として記載されているが、その他のグローバル参照例がない
- 末尾のRuneブロック：```rune ``` で囲まれているが、内容が空（`\n`のみ）
  - パーサーは通過するが、トランスパイラーでエラーになる可能性

---

## 2. 要件と既存ドキュメントのマッピング

### Requirement 1: pasta2.pest文法の権威性確立
**AC 1-5 分析**:
- AC 1-4: 既存ドキュメント内のマーカー、識別子、式定義はpasta2.pestと基本的に整合しているが、**グローバル参照機能（$*, @*, >*）の説明が不足**
- **ギャップ**: グローバルスコープ修飾子（`global_marker = "＊"` | `"*"`）の詳細説明が SPECIFICATION.md に不足

### Requirement 2: SPECIFICATION.mdの文法整合性
**AC 1-15 分析**:
- AC 1（マーカー）: ✅ 大体整合
- AC 2（識別子）: ✅ 整合（reserved_id, identifier定義あり）
- AC 3（式）: ⚠️ **重大な不整合**
  - SPECIFICATION.md の「式の制約」では「式を記述できない」と明記
  - しかし pasta2.pest では `expr = { term ~ s ~ bin* }` として式を定義
  - **ギャップ**: 仕様書は「式を記述できない」が、文法は式をサポート
  - **対応**: pasta2.pestの定義が新規仕様。仕様書を更新して式サポートを明記する必要
- AC 4（変数）: ⚠️ **グローバル変数説明不足**
  - `var_ref_local` vs `var_ref_global` の区別が説明されていない
- AC 5（関数呼び出し）: ⚠️ **グローバル関数呼び出し説明不足**
  - `fn_call_local` vs `fn_call_global` の対応表が不足
- AC 6（文字列）: ✅ 基本整合（「」と"両対応）
- AC 7（単語辞書）: ✅ 整合（words, word_nofenced, comma_sep対応）
- AC 8（属性）: ✅ 整合（attr, key_attr, attr_value対応）
- AC 9（シーン）: ⚠️ **グローバルシーン継続行の説明不足**
  - pasta2.pest の `global_scene_continue_line = { global_marker ~ or_comment_eol }` が説明されていない
- AC 10（アクション行）: ✅ 大体整合
- AC 11（Call文）: ⚠️ **グローバルCall説明不足**
  - `call_scene_local` vs `call_scene_global` の説明が不足
- AC 12（さくらスクリプト）: ✅ 整合（半角\のみ要件あり）
- AC 13（Rune）: ✅ 整合
- AC 14（スコープ）: ✅ 整合
- AC 15（廃止構文）: ✅ 廃止構文記載なし

### Requirement 3: GRAMMAR.mdの学習者向け文法整合性
**AC 1-10 分析**:
- AC 1（マーカー）: ✅ 整合
- AC 2-9（各セクション）: ⚠️ **式とグローバル参照の説明不足**
  - 「式の制約」セクションで「式は記述できない」と説明
  - しかし pasta2.pest は式をサポート → **更新必要**
  - グローバル参照（`@*word`, `$*var`, `>*scene`）の例がない

### Requirement 4: comprehensive_control_flow2.pastaのパース可能性
**AC 1-11 分析**:
- AC 2（グローバルシーン）: ✅ 整合
- AC 3（ローカルシーン）: ✅ 整合
- AC 4（アクション行）: ✅ 整合
- AC 5（変数代入）: ⚠️ **括弧の形式不明確**
  - 行25-27: `＞カウント表示（＄カウンタ）` - 全角括弧使用
  - pasta2.pest の `args = { lparen ~ s ~ ... ~ rparen }` で `lparen = "（" | "("` 
  - **確認必要**: 実装例と定義の矛盾解決
- AC 6（Call文）: ⚠️ 上記同様、引数括弧の形式確認必要
- AC 7（単語定義）: ✅ 整合
- AC 8（属性定義）: ✅ 整合
- AC 9（さくらスクリプト）: ✅ 整合（使用例なし）
- AC 10（Runeブロック）: ⚠️ **コンテンツが空**
  - 末尾のRuneブロック内容が空だけだと、トランスパイラーでエラー

### Requirement 5: ドキュメント間の一貫性維持
**AC 1-5 分析**:
- AC 1（マーカー）: ✅ 一貫
- AC 2（スコープ）: ⚠️ **グローバル参照の記法が不統一**
- AC 3（シーン定義）: ✅ 一貫
- AC 4（さくらスクリプト）: ✅ 一貫（半角\統一）
- AC 5（文字列リテラル）: ✅ 一貫

### Requirement 6: 廃止構文の完全削除
**AC 1-9 分析**:
- AC 1-3（SPECIFICATION.md）: ✅ Jump文記載なし
- AC 4-6（GRAMMAR.md）: ✅ Jump廃止警告あり（行262）
- AC 7-9（comprehensive_control_flow2.pasta）: ✅ 廃止構文使用なし

### Requirement 7: 新規構文の完全反映
**AC 1-8 分析**:
- AC 1（式のサポート）: ❌ **式説明不足**
  - pasta2.pest の expr, term, bin 規則が SPECIFICATION.md で詳細に説明されていない
  - SPECIFICATION.md は逆に「式は記述できない」と述べている
  - **最大の不整合**: 文法定義と仕様書が矛盾
- AC 2-5（グローバル参照）: ⚠️ **説明不足**
  - `$*id`, `@*id`, `>*id` の説明がほぼない

---

## 3. 要件-資産ギャップマップ

| 要件 | 資産 | ギャップ | 重要度 | 対応 |
|------|------|--------|--------|------|
| Req 2, AC 3（式） | SPECIFICATION.md 1.3節 | **矛盾**：文法は式をサポート、仕様書は非サポート | 🔴 高 | 式サポート説明を追加 |
| Req 2, AC 4（グローバル変数） | SPECIFICATION.md 2.3節 | 説明不足：$*idの詳細が不足 | 🟡 中 | グローバルスコープ修飾説明追加 |
| Req 2, AC 5（グローバル関数） | SPECIFICATION.md 2.3節 | 説明不足：@*id呼び出しの詳細が不足 | 🟡 中 | グローバル関数呼び出し説明追加 |
| Req 2, AC 11（グローバルCall） | SPECIFICATION.md 4節 | 説明不足：>*idの詳細が不足 | 🟡 中 | グローバルCall説明追加 |
| Req 3, AC（式） | GRAMMAR.md 3.5節 | **矛盾**：「式は記述できない」と説明 | 🔴 高 | 式サポート説明更新 |
| Req 3, AC（グローバル参照） | GRAMMAR.md全体 | 説明不足：例がない | 🟡 中 | グローバル参照例追加 |
| Req 4, AC 5-6（括弧形式） | comprehensive_control_flow2.pasta | 不明確：全角括弧使用、pasta2.pestで両対応 | 🟡 中 | 括弧形式の実装確認・統一 |
| Req 4, AC 10（Rune内容） | comprehensive_control_flow2.pasta末尾 | エラー：空のRuneブロック | 🟡 中 | Runeブロック内容追加 |
| Req 7, AC 1（式） | SPECIFICATION.md + GRAMMAR.md | 説明不足：expr, term, bin詳細説明が不足 | 🔴 高 | 式規則の詳細説明追加 |
| Req 7, AC 2-5（グローバル参照） | SPECIFICATION.md + GRAMMAR.md | 説明不足：$*, @*, >*の説明が不足 | 🟡 中 | グローバル参照の詳細説明・例追加 |

---

## 4. 実装アプローチオプション

### Option A: 最小修正アプローチ（Extend Existing）

**戦略**: 既存ドキュメント構造を維持し、**不足セクション追加**と**矛盾解決**に最小限

**具体的対応**:
1. SPECIFICATION.md 1.3節「式の制約」を改名し、「式サポート」として pasta2.pest の expr, term, bin, bin_op 規則を説明
2. SPECIFICATION.md 2.3節「変数・関数」に新規サブセクション「2.3.3 グローバルスコープ修飾子」追加（$*id, @*id説明）
3. SPECIFICATION.md 2.4節「Call」に新規サブセクション「グローバルCall」追加（>*id説明）
4. GRAMMAR.md の「式の制約」セクション（3.5）を削除し、「式サポート」セクションに置換
5. GRAMMAR.md に「グローバル参照」セクション追加（@*word, $*var, >*scene例）
6. comprehensive_control_flow2.pasta の括弧形式を確認（全角or半角統一）
7. comprehensive_control_flow2.pasta のRuneブロック内容を追加

**実装複雑度**: S（1-3日）
- セクション追加・置換は局所的
- 既存文言を活用可能

**リスク**: 低
- 既存構造維持で破壊的変更なし
- pasta2.pest定義に基づいた追加のみ

**トレードオフ**:
- ✅ 既存ドキュメント構造そのまま
- ✅ 修正が局所的で影響最小
- ❌ 1.3節の大幅改名により参照リンク更新必要

---

### Option B: ドキュメント再構成アプローチ（Create New Sections）

**戦略**: SPECIFICATION.mdを pasta2.pest に合わせて全体的に再構成。セクション順序を文法定義順に変更

**具体的対応**:
1. SPECIFICATION.mdの第2章「キーワード・マーカー定義」を pasta2.pest の構成順（space_chars → id → marker → expr → var_ref → fn_call → etc）に再構成
2. 各セクション見出しを pasta2.pest の ルール名に対応させる
3. 「式の制約」を削除し、「2.4 式（Expression）」として独立セクション化
4. GRAMMAR.md を簡潔化し、SPECIFICATION.md の詳細説明を参照する構成へ

**実装複雑度**: M（3-7日）
- セクション順序変更・統合が必要
- パスタルール順序への再編成

**リスク**: 中
- 既存参照リンクの大量更新
- セクション見出し変更による外部参照影響

**トレードオフ**:
- ✅ pasta2.pest との一対一対応で保守性向上
- ✅ 長期的には参照しやすい構成
- ❌ 既存参照・インデックスの更新コスト大

---

### Option C: ハイブリッドアプローチ（Extend + Add）

**戦略**: Option A（最小修正）を段階1として実施し、段階2で SPECIFICATION.md 一部再構成

**段階1（今回実装）**: Option A と同じ最小修正
**段階2（将来）**: Section 2 の再構成と pasta2.pest との完全対応化

**実装複雑度**: S（段階1） → M（段階2）

**リスク**: 低 → 中

**トレードオフ**:
- ✅ 即時は影響最小（Option A）
- ✅ 将来の保守性向上（段階2）
- ❌ 2段階実装のコスト

---

## 5. 推奨実装アプローチ

**推奨: Option A（最小修正アプローチ）**

**理由**:
1. 本仕様は「要件承認 → 設計 → 実装」の流れで進行中。設計フェーズで最小コストで最大効果を実現する戦略が有効
2. 既存ドキュメント参照・リンク、ツール統合（CI/CD等）への影響を最小化
3. pasta2.pest との不整合は追加セクション・説明で解決可能
4. 将来のドキュメント再構成（段階2）への基盤を構築

**段階2実施時期**:
- Requirement 7（新規構文完全反映）が完全に承認・実装された後
- ドキュメント全体の大規模レビューのタイミング

---

## 6. 研究・実装時の未解決事項

### 6.0 式（Expression）サポート ✅ RESOLVED
**決定**: pasta2.pestは式を正式採用
**対応**: SPECIFICATION.md 1.3節「式の制約」を「式（Expression）のサポート」に改名し、expr, term, bin, bin_op規則を詳細説明に変更。既存の2.7節「演算子」セクション参照を追加
**影響**: Requirement 7 AC 1-8（式説明）が自動的に満たされる

### 6.3 括弧形式の確定 (Pending)
- **現状**: comprehensive_control_flow2.pasta で全角括弧（）を使用。pasta2.pest は両対応（`lparen = "（" | "("`)
- **確認項目**:
  - Call文引数括弧は全角（）と半角()のどちらを推奨するか？
  - パーサー・トランスパイラーでの実装状況

### 6.4 Rune コンテンツ検証 (Pending)
- **現状**: comprehensive_control_flow2.pasta 末尾のRuneブロックが空
- **確認項目**:
  - 有効な関数定義例を追加すべきか、あるいは空でよいか？

---

## 7. 実装複雑度・リスク評価

| 項目 | 複雑度 | リスク | 理由 |
|------|--------|--------|------|
| SPECIFICATION.md 式説明更新 | S | 低 | セクション追加・説明追記のみ |
| SPECIFICATION.md グローバル参照説明追加 | S | 低 | 新規サブセクション追加のみ |
| GRAMMAR.md 式セクション更新 | S | 低 | 既存セクション置換・例追加のみ |
| GRAMMAR.md グローバル参照説明追加 | S | 低 | 新規セクション・例追加のみ |
| comprehensive_control_flow2.pasta 括弧確認・統一 | S | 中 | parser実装確認が必要 |
| comprehensive_control_flow2.pasta Rune内容追加 | S | 低 | 有効な関数定義例の追加 |
| **全体（Option A）** | **S** | **低** | 局所的な追加・置換のみ |

---

## 結論

既存ドキュメント（SPECIFICATION.md、GRAMMAR.md）と pasta2.pest 定義の間には、主に以下の乖離が存在：

1. **式サポートの矛盾（最重要）**: 仕様書は「式不可」、文法は「式可」 → pasta2.pest を正として、仕様書を更新
2. **グローバル参照の説明不足**: $*, @*, >* の説明が不足 → 詳細セクション追加
3. **サンプルスクリプトの小問題**: 括弧形式・Rune内容の確認 → 実装確認後に修正

**推奨アプローチ（Option A）で S 複雑度、低リスク で対応可能。**

設計フェーズで詳細な修正箇所と方法を確定し、実装フェーズで段階的に反映される。

