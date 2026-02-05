# Requirements Document

## Introduction

本ドキュメントは、pasta_sample_ghostのテスト `test_event_files_do_not_contain_actor_dictionary` が誤ってfailしている問題を修正するための要件を定義する。テストがアクターマーカー（`％`）の2つの異なる用途を混同していることが根本原因であり、テストロジックを改善して正確な検出を実現する。

---

## Background Analysis（背景分析）

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

**ファイル**: [scripts.rs](../../../crates/pasta_sample_ghost/src/scripts.rs#L213-L240)

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

テストは「`％女の子` という文字列がboot.pastaに含まれていない」ことを確認しているが、**シーン内アクタースコープ指定（`　％女の子、男の子`）は設計上正当な使用法**である。

テストの意図は「アクター**辞書**（単語定義を含むブロック）がactors.pasta以外に存在しないこと」を確認することだが、**単純な文字列検索では2つの用途を区別できない**。

### 構文上の区別

| 用途               | パターン                                            | 例                          |
| ------------------ | --------------------------------------------------- | --------------------------- |
| グローバル辞書定義 | 行頭（インデントなし）`％name` + 次行に`　＠word：` | `％女の子\n　＠笑顔：\s[0]` |
| シーン内スコープ   | インデント付き `　％name`                           | `　％女の子、男の子`        |

### 影響範囲

- **BOOT_PASTA**: `　％女の子、男の子` をシーンアクタースコープとして使用中（正当な使用）
- **TALK_PASTA**: アクタースコープを使用していない（会話行でアクター名直接使用）
- **CLICK_PASTA**: アクタースコープを使用していない

---

## Requirements

### Requirement 1: テストロジックの修正

**Objective:** As a 開発者, I want テストが2種類のアクターマーカー用途を区別できるようになる, so that 正当なシーン内アクタースコープ指定がテスト失敗の原因にならない。

#### Acceptance Criteria

1. When テストが実行される, the Test Suite shall グローバルアクター辞書定義（行頭`％name`＋次行`　＠word：`）のみを検出対象とする
2. When boot.pastaにシーン内アクタースコープ（`　％女の子、男の子`）が含まれる, the Test Suite shall テストをpassさせる
3. When boot.pastaに誤ってグローバルアクター辞書定義がコピーされた, the Test Suite shall テストをfailさせる
4. The Test Suite shall テスト名をその目的を正確に反映したものに変更する（例: `test_event_files_do_not_contain_global_actor_dictionary`）

### Requirement 2: 検出パターンの定義

**Objective:** As a 開発者, I want グローバルアクター辞書定義を正確に検出するパターンを持つ, so that テストが仕様に準拠した検証を行える。

#### Acceptance Criteria

1. The Test Suite shall グローバルアクター辞書を「行頭（インデントなし）の`％actor_name`」パターンで識別する
2. The Test Suite shall シーン内アクタースコープを「インデント付きの`　％actor_name`」パターンとして許容する
3. When 検出パターンを適用する, the Test Suite shall 正規表現または行頭文字列マッチングを使用する（`\n％` または文字列先頭での`％`）
4. The Test Suite shall SPECIFICATION.mdのアクター定義仕様（§7.2）と整合する検出ロジックを実装する

### Requirement 3: スクリプトテンプレートの整合性維持

**Objective:** As a 開発者, I want スクリプトテンプレートがpasta DSLの設計哲学に従っていることを確認できる, so that ファイルの役割分担が明確に保たれる。

#### Acceptance Criteria

1. The Test Suite shall actors.pastaにグローバルアクター辞書定義が存在することを確認する
2. The Test Suite shall boot.pasta、talk.pasta、click.pastaにグローバルアクター辞書定義が存在しないことを確認する
3. While シーン内でアクタースコープ指定（`　％name`）が使用されている, the Test Suite shall これを正当な使用として許容する
4. The Test Suite shall コメント（`＃ ※アクター辞書は actors.pasta で共通定義`）と実装の整合性を維持する

### Requirement 4: テストの安全網としての価値維持

**Objective:** As a 開発者, I want 修正後もテストが誤った辞書定義の混入を検出できる, so that コードベースの品質が維持される。

#### Acceptance Criteria

1. If グローバルアクター辞書定義（`％name\n　＠word：`パターン）がboot.pastaに追加された, then the Test Suite shall テストをfailさせる
2. If グローバルアクター辞書定義がtalk.pastaまたはclick.pastaに追加された, then the Test Suite shall テストをfailさせる
3. When cargo test -p pasta_sample_ghost が実行される, the Test Suite shall 全テストをpassさせる（現在失敗しているテストが修正される）
4. The Test Suite shall integration_test.rsの同等テストも同じロジックで修正する（DRY原則）
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
