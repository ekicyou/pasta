# Requirements Document

## Introduction

pasta_sample_ghost（サンプルゴースト「hello-pasta」）において、イベントスクリプト（`boot.pasta`, `talk.pasta`, `click.pasta`）で `image_generator.rs` に対応する定義を持たない表情名（`＠元気`, `＠考え`）が使用されている。スクリプトを修正し、`image_generator.rs` が生成するサーフェスに対応する表情名のみを使用する状態にする。

### Source of Truth の階層

```
image_generator.rs（Expression enum → サーフェス画像生成）  ← 憲法・最上位
        ↓ 定義するもの以外は存在しない
actors.pasta（アクター辞書 ＠表情名：\s[N]）              ← 憲法に基づく宣言（現状正しい）
        ↓ 宣言されたものだけ使える
boot/talk/click.pasta（シーンスクリプト）                  ← 辞書に従う消費者（← 修正対象）
```

### 正しい表情一覧（Source of Truth: `image_generator.rs`）

| # | `Expression` variant | 目の描画 | `＠表情名` | 女の子 surface | 男の子 surface | スクリプト使用状況 |
|---|---------------------|----------|-----------|---------------|---------------|------------------|
| 0 | `Happy` | `^ ^` | `＠笑顔` | `\s[0]` | `\s[10]` | ✅ 使用中 |
| 1 | `Normal` | `- -` | `＠通常` | `\s[1]` | `\s[11]` | ✅ 使用中 |
| 2 | `Shy` | `> <` | `＠照れ` | `\s[2]` | `\s[12]` | ✅ 使用中 |
| 3 | `Surprised` | `o o` | `＠驚き` | `\s[3]` | `\s[13]` | ✅ 使用中 |
| 4 | `Crying` | `; ;` | `＠泣き` | `\s[4]` | `\s[14]` | 辞書定義済・未使用 |
| 5 | `Confused` | `@ @` | `＠困惑` | `\s[5]` | `\s[15]` | 辞書定義済・未使用 |
| 6 | `Sparkle` | `* *` | `＠キラキラ` | `\s[6]` | `\s[16]` | 辞書定義済・未使用 |
| 7 | `Sleepy` | `= =` | `＠眠い` | `\s[7]` | `\s[17]` | ✅ 使用中 |
| 8 | `Angry` | `# #` | `＠怒り` | `\s[8]` | `\s[18]` | ✅ 使用中 |

### 現状の不一致

| 表情名 | `Expression` variant | actors.pasta 辞書 | スクリプトでの使用 | 状態 |
|--------|---------------------|-------------------|-------------------|------|
| `＠元気` | ❌ なし | ❌ 未定義 | ✅ 男の子で多用（boot, talk, click）7箇所 | **スクリプト側が誤り** |
| `＠考え` | ❌ なし | ❌ 未定義 | ✅ 両アクターで使用（talk）4箇所 | **スクリプト側が誤り** |

### 実装方針

**Option B（スクリプト修正アプローチ）** を採用する。`image_generator.rs` と `actors.pasta` は変更せず、スクリプト内の `＠元気` `＠考え` を既存の辞書定義済み表情名に置換する。

### 影響範囲

- **修正対象**: `crates/pasta_sample_ghost/src/scripts.rs` 内の `BOOT_PASTA` / `TALK_PASTA` / `CLICK_PASTA` 定数
- **変更不要**: `image_generator.rs`（憲法）、`ACTORS_PASTA`（既に正しい）
- **生成物**: `ghosts/hello-pasta/ghost/master/dic/*.pasta`（`generate_ghost()` → `generate_scripts()` で生成、ソース変更で自動反映）
- **テスト**: `crates/pasta_sample_ghost/tests/`（影響は pasta_sample_ghost 内に閉じる）

## Requirements

### Requirement 1: スクリプト表情名と辞書定義の完全一致

**Objective:** As a ゴースト開発者, I want スクリプトで使用される全ての表情名が `image_generator.rs` に対応する `actors.pasta` の辞書定義に存在する状態, so that 未定義の表情名参照による実行時エラーや意図しない表示を防ぐことができる

#### Acceptance Criteria

1. The pasta_sample_ghost shall スクリプト（`boot.pasta`, `talk.pasta`, `click.pasta`）で使用される全ての `＠表情名` が `actors.pasta` のアクター辞書に定義済みである状態とする
2. When スクリプト内で `＠元気` または `＠考え` が使用されている箇所がある場合, the pasta_sample_ghost shall それらを `actors.pasta` に定義済みの既存表情名に置換する
3. The pasta_sample_ghost shall スクリプトで使用される全表情名がアクター辞書に定義済みであることを検証するテストを持つ

### Requirement 2: 置換後の意味的自然さ

**Objective:** As a ゴースト開発者, I want 置換後の表情名がセリフの文脈に対して意味的に自然である状態, so that サンプルゴーストの教育的価値が維持される

#### Acceptance Criteria

1. The pasta_sample_ghost shall `＠元気`（男の子のポジティブ表情）の置換先として、セリフの文脈に最も適した既存表情名を使用する（設計フェーズで決定）
2. The pasta_sample_ghost shall `＠考え`（思索・疑問の文脈）の置換先として、セリフの文脈に最も適した既存表情名を使用する（設計フェーズで決定）

### Requirement 3: ソースと生成物の一貫性

**Objective:** As a ゴースト開発者, I want Rustソース（`scripts.rs`）と生成される `.pasta` ファイルの内容が完全に一致する状態, so that ビルド後の配布物が常に正しい状態になる

#### Acceptance Criteria

1. The pasta_sample_ghost shall `scripts.rs` の定数文字列と `ghosts/hello-pasta/ghost/master/dic/*.pasta` ファイルの内容を一致させる
2. When `cargo test -p pasta_sample_ghost` を実行した場合, the pasta_sample_ghost shall `scripts.rs` 定数と生成される `.pasta` ファイルの内容が一致することをテストで検証する

### Requirement 4: 既存テストとの互換性

**Objective:** As a ゴースト開発者, I want 表情名の修正後も既存のテストが全てパスする状態, so that リグレッションを防止できる

#### Acceptance Criteria

1. When 表情名の変更を行った場合, the pasta_sample_ghost shall `cargo test --all` が全てパスする状態を維持する
2. The pasta_sample_ghost shall 既存の全テスト（`cargo test --all`）が変更後もパスする状態を維持する（※ E2E テストは独自フィクスチャを使用しておりサンプルゴースト非参照のため直接影響なし）
