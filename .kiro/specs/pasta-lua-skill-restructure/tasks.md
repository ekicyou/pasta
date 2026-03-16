# Implementation Plan

- [ ] 1. SKILL.md をコンパクト化する
- [ ] 1.1 §1 Purpose & Prerequisites と §2 Quick Reference を現行のまま維持しつつ、§3〜§7 を各セクション 3〜5 行の要約に圧縮する
  - 各要約には必須キーワード（設計書の「§3-§7 各セクションの必須キーワード」テーブル）を含める
  - 各要約末尾に `> 📖 詳細: [references/ファイル名.md](references/ファイル名.md)` リンクを付与する
  - 現行の「（情報ソース: ...）」フッターは削除する
  - _Requirements: 1.2, 5.1, 5.2, 5.3_

- [ ] 1.2 References インデックスセクションを新設する
  - SKILL.md 末尾に `## References` セクションを追加する
  - 5 つのリファレンスファイルへの相対パスリンクと一行説明を含むテーブルを作成する
  - テーブル直上に `read_file でロードすること` 旨の案内を追加する
  - 最終行数が 500 行未満であることを確認する
  - _Requirements: 1.1, 2.3, 5.4_

- [ ] 2. references/ の 5 つのリファレンスファイルを作成する

- [ ] 2.1 (P) Runtime API リファレンスを作成する（`references/runtime-api.md`）
  - SKILL.md §4 の内容を詳細版に展開する
  - LUA_API.md §2〜§6, §8 の内容を統合・リッチ化する
  - `@pasta_search`, `@pasta_persistence`, `@pasta_config`, `@pasta_sakura_script`, `@enc`, mlua-stdlib の完全な API シグネチャ（全パラメータ・戻り値型・エラー条件）を記述する
  - pcall 保護の要否・フォールバック戦略・エッジケース（`@enc` プラットフォーム依存性、`@pasta_config` pcall 必須理由）を明記する
  - ファイル末尾に internal-modules.md・shiori-handlers.md への「関連リファレンス」セクションを配置する
  - _Requirements: 2.2, 2.4, 3.1, 4.1, 4.2, 4.3, 4.4_

- [ ] 2.2 (P) Internal Modules リファレンスを作成する（`references/internal-modules.md`）
  - SKILL.md §5 の内容を詳細版に展開する
  - lua-coding.md §6 および LUA_API.md §7 の内容を統合・リッチ化する
  - STORE, ACT, SCENE, WORD, GLOBAL, SAVE, finalize_scene の完全な API シグネチャとメソッド一覧を記述する
  - 循環参照回避の原則（STORE パターン）を詳説する
  - ファイル末尾に runtime-api.md・shiori-handlers.md への「関連リファレンス」セクションを配置する
  - _Requirements: 2.2, 2.4, 3.2, 4.1, 4.2, 4.3, 4.4_

- [ ] 2.3 (P) SHIORI Handlers リファレンスを作成する（`references/shiori-handlers.md`）
  - SKILL.md §6 の内容を詳細版に展開する
  - LUA_API.md §9 の内容を統合・リッチ化する
  - REG, RES, 主要 SHIORI イベント一覧（reference[N] の意味を含む）を完全に記述する
  - フォールバックチェーン（REG → SCENE.search → 204 No Content）と仮想ディスパッチャ（OnTalk/OnHour）を詳説する
  - ファイル末尾に internal-modules.md・runtime-api.md への「関連リファレンス」セクションを配置する
  - _Requirements: 2.2, 2.4, 3.3, 4.1, 4.2, 4.3, 4.4_

- [ ] 2.4 (P) Coding Conventions リファレンスを作成する（`references/coding-conventions.md`）
  - SKILL.md §3 の内容を詳細版に展開する
  - lua-coding.md §1〜§5 の内容を統合・リッチ化する
  - MODULE/MODULE_IMPL 分離・STORE パターンの命名規約・禁止パターン・EmmyLua 型注釈（`@class`, `@field`, `@param`, `@return`）・ガードクローズを網羅する
  - 他ファイルから独立しているため「関連リファレンス」セクションは省略可
  - _Requirements: 2.2, 2.4, 3.4, 4.1, 4.2, 4.3, 4.4_

- [ ] 2.5 (P) Testing & Lint リファレンスを作成する（`references/testing-lint.md`）
  - SKILL.md §7 の内容を詳細版に展開する
  - lua-coding.md §7 の内容を統合・リッチ化する
  - lua_test の API（describe, test, expect, マッチャー一覧）・テストファイル規約・決定論的テスト（set_scene_selector / set_word_selector）・luacheck 設定例を網羅する
  - ファイル末尾に runtime-api.md（@pasta_search test selectors）への「関連リファレンス」セクションを配置する
  - _Requirements: 2.2, 2.4, 3.5, 4.1, 4.2, 4.3, 4.4_

- [ ] 3. 旧権威ドキュメントを廃止し、クロスリファレンスを更新する

- [ ] 3.1 references/ 各ファイルが LUA_API.md の全セクションをカバーしていることを検証してから LUA_API.md を削除する
  - LUA_API.md の 9 セクション（§1〜§9）を references/ 内のいずれかのファイルがカバーしているか確認する
  - 漏れがあれば対象 references/ ファイルに追記してから削除を実行する
  - _Requirements: 6.1_

- [ ] 3.2 references/ 各ファイルが lua-coding.md の全セクションをカバーしていることを検証してから lua-coding.md を削除する
  - lua-coding.md の 7 セクション（§1〜§7）を references/ 内のいずれかのファイルがカバーしているか確認する
  - 漏れがあれば対象 references/ ファイルに追記してから削除を実行する
  - _Requirements: 6.2_

- [ ] 3.3 (P) SOUL.md の LUA_API.md リンクをスキルリファレンスへ更新する
  - Line 24 のリンクを `[pasta-lua-coding skill](.agents/skills/pasta-lua-coding/SKILL.md) - Lua APIリファレンス（references/に詳細）` に変更する
  - _Requirements: 6.3_

- [ ] 3.4 (P) GRAMMAR.md の LUA_API.md リンクをスキルリファレンスへ更新する
  - Line 753 のリンクを `[pasta-lua-coding skill](.agents/skills/pasta-lua-coding/SKILL.md) - Lua APIリファレンス（references/に詳細）` に変更する
  - _Requirements: 6.4_

- [ ] 4. references/ 各ファイルの自己完結性とクロスリファレンスリンクを検証する
  - 各ファイルが SKILL.md コンテキストなしで独立して読めることを確認する（冒頭の導入文があるか）
  - ファイル間のクロスリファレンスリンク（相対パス + GFM アンカー）が正しいことを確認する
  - SKILL.md の総行数が 500 行未満であることを確認する
  - _Requirements: 1.1, 2.4, 4.3_
