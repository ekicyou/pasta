# リサーチ＆設計判断ログ

## サマリ
- **機能**: `onhour-fallback-chain`
- **ディスカバリ範囲**: Extension（既存システムの拡張）
- **主要発見事項**:
  1. `resolve_scene_id()` は `iter_prefix()` を使用しており、`OnHour` で検索すると `OnHour00`〜`OnHour23` および `OnHourOther` がすべてマッチする ─ 前方一致バグの根拠
  2. `act:find_scene()` の Level 2/5 が `SCENE.search()` → `resolve_scene_id()` を経由するため、フォールバックチェーンでは `act:find_scene()` を直接利用すべき
  3. `create_scene_thread()` は `scene_executor` モック注入に対応しているが、フォールバックチェーン実装後はこの関数をバイパスして直接 `act:find_scene()` + `coroutine.create()` を行う設計が最適

## リサーチログ

### 前方一致検索（prefix search）の動作確認

- **調査契機**: Req4 議論で「`OnHour` を候補5として追加すると前方一致バグが出る」との開発者指摘
- **調査対象**: `crates/pasta_core/src/registry/scene_table.rs` L157-250
- **発見**:
  - `resolve_scene_id()` は `self.prefix_index.iter_prefix(search_key.as_bytes())` を呼ぶ（L171）
  - `prefix_index` は `fast_radix_trie` ベースの RadixMap
  - `"OnHour"` で検索すると `"OnHour"`, `"OnHour00"`, `"OnHour12"`, `"OnHourOther"` すべてがマッチ
  - ランダム選択 + キャッシュ付き順次選択が後段で行われる（L199-250）
- **影響**: `OnHour` をフォールバック候補に含めると、`OnHour12` 等の時刻別シーンが `OnHour` の検索でもヒットし、意図しないシーンが選択される。**案A（`OnHour` を候補から除外）が正しい**

### `act:find_scene()` の5段階フォールバック

- **調査対象**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` L335-380
- **発見**:
  - Level 1: `current_scene[key]` → シーンローカル完全一致
  - Level 2: `SCENE.search(key, global_scene_name)` → `resolve_scene_id_unified()` → prefix search
  - Level 3: `GLOBAL[key]` → グローバル関数テーブル完全一致
  - Level 4: `self[key]` → actメソッド
  - Level 5: `SCENE.search(key, nil)` → `resolve_scene_id()` → prefix search
  - 戻り値: `function | nil`（実行しない）
- **影響**: OnHourフォールバックチェーンでは `act:find_scene()` を候補名ごとに呼べば、既存の5段階フォールバックがそのまま機能する

### `create_scene_thread()` と `_set_scene_executor` モック

- **調査対象**: `virtual_dispatcher.lua` L45-59, L195-198
- **発見**:
  - `create_scene_thread(event_name, act)` は単一シーン名を受け取り `SCENE.co_exec()` を呼ぶ
  - `scene_executor` が設定されていればそちらを優先（テスト用）
  - テスト側は `_set_scene_executor(fn)` で `fn(event_name, act) → thread|nil` のモック関数を注入
  - フォールバックチェーン化すると、`create_scene_thread()` を1回だけ呼ぶ現行パターンでは不十分
- **影響**: フォールバックチェーンは `create_scene_thread()` を使わず、直接 `act:find_scene()` を候補ごとに呼ぶ。テストモックは新しいフック `_set_hour_scene_resolver` で置き換える

### 既存テストパターン調査

- **調査対象**:
  - `tests/lua_specs/virtual_dispatcher_thread_test.lua` - `_set_scene_executor` モックで thread 返却をテスト
  - `tests/lua_specs/global_fallback_integration_test.lua` - `GLOBAL.OnHour` で `EVENT.fire` 経由テスト
  - `tests/shiori/virtual_event_dispatch_test.rs` - Rust 統合テストで OnHour 発火タイミングテスト
  - `tests/lua_specs/second_change_thread_test.lua` - dispatch() のスレッド透過テスト
- **発見**:
  - `virtual_dispatcher_thread_test.lua` の `check_hour()` テストでは、`scene_executor` が固定で `"OnHour"` の event_name を受け取る前提。フォールバック化で event_name が変わるため **テスト更新必須**
  - `global_fallback_integration_test.lua` は `GLOBAL.OnHour` を直接設定。フォールバック化で `GLOBAL.OnHourOther` に変更が必要
  - Rust統合テストは `_set_scene_executor` をLuaラッパー経由で呼んでおり、影響範囲は Lua側と同等
  - `second_change_thread_test.lua` は `dispatch()` 結果の透過テスト。`scene_executor` レベルのモックなので影響軽微

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク / 制限 | 備考 |
|-----------|------|------|-------------|------|
| **A: `check_hour()` 末尾の直接拡張** | `check_hour()` の末尾で4候補を順次 `act:find_scene()` で検索 | 変更箇所最小、既存構造を維持 | テストモック要再設計 | **推奨** |
| B: `find_hour_scene()` ヘルパー追加 | 新関数でフォールバックロジックを分離 | テスト容易性、ロジック分離 | 新関数追加は不要な複雑化 | ヘルパー1つでは過剰 |
| C: 汎用フォールバックチェーン関数 | `create_fallback_scene_thread(candidates, act)` | 将来再利用可能 | YAGNI、OnHourのみの要件 | 過剰設計 |

## 設計判断

### 判断: フォールバックチェーン実装方式

- **コンテキスト**: `check_hour()` が固定シーン名 `"OnHour"` で `create_scene_thread()` を呼ぶ現行実装を、4候補の逐次検索に変更する
- **検討した選択肢**:
  1. Option A: `check_hour()` 末尾を直接書き換え
  2. Option B: ヘルパー関数を追加
  3. Option C: 汎用フォールバック関数
- **採用**: **Option A** — `check_hour()` 末尾の `create_scene_thread("OnHour", act)` を、4候補ループに置き換え
- **根拠**: 変更箇所が1関数の末尾数行に限定される。OnHour以外の仮想イベント（OnTalk）には影響しない。YAGNI原則に従い、汎用化は行わない
- **トレードオフ**: ロジックが `check_hour()` に埋め込まれるが、4候補の逐次検索は十分シンプルで分離の必要がない
- **フォローアップ**: 将来 OnTalk にも同種のフォールバックが必要になった場合にヘルパー化を検討

### 判断: テストモック戦略

- **コンテキスト**: `_set_scene_executor` は `create_scene_thread()` をバイパスするが、フォールバックチェーンでは `create_scene_thread()` 自体を使わなくなる
- **検討した選択肢**:
  1. `_set_scene_executor` をフォールバック内部にも適用（event_name を各候補名で呼ぶ）
  2. 新しいフック `_set_hour_scene_resolver(resolver)` を追加し、`resolver(act, hour) → thread|nil` でフォールバック全体を差し替え
  3. `_set_scene_executor` を廃止し、辞書ベースの統合テストに移行
- **採用**: **選択肢1 — `_set_scene_executor` をフォールバックループ内で候補名ごとに呼ぶ**
- **根拠**: 既存テストの `_set_scene_executor` パターンを維持しつつ、フォールバック候補名が `event_name` として渡されるようにすれば、テスト側で候補名ごとの返却値を制御できる。新しいフック追加は不要
- **トレードオフ**: テスト側で候補名をハンドリングする必要があるが、既存パターンの延長で対応可能

### 判断: `OnHour` シーン名の非使用

- **コンテキスト**: 候補5として `OnHour` を追加する案が検討されたが、開発者により却下
- **根拠**: `resolve_scene_id()` の `iter_prefix()` が `"OnHour"` で `"OnHour00"`〜`"OnHour23"` および `"OnHourOther"` をすべてマッチさせるため、意図しないシーン解決が発生する
- **結論**: 4段階固定、`OnHour` は候補に含めない

## リスク＆緩和策
- **リスク1**: テストモック更新漏れ → `_set_scene_executor` のevent_name引数にフォールバック候補名が渡ることをテストで検証
- **リスク2**: `global_fallback_integration_test.lua` の `GLOBAL.OnHour` 設定が壊れる → `GLOBAL.OnHourOther` に変更。ただし GLOBAL テーブルの Level 3 検索は完全一致なので prefix 問題なし
- **リスク3**: サンプルゴースト辞書の `＊OnHour` がフォールバックで見つからなくなる → Req5 でリネーム対応（必須タスク）

## 参考資料
- `crates/pasta_core/src/registry/scene_table.rs` — prefix_index (`fast_radix_trie`) によるシーン解決
- `crates/pasta_lua/pasta_scripts/pasta/act.lua` L335-380 — `find_scene()` 5段階フォールバック
- `crates/pasta_lua/pasta_scripts/pasta/scene.lua` L150-210 — `SCENE.search()`, `SCENE.co_exec()`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` — 変更対象
