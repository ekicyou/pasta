# Requirements Document

## Project Description (Input)

Pastaエンジンにおいて、Call文（＞label）で呼び出されたラベル関数内のyieldイベントが親関数に伝搬されない問題が発覚。Runeはネストした関数内のyieldを透過的に返さない仕様のため、現在のトランスパイル結果では呼び出し先の会話イベントが全て失われる。この問題の解決方法を要件定義し、設計・実装せよ。Jump文（－label）も同様の問題がないか調査し、必要なら対応すること。

---

## Introduction

### 背景

PastaはRust製の対話スクリプトエンジンであり、Rune VMを使用してトランスパイルされたスクリプトを実行します。Pastaスクリプトのラベル関数はRuneのgenerator関数として実装され、`yield`で会話イベント（`ScriptEvent`）を順次返します。

現在、Call文（`＞label`）でラベル関数を呼び出す際、トランスパイラは単純な関数呼び出し `label(ctx)` を生成しますが、**Runeはネストした関数内のyieldを親関数に透過的に伝搬しない**ため、呼び出し先のイベントが全て失われます。

**検証結果**（`test_yield_behavior.rs`）:
```rune
pub fn a() {
    yield 11;
    yield 12;
    b();      // ← b()内のyield 21, 22は返らない
    yield 13;
}
fn b() {
    yield 21;
    yield 22;
}
// 実際の出力: 11, 12, 13 （21, 22が消失）
```

この問題により、Call文で呼び出されたラベルの全ての会話イベントが無視され、Pastaスクリプトが正しく動作しません。

### 目的

本仕様は、Pastaエンジンにおいて**Call文とJump文で呼び出されたラベル関数のyieldイベントを正しく伝搬する機構**を実装することを目的とします。

### スコープ

- **対象**: Call文（`＞label`）、Jump文（`－label`）、Transpiler出力形式
- **アプローチ**: Transpiler修正によるyield伝搬ロジックの生成とモジュール構造化
- **制約**: 既存のPastaスクリプトとの後方互換性を維持、Rune VM仕様に準拠

### Transpiler出力形式の方針

トランスパイラは以下の構造を持つRune IRを生成する：

1. **モジュール構造**: グローバルラベルごとに`pub mod ラベル名_連番`を生成
2. **エントリーポイント**: 各モジュールに`pub fn __start__(ctx)`関数（ラベルなしローカルスコープ）
3. **ローカルラベル関数**: ローカルラベルごとに`pub fn ラベル名_連番(ctx)`を生成
4. **yield伝搬**: Call文は`while let Some(a) = ctx.pasta.call(...).next() { yield a; }`形式
5. **Jump文**: `while let Some(a) = ctx.pasta.jump(...).next() { yield a; }`形式
6. **単語定義**: `ctx.pasta.add_words()`で辞書登録、`commit_words()`で確定
7. **単語参照**: `while let Some(a) = ctx.pasta.word(ctx, "名前").next() { yield a; }`形式

---

## Requirements

### Requirement 1: Call文のyield伝搬

**Objective:** Pastaスクリプト作成者として、Call文で呼び出したラベルの会話イベントが全て実行されることを期待する。これにより、サブルーチン的なラベル呼び出しが正しく機能する。

#### Acceptance Criteria

1.1. When Pasta Call文（`＞label`）でラベル関数を呼び出す, the Pasta Transpiler shall 呼び出し先のyieldイベントを親関数に伝搬するRune IRを生成する

1.2. When 呼び出し先ラベル関数が複数のyieldイベントを返す, the Pasta Runtime shall 全てのイベントを順次実行する

1.3. When 呼び出し先ラベル関数がさらに別のラベルをCallする（ネストしたCall）, the Pasta Runtime shall 全階層のyieldイベントを正しく伝搬する

1.4. When Call文で呼び出されたラベル関数が完了する, the Pasta Runtime shall 呼び出し元の次の処理を継続する

1.5. If 呼び出し先ラベル関数が実行時エラーをyieldする（`ScriptEvent::Error`）, then the Pasta Runtime shall エラーイベントを親関数に伝搬し、実行を継続する

### Requirement 2: Jump文のyield伝搬

**Objective:** Pastaスクリプト作成者として、Jump文で遷移したラベルの会話イベントが全て実行されることを期待する。これにより、ラベル間の遷移が正しく機能する。

#### Acceptance Criteria

2.1. When Pasta Jump文（`－label`）でラベル関数に遷移する, the Pasta Transpiler shall 遷移先のyieldイベントを親関数に伝搬するRune IRを生成する（または`return`による完全な制御移譲）

2.2. When Jump文で遷移したラベル関数が完了する, the Pasta Runtime shall 元のラベル関数には戻らず、スクリプト実行を終了する（または次のイベントループに移行）

2.3. When Jump文がネストしたラベル内で実行される（Call先からのJump）, the Pasta Runtime shall 全てのCall階層を抜けて、Jump先ラベルに完全に遷移する

2.4. If Jump先ラベル関数が存在しない, then the Pasta Runtime shall エラーイベントをyieldし、スクリプト実行を停止する

### Requirement 3: ctx引数の伝搬

**Objective:** Pastaエンジン開発者として、Call/Jump文で呼び出されたラベル関数に実行コンテキスト（`ctx`）が正しく渡されることを保証する。これにより、アクター情報やスコープ情報が継承される。

#### Acceptance Criteria

3.1. When Call文またはJump文でラベル関数を呼び出す, the Pasta Transpiler shall 現在の`ctx`引数を呼び出し先関数に渡すRune IRを生成する

3.2. When 呼び出し先ラベル関数内でアクターが変更される（`change_speaker`）, the Pasta Runtime shall 変更されたアクター情報を`ctx`に反映する（またはイミュータブルなctxの場合、新しいctxを生成する）

3.3. When Call文で呼び出されたラベル関数が完了して戻る, the Pasta Runtime shall 呼び出し元のctx状態を復元する（または適切に継承する）

3.4. The Pasta Transpiler shall 全てのラベル関数定義に`ctx`引数を含める

### Requirement 4: Transpiler出力形式

**Objective:** Pastaエンジン開発者として、Transpilerが生成するRune IRの構造を標準化し、モジュール化されたコード生成を実現する。

#### Acceptance Criteria

4.1. When グローバルラベルをトランスパイルする, the Pasta Transpiler shall `pub mod ラベル名_連番 { ... }`形式のモジュールを生成する

4.2. When グローバルラベル直後のローカルスコープ（ラベルなし）をトランスパイルする, the Pasta Transpiler shall モジュール内に`pub fn __start__(ctx)`関数を生成する

4.3. When ローカルラベルをトランスパイルする, the Pasta Transpiler shall 同一モジュール内に`pub fn ラベル名_連番(ctx)`関数を生成する

4.4. When 同名のローカルラベルが複数存在する, the Pasta Transpiler shall 連番サフィックス（`_1`, `_2`）で区別する

4.5. When Call文（`＞label`）をトランスパイルする, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.call(ctx, "モジュール名", "関数名").next() { yield a; }`形式のyield伝搬ループを生成する

4.6. When Jump文（`－label`）をトランスパイルする, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.jump(ctx, "モジュール名", "関数名").next() { yield a; }`形式を生成する（制御移譲後は戻らない）

4.7. When 単語定義をトランスパイルする, the Pasta Transpiler shall `ctx.pasta.add_words("名前", ["単語1", "単語2"])`でローカル辞書に登録し、`ctx.pasta.commit_words()`で確定する

4.8. When 単語参照（`＠単語`）をトランスパイルする, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.word(ctx, "名前").next() { yield a; }`形式でyield伝搬を行う

4.9. When 会話行をトランスパイルする, the Pasta Transpiler shall `ctx.actor = アクター名; yield Actor("名前"); yield Talk("テキスト");`形式で生成する

4.10. When Runeブロックが存在する, the Pasta Transpiler shall モジュール末尾にインライン展開する

### Requirement 5: pasta_stdlib API拡張

**Objective:** Pastaエンジン開発者として、yield伝搬を実現するためのstdlib関数を定義し、Rune側から呼び出し可能にする。

#### Acceptance Criteria

5.1. When `ctx.pasta.call()`関数を実装する, the pasta_stdlib shall 指定されたモジュール・関数をgeneratorとして実行し、全yieldイベントをiteratorとして返す

5.2. When `ctx.pasta.jump()`関数を実装する, the pasta_stdlib shall Jump先関数を実行し、完了後に制御を返さない仕組みを提供する

5.3. When `ctx.pasta.word()`関数を実装する, the pasta_stdlib shall 単語辞書検索結果（前方一致+キャッシュ）をgeneratorまたはiteratorとして返す

5.4. When `ctx.pasta.add_words()`関数を実装する, the pasta_stdlib shall ローカル単語辞書に単語を追加する（スコープは`ctx.scope.global`で判定）

5.5. When `ctx.pasta.commit_words()`関数を実装する, the pasta_stdlib shall 追加した単語をTrieに確定し、検索可能にする

5.6. The pasta_stdlib shall 全ての関数をRune VMに登録し、`ctx.pasta.*`形式でアクセス可能にする

### Requirement 6: 後方互換性とテスト

**Objective:** Pastaエンジン開発者として、yield伝搬実装が既存のPastaスクリプトを破壊しないことを保証する。

#### Acceptance Criteria

6.1. When yield伝搬機構を実装する, the Pasta Engine shall 既存のPastaスクリプト例（`examples/*.pasta`）が正しく動作することを検証する

6.2. When テストケースを作成する, the Pasta Test Suite shall ネストしたCall文のyield伝搬を検証するテストを含む

6.3. When テストケースを作成する, the Pasta Test Suite shall Jump文による制御移譲を検証するテストを含む

6.4. When テストケースを作成する, the Pasta Test Suite shall Call/Jump文のエラーハンドリングを検証するテストを含む

6.5. The Pasta Test Suite shall Rune generator APIの動作を検証する単体テスト（`test_yield_behavior.rs`の拡張）を含む

### Requirement 7: ドキュメント更新

**Objective:** Pastaエンジン利用者として、Call/Jump文の動作仕様とyield伝搬の仕組みを理解する。

#### Acceptance Criteria

7.1. When yield伝搬機構を実装する, the Pasta Documentation shall Call文とJump文の動作説明を更新する

7.2. When 実装方式を決定する, the Pasta Documentation shall 採用したアプローチ（Transpiler修正 vs Engine修正）の根拠を記述する

7.3. The Pasta Documentation shall トランスパイル後のRune IRの例を含む（モジュール構造、Call/Jump文のyield伝搬、単語定義・参照）

7.4. The Pasta Documentation shall スクリプト作成者向けにCall/Jump文の使い分けガイドラインを提供する

---

## Transpiler出力例

### Pastaソース
```pasta
＠グローバル単語：　はろー　わーるど

＊会話
　＠場所：東京　大阪
　＄変数＝１０

　＃　ここは「ラベルが無い」最初のローカルスコープ
　＞コール１
　＞コール２
　－ジャンプ

　ージャンプ
　＃　一つ目のローカルラベル
　さくら　：良い天気だね。
　うにゅう：せやね。

　ージャンプ
　＃　二つ目のローカルラベル（同名）
　さくら　：＠場所　では雨が降ってる。
　うにゅう：ぐんにょり。

　ーコール１
　さくら　：はろー。
  
　ーコール２
　うにゅう：わーるど。
```

### 期待されるRune IR
```rune
use pasta::add_words;

add_words("グローバル単語", ["はろー", "わーるど"]);

pub mod 会話_1 {
    pub fn __start__(ctx) {
        ctx.pasta.add_words("場所", ["東京", "大阪"]); 
        ctx.pasta.commit_words();
        ctx.var.変数 = 10;
        
        while let Some(a) = ctx.pasta.call(ctx, "会話_1", "コール１").next() { yield a; }
        while let Some(a) = ctx.pasta.call(ctx, "会話_1", "コール２").next() { yield a; }
        while let Some(a) = ctx.pasta.jump(ctx, "会話_1", "ジャンプ").next() { yield a; }
    }
    
    pub fn ジャンプ_1(ctx) {
        //  さくら：良い天気だね。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("良い天気だね。");
        //  うにゅう：せやね。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("せやね。");
    }
    
    pub fn ジャンプ_2(ctx) {
        //  さくら：＠場所　では雨が降ってる。
        ctx.actor = さくら;
        yield Actor("さくら");
        while let Some(a) = ctx.pasta.word(ctx, "場所").next() { yield a; };
        yield Talk("では雨が降ってる。");
        //  うにゅう：ぐんにょり。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("ぐんにょり。");
    }
    
    pub fn コール１(ctx) {
        //  さくら：はろー。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("はろー。");
    }
    
    pub fn コール２(ctx) {
        //  うにゅう：わーるど。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("わーるど。");
    }
    
    // もし、runeブロックがあれば、ここに展開される
}
```

---

## Non-Functional Requirements

### Performance
- yield伝搬機構のオーバーヘッドは、既存のPastaスクリプト実行速度に対して10%以内の劣化に留める

### Maintainability
- yield伝搬ロジックは、Transpiler層に集約し、pasta_stdlib APIで抽象化する
- モジュール構造により、グローバルラベルごとの独立性を確保

### Compatibility
- Rune VM 0.14のgenerator APIに準拠する
- 将来的なRune VMアップデートに備え、generator操作をpasta_stdlibに集約

---

## Glossary

- **Call文**: Pastaスクリプトの`＞label`構文。ラベル関数を呼び出し、完了後に元のラベルに戻る
- **Jump文**: Pastaスクリプトの`－label`構文。ラベル関数に遷移し、元のラベルには戻らない
- **yield伝搬**: ネストした関数内のyieldイベントを親関数に順次返す処理（`while let Some(a) = gen.next() { yield a; }`形式）
- **ctx**: Pastaラベル関数の第一引数。アクター情報（`ctx.actor`）、スコープ情報（`ctx.scope`）、セーブデータ（`ctx.save`）、pasta API（`ctx.pasta`）、変数（`ctx.var`）を含む実行コンテキスト
- **ScriptEvent**: Pastaランタイムが処理する会話イベント（Talk、Actor、Wait等）
- **Rune generator**: Runeの`yield`を含む関数。`into_generator()`でGenerator型に変換され、`resume()`で実行再開
- **モジュール構造**: グローバルラベルごとに生成される`pub mod ラベル名_連番`。ローカルラベル関数を含む
- **__start__関数**: グローバルラベル直後のローカルスコープ（ラベルなし）に対応するエントリーポイント関数
- **pasta_stdlib**: Rune VMに登録されるPasta標準ライブラリ。`ctx.pasta.*`形式でyield伝搬や単語辞書操作を提供
