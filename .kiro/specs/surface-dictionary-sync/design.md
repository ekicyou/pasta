# Design Document: surface-dictionary-sync

## Overview

**Purpose**: pasta_sample_ghost のイベントスクリプトで使用されている未定義表情名（`＠元気`, `＠考え`）を、`image_generator.rs` が定義する既存の表情名に置換し、サーフェス定義とスクリプトの完全な整合性を実現する。

**Users**: ゴースト開発者がサンプルゴーストを参考にする際、全表情名が辞書定義済みかつサーフェス画像と対応している状態を保証する。

**Impact**: `scripts.rs` 内の 3 定数（`BOOT_PASTA`, `TALK_PASTA`, `CLICK_PASTA`）のテキストを修正する。`image_generator.rs` と `ACTORS_PASTA` は変更しない。

### Goals
- スクリプトで使用される全 `＠表情名` が `actors.pasta` の辞書定義に存在する状態を達成する
- 置換後のセリフと表情名が意味的に自然な対応を維持する
- サンプルゴーストの教育的価値を維持または向上させる（表情バリエーションの拡大）

### Non-Goals
- `Expression` enum の追加・変更（`image_generator.rs` は不変）
- `actors.pasta` の表情セットの変更（既に正しい）
- サーフェス番号体系（0-8, 10-18）の変更
- スクリプトのセリフ文面の変更（表情名のみ置換）

## Architecture

### Existing Architecture Analysis

本機能はデータ修正であり、アーキテクチャの変更は伴わない。既存の Source of Truth 階層を遵守する。

```
image_generator.rs (Expression enum)    ← 憲法・不変
        ↓
actors.pasta (＠表情名：\s[N])          ← 不変（既に正しい）
        ↓
boot/talk/click.pasta                   ← 修正対象
```

**修正対象ファイル**: `crates/pasta_sample_ghost/src/scripts.rs`
- `BOOT_PASTA` 定数: 1箇所
- `TALK_PASTA` 定数: 7箇所
- `CLICK_PASTA` 定数: 3箇所

**影響しないファイル**:
- `image_generator.rs` — 変更なし
- `ACTORS_PASTA` 定数 — 変更なし
- `integration_test.rs` — `＠笑顔`/`＠怒り` の存在チェックのみで変更不要

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Backend | Rust 2024 edition | `scripts.rs` 定数文字列の編集 | 既存 |
| Data | pasta DSL テキスト | `const &str` 内の表情名置換 | 既存 |
| Testing | `cargo test` | リグレッション確認 | 既存 |

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1 | 全 `＠表情名` が辞書定義済み | ScriptConstants | — | — |
| 1.2 | `＠元気`/`＠考え` を既存表情名に置換 | ScriptConstants | — | TextSubstitution |
| 1.3 | 辞書一致検証テスト | ExpressionConsistencyTest | — | — |
| 2.1 | `＠元気` の意味的自然な置換 | ScriptConstants | — | TextSubstitution |
| 2.2 | `＠考え` の意味的自然な置換 | ScriptConstants | — | TextSubstitution |
| 3.1 | `scripts.rs` と `.pasta` の一致 | ScriptConstants | — | GenerationFlow |
| 3.2 | テストで一致検証 | ExpressionConsistencyTest | — | — |
| 4.1 | `cargo test --all` パス | 全体 | — | — |
| 4.2 | 既存テスト互換性維持 | 全体 | — | — |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|------------|--------|--------------|------------------|-----------|
| ScriptConstants | Data | スクリプト定数の表情名修正 | 1.1, 1.2, 2.1, 2.2 | image_generator.rs (P0) | — |
| ExpressionConsistencyTest | Testing | 辞書↔スクリプト一致検証 | 1.3, 3.2 | scripts.rs (P0) | — |
| GeneratedArtifacts | Build | `.pasta` ファイル生成 | 3.1 | scripts.rs (P0) | — |

### Data Layer

#### ScriptConstants

| Field | Detail |
|-------|--------|
| Intent | `scripts.rs` 内のスクリプト定数で未定義表情名を既存表情名に置換する |
| Requirements | 1.1, 1.2, 2.1, 2.2 |

**Responsibilities & Constraints**
- `BOOT_PASTA`, `TALK_PASTA`, `CLICK_PASTA` 内の `＠元気` / `＠考え` を文脈に適した既存表情名に置換する
- `ACTORS_PASTA` は変更しない
- セリフ文面は変更しない（表情名のみ）

**Dependencies**
- Inbound: なし
- Outbound: `generate_scripts()` — `.pasta` ファイル生成 (P0)
- External: なし

**Contracts**: State [x]

##### State Management

置換マッピング（全11箇所）:

**`＠元気` → 文脈依存置換（7箇所）**

| # | 定数 | シーン | セリフ | 置換先 | 根拠 |
|---|------|--------|--------|--------|------|
| 1 | BOOT | OnFirstBoot | `ぼくは男の子。ちゃんと使ってよね。` | `＠笑顔` | 初対面の明るい自己紹介 |
| 2 | TALK | OnTalk | `Lua 側も触ってみなよ。` | `＠笑顔` | 自信を持って推薦する快活さ |
| 3 | TALK | OnTalk | `しょうがないなあ。` | `＠照れ` | ツンデレ的な照れ隠しの承諾 |
| 4 | TALK | OnHour | `もう ＄時１２ か、早いね。` | `＠笑顔` | 時報への明るいリアクション |
| 5 | CLICK | OnMouseDoubleClick | `どうしたの？` | `＠笑顔` | 気さくな声かけ |
| 6 | CLICK | OnMouseDoubleClick | `照れてるの？` | `＠キラキラ` | いたずらっぽいからかい |
| 7 | CLICK | OnMouseDoubleClick | `ふふん、ぼくのことが気になる？` | `＠キラキラ` | 得意気なナルシスト的からかい |

**`＠考え` → 文脈依存置換（4箇所）**

| # | 定数 | シーン | アクター | セリフ | 置換先 | 根拠 |
|---|------|--------|----------|--------|--------|------|
| 8 | TALK | OnTalk | 女の子 | `今日は何しようかな...` | `＠眠い` | ぼんやりした迷い（半目のぼーっと感） |
| 9 | TALK | OnTalk | 男の子 | `さあ、外見てないからわかんないや。` | `＠困惑` | 答えが出ない軽い困り感 |
| 10 | TALK | OnHour | 女の子 | `＄時１２ ...時間が経つのって不思議だね。` | `＠通常` | 穏やかな哲学的内省 |
| 11 | TALK | OnHour | 男の子 | `哲学的だね。` | `＠通常` | 静かな同調コメント |

### Testing Layer

#### ExpressionConsistencyTest

| Field | Detail |
|-------|--------|
| Intent | スクリプトで使用される全 `＠表情名` が `ACTORS_PASTA` に定義済みであることを検証する |
| Requirements | 1.3, 3.2 |

**Responsibilities & Constraints**
- `BOOT_PASTA`, `TALK_PASTA`, `CLICK_PASTA` から `＠` で始まるアクター表情名を抽出する
- 抽出した表情名が全て `ACTORS_PASTA` に存在することを確認する
- グローバル単語辞書定義（`＠終了挨拶` / `＠雑談` 等、シーン定義の外にある `＠`）はアクター表情名ではないため検証対象外とする

**Dependencies**
- Inbound: `scripts.rs` の定数群 (P0)
- Outbound: なし

**Implementation Notes**
- シーン内（`＊` ブロック内）のアクション行で使用される `＠表情名` のみを検証対象とする
- グローバルスコープの `＠単語名：値` 形式（ランダム単語定義）は除外する必要がある
- 判別基準: アクション行内の `アクター名：＠表情名　セリフ` パターンにマッチするもの

## Data Models

本機能はデータモデルの変更を伴わない。`const &str` 定数内のテキスト置換のみ。

## Testing Strategy

### Unit Tests
- `test_actors_pasta_contains_all_characters` — 既存テスト、変更不要
- `test_boot_pasta_contains_events` — 既存テスト、変更不要
- `test_talk_pasta_contains_events` — 既存テスト、変更不要
- `test_click_pasta_contains_events` — 既存テスト、変更不要

### 新規テスト
- `test_script_expression_names_defined_in_actors` — スクリプト内の全 `＠表情名` が `ACTORS_PASTA` に定義済みであることを検証（1.3, 3.2）

### Integration Tests
- `test_pasta_scripts` — 既存テスト、`＠笑顔`/`＠怒り` の存在チェックで変更不要
- `test_expression_variations` — 既存テスト、`Expression::all()` が9種であることを確認、変更不要

### Regression
- `cargo test --all` — 全テストパスを確認（4.1, 4.2）

## 置換後の表情使用状況

| # | `＠表情名` | 置換前使用数 | 置換後使用数 | 変化 |
|---|-----------|-------------|-------------|------|
| 0 | ＠笑顔 | ✅ 多数 | +4 | 増加 |
| 1 | ＠通常 | ✅ 多数 | +2 | 増加 |
| 2 | ＠照れ | ✅ 少数 | +1 | 増加 |
| 3 | ＠驚き | ✅ 少数 | ±0 | 変化なし |
| 4 | ＠泣き | ❌ 未使用 | 0 | 未使用のまま |
| 5 | ＠困惑 | ❌ 未使用 | +1 | **新規使用** |
| 6 | ＠キラキラ | ❌ 未使用 | +2 | **新規使用** |
| 7 | ＠眠い | ✅ 少数 | +1 | 増加 |
| 8 | ＠怒り | ✅ 少数 | ±0 | 変化なし |

置換後は9種中8種が使用状態に。`＠泣き` のみスクリプト未使用だが、`Expression::Crying` に対応するサーフェス画像は存在するため辞書定義として正しい。
