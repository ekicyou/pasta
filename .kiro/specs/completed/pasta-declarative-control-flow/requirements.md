# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta DSL 宣言的コントロールフロー 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-11 |
| **Parent Spec** | areka-P0-script-engine (completed) |
| **Priority** | P0 (既存実装の修正) |

---

## Introduction

本仕様書は、現在の`04_control_flow.pasta`サンプルファイルに含まれる誤った実装（命令型プログラミング構文 `if/elif/else/while`）を修正し、元仕様「areka-P0-script-engine」に基づいた正しい宣言的コントロールフロー構文を再定義する。

### 問題の本質

現在の実装には以下の問題がある：

1. **仕様外構文**: `＠if`, `＠elif`, `＠else`, `＠while` は元仕様に存在しない
2. **設計意図の逸脱**: Pasta DSLは宣言的言語であり、命令型の制御構文は本質的に含まない
3. **実装の混乱**: Runeブロック内で実装すべきロジックをDSL構文レベルで提供している

### 元仕様の正しいコントロールフロー

元仕様（`areka-P0-script-engine`およびGRAMMAR.md）で定義された宣言的コントロールフローは以下で構成される：

1. **call** (`＞`): サブルーチン呼び出し（戻り先を記憶）
2. **jump** (`？`): 無条件ジャンプ（戻らない）
3. **ラベル定義**: グローバル(`＊`)とローカル(`ー`)
4. **ランダム選択**: 同名ラベルの複数定義と前方一致選択
5. **キャッシュベース消化**: 選択肢を順に消化する仕組み
6. **発言とコンテキスト**: 会話フローの本質

### スコープ

**含まれるもの:**
- 元仕様に基づいた正しいコントロールフロー構文の要件定義
- `04_control_flow.pasta`の修正実装例
- call/jump/ラベル定義/ランダム選択の正しい使用例
- **トランスパイラー出力仕様**: Pasta DSLからRuneコードへの変換規則

**含まれないもの:**
- 命令型制御構文（`if/elif/else/while`）
- Runeブロック内の条件分岐・ループ（別途Rune機能として実装可能）
- 新しいDSL構文の追加

---

## Requirements

### Requirement 1: ラベルベースのコントロールフロー

**Objective:** スクリプト作成者として、callとjumpを使用した宣言的なコントロールフローを記述できるようにし、会話の流れを自然に表現できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者がグローバルラベルを定義する, the Pasta Engine shall `＊`記号で始まる行をグローバルラベル定義として認識する
2. When スクリプト作成者がローカルラベルを定義する, the Pasta Engine shall 親グローバルラベル内の`ー`記号で始まる行をローカルラベル定義として認識する
3. When スクリプト作成者が`＞ローカル名`構文を使用する, the Pasta Engine shall 現在のグローバルラベル内のローカルラベルを呼び出し、実行後に呼び出し元に戻る
4. When スクリプト作成者が`＞＊グローバル名`構文を使用する, the Pasta Engine shall ファイル全体スコープでグローバルラベルを呼び出し、実行後に呼び出し元に戻る
5. When スクリプト作成者が`？ローカル名`構文を使用する, the Pasta Engine shall 現在のグローバルラベル内のローカルラベルにジャンプし、呼び出し元には戻らない
6. When スクリプト作成者が`？＊グローバル名`構文を使用する, the Pasta Engine shall ファイル全体スコープでグローバルラベルにジャンプし、呼び出し元には戻らない
7. When スクリプト作成者が`＞＊グローバルーローカル`構文を使用する, the Pasta Engine shall グローバルラベル配下のローカルラベルをロングジャンプ形式で呼び出す

### Requirement 2: ランダム選択と前方一致

**Objective:** スクリプト作成者として、同名ラベルの複数定義と前方一致ランダム選択を活用し、会話バリエーションを効率的に記述できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が同一名のラベルを複数定義する, the Pasta Engine shall すべての定義を内部的に連番付きで管理する（例: `挨拶_1`, `挨拶_2`, `挨拶_3`）
2. When スクリプト作成者が前方一致するラベル名でcall/jumpを実行する, the Pasta Engine shall 前方一致するすべてのラベルからランダムに1つを選択して実行する
3. When スクリプト作成者がロングジャンプ構文（`＊グローバルーローカル`）を使用する, the Pasta Engine shall グローバルとローカルの組み合わせをフラット化し、すべての組み合わせから1つをランダム選択する
4. When Pasta Engineが同じ選択キーワードで2回目以降の呼び出しを受ける, the Pasta Engine shall キャッシュから未消化の選択肢を順に返す
5. When キャッシュ内の選択肢がすべて消化される, the Pasta Engine shall キャッシュをクリアし、次回は再構築する

### Requirement 3: 動的call/jump

**Objective:** スクリプト作成者として、変数の値をラベル名として動的に解決し、実行時に柔軟なフロー制御を実現できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が`＞＠変数名`構文を使用する, the Pasta Engine shall 変数の値をラベル名として解決し、該当ラベルを呼び出す
2. When スクリプト作成者が`？＠変数名`構文を使用する, the Pasta Engine shall 変数の値をラベル名として解決し、該当ラベルにジャンプする
3. If 変数の値が存在しないラベル名を示す, the Pasta Engine shall エラーメッセージを生成し、スクリプト実行を中断する

### Requirement 4: 宣言的な会話フロー表現

**Objective:** スクリプト作成者として、命令型構文なしで条件分岐やメニュー選択を表現し、宣言的なコントロールフローのみで会話を記述できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が条件に応じた会話分岐を実装する, the Pasta Engine shall Runeブロック内で条件評価を行い、結果に応じて変数に適切なラベル名を設定し、動的jumpで分岐を実現できる
2. When スクリプト作成者がメニュー選択機能を実装する, the Pasta Engine shall 各選択肢に対応するローカルラベルを定義し、選択結果に応じてcallまたはjumpで処理を分岐できる
3. When スクリプト作成者がループ的な処理を実装する, the Pasta Engine shall jumpで自身または別のラベルに戻ることで繰り返し実行を実現できる

### Requirement 5: トランスパイラー出力仕様

**Objective:** 開発者として、Pasta DSLからRuneコードへのトランスパイル規則を明確に定義し、宣言的コントロールフロー構文が正しくRuneのyield文とwhile-let-yieldパターンに変換されることを保証する。

#### トランスパイラー出力の基本原則

1. **モジュール構造**: グローバルラベル1つにつきRuneモジュール1つを生成（`pub mod ラベル名_番号 { ... }`）
2. **`__start__`関数**: グローバルラベルの最初のスコープ（ローカルラベル定義前の処理）は必ず`pub fn __start__(ctx)`関数として生成
3. **ローカルラベル関数**: 各ローカルラベルは親モジュール内の個別関数（`pub fn ラベル名_番号(ctx)`）として生成
4. **環境引数**: すべての関数は`ctx`（コンテキストオブジェクト、Rune Object型）を第一引数として受け取るジェネレーター関数
5. **`ctx`オブジェクト構造**:
   - `ctx.pasta`: Pastaランタイムが提供するローカル処理関数（`call`, `jump`, `word`など）
   - `ctx.actor`: 現在の発言者オブジェクト（グローバル変数として定義された発言者）
   - `ctx.actor.name`: 発言者名（例: `"さくら"`）
   - `ctx.scope`: スコープ情報オブジェクト
   - `ctx.scope.global`: 現在のグローバルスコープラベル名
   - `ctx.scope.local`: 現在のローカルスコープラベル名
   - `ctx.save`: `＄変数＝値`で設定される永続化変数のオブジェクト
   - `ctx.args`: 関数呼び出し時に渡される引数配列（`＞ラベル（引数1　引数2）`で指定）
6. **単語定義スコープ**: 
   - グローバル単語定義（`＠グローバル単語：...`）→ モジュール外で`add_words()`を直接呼び出し
   - ローカル単語定義（`＠単語：...`）→ 関数内で`ctx.pasta.add_words()` + `ctx.pasta.commit_words()`

**注**: 
- `ctx`オブジェクトのフィールド構造は上記の通り
- 詳細な型定義、`ctx.pasta`の完全なメソッドシグネチャ、内部実装メカニズムは設計フェーズで定義する
- **重要**: `ctx.pasta.call()`は呼び出し前に`ctx.args`を保存し、呼び出し完了後に復元することで、`ctx`のミュータビリティによる副作用を最小化する（設計原則）

#### 現在の実装との差異

**⚠️ 重要**: 現在のトランスパイラー実装（`crates/pasta/src/transpiler/mod.rs`）は以下の点で要件と乖離している：

1. グローバルラベルがモジュール化されず、フラットな関数として生成されている
2. `__start__`関数が生成されていない
3. ローカルラベルが親モジュール内に配置されず、`親名__子名`形式でフラット化されている
4. `ctx.pasta.call()`/`ctx.pasta.jump()`形式ではなく、直接関数呼び出しになっている

これらは設計フェーズで全面的に修正する必要がある。

#### Acceptance Criteria

**トランスパイラー実装の前提条件**:
- トランスパイラーは`std::io::Write`トレイトを出力先として受け取る
- Pass 1は複数回呼び出し可能（各PastaFileごとに処理、LabelRegistryに蓄積）
- Pass 2は全ファイル処理後に1回のみ呼び出し
- Pass 1とPass 2は文字列生成のみ行い、Runeコンパイルは実行しない
- Runeコンパイルは最終的な完全なRuneコード文字列に対して1回のみ実行される
- 出力先はメモリ（String）、ファイル、標準出力など柔軟に対応可能
- `transpile_to_string()`便利メソッドはテスト専用（本番コードでは使用しない）

1. When トランスパイラーがグローバル単語定義（`＠グローバル単語：値1 値2`）を処理する, the Pasta Transpiler shall `add_words("単語名", ["値1", "値2"])`をモジュール外部で生成する
2. When トランスパイラーがグローバルラベルを処理する, the Pasta Transpiler shall `pub mod ラベル名_番号 { ... }`形式のRuneモジュールを生成する
3. When トランスパイラーがグローバルラベルの最初のスコープ（ローカルラベルなし）を処理する, the Pasta Transpiler shall `pub fn __start__(ctx) { ... }`関数を生成する
4. When トランスパイラーがローカルラベルを処理する, the Pasta Transpiler shall `pub fn ラベル名_番号(ctx) { ... }`関数を生成する
5. When トランスパイラーがローカル単語定義（`＠単語：値1 値2`）を処理する, the Pasta Transpiler shall `ctx.pasta.add_words("単語", ["値1", "値2"]); ctx.pasta.commit_words();`を生成する
6. When トランスパイラーが変数代入（`＄変数＝値`）を処理する, the Pasta Transpiler shall `ctx.save.変数 = 値;`を生成する
7. When トランスパイラーが引数なしcall文（`＞ラベル名`）を処理する, the Pasta Transpiler shall `for a in pasta::call(ctx, "検索キー", #{}, []) { yield a; }`を生成する（検索キー: グローバル="会話", ローカル="会話_1::選択肢"）
8. When トランスパイラーが引数付きcall文（`＞ラベル名（引数1　引数2）`）を処理する, the Pasta Transpiler shall `for a in pasta::call(ctx, "検索キー", #{}, [引数1, 引数2]) { yield a; }`を生成する
9. When トランスパイラーがjump文（`？ラベル名`）を処理する, the Pasta Transpiler shall call文と同様の形式で`pasta::jump(ctx, "検索キー", #{}, [...])`を生成する
10. When トランスパイラーが発言者切り替え（`さくら：`）を処理する, the Pasta Transpiler shall `ctx.actor = さくら; yield Actor("さくら");`を生成する（`さくら`はグローバル変数として定義された発言者オブジェクト）
11. When トランスパイラーがローカルラベル関数を生成する, the Pasta Transpiler shall すべての関数を`pub fn 名前(ctx)`シグネチャで統一し、引数は`ctx.args`経由でアクセスする
12. When Pastaランタイムが`pasta::call()`を実装する, the Pasta Runtime shall `ctx.args`を第4引数として渡し、呼び出し先関数で`ctx.args`経由でアクセスできるようにする
13. When Pastaランタイムが`pasta::call()`または`pasta::jump()`を実装する, the Pasta Runtime shall 呼び出し先関数からyieldされるイベントを`yield event`で透過的に伝播し、結果を配列に蓄積してはならない（ジェネレーター関数の本質）

#### 出力例（リファレンス実装）

**入力 Pasta DSL:**
```pasta
＠グローバル単語：はろー　わーるど

＊会話
　＠場所：東京　大阪
　＄変数＝１０
　＞コール１
　＞コール２
　？ジャンプ

　ージャンプ
　さくら：良い天気だね。
　うにゅう：せやね。

　ージャンプ
　さくら：＠場所　では雨が降ってる。
　うにゅう：ぐんにょり。

　ーコール１
　さくら：はろー。

　ーコール２
　うにゅう：わーるど。
```

**期待される出力 Rune:**
```rune
use pasta_stdlib::*;

add_words("グローバル単語", ["はろー", "わーるど"]);

pub mod 会話_1 {
    pub fn __start__(ctx) {
        ctx.pasta.add_words("場所", ["東京", "大阪"]); 
        ctx.pasta.commit_words();
        ctx.save.変数 = 10;
        for a in pasta::call(ctx, "会話_1::コール１", #{}, []) { yield a; }
        for a in pasta::call(ctx, "会話_1::コール２", #{}, []) { yield a; }
        for a in pasta::jump(ctx, "会話_1::ジャンプ", #{}, []) { yield a; }
    }

    pub fn ジャンプ_1(ctx) {
        // さくら：良い天気だね。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("良い天気だね。");
        // うにゅう：せやね。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("せやね。");
    }

    pub fn ジャンプ_2(ctx) {
        // さくら：＠場所　では雨が降ってる。
        ctx.actor = さくら;
        yield Actor("さくら");
        for a in pasta::word(ctx, "場所", []) { yield a; }
        yield Talk("では雨が降ってる。");
        // うにゅう：ぐんにょり。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("ぐんにょり。");
    }

    pub fn コール１_1(ctx) {  
        // さくら：はろー。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("はろー。");
    }

    pub fn コール２_1(ctx) {
        // うにゅう：わーるど。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("わーるど。");
    }
}

// Pass 2で生成
pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for event in label_fn(ctx, args) { yield event; }
    }
    
    pub fn call(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for event in label_fn(ctx, args) { yield event; }
    }
    
    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::会話_1::コール１_1,
            3 => crate::会話_1::コール２_1,
            4 => crate::会話_1::ジャンプ_1,
            5 => crate::会話_1::ジャンプ_2,
            _ => |ctx, args| {
                yield Error(`ラベルID ${id} が見つかりませんでした。`);
            },
        }
    }
    
    pub fn word(ctx, keyword, args) {
        // P1実装対象
        yield Talk("[単語未実装]");
    }
}
```

### Requirement 6: サンプルファイルの修正

**Objective:** 開発者として、`04_control_flow.pasta`を元仕様に準拠した正しい実装例として書き直し、宣言的コントロールフロー構文の使用方法を示す。

#### Acceptance Criteria

1. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall 現在含まれているすべての`＠if`, `＠elif`, `＠else`, `＠while`構文を削除する
2. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall call/jump/ラベル定義を使用した宣言的な実装例を提供する
3. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall ランダム選択とキャッシュベース消化の実装例を含める
4. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall 動的call/jumpを使用したメニュー選択の実装例を含める
5. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall ファイル冒頭のコメントを修正し、「宣言的コントロールフロー」を正しく説明する

### Requirement 7: 包括的なリファレンス実装とテストスイート

**Objective:** 開発者として、実装初期段階で仕様の理解齟齬を防ぐため、包括的なコントロールフロー実装例とトランスパイル結果の参照実装を用意する。

**Background**: トランスパイラー実装時の最初のタスクとして、期待されるトランスパイル結果を`.rn`ファイルとして定義し、包括的なテストスイートを設定することで、実装の早い段階で勘違いの発生を抑制する。

#### Acceptance Criteria

1. When 開発者が実装タスクを開始する, the Development Team shall 最初のタスクとして包括的な`comprehensive_control_flow.pasta`サンプルファイルを作成する
2. When 開発者が`comprehensive_control_flow.pasta`を作成する, the Development Team shall 以下の全機能を網羅した実装例を含める:
   - グローバルラベル定義とローカルラベル定義
   - call文（引数なし、引数あり）
   - jump文（引数なし、引数あり）
   - ロングジャンプ（`＞＊グローバルーローカル`）
   - 動的call/jump（`＞＠変数名`）
   - 同名ラベルの複数定義（ランダム選択）
   - 前方一致選択
   - 変数代入（ローカル/グローバル）
   - Runeブロック内での条件分岐
   - 発言者切り替え
   - 単語定義（グローバル/ローカル）
   - 単語展開（`＠単語名`）
3. When 開発者が`comprehensive_control_flow.pasta`を作成する, the Development Team shall 期待されるトランスパイル結果を`comprehensive_control_flow.rn`として作成する
4. When 開発者が`comprehensive_control_flow.rn`を作成する, the Development Team shall Requirement 5のトランスパイラー出力仕様（モジュール構造、`__start__`関数、while-let-yield、ctx構造）に厳密に準拠した内容とする
5. When 開発者が実装タスクを開始する, the Development Team shall `comprehensive_control_flow.pasta`のトランスパイル結果と`comprehensive_control_flow.rn`を比較する包括的なユニットテストを作成する
6. When 開発者がユニットテストを作成する, the Development Team shall 以下の検証項目を含める:
   - モジュール構造の正確性（グローバルラベル → `pub mod`）
   - `__start__`関数の生成
   - ローカルラベル関数の親モジュール内配置
   - call/jump文のwhile-let-yieldパターン生成
   - 引数配列の正確な生成
   - `ctx.pasta.call()`/`ctx.pasta.jump()`呼び出し形式
7. When 開発者がトランスパイラー実装を進める, the Development Team shall ユニットテストが全てパスすることを確認してから次のタスクに進む

#### ファイル配置

- **入力**: `crates/pasta/tests/fixtures/comprehensive_control_flow.pasta`
- **期待出力**: `crates/pasta/tests/fixtures/comprehensive_control_flow.expected.rn`
- **テストコード**: `crates/pasta/tests/transpiler_comprehensive_test.rs`

### Requirement 8: ラベル検索装置と単語検索装置のVM初期化

**Objective:** 開発者として、ラベル検索装置（LabelTable）と単語検索装置（WordDictionary）をRune VM内で安全に動作させるため、Send traitを実装し、VM初期化後の最初の関数呼び出し時にVM内に送り込む仕組みを実装する。

#### Background

Pastaエンジン自体はRune VMインスタンスを保持しているため、エンジンオブジェクト自体を直接VM内に送り込むことはできない（所有権の問題）。そのため、ラベル検索と単語検索の機能を独立した構造体として実装し、それらをVM初期化後に`ctx.pasta`オブジェクトの一部としてVM内に送り込む必要がある。

**ライフサイクルの特性**:
- ラベル検索装置と単語検索装置は、**DSLの解釈（トランスパイル）が完了したタイミングで完成**する
- 完成後は、**Rune VM内部からのみ参照**される（Rust側からの直接アクセスは不要）
- VMへの送り込みは`VM::send_execute()`を使用し、これには`Send` traitが必須

#### Acceptance Criteria

1. When 開発者がラベル検索装置（LabelTable）を実装する, the Pasta Runtime shall `Send` traitを実装し、`VM::send_execute()`でVM内に送り込み可能にする
2. When 開発者が単語検索装置（WordDictionary）を実装する, the Pasta Runtime shall `Send` traitを実装し、`VM::send_execute()`でVM内に送り込み可能にする
3. When PastaエンジンがDSL解釈（トランスパイル）を完了する, the Pasta Engine shall ラベル検索装置と単語検索装置のインスタンスを作成し、Runeの型システムに登録する
4. When Pastaエンジンが最初のスクリプト関数を呼び出す, the Pasta Engine shall `ctx.pasta`オブジェクトを構築し、ラベル検索装置と単語検索装置への参照を含めて`VM::send_execute()`でVM内に送り込む
5. When `ctx.pasta.call()`/`ctx.pasta.jump()`/`ctx.pasta.word()`が実行される, the Pasta Runtime shall VM内に送り込まれたラベル検索装置と単語検索装置を使用してラベル解決・単語展開を実行する
6. When 開発者がPastaエンジン構造体を設計する, the Development Team shall エンジン自体をVM内に送り込まないアーキテクチャを採用し、検索装置のみをVM内で動作させる
7. When 開発者が`ctx`オブジェクト構造を実装する, the Development Team shall `ctx.pasta`フィールドに検索装置への参照を保持し、Rune関数から直接アクセス可能にする
8. When 検索装置がVM内に送り込まれた後, the Pasta Runtime shall Rust側からの直接アクセスを行わず、Rune VM内部からのみ参照されることを保証する

#### アーキテクチャ原則

**所有権の分離**:
```
PastaEngine (Rust側)
  ├── vm: rune::Vm             // VMインスタンスを所有
  ├── unit: Arc<rune::Unit>    // コンパイル済みコード
  └── (検索装置は所有しない)

ctx.pasta (Rune VM内)
  ├── label_table: LabelTable     // Send実装、VM内で動作
  ├── word_dict: WordDictionary   // Send実装、VM内で動作
  └── (メソッド: call, jump, word, add_words等)
```

**初期化フロー**:
1. Pastaエンジン作成時: `LabelTable`と`WordDictionary`をRune型システムに登録
2. スクリプト実行開始時: 検索装置インスタンスを作成
3. `__start__`関数呼び出し時: `ctx`オブジェクトを構築し、検索装置を含めてVM内に送り込む
4. Rune関数実行中: `ctx.pasta.*`メソッドが検索装置にアクセス

**Send要件の理由**:
- `VM::send_execute()`の使用には`Send` traitが必須（API制約）
- Rune VMはGenerator実行をサポートし、内部で中断・再開が発生する
- 実質的にはマルチタスクは発生しないが、Generator動作のため念のため`Send`付きで実装
- `Send`がないと、Rune型システムに登録できない（コンパイルエラー）
- 検索装置の内部状態（HashMap等）も`Send`を満たす必要がある

---

## Related Documentation

- `.kiro/specs/completed/areka-P0-script-engine/requirements.md` - 元仕様の要件定義
- `crates/pasta/GRAMMAR.md` - Pasta DSL文法リファレンス
- `crates/pasta/examples/scripts/` - サンプルスクリプト集

---

## Acceptance Validation

以下の基準をすべて満たす場合、本仕様の実装は成功とみなされる：

1. `comprehensive_control_flow.pasta`が全機能を網羅し、期待される`.rn`ファイルとの照合テストがパスする
2. `comprehensive_control_flow.rn`が要件5のトランスパイラー出力仕様に厳密に準拠している
3. 包括的なユニットテスト（`transpiler_comprehensive_test.rs`）が全てパスする
4. `04_control_flow.pasta`に命令型構文（`＠if`, `＠elif`, `＠else`, `＠while`）が含まれない
5. call/jump/ラベル定義を使用した宣言的なコントロールフロー例が実装されている
6. ランダム選択と前方一致の動作例が含まれている
7. 動的call/jumpを使用したメニュー選択例が含まれている
8. トランスパイラーが要件5で定義された出力規則に従ってRuneコードを生成する
9. 生成されたRuneコードがwhile-let-yieldパターンを使用してyieldイベントを正しく伝播する
10. すべてのサンプルコードがPasta Engineで正常に実行できる
11. `LabelTable`と`WordDictionary`が`Send` traitを実装している
12. VM初期化後の最初の関数呼び出し時に、検索装置がVM内に正しく送り込まれる
13. `ctx.pasta`オブジェクトが検索装置への参照を保持し、Rune関数からアクセス可能である
