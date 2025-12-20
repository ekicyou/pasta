# 単語辞書テストパターン完全ガイド

## テスト構成概要

単語定義DSL機能のテストは3層で構成されています：

| テスト層 | ファイル | テスト数 | 対象 |
|---------|---------|--------|------|
| **ユニットテスト** | `src/transpiler/word_registry.rs` | 9個 | WordDefRegistry（単語定義収集） |
| **ユニットテスト** | `src/runtime/words.rs` | 13個 | WordTable（ランタイム単語検索） |
| **統合テスト** | `tests/pasta_transpiler_word_code_gen_test.rs` | 6個 | Pass 1→Pass 2 コード生成 |
| **E2Eテスト** | `tests/pasta_word_definition_e2e_test.rs` | 6個 | パース→トランスパイル→実行 |
| **分離テスト** | `tests/pasta_stdlib_call_jump_separation_test.rs` | 5個 | Call/Jump と単語辞書の分離 |
| **サンプルスクリプト** | `examples/scripts/dic/` | 3個 | 実用例・デモンストレーション |

**合計**: 42個テスト + 3個サンプル

---

## Layer 1: トランスパイラユニットテスト（WordDefRegistry）

### ファイル: `src/transpiler/word_registry.rs`

#### テスト 1.1: `test_new()`
**目的**: 新規レジストリ作成  
**入力**: なし  
**期待される動作**:
```rust
let registry = WordDefRegistry::new();
// 空のレジストリが作成される
```
**検証項目**: `entries.is_empty() == true`

#### テスト 1.2: `test_register_global()`
**目的**: グローバル単語登録  
**入力パターン**:
```rust
registry.register_global("挨拶", vec!["こんにちは", "おはよう"])
```
**期待される結果**:
- エントリID: 0
- キー形式: `"挨拶"` （":" プレフィックスなし）
- 値: `["こんにちは", "おはよう"]`

#### テスト 1.3: `test_register_local()`
**目的**: ローカル単語登録  
**入力パターン**:
```rust
registry.register_local("会話_1", "挨拶", vec!["やあ", "おす"])
```
**期待される結果**:
- エントリID: 0
- キー形式: `":会話_1:挨拶"` （コロン接頭辞、モジュール名含む）
- 値: `["やあ", "おす"]`

#### テスト 1.4: `test_multiple_entries_same_key()`
**目的**: 同じキーで複数回登録（早期マージなし）  
**入力パターン**:
```rust
registry.register_global("場所", vec!["東京"]);
registry.register_global("場所", vec!["大阪", "京都"]);
```
**期待される結果**:
- エントリ数: 2（マージされない）
- Entry 0: key="場所", values=["東京"]
- Entry 1: key="場所", values=["大阪", "京都"]

#### テスト 1.5: `test_duplicate_global_names()`
**目的**: グローバル重複名の個別保持  
**入力**:
```rust
registry.register_global("挨拶", vec!["こんにちは"]);
registry.register_global("挨拶", vec!["おはよう"]);
```
**期待される結果**: 2個のエントリが個別保持

#### テスト 1.6: `test_duplicate_local_names()`
**目的**: ローカル重複名の個別保持  
**入力**:
```rust
registry.register_local("会話", "単語", vec!["a"]);
registry.register_local("会話", "単語", vec!["b"]);
```
**期待される結果**: 2個のエントリが個別保持（同じ `:会話:単語` キー）

#### テスト 1.7: `test_into_entries()`
**目的**: エントリ移譲機能（WordTableへの引き渡し）  
**入力**:
```rust
registry.register_global("test", vec!["a", "b"]);
let entries = registry.into_entries();
```
**期待される結果**:
- `entries.len() == 1`
- `entries[0].key == "test"`
- WordDefRegistry の所有権が失われる

#### テスト 1.8: `test_sanitize_name_ascii()`
**目的**: ASCII名前のサニタイズ  
**入力パターン**:
```
"hello" → "hello"
"hello-world" → "hello_world"
"＊会話" → "_会話"
```

#### テスト 1.9: `test_sanitize_name_unicode()`
**目的**: Unicode（日本語）名前のサニタイズ  
**入力パターン**:
```
"会話" → "会話"
"会話_朝" → "会話_朝"
"＊会話" → "_会話"
```

---

## Layer 2: ランタイムユニットテスト（WordTable）

### ファイル: `src/runtime/words.rs`

#### テスト 2.1: `test_from_word_def_registry()`
**目的**: WordDefRegistry → WordTable の変換  
**入力レジストリ**:
```rust
registry.register_global("挨拶", vec!["こんにちは", "おはよう"]);
registry.register_global("場所", vec!["東京"]);
registry.register_local("会話_1", "挨拶", vec!["やあ"]);
```
**期待される結果**:
- テーブルエントリ数: 3
- RadixMap インデックス構築完了
- 各キーにマッチするエントリIDリスト作成完了

#### テスト 2.2: `test_search_word_global_exact()`
**目的**: グローバル単語の完全一致検索  
**入力**:
```rust
table.search_word("", "挨拶", &[])
```
**期待される結果**:
```
Ok("こんにちは")  // MockSelector の配列[0] を使用
```

#### テスト 2.3: `test_search_word_global_prefix()`
**目的**: グローバル単語の前方一致検索  
**入力レジストリ**:
```rust
registry.register_global("場所", vec!["東京"]);
registry.register_global("場所_日本", vec!["大阪", "京都"]);
```
**検索**: `table.search_word("", "場所", &[])`  
**期待される結果**:
```
Ok(word) where word ∈ ["東京", "大阪", "京都"]
// "場所" と "場所_日本" がマッチ → 統合
```

#### テスト 2.4: `test_search_word_local()`
**目的**: ローカル＋グローバル統合検索  
**入力レジストリ**:
```rust
registry.register_global("挨拶", vec!["こんにちは", "おはよう"]);
registry.register_local("会話_1", "挨拶", vec!["やあ"]);
```
**検索**: `table.search_word("会話_1", "挨拶", &[])`  
**期待される結果**:
```
Ok(word) where word ∈ ["やあ", "こんにちは", "おはよう"]
// ローカル：":会話_1:挨拶" マッチ → [やあ]
// グローバル："挨拶" マッチ → [こんにちは, おはよう]
// 統合 → [やあ, こんにちは, おはよう]
```

#### テスト 2.5: `test_search_word_not_found()`
**目的**: 未定義単語のエラーハンドリング  
**入力**: `table.search_word("", "存在しない", &[])`  
**期待される結果**:
```rust
Err(PastaError::WordNotFound { key: "存在しない" })
// panic しない、Err を返却
```

#### テスト 2.6: `test_search_word_cache_sequential()`
**目的**: シャッフルキャッシュの順序保証（Pop方式）  
**入力レジストリ**:
```rust
registry.register_global("test", vec!["a", "b", "c"]);
```
**実行**:
```rust
r1 = table.search_word("", "test", &[]).unwrap();
r2 = table.search_word("", "test", &[]).unwrap();
r3 = table.search_word("", "test", &[]).unwrap();
```
**期待される結果**:
```
r1 = "a"
r2 = "b"
r3 = "c"
// キャッシュから順にPop（シャッフルなし、MockSelector使用）
```

#### テスト 2.7: `test_search_word_cache_reshuffle()`
**目的**: キャッシュ枯渇時の再シャッフル  
**入力レジストリ**:
```rust
registry.register_global("test", vec!["a", "b"]);
```
**実行**:
```rust
r1 = table.search_word("", "test", &[]).unwrap();  // "a"
r2 = table.search_word("", "test", &[]).unwrap();  // "b"
r3 = table.search_word("", "test", &[]).unwrap();  // キャッシュ枯渇 → 再シャッフル
```
**期待される結果**:
```
r1 = "a"
r2 = "b"
r3 = "a"  // 再シャッフルして最初から
```

#### テスト 2.8: `test_cache_key_separation()`
**目的**: モジュール別キャッシュ分離  
**入力レジストリ**:
```rust
registry.register_global("word", vec!["global"]);
registry.register_local("mod1", "word", vec!["local1"]);
registry.register_local("mod2", "word", vec!["local2"]);
```
**実行**:
```rust
r1 = table.search_word("mod1", "word", &[]).unwrap();
r2 = table.search_word("mod2", "word", &[]).unwrap();
```
**期待される結果**:
```
r1 = "local1" or "global"  // ("mod1", "word") キャッシュ
r2 = "local2" or "global"  // ("mod2", "word") キャッシュ（独立）
```

#### テスト 2.9: `test_local_does_not_match_global_prefix()`
**目的**: ローカルエントリがグローバルプレフィックス検索に影響しないこと  
**入力レジストリ**:
```rust
registry.register_global("abc", vec!["global_abc"]);
registry.register_local("mod", "abc", vec!["local_abc"]);
```
**検索**: `table.search_word("", "abc", &[])`（モジュール名なし = グローバルスコープ）  
**期待される結果**:
```
Ok("global_abc")
// ローカル ":mod:abc" はマッチしない（グローバルスコープなので）
```
#### テスト 2.10: `test_search_word_merge_duplicate_entries()` ⭐ **NEW**
**目的**: 同じキーで複数登録したエントリをランタイムで統合・マージ  
**入力レジストリ（複数登録）**:
```rust
registry.register_global("挨拶", vec!["おはよう".to_string(), "こんにちわ".to_string()]);
registry.register_global("挨拶", vec!["はろー".to_string(), "ぐっもーにん".to_string()]);
// 同じキー "挨拶" で2回登録 → 2つのWordEntryが作成される
```
**検索**: `table.search_word("", "挨拶", &[])`  
**期待される結果**:
```
Ok(word) where word ∈ ["おはよう", "こんにちわ", "はろー", "ぐっもーにん"]
// 4個の単語候補をすべてマージして統合
```
**検証項目**:
- 複数エントリが正しく統合されること
- すべての単語が候補に含まれること

#### テスト 2.11: `test_search_word_merge_duplicate_entries_all_words_reachable()` ⭐ **NEW**
**目的**: 複数登録エントリのすべての単語が順次取得可能であること  
**入力レジストリ**:
```rust
registry.register_global("挨拶", vec!["おはよう", "こんにちわ"]);
registry.register_global("挨拶", vec!["はろー", "ぐっもーにん"]);
```
**実行（4回連続検索）**:
```rust
r1 = table.search_word("", "挨拶", &[]).unwrap();  // "おはよう"
r2 = table.search_word("", "挨拶", &[]).unwrap();  // "こんにちわ"
r3 = table.search_word("", "挨拶", &[]).unwrap();  // "はろー"
r4 = table.search_word("", "挨拶", &[]).unwrap();  // "ぐっもーにん"
```
**期待される結果**:
```
results = ["おはよう", "こんにちわ", "はろー", "ぐっもーにん"]
// すべての4個単語が1回ずつ返却される（シャッフルキャッシュで順序管理）
```

#### テスト 2.12: `test_search_word_merge_duplicate_local_entries()` ⭐ **NEW**
**目的**: ローカル単語の複数登録エントリマージ  
**入力レジストリ**:
```rust
registry.register_local("会話", "挨拶", vec!["やあ", "よう"]);
registry.register_local("会話", "挨拶", vec!["おす", "ういっす"]);
```
**検索**: `table.search_word("会話", "挨拶", &[])`  
**期待される結果**:
```
Ok(word) where word ∈ ["やあ", "よう", "おす", "ういっす"]
// ローカル複数エントリも同様に統合
```

#### テスト 2.13: `test_search_word_merge_duplicate_with_global()` ⭐ **NEW**
**目的**: 複数登録エントリ（2個+3個）をすべてマージ  
**入力レジストリ**:
```rust
registry.register_global("word", vec!["a", "b"]);
registry.register_global("word", vec!["c", "d", "e"]);
// 合計5個の単語候補
```
**実行（5回連続検索）**:
```rust
r1, r2, r3, r4, r5 = 各検索実行
```
**期待される結果**:
```
results ⊇ ["a", "b", "c", "d", "e"]
// 5個すべてが取得可能
```
---

## Layer 3: トランスパイラ統合テスト（Pass 1 → Pass 2）

### ファイル: `tests/pasta_transpiler_word_code_gen_test.rs`

#### テスト 3.1: `test_word_reference_code_generation()`
**Pastaスクリプト入力**:
```pasta
＊ラベル
　キャラ：＠場所
```
**期待される生成Runeコード**:
```rust
yield Talk(pasta_stdlib::word("", "場所", []))
```
**検証項目**: 
- グローバルスコープなので第1引数が `""` （空文字列）
- 第2引数が `"場所"` （検索キー）
- 第3引数が `[]` （フィルタなし）

#### テスト 3.2: `test_global_word_definition_collection()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京　大阪　名古屋
＊メイン
　キャラ：＠場所
```
**Pass 1 検証**:
- WordDefRegistry に1エントリが収集される
- キー: `"場所"`
- 値: `["東京", "大阪", "名古屋"]`

**Pass 2 検証**:
- 単語参照がコード生成される

#### テスト 3.3: `test_transpile_with_registry_returns_word_registry()`
**Pastaスクリプト入力**:
```pasta
＠グローバル場所：東京　大阪
＊メイン
　キャラ：＠グローバル場所
```
**期待される結果**:
```rust
(generated_code, label_registry, word_registry)

// word_registry.into_entries() → 
// [WordEntry { key: "グローバル場所", values: ["東京", "大阪"], .. }]

// generated_code contains:
// pasta_stdlib::word("", "グローバル場所", [])
```

#### テスト 3.4: `test_multiple_global_word_definitions()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京　大阪
＠時間：朝　昼　夜
＊メイン
　キャラ：行くよ
```
**Pass 1 検証**:
- 2エントリが収集される
- Entry 1: key="場所", values=["東京", "大阪"]
- Entry 2: key="時間", values=["朝", "昼", "夜"]

#### テスト 3.5: `test_word_generates_talk_yield()`
**Pastaスクリプト入力**:
```pasta
＊テスト
　太郎：＠挨拶
```
**期待される生成コード**:
```rust
yield Talk(pasta_stdlib::word(...))
```
**検証項目**: 
- `yield Talk()` ラッピング存在
- `pasta_stdlib::word()` 呼び出し存在

#### テスト 3.6: `test_word_reference_standalone()`
**Pastaスクリプト入力**:
```pasta
＊テスト
　キャラ：＠名前
```
**期待される生成コード**:
```rust
pasta_stdlib::word("", "名前", [])
```

---

## Layer 4: E2E統合テスト（パース → 実行）

### ファイル: `tests/pasta_word_definition_e2e_test.rs`

#### テスト 4.1: `test_global_word_definition_e2e()`
**Pastaスクリプト入力**:
```pasta
＠挨拶：こんにちは　おはよう　こんばんは

＊会話
　さくら：＠挨拶
```
**実行フロー**:
1. Parser: Pastaをパース → AST + global_words
2. Transpiler: Pass 1 で WordDefRegistry に登録
3. Transpiler: Pass 2 で Rune コード生成
4. Engine: Rune VM 実行
5. Runtime: WordTable::search_word() で単語選択

**期待される結果**:
```
ScriptEvent::Talk {
    content: vec![ContentPart::Text(word)]
}

where word ∈ ["こんにちは", "おはよう", "こんばんは"]
```

#### テスト 4.2: `test_local_word_definition_e2e()`
**Pastaスクリプト入力**:
```pasta
＠挨拶：おはよう　早起き

＊会話_朝
　さくら：＠挨拶
```
**期待される結果**:
```
word ∈ ["おはよう", "早起き"]
```

#### テスト 4.3: `test_global_local_word_merge_e2e()`
**Pastaスクリプト入力**:
```pasta
＠挨拶：グローバル挨拶

＊会話
　さくら：＠挨拶
```
**期待される結果**:
```
word = "グローバル挨拶"
```

#### テスト 4.4: `test_prefix_match_word_search_e2e()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京
＠場所_日本：大阪　京都

＊会話
　キャラ：行き先は＠場所
```
**実行時検索**: `search_word("", "場所", [])`  
**前方一致マッチ**: 
- "場所" キーにマッチ → ["東京"]
- "場所_日本" キーにマッチ → ["大阪", "京都"]
- 統合 → ["東京", "大阪", "京都"]

**期待される結果**:
```
word ∈ ["東京", "大阪", "京都"]
```

#### テスト 4.5: `test_not_found_word_returns_empty_e2e()`
**Pastaスクリプト入力**:
```pasta
＊会話
　キャラ：＠存在しない単語
```
**期待される結果**:
```
ScriptEvent::Talk {
    content: vec![ContentPart::Text("")]  // 空文字列
}
// エラーログは出力されるが、panic しない
```

#### テスト 4.6: `test_word_called_multiple_times_e2e()`
**Pastaスクリプト入力**:
```pasta
＠挨拶：a　b　c

＊会話
　キャラ：1回目：＠挨拶
　キャラ：2回目：＠挨拶
　キャラ：3回目：＠挨拶
```
**実行時動作**: 各呼び出しがシャッフルキャッシュから異なる単語を取得

**期待される結果**:
```
イベント1: word = a
イベント2: word = b
イベント3: word = c
```

---

## Layer 5: Call/Jump 分離テスト

### ファイル: `tests/pasta_stdlib_call_jump_separation_test.rs`

#### テスト 5.1: `test_call_does_not_access_word_table()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京　大阪

＊会話
　キャラ：行動実行
　＞場所
```
**期待される結果**:
- Call文 `＞場所` は LabelTable を検索
- WordTable（単語辞書）は検索**しない**
- "場所" という名前の単語定義が存在しても、ラベルとして扱う

#### テスト 5.2: `test_jump_does_not_access_word_table()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京　大阪

＊会話
　キャラ：ジャンプ
　－場所
```
**期待される結果**:
- Jump文 `－場所` は LabelTable を検索
- WordTable は検索**しない**

#### テスト 5.3: `test_word_not_found_in_call()`
**Pastaスクリプト入力**:
```pasta
＠場所：東京　大阪

＊会話
　キャラ：呼び出し
　＞存在しないラベル
```
**期待される結果**:
- `存在しないラベル` が LabelTable で未検出
- WordTable で検索**しない**
- "ラベル未定義" エラー発生

#### テスト 5.4: `test_word_call_vs_label_priority()`
**パターン**: Call文で同名の単語とラベルが存在

**期待される結果**: ラベル優先（単語辞書は検索対象外）

#### テスト 5.5: `test_word_jump_vs_label_priority()`
**パターン**: Jump文で同名の単語とラベルが存在

**期待される結果**: ラベル優先（単語辞書は検索対象外）

---

## Layer 6: サンプルスクリプト（実用例）

### ファイル: `examples/scripts/dic/`

#### サンプル 6.1: `07_word_definition_basic.pasta`
**パターン**: グローバル単語定義の基本

**入力パターン**:
```pasta
# グローバル単語定義
＠挨拶：こんにちは　おはよう　こんばんは　やあ
＠天気：晴れ　曇り　雨　雪
＠感情：嬉しい　楽しい　ワクワク

# ラベル定義
＊会話_挨拶
    さくら：＠挨拶！今日はいい＠天気だね。
    うにゅう：＠挨拶。僕は今日＠感情気分だよ。
```

**実行例**:
```
ラベル「会話_挨拶」実行 → 
イベント1: Talk("こんにちは！今日はいい晴れだね。")  
イベント2: Talk("おはよう。僕は今日嬉しい気分だよ。")
```

**実行例（異なる乱数）**:
```
ラベル「会話_挨拶」実行 → 
イベント1: Talk("やあ！今日はいい雪だね。")
イベント2: Talk("こんばんは。僕は今日ワクワク気分だよ。")
```

---

#### サンプル 6.2: `08_word_definition_prefix_search.pasta`
**パターン**: 前方一致検索・複合マッチング

**入力パターン**:
```pasta
# 階層化された単語定義
＠挨拶：こんにちは　ハロー
＠挨拶_朝：おはよう　グッドモーニング　おはようございます
＠挨拶_昼：こんにちは　お疲れ様です
＠挨拶_夜：こんばんは　Good evening　お疲れ様でした

＠場所：東京
＠場所_関東：東京　横浜　千葉
＠場所_関西：大阪　京都　神戸

＊朝の挨拶
    さくら：＠挨拶_朝！朝だね。

＊全時間帯の挨拶
    さくら：＠挨拶！どの挨拶が出るかな？
    # 前方一致: "挨拶" が以下にマッチ
    # - "挨拶" → [こんにちは, ハロー]
    # - "挨拶_朝" → [おはよう, グッドモーニング, おはようございます]
    # - "挨拶_昼" → [こんにちは, お疲れ様です]
    # - "挨拶_夜" → [こんばんは, Good evening, お疲れ様でした]
    # 統合 → 9個の単語から選択
```

**実行例**:
```
ラベル「朝の挨拶」実行 →
イベント: Talk("おはよう！朝だね。")
// "挨拶_朝" のみマッチ → 3個の候補

ラベル「全時間帯の挨拶」実行 →
イベント: Talk("お疲れ様です！どの挨拶が出るかな？")
// "挨拶" プレフィックスマッチ → 9個の候補から選択
```

---

#### サンプル 6.3: `09_word_definition_scope.pasta`
**パターン**: グローバル/ローカルスコープとマージ

**入力パターン**:
```pasta
# グローバル単語定義
＠挨拶：グローバル挨拶1　グローバル挨拶2
＠感想：すごいね　びっくりだね

# グローバルラベル
＊グローバルのみ使用
    さくら：＠挨拶！＠感想
    # グローバルのみ参照

＊ローカル定義あり
    # ローカル単語定義
    ＠挨拶：ローカル挨拶1　ローカル挨拶2
    ＠食べ物：りんご　みかん　バナナ

    # 「挨拶」参照 → グローバル+ローカル統合
    さくら：＠挨拶！
    # 候補: グローバル挨拶1, グローバル挨拶2, ローカル挨拶1, ローカル挨拶2

    # 「食べ物」はローカルのみ
    さくら：好きな果物は＠食べ物だよ。

    # 「感想」はグローバルのみ
    うにゅう：＠感想

＊別のラベル
    # このラベルでは「食べ物」は見えない（ローカル定義だから）
    さくら：食べ物は：＠食べ物
    # 実行時: 空文字列 ""（未定義）

    # 「挨拶」はグローバル定義のみ
    さくら：＠挨拶

# ＠エスケープ例
＊メールアドレス
    さくら：連絡先は example＠＠test.com です
    # "＠＠" → "@" に変換
```

**実行例1（グローバルのみ使用）**:
```
ラベル「グローバルのみ使用」実行 →
イベント1: Talk("グローバル挨拶1！すごいね")
// グローバル定義から選択
```

**実行例2（ローカル定義あり）**:
```
ラベル「ローカル定義あり」実行 →
イベント1: Talk("ローカル挨拶2！")
// グローバル+ローカル統合（4個候補から選択）
イベント2: Talk("好きな果物はみかんだよ。")
// ローカル「食べ物」から選択
イベント3: Talk("すごいね")
// グローバル「感想」から選択

ラベル「別のラベル」実行 →
イベント1: Talk("食べ物は：")
// 「食べ物」はローカルなので未定義 → 空文字列

イベント2: Talk("グローバル挨拶1")
// グローバル定義のみ
```

**実行例3（エスケープ）**:
```
ラベル「メールアドレス」実行 →
イベント: Talk("連絡先は example@test.com です")
// "＠＠" がシングル "@" に変換される
```

---

## テストパターン総括表

| No. | レイヤー | テスト名 | 入力パターン | 期待される結果 | キー検証項目 |
|-----|---------|---------|-----------|------------|-----------|
| 1.1 | Unit (Registry) | test_new | - | 空レジストリ | 初期化 |
| 1.2 | Unit (Registry) | test_register_global | `("挨拶", ["a","b"])` | key="挨拶" | グローバル登録 |
| 1.3 | Unit (Registry) | test_register_local | `("mod", "k", ["x"])` | key=":mod:k" | ローカル登録 |
| 1.4 | Unit (Registry) | test_multiple_same_key | 2回登録 | 2エントリ（非マージ） | 早期マージなし |
| 2.1 | Unit (Table) | test_from_registry | Registry→Table | 3エントリ+インデックス | 変換 |
| 2.2 | Unit (Table) | test_search_global_exact | `("", "k", [])` | 値から選択 | 完全一致 |
| 2.3 | Unit (Table) | test_search_global_prefix | `("", "場", [])` | 統合候補から選択 | 前方一致 |
| 2.4 | Unit (Table) | test_search_local | `("mod", "k", [])` | ローカル+グローバル統合 | 2段階検索 |
| 2.5 | Unit (Table) | test_not_found | `("", "none", [])` | Err | エラーハンドリング |
| 2.6 | Unit (Table) | test_cache_sequential | 3回呼び出し | a,b,c順 | Pop方式 |
| 2.7 | Unit (Table) | test_cache_reshuffle | 4回呼び出し | a,b,a,... | 再シャッフル |
| 3.1 | Transpiler | test_word_ref_codegen | `＠場所` | `word("", "場所", [])` | コード生成 |
| 3.2 | Transpiler | test_global_collection | `＠k：v1 v2` | key="k", vals=["v1","v2"] | Pass 1収集 |
| 4.1 | E2E | test_global_e2e | `＠k：v1 v2 / ＠k` | Talk(v1 or v2) | 全フロー |
| 4.4 | E2E | test_prefix_e2e | `＠k：v1 / ＠k_sub：v2 / ＠k` | Talk(v1 or v2) | 前方一致マージ |
| 5.1 | Separation | test_call_no_word | `＠k + ＞k` | ラベルのみ検索 | Call分離 |
| 6.1 | Sample | basic | グローバル定義3個 | 実行可能 | 基本デモ |
| 6.2 | Sample | prefix | 階層定義4個 | 前方一致動作 | 複合検索デモ |
| 6.3 | Sample | scope | グローバル+ローカル | スコープマージ | 統合マージデモ |

---

## まとめ

### テストカバレッジ範囲
- ✅ **ユニット層**: WordDefRegistry（9個）+ WordTable（9個）= 18個
- ✅ **統合層**: Pass 1→Pass 2（6個）+ E2E（6個）= 12個
- ✅ **分離層**: Call/Jump非アクセス（5個）
- ✅ **実用層**: サンプル（3個）

**合計**: 38個テスト + 3個サンプル = 高度なカバレッジ

### テストの特性
| 特性 | 実装状況 |
|-----|---------|
| **エッジケース** | ✅ 未定義、再シャッフル、複数マッチ、スコープ分離 |
| **エラーハンドリング** | ✅ graceful degradation（panic なし） |
| **シャッフル戦略** | ✅ Pop方式・再シャッフル両対応 |
| **スコープ管理** | ✅ グローバル/ローカル統合、分離テスト |
| **前方一致** | ✅ 複数キー同時マッチ、統合マージ |
| **実用例** | ✅ 3つのシナリオで実装確認 |

