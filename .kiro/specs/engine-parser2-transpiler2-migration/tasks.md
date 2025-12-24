# Implementation Plan

## Task Breakdown

### Phase 1: Import更新とコンパイルエラー確認

- [ ] 1. engine.rsのimport文を新スタックに切り替え
  - `use crate::parser` → `use crate::parser2`
  - `use crate::transpiler::Transpiler` → `use crate::transpiler2::Transpiler2`
  - `use crate::error::Transpiler2Pass`を追加
  - `use crate::registry::{SceneRegistry, WordDefRegistry}`を追加
  - 旧スタックへの参照がすべて削除されていることを確認
  - _Requirements: 1, 7_

### Phase 2: Parse Loop変更とエラーハンドリング

- [ ] 2. Parse Loop実装をparser2に切り替え
  - `parser::parse_file()`呼び出しを`parser2::parse_file()`に変更
  - エラー型は既にPastaError::ParseErrorに変換済みのため追加処理不要
  - parsed_filesの型がparser2::PastaFileに適合することを確認
  - _Requirements: 1, 5_

### Phase 3: ASTマージロジック変更

- [ ] 3. ASTマージをitems構造対応に実装
  - 旧`all_scenes`/`all_global_words`分離ロジックを削除
  - 新`all_items: Vec<FileItem>`を用意してitemsをextend
  - merged_fileをPastaFile構築（path: "merged", items: all_items, span: None）
  - _Requirements: 2, 5_

### Phase 4: Transpile 2-Pass実装

- [ ] 4.1 (P) Registry初期化とバッファ準備
  - `SceneRegistry::new()`と`WordDefRegistry::new()`を呼び出し
  - `Vec<u8>`バッファを作成
  - _Requirements: 3, 4_

- [ ] 4.2 Pass1トランスパイル実装
  - `Transpiler2::transpile_pass1()`を呼び出し
  - エラーを`map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass1))`で変換
  - _Requirements: 3, 4_

- [ ] 4.3 Pass2トランスパイル実装
  - `Transpiler2::transpile_pass2()`を呼び出し
  - エラーを`map_err(|e| e.into_pasta_error(Transpiler2Pass::Pass2))`で変換
  - バッファをString::from_utf8()で変換してrune_codeを取得
  - _Requirements: 3, 4_

- [ ] 4.4 旧Transpiler::transpile_with_registry呼び出しを削除
  - 新2-passロジックで完全に置き換えられていることを確認
  - _Requirements: 3, 7_

### Phase 5: 統合検証とテスト

- [ ] 5.1 (P) 全611テスト実行と回帰検証
  - `cargo test --all`を実行
  - 全611テスト（ignored 3件除く）が合格することを確認
  - エンジン統合テスト（`pasta_engine_rune_*`）の重点確認
  - E2Eテスト（`pasta_integration_*`）の動作確認
  - _Requirements: 5, 6_

- [ ] 5.2 (P) ランタイム層互換性確認
  - SceneTable/WordTable構築が旧スタックと同等の動作をすることを確認
  - ScriptEvent出力が既存フィクスチャで一致することを確認
  - 前方一致選択の挙動が維持されていることを確認
  - _Requirements: 5_

### Phase 6: ドキュメント更新

- [ ] 6. ドキュメントを新スタック反映に更新
  - README.mdでparser2/transpiler2採用を明記
  - engine.rs関連ドキュメントで旧スタック記述を新スタックに置き換え
  - 2-pass戦略の説明を追加
  - _Requirements: 7_

---

## Requirements Coverage

| Requirement | Mapped Tasks |
|-------------|--------------|
| REQ 1: Parser2統合 | 1, 2 |
| REQ 2: ASTマージitems化 | 3 |
| REQ 3: Transpiler2二段トランスパイル | 4.1, 4.2, 4.3, 4.4 |
| REQ 4: Registry統合とランタイム生成 | 4.1, 4.2, 4.3 |
| REQ 5: 後方互換性と動作維持 | 2, 3, 5.1, 5.2 |
| REQ 6: テストおよび品質ゲート | 5.1 |
| REQ 7: ドキュメント更新 | 1, 4.4, 6 |

---

## Task Progression Notes

- **Phase 1**: コンパイルエラーが出るが正常（後続フェーズで解消）
- **Phase 2-4**: 段階的に実装、各フェーズ完了時に`cargo test`で検証
- **Phase 5**: 全テスト合格確認（DoD Gate）
- **Phase 6**: ドキュメント整合性確保

**並列実行可能**: タスク4.1, 5.1, 5.2（依存関係なし）
