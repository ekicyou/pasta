# SHIORI Handlers リファレンス

SHIORI/3.0プロトコルにおけるイベントハンドリング機構の完全リファレンス。  
REGテーブルへのハンドラ登録、RESレスポンス生成、主要SHIORIイベント一覧、シーン関数フォールバック、仮想ディスパッチャ（OnTalk/OnHour自動発行）を網羅する。

---

## REG テーブル登録

`pasta.shiori.event.register`（REGテーブル）にイベントハンドラを登録する。

```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
```

### 登録パターン

```lua
REG.イベント名 = function(req)
    -- req: SHIORIリクエスト情報
    return RES.ok("\\h\\s[0]応答\\e")  -- または RES.no_content()
end
```

### req パラメータ

SHIORIリクエスト情報テーブル。

| フィールド         | 型          | 説明                                                     |
| ------------------ | ----------- | -------------------------------------------------------- |
| `req.id`           | string      | イベント名（`"OnBoot"` 等）                              |
| `req.reference[N]` | string\|nil | Referenceヘッダ（0始まりインデックス）。未送信時は `nil` |
| `req.date`         | table       | 日時情報（`req.date.unix` 等）                           |
| `req.status`       | string\|nil | ステータス（`"talking"` 等）                             |

**Reference パラメータへのアクセス**:

```lua
local ref0 = req.reference[0]  -- Reference0 の値
local ref1 = req.reference[1]  -- Reference1 の値
-- 存在しない Reference は nil
if req.reference[5] == nil then
    print("Reference5 は送信されていません")
end
```

---

## RES レスポンス生成

`pasta.shiori.res` — SHIORI/3.0 レスポンス文字列を生成する。

```lua
local RES = require("pasta.shiori.res")
```

### API一覧

| 関数                   | ステータス                | 説明                        |
| ---------------------- | ------------------------- | --------------------------- |
| `RES.ok(value)`        | 200 OK                    | 成功レスポンス + Value      |
| `RES.ok_with(headers)` | 200 OK                    | 成功レスポンス + 複数ヘッダ |
| `RES.no_content()`     | 204 No Content            | 空レスポンス                |
| `RES.err(message)`     | 500 Internal Server Error | エラーレスポンス            |

### 使用例

```lua
-- 基本的な応答（さくらスクリプト）
return RES.ok("\\h\\s[0]こんにちは\\e")

-- 空応答（イベントを処理したが表示なし）
return RES.no_content()

-- エラー応答
return RES.err("設定ファイルが見つかりません")

-- 複数ヘッダ付き応答
return RES.ok_with({
    Value = "\\h\\s[0]こんにちは\\e",
    Reference0 = "追加情報",
})
```

---

## 主要SHIORIイベント一覧

SSP（ベースウェア）から送信される主要イベント。

### 起動・終了系

#### OnFirstBoot — 初回起動

ゴーストが初めて起動されたとき、またはバニッシュから復帰したときに発火。

| Reference          | 型     | 説明                                        |
| ------------------ | ------ | ------------------------------------------- |
| `req.reference[0]` | string | バニッシュ復帰フラグ ("0": 初回, "1": 復帰) |

```lua
REG.OnFirstBoot = function(req)
    local is_vanish_return = req.reference[0] == "1"
    if is_vanish_return then
        return RES.ok("\\h\\s[0]帰ってきたわ。\\e")
    end
    return RES.ok("\\h\\s[0]はじめまして。\\e")
end
```

#### OnBoot — 通常起動

ゴーストが起動されるたびに発火。

| Reference          | 型     | 説明         |
| ------------------ | ------ | ------------ |
| `req.reference[0]` | string | シェル名     |
| `req.reference[6]` | string | シェルパス   |
| `req.reference[7]` | string | ゴーストパス |

```lua
REG.OnBoot = function(req)
    local shell_name = req.reference[0]
    return RES.ok("\\h\\s[0]起動しました。シェル: " .. (shell_name or "不明") .. "\\e")
end
```

#### OnClose — 終了

ゴーストが終了するときに発火。

| Reference          | 型     | 説明                             |
| ------------------ | ------ | -------------------------------- |
| `req.reference[0]` | string | 終了理由 ("user", "shutdown" 等) |

```lua
REG.OnClose = function(req)
    local reason = req.reference[0]
    if reason == "user" then
        return RES.ok("\\h\\s[0]またね。\\e")
    end
    return RES.ok("\\h\\s[0]終了します。\\e")
end
```

#### OnGhostChanged — ゴースト切り替え

他のゴーストに切り替わるときに発火。

| Reference          | 型     | 説明                 |
| ------------------ | ------ | -------------------- |
| `req.reference[0]` | string | 切り替え先ゴースト名 |
| `req.reference[1]` | string | 切り替え元ゴースト名 |

```lua
REG.OnGhostChanged = function(req)
    local to_ghost = req.reference[0]
    return RES.ok("\\h\\s[0]" .. (to_ghost or "別のゴースト") .. "に交代するわ。\\e")
end
```

### 選択肢系

#### OnChoiceSelectEx — 選択肢選択

ユーザーが `\q[表示,ID]` 選択肢をクリックしたときにSSPが発火する。
デフォルトハンドラが `pasta.shiori.event.choice_select` で自動登録されるため、通常はゴースト作者の明示的な登録は不要。

| Reference          | 型     | 説明                   |
| ------------------ | ------ | ---------------------- |
| `req.reference[0]` | string | 選択ID（`\q` の第2引数）|
| `req.reference[1]` | string | 表示テキスト           |

**デフォルト動作**（自動登録ハンドラ）:
1. `＊OnChoiceSelectEx` 明示シーンが存在すれば優先実行
2. `STORE.last_global_scene` をスコープとして `SCENE.search(選択ID)` で前方一致検索
3. マッチするシーンが見つかればコルーチンとして実行
4. 見つからなければ `nil`（204 No Content）

**カスタマイズ**:
```lua
REG.OnChoiceSelectEx = function(act)
    local choice_id = act.req.reference[0]
    -- カスタム処理
end
```

### マウス操作系

#### OnMouseDoubleClick — ダブルクリック

キャラクターをダブルクリックしたときに発火。

| Reference          | 型     | 説明                              |
| ------------------ | ------ | --------------------------------- |
| `req.reference[0]` | string | スコープ ("0": sakura, "1": kero) |
| `req.reference[4]` | string | 当たり判定 ID                     |

```lua
REG.OnMouseDoubleClick = function(req)
    local scope = req.reference[0]
    local hit_area = req.reference[4]
    if scope == "0" then
        return RES.ok("\\h\\s[0]なあに？\\e")
    else
        return RES.ok("\\u\\s[0]呼んだ？\\e")
    end
end
```

### 時間系

#### OnSecondChange — 毎秒

毎秒発火する（高頻度）。仮想ディスパッチャのトリガーとして使用される。  
**非同期コールバック保留中のコルーチン再開もこのイベントで行われる**（`CALLBACK.resume_pending()`）。

| Reference          | 型     | 説明          |
| ------------------ | ------ | ------------- |
| `req.reference[0]` | string | 現在秒 (0-59) |
| `req.reference[1]` | string | 累積秒        |

```lua
REG.OnSecondChange = function(req)
    return RES.no_content()
end
```

#### OnNotifyCallbackResponse — SSPコールバック応答

SSP からのコールバック応答（`get_property` 等の結果）を受信するイベント。  
ゴースト作者が直接ハンドラを登録する必要はなく、フレームワーク内部の `pasta.shiori.callback` モジュールが自動処理する。

| Reference          | 型     | 説明                     |
| ------------------ | ------ | ------------------------ |
| `req.reference[0]` | string | コールバック ID          |
| `req.reference[1]` | string | 結果値（プロパティ値等） |

> **注意**: このイベントのハンドラは `pasta.shiori.event.init` で自動登録される。  
> ゴースト作者が REG に独自ハンドラを登録した場合、コールバック機構が動作しなくなる。

#### OnMinuteChange — 毎分

毎分発火する。

| Reference          | 型     | 説明          |
| ------------------ | ------ | ------------- |
| `req.reference[0]` | string | 現在分 (0-59) |
| `req.reference[1]` | string | 現在時 (0-23) |

```lua
REG.OnMinuteChange = function(req)
    local minute = req.reference[0]
    local hour = req.reference[1]
    if minute == "0" then
        return RES.ok("\\h\\s[0]" .. hour .. "時よ。\\e")
    end
    return RES.no_content()
end
```

---

## シーン関数フォールバック

REGテーブルにハンドラが登録されていない場合、`SCENE.search` でグローバルシーンを検索する。

### フォールバックチェーン

```
EVENT.fire(req)
  ↓
REG[req.id] 存在？
  ├─ Yes → ハンドラ実行 → レスポンス返却
  └─ No  → EVENT.no_entry(req)
              ↓
           SCENE.search(req.id)
              ├─ 見つかった → シーン関数実行 → 204 No Content
              └─ 見つからない → 204 No Content
```

### DSLシーンとの連携

Pasta DSLで定義したシーンは、イベント名と同じグローバルシーン名で検索される:

```
＊OnBoot
こんにちは。
```

上記シーンは `OnBoot` イベントでREGハンドラがない場合に自動的に呼び出される。

### エラーハンドリング

- REGハンドラ実行時の例外は `xpcall` でキャッチされ、`RES.err()` でエラーレスポンスが生成
- シーン関数フォールバック時も `pcall` でエラーをキャッチ

```lua
REG.OnBoot = function(req)
    error("何かがおかしい")
    -- → SHIORI/3.0 500 Internal Server Error
end
```

---

## 仮想ディスパッチャ

`pasta.shiori.event.virtual_dispatcher` — OnSecondChangeをトリガーとしてOnTalk/OnHour仮想イベントを自動発行する。

```lua
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
```

### dispatch(act)

メインエントリポイント。OnSecondChangeリクエストからOnHour/OnTalkイベントを判定・発行。

```lua
--- @param act ShioriAct actオブジェクト
--- @return thread|nil コルーチンまたはnil
local result = dispatcher.dispatch(act)
```

- Statusブロックガードは `dispatch()` 入口で一括判定される。`M.is_blocked(act.req.status)` が `true` を返した場合、即座に `nil` を返して発行をブロックする
- OnHourを優先判定し、発火しなければOnTalkを判定
- `act.req.date` フィールドがない場合は `nil` を返却

#### ブロック対象 Status キーワード

以下のSSP Statusキーワードが `act.req.status` に含まれている場合、`dispatch()` はイベント発行をブロックする:

| キーワード     | 意味                           | 対応SSP状態               |
| -------------- | ------------------------------ | ------------------------- |
| `talking`      | トーク中                       | さくらスクリプト実行中    |
| `choosing`     | 選択肢表示中                   | `\q` 選択肢待ち           |
| `online`       | ネットワーク通信中             | 更新チェック等            |
| `opening`      | 入力ボックス等が開いている     | `opening(communicate)` 等 |
| `passive`      | パッシブモード中               | 他ゴーストから制御中      |
| `induction`    | インダクションモード中         | 他ゴーストを呼び出し中    |
| `timecritical` | タイムクリティカルセクション中 | `\![set,timecritical]`    |
| `nouserbreak`  | ユーザーブレイク禁止中         | `\![set,nouserbreak]`     |
| `minimizing`   | 最小化中                       | バルーン非表示            |

### is_blocked(status)

SSP Status文字列にブロック対象キーワードが含まれるか判定する汎用公開関数。`dispatch()` で内部使用されるほか、他イベントハンドラからも再利用可能。

```lua
--- @param status string|nil act.req.status値
--- @return boolean true=発行ブロック, false=発行許可
local blocked = dispatcher.is_blocked(status)
```

**使用例**（他イベントハンドラからの利用）:

```lua
-- 撫で反応イベントハンドラの例
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---@param act ShioriAct
---@return thread|nil
local function handle_touch(act)
    if dispatcher.is_blocked(act.req.status) then return nil end
    -- ... 撫で反応処理
end
```

### OnHour — 時報自動発行（4段階フォールバックチェーン）

#### check_hour(act)

```lua
--- @param act ShioriAct actオブジェクト
--- @return thread|nil コルーチンまたはnil
local result = dispatcher.check_hour(act)
```

- 初回呼び出し時は次の正時を計算してスキップ
- 正時超過時に4段階フォールバックチェーンでシーンを解決:
  1. `時報{HH}` — 時刻別シーン（例: `時報12` で正午専用）
  2. `OnHour{HH}` — 時刻別英語シーン（例: `OnHour12`）
  3. `時報その他` — 汎用時報シーン
  4. `OnHourOther` — 汎用英語時報シーン
- `{HH}` は `act.req.date.hour` の0埋め2桁（00〜23）
- 最初にハンドラが見つかった候補で即リターン（早期打ち切り）
- 全候補未発見の場合は `nil` を返す
- **注意**: 旧シーン名 `OnHour` はフォールバック候補に含まれない（前方一致バグ回避）

### OnTalk — ランダムトーク自動発行

#### check_talk(act)

```lua
--- @param act ShioriAct actオブジェクト
--- @return thread|nil コルーチンまたはnil
local result = dispatcher.check_talk(act)
```

### pasta.toml設定

`[ghost]` セクションで仮想ディスパッチャの動作を設定。

| 設定                | 型      | デフォルト | 説明                         |
| ------------------- | ------- | ---------- | ---------------------------- |
| `talk_interval_min` | integer | 180        | 最小トーク間隔（秒）         |
| `talk_interval_max` | integer | 300        | 最大トーク間隔（秒）         |
| `hour_margin`       | integer | 30         | 時報前スキップマージン（秒） |

```toml
[ghost]
talk_interval_min = 180
talk_interval_max = 300
hour_margin = 30
```

- 時報前マージン内の場合はランダムトークをスキップし、時報を優先
- **セッション定義**: SHIORI load 〜 unload 間。unload時にLua VMごとドロップされ、モジュールローカル変数は自動リセット

### テスト用関数

```lua
-- 状態リセット（セッション開始時相当）
dispatcher._reset()

-- 内部状態取得
local state = dispatcher._get_internal_state()
-- { next_hour_unix, next_talk_time }

-- シーン実行関数のモック差し替え
dispatcher._set_scene_executor(function(event_name)
    return "mocked_result"
end)
```

---

## 関連リファレンス

- [internal-modules.md](internal-modules.md) — ACTオブジェクトの全メソッド、SCENE.searchの詳細
- [runtime-api.md](runtime-api.md) — RESレスポンス生成で使用する `@pasta_sakura_script` の変換仕様
