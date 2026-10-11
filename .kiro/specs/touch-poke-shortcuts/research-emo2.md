# 調査ノート: emo2 の撫で反応とつつき反応

2026-10-11 の `/kiro-discovery` で、ghost_dev の「emo2 開発」セッションに調査を頼み、受け取った内容の控えである。パスは、ghost_dev のリポジトリ（`C:\home\maz\git\ghost_dev`）からの相対である。`touch_detect.lua` と `touch.pasta` は、同じ日に元のファイルを直に読んで写した。ほかの抜粋は、受け取った報告のままである。

要約と、この spec での使い方は `brief.md` にある。ここには、実装の参考にするコードを置く。

## 標準と自前の切り分け

- emo2 の SHIORI は pasta.dll である。
- pasta の標準に乗っているのは 3 つだけである。イベントと同じ名前のシーン（`＊OnMouseMove`・`＊OnMouseDoubleClick`）、`act:transfer_req_to_var()`、`act:call` の前方一致。
- 撫での判定は、全部 emo2 の自前である（`scripts/touch_detect.lua`）。前身の emo-gs（自作の Lua 製 SHIORI）の `has_talk.touch` を移植した。
- 同じ `touch_detect.lua` が、`project/pasta-in-windows` にも複製されている（差分なし）。

## 撫での判定（emo2）

`project/emo2/ghost/emo2/ghost/master/scripts/touch_detect.lua`（全文）

```lua
-- touch_detect.lua - ストローキング検知モジュール
-- OnMouseMove の連続イベントから「撫でる」動作を検知する

local log = require "@pasta_log"
local M = {}

-- 定数
local INITIAL_DELAY = 2    -- 初期遅延（秒）
local DEBOUNCE_TIMEOUT = 2 -- デバウンスタイムアウト（秒）

-- 内部状態
local state = "OFF"        -- "OFF" | "COUNTING" | "FIRED"
local current_target = nil -- "region:actor" 形式
local start_time = 0       -- 蓄積開始時刻
local last_time = 0        -- 最終受信時刻

--- ストローキング判定
--- @param actor string アクター番号（"0" or "1"）
--- @param region string 当たり判定名（"Head", "Bust" 等）
--- @return boolean 発火すべきなら true
function M.check(actor, region)
    local now = os.time()
    local target = region .. ":" .. actor

    -- check() 呼び出し状態ログ (Req 2.1)
    log.trace({ fn = "check", state = state, target = current_target, elapsed = now - start_time })

    -- ターゲット変更 or デバウンスタイムアウト → リセット
    if target ~= current_target or (now - last_time) > DEBOUNCE_TIMEOUT then
        state = "OFF"
        -- leave / リセット理由ログ (Req 2.4)
        local reason = (target ~= current_target) and "target_change" or "timeout"
        if current_target ~= nil then
            log.info({ fn = "check", action = "leave", reason = reason, target = current_target })
        end
        current_target = target
        start_time = now
    end

    last_time = now

    if state == "OFF" then
        state = "COUNTING"
        -- OFF→COUNTING 遷移ログ (Req 2.2)
        log.info({ fn = "check", transition = "OFF->COUNTING", target = target })
        start_time = now
        return false
    elseif state == "COUNTING" then
        if (now - start_time) >= INITIAL_DELAY then
            state = "FIRED"
            -- COUNTING→FIRED 遷移ログ (Req 2.3)
            log.info({ fn = "check", transition = "COUNTING->FIRED", target = target, elapsed = now - start_time })
            return true
        end
        return false
    elseif state == "FIRED" then
        -- 発火済み → 次のリセットまで無視
        return false
    end

    return false
end

--- 状態リセット（テスト用）
function M.reset()
    state = "OFF"
    current_target = nil
    start_time = 0
    last_time = 0
end

return M
```

読み取れる挙動:

- 使うイベントは `OnMouseMove` だけ。`OnMouseWheel` は使わない。
- 同じ「部位:キャラ」の上で 2 秒以上続いたら、1 回だけ発火する。
- 部位かキャラが変わるか、イベントが 2 秒より長く途切れたら、リセットする。
- 発火した後は、リセットされるまで再発火しない。クールダウンの秒数は無い。
- トーク中の抑止は無い。

## 振り分け（emo2）

`project/emo2/ghost/emo2/ghost/master/dic/touch.pasta` の 7〜48 行

````pasta
＊OnMouseMove
　＞on_mouse_move
```lua
function SCENE.on_mouse_move(act)
    local save, var = act:init_scene(SCENE)
    act:transfer_req_to_var()
    local td = require("touch_detect")
    local log = require "@pasta_log"
    local actor = var.r3
    local region = var.r4

    -- 空regionはSSPがcollision領域外で送る正常値。ログ不要で即リターン
    if region == nil or region == "" then return nil end

    if actor == nil then
        log.warn({event="OnMouseMove", warn="nil_actor", region=region})
        return nil
    end

    log.trace({event="OnMouseMove", actor=actor, region=region})

    local fired = td.check(actor, region)
    if not fired then return nil end

    local key = region .. actor
    log.info({event="touch_fired", key=key})
    return act:call(SCENE.__global_name__, key, {})
end
```
　・Head0通常
　むらさき：＠笑顔　撫でて伸ばすタイプなんや～。
　　　エモ：＠冷笑　うそっぽ～い。

　・Head0照れ
　むらさき：＠静観　ん‥‥、\w9＠可愛いor小悪魔　なでなでしてくれるん？
　　　エモ：＠冷笑　ご主人が犬を撫でる感じ。
　むらさき：＠怒り　犬やないねん！

　・Head0受け流し
　むらさき：＠いたずら笑顔　撫でてくれるんや‥‥。\w5＠笑顔　ま、ええけど。
　　　エモ：＠通常　嬉しそう。
　むらさき：＠照れ怒り　ち゛が！
````

- キーは「部位＋キャラ番号」（`Head0`・`Bust1`）。
- ローカルシーンの名前は「キー＋変化の名前」（`Head0通常`・`Head0照れ`・`Head0受け流し`・`Head0掛け合い`）。`act:call(…, "Head0", {})` の前方一致で 4 つが候補になり、1 つが抽選される。
- 同じ形で `Bust0`・`Head1`・`Bust1` が続く（4 キー × 4 通りで 16 シーン）。
- 部位の名前は、シェルの当たり判定の名前そのままである（`project/emo2/ghost/emo2/shell/master/surfaces.txt` の `Head`・`Bust`）。

## つつき（emo2）

`project/emo2/ghost/emo2/ghost/master/dic/menu.pasta` の 10〜15 行

```pasta
＊OnMouseDoubleClick
　むらさき：＠通常　ダブルクリックしたね？メニューやで。
　＞メインメニュー選択肢

＊メインメニュー選択肢
　　　エモ：＠通常　\q[おしゃべり頻度,Onおしゃべり頻度メニュー]\n\q[エモの位置調整,Onエモの位置調整メニュー]\_l[5em,2lh]\q[閉じる,Onメニュー閉じる]
```

- ダブルクリックは、メニューを開く操作に全部使っている。Reference3（キャラ）・Reference4（部位）・Reference5（ボタン）は読まない。
- 部位つきのつつき反応の実例は無い。この spec が入った後も、emo2 のメニューは 7 段目（`＊OnMouseDoubleClick`）で今までどおり開く。

## フォールバックと除外（pasta-in-windows）

emo2 は 1 段だけである。部位が空なら、判定に入る前に 204 を返す。キーに合うシーンが無ければ、`act:call` が警告のログを出して nil を返し、204 になる。

pasta-in-windows は「代表の部位は個別、ほかの定義済みの部位は汎用 1 つ」の 2 段を、Lua の `if` で書いている。汎用の段に落とす前に、次の除外を入れている（`project/pasta-in-windows/ghost/pasta-in-windows/ghost/master/dic/touch.pasta` の Lua ブロックの先頭）。

```lua
    if region == nil or region == "" then return nil end
    -- 「閉じる」は窓chrome＝クローズ系(dic/window.pasta)へ委譲。タッチ反応しない（5.4 非競合）
    if region == "閉じる" then return nil end
    -- 「セキュリティボール」は前面オーバーレイの依代＝身体ではない。身体タッチ反応の
    -- 汎用フォールバック誤発火を防ぐため明示除外（ボール専用反応が要るなら個別分岐を新設）。
    if region == "セキュリティボール" then return nil end
    -- 単独キャラ。sakura(scope="0")以外は無反応（kero 由来の誤発火防止）
    if scope ~= "0" then return nil end
```

pasta-in-windows の部位の名前は日本語である（`頭`・`顔`・`胸右` など）。

## 移植元の emo-gs（トーク中の抑止があった版）

`project/sample-ghost/emo-gs/ghost/master/emo/shiori/has_talk.lua` の 54〜92 行

```lua
function M.touch(data, req)
    local x = utils.get_tree_entry(data, "save", "touch")
    local now           = os.time()
    local status_dic    = req.status_dic
    local ref           = req.reference
    local is_touch      = true
    --- 会話中はタッチ状態としない
    is_touch = is_touch and (not status_dic.talking)

    --- 対象が変わった場合はタッチ状態としない
    is_touch = is_touch and x.ref3 == ref[3]
    is_touch = is_touch and x.ref4 == ref[4]
    x.ref3 = ref[3]
    x.ref4 = ref[4]

    --- 最後のイベントから２秒経過したならタッチ状態としない
    is_touch = is_touch and x.time_event+2 > now
    x.time_event = now

    --- タッチ状態ではないときの処理
    if not is_touch then
        x.time_start    = now
        x.last_flag     = false
        return false
    end

    --- 時間経過を確認
    if not x.limit_sec then
        x.limit_sec = 3
    end
    if x.time_start+x.limit_sec > now then
        return false
    end

    --- タッチがOFF→ONになったときだけtrue
    local last_flag = x.last_flag
    x.last_flag = is_touch
    return last_flag ~= is_touch
end
```

- emo-gs は、限界 3 秒・途切れ 2 秒・会話中は抑止・状態は保存する、だった。
- emo2 は、2 秒・2 秒・抑止なし・保存なしに変わっている。
- 会話中のイベントを「撫でていない」と数えるので、会話が終わってから改めて 3 秒撫でる必要がある。

## はまりどころ（emo2 の経験）

1. **前方一致の衝突**: pasta のシーンの検索は、完全一致を優先しない。前方一致の候補をまとめて抽選する。`OnInstallComplete` と `OnInstallCompleteAll` で、約半数が誤って抽選された（ghost_dev `4a2ba33` で直した）。
2. **トーク中の抑止が無い**: トーク中に 2 秒撫でると、反応がトークを打ち切る。
3. **`os.time()` は秒単位**: しきい値が 2 秒でも、実際の発火は 2〜3 秒後になる（実機のログで `elapsed = 3` を確かめた）。しきい値を短くするほど、ぶれが目立つ。
4. **当たり判定の外へ出てもリセットされない**: 部位が空のイベントが判定に届かない。外へ出て 2 秒以内に同じ部位へ戻ると、発火済みのままになる。同じ部位を撫で続けても、2 回目は出ない。
5. **状態が 1 つ**: Lua のモジュールの中の変数で、保存されず、リロードで消える。2 人のキャラを行き来すると、毎回リセットされる。
6. **Lua ブロックの中の誤りが無言で 204 になる**: 以前、未定義の変数で反応が出ず、原因が分からないのでログを足した（ghost_dev の `.kiro/specs/completed/emo2-touch-debug`）。
7. **身体でない当たり判定**: シェルが「閉じる」ボタンなどにも当たり判定を付けていると、汎用のフォールバックがそこで鳴る。
8. **部位の名前はシェル任せ**: 英字と日本語の両方が来る。
9. **ダブルクリックの衝突**: emo2 はメニューに全部使っているので、つつき反応と同居させた実績が無い。
10. **SSP と areka の違い**: 撫で・つつきで届くイベントが同じかは、確かめていない。実機のログは SSP のものである。
