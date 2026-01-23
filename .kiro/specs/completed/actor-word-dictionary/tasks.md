# Implementation Plan

## 1. パーサー層の拡張（ActorScope.code_blocks対応）

- [x] 1.1 (P) ActorScope構造体にcode_blocksフィールドを追加
  - ActorScopeにVec<CodeBlock>型のcode_blocksフィールドを追加
  - GlobalSceneScope/LocalSceneScopeと同じCodeBlock型を使用
  - new()メソッドにcode_blocks初期化を追加（Vec::new()）
  - _Requirements: 4_

- [x] 1.2 (P) parse_actor_scope関数でcode_blockルールをパース
  - Rule::code_blockをmatchケースに追加
  - parse_code_block()を呼び出してcode_blocksベクタに追加
  - GlobalSceneScope/LocalSceneScopeと同じパターンを踏襲
  - パースエラー時はParseError::InvalidCodeBlockを返す
  - _Requirements: 4_

- [x] 1.3 (P) パーサー層のユニットテストを作成
  - code_blockを含むactor_scopeが正しくパースされることを検証
  - code_blocks配列に正しく格納されることを検証
  - 複数のcode_blockが順序通りに格納されることを検証
  - _Requirements: 4_

---

## 2. トランスパイラ層の拡張（pasta_lua）

- [x] 2.1 (P) generate_actor関数でLua配列形式の単語定義を出力
  - 複数値の単語定義を`{ [=[値1]=], [=[値2]=] }`配列形式で出力
  - 単一値も単一要素配列として出力（後方互換性維持）
  - StringLiteralizerを使用して文字列リテラル化
  - 空のwordsは出力しない
  - _Requirements: 2, 6_

- [x] 2.2 (P) generate_actor関数でcode_blocksを展開
  - code_blocks配列を反復処理
  - language == "lua"のコードブロックのみ展開
  - アクター定義ブロック（doブロック）内に展開
  - コードブロック前後に空行を追加（可読性向上）
  - _Requirements: 4_

- [x] 2.3 (P) WordDefRegistryにregister_actor()メソッドを追加
  - アクター名と単語キーから`:__actor_{sanitized_name}__:{word}`形式のキーを生成
  - アクター名のサニタイズ処理を実装（空白・記号を`_`に置換）
  - 値配列をWordEntryとして登録
  - 既存のinsert()メソッドを内部で使用
  - _Requirements: 4_

- [x] 2.4 (P) トランスパイラ層のユニットテストを作成
  - 複数値が配列形式で正しく出力されることを検証
  - code_blocksが正しく展開されることを検証
  - WordDefRegistry.register_actor()が正しいキー形式で登録することを検証
  - 単一値配列が正しく出力されることを検証（後方互換性）
  - _Requirements: 2, 4, 6_

---

## 3. ランタイム層の拡張（word.lua）

- [x] 3.1 (P) アクター単語辞書レジストリを実装
  - actor_wordsグローバルテーブルを作成（actor_name → {key → values[][]}）
  - create_actor()関数を実装（WordBuilderパターン）
  - get_actor_words()関数を実装（辞書取得）
  - グローバル/シーン単語辞書と同じビルダーパターンを踏襲
  - _Requirements: 4_

- [x] 3.2 (P) pasta.globalモジュールを作成
  - 空のGLOBALテーブルを返すモジュールを作成
  - ユーザーが関数を追加可能なドキュメントコメントを記載
  - `crates/pasta_lua/scripts/pasta/global.lua`に配置
  - _Requirements: 4_

- [x] 3.3 search_prefix_lua()ローカル関数を実装
  - 辞書キーを反復処理し前方一致検索
  - マッチした全候補の値を配列に収集
  - value_arrays は `[[値1, 値2], [値3]]` 形式を展開
  - 候補がない場合はnilを返す
  - _Requirements: 4_

- [x] 3.4 resolve_value()ローカル関数を実装
  - valueがnilなら nilを返す
  - valueが関数なら実行結果を返す
  - valueが配列なら最初の要素を返す
  - その他の値なら文字列化して返す
  - _Requirements: 4_

- [x] 3.5 PROXY:word()メソッドで6レベルフォールバック検索を実装
  - L1: アクター完全一致（self.actor[name]）→ resolve_value()で解決
  - L2: アクター辞書前方一致 → search_prefix_lua() → ランダム選択
  - L3: シーン完全一致（scene[name]）→ resolve_value()で解決
  - L4: シーン辞書前方一致 → search_prefix_lua() → ランダム選択
  - L5: グローバル完全一致（GLOBAL[name]）→ resolve_value()で解決
  - L6: グローバル辞書前方一致 → search_prefix_lua() → ランダム選択
  - 各レベルでマッチしたらそのスコープで検索終了
  - すべて失敗したらnilを返す
  - _Requirements: 3, 4_

- [x] 3.6 (P) ランタイム層のユニットテストを作成
  - create_actor/get_actor_wordsの動作を検証
  - search_prefix_lua()が正しく前方一致検索することを検証
  - resolve_value()が各型（nil/関数/配列/値）を正しく解決することを検証
  - _Requirements: 3, 4_

---

## 4. 統合テスト（6レベル網羅 + エッジケース）

- [x] 4.1 テストフィクスチャcomprehensive_fallback_test.pastaを作成
  - グローバル単語定義（＠天気、＠天気予報、＠挨拶）
  - アクター定義（さくら: 単語＋関数、うにゅう: 単語のみ）
  - シーン定義（＠季節、＠季節感）＋シーン関数（日付）
  - luaコードブロックでアクター関数（時刻、天気オーバーライド）定義
  - _Requirements: 1, 2, 4, 5_

- [x] 4.2 基本検索テスト（T1-T6: 各レベル確認）
  - T1: さくら.word("時刻") → "朝"（L1 アクター関数）
  - T2: さくら.word("表情") → "\s[0]" or "\s[1]"（L2 アクター辞書）
  - T3: さくら.word("日付") → "1月1日"（L3 シーン関数）
  - T4: さくら.word("季節") → "春" or "夏"（L4 シーン辞書）
  - T5: さくら.word("時報") → "正午です"（L5 グローバル関数、テストセットアップで定義）
  - T6: さくら.word("挨拶") → "こんにちは" or "おはよう"（L6 グローバル辞書）
  - _Requirements: 3, 4_

- [x] 4.3 フォールスルー確認テスト（T7-T8）
  - T7: うにゅう.word("天気") → "雨" or "雪" or "台風"（アクター辞書なし→グローバルへ）
  - T8: うにゅう.word("表情") → "\s[10]" or "\s[11]"（別アクターの辞書）
  - _Requirements: 4_

- [x] 4.4 前方一致テスト（T9-T11）
  - T9: さくら.word("表") → "\s[0]" or "\s[1]"（L2 前方一致）
  - T10: さくら.word("季") → "春" or "夏" or "暖かい" or "涼しい"（L4 前方一致、複数キーマッチ）
  - T11: うにゅう.word("天") → "雨" or "雪" or "台風" or "晴れのち曇り"（L6 前方一致）
  - _Requirements: 4_

- [x] 4.5 関数優先テスト（T12: オーバーライド）
  - T12: さくら.word("天気") → "アクター関数の天気"（同名関数が辞書より優先）
  - _Requirements: 4_

- [x] 4.6 エッジケーステスト（T13-T15）
  - T13: さくら.word("存在しない") → nil（全レベル検索失敗）
  - T14: さくら.word("単一") → "固定値"（単一値配列、後方互換）
  - T15: さくら.word("") → nil（空文字キー）
  - _Requirements: 4, 6_

---

## 改訂履歴

| 日付 | 内容 |
|------|------|
| 2026-01-23 | タスク生成（議題1-3決定反映、Lua側実装、6レベル網羅テスト、15テストケース） |
| 2026-01-24 | 全タスク完了（TDD実装、22統合テスト、8 Luaユニットテスト） |
