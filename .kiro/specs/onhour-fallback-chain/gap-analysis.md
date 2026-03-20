# ギャップ分析: onhour-fallback-chain

## 1. 現状調査

### 変更対象ファイル

| ファイル | 役割 | 変更種別 |
|---------|------|---------|
| `pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | OnHour発火ロジック（`check_hour()`） | **主変更** |
| `pasta_sample_ghost/dist-src/ghost/master/dic/talk.pasta` | サンプルゴースト辞書 | シーン名変更+追加 |

### 変更不要のファイル

| ファイル | 理由 |
|---------|------|
| `pasta/act.lua` - `ACT_IMPL.find_scene()` | 5段階フォールバックは既存のまま利用。呼び出し側でフォールバック候補を切り替えるだけ |
| `pasta/scene.lua` - `SCENE.co_exec()` | `act:find_scene()` をラップするだけなので変更不要 |
| `pasta/shiori/act.lua` - `transfer_date_to_var()` | 既存の呼び出しタイミング（OnHour発火時）を維持 |
| `pasta/shiori/event/second_change.lua` | `dispatcher.dispatch()` を呼ぶだけ。変更不要 |
| `pasta/shiori/event/init.lua` | イベント発火フレームワーク。変更不要 |
| Rust側（`search/context.rs`, `scene_table.rs`） | シーン検索のRust実装。Lua側のフォールバック処理で完結 |

### 既存パターン・規約

- **`create_scene_thread(event_name, act)`**: 固定シーン名文字列を受け取り `SCENE.co_exec()` でコルーチン生成（test用mock injection対応済み）
- **`_set_scene_executor(executor)`**: テスト用に `scene_executor(event_name, act) → thread|nil` をモック差替え可能
- **`act:find_scene(key)`**: Level 5までの5段階フォールバック。**返り値は関数（実行しない）**
- **`SCENE.co_exec(act, name)`**: `find_scene` → コルーチン生成。**返り値はthread|nil**
- **`act.req.date.hour`**: 0〜23の整数値（`transfer_date_to_var` より前に利用可能）

## 2. 要件と既存資産の対応表（ギャップ分析）

| 要件 | 既存資産 | ギャップ |
|------|---------|---------|
| Req1: 4段階フォールバック | `create_scene_thread()` は単一シーン名のみ対応 | **Missing**: 複数候補名の逐次検索ロジックが未実装 |
| Req2: HH 0埋め2桁 | `act.req.date.hour` (0-23整数) が利用可能 | **Missing**: `string.format("%02d", hour)` の呼び出し箇所がない |
| Req3: `＄時１２` 互換 | `transfer_date_to_var()` は `check_hour()` 内で既に呼び出し済み | **Gap なし** |
| Req4: `＊OnHour` 後方互換 | 既存辞書は `＊OnHour` で定義。フォールバック候補4は `OnHourOther` | **設計判断**: `＊OnHour` → `＊OnHourOther` 移行 or `＊OnHour` をフォールバック4に含めるか |
| Req5: サンプル辞書更新 | `talk.pasta` に `＊OnHour` が3シーン定義済み | **Missing**: リネーム + 時刻別シーン追加 |

### Req4 に関する設計判断ポイント

現在の要件定義では候補4を `＊OnHourOther` としているが、既存辞書の `＊OnHour` との互換性に2つのアプローチがある:

- **案A**: 候補4を `OnHourOther` 固定 → 既存辞書は `＊OnHour` → `＊OnHourOther` へリネーム必須
- **案B**: 候補4と5として `OnHourOther` + `OnHour` を追加（5段階フォールバック）→ 既存辞書はそのまま動作

ユーザの要件は明確に「4段階」を指定しており、候補は `時報{HH}` / `OnHour{HH}` / `時報その他` / `OnHourOther` の4つ。**案Aが要件準拠**。既存辞書のリネームはサンプルゴースト更新（Req5）で対応。

## 3. 実装アプローチ

### Option A: `check_hour()` 内 `create_scene_thread()` の直接拡張（**推奨**）

**変更内容**: `check_hour()` 内で `create_scene_thread("OnHour", act)` を呼んでいる箇所を、4候補を順次 `act:find_scene()` で検索し、最初にヒットした関数をコルーチン化する処理に置き換え。

**変更箇所**:
1. `virtual_dispatcher.lua` の `check_hour()` 末尾（L109付近）
2. `create_scene_thread()` は汎用なので変更せず、`check_hour()` から直接 `SCENE.co_exec()` 相当の処理を呼ぶ、**もしくは** `create_scene_thread()` を候補リスト対応に拡張

**トレードオフ**:
- ✅ 変更箇所最小（1関数の末尾数行）
- ✅ 既存テストの `_set_scene_executor` モック構造との互換性維持が容易
- ✅ `act:find_scene()` の5段階検索がそのまま利用可能
- ❌ テスト用モック `scene_executor` との整合性を検討する必要あり

### Option B: 新規ヘルパー関数 `find_hour_scene()` を追加

**変更内容**: `virtual_dispatcher.lua` に `find_hour_scene(act, hour)` ヘルパーを追加。4候補名を生成して順次 `act:find_scene()` で検索。`check_hour()` から呼び出し。

**トレードオフ**:
- ✅ ロジック分離が明確
- ✅ テスト容易性向上（ヘルパー単体でテスト可能）
- ❌ 新関数追加（ただし同一ファイル内）

### Option C: 汎用フォールバックチェーン関数

**変更内容**: `create_scene_thread()` を汎用的な候補リスト版に拡張（`create_fallback_scene_thread(candidates, act)`）

**トレードオフ**:
- ✅ OnTalkなど将来の他イベントにも再利用可能
- ❌ 現時点ではOnHourのみの要件であり、YAGNI（過剰設計のリスク）

## 4. テスト影響分析

### 既存テスト

| テストファイル | 影響 |
|-------------|------|
| `tests/shiori/virtual_event_dispatch_test.rs` | `_set_scene_executor` モックで `OnHour` キーを使用。**フォールバック版に対応するには `act:find_scene()` が検索する名前が変わるため、モック構造の見直しが必要** |
| `tests/lua_specs/virtual_dispatcher_thread_test.lua` | 同上。`scene_executor(event_name)` のevent_nameが `"OnHour"` 固定 |
| `tests/lua_specs/global_fallback_integration_test.lua` | `GLOBAL.OnHour` をテスト。フォールバック候補4が `OnHourOther` になるため影響あり |
| `tests/lua_specs/second_change_thread_test.lua` | `dispatcher.dispatch()` 経由テスト。モック依存 |

### テスト方針

**重要**: 現在の `_set_scene_executor` モックは `create_scene_thread()` をバイパスする設計。フォールバックチェーンが `act:find_scene()` を直接呼ぶ場合、モック戦略の変更が必要：

- **案1**: `_set_scene_executor` を維持しつつ、フォールバックロジック自体は本物の `act:find_scene()` を呼ぶ → 統合テストで辞書付きテスト
- **案2**: フォールバック候補名生成ロジックの単体テスト + `_set_scene_executor` でthread返却のE2Eテスト

## 5. 複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **S** (1〜3日) | Lua1ファイルの1関数改修 + 辞書リネーム + テスト更新 |
| **リスク** | **Low** | 既存パターン（`act:find_scene`）の組み合わせ。新規技術要素なし |

### リスク要因
- テストモック構造の整合性（`_set_scene_executor` がバイパスする範囲と、フォールバック検索の実行範囲）
- `global_fallback_integration_test.lua` の `GLOBAL.OnHour` テストが `OnHourOther` への変更で影響

## 6. 設計フェーズへの推奨事項

1. **Option A or B** を選択（変更スコープ最小: `virtual_dispatcher.lua` の `check_hour()` 末尾のみ）
2. **テストモック戦略**: `_set_scene_executor` との整合性を設計で明確化
3. **Req4 後方互換**: 4段階固定（要件準拠）。既存 `＊OnHour` は `＊OnHourOther` にリネーム
4. **`act.req.date.hour`** が `check_hour()` 呼び出し時点で利用可能であることの確認（既存コードで `transfer_date_to_var` の直前に `act.req.date` は確実に存在）
