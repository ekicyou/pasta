# Implementation Plan

## タスク一覧

- [ ] 1. 監査環境セットアップ・脆弱性チェック
- [x] 1.1 cargo-auditのインストールとRustSec Advisory DBに基づく全依存の脆弱性スキャンを実行する
  - `cargo install cargo-audit` でツールを導入し、`cargo audit` をワークスペースルートで実行する
  - 検出されたadvisoryがあれば、ID・深刻度・影響範囲・推奨対処をresearch.mdに記録する
  - advisoryが0件の場合はクリーンステータスをresearch.mdに記録する
  - 実行完了後、脆弱性チェック結果がresearch.mdに構造化されて記録されている
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 6.1, 6.2_

- [x] 1.2 cargo-denyのインストールとdeny.toml設定ファイルを作成する
  - `cargo install cargo-deny` でツールを導入する
  - ワークスペースルートにdeny.tomlを作成し、ライセンス許可リスト（MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib, BSL-1.0）を設定する
  - advisoriesセクションでvulnerability=deny, unmaintained=warn, yanked=warnを設定する
  - sourcesセクションで未知レジストリ・Gitソースへの警告を設定する
  - `cargo deny check` が成功し、deny.tomlがワークスペースルートに存在する
  - _Requirements: 2.1, 2.2, 2.3, 6.1_

- [ ] 2. ライセンス互換性監査
- [x] 2.1 cargo-denyによるライセンス監査を実行し結果を記録する
  - `cargo deny check licenses` を実行し、全依存のライセンスを検査する
  - vendoredソース（mlua/LuaJIT — MIT License）のライセンスも確認する
  - 非互換ライセンスが検出された場合は該当クレート名とライセンス種別をresearch.mdに記録する
  - ライセンス監査結果（許可・警告・エラー）がresearch.mdに記録されている
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 6.2_

- [ ] 3. 不要依存の分析と除去
- [x] 3.1 cargo-treeによる依存ツリー分析と未使用依存の特定を行う
  - `cargo tree` で全クレートの依存ツリーを可視化する
  - `cargo tree --duplicates` で重複依存を確認する
  - 各クレートのCargo.tomlとソースコードを照合し、実際に使用されていない直接依存を特定する
  - 未使用依存の一覧（クレート名・使用元・判定根拠）がresearch.mdに記録されている
  - _Requirements: 3.1, 3.2, 6.2_

- [x] 3.2 特定された未使用依存をCargo.tomlから除去し、回帰テストを実行する
  - 未使用と判定された依存を該当クレートのCargo.tomlから除去する
  - `cargo build` が全クレートで成功することを確認する
  - `cargo test` で全テストがパスすることを確認する
  - 除去した依存の一覧と回帰テスト結果がresearch.mdに記録されている
  - _Requirements: 3.2, 3.3, 3.4, 7.1, 7.2_

- [ ] 4. バージョン固定戦略の改善
- [x] 4.1 workspace管理外の依存をworkspace.dependenciesに統合する (P)
  - ルートCargo.tomlのworkspace.dependenciesに以下を追加: lexopt, md5, zip, tower-lsp, image, imageproc, wasm-bindgen, wasm-bindgen-futures, js-sys, serde-wasm-bindgen, tokio
  - 各クレートのCargo.tomlを`.workspace = true`参照に変更する
  - features指定がある依存（zip, tower-lsp）はworkspace定義でfeaturesを含める
  - 全直接依存がworkspace.dependenciesで管理され、`cargo build` が成功する
  - _Requirements: 4.1, 4.2, 7.1_
  - _Boundary: Cargo.toml（ルート + 各クレート）_

- [x] 4.2 マイナー/パッチバージョンの更新候補を調査し、安全な更新を実施する (P)
  - `cargo update --dry-run` でマイナー/パッチ更新可能な依存を確認する
  - 各更新の変更履歴（CHANGELOG）を確認し、破壊的変更がないことを確認する
  - 安全と判断された更新を`cargo update`で適用する
  - `cargo build && cargo test` で全テストパスを確認する
  - 更新した依存の一覧（旧バージョン→新バージョン）がresearch.mdに記録されている
  - _Requirements: 4.3, 4.4, 7.1, 7.2_
  - _Boundary: Cargo.lock_

- [ ] 5. MD5用途適切性の統合評価
- [x] 5.1 MD5クレートの全使用箇所を調査し、用途の適切性を統合的に評価・文書化する
  - pasta_check内のMD5使用箇所（update_files.rs）を確認する
  - 用途がファイル変更検出（SSP仕様準拠）であり暗号学的用途でないことを再確認する
  - Wave 1 audit-pasta-checkでの文書化内容と整合性を確認する
  - MD5用途評価結果がresearch.mdに統合的に記録され、deny.tomlでmd5クレートが明示的に許可されている
  - _Requirements: 5.1, 5.2, 5.3, 6.4_

- [ ] 6. 監査レポートの最終化と回帰検証
- [ ] 6.1 全監査カテゴリの結果をresearch.mdに統合し、Wave 1知見との整合性を確認する
  - 脆弱性チェック・ライセンス監査・不要依存分析・バージョン戦略の各結果を構造化してresearch.mdに記録する
  - Wave 1全spec（audit-pasta-core〜audit-pasta-sample-ghost）の依存関連知見を本レポートに統合する
  - 監査実行日時・使用ツールバージョン・監査対象範囲を記録する
  - research.mdに全監査カテゴリの結果が構造化されて記録されている
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 6.2 最終回帰テストとクロスコンパイル検証を実施する
  - `cargo build` が全クレートで成功することを確認する
  - `cargo test` で全テスト（950+件）がパスすることを確認する
  - `cargo build --target i686-pc-windows-msvc` でクロスコンパイルが成功することを確認する
  - `cargo deny check` がエラー0件で完了することを確認する
  - `cargo audit` がadvisory 0件（または既知・許容済み）で完了することを確認する
  - 全回帰テスト・ポリシーチェックがパスし、結果がresearch.mdに記録されている
  - _Requirements: 7.1, 7.2, 7.3, 7.4_
  - _Depends: 3.2, 4.1, 4.2_
