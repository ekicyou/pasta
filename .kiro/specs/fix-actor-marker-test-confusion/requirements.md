# Requirements Document

## Project Description (Input)
Issue 1: pasta_sample_ghost テスト失敗 - `test_event_files_do_not_contain_actor_dictionary` テストが誤ってfailしている問題の修正。テストがアクターマーカー(`％`)の2つの異なる用途を混同している。

---

## 問題分析結果

### 問題の背景

pasta DSLにおいて、アクターマーカー `％` は**2つの異なるコンテキスト**で使用される：

1. **グローバルアクター辞書定義** (Actor Dictionary Definition)
   - 場所: ファイルレベル（インデントなし）
   - 目的: アクターごとの表情・属性辞書を定義
   - 例: actors.pasta
   ```pasta
   ％女の子
     ＠笑顔：\s[0]
     ＠通常：\s[1]
   ```

2. **シーン内アクタースコープ指定** (Scene Actor Scope)
   - 場所: シーンスコープ内（インデントあり）
   - 目的: そのシーンで使用するアクターを宣言
   - 例: boot.pasta, talk.pasta, click.pasta
   ```pasta
   ＊OnBoot
     ％女の子、男の子
     女の子：＠通常　起動したよ～。
   ```

### 失敗しているテスト

**ファイル**: `crates/pasta_sample_ghost/src/scripts.rs:213-240`

```rust
#[test]
fn test_event_files_do_not_contain_actor_dictionary() {
    // アクター辞書は actors.pasta のみに定義されることを確認
    assert!(
        !BOOT_PASTA.contains("％女の子"),  // ← 失敗！
        "boot.pasta にアクター辞書が含まれています"
    );
    // ...
}
```

### 失敗の根本原因

テストは「`％女の子` という文字列がboot.pastaに含まれていない」ことを確認しているが、**シーン内アクタースコープ指定（`％女の子、男の子`）は設計上正当な使用法**である。

テストの意図は「アクター**辞書**（単語定義を含むブロック）がactors.pasta以外に存在しないこと」を確認することだが、**単純な文字列検索では2つの用途を区別できない**。

### 影響範囲

- **BOOT_PASTA**: `％女の子、男の子` をシーンアクタースコープとして使用中
- **TALK_PASTA**: アクタースコープを使用していない（会話行でアクター名直接使用）
- **CLICK_PASTA**: アクタースコープを使用していない

---

## Requirements

### REQ-1: テストの目的再定義

**要件**: テストの真の目的を明確化し、2つのアクターマーカー用途を区別できるテストに修正する。

**受け入れ基準**:
- テストが「グローバルアクター辞書定義」と「シーン内アクタースコープ指定」を区別できること
- テスト名と説明が目的を正確に反映すること
- テストがpassすること（cargo test -p pasta_sample_ghost）

### REQ-2: アクター辞書定義の検出方法

**要件**: グローバルアクター辞書定義を正確に検出するメカニズムを定義する。

**受け入れ基準**:
- グローバルアクター辞書は「行頭（インデントなし）の `％actor_name`」で開始
- 直後に単語定義行（`　＠word_name：value`）が続く構造を持つ
- シーン内アクタースコープは「インデント付き `％actor_name` 」で始まり、シーン定義 (`＊`) の配下にある
- 検出方法がSPECIFICATION.mdの仕様と一致すること

### REQ-3: スクリプトテンプレートの設計整合性

**要件**: スクリプトテンプレート（BOOT_PASTA等）が pasta DSL の設計哲学に従っていることを確認。

**受け入れ基準**:
- 各ファイルの役割分担が明確（actors.pasta = 辞書定義、その他 = シーン定義）
- シーン内でのアクタースコープ指定は正当な使用法として許容
- コメント（`＃ ※アクター辞書は actors.pasta で共通定義`）と実装の整合性

### REQ-4: テストカバレッジの維持

**要件**: 修正後もテストの安全網としての価値を維持する。

**受け入れ基準**:
- アクター辞書が actors.pasta 以外に定義されたら検出できること
- 誤って辞書定義がコピーされた場合にテストがfailすること
- false positiveを排除しつつtrue positiveを維持

---

## 参照資料

### 関連ファイル
- [SPECIFICATION.md](../../SPECIFICATION.md) - Section 11: アクター辞書（Actor Dictionary）
- [crates/pasta_core/src/parser/ast.rs](../../crates/pasta_core/src/parser/ast.rs) - ActorScope, SceneActorItem 定義
- [crates/pasta_sample_ghost/src/scripts.rs](../../crates/pasta_sample_ghost/src/scripts.rs) - 失敗しているテスト

### 構文仕様引用（SPECIFICATION.md Section 11.2-11.3）

**グローバルアクター辞書定義**:
```
％actor_name
  ＠word_name：value1、value2、...
```
- ファイルレベル（インデントなし）で `％actor_name` を記述

**シーンスコープ内でのアクター指定**:
```
＊scene_name
  ％actor1、actor2
  actor1：＠通常　こんにちは
```
- シーン内で `％actor1、actor2` と記述するとアクター名プレフィックスが使用可能になる

---

## 解決アプローチ候補

### アプローチA: 正規表現による構文パターン検出
- `^％actor_name` (行頭、インデントなし) をグローバル辞書として検出
- `^\s+％actor_name` (インデントあり) をシーンスコープとして許容
- 利点: 単純で理解しやすい
- 欠点: 複雑なケースで誤検出の可能性

### アプローチB: パーサーを使った構造検証
- pasta_core のパーサーでAST生成
- `FileItem::ActorScope` の存在をチェック
- 利点: 文法的に正確
- 欠点: pasta_sample_ghost → pasta_core 依存追加

### アプローチC: テストの目的変更
- テストの意図を「辞書定義行のパターン検出」に変更
- 例: `＠通常：\s[` のような単語定義構文をチェック
- 利点: シンプルで目的が明確
- 欠点: アクターマーカー自体のチェックではなくなる

**推奨**: アプローチAまたはCを組み合わせた形で、**インデントなしの `％` 行の後に単語定義パターン `＠...：` が続く構造**を検出する。

---

## Next Steps

`/kiro-spec-design fix-actor-marker-test-confusion` で設計フェーズへ進む。
