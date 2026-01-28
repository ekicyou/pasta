# Requirements Document

## Project Description (Input)
`act:word(name)` のLua実装において、グローバル単語辞書への検索を実装する。現在はシーンフィールドのみ検索しているが、これを拡張してグローバル単語辞書（`WORD.get_global_words()`）への前方一致検索を追加する。

実装方針として、`PROXY_IMPL.word()`の6レベルフォールバック実装をリファクタリングする。具体的には、アクター辞書参照部分（Level 1, Level 2）を分離し、それ以降の検索ロジック（Level 3-6）を`ACT_IMPL.word()`として共通化する。これにより`PROXY_IMPL.word()`は「アクター辞書検索（L1, L2）→ 失敗時にact:word()へフォールバック」という明確な責務分離を実現する。

---

## Introduction

本仕様は、PASTAスクリプトエンジンにおける単語検索機能の拡張を定義する。`act:word(name)` API をグローバル単語辞書まで検索するよう拡張し、既存の `PROXY_IMPL.word()` と検索ロジックを共通化することで、保守性と一貫性を向上させる。

## Requirements

### Requirement 1: ACT_IMPL.word 4レベルフォールバック検索

**Objective:** PASTAスクリプト開発者として、`act:word(name)` からアクター非依存の単語検索を行いたい。これにより、シーンローカル単語だけでなくグローバル単語辞書からも単語を取得できる。

#### Acceptance Criteria

1. When `act:word(name)` が呼び出される, the ACT_IMPL shall 現在のシーンテーブルから `name` に完全一致するエントリを検索する
2. When シーンテーブルに完全一致がない, the ACT_IMPL shall シーンローカル単語辞書（`WORD.get_local_words(scene_name)`）から前方一致検索を行う
3. When シーンローカル単語辞書に一致がない, the ACT_IMPL shall グローバルテーブル（`GLOBAL[name]`）から完全一致で検索する
4. When グローバルテーブルに完全一致がない, the ACT_IMPL shall グローバル単語辞書（`WORD.get_global_words()`）から前方一致検索を行う
5. If 全レベルで一致がない, then the ACT_IMPL shall `nil` を返す

### Requirement 2: PROXY_IMPL.word 責務分離リファクタリング

**Objective:** PASTAスクリプトエンジンのメンテナとして、`PROXY_IMPL.word()` と `ACT_IMPL.word()` の検索ロジックを共通化したい。これにより、コードの重複を排除し保守性を向上させる。

#### Acceptance Criteria

1. The PROXY_IMPL.word shall アクター固有検索（Level 1-2）を担当し、それ以降は `act:word()` へ委譲する
2. When アクター完全一致（Level 1）で見つかる, the PROXY_IMPL shall その値を返し、後続検索を行わない
3. When アクター辞書前方一致（Level 2）で見つかる, the PROXY_IMPL shall その値を返し、後続検索を行わない
4. When アクター検索（Level 1-2）で見つからない, the PROXY_IMPL shall `act:word(name)` を呼び出して結果を返す

### Requirement 3: 値解決の一貫性

**Objective:** PASTAスクリプト開発者として、単語検索の結果が関数である場合は実行結果を得たい。これにより、動的単語生成をサポートする。

#### Acceptance Criteria

1. When 検索結果が関数である, the ACT_IMPL shall 関数を `act` オブジェクトを引数として呼び出し、その戻り値を返す
2. When 検索結果が文字列である, the ACT_IMPL shall その文字列をそのまま返す
3. When 完全一致で検索結果がテーブルである, the ACT_IMPL shall テーブル内から最初の要素を選択して返す
4. When 前方一致検索で複数の候補がある, the ACT_IMPL shall ランダムに1つを選択して返す

### Requirement 4: 後方互換性の維持

**Objective:** 既存のPASTAスクリプトの動作を壊さないこと。

#### Acceptance Criteria

1. The ACT_IMPL shall 既存の `act:word()` 呼び出し箇所で従来通りシーンテーブル検索が動作すること
2. The PROXY_IMPL shall 既存の `act.さくら:word("通常")` 呼び出し箇所で6レベルフォールバック動作が維持されること
3. While 既存テストが存在する, the ACT_IMPL shall 全ての既存テストがパスすること

---

## 検索レベル対照表

| レベル | PROXY_IMPL.word（従来）    | ACT_IMPL.word（新規）      | 責務分担後 |
| ------ | -------------------------- | -------------------------- | ---------- |
| L1     | アクター完全一致           | -                          | PROXY_IMPL |
| L2     | アクター辞書（前方一致）   | -                          | PROXY_IMPL |
| L3     | シーン完全一致             | シーン完全一致             | ACT_IMPL   |
| L4     | シーン辞書（前方一致）     | シーン辞書（前方一致）     | ACT_IMPL   |
| L5     | グローバル完全一致         | グローバル完全一致         | ACT_IMPL   |
| L6     | グローバル辞書（前方一致） | グローバル辞書（前方一致） | ACT_IMPL   |

---

## 関連ファイル

| ファイル                                                         | 役割                                    |
| ---------------------------------------------------------------- | --------------------------------------- |
| [act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua)       | ACT_IMPL 実装（修正対象）               |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua)   | PROXY_IMPL 実装（リファクタリング対象） |
| [word.lua](../../../crates/pasta_lua/scripts/pasta/word.lua)     | WORD モジュール（単語辞書アクセス）     |
| [global.lua](../../../crates/pasta_lua/scripts/pasta/global.lua) | GLOBAL テーブル                         |
