# Implementation Plan

## Task List

- [ ] 1. ファイル・ディレクトリのリネームと mod.rs 更新
  - [ ] 1.1 ファイルリネーム実行
    - Rustソースファイルをリネーム：`label_registry.rs` → `scene_registry.rs`、`labels.rs` → `scene.rs`
    - テストファイルをリネーム：3ファイル（label_id_consistency_test.rs、pasta_engine_label_resolution_test.rs、pasta_transpiler_label_registry_test.rs）
    - `git mv` コマンドで履歴を保持しながら実行
    - _Requirements: 3.3, 5.1_
  - [ ] 1.2 モジュール宣言の更新
    - `src/transpiler/mod.rs` で `mod label_registry;` を `mod scene_registry;` に変更
    - `src/runtime/mod.rs` で `mod labels;` を `mod scene;` に変更
    - マイルストーン1コミット: `refactor(label-to-scene): Phase 1 ファイルリネーム完了`
    - _Requirements: 3.3_

- [ ] 2. Rust型名の型安全リネーム（4段階の段階的実行）
  - [ ] 2.1 Parser層の型リネーム
    - IDE Rename機能で `LabelDef` → `SceneDef`、`LabelScope` → `SceneScope` をリネーム
    - `cargo test --all` で検証
    - 失敗時はマニュアル修正
    - マイルストーン2.1コミット: `refactor(label-to-scene): Phase 2.1 Parser層型リネーム完了`
    - _Requirements: 3.6, 3.7_
  - [ ] 2.2 Transpiler層の型リネーム
    - IDE Rename機能で `LabelRegistry` → `SceneRegistry`、`LabelInfo` → `SceneInfo` をリネーム
    - `cargo test --all` で検証
    - マイルストーン2.2コミット: `refactor(label-to-scene): Phase 2.2 Transpiler層型リネーム完了`
    - _Requirements: 3.1, 3.2_
  - [ ] 2.3 Runtime層の型リネーム
    - IDE Rename機能で `LabelTable` → `SceneTable`、`LabelId` → `SceneId`、`LabelInfo` → `SceneInfo` をリネーム（Runtime版）
    - `cargo test --all` で検証
    - マイルストーン2.3コミット: `refactor(label-to-scene): Phase 2.3 Runtime層型リネーム完了`
    - _Requirements: 3.4_
  - [ ] 2.4 Error型と その他の型リネーム
    - IDE Rename機能で `LabelNotFound` → `SceneNotFound` をリネーム
    - `cargo test --all` で検証
    - テストコード内のアサーションメッセージを修正（必要に応じて）
    - マイルストーン2.4コミット: `refactor(label-to-scene): Phase 2.4 Error型リネーム完了`
    - _Requirements: 3.10, 6.1_

- [ ] 3. Rustソースコード内の変数名・コメント置換（3段階の正規表現置換）
  - [ ] 3.1 スネークケース変数名の置換
    - `find src/transpiler src/runtime -name "*.rs" -exec sed -i 's/\blabel_/scene_/g'` を実行
    - `cargo test --all` で検証
    - テスト失敗時は関連ファイルを修正
    - マイルストーン3.1コミット: `refactor(label-to-scene): Phase 3.1 label_* 置換完了`
    - _Requirements: 4.4, 4.5, 4.6_
  - [ ] 3.2 ローカル変数・引数のlabel→scene置換
    - `src/**/*.rs` と `tests/**/*.rs` で単語境界を考慮した置換：`\blabel\b` → `scene`
    - `cargo test --all` で検証
    - マイルストーン3.2コミット: `refactor(label-to-scene): Phase 3.2 label→scene 置換完了`
    - _Requirements: 4.1, 4.2_
  - [ ] 3.3 複数形・コメント内のキャメルケース置換
    - `\blabels\b` → `scenes`、`\bLabel\b` → `Scene` を置換
    - `cargo test --all` で検証、テスト失敗時はアサーションメッセージ修正
    - エラーメッセージ内の「ラベル」を「シーン」に変更（日本語メッセージ）
    - マイルストーン3.3コミット: `refactor(label-to-scene): Phase 3.3 複数形・コメント置換完了`
    - _Requirements: 4.3, 6.3_

- [ ] 4. Transpiler生成コードの文字列リテラル修正
  - [ ] 4.1 生成Rune関数名と変数名の修正
    - `src/transpiler/mod.rs` で `label_selector` → `scene_selector`、`label_fn` → `scene_fn` などの生成コード用文字列を修正
    - `src/stdlib/mod.rs` で `select_label_to_id` → `select_scene_to_id` に修正
    - `cargo test --all` で検証
    - マイルストーン4.1コミット: `refactor(label-to-scene): Phase 4.1 生成Rune関数修正完了`
    - _Requirements: 3.8, 3.9, 7.1, 7.2, 7.3, 7.4_
  - [ ] 4.2 生成Runeコード内のエラーメッセージ修正
    - 「ラベルID」→「シーンID」に修正
    - 「Label not found」→「Scene not found」に修正
    - `cargo test --all` で検証
    - テスト失敗時は期待値を修正
    - マイルストーン4.2コミット: `refactor(label-to-scene): Phase 4.2 エラーメッセージ修正完了`
    - _Requirements: 6.2, 7.5_

- [ ] 5. Markdown文書の用語置換（2段階）
  - [ ] 5.1 ステアリングドキュメント・メイン仕様の置換
    - `.kiro/steering/` (product.md, tech.md, structure.md) と `GRAMMAR.md`、`SPECIFICATION.md`、`README.md` の Markdown 置換
    - 複合語（グローバルラベル、ローカルラベル）から順番に置換（長いパターンを先に処理）
    - 最後に基本形（ラベル→シーン）を置換
    - `grep -r "ラベル" .kiro/steering/ GRAMMAR.md SPECIFICATION.md README.md` で確認
    - マイルストーン5.1コミット: `refactor(label-to-scene): Phase 5.1 ステアリング・メイン仕様置換完了`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4_
  - [ ] 5.2 サンプル・完了仕様ドキュメントの置換
    - `examples/**/*.md` と `.kiro/specs/completed/` 内のMarkdown置換
    - 同じパターン順序で置換
    - `grep -r "ラベル" examples/ .kiro/specs/completed/` で確認
    - マイルストーン5.2コミット: `refactor(label-to-scene): Phase 5.2 Examples・完了仕様置換完了`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 8.1, 8.2_

- [ ] 6. 最終検証と整理
  - [ ] 6.1 全コンパイル・テスト・Lint検証
    - `cargo test --all` で全テスト実行
    - `cargo clippy` でLint警告確認
    - 新規警告がないことを確認
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10_
  - [ ] 6.2 残存用語の grep 確認
    - `grep -r "\blabel\b\|\bLabel\b" src/ tests/` で Rust ファイル内の残存確認
    - `grep -r "ラベル" *.md .kiro/ examples/` で Markdown 内の残存確認
    - 残存があれば修正
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 8.1, 8.2_
  - [ ] 6.3 最終コミット
    - `git add -A && git commit -m "refactor(label-to-scene): Phase 6 最終検証完了"`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2_

- [ ] 7. 仕様ディレクトリのリネーム（オプション）
  - [ ] 7.1 最終的なディレクトリリネーム
    - `git mv .kiro/specs/refactor-label-to-scene .kiro/specs/refactor-scene` でディレクトリをリネーム（仕様完了後、オプション）
    - マイルストーン7コミット: `refactor(label-to-scene): Phase 7 ディレクトリリネーム・リファクタリング完了`
    - _Requirements: 8.3_

