# Implementation Tasks

## Task Breakdown

- [ ] 1. SHIORI_ACT.yield()メソッドの実装
- [ ] 1.1 (P) yield()メソッドをオーバーライド
  - `SHIORI_ACT_IMPL.yield(self)` を実装し、`coroutine.yield(self:build())` を呼び出す
  - さくらスクリプト文字列をyieldする（トークンオブジェクトではない）
  - コルーチン外で呼び出された場合はLuaネイティブエラーを発生させる
  - _Requirements: 1_

- [ ] 2. build()メソッドの自動リセット実装
- [ ] 2.1 (P) build()に自動リセット機能を追加
  - さくらスクリプト文字列構築後、返却前に `self:reset()` を呼び出す
  - リセット後は `_buffer` が空、`_current_spot` が nil になることを保証
  - 「1 yield = 1 build」原則を実装レベルで強制
  - _Requirements: 1.1_

- [ ] 2.2 (P) 変数名をscope→spotに変更
  - `_current_scope` を `_current_spot` に改名
  - `spot_to_scope()` を `spot_to_id()` に改名
  - `scope_to_tag()` を `spot_to_tag()` に改名
  - _Requirements: 1.1_

- [ ] 2.3 (P) spot_to_tag()をSSP仕様に準拠
  - すべてのスポットで `\p[ID]` 形式を使用するよう変更
  - 既存の `\0`, `\1` を廃止し、`\p[0]`, `\p[1]` に統一
  - _Requirements: 1.1, 2_

- [ ] 3. スポット切り替え改行設定の実装
- [ ] 3.1 (P) pasta.configモジュールの作成
  - `crates/pasta_lua/scripts/pasta/config.lua` を新規作成
  - `@pasta_config` をrequireし、`PASTA_CONFIG.get(section, key, default)` メソッドを実装
  - セクション指定でTOML設定値を取得する機能を提供
  - 設定値が存在しない場合はデフォルト値を返す
  - _Requirements: 3_

- [ ] 3.2 SHIORI_ACT.new()で設定読み込み
  - `pasta.config` をrequireし、`spot_switch_newlines` 設定を読み込む
  - `self._spot_switch_newlines` フィールドに保持（デフォルト: 1.5）
  - 設定ファイル未存在時はデフォルト値を使用
  - _Requirements: 2, 3_

- [ ] 3.3 talk()でスポット切り替え改行を挿入
  - スポット切り替え時、既存の固定改行（`\n`）を削除
  - スポットタグ（`\p[ID]`）の後に段落区切り改行（`\n[percent]`）を挿入
  - `spot_switch_newlines` 設定値を元にパーセント計算（`* 100`）
  - `spot_switch_newlines = 0` の場合は改行を挿入しない
  - _Requirements: 2_

- [ ] 4. テストfixtureの準備
- [ ] 4.1 (P) ghost設定付きfixtureの作成
  - `crates/pasta_lua/tests/fixtures/loader/with_ghost_config/pasta.toml` を作成
  - `[ghost]` セクションに `spot_switch_newlines = 2.0` を設定
  - 総合テストで使用する設定ファイルとして準備
  - _Requirements: 2, 4_

- [ ] 5. 既存テストの拡充
- [ ] 5.1 yield()メソッドのテストを追加
  - `pasta.co.safe_wrap()` でコルーチンをラップしてyield()をテスト
  - さくらスクリプト文字列がyieldされることを検証（`err=nil`）
  - yield後にバッファがリセットされることを検証（`_buffer`空、`_current_spot` nil）
  - コルーチン外でyield呼び出し時のエラーを検証
  - _Requirements: 1, 1.1, 5_

- [ ] 5.2 (P) pasta.configモジュールのテストを追加
  - デフォルト値の取得テスト（セクション・キー未定義時）
  - 設定値の取得テスト（存在するキー）
  - 存在しないキーのデフォルト値フォールバックテスト
  - セクション指定の動作確認（`[ghost]` セクション等）
  - _Requirements: 3, 5_

- [ ] 5.3 build()自動リセットのテスト修正
  - build()を複数回呼ぶテストを修正（2回目は空文字列+`\e`を期待）
  - バッファ状態検証テストを更新（build()後は空を期待）
  - リグレッション検証: 既存テストがすべてパスすることを確認
  - _Requirements: 1.1, 5_

- [ ] 5.4 (P) スポット切り替え改行のテストを追加
  - 既存の固定改行が削除され、設定可能な段落区切り改行に置き換わることを検証
  - `spot_switch_newlines = 1.5` で `\p[ID]\n[150]` が出力されることを確認
  - `spot_switch_newlines = 0` で改行が挿入されないことを確認
  - _Requirements: 2, 5_

- [ ] 6. 総合フィーチャーテストの実装
- [ ] 6.1 shiori_act_integration_test.luaの作成
  - `crates/pasta_lua/tests/lua_specs/shiori_act_integration_test.lua` を新規作成
  - 会話開始時の初期化状態を検証
  - 複数アクター会話（sakura, kero, char2）のスポット切り替えを検証
  - 表情変更（surface）とテキストの組み合わせを検証
  - 待機（wait）と改行（newline）のタイミング制御を検証
  - _Requirements: 4_

- [ ] 6.2 総合テストのシナリオ実装
  - メソッドチェーン（`act:talk(...):surface(5):wait(500)`）を検証
  - yield後のバッファリセットと継続を検証
  - 設定ファイルによる改行数変更を検証（`with_ghost_config` fixture使用）
  - 期待されるさくらスクリプト出力との完全一致を検証
  - エラーケース（無効なactor、コルーチン外yield等）を検証
  - _Requirements: 4_

- [ ] 7. SOUL.mdドキュメント更新
- [ ] 7.1 (P) スポット概念の追加
  - 「5.1 映画撮影のメタファー」セクションに「スポット」を追加
  - 用語定義表に「スポット」行を追加（照明位置、数値/オブジェクト、役割）
  - SHIORI実装における具体例を記載（sakura=0, kero=1）
  - 将来のノベルゲーム対応での拡張可能性に言及
  - スポット切り替えが会話のリズムを制御する重要な要素であることを明記
  - _Requirements: 6_

- [ ] 8. ドキュメント整合性の確認と更新
- [ ] 8.1 (P) プロジェクトドキュメントの整合性確認
  - SOUL.md - コアバリュー・設計原則との整合性確認
  - SPECIFICATION.md - 言語仕様の更新確認（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期確認（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - クレートREADME - API変更の反映確認（該当なし）
  - steering/* - 該当領域のステアリング更新確認
  - _Requirements: 1, 1.1, 1.2, 2, 3, 4, 5, 6_
