# ギャップ分析: shiori-async-talk

## 1. 現状調査

### 1.1 関連アセットとディレクトリ配置

| アセット             | パス                                            | 役割                                                          |
| -------------------- | ----------------------------------------------- | ------------------------------------------------------------- |
| イベントルーター     | `pasta_scripts/pasta/shiori/event/init.lua`     | `EVENT.fire(req)` — SHIORIイベントディスパッチ                |
| ハンドラ登録         | `pasta_scripts/pasta/shiori/event/register.lua` | `REG` テーブル（イベント→ハンドラのフラットマップ）           |
| act基底クラス        | `pasta_scripts/pasta/act.lua`                   | `ACT_IMPL` — トークン蓄積・build・yield                       |
| SHIORI act           | `pasta_scripts/pasta/shiori/act.lua`            | `SHIORI_ACT_IMPL` — set_property、さくらスクリプトbuild       |
| さくらスクリプト生成 | `pasta_scripts/pasta/shiori/sakura_builder.lua` | `BUILDER.build()` — トークン→さくらスクリプト変換             |
| レスポンス構築       | `pasta_scripts/pasta/shiori/response.lua`       | `RES.ok()`, `RES.no_content()` 等                             |
| シーンコルーチン     | `pasta_scripts/pasta/scene.lua`                 | `SCENE.co_exec()` — シーン実行用コルーチン生成                |
| グローバル定義       | `pasta_scripts/pasta/global.lua`                | `GLOBAL.yield`, `GLOBAL.チェイントーク`                       |
| 状態ストア           | `pasta_scripts/pasta/store.lua`                 | `STORE.co_scene` — 中断コルーチン保持                         |
| モックライブラリ     | `scriptlibs/lua_test/mocks.lua`                 | テスト用スタブ一括注入                                        |
| SHIORI Rustレイヤー  | `pasta_shiori/src/`                             | `parse_request()` → `call_lua_request()` → `SHIORI.request()` |
| テスト環境           | `pasta_shiori/tests/common/test_env.rs`         | `ShioriTestEnv` — 統合テスト用環境                            |

### 1.2 既存アーキテクチャパターン

#### イベントディスパッチフロー
```
Rust: SHIORI request テキスト
  → parse_request() → Lua テーブル {id, method, reference[], ...}
  → call_lua_request() → SHIORI.request(req)
  → EVENT.fire(req)
    → create_act(req) → SHIORI_ACT インスタンス
    → REG[req.id] でハンドラ検索
    → ハンドラ結果型に応じた処理:
      - thread → resume_until_valid(co, act) → set_co_scene(co) → RES.ok(value)
      - string → RES.ok(value)
      - nil → RES.no_content()
```

#### yield/resumeフロー（現行）
```
シーン関数 → act:yield()
  → act:build() → グループ化トークン配列
  → coroutine.yield(result)
  → EVENT.fire が result を受け取り RES.ok(result) で返却
  → STORE.co_scene = co（suspended状態のみ）

次イベント → EVENT.fire(req)
  → STORE.co_scene が存在 + 通常チェーントーク条件
  → resume_until_valid(co) → 第2引数なし（actは渡さない）
  → coroutine.yield() の戻り値は nil（引数なし resume）
  → act:yield() は return self → メソッドチェーン継続
```

#### コーディング規約
- **バリデーション**: `name == nil or name == ""` → `error()` パターン（set_propertyと同一）
- **エスケープ**: `escape_tag_arg()` — バックスラッシュ→%→]の順、カンマ/引用符含む場合はクォーティング
- **トークン蓄積**: `table.insert(self.token, {type="raw_script", text=tag})` → `return self`
- **テスト**: `lua_test` BDD（`describe`/`test`/`expect`）+ `mocks.install()` + `package.loaded` リセット

### 1.3 統合サーフェス

- **STORE.co_scene**: 単一スロット。新スレッド設定時に旧スレッドを `coroutine.close()` でクリーンアップ
- **resume_until_valid(co, ...)**: 第2引数以降は初回resumeのみ渡す。以降のresumeは引数なし
- **REG テーブル**: フラットな `{[event_id]: handler_fn}` マップ。コールバックルーティング機構なし
- **RES モジュール**: `RES.ok(dic)`, `RES.no_content()`, `RES.bad_request(reason)` 等

## 2. 要件実現可能性分析

### 2.1 要件→技術ニーズマッピング

| 要件                              | 技術ニーズ                                                                         | 既存アセット                                     | ギャップ                                                                                  |
| --------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| **Req 1: 単一プロパティ取得**     | `\![get,property,event,name]` タグ発行 + yield + コールバック受信 + resume(値付き) | `act:raw_script()`, `act:yield()` 部分的に利用可 | **Missing**: Tagged yield（特定イベント待ち）、コールバックルーティング、resume時の値渡し |
| **Req 2: 複数プロパティ一括取得** | `\![get,property,event,name1,name2,...]` タグ + 複数Reference値の抽出              | 同上                                             | **Missing**: 同上 + 複数Reference値の抽出・多値返却                                       |
| **Req 3: 入力バリデーション**     | `name` 引数チェック + コルーチンコンテキスト検証                                   | `set_property` のバリデーションパターン流用可    | **Missing**: コルーチンコンテキスト外検出（`coroutine.running()` ベース）                 |
| **Req 4: コールバック未着エラー** | タイムアウトまたはイベントカウンタ方式のフォールバック                             | なし                                             | **Missing**: 待機上限判定メカニズム                                                       |
| **Req 5: 既存フロー互換**         | 既存 `act:yield()` / `STORE.co_scene` / `resume_until_valid` の非破壊的拡張        | 既存コード全体                                   | **Constraint**: 拡張が既存動作を変えないことの保証が必要                                  |
| **Req 6: 汎用非同期基盤**         | プロパティ非依存のコールバック登録・ルーティング・resume API                       | なし                                             | **Missing**: 汎用コールバックレジストリモジュール                                         |

### 2.2 主要ギャップ詳細

#### Gap 1: resume時の値渡し不在

**現状**: `resume_until_valid(co)` は引数なしでresumeする。`coroutine.yield()` の戻り値は常に `nil`。

**影響**: コールバックで受け取ったプロパティ値をコルーチンに渡す手段がない。

**解決方向**:
- (a) `resume_until_valid` を拡張して値を渡せるようにする
- (b) コールバック専用のresumeパスを新設し、`resume_until_valid` を迂回する
- (c) クロージャ経由（act.var や STORE 経由）で値を渡す（yield戻り値は使わない）

**複雑度**: (a) は既存関数の引数変更で影響範囲が広い。(b) は新パスだが既存との一貫性維持が必要。(c) は最も安全だが API が不自然。

#### Gap 2: 特定イベント待ちyield（Tagged Yield）

**現状**: `act:yield()` → `coroutine.yield(result)` は「次の任意イベントで再開」セマンティクス。特定のイベントIDを指定して待機する仕組みはない。

**影響**: `get_property` はSSPが指定イベント名で発火するコールバックを待つ必要がある。

**解決方向**:
- yield時に「待機イベントID」をメタデータとして記録
- EVENT.fire がイベント受信時にメタデータを照合

#### Gap 3: コールバックルーティング

**現状**: `EVENT.fire(req)` は常に `REG[req.id]` を検索する。コールバックイベントが到着しても通常のREGディスパッチに流れる。

**影響**: `\![get,property,eventId,name]` のコールバック（`eventId` で発火）が、待機中コルーチンではなくREGハンドラに行ってしまう。

**解決方向**:
- `EVENT.fire` の冒頭で pending callback テーブルをチェック
- 該当イベントなら REG ディスパッチをバイパスし、待機中コルーチンを直接resume
- 非該当なら従来どおり REG ディスパッチ

#### Gap 4: ユニークイベントID生成

**現状**: イベントIDはSSP側が定義する固定文字列。ユーザー定義のユニークID生成ユーティリティはない。

**影響**: `\![get,property,eventId,name]` の `eventId` は呼び出しごとにユニークでなければ、複数の `get_property` コールが衝突する。

**解決方向**:
- カウンタベース（`__pasta_cb_1`, `__pasta_cb_2`, ...）— シンプル、衝突リスクなし（単一スレッド）
- UUID — 過剰（LuaJIT標準にUUID生成なし）

#### Gap 5: STORE.co_scene 単一スロット制約

**現状**: `STORE.co_scene` は1つのコルーチンのみ保持。新コルーチンが設定されると旧コルーチンは `coroutine.close()` で破棄。

**影響**: コールバック待ちのコルーチンが `STORE.co_scene` に入っている間に、通常のイベントが来たらどうなるか？

**解決方向**:
- コールバック待ちコルーチンは `STORE.co_scene` ではなく別のレジストリに保持
- `STORE.co_scene` は通常チェーントーク専用のまま維持
- これにより既存フローとの干渉を完全に回避

#### Gap 6: コールバック未着時のクリーンアップ

**現状**: 中断コルーチンのタイムアウト機構なし。

**影響**: SSPがコールバックを発火しない場合（プロトコルエラー、SSP非対応等）、コルーチンが永久に中断。

**解決方向**:
- イベントカウンタ方式: N回のイベント到着後にタイムアウト判定
- 時間ベース: `os.clock()` や `req.date` を使った経過時間判定
- 即時フォールバック: コールバック非該当イベント到着時に即座にnil返却（最も単純だが制約あり）

**Research Needed**: SSPが `\![get,property,...]` に対してコールバックを発火しないケースの仕様確認。無効なプロパティ名でもコールバックは来るのか？

### 2.3 複雑度シグナル

| 側面                 | 評価                                                               |
| -------------------- | ------------------------------------------------------------------ |
| アルゴリズム的複雑さ | 中 — コルーチン状態管理、イベントルーティング分岐                  |
| 統合面               | 中〜高 — EVENT.fire への割り込み、既存フロー非破壊保証             |
| テスト               | 中 — SHIORIリクエスト/レスポンスサイクルのモック（基盤は整備済み） |
| 外部依存             | 低 — SSPプロトコル仕様のみ（ドキュメント化済み）                   |

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張

**方針**: EVENT.fire、resume_until_valid、STORE を直接拡張

**変更対象**:
- `event/init.lua`: `fire()` 冒頭にコールバックチェック分岐追加、`resume_until_valid` に値渡しサポート追加
- `store.lua`: `STORE.pending_callbacks` テーブル追加
- `shiori/act.lua`: `get_property()` メソッド追加

**トレードオフ**:
- ✅ 新規ファイル最小（0〜1ファイル）
- ✅ 既存パターンに沿った自然な拡張
- ❌ `EVENT.fire` の複雑度が増大（すでに約40行、分岐が倍増）
- ❌ `resume_until_valid` の引数セマンティクス変更による影響範囲
- ❌ 将来の `\![get,...]` コンシューマ追加時に再度 EVENT.fire を変更する必要

### Option B: 新規コールバックモジュール

**方針**: `pasta_scripts/pasta/shiori/callback.lua` （新規）にコールバック管理を集約

**変更対象**:
- **新規**: `callback.lua` — Pending Callback Registry + ルーティング + タイムアウト + イベントID生成
- `event/init.lua`: `fire()` 冒頭で `CALLBACK.try_route(req)` を呼び出す1行追加のみ
- `shiori/act.lua`: `get_property()` メソッド追加（`CALLBACK` モジュールを使用）

**トレードオフ**:
- ✅ 単一責任: コールバック管理が独立モジュールに集約
- ✅ EVENT.fire への変更が最小（1行の分岐追加）
- ✅ 将来の `\![get,...]` コンシューマは CALLBACK API を使うだけ
- ✅ テスト容易性: コールバックモジュール単体テスト可能
- ❌ 新規ファイル追加（1ファイル）
- ❌ モジュール間の依存関係設計が必要

### Option C: ハイブリッド

**方針**: コールバックレジストリは新規モジュール、resume パスは既存拡張

**変更対象**:
- **新規**: `callback.lua` — レジストリのみ（ルーティングは EVENT.fire 内）
- `event/init.lua`: `fire()` 内にコールバックチェック + resume ロジック追加
- `shiori/act.lua`: `get_property()` メソッド追加

**トレードオフ**:
- ✅ レジストリの分離と既存フローへの自然な統合のバランス
- ❌ ルーティングロジックが EVENT.fire に残り、Option A の複雑度増大問題を部分的に引き継ぐ
- ❌ コールバック関連コードがモジュールとルーターに分散

## 4. 実装複雑度・リスク評価

### 工数: **M（3〜7日）**
- 新規パターン（コールバック待ちyield）だが、既存基盤が健全で拡張ポイントが明確
- テスト基盤（mocks.lua, lua_test BDD）が整備済みで、テスト作成のオーバーヘッドは低い
- `set_property` の実装パターンが対称的に流用可能（バリデーション、エスケープ、トークン蓄積）

### リスク: **中**
- **中リスク**: EVENT.fire への割り込み変更が既存チェーントークフローを壊す可能性
  - **緩和**: 既存の event_coroutine_test、integration_coroutine_test がリグレッション検出
- **中リスク**: resume時の値渡しパス変更がコルーチンライフサイクルに影響
  - **緩和**: コールバック用resumeパスを既存 `resume_until_valid` と分離することで影響を局所化
- **低リスク**: SSPプロトコルの `\![get,property,...]` 動作が想定と異なる可能性
  - **緩和**: SSP仕様はドキュメント化済み、既存の `set_property` 実績あり

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B（新規コールバックモジュール）** ← 採用確定

**確定事項**（開発者合意）:
- 汎用コールバック処理モジュール `pasta/shiori/callback.lua` を新設
- コールバックの登録・サスペンドレスポンス発行・コールバック検索・レジュームを集約管理
- EVENT.fire への変更は `create_act` より前でのコールバック照合分岐のみ（最小侵襲）
- コールバック待ちレスポンスには `\e` を付与しない（SSPがタグ処理後にコールバックを発火するため不要）
- コールバックイベントID命名: `OnPastaCallBack{N}` 形式（SSPの `On` プレフィックス慣例に準拠）

**理由**:
1. Req 6（汎用非同期基盤）の要件に最も自然にフィット — コールバック管理が独立モジュールとして再利用可能
2. EVENT.fire への変更が最小（1行の分岐追加）で、Req 5（既存互換性）のリスクが最も低い
3. コールバックモジュール単体のテストが容易で、テスト網羅度を上げやすい
4. `set_property` → `get_property` の対称的API設計と、コールバック基盤の分離が整理された責務分割

### 設計負荷の高い複合問題: yield 分岐問題

Gap 2（Tagged Yield）+ Gap 3（コールバックルーティング）+ Gap 5（STORE.co_scene 単一スロット）が絡み合う複合問題。Scenario 3（チェーントーク → コールバック遷移）で具体化される。

**核心**: 同一コルーチン内で yield の種別が遷移する場合（通常チェーントーク yield → コールバック待ち yield）、EVENT.fire の result_handler はどちらの yield かを判定し、適切なレジストリに振り分ける必要がある。

### 設計フェーズで決定すべき事項

1. **コールバックresumeのデータ渡し戦略**: `coroutine.resume(co, value)` vs クロージャ変数 vs act.var経由
2. **コールバック待ちコルーチンの保持場所**: 新テーブル（CALLBACK.pending）vs STORE拡張
3. **タイムアウト戦略**: イベントカウンタ vs 時間ベース vs 即時フォールバック（Req 4の具体化）
4. **ユニークイベントID体系**: カウンタ方式の名前空間（プレフィックス選定）
5. **EVENT.fire 内のコールバックチェックタイミング**: REGディスパッチ前 vs STORE.co_scene チェック前

### Research Needed（設計フェーズで調査）

- SSP `\![get,property,eventId,name1,name2,...]` の複数プロパティ指定時のReference配置仕様（Reference0, Reference1, ... の順序保証）
- SSP が無効なプロパティ名に対してコールバックを発火するか否か（エラー時のReference値）
- `coroutine.running()` がLuaJIT 2.1でメインスレッド判定に使えるか（Req 3 AC3のコンテキスト外検出）

## 6. 設計フェーズ決定事項（2026-05-26 追記）

### 6.1 採用アーキテクチャ: Option B 確定

`pasta/shiori/event/callback.lua` を新設し、コールバック管理を独立モジュールに集約。EVENT.fire への変更は 2 箇所の局所介入のみ。詳細は `design.md` 参照。

### 6.2 yield 分岐問題の解決: ステージング → 消費パターン

- `coroutine.yield()` / `resume_until_valid()` のシグネチャ・セマンティクスは非変更
- `get_property` は yield 直前に `CALLBACK.stage_pending(event_id, timeout_at, on_timeout)` を呼んで「コールバック待ちで yield する意図」を単一スロットに記録
- `EVENT.fire` は resume 直後に `CALLBACK.consume_staged(co, act)` を呼び、戻り値で「コールバック待ち（→ CALLBACK.pending に登録）」と「通常チェーントーク（→ STORE.co_scene に登録）」を分岐
- LuaJIT 単一スレッドモデルにより、ステージングと消費の間に他のコルーチン処理が割り込まないことが保証される

### 6.3 設計フェーズ決定事項マッピング

| 設計課題                  | 採用方針                                                                         |
| ------------------------- | -------------------------------------------------------------------------------- |
| データ渡し戦略            | `coroutine.resume(co, ref_array)` 標準セマンティクス                             |
| コールバック保持場所      | `callback.lua` のモジュール局所 `pending` テーブル（STORE 非汚染）               |
| タイムアウト戦略          | 時間ベース（`os.time()` 絶対時刻）、`OnSecondChange` で sweep                    |
| ユニーク ID 体系          | `OnPastaCallBack{N}` 形式、モジュール局所カウンタで単調増加                      |
| EVENT.fire 介入タイミング | (a) 冒頭 `try_route`（create_act より前） (b) resume 結果処理時 `consume_staged` |

### 6.4 Build vs Adopt 評価

- **コールバックレジストリ**: 既存ライブラリなし → 新規実装（Build）
- **タイムアウト sweep**: 既存 `OnSecondChange` イベントを再利用（Adopt）
- **エラーレスポンス生成**: 既存 `pasta.shiori.res` の 500 ビルダーを再利用（Adopt）
- **エスケープ処理**: 既存 `escape_tag_arg` を再利用（Adopt、set_property と共通）

### 6.5 Simplification 適用

- タイムアウトの 500 能動送信は SHIORI プロトコル上不可能なため断念。sweep 時に pending 削除 + コルーチン側エラー処理のみ実施し、「遅延したコールバックイベントが届いたとき」に 500 を返す経路に統一
- 多重ステージング検出は `error()` の早期失敗で対応（バグ検出用）
- ⨉ ~~options 引数は最後の table のみで判別~~ → 設計ディスカッション B2 で C1 位置引数方式に変更（options table 廃止）

### 6.6 残存リスク

| ID  | リスク                    | 緩和策                                                            |
| --- | ------------------------- | ----------------------------------------------------------------- |
| R1  | ステージング忘却          | `get_property` 内でバリデーション → ステージング → yield 順序厳守 |
| R2  | sweep 中の再ステージング  | 既存設計でカバー（新規 pending として登録）                       |
| R3  | 500 能動送信不可          | 遅延コールバック到着時の応答経路で代替                            |
| R4  | コールバックチェーン      | callback module 内で stage→consume→yield ループを完結             |
| R5  | LuaJIT メインスレッド判定 | `coroutine.running()` の `(co, is_main)` 戻り値で確認             |

### 6.7 設計フェーズで未解決（実装フェーズで検証）

- SSP 仕様: 無効プロパティ名でコールバックが発火するか / 空文字列 Reference の挙動 → 統合テスト（Scenario 4 拡張）で実機確認
- SSP 仕様: 複数プロパティ Reference 順序保証 → 統合テスト Req 3.1 検証で実機確認
