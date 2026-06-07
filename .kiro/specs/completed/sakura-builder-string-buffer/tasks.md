# Implementation Plan

- [ ] 1. Foundation: 新規葉ユーティリティモジュール
- [x] 1.1 (P) pasta.buf 実装と単体テスト
  - LuaJIT String Buffer が利用可能なら new() がネイティブ生成関数を採用し、不在時は put/tostring を備えた最小実装へ pcall で安全にフォールバックする
  - new() が返すバッファは put（追記・self 返却）と tostring（追記順連結・非破壊）を提供する
  - backend フィールドでバックエンド種別を公開し、new_fallback() で最小実装を明示生成可能にする
  - 単体テストで put→tostring 連結（"abc"）・空連結（""）・backend がネイティブを示すことを検証する
  - Observable: buf 単体テストが全パスし、ネイティブ採用と最小実装の双方が同一連結結果を返す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 4.3, 5.1_
  - _Boundary: pasta.buf_
- [x] 1.2 (P) pasta.lua_version 実装と単体テスト
  - jit テーブルの有無で LuaJIT を判定し（_VERSION に依存しない）、LuaJIT は 200+major×10+minor、標準 Lua は 100+major×10+minor の整数を返す
  - 例外を送出せず常に整数を返す
  - 単体テストで整数返却・本ランタイムで 221・>=200（LuaJIT 判定）を検証する
  - Observable: lua_version 単体テストがパスし、本 LuaJIT 2.1 ランタイムで get() が 221 を返す
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Boundary: pasta.lua_version_
- [x] 1.3 新規 Lua spec のテストスイート登録
  - 新規 spec（buf／lua_version）をテストエントリポイントへ登録し、ユニットテストランナーから実行されるようにする
  - Observable: テストランナー実行時に buf／lua_version の spec が読み込まれ実行される
  - 達成済み: buf_test は task 1.1、lua_version_test は task 1.2 で init.lua に登録（逐次実行のため競合なし）。ランナーで 36 スイート（うち両 spec）実行を確認
  - _Depends: 1.1, 1.2_
  - _Requirements: 4.3, 6.1_
  - _Boundary: テストスイート登録_

- [ ] 2. Core: sakura_builder のバッファ化
- [x] 2.1 build() のバッファ化と buffer_factory 注入
  - build 内部の table 蓄積＋concat を buf 経由（put/tostring）へ置換し、末尾 \e も put して tostring で返す
  - 設定に任意の buffer_factory（既定は buf.new）を受け、既存呼び出しは無改修で同一挙動を保つ
  - Observable: 既存の sakura_builder テストがバイト一致で全パス（出力契約不変）
  - _Depends: 1.1_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Boundary: sakura_builder.build_

- [ ] 3. Validation: テスト拡充
- [x] 3.1 (P) sakura_builder 回帰＋新規ケース
  - 空 grouped_tokens で \e のみを返すことを検証する
  - buffer_factory に最小実装を注入して build をフォールバックで実走し、既定（ネイティブ）出力とバイト一致を比較する
  - Observable: 空入力・フォールバック実走比較を含む sakura_builder テストが全パス
  - _Depends: 2.1_
  - _Requirements: 2.4, 3.4, 3.5, 4.1, 4.2_
  - _Boundary: sakura_builder テスト_
- [x] 3.2 (P) String Buffer 採用の実機検証テスト（Rust）
  - 本番同一経路（RuntimeConfig 既定の to_stdlib + unsafe_new_with）でランタイムを構築し、string.buffer のロードと new() 機能を検証する
  - 利用不可なら finding（lua_version の数値併記）としてテスト失敗で明示し、黙ってフォールバックを合格としない
  - Observable: 本番相当ランタイムで string.buffer が採用されることを示す Rust テストがパス（不可ならランタイム版番号付きで失敗）
  - _Depends: 1.1, 1.2_
  - _Requirements: 5.1, 5.2, 5.3_
  - _Boundary: string_buffer_availability_test_

- [x] 4. luacheck＋全テストスイート通過確認
  - 新規 Lua モジュールが luacheck（Lua 5.1 規約）を通過する
  - cargo test -p pasta_lua（lua ユニットランナー＋可用性テスト）が全パスし、リグレッション0
  - Observable: luacheck と全テストがグリーンで、さくらスクリプト出力のバイト一致が維持されている
  - _Depends: 1.3, 2.1, 3.1, 3.2_
  - _Requirements: 4.1, 4.3, 5.1_
  - 達成: `cargo test -p pasta_lua` 全バイナリ green（unit 183 / Lua runner 36 suites / availability 1 / 他統合すべて 0 failed）。新規警告ゼロ（`unused import PathBuf` は src/loader/extract.rs の既存・境界外）。
  - luacheck/lua/luajit は本環境の PATH に未インストール（CI 未使用・dev 手動ツール）。新規 Lua（buf.lua/lua_version.lua/各 test）は `.luacheckrc`（lua51）準拠を手動照合: グローバル書込みなし・未使用変数なし・許可グローバルのみ使用。luacheck CLI 実証は環境制約で未実施。

## Implementation Notes

- **ビルド/テスト環境の前提（重要）**: この実行環境は `NoDefaultCurrentDirectoryInExePath=1` が設定されており、LuaJIT vendored ビルド（mlua-sys）の `minilua` カレントディレクトリ解決が壊れ、`cargo build/test` が `luajit.h` コピー失敗で落ちる。**cargo コマンドは必ず `unset NoDefaultCurrentDirectoryInExePath` を前置すること**。mlua-sys は既にビルド済みキャッシュあり（再ビルド時のみ minilua が必要だが、前置すれば安全）。
- **検証コマンド**:
  - Lua 単体: `unset NoDefaultCurrentDirectoryInExePath && cargo test -p pasta_lua --test lua_unittest_runner run_lua_unit_tests -- --nocapture`
  - クレート全体: `unset NoDefaultCurrentDirectoryInExePath && cargo test -p pasta_lua`
  - baseline: 34 suites passed（実装前の green 基準）
- **新規 Lua spec の実行**: `crates/pasta_lua/tests/lua_specs/init.lua` の `specs` テーブルに spec 名を追加しないとランナーから実行されない。新規テストを書いたタスクは自分の spec を init.lua に登録して検証すること（逐次実行のため init.lua の競合は起きない）。
