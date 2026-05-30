# Research & Design Decisions

## Summary
- **Feature**: `audit-dependency-supply-chain`
- **Discovery Scope**: Extension（既存ワークスペース依存の横断的監査）
- **Key Findings**:
  - 全7クレートで27の外部直接依存（重複排除後約20種）を使用。workspace.dependenciesで大部分を管理済み
  - pasta_checkの`md5`、`zip`、`lexopt`はworkspace管理外。pasta_lspの`tower-lsp`、WASM依存群も同様
  - Wave 1全specで外部依存サプライチェーン監査は本specに委任済み

## Research Log

### 依存クレート全量調査
- **Context**: ワークスペース全体の依存構造把握
- **Sources Consulted**: 全7クレートのCargo.toml、ルートCargo.toml（workspace.dependencies）
- **Findings**:
  - **pasta_core** (4依存): thiserror, fast_radix_trie, rand, tracing
  - **pasta_dsl** (3依存): pest, pest_derive, thiserror
  - **pasta_lua** (15依存): pasta_core, pasta_dsl, tracing, tracing-appender, tracing-subscriber, thiserror, mlua, mlua-stdlib, toml, serde, serde_json, glob, flate2, regex, budoux, unicode-width + windows-sys (cfg(windows))
  - **pasta_shiori** (7依存): pasta_core, pasta_lua, time, tracing, thiserror, pest, pest_derive + windows-sys (cfg(windows))
  - **pasta_check** (4依存): lexopt, md5, zip, thiserror + pasta_lua
  - **pasta_lsp** (5+4依存): pasta_dsl, thiserror, serde, serde_json, tower-lsp + wasm-bindgen, wasm-bindgen-futures, js-sys, serde-wasm-bindgen (wasm32)
  - **pasta_sample_ghost** (3依存): image, imageproc, thiserror
  - **dev-dependencies** (共通): tempfile, insta, tracing-test + tokio (pasta_lspのみ)
- **Implications**: 外部依存は約20種で中規模。workspace.dependenciesで共通管理されていないものの統合が改善候補

### Workspace管理状況の調査
- **Context**: バージョン固定戦略の網羅性確認
- **Findings**:
  - workspace.dependenciesで管理済み: pest, pest_derive, thiserror, fast_radix_trie, rand, tracing, mlua, mlua-stdlib, regex, toml, serde, serde_json, glob, flate2, budoux, unicode-width, time, windows-sys, tracing-subscriber, tracing-appender, tempfile, insta, tracing-test
  - workspace管理外（個別クレートで直接指定）: lexopt 0.3, md5 0.8, zip 8.6, tower-lsp 0.20, image 0.25, imageproc 0.26, wasm-bindgen 0.2, wasm-bindgen-futures 0.4, js-sys 0.3, serde-wasm-bindgen 0.6, tokio 1
- **Implications**: 10個の依存がworkspace管理外。利用箇所が1クレートのみのものはworkspace統合の必要性は低いが、一貫性のため統合検討の価値あり

### Wave 1監査からの依存関連知見
- **Context**: 各クレート監査で発見された依存関連の指摘
- **Findings**:
  - **audit-pasta-check**: MD5は非暗号学的ファイル変更検出用途（SSP仕様準拠）。用途コメント追記済み
  - **audit-pasta-lua**: mlua vendoredビルド。unsafeはmlua API制約に起因。外部依存バージョン変更は本specに委任
  - **audit-pasta-shiori**: 外部依存クレートのサプライチェーン監査は本specに委任
  - **audit-pasta-lsp**: 外部依存クレートのサプライチェーン監査は本specに委任
  - **audit-pasta-core**: 新規外部依存の追加は禁止
  - **audit-pasta-dsl**: 新しい外部依存の追加は不可
  - **audit-pasta-sample-ghost**: image/imageproc依存クレートの更新は本specの範囲外だが監査は対象
- **Implications**: Wave 1全specが外部依存のサプライチェーン監査を本specに委任している。MD5用途適切性は既に文書化済みだが、本specで統合的に再確認する

### 監査ツール調査
- **Context**: Rustエコシステムにおける依存監査ツール
- **Findings**:
  - **cargo-audit**: RustSec Advisory DBに基づく脆弱性検査。`cargo install cargo-audit`で導入
  - **cargo-deny**: ライセンス監査、ban対象クレートの管理、重複依存検出。deny.tomlで設定
  - **cargo-tree**: 依存ツリーの可視化。重複依存やfeature分析に有用。Rust標準ツール
  - **cargo-udeps**: 未使用依存の検出。nightlyが必要な場合あり
  - **cargo-outdated**: 更新可能な依存の一覧表示
- **Implications**: cargo-audit + cargo-deny + cargo-tree で主要な監査はカバー可能。cargo-udepsはnightly制約あり、手動分析で代替可能

### プロジェクトライセンス互換性
- **Context**: MIT OR Apache-2.0ライセンスとの互換性基準
- **Findings**:
  - MIT: 非常に寛容、ほぼ全てのライセンスと互換
  - Apache-2.0: GPLv2との非互換以外は寛容
  - 警戒対象: GPL, LGPL（静的リンク時）, AGPL, SSPL, BUSL
  - Rust依存の大多数はMIT/Apache-2.0/BSD系
  - mlua vendored（LuaJIT）: MIT License — 互換
- **Implications**: GPL汚染の可能性は低いが、体系的に確認が必要

## Design Decisions

### Decision: 監査レポートの形式
- **Context**: 監査結果をどの形式で文書化するか
- **Alternatives Considered**:
  1. Markdown形式のレポートファイル（audit-report.md）
  2. cargo-deny設定ファイル（deny.toml）+ CI統合
  3. 既存のresearch.mdへの追記
- **Selected Approach**: deny.toml設定ファイル + Markdownレポート（research.mdに統合）
- **Rationale**: deny.tomlは再現可能な機械的チェックを提供し、将来のCI統合の基盤となる。レポートはresearch.mdに監査結果として記録
- **Trade-offs**: deny.tomlの初期設定コストはあるが、将来の反復監査で回収できる

### Decision: Workspace依存の統合方針
- **Context**: workspace管理外の依存をどこまで統合するか
- **Alternatives Considered**:
  1. 全依存をworkspace.dependenciesに統合
  2. 複数クレートで使用されるもののみ統合
  3. 現状維持
- **Selected Approach**: 1クレートのみで使用される依存もworkspace.dependenciesに統合（一貫性優先）
- **Rationale**: バージョン管理の一元化により、将来のアップデート作業を効率化。ワークスペースが小規模なため管理負荷は低い
- **Trade-offs**: ルートCargo.tomlの行数増加だが、管理の一貫性が上回る

## Risks & Mitigations
- cargo-auditで既知脆弱性が検出された場合 → 深刻度に応じてパッチバージョンアップまたはワークアラウンド
- ライセンス非互換が発見された場合 → 代替クレートへの移行検討（本spec範囲で対応可能な範囲）
- 依存除去でビルド破壊 → 全テスト回帰確認を必須とする
- cargo-udepsがnightlyを要求 → cargo-treeの手動分析で代替

## References
- [RustSec Advisory Database](https://rustsec.org/) — Rust脆弱性データベース
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) — 脆弱性スキャナ
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) — ライセンス・ban・重複チェック
- [SPDX License List](https://spdx.org/licenses/) — ライセンス識別子一覧

## Vulnerability Scan Results

### 2026-05-30 cargo-audit実行結果
- **対応要件**: 1.1, 1.2, 1.3, 1.4, 6.1, 6.2
- **実行日時**: 2026-05-30T08:11:33.1324607+09:00
- **使用ツール**: `cargo-audit-audit 0.22.1`
- **データソース**: RustSec Advisory DB (`last-updated`: `2026-05-29T20:55:26+02:00`, `last-commit`: `eaf48e749baa3d5e27d304107d8abf175fd756bb`, `advisory-count`: 1099)
- **監査対象範囲**: ワークスペースルート `C:\home\maz\git\pasta` の `Cargo.lock` を対象に監査を実行。全7クレートの直接依存（Cargo.toml）および間接依存（Cargo.lock経由）を含む351依存を走査
- **実行コマンド**: `cargo audit --json`
- **終了コード**: `0`

#### 脆弱性（Requirement 1）
- **結果**: クリーン。`vulnerabilities.found = false`、既知脆弱性 advisory は **0件**
- **記録対象**: Requirement 1.3 に基づき、脆弱性未検出のためクリーンステータスを明示

#### Informational Warnings（参考）
- **件数**: 1件
- **ID**: `RUSTSEC-2024-0436`
- **深刻度**: informational / unmaintained
- **影響パッケージ**: `paste 1.0.15`
- **影響範囲**: `pasta_sample_ghost` の間接依存チェーン（`pasta_sample_ghost` → `imageproc`/`image` → `ravif`/`nalgebra` → `paste`）
- **内容**: `paste` クレートはメンテナンス終了としてRustSecに登録されている
- **推奨対処**: 直ちに脆弱性対応は不要だが、`image` / `imageproc` 系の更新時に `paste` 依存の解消有無を確認し、必要に応じて upstream 側の更新または代替（例: `pastey`）を追跡する

#### 総括
- Requirement 1.1 / 1.4 を満たす形で、Cargo.lockベースの全直接・間接依存監査を実行済み
- Requirement 1.2 は「既知脆弱性が検出された場合」に備える記録項目だが、今回は脆弱性0件のため該当なし
- Requirement 6.1 / 6.2 を満たす形で、実行日時・ツールバージョン・監査範囲・構造化結果を本書に記録済み

## License Audit Results

### 2026-05-30 cargo-deny実行結果
- **対応要件**: 2.1, 2.2, 2.3, 2.4, 6.2
- **実行日時**: 2026-05-30T08:39:33.6233350+09:00
- **使用ツール**: `cargo-deny 0.19.8`
- **監査対象範囲**: `C:\home\maz\git\pasta` の `Cargo.lock` を対象にライセンス監査を実行。`cargo deny list -f json -l crate` で確認できた外部依存は 291 クレートで、workspace の path 依存 7 クレートは下記在庫から除外した。
- **実行コマンド**:
  - `cargo deny check licenses` → 終了コード `0`
  - `cargo deny check licenses --format json` → 終了コード `2`（`cargo-deny 0.19.8` では `--format` が未対応だったため、代替として `cargo deny -f json check licenses` を使用）
  - `cargo deny -f json check licenses` → 終了コード `0`
  - `cargo deny list -f json -l crate` → 終了コード `0`
- **JSON要約**: errors=`0`, warnings=`1`, notes=`0`, helps=`298`

#### ライセンス監査結果（Requirement 2）
- **互換性判定**: **PASS**。`cargo deny check licenses` は終了コード 0 で完了し、deny.toml ポリシーに対する非互換ライセンスのエラーは検出されなかった。
- **非互換ライセンス記録**: 該当なし。クレート名とライセンス種別の記録が必要な拒否項目は 0 件。
- **警告**: `license-not-encountered`（warning）。deny.toml の `ISC` 許可設定が今回の依存グラフでは未使用であることを示す設定警告であり、依存クレート自体の不整合ではない。
- **補足**: `LGPL-2.1-or-later` と `0BSD` は SPDX 識別子として観測されたが、実際には `r-efi 5.3.0/6.0.0 = MIT OR Apache-2.0 OR LGPL-2.1-or-later`、`adler2 2.0.1 = 0BSD OR MIT OR Apache-2.0` のような複数選択肢ライセンスの一部として現れており、cargo-deny は許可済みの `MIT` / `Apache-2.0` 選択肢を含むためエラーにしていない。

#### Vendored ソース確認（Requirement 2.4）
- ルート `Cargo.toml` では `mlua = { version = "0.11", features = ["luajit52", "vendored", "serialize"] }` を確認し、`mlua` が vendored LuaJIT を使う構成であることを確認した。
- `Cargo.lock` では `mlua 0.11.6` が `mlua-sys 0.10.0` に依存し、`mlua-sys 0.10.0` が `luajit-src 210.6.6+707c12b` を build dependency に含むことを確認した。
- `C:\Users\maz\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\mlua-sys-0.10.0\Cargo.toml` では `license = "MIT"` と `vendored = ["lua-src", "luajit-src"]` を確認した。
- `C:\Users\maz\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\luajit-src-210.6.6+707c12b\Cargo.toml` では `license = "MIT"`、同梱 `luajit2\COPYRIGHT` では `[ MIT license: https://www.opensource.org/licenses/mit-license.php ]` を確認した。
- **vendored 判定**: `mlua` が同梱する LuaJIT ソースは **MIT License** であり、プロジェクトライセンス（MIT OR Apache-2.0）と互換であることを明示確認した。

#### ライセンス在庫（外部依存のみ）
- 集計方法: `cargo deny list -f json -l crate` の結果を SPDX 識別子ごとに再集計した。複数ライセンスのクレートは各識別子に重複して現れるため、下記件数の合計は外部依存クレート数 291 を上回る。
- **観測された SPDX 識別子**: `0BSD`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `BSL-1.0`, `CC0-1.0`, `LGPL-2.1-or-later`, `MIT`, `NCSA`, `Unicode-3.0`, `Unlicense`, `Zlib`
- **識別子別件数**:
  - `0BSD`: 1 crate
  - `Apache-2.0`: 202 crates
  - `Apache-2.0 WITH LLVM-exception`: 5 crates
  - `BSD-2-Clause`: 5 crates
  - `BSD-3-Clause`: 6 crates
  - `BSL-1.0`: 1 crate
  - `CC0-1.0`: 1 crate
  - `LGPL-2.1-or-later`: 2 crates
  - `MIT`: 253 crates
  - `NCSA`: 1 crate
  - `Unicode-3.0`: 19 crates
  - `Unlicense`: 3 crates
  - `Zlib`: 8 crates

#### 総括
- Requirement 2.1 / 2.3 を満たす形で、cargo-deny によるライセンス監査を実行し、依存グラフ上で使用中のライセンス識別子を research.md に記録した。
- Requirement 2.2 については、非互換ライセンス検出時に記録すべき対象は今回は 0 件であり、その旨を明示した。
- Requirement 2.4 を満たす形で、`mlua` の vendored LuaJIT ソースが MIT License であることを crate metadata と同梱 `COPYRIGHT` の両方で確認した。
- Requirement 6.2 を満たす形で、実行コマンド、結果、警告、ライセンス在庫、互換性 verdict を本書に追記した。

## Dependency Tree & Unused Dependency Analysis

### 2026-05-30 cargo-tree 実行結果
- **対応要件**: 3.1, 3.2, 6.2
- **実行日時**: 2026-05-30T09:05:14.9239147+09:00
- **使用ツール**: `cargo 1.96.0 (30a34c682 2026-05-25)` の組み込み `cargo tree`
- **監査対象範囲**: ワークスペース `C:\home\maz\git\pasta` の全 7 クレート（`pasta_core`, `pasta_dsl`, `pasta_lua`, `pasta_shiori`, `pasta_check`, `pasta_lsp`, `pasta_sample_ghost`）。`Cargo.toml` の `[dependencies]`, `[dev-dependencies]`, `[target.*.dependencies]` を対象に、`src/**/*.rs`, `tests/**/*.rs`, `build.rs` を照合した。
- **実行コマンド**:
  - `cargo tree --workspace`
  - `cargo tree --workspace --duplicates`
- **照合方法**:
  - 通常依存は `use <crate>`, `<crate>::`, `extern crate <crate>` を検索
  - proc-macro / derive 系は `#[derive(...)]` も確認（`pest_derive`, `thiserror`, `serde`）
  - target 条件付き依存は `cfg(windows)` / `cfg(target_arch = "wasm32")` 配下のコードまで確認
  - dev-dependencies は `tests/**/*.rs` に加えて `#[cfg(test)]` を含む `src/**/*.rs` も確認

#### 依存ツリー要約
- `cargo tree --workspace` は全 7 クレートの依存木を正常に可視化した。主要な深いサブツリーは、`pasta_check -> pasta_lua -> mlua/mlua-stdlib/tracing-*`、`pasta_lsp -> tower-lsp -> lsp-types`、`pasta_sample_ghost -> image/imageproc -> ravif/rand 0.9` だった。
- 直接依存・target 依存・dev-dependency の宣言は合計 **58 件**。このうち **49 件**は現行コードから使用を確認し、**9 件**はコード参照が見つからない未使用候補だった。
- `cargo tree` 出力では Git 依存や独自レジストリ依存は観測されず、外部依存は crates.io 系と workspace path 依存で構成されていた。

#### 重複依存 (`cargo tree --workspace --duplicates`)
- 重複エントリは **17 エントリ / 8 ファミリ** を確認した。

| ファミリ | 観測バージョン | 主な導入元 | 所見 |
|---|---|---|---|
| `bitflags` | `1.3.2`, `2.11.1` | `tower-lsp -> lsp-types`, `image -> png` | `pasta_lsp` 系と `pasta_sample_ghost` 系で別系統。直接依存の重複指定ではない |
| `cpufeatures` | `0.2.17`, `0.3.0` | `pest_derive -> pest_meta -> sha2`, `rand 0.10` | proc-macro build dependency と `rand` 系で分離 |
| `getrandom` | `0.3.4`, `0.4.2` | `quick_cache`, `rand 0.9`, `rand 0.10`, `tempfile` | `imageproc` 側の `rand 0.9` と workspace 側 `rand 0.10` の差異が原因 |
| `hashbrown` | `0.14.5`, `0.16.1`, `0.17.1` | `dashmap`, `quick_cache`, `indexmap` | `tower-lsp`, `mlua-stdlib`, `zip`/`serde_yaml` 系の transitive duplicate |
| `rand` | `0.9.4`, `0.10.1` | `imageproc`, `pasta_core` | `pasta_core` の直接依存は `0.10`、`imageproc` 側が `0.9` を維持 |
| `rand_core` | `0.9.5`, `0.10.1` | `rand 0.9`, `rand 0.10` | `rand` の二系統に追随 |
| `thiserror` | `1.0.69`, `2.0.18` | `tower-lsp -> async-codec-lite`, workspace 直接依存群 | workspace 側は `2.x` に統一済みだが upstream が `1.x` を保持 |
| `thiserror-impl` | `1.0.69`, `2.0.18` | `thiserror 1.x`, `thiserror 2.x` | `thiserror` の二系統に付随 |

- **総評**: duplicate は主に upstream 由来の transitive dependency であり、workspace 自身の直接依存指定ミスによるものは確認できなかった。特に `tower-lsp` 系と `image/imageproc` 系が duplicate の主因だった。

#### 直接依存・dev-dependency 照合結果

##### `pasta_core`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `thiserror` | 使用中 | `src/error.rs:7` で `use thiserror::Error;` |
| `dependencies` | `fast_radix_trie` | 使用中 | `src/registry/scene_table.rs:9` で `use fast_radix_trie::RadixMap;` |
| `dependencies` | `rand` | 使用中 | `src/registry/random.rs:6` で `use rand::prelude::*;` |
| `dependencies` | `tracing` | **未使用候補** | `crates/pasta_core` 配下の `src/**/*.rs`, `tests/**/*.rs`, `build.rs` を走査したが `tracing::`, `use tracing`, `debug!` などの参照なし |

##### `pasta_dsl`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `pest` | 使用中 | `src/parser/mod.rs:59` で `use pest::Parser as PestParser;` |
| `dependencies` | `pest_derive` | 使用中 | `src/parser/mod.rs:61` で `use pest_derive::Parser;`、`src/parser/mod.rs:70` で `#[derive(Parser)]` |
| `dependencies` | `thiserror` | 使用中 | `src/error.rs:7` で `use thiserror::Error;` |

##### `pasta_lua`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `pasta_core` | 使用中 | `src/context.rs:5` で `use pasta_core::registry::{SceneRegistry, WordDefRegistry};` |
| `dependencies` | `pasta_dsl` | 使用中 | `src/context.rs:6` で `use pasta_dsl::parser::{...};` |
| `dependencies` | `tracing` | 使用中 | `src/loader/cache.rs:11` で `use tracing::{debug, info, warn};` |
| `dependencies` | `tracing-appender` | 使用中 | `src/logging/logger.rs:8` で `use tracing_appender::non_blocking::{...};` |
| `dependencies` | `tracing-subscriber` | 使用中 | `src/logging/tracing_init.rs:8` で `use tracing_subscriber::filter::EnvFilter;` |
| `dependencies` | `thiserror` | 使用中 | `src/error.rs:7` で `use thiserror::Error;` |
| `dependencies` | `mlua` | 使用中 | `src/lib.rs:65` で `pub use mlua;`、`src/loader/error.rs:49` で `mlua::Error` |
| `dependencies` | `mlua-stdlib` | 使用中 | `src/runtime/mod.rs:119` で `mlua_stdlib::assertions::register(...)` |
| `dependencies` | `toml` | 使用中 | `src/loader/config.rs:22` で `toml::Table` |
| `dependencies` | `serde` | 使用中 | `src/loader/config.rs:6` で `use serde::Deserialize;` |
| `dependencies` | `serde_json` | 使用中 | `src/runtime/log.rs:120` で `serde_json::Value` |
| `dependencies` | `glob` | 使用中 | `src/loader/discovery.rs:5` で `use glob::glob;` |
| `dependencies` | `flate2` | 使用中 | `src/runtime/persistence.rs:25` で `use flate2::Compression;` |
| `dependencies` | `regex` | 使用中 | `src/sakura_script/line_breaker.rs:6` で `use regex::Regex;` |
| `dependencies` | `budoux` | 使用中 | `src/sakura_script/line_breaker.rs:105` で `budoux::Model` |
| `dependencies` | `unicode-width` | 使用中 | `src/sakura_script/line_breaker.rs:7` で `use unicode_width::UnicodeWidthStr;` |
| `target.'cfg(windows)'.dependencies` | `windows-sys` | 使用中 | `src/encoding/windows.rs:13` で `use windows_sys::Win32::Globalization::*;` |
| `dev-dependencies` | `tempfile` | 使用中 | `src/loader/discovery.rs:97` のテストで `use tempfile::TempDir;` |
| `dev-dependencies` | `insta` | 使用中 | `tests/transpiler/snapshot_test.rs:9` で `use insta::assert_snapshot;` |
| `dev-dependencies` | `tracing-test` | 使用中 | `tests/log/integration_test.rs:8` で `use tracing_test::traced_test;` |

##### `pasta_shiori`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `pasta_core` | **未使用候補** | `crates/pasta_shiori` 配下の `src/**/*.rs`, `tests/**/*.rs`, `build.rs` を走査したが `pasta_core::` 参照なし |
| `dependencies` | `pasta_lua` | 使用中 | `src/error.rs:46` で `impl From<pasta_lua::LoaderError> for MyError` |
| `dependencies` | `time` | 使用中 | `src/lua_request.rs:7` で `use time::OffsetDateTime;` |
| `dependencies` | `tracing` | 使用中 | `src/shiori.rs:6` で `use tracing::{debug, error, info, trace, warn};` |
| `dependencies` | `thiserror` | 使用中 | `src/error.rs:4` で `use thiserror::Error;` |
| `dependencies` | `pest` | 使用中 | `src/lua_request.rs:5` で `use pest::Parser as _;` |
| `dependencies` | `pest_derive` | 使用中 | `src/util/parsers/req_parser.rs:9` で `#[derive(Parser)]` |
| `target.'cfg(windows)'.dependencies` | `windows-sys` | 使用中 | `src/windows.rs:11` で `use windows_sys::Win32::Foundation::*;` |
| `dev-dependencies` | `tempfile` | 使用中 | `src/shiori_tests.rs:3` で `use tempfile::TempDir;` |

##### `pasta_check`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `lexopt` | 使用中 | `src/main.rs:34` で `lexopt::Parser::from_env()` |
| `dependencies` | `md5` | 使用中 | `src/update_files.rs:177` で `md5::Context::new()` |
| `dependencies` | `zip` | 使用中 | `src/nar.rs:4` で `use zip::write::SimpleFileOptions;` |
| `dependencies` | `thiserror` | **未使用候補** | `crates/pasta_check` 配下に `thiserror::Error` / `#[derive(Error)]` の参照なし。実装上のエラー処理は `io::Result` と `lexopt::Error` で完結 |
| `dependencies` | `pasta_lua` | **未使用候補** | `crates/pasta_check` 配下の `src/**/*.rs`, `tests/**/*.rs`, `build.rs` を走査したが `pasta_lua::` 参照なし。`Cargo.toml` コメントどおり「将来拡張」用の先行宣言に留まる |
| `dev-dependencies` | `tempfile` | 使用中 | `src/copy.rs:67` のテストで `use tempfile::TempDir;` |

##### `pasta_lsp`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `pasta_dsl` | 使用中 | `src/analysis/mod.rs:55` で `pasta_dsl::parse_str(source, "<lsp>")` |
| `dependencies` | `thiserror` | 使用中 | `src/error.rs:7` で `#[derive(Debug, thiserror::Error)]` |
| `dependencies` | `serde` | 使用中 | `src/transport.rs:8` で `use serde::Serialize;` |
| `dependencies` | `serde_json` | 使用中 | `src/transport.rs:282` で `serde_json::to_string(&wasm_result)` |
| `dependencies` | `tower-lsp` | 使用中 | `src/server.rs:8` で `use tower_lsp::jsonrpc::Result;` |
| `target.'cfg(target_arch = "wasm32")'.dependencies` | `wasm-bindgen` | 使用中 | `src/transport.rs:111` で `use wasm_bindgen::prelude::*;`、`src/transport.rs:125` で `#[wasm_bindgen]` |
| `target.'cfg(target_arch = "wasm32")'.dependencies` | `wasm-bindgen-futures` | **未使用候補** | `src/transport.rs` の wasm モジュールは `wasm_bindgen` と `serde_wasm_bindgen` のみ使用。workspace 走査で `wasm_bindgen_futures::` / `future_to_promise` 参照なし |
| `target.'cfg(target_arch = "wasm32")'.dependencies` | `js-sys` | **未使用候補** | `src/transport.rs` の wasm モジュールに `js_sys::` 参照なし |
| `target.'cfg(target_arch = "wasm32")'.dependencies` | `serde-wasm-bindgen` | 使用中 | `src/transport.rs:145` で `serde_wasm_bindgen::to_value(&analysis_result)` |
| `dev-dependencies` | `tokio` | **未使用候補** | `crates/pasta_lsp` 配下の `src/**/*.rs`, `tests/**/*.rs`, `build.rs` を走査したが `tokio::` / `#[tokio::test]` 参照なし |

##### `pasta_sample_ghost`
| 宣言セクション | 依存 | 判定 | 根拠 |
|---|---|---|---|
| `dependencies` | `image` | 使用中 | `src/image_generator.rs:7` で `use image::{Rgba, RgbaImage};` |
| `dependencies` | `imageproc` | 使用中 | `src/image_generator.rs:8` で `use imageproc::drawing::draw_filled_circle_mut;` |
| `dependencies` | `thiserror` | 使用中 | `src/lib.rs:13` で `use thiserror::Error;` |
| `dev-dependencies` | `pasta_shiori` | **未使用候補（要意図確認）** | Rust コード上の `pasta_shiori::` 参照は見つからない。`build.rs:16-25` は `crates/pasta_shiori/src` を監視するだけで、`tests/common/mod.rs:23-35` も `target/.../pasta_shiori.dll` をファイルパスでコピーしている |
| `dev-dependencies` | `pasta_lua` | **未使用候補（要意図確認）** | Rust コード上の `pasta_lua::` 参照は見つからない。`tests/common/mod.rs:57-73` は `crates/pasta_lua/pasta_scripts` をファイルパスでコピーしているだけ |
| `dev-dependencies` | `tempfile` | 使用中 | `tests/integration_test.rs:7` で `use tempfile::TempDir;` |

#### 未使用依存候補一覧（Task 3.2 向け）
| クレート | 宣言場所 | 依存 | 判定根拠 | 推奨 |
|---|---|---|---|---|
| `pasta_core` | `crates/pasta_core/Cargo.toml` `[dependencies]` | `tracing` | ソース・テスト・build script に `tracing` 系参照なし | Task 3.2 で除去候補 |
| `pasta_shiori` | `crates/pasta_shiori/Cargo.toml` `[dependencies]` | `pasta_core` | `pasta_core::` 参照なし。`pasta_lua` 経由で必要型を取得しているわけでもない | Task 3.2 で除去候補 |
| `pasta_check` | `crates/pasta_check/Cargo.toml` `[dependencies]` | `thiserror` | `thiserror::Error` / `#[derive(Error)]` 不使用。既存コードは `io::Result` / `lexopt::Error` のみ | Task 3.2 で除去候補 |
| `pasta_check` | `crates/pasta_check/Cargo.toml` `[dependencies]` | `pasta_lua` | `pasta_lua::` 参照なし。`Cargo.toml` コメントどおり将来拡張の先行宣言 | Task 3.2 で除去候補 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[target.'cfg(target_arch = "wasm32")'.dependencies]` | `wasm-bindgen-futures` | wasm モジュールで `future_to_promise` / `wasm_bindgen_futures::` 不使用 | Task 3.2 で除去候補 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[target.'cfg(target_arch = "wasm32")'.dependencies]` | `js-sys` | wasm モジュールで `js_sys::` 不使用 | Task 3.2 で除去候補 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[dev-dependencies]` | `tokio` | `tokio::` / `#[tokio::test]` 参照なし | Task 3.2 で除去候補 |
| `pasta_sample_ghost` | `crates/pasta_sample_ghost/Cargo.toml` `[dev-dependencies]` | `pasta_shiori` | crate API 参照なし。テストはビルド済み DLL をファイルパスで扱うのみ | **意図確認後** に Task 3.2 で除去候補 |
| `pasta_sample_ghost` | `crates/pasta_sample_ghost/Cargo.toml` `[dev-dependencies]` | `pasta_lua` | crate API 参照なし。テストは `pasta_scripts/` をファイルパスでコピーするのみ | **意図確認後** に Task 3.2 で除去候補 |

#### 総括
- Requirement 3.1 を満たす形で、`cargo tree` と `cargo tree --duplicates` を実行し、依存木と duplicate を可視化した。
- Requirement 3.2 の前段として、各クレートの直接依存・target 依存・dev-dependency をコード実参照と突き合わせ、**9 件の未使用候補**を抽出した。
- Requirement 6.2 を満たす形で、依存ツリー要約、duplicate 一覧、依存ごとの使用/未使用判定、Task 3.2 向け推奨アクションを research.md に構造化して記録した。

### 2026-05-30 Task 3.2 実施結果
- **対応要件**: 3.2, 3.3, 3.4, 6.2, 7.1, 7.2
- **実行日時**: 2026-05-30T09:38:39.4233519+09:00
- **実行コマンド**:
  - 事前確認: `cargo build --workspace`, `cargo test --workspace`
  - Phase 1: 安全候補 6 件を除去後に `cargo build --workspace`, `cargo test --workspace`
  - Phase 2: `pasta_shiori` の `pasta_core` を除去後に `cargo build --workspace`, `cargo test --workspace`
  - Phase 3: `pasta_sample_ghost` の `pasta_shiori`, `pasta_lua` dev-dependencies を除去後に `cargo build --workspace`, `cargo test --workspace`
  - 補助確認: `cargo test --workspace -- --list`
- **回帰結果**:
  - 事前確認の `cargo build --workspace` は終了コード `0`
  - 事前確認の `cargo test --workspace` は終了コード `0`
  - 最終状態の `cargo build --workspace` は終了コード `0`（workspace 全クレートのコンパイル成功）
  - 最終状態の `cargo test --workspace` は終了コード `0`（失敗 0 件）
  - `cargo test --workspace -- --list` で **1224 件**のテストを列挙し、Requirement 3.4 / 7.2 の「950+件」期待値を上回ることを確認
  - テスト実行中に既存の `mlua::Value::as_str` 非推奨警告は継続して出力されたが、新規 failure は発生しなかった

#### 除去した依存一覧
| クレート | 宣言場所 | 除去した依存 | 判定 |
|---|---|---|---|
| `pasta_core` | `crates/pasta_core/Cargo.toml` `[dependencies]` | `tracing` | 除去完了 |
| `pasta_check` | `crates/pasta_check/Cargo.toml` `[dependencies]` | `thiserror` | 除去完了 |
| `pasta_check` | `crates/pasta_check/Cargo.toml` `[dependencies]` | `pasta_lua` | 除去完了 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[target.'cfg(target_arch = "wasm32")'.dependencies]` | `wasm-bindgen-futures` | 除去完了 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[target.'cfg(target_arch = "wasm32")'.dependencies]` | `js-sys` | 除去完了 |
| `pasta_lsp` | `crates/pasta_lsp/Cargo.toml` `[dev-dependencies]` | `tokio` | 除去完了 |
| `pasta_shiori` | `crates/pasta_shiori/Cargo.toml` `[dependencies]` | `pasta_core` | 除去完了 |
| `pasta_sample_ghost` | `crates/pasta_sample_ghost/Cargo.toml` `[dev-dependencies]` | `pasta_shiori` | 除去完了 |
| `pasta_sample_ghost` | `crates/pasta_sample_ghost/Cargo.toml` `[dev-dependencies]` | `pasta_lua` | 除去完了 |

#### 補足
- `pasta_sample_ghost` の 2 件は build-order 用の暫定依存の可能性を考慮して最後に検証したが、workspace 全体の `cargo build` / `cargo test` はどちらも成功したため、Task 3.2 の回帰条件下では未使用依存として除去可能と判断した。
- 今回の除去では `Cargo.lock` の追加更新は発生しなかった。
- Task 3.2 では候補 9 件すべてが回帰なしで除去でき、個別の revert は不要だった.

## Dependency Update Results

### 2026-05-30 Task 4.2 実施結果
- **対応要件**: 4.3, 4.4, 6.2, 7.1, 7.2
- **実行日時**: 2026-05-30T10:36:35.3507210+09:00
- **実行コマンド**:
  - 事前確認: `cargo build --workspace`, `cargo test --workspace`
  - 更新候補確認: `cargo update --dry-run`
  - 適用: `cargo update`
  - 回帰確認: `cargo build --workspace`, `cargo test --workspace`
- **dry-run 結果**:
  - `typenum` `1.20.0 -> 1.20.1`
  - `zerocopy` `0.8.49 -> 0.8.50`
  - `zerocopy-derive` `0.8.49 -> 0.8.50`
  - Cargo は「latest まで未到達の依存が 3 件ある」と補足したが、いずれも今回の version requirement 外であり、Task 4.2 の更新対象ではない

#### 変更履歴確認
- `typenum 1.20.1` の upstream `CHANGELOG.md` では、変更点は `tarr` import resolution の修正 1 件のみだった。公開 API の追加・削除や major/minor bump はなく、パッチ更新として安全と判断した。
- `zerocopy 0.8.50` のパッケージ同梱 `CHANGELOG.md` は GitHub Releases 参照のみだったため、公開 crate の VCS commit (`5fc5d5b -> f70e422`) を比較した。差分は 1 commit のみで、内容は ``[pointer] `Ptr::iter` takes `self` by value`` による soundness fix（`Release 0.8.50`）だった。
- 上記 zerocopy の修正は commit message 上で unstable pointer API の semver check 除外が明示されており、workspace からは `zerocopy` / `zerocopy-derive` を直接利用していない（`mlua-stdlib` / `image` / `imageproc` 系の間接依存のみ）ため、実運用上の破壊的変更リスクは低いと判断した。
- `zerocopy-derive 0.8.50` は `zerocopy` と同一リリースで同期更新されており、別個の破壊的変更記録は確認されなかった。

#### 適用した更新一覧
| 依存 | 更新前 | 更新後 | 導入経路の要約 | 判定 |
|---|---|---|---|---|
| `typenum` | `1.20.0` | `1.20.1` | `pest_derive -> pest_generator -> pest_meta -> sha2` の build 依存、および `imageproc -> nalgebra` 経由 | 適用 |
| `zerocopy` | `0.8.49` | `0.8.50` | `mlua-stdlib -> quick_cache -> ahash` と `image` / `imageproc` 系の間接依存 | 適用 |
| `zerocopy-derive` | `0.8.49` | `0.8.50` | `zerocopy` の proc-macro 依存 | 適用 |

#### 回帰確認結果
- 事前確認の `cargo build --workspace` は終了コード `0`
- 事前確認の `cargo test --workspace` は終了コード `0`
- 更新後の `cargo build --workspace` は終了コード `0`
- 更新後の `cargo test --workspace` は終了コード `0`
- 更新後ビルドでは `typenum 1.20.1`, `zerocopy 0.8.50`, `zerocopy-derive 0.8.50` の再コンパイルを確認し、workspace 全体が正常に完了した

#### 補足
- ルート `.gitignore` に `Cargo.lock` が含まれるため Git の `status` には表示されないが、実ファイル `Cargo.lock` には上記バージョン更新が反映されている。
- 既存の作業ツリー変更として `crates/pasta_lua/tests/fixtures/sample.generated.lua` が事前から変更済みだったため、本タスクでは触れていない。

## MD5 Usage Assessment

### 2026-05-30 MD5クレート用途の統合監査
- **対応要件**: 5.1, 5.2, 5.3, 6.4
- **設計参照**: `design.md` Security Considerations > MD5用途の安全性
- **監査対象範囲**: ワークスペース全体の `Cargo.toml`, `crates/**/Cargo.toml`, `crates/**/*.rs`, `deny.toml`, Wave 1 `completed/audit-pasta-check/research.md`
- **調査方法**:
  - `grep -Rni "md5"` 相当でワークスペース内の宣言・実装・テスト参照を抽出
  - `crates/pasta_check/src/update_files.rs` の実装を確認し、`updates.txt` 生成フローにおける MD5 利用箇所を追跡
  - `cargo metadata --format-version 1 --locked` で `md5 0.8.0` のライセンスが `Apache-2.0/MIT` であることを確認
  - `deny.toml` の `[licenses].allow` に `MIT` / `Apache-2.0` が含まれることを確認
  - Wave 1 `audit-pasta-check` の既存評価と照合
- **要約 verdict**: **適切**。MD5 の実装使用は `pasta_check` の `updates.txt` 生成に限定され、SSP 仕様準拠のファイル変更検出用途のみである。暗号学的用途は確認されず、移行は不要。

#### 使用箇所一覧
| ファイル | 行 | 区分 | 内容 | 評価 |
|---|---|---|---|---|
| `Cargo.toml` | `45` | workspace依存宣言 | `md5 = "0.8"` を workspace 管理 | 宣言のみ |
| `crates/pasta_check/Cargo.toml` | `23` | crate依存宣言 | `md5.workspace = true` | 宣言のみ |
| `crates/pasta_check/src/update_files.rs` | `21-22` | データ保持 | `FileEntry.md5` に `updates.txt` 出力用ハッシュを保持 | 非暗号学的 |
| `crates/pasta_check/src/update_files.rs` | `160-163` | 呼び出し | `collect_files_recursive` が各ファイルに対して `calculate_md5(&path)` を実行 | 非暗号学的 |
| `crates/pasta_check/src/update_files.rs` | `172-189` | 実装本体 | `md5::Context::new()` でファイル内容の MD5 を計算。コメントでも「SSP 仕様準拠の非暗号学的ファイル変更検出用途」と明記 | 非暗号学的 |
| `crates/pasta_check/src/update_files.rs` | `205-208` | 出力 | `file,<path>\x01<md5>\x01size=...` 形式で `updates.txt` に MD5 を書き出し | SSP仕様準拠 |
| `crates/pasta_check/src/update_files.rs` | `221-227` | 単体テスト | `test_calculate_md5` が既知文字列の MD5 値を検証 | 実装確認 |
| `crates/pasta_check/src/update_files.rs` | `297-303` | 形式テスト | `updates.txt` の `file` 行に MD5 スロットが含まれることを確認 | SSP仕様準拠 |

#### 用途評価
- `update_files.rs` 冒頭コメント（`1-3`行目）はこのモジュールの責務を「`updates.txt` を SSP 仕様に準拠して生成」と定義している。
- `generate_update_files` → `collect_files_recursive` → `calculate_md5` → `generate_updates_txt` の呼び出し連鎖を確認した結果、MD5 は各配布対象ファイルの内容ハッシュを `updates.txt` レコードへ埋め込むためだけに使われている。
- 生成されるレコードは `file,<filepath>\x01<md5>\x01size=<bytes>\x01date=<...>` 形式で、認証・署名・鍵導出・パスワード保存・改ざん耐性保証などの暗号学的処理には接続していない。
- `crates/**/*.rs` の検索では、MD5 の実装参照は `crates/pasta_check/src/update_files.rs` のみであり、他クレートでの暗号学的利用は確認されなかった。

#### deny.toml との整合
- `deny.toml:2-13` の `[licenses].allow` には `MIT` と `Apache-2.0` が含まれている。
- `cargo metadata --format-version 1 --locked` で、依存クレート `md5 0.8.0` のライセンスは `Apache-2.0/MIT` と確認できた。
- したがって `md5` クレートは deny ポリシー上すでに許可対象であり、追加の例外設定は不要である。MD5 の用途妥当性は本節で明示記録した。

#### Wave 1 audit-pasta-check との整合
- `completed/audit-pasta-check/research.md:30-37` では、`update_files.rs` の `calculate_md5` が「ファイル変更検出」「SSPネットワーク更新仕様」「暗号学的用途ではない」と評価されていた。
- 今回のワークスペース横断監査でも同じ結論を再確認した。Wave 1 の文書化内容と矛盾はない。

#### Requirement 5.3 判定
- **暗号学的用途の検出結果**: 該当なし。
- **移行推奨**: 現時点では不要。将来もし認証・署名・改ざん検知などの暗号学的用途が発生する場合は、MD5 継続利用ではなく SHA-256 / BLAKE3 等への移行を推奨する。

#### 総括
- Requirement 5.1: 使用箇所・用途をワークスペース横断で列挙し、`pasta_check` の `updates.txt` 生成に限定されることを記録した。
- Requirement 5.2: SSP 仕様準拠の非暗号学的ファイル変更検出用途であることを、実装・テスト・Wave 1 記録の3点から再確認した。
- Requirement 5.3: 暗号学的利用は検出されず、移行不要と判断した。
- Requirement 6.4: Wave 1 `audit-pasta-check` の依存関連知見を本 spec の research.md に統合した。
