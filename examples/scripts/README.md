# Pasta DSL Sample Scripts

このディレクトリには、Pasta DSLの機能を示すサンプルスクリプトが含まれています。

## サンプル一覧

### 01_basic_conversation.pasta
**基本的な会話**

- 複数キャラクターの発言
- スピーカーの切り替え
- 同じシーン名での複数定義（ランダム選択）
- 挨拶、自己紹介、雑談、別れの基本パターン

**学べること:**
- シーン定義の基本
- 発言文の記述方法
- ランダムバリエーションの実装

### 02_sakura_script.pasta
**さくらスクリプトエスケープ**

- 表情変更 (`\s[ID]`)
- ウェイト (`\w数字`)
- 改行 (`\n`)
- キャラクター切り替え (`\![raise,name]`)
- 複合表現とアニメーション風の演出

**学べること:**
- さくらスクリプトエスケープシーケンスの使用
- 表情とタイミングの制御
- 視覚的表現の強化

### 03_variables.pasta
**変数と状態管理**

- ローカル変数 (`@変数名`)
- グローバル変数 (`@*変数名`)
- システム変数 (`@**変数名`)
- 変数の展開 (`{変数名}`)
- 型のサポート（文字列、整数、浮動小数点数、真偽値）
- 変数の演算

**学べること:**
- 変数スコープの理解
- 状態管理の実装
- 動的な会話の生成

### 04_control_flow.pasta
**制御構文**

- 条件分岐 (`if/elif/else`)
- ループ (`while`)
- ジャンプ (`jump`)
- 関数呼び出し（グローバル/ローカル）
- 複雑な制御フロー

**学べること:**
- 条件分岐とループの使用
- プログラムフローの制御
- サブルーチンの実装

### 05_synchronized_speech.pasta
**同期セクション**

- `@同時発言開始` / `@同時発言終了`
- `@同期` ポイント
- 複数キャラクターの掛け合い
- 感情表現を含む同期
- パフォーマンス風の表現

**学べること:**
- 複数キャラクターの同時発言
- 同期ポイントの配置
- タイミングを意識した会話設計

### 06_event_handlers.pasta
**イベントハンドリング**

- イベント命名規則 (`On<EventName>`)
- システムイベント（起動、終了、クリックなど）
- 時間ベースのイベント
- カスタムイベント
- 属性フィルタリング
- イベントチェーン

**学べること:**
- イベントドリブンな対話
- イベントハンドラの実装
- 属性によるフィルタリング

## 使用方法

### Rustコードからの実行

```rust
use pasta_rune::PastaEngine;
use std::fs;

// スクリプトファイルを読み込む
let script = fs::read_to_string("examples/scripts/01_basic_conversation.pasta")?;

// エンジンを初期化
let mut engine = PastaEngine::new(&script)?;

// シーンを実行
let events = engine.execute_label("挨拶")?;

// イベントを処理
for event in events {
    match event {
        pasta::ScriptEvent::Talk { speaker, content } => {
            println!("{}: {:?}", speaker, content);
        }
        pasta::ScriptEvent::Wait { duration } => {
            println!("Wait: {}s", duration);
        }
        _ => {}
    }
}
```

### イベントハンドラの実行

```rust
use std::collections::HashMap;

// イベントを発火
let events = engine.on_event("Click", HashMap::new())?;

// または、属性フィルタを使用
let mut filters = HashMap::new();
filters.insert("time".to_string(), "morning".to_string());
let events = engine.on_event("Weather", filters)?;
```

## 学習パス

1. **初心者**: 
   - `01_basic_conversation.pasta` で基本を学ぶ
   - シーンと発言の基礎を理解

2. **初級者**:
   - `03_variables.pasta` で変数を学ぶ
   - `04_control_flow.pasta` で制御構文を学ぶ

3. **中級者**:
   - `02_sakura_script.pasta` でエスケープシーケンスを学ぶ
   - `06_event_handlers.pasta` でイベント処理を学ぶ

4. **上級者**:
   - `05_synchronized_speech.pasta` で同期セクションを学ぶ
   - 複数のサンプルを組み合わせて複雑なシナリオを作成

## ベストプラクティス

### 1. シーン命名

```pasta
// ✓ Good: 説明的な名前
＊挨拶_朝
＊ゲーム開始

// ✗ Bad: 曖昧な名前
＊test
＊a
```

### 2. コメントの活用

```pasta
// スクリプトの目的を明記
// セクションを区切る
// 複雑なロジックには説明を追加
```

### 3. 変数スコープの選択

```pasta
// ローカル: 一時的な計算
＊計算
    ＠temp：10 + 20
    さくら：結果は{temp}です

// グローバル: ゲーム状態
＊スコア更新
    ＠＊score：{＊score} + 10

// システム: ユーザー設定
＊設定保存
    ＠＊＊username：太郎
```

### 4. 同期セクションの設計

```pasta
// 明確な開始と終了
＠同時発言開始
    // 適切な同期ポイント配置
    キャラクター1：発言
    ＠同期
    キャラクター2：返答
＠同時発言終了
```

### 5. イベントハンドラの整理

```pasta
// イベント名は明確に
＊OnUserLogin
＊OnGameStart
＊OnError

// 属性で細分化
＊OnNotification
    ＠priority：high
```

## トラブルシューティング

### よくあるエラー

1. **Parse error**: インデントが揃っていない
   ```pasta
   // ✗ Bad
   ＊シーン
   さくら：発言  // インデントなし
   
   // ✓ Good
   ＊シーン
       さくら：発言  // インデントあり
   ```

2. **LabelNotFound**: シーン名のタイポ
   ```pasta
   ＊挨拶
       ＠jump：あいさつ  // ✗ タイポ
   ```

3. **変数未定義**: 使用前に定義
   ```pasta
   // ✗ Bad
   さくら：スコアは{score}です  // 未定義
   
   // ✓ Good
   ＠score：100
   さくら：スコアは{score}です
   ```

## 追加リソース

- [文法リファレンス](../../GRAMMAR.md) - 完全な文法ドキュメント
- [設計ドキュメント](.kiro/specs/areka-P0-script-engine/design.md) - アーキテクチャと設計決定
- [APIドキュメント](https://docs.rs/pasta) - Rust API リファレンス

## コントリビューション

新しいサンプルスクリプトの追加を歓迎します！以下のガイドラインに従ってください：

1. **ファイル名**: `番号_説明的な名前.pasta`
2. **コメント**: スクリプトの目的と学べることを明記
3. **README更新**: このREADMEに新しいサンプルの説明を追加
4. **テスト**: サンプルが正しく動作することを確認

---

**Happy Scripting with Pasta!** 🍝
