# ukadoc設定ファイル仕様書

## 概要

本ドキュメントは、伺かゴースト配布に必要な設定ファイル群の仕様をukadocから調査・整理したものである。
alpha04-sample-ghost（hello-pasta）の実装時に参照するリファレンスとして使用する。

**調査日**: 2026-01-31  
**情報源**: [ukadoc](https://ssp.shillest.net/ukadoc/manual/)

---

## ファイル構成概要

```
hello-pasta/
├── install.txt              # インストール設定（必須）
├── ghost/
│   └── master/
│       └── descript.txt     # ゴースト設定（必須）
└── shell/
    └── master/
        ├── descript.txt     # シェル設定（必須）
        ├── surfaces.txt     # サーフェス定義（推奨）
        ├── surface0.png     # sakura側デフォルト（必須）
        └── surface10.png    # kero側デフォルト（必須）
```

---

## 1. install.txt

**場所**: ゴーストルート直下  
**参照**: https://ssp.shillest.net/ukadoc/manual/descript_install.html

### 書式

カンマ区切りの `key,value` 形式。1行1設定。

### 必須フィールド

| キー | 説明 | 例 |
|------|------|-----|
| `charset` | 文字コード | `UTF-8` |
| `name` | ゴースト表示名（SSP上で表示） | `hello-pasta` |
| `type` | ファイルセット種別 | `ghost` |
| `directory` | インストール先ディレクトリ名 | `hello-pasta` |

### オプションフィールド

| キー | 説明 | 既定値 |
|------|------|--------|
| `accept` | 既存ゴーストへの上書き許可ID | なし |
| `refresh` | ネット更新用ファイルリスト対応 | なし |
| `refreshundeletemask` | ネット更新時の削除禁止マスク | なし |
| `*.directory` | 追加インストール先（shell等） | なし |

### 記述例

```
charset,UTF-8
name,hello-pasta
type,ghost
directory,hello-pasta
```

---

## 2. ghost/master/descript.txt

**場所**: `ghost/master/descript.txt`  
**参照**: https://ssp.shillest.net/ukadoc/manual/descript_ghost.html

### 書式

カンマ区切りの `key,value` 形式。1行1設定。

### 必須フィールド

| キー | 説明 | 例 |
|------|------|-----|
| `charset` | 文字コード | `UTF-8` |
| `type` | ファイルセット種別 | `ghost` |
| `name` | ゴーストID（内部識別子） | `hello-pasta` |
| `sakura.name` | 本体側キャラクタ名 | `Pasta` |
| `kero.name` | 相方側キャラクタ名 | `Lua` |
| `craftman` | 作者名（半角英数のみ） | `pasta-team` |
| `craftmanw` | 作者名（日本語可） | `pasta開発チーム` |

### 推奨フィールド

| キー | 説明 | 例 |
|------|------|-----|
| `shiori` | SHIORI DLLファイル名 | `pasta.dll` |
| `homeurl` | ネットワーク更新用URL | `https://example.com/` |

### その他のオプション

| キー | 説明 |
|------|------|
| `sakura.name2` | 本体側愛称 |
| `kero.name2` | 相方側愛称 |
| `icon` | アイコンファイル名 |
| `balloon.directory` | 推奨バルーン名 |
| `seriko.alignmenttodesktop` | デフォルト表示位置 (`top`/`bottom`/`free`) |

### 記述例

```
charset,UTF-8
type,ghost
name,hello-pasta
sakura.name,Pasta
kero.name,Lua
craftman,pasta-team
craftmanw,pasta開発チーム
shiori,pasta.dll
```

---

## 3. shell/master/descript.txt

**場所**: `shell/master/descript.txt`  
**参照**: https://ssp.shillest.net/ukadoc/manual/descript_shell.html

### 書式

カンマ区切りの `key,value` 形式。1行1設定。

### 必須フィールド

| キー | 説明 | 例 |
|------|------|-----|
| `charset` | 文字コード | `UTF-8` |
| `type` | ファイルセット種別（省略時SSPが自動判定） | `shell` |
| `name` | シェル名 | `master` |

### 推奨フィールド

| キー | 説明 | 例 |
|------|------|-----|
| `craftman` | 作者名（半角英数） | `pasta-team` |
| `craftmanw` | 作者名（日本語可） | `pasta開発チーム` |

### バルーン位置設定

| キー | 説明 | 既定値 |
|------|------|--------|
| `sakura.balloon.offsetx` | 本体側バルーンX座標オフセット | ghostのdescript |
| `sakura.balloon.offsety` | 本体側バルーンY座標オフセット | ghostのdescript |
| `kero.balloon.offsetx` | 相方側バルーンX座標オフセット | ghostのdescript |
| `kero.balloon.offsety` | 相方側バルーンY座標オフセット | ghostのdescript |
| `sakura.balloon.alignment` | バルーン表示位置 (`none`/`left`/`right`) | ghostのdescript |

### その他のオプション

| キー | 説明 |
|------|------|
| `homeurl` | シェル単体でのネットワーク更新URL |
| `menu,hidden` | シェル切り替えメニューから非表示 |
| `seriko.dpi` | 推奨DPI値 (96/120/144等) |
| `seriko.use_self_alpha` | 32bit PNGのアルファチャンネル使用 |

### 着せ替え関連（オプション）

| キー | 説明 |
|------|------|
| `sakura.bindgroup*.name` | 着せ替えカテゴリ・パーツ名定義 |
| `sakura.bindgroup*.default` | 着せ替えデフォルト状態 |
| `sakura.menuitem*` | 着せ替えメニュー配置 |

### 記述例

```
charset,UTF-8
type,shell
name,master
craftman,pasta-team
craftmanw,pasta開発チーム
sakura.balloon.offsetx,0
sakura.balloon.offsety,50
kero.balloon.offsetx,0
kero.balloon.offsety,30
```

---

## 4. shell/master/surfaces.txt

**場所**: `shell/master/surfaces.txt`  
**参照**: https://ssp.shillest.net/ukadoc/manual/descript_shell_surfaces.html

### 概要

シェルの各種定義を行うファイル。基本的に全て省略可能だが、当たり判定やアニメーション定義はここで行う。

### 書式

- 1行目に `charset` 設定可能
- `{` と `}` で囲まれたブレス内に設定を記述
- `//` 以降はコメント

### descriptブレス

```
descript
{
version,1
}
```

| キー | 説明 | 既定値 |
|------|------|--------|
| `version` | SERIKO定義バージョン (0=旧, 1=新) | 0 |
| `collision-sort` | 当たり判定ソート順 (`ascend`/`descend`/`none`) | none |
| `animation-sort` | アニメーションソート順 | descend |

### surfaceブレス

```
surface0
{
collision0,10,10,100,50,Head
collision1,10,60,100,150,Face
}
```

### 当たり判定定義

| 書式 | 説明 |
|------|------|
| `collision*,X1,Y1,X2,Y2,ID` | 矩形当たり判定 |
| `collisionex*,ID,rect,X1,Y1,X2,Y2` | 拡張矩形 |
| `collisionex*,ID,ellipse,X1,Y1,X2,Y2` | 楕円 |
| `collisionex*,ID,circle,CX,CY,R` | 真円 |
| `collisionex*,ID,polygon,X1,Y1,...` | 多角形 |

### アニメーション定義（SERIKO新定義）

```
surface0
{
// 瞬き
animation0.interval,rarely
animation0.pattern0,overlay,101,100,168,67
animation0.pattern1,overlay,-1,100,0,0
}
```

| キー | 説明 |
|------|------|
| `animation*.interval` | アニメーション開始タイミング |
| `animation*.pattern*` | 描画メソッド,サーフェスID,ウェイト,X,Y |

### アニメーションインターバル

| 値 | 説明 |
|----|------|
| `sometimes` | 毎秒1/2の確率 |
| `rarely` | 毎秒1/4の確率 |
| `random,N` | 毎秒1/Nの確率 |
| `periodic,N` | N秒間隔 |
| `always` | ループ再生 |
| `runonce` | サーフェス切替時1回 |
| `never` | 自動実行なし |
| `talk,N` | N文字喋るごとに再生 |
| `bind` | 着せ替え定義 |

### 描画メソッド

| 値 | 説明 |
|----|------|
| `overlay` | 単純重ね合わせ |
| `base` | ベースサーフェス置換 |
| `replace` | 部分置換（透過含む） |
| `add` | 着せ替えパーツ追加 |

### バルーンオフセット（サーフェス個別）

```
surface0
{
sakura.balloon.offsetx,80
sakura.balloon.offsety,-100
}
```

---

## 5. サーフェス画像

### 必須サーフェス

| ファイル | 説明 |
|---------|------|
| `surface0.png` | sakura側デフォルトサーフェス（必須） |
| `surface10.png` | kero側デフォルトサーフェス（必須、透明可） |

### 画像仕様

- **形式**: PNG推奨
- **透過色**: 左上1ピクセルの色が透過色として扱われる
- **透過方法**: 
  - 透過色方式（従来）
  - pnaファイル（アルファマスク）
  - 32bit PNGアルファチャンネル（`seriko.use_self_alpha,1` 必要）

### サーフェスID慣例

**sakura側 (0-9)**
| ID | 表情 |
|----|------|
| 0 | 素 |
| 1 | 照れ |
| 2 | 驚き |
| 3 | 不安 |
| 4 | 落ち込み |
| 5 | 微笑み |
| 6 | 目閉じ |
| 7 | 怒り |
| 8 | 冷笑 |
| 9 | 照れ怒り |

**kero側 (10-19)**
| ID | 表情 |
|----|------|
| 10 | 素 |
| 11 | 刮目 |

---

## 6. hello-pasta向け最小構成案

### 推奨構成

```
hello-pasta/
├── install.txt
├── ghost/
│   └── master/
│       ├── descript.txt
│       └── dict/
│           └── *.pasta    # pastaスクリプト
└── shell/
    └── master/
        ├── descript.txt
        ├── surfaces.txt
        ├── surface0.png   # sakura (Pasta)
        └── surface10.png  # kero (Lua) ※透明可
```

### 最小限の設定値案

#### install.txt
```
charset,UTF-8
name,hello-pasta
type,ghost
directory,hello-pasta
```

#### ghost/master/descript.txt
```
charset,UTF-8
type,ghost
name,hello-pasta
sakura.name,Pasta
kero.name,Lua
craftman,pasta-team
craftmanw,pasta開発チーム
shiori,pasta.dll
```

#### shell/master/descript.txt
```
charset,UTF-8
type,shell
name,master
craftman,pasta-team
craftmanw,pasta開発チーム
```

#### shell/master/surfaces.txt
```
charset,UTF-8

descript
{
version,1
}

surface0
{
// 最小構成：当たり判定なし
}

surface10
{
// kero側（透明サーフェス可）
}
```

---

## 決定が必要な項目

以下の項目は要件フェーズで確定が必要：

| 項目 | 候補 | 備考 |
|------|------|------|
| `sakura.name` | `Pasta` / `パスタ` | キャラクタ名 |
| `kero.name` | `Lua` / `ルア` / なし | 2人構成 or 1人構成 |
| `craftman` | `pasta-team` | 半角英数制限 |
| `craftmanw` | `pasta開発チーム` | 日本語可 |
| 当たり判定 | あり / なし | 最小構成ではなしも可 |
| アニメーション | あり / なし | 瞬き程度は推奨 |
| kero表示 | 表示 / 非表示(透明) | 1人構成の場合透明 |

---

## 参考リンク

- [install.txt](https://ssp.shillest.net/ukadoc/manual/descript_install.html)
- [ghost/descript.txt](https://ssp.shillest.net/ukadoc/manual/descript_ghost.html)
- [shell/descript.txt](https://ssp.shillest.net/ukadoc/manual/descript_shell.html)
- [surfaces.txt](https://ssp.shillest.net/ukadoc/manual/descript_shell_surfaces.html)
- [Sakura Script](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html)
- [SHIORI Event](https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html)
