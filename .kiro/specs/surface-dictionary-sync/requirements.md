# Requirements Document

## Introduction

pasta_sample_ghost（サンプルゴースト「hello-pasta」）において、アクター辞書（`actors.pasta`）で定義された表情名と、イベントスクリプト（`boot.pasta`, `talk.pasta`, `click.pasta`）で実際に使用されている表情名の間に不一致が存在する。辞書に定義されていない表情名がスクリプト中で参照されており、これを解消してサーフェス番号と日本語表情名を完全に一致させる。

### 現状の不一致

| 表情名 | actors.pasta 辞書 | スクリプトでの使用 | 状態 |
|--------|-------------------|-------------------|------|
| `＠元気` | ❌ 未定義 | ✅ 男の子で多用（boot, talk, click） | **不一致** |
| `＠考え` | ❌ 未定義 | ✅ 両アクターで使用（talk） | **不一致** |

### 影響範囲

- **ソース**: `crates/pasta_sample_ghost/src/scripts.rs`（Rust定数として定義）
- **生成物**: `ghosts/hello-pasta/ghost/master/dic/*.pasta`（`generate_ghost()` → `generate_scripts()` で生成）
- **画像生成**: `crates/pasta_sample_ghost/src/image_generator.rs`（`Expression` enum）
- **テスト**: `crates/pasta_sample_ghost/tests/`

## Requirements

### Requirement 1: アクター辞書と使用表情名の完全一致

**Objective:** As a ゴースト開発者, I want アクター辞書に定義された表情名のみがスクリプトで使用される状態, so that 未定義の表情名参照による実行時エラーや意図しない表示を防ぐことができる

#### Acceptance Criteria

1. The pasta_sample_ghost shall `actors.pasta` のアクター辞書に、スクリプト（`boot.pasta`, `talk.pasta`, `click.pasta`）で使用される全ての表情名（`＠表情名`）の定義を持つ
2. When スクリプト内でアクター表情名（`＠表情名`）が参照された場合, the pasta_sample_ghost shall 対応する `％アクター名` のアクター辞書に該当する `＠表情名` エントリを持つ
3. If アクター辞書に未定義の表情名がスクリプトで使用されている場合, then the pasta_sample_ghost shall 辞書への追加またはスクリプト側の表情名修正により不一致を解消する

### Requirement 2: サーフェス番号と表情名の意味的対応

**Objective:** As a ゴースト開発者, I want サーフェス番号と表情名の対応が意味的に自然である状態, so that 画像（ピクトグラム）の見た目と表情名の語感が一致する

#### Acceptance Criteria

1. The pasta_sample_ghost shall 各サーフェス番号に対応する画像（`Expression` enum）と、アクター辞書の日本語表情名を意味的に一致させる
2. The pasta_sample_ghost shall 女の子（sakura, surface 0-8）と男の子（kero, surface 10-18）で同一の表情名セットを使用する
3. When 新しい表情名をアクター辞書に追加する場合, the pasta_sample_ghost shall 対応する `Expression` variant と画像生成ロジック（`image_generator.rs`）を同期させる

### Requirement 3: ソースと生成物の一貫性

**Objective:** As a ゴースト開発者, I want Rustソース（`scripts.rs`）と生成される `.pasta` ファイルの内容が完全に一致する状態, so that ビルド後の配布物が常に正しい状態になる

#### Acceptance Criteria

1. The pasta_sample_ghost shall `scripts.rs` の定数文字列と `ghosts/hello-pasta/ghost/master/dic/*.pasta` ファイルの内容を一致させる
2. When `cargo test -p pasta_sample_ghost` を実行した場合, the pasta_sample_ghost shall `scripts.rs` 定数と生成される `.pasta` ファイルの内容が一致することをテストで検証する
3. The pasta_sample_ghost shall スクリプトで使用される全表情名がアクター辞書に定義済みであることを検証するテストを持つ

### Requirement 4: 既存テストとの互換性

**Objective:** As a ゴースト開発者, I want 表情名の修正後も既存のテストが全てパスする状態, so that リグレッションを防止できる

#### Acceptance Criteria

1. When 表情名の追加・変更を行った場合, the pasta_sample_ghost shall `cargo test --all` が全てパスする状態を維持する
2. The pasta_sample_ghost shall 既存の全テスト（`cargo test --all`）が変更後もパスする状態を維持する（※ E2E テストは独自フィクスチャを使用しておりサンプルゴースト非参照のため直接影響なし）
3. If 表情名の変更がさくらスクリプト出力（`\s[N]`）に影響する場合, then the pasta_sample_ghost shall 期待される出力値を更新する
