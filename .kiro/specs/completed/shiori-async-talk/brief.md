# Brief: shiori-async-talk

## 問題
トーク合成（シーン実行）中にSSPから情報を取得したい場面がある（プロパティ値の読み取り、他ゴーストの状態確認など）。SSPのプロパティシステムは `\![get,property,event,prop,...]` さくらスクリプトタグでSHIORI Eventコールバックを発火させる仕組みであり、値を取得するには：

1. さくらスクリプトタグを含むレスポンスをSSPに返す
2. SSPがタグを処理し、指定イベント名で新たなSHIORIリクエストを発火
3. Reference ヘッダにプロパティ値が格納されて戻ってくる

この「レスポンス送信 → コールバック受信 → 処理再開」パターンは現在のyield/resume基盤では対応できない。現在の `act:yield()` は「次の一般イベントで再開」であり、「特定のコールバックイベントを待って再開」という仕組みがない。

ゴースト作者がこのパターンを手動で実装するには、yield/コールバック登録/コルーチン管理を自力で行う必要があり、改造ハードルが非常に高い。

## 現状
- `act:yield()` → `coroutine.yield(result)` でさくらスクリプトを返し、`STORE.co_scene` にコルーチンを保存
- 次回イベント（`OnSecondChange` 等）で `resume_until_valid()` が再開
- 再開時に「どのイベントで再開するか」を指定する仕組みはない
- `\![get,property,...]` によるコールバックイベントのルーティング機構はない
- ゴースト作者が利用可能なプロパティ読み取りAPIはゼロ

## 期待する成果
- ゴースト作者がトーク合成中に `local version = act:get_property("baseware.version")` のように同期的なスタイルでSSPプロパティを取得できること
- yield/コールバック/resumeの複雑さが完全にフレームワーク内部に隠蔽されること
- 将来的に `\![get,property,...]` 以外の非同期SHIORIパターン（`\![get,...]` 系タグ全般）にも基盤を再利用できること

## アプローチ
「トーク合成中のSHIORI非同期通信」汎用基盤を構築し、`act:get_property(name)` を最初のコンシューマとして実装する。

### 基盤の仕組み
1. **Pending Callback Registry**: `act` が「次に来るべきコールバックイベントID」を登録する仕組み
2. **Tagged Yield**: `act:yield()` の拡張。通常のyield（次イベントで再開）に加え、「特定イベントID待ちyield」をサポート
3. **Event Router拡張**: `EVENT.fire()` がイベント受信時に pending callback をチェックし、該当イベントなら待機中コルーチンに値を渡してresumeする（通常のREGディスパッチをバイパス）
4. **`act:get_property(name)`**: 上記基盤を使い、内部でユニークなイベントIDを生成 → `\![get,property,eventId,name]` タグ発行 → tagged yield → コールバック受信 → Reference0の値を返す

### 設計上の考慮点
- 複数プロパティの一括取得: `act:get_property(name1, name2, ...)` → 複数Referenceを返すパターンへの対応
- エラーハンドリング: コールバックが来ない場合のタイムアウト or フォールバック
- ネスト: 1つのトーク中に複数回の `get_property` 呼び出し
- イベントID衝突回避: ユニークID生成戦略（UUIDまたはカウンタ）

## スコープ
- **対象**:
  - 汎用 Pending Callback Registry
  - Tagged Yield メカニズム（特定イベント待ちyield）
  - Event Router の callback ルーティング拡張
  - `act:get_property(name)` — 単一プロパティ取得
  - `act:get_property(name1, name2, ...)` — 複数プロパティ一括取得
  - エラーハンドリング（コールバック未着時）
- **対象外**:
  - `act:set_property()` — `property-write-helpers` specの範囲
  - DSL構文 — `property-dsl-extension` の範囲
  - プロパティ値の型変換（文字列のまま返す）
  - `%property[name]` 環境変数展開

## 境界候補
- Pending Callback Registry（新規モジュール、`pasta_scripts/pasta/shiori/event/` 配下）
- EVENT.fire() のコールバックルーティング拡張（`event/init.lua`）
- act オブジェクトへの tagged yield / get_property メソッド追加
- コールバックイベントID生成ユーティリティ

## 対象外
- Rust側（pasta_shiori、pasta_lua src/）の変更（Lua層で完結させる想定）
- 通常の yield/resume フロー（`STORE.co_scene`）の破壊的変更
- SHIORIプロトコルレベルの拡張

## 上流 / 下流
- **上流**: 
  - 既存 yield/resume 基盤（`STORE.co_scene`、`resume_until_valid`、`set_co_scene`）
  - `EVENT.fire()` イベントディスパッチャー
  - `property-write-helpers`（set_propertyとの対称的なAPI設計）
- **下流**: 
  - `property-dsl-extension`（DSLトランスパイルのターゲットAPI）
  - 将来の `\![get,...]` 系タグ対応（非同期基盤の再利用）

## 既存Specとの接点
- **拡張**: `yield-continuation-token`（完了済み、yield/resume基盤の先行実装。Tagged Yieldはその拡張）
- **隣接**: `coroutine-resume-loop`（完了済み、`resume_until_valid` の実装。ルーティング拡張で変更対象）

## 制約
- 既存の yield/resume フロー（`STORE.co_scene` 継続トーク）を壊さないこと
- LuaJIT 2.1 のコルーチンモデル範囲内で実装
- SSP の `\![get,property,...]` プロトコル仕様に準拠
- テストは `lua_test` BDDフレームワークで記述（SHIORIリクエスト/レスポンスサイクルのモック含む）
