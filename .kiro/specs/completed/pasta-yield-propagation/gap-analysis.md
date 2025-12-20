# ギャップ分析: pasta-yield-propagation

## エグゼクティブサマリー

- **スコープ**: Call文のyield伝搬機構は**既に実装済み**（`for-in`パターン使用）
- **主要課題**: Jump文は**廃止済み**（REQ-BC-1）、単語参照は**スタブ実装**のみ
- **推奨アプローチ**: オプションC（ハイブリッド）- 既存Call実装を保持し、単語辞書機能を拡張
- **複雑度**: M（3-7日）、既存パターン活用だが単語辞書が新規実装
- **リスク**: Medium、単語辞書APIとRadixMap統合が未実装

## 1. 現状調査

### 1.1 既存資産マップ

| コンポーネント | ファイルパス | 状態 | 備考 |
|--------------|------------|------|------|
| **Transpiler** | `src/transpiler/mod.rs` | ✅ 実装済み | Call文の`for-in`パターン生成（L361-367） |
| `pasta::call()` | `src/transpiler/mod.rs` | ✅ 実装済み | yield伝搬ラッパー生成（L202-209） |
| `label_selector()` | `src/transpiler/mod.rs` | ✅ 実装済み | ラベルID→関数ポインタ解決 |
| **Runtime** | | | |
| LabelTable | `src/runtime/labels.rs` | ✅ 実装済み | RadixMapによる前方一致検索 |
| RandomSelector | `src/runtime/random.rs` | ✅ 実装済み | 重複ラベルのシャッフル選択 |
| **Stdlib** | | | |
| `select_label_to_id()` | `src/stdlib/mod.rs` | ✅ 実装済み | ラベル解決API（L70-91） |
| `word_expansion()` | `src/stdlib/mod.rs` | ⚠️ スタブのみ | 単語名をそのまま返す（L139-154） |
| **Jump文** | N/A | ❌ 廃止 | REQ-BC-1により`？`は削除、`＞`Call使用 |

### 1.2 アーキテクチャパターン

**既存のyield伝搬パターン**（実装済み）:

```rune
// Transpiler生成コード（src/transpiler/mod.rs:367）
for a in crate::pasta::call(ctx, "ラベル名", #{}, [args]) { 
    yield a; 
}

// pasta::call()実装（src/transpiler/mod.rs:202-209）
pub fn call(ctx, label, filters, args) {
    let func = crate::__pasta_trans2__::label_selector(label, filters);
    for a in func(ctx, args) { yield a; }  // ← yield伝搬実現
}
```

**モジュール構造**（実装済み）:
- グローバルラベル → `pub mod ラベル名_連番`
- ラベルなしローカルスコープ → `pub fn __start__(ctx, args)`
- ローカルラベル → `pub fn ラベル名_連番(ctx, args)`

**ラベル解決フロー**（実装済み）:
1. Transpiler: Call文 → `crate::pasta::call(ctx, "検索キー", #{filters}, [args])`
2. Stdlib: `select_label_to_id()` → LabelTableで前方一致検索 + フィルタリング
3. Transpiler: `label_selector()` → ラベルID → 関数ポインタ
4. Runtime: 関数実行 → generator → `for-in`でyield伝搬

### 1.3 統合ポイント

**既存の統合面**:
- **Pest文法**: `src/parser/pasta.pest` - Call文（`＞`）パース済み
- **AST定義**: `src/parser/ast.rs::Statement::Call` - 構造化済み
- **テストフィクスチャ**: `tests/fixtures/comprehensive_control_flow.transpiled.rn` - Call伝搬検証済み

**未実装の統合面**:
- **単語辞書**: `add_words()`, `commit_words()`, `word()` API未実装
- **単語ストレージ**: ローカルスコープ別の単語管理未実装
- **単語参照のyield伝搬**: 現在は単語名を返すだけ（伝搬なし）

## 2. 要件実現可能性分析

### 2.1 技術的ニーズマッピング

| 要件 | 技術要素 | 現状 | ギャップ |
|------|---------|------|---------|
| **Req 1: Call文yield伝搬** | | | |
| 1.1 Call文でyield伝搬 | `for-in`ループ生成 | ✅ 実装済み | なし |
| 1.2 複数yieldイベント順次実行 | Generator resume | ✅ 実装済み | なし |
| 1.3 ネストCall伝搬 | 再帰的generator呼び出し | ✅ 実装済み | なし（Runeが自動処理） |
| 1.4 Call完了後の継続 | `for-in`完了後の次行 | ✅ 実装済み | なし |
| 1.5 エラーイベント伝搬 | `ScriptEvent::Error` yield | ✅ 実装済み | なし |
| **Req 2: Jump文yield伝搬** | | | |
| 2.1-2.4 Jump文実装 | 廃止済み（REQ-BC-1） | ❌ N/A | **要件と実装が不整合** |
| **Req 3: ctx引数伝搬** | | | |
| 3.1 ctx引数渡し | 関数シグネチャ`(ctx, args)` | ✅ 実装済み | なし |
| 3.2 アクター変更反映 | `ctx.actor = ...` | ✅ 実装済み | なし |
| 3.3 ctx状態復元 | Runeのスコープ機構 | ✅ 実装済み | なし（自動処理） |
| 3.4 全関数にctx引数 | Transpiler生成 | ✅ 実装済み | なし |
| **Req 4: Transpiler出力** | | | |
| 4.1 モジュール生成 | `pub mod ラベル名_連番` | ✅ 実装済み | なし |
| 4.2 __start__関数 | エントリーポイント | ✅ 実装済み | なし |
| 4.3 ローカルラベル関数 | `pub fn ラベル名_連番` | ✅ 実装済み | なし |
| 4.4 同名ラベルの連番 | LabelRegistry管理 | ✅ 実装済み | なし |
| 4.5 Call文yield伝搬 | `for-in` パターン | ✅ 実装済み | なし |
| 4.6 Jump文yield伝搬 | 廃止済み | ❌ N/A | **要件と実装が不整合** |
| 4.7 単語定義 | `add_words()`, `commit_words()` | ⚠️ コール生成のみ | **API未実装** |
| 4.8 単語参照 | `word()` API | ⚠️ スタブのみ | **辞書検索未実装** |
| 4.9 会話行生成 | Actor/Talk yield | ✅ 実装済み | なし |
| 4.10 Runeブロック | インライン展開 | ✅ 実装済み | なし |
| **Req 5: pasta_stdlib API** | | | |
| 5.1 `ctx.pasta.call()` | ラッパー関数 | ✅ 実装済み | なし |
| 5.2 `ctx.pasta.jump()` | 廃止済み | ❌ N/A | **要件と実装が不整合** |
| 5.3 `ctx.pasta.word()` | generator返却 | ❌ 未実装 | **Missing** |
| 5.4 `ctx.pasta.add_words()` | ローカル辞書追加 | ❌ 未実装 | **Missing** |
| 5.5 `ctx.pasta.commit_words()` | Trie確定 | ❌ 未実装 | **Missing** |
| 5.6 Rune VM登録 | `Module::function()` | ✅ 実装済み | API追加が必要 |
| **Req 6: 後方互換性** | | | |
| 6.1-6.5 テストケース | 既存テスト拡張 | ⚠️ 一部のみ | 単語辞書テスト不足 |
| **Req 7: ドキュメント** | | | |
| 7.1-7.4 ドキュメント更新 | GRAMMAR.md, README.md | ⚠️ 更新必要 | Jump廃止を明記 |

### 2.2 制約と未知領域

**既知の制約**:
- ✅ **Rune 0.14 Generator**: `for-in` パターンが正常動作することを確認済み
- ✅ **モジュール分離**: グローバルラベルごとのモジュール構造が確立
- ❌ **Jump文廃止**: REQ-BC-1により`？`は削除済み、要件文書と不整合

**Research Needed**（設計フェーズで調査）:
- **単語辞書スコープ管理**: グローバル単語とローカル単語の分離方法
  - オプション1: `ctx.pasta.words_global` / `ctx.pasta.words_local` 分離
  - オプション2: LabelTableと同様のスコープ管理機構
- **単語Trie統合**: LabelTableのRadixMapと単語検索Trieの統合設計
  - 前方一致検索の共通化可能性
  - ランダム選択ロジックの再利用
- **単語参照のgenerator化**: `word()` が直接yieldするかiteratorを返すか
  - 現在: `yield pasta_stdlib::word(ctx, "単語", [])`（即時yield）
  - 推奨: `for a in ctx.pasta.word(ctx, "単語") { yield a; }`（伝搬パターン）

## 3. 実装アプローチオプション

### オプションA: 既存コンポーネント拡張（最小変更）

**対象ファイル**:
- `src/stdlib/mod.rs`: 単語辞書API実装（`add_words`, `commit_words`, `word`）
- `src/transpiler/mod.rs`: 単語参照の出力形式変更（L448: `yield pasta_stdlib::word()` → `for-in`）

**互換性評価**:
- ✅ 既存のCall伝搬パターンをそのまま活用
- ✅ LabelTableのRadixMap実装を参考に単語Trieを追加
- ❌ Jump要件は実装不可（廃止済み）

**複雑度とメンテナンス性**:
- 認知負荷: 低（既存パターンの再利用）
- 単一責任原則: 維持（stdlib内に単語辞書機能追加）
- ファイルサイズ: 許容範囲（`mod.rs`が+100行程度）

**トレードオフ**:
- ✅ 最小ファイル追加、既存パターン活用
- ✅ Call伝搬実装を再利用
- ❌ Jump要件は削除が必要（要件文書修正）
- ❌ 単語辞書がstdlib内で肥大化リスク

### オプションB: 新規コンポーネント作成（分離設計）

**新規ファイル**:
- `src/runtime/word_dict.rs`: 単語辞書管理（WordDict構造体、Trie実装）
- `src/stdlib/word.rs`: 単語API（add_words, commit_words, word関数）

**統合ポイント**:
- Stdlib: `create_module()`で`WordDict`を渡す
- Transpiler: 単語定義/参照のコード生成は変更なし

**責任境界**:
- `WordDict`: 単語登録、Trie構築、前方一致検索、ランダム選択
- `word.rs`: Rune APIラッパー、generatorへの変換

**トレードオフ**:
- ✅ 明確な責任分離（runtime/stdlib層の分離）
- ✅ LabelTableと並列な設計（一貫性）
- ✅ テストが独立しやすい
- ❌ ファイル数増加（+2ファイル）
- ❌ インターフェース設計が必要

### オプションC: ハイブリッドアプローチ（推奨）★

**戦略**:
1. **フェーズ1（即時実装）**: オプションA - 単語辞書スタブをAPI完成（`add_words`, `commit_words`）
   - `src/stdlib/mod.rs`に最小実装追加（ハッシュマップベース）
   - 単語参照を`for-in`パターンに変更（yield伝搬実現）
   - Jump要件を削除（要件文書修正）
2. **フェーズ2（将来リファクタリング）**: オプションB - 単語辞書をランタイム層に移行
   - Trie実装とランダム選択の最適化
   - `src/runtime/word_dict.rs`に移行

**リスク軽減**:
- フェーズ1でMVP実装、テスト検証
- フェーズ2は非破壊的リファクタリング（API互換維持）

**トレードオフ**:
- ✅ 段階的実装で早期検証可能
- ✅ 既存Call実装を即座に活用
- ✅ 将来の最適化パスが明確
- ⚠️ フェーズ2への移行計画が必要（技術負債管理）

## 4. 実装複雑度とリスク評価

### 4.1 工数見積もり

**オプションA（最小変更）**: **S（1-3日）**
- 単語辞書ハッシュマップ実装: 0.5日
- 単語参照の`for-in`変更: 0.5日
- 要件文書のJump削除: 0.5日
- テスト作成・検証: 1日

**オプションC（推奨ハイブリッド）**: **M（3-7日）**
- フェーズ1実装: 2日（オプションAと同等 + 設計文書化）
- 単語Trie統合設計: 1日（Research Needed項目の調査）
- テスト拡張（ネストCall、単語辞書）: 2日
- ドキュメント更新（GRAMMAR.md、要件修正）: 1日

**オプションB（新規分離）**: **M（3-7日）**
- WordDict設計・実装: 2日
- Rune API実装: 1日
- 統合テスト: 2日
- ドキュメント: 1日

### 4.2 リスク分析

**High Risk**: なし

**Medium Risk**:
- **単語辞書スコープ管理**: グローバル/ローカル単語の分離方法が未確定
  - 緩和策: フェーズ1でハッシュマップ実装、フェーズ2で最適化
- **単語参照のgenerator化**: `word()` API形式が要件と実装で不整合
  - 緩和策: 設計フェーズで`for-in`パターン採用を明確化
- **Jump要件の不整合**: 要件文書がJump実装を求めるが、REQ-BC-1で廃止済み
  - 緩和策: 要件文書からJump関連を削除（設計フェーズで修正）

**Low Risk**:
- **Call伝搬パターン**: 既に実装済み、検証済み（`test_combined_code.rn`）
- **モジュール構造**: 確立済み、変更不要
- **ラベル解決**: LabelTableで実績あり、単語辞書も同様のパターン適用可能

## 5. 推奨事項（設計フェーズへの引継ぎ）

### 5.1 推奨アプローチ

**オプションC（ハイブリッド）を推奨**:

**理由**:
1. **既存実装の活用**: Call伝搬は完成しており、単語辞書のみ追加
2. **段階的リスク軽減**: フェーズ1で基本機能検証、フェーズ2で最適化
3. **要件との整合性**: Jump廃止を要件に反映することで整合性確保

### 5.2 重要な設計判断

**即決事項**:
- ✅ **Call伝搬実装を維持**: 既存の`for-in`パターンは変更不要
- ✅ **Jump要件を削除**: REQ-BC-1に合わせて要件文書を修正
- ✅ **単語参照を`for-in`化**: `yield pasta_stdlib::word()` → `for a in ctx.pasta.word() { yield a; }`

**設計フェーズでの判断事項**:
1. **単語辞書スコープ管理方式**:
   - Option 1: `ctx.pasta.words_global` / `ctx.pasta.words_local` 分離
   - Option 2: LabelTableと統合したスコープ管理
   - **推奨**: Option 1（シンプル、フェーズ1で実装可能）

2. **単語Trie実装タイミング**:
   - フェーズ1: HashMap（O(1)検索、前方一致は線形走査）
   - フェーズ2: RadixMap（O(k)検索、kは検索キー長）
   - **推奨**: フェーズ1はHashMap、性能問題があればフェーズ2でTrie化

3. **ランダム選択の統合**:
   - 既存の`RandomSelector`を単語辞書でも再利用
   - LabelTableのキャッシュ機構を参考に単語選択履歴管理

### 5.3 Research Needed（設計フェーズで詳細化）

1. **単語辞書API仕様**:
   - `add_words(name, [values])`: ローカルスコープに追加か、グローバルに追加か
   - `commit_words()`: スコープ終了時に自動破棄するか、明示的に管理するか
   - `word(ctx, name)`: iteratorかgeneratorか（Rune 0.14の型制約確認）

2. **テスト戦略**:
   - 既存の`comprehensive_control_flow_test.rs`拡張
   - 新規: `word_dictionary_test.rs`（単語辞書独立テスト）
   - 新規: `yield_propagation_integration_test.rs`（ネストCall + 単語参照）

3. **ドキュメント更新箇所**:
   - `GRAMMAR.md`: Jump廃止を明記、単語定義文法を追加
   - `README.md`: yield伝搬機構の説明
   - `.kiro/specs/pasta-yield-propagation/requirements.md`: Jump要件削除

## 6. 結論

**現状**: Call文のyield伝搬は**既に実装済み**。Jump文は**廃止済み**（REQ-BC-1）。単語辞書は**スタブのみ**。

**推奨**: オプションC（ハイブリッド）
- フェーズ1: 単語辞書API実装（HashMap）、要件からJump削除
- フェーズ2: Trie最適化（将来）

**複雑度**: M（3-7日）
**リスク**: Medium（単語辞書スコープ管理が未確定）

**次のステップ**: `/kiro-spec-design pasta-yield-propagation` で設計文書作成
- 単語辞書API詳細仕様
- 単語参照の`for-in`パターン統合
- Jump要件削除とREQ-BC-1整合性確保
