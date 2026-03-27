# Requirements Document

## Project Description (Input)
## ハンドラー解決フォールバックの統一
以下のハンドラー解決ロジックを経路別に整理し、主関数の経路を統一せよ。

### 入口対象
| 種別            |act:XX()                |actor_proxy:XX()       |
|:================|:=======================|:======================|
| シーン解決　　　|`ACT_IMPL.find_scene()` |                       |
| ワード取得　　　|`ACT_IMPL.word()`       |`PROXY_IMPL.word()`    |
| expr関数呼び出し|`ACT_IMPL.expr_fn()`    |`PROXY_IMPL.expr_fn()` |

### expr関数呼び出し
「expr関数呼び出し」は今回新規に作成してもらう。
ローカル関数呼び出しのトランスパイルを変更。
「さくら：＠XX（...）」⇒「proxy:expr_fn("XX", ...)」
「$=＠XX（...）」⇒「act:expr_fn("XX",...)」

### 共通ハンドラー検索処理（ハンドラー`XX`を解決するルール）
ハンドラー解決は、共通関数`find_handler()`を用意し、モードにより判定を調整する。

#### 検索関数宣言（要件案）
ここで、mode = "scene" or "word" or "expr"

```lua
function PROXY_IMPL.find_handler(proxy, mode, key)
    do
        local h = actor:find_actor_handler(mode, key)
        if h then return h end
    end
    return act:find_act_handler(mode, key)
end

function ACT_IMPL.find_handler(act, mode, key)
    return act:find_act_handler(mode, key)
end
```

#### コア検索関数の宣言
```lua
function PROXY_IMPL.find_actor_handler(proxy, mode, key)

function ACT_IMPL.find_act_handler(act, scene, mode, key)


```

### `key = 'XX'`が与えられたときのフォールバック戦略

以下のフォールバックを順番に検索し、最初にマッチした要素で確定する。

#### 1. （アクタープロキシなら）
1. `proxy.actor.XX`があれば確定
2. `mode= 'word'`なら、アクター単語辞書でXXがマッチすれば確定

#### 2. ローカルシーン
1. `scene.XX`があれば確定
2. `mode= 'word'`なら、ローカル単語辞書でXXがマッチすれば確定
3. `mode= 'scene' or 'expr'`なら、ローカルシーン辞書でXXがマッチすれば確定

#### 3. グローバルシーン
1. `GLOBAL.XX`があれば確定
2. `mode= 'word'`なら、グローバル単語辞書でXXがマッチすれば確定
3. `mode= 'scene' or 'expr'`なら、ローカルシーン辞書でXXがマッチすれば確定

#### 4. いずれもマッチしなければnil

### ワード辞書：ハンドラー`h`取得後の処理
+ nil ⇒ keyが見つからないエラーログだけ出力し、なにもせずreturn
+ function ⇒ `return h(proxy or act)`。
+ その他 ⇒ `return tostring(h)`

### シーン辞書：ハンドラー`h`取得後の処理
+ function ⇒ コルーチン化
+ その他 ⇒ keyが見つからないエラーログだけ出力し、なにもせずreturn

### expr_fn：ハンドラー`h`取得後の処理
+ function ⇒ `return h(proxy or act, ...)`。
+ その他 ⇒ keyが見つからないエラーログだけ出力し、なにもせずreturn

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
