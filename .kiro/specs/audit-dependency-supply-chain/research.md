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
