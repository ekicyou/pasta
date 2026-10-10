# hello-pasta の立ち絵 — 生成の記録

サンプルゴースト hello-pasta のシェル（`ghosts/hello-pasta/shell/master/`）に置いた立ち絵の、生成元・手順・費用の記録です。このディレクトリは配布物（`.nar`）に入りません。

- `reference-girl.png` — 女の子の設定画像（基本のポーズ・通常の顔。1024×1536 の透過 PNG）
- `reference-boy.png` — 男の子の設定画像（同上）

## 1. 出典とライセンス

（タスク 6.2 で記入）

## 2. 著作権の注記

（タスク 6.2 で記入）

## 3. 使ったサービスとモデル

（タスク 6.2 で記入）

## 4. 設定画像の作り方

### 女の子（`reference-girl.png`）

- 1024×1536 px・RGBA。設計の段階で生成・承認し、ffmpeg で可逆に再エンコードしてコミットしたもの（生成の request_id は `01a12595-936e-70f0-a9ce-3c2aaf6513e8`）。
- 表情の編集に渡した入力は、fal の CDN に残っていた生成時の原本 `https://v3b.fal.media/files/b/0aadd038/_pLeTaf91214k_iEuaW2g_rWw2gdHU.png`。コミットした `reference-girl.png` と画素（RGBA）が完全に一致することを、両方を raw RGBA に展開して `cmp` で確かめた（2026-10-10）。
- 不透明な部分のアルファは最大 254（255 の画素は無い）。背景はアルファ 0。
- 指示文の全文と参照画像は、設計の段階の記録としてタスク 6.2 で記入する。

## 5. 表情とポーズの編集

モデルは `openai/gpt-image-2.5/flare/edit`（fal.ai・MCP 経由）。呼んだ日付は 2026-10-10。seed は無い。

共通のパラメータ:

```text
image_urls    = [女の子の設定画像（4 節の URL）]
image_size    = {width: 1024, height: 1536}
quality       = high
background    = transparent
output_format = png
num_images    = 1
```

### 女の子の表情（8 種。`通常` は設定画像そのもの）

指示文（`{expression}` を差し替える。改行なしの 1 段落で渡した）:

```text
Change only the facial expression to {expression}. Keep everything else identical: hair, head outline, body, pose, colors, lighting, background. Do not move or resize the head. Keep all changes inside the face (eyes, eyebrows, mouth, cheeks); no marks outside the face.
```

| サーフェス | 表情 | `{expression}` | request_id | 採否 |
|---|---|---|---|---|
| 0 | 笑顔 | a big happy smile, eyes curved into happy arcs | `01a125d3-716d-73b3-81ba-c021d42f809d` | 採用 |
| 2 | 照れ | shy smile, pink blush on both cheeks, eyes glancing slightly aside | `01a125d3-71ad-75c0-9001-7afcce8b9ffe` | 採用 |
| 3 | 驚き | wide-open round eyes, raised eyebrows, small open mouth | `01a125d3-71a5-7ce3-bbe9-c419f8b6b6b6` | 採用 |
| 4 | 泣き | teary eyes with tears on the cheeks, downturned mouth | `01a125d3-71b0-7730-8b7a-6896c154e98a` | 不採用 |
| 4 | 泣き | （下の作り直しの指示文） | `01a125d7-4967-7332-95e6-a26b61c54d44` | 採用 |
| 5 | 困惑 | troubled slanted eyebrows, wavy mouth, a single sweat drop on the temple | `01a125d3-71aa-78b2-8644-6bf90697a48b` | 採用 |
| 6 | キラキラ | sparkling star highlights in the eyes, excited open smile | `01a125d3-71a5-7610-97c7-ecdba1b52e6d` | 採用 |
| 7 | 眠い | half-closed droopy eyes, small sleepy mouth | `01a125d3-71a8-7493-8a9f-03de353828e8` | 採用 |
| 8 | 怒り | angry eyebrows pulled down, puffed cheeks, frowning mouth | `01a125d3-71ad-75c0-9001-7ae6198adf4d` | 採用 |

泣きの 1 回目を不採用にした理由: 涙の粒が顎の輪郭（1024×1536 で y≈492〜497）まで流れ、顔の矩形の下端（496）と切り線（500）にかかって途中で切れるため。作り直しでは、涙が口の高さより上に収まるよう指示文を具体的にした。作り直しの指示文の全文:

```text
Change only the facial expression to teary eyes with big tears welling up at the lower eyelids and one short tear streak on each upper cheek, downturned mouth. The tears stay on the cheeks above the mouth line and do not reach the jaw line or the chin. Keep everything else identical: hair, head outline, body, pose, colors, lighting, background. Do not move or resize the head. Keep all changes inside the face (eyes, eyebrows, mouth, cheeks); no marks outside the face.
```

表情の編集の結果について分かったこと:

- 8 枚とも、背景が透明にならず、半透明の暗い色で返ってきた（アルファが全面で 9〜254。指示文の「background を保つ」が、設定画像の透明画素の下に残っている色を背景として再現させたと見られる）。顔の中はほぼ不透明（アルファ 246〜253）。このため表情の絵のアルファは使わず、色だけを顔の矩形に使う（6 節）。
- 頭の位置ずれ（設定画像に対する最良の平行移動。±8 px を総当たり。顔の中央を除いた髪の領域で、色の差の平均が最小になる移動量）: 照れだけ (dx=1, dy=0)、ほかの 7 枚と作り直した泣きは (0, 0)。6 px を超えたものは無い。照れは 1 px ずらして貼った。
- 色味: 設定画像より 1 チャンネルあたり 1〜5 階調だけ明るい。補正はしていない（貼り替えの境目は等倍でも 3 倍拡大でも見えなかった）。
- 困惑の汗の粒は前髪の上（1024×1536 で x≈625, y≈290〜320）に描かれた。顔の矩形の中に入っている。

### 女の子のポーズ（2 種。設計の段階の試作で生成したものをそのまま使った）

指示文（設定画像を入力に。`{POSE}` を差し替える）:

```text
Edit this character image. Change ONLY the pose of the arms: {POSE}. Keep everything else
exactly identical: the same character, same art style, same head, face, facial expression,
hair, braids, ribbons, dress, colors and line weight. Keep the head at exactly the same
position and size, keep the feet at exactly the same place and keep the character exactly the
same height. Hands must not touch or overlap the face or hair. Transparent background, no
shadow, no text.
```

| ポーズ | `{POSE}` | request_id | 使うサーフェス |
|---|---|---|---|
| 基本 | （設定画像そのもの） | — | 1・0・7・5 |
| びっくり | both arms are raised and spread outward at shoulder height with open palms facing the viewer, as if startled in surprise | `01a12596-a6a6-7dc1-8f6c-7cad30b2a98d` | 3・8 |
| わくわく | both hands are clasped together in front of her chest, fingers interlaced, elbows bent, an excited and bouncy 'can't wait' pose. Hands must stay below the chin | `01a12597-0fb7-7cc1-80a1-2c8ecdfcdeb4` | 6・2・4 |

ポーズの 2 枚は背景が正しく透明（アルファ 0）で、頭の位置ずれは、表情と同じ測り方で (0, 0)。切り線より上に手や指は出ていない。

### 男の子

（タスク 2.2 で記入）

## 6. 貼り合わせ

貼り合わせは縮小前（1024×1536）で行う。座標はすべて縮小前のもの。

### 女の子

層の一覧:

| 層 | 元 | 枚数 |
|---|---|---|
| 頭（通常の顔） | `reference-girl.png` の切り線より上 | 1 |
| 顔の矩形 | 表情の編集の結果（8 種）から、矩形の中の色だけ | 8 |
| 体 | 基本 = `reference-girl.png`、びっくり・わくわく = ポーズの編集の結果 | 3 |

定数:

| 定数 | 値 | 意味 |
|---|---|---|
| `Y_CUT` | 500 | 頭と体の切り線（顎の下・襟の上） |
| `FEATHER` | 12 | ぼかし帯の幅。切り線では 494〜506 行で頭と体を混ぜる |
| 顔の矩形 | x 352〜663・y 216〜495（左上 (352, 216)・右下 (664, 496) の手前まで。幅 312・高さ 280） | 9 枚で共通 |

顔の矩形の決め方: 9 種の表情が設定画像と強く違う範囲（x 365〜650・y 230〜486。上端は驚き・キラキラの上がった眉、下端はキラキラの開いた口）に、ぼかし幅 12 px を足した。下端 496 は顎の線（y≈490）のすぐ下で、切り線 500 より上。

顔の矩形のぼかしは**内向き**に置いた。重みは矩形の縁で 0、縁から 12 px 内側で 1 になる。したがって矩形の外の画素は、表情によらず設定画像のまま変わらない（切り線のぼかし帯は、試作と同じく切り線を中心に上下 6 px）。顔の矩形の中では、アルファは設定画像のものを保ち、色だけをアルファ乗算で混ぜる。設定画像と表情の絵のどちらかが不透明でない画素（設定画像のアルファ 250 未満、または表情の絵のアルファ 240 未満）は貼り替えない。

出力（333×500）の座標では、顔の矩形は x 120〜209・y 125〜204（左上 (120, 125)・右下 (210, 205) の手前まで）。同じポーズの絵どうしで実際に画素が違った範囲の外接矩形は x 118〜210・y 123〜206（縮小フィルタが 2 px 外へにじむ）。体のポーズが基本と違い始めるのは 202 行目から（切り線は 205.6 行目）。

一回限りのスクリプト（Node。依存なし。入出力は ffmpeg で展開した raw RGBA。保守の対象ではなく記録）:

```js
// One-off compositing for the girl (surface0..8). Raw RGBA 1024x1536 in/out, no npm deps.
//   node composite.mjs <dir>
// in : ref.rgba (art/reference-girl.png), e0,e2,e3,e4b,e5,e6,e7,e8.rgba (expression edits),
//      pose-surprise.rgba, pose-clasp.rgba (pose edits)
// out: comp0.rgba .. comp8.rgba
import fs from 'node:fs';
const W = 1024, H = 1536, dir = process.argv[2];
const Y_CUT = 500, FEATHER = 12;
const RECT = { L: 352, T: 216, R: 664, B: 496 };            // face rectangle, R/B exclusive
// surface -> [expression layer, head shift (dx,dy) measured by analyze.mjs, body layer]
const SURFACES = {
  0: ['e0', 0, 0, 'ref'], 1: [null, 0, 0, 'ref'], 2: ['e2', 1, 0, 'pose-clasp'],
  3: ['e3', 0, 0, 'pose-surprise'], 4: ['e4b', 0, 0, 'pose-clasp'], 5: ['e5', 0, 0, 'ref'],
  6: ['e6', 0, 0, 'pose-clasp'], 7: ['e7', 0, 0, 'ref'], 8: ['e8', 0, 0, 'pose-surprise'],
};
const load = n => fs.readFileSync(`${dir}/${n}.rgba`);
const ref = load('ref');

for (const [id, [exp, dx, dy, bodyName]] of Object.entries(SURFACES)) {
  // 1) head layer = reference, with the face rectangle replaced by the expression's.
  //    The weight ramps from 0 at the rectangle edge to 1 at FEATHER px inside, so nothing
  //    outside RECT is ever touched. Alpha stays the reference's (the expression edits come back
  //    with a semi-opaque background, so their alpha is not usable); colour is blended
  //    alpha-premultiplied, and only where both layers are opaque.
  const head = Buffer.from(ref);
  if (exp) {
    const e = load(exp);
    for (let y = RECT.T; y < RECT.B; y++) for (let x = RECT.L; x < RECT.R; x++) {
      const i = (y * W + x) * 4, j = ((y + dy) * W + (x + dx)) * 4;
      if (ref[i + 3] < 250 || e[j + 3] < 240) continue;
      const d = Math.min(x - RECT.L, RECT.R - 1 - x, y - RECT.T, RECT.B - 1 - y) + 0.5;
      const w = Math.min(1, d / FEATHER);
      const aR = ref[i + 3] * (1 - w), aE = e[j + 3] * w;
      for (let c = 0; c < 3; c++) head[i + c] = Math.round((ref[i + c] * aR + e[j + c] * aE) / (aR + aE));
    }
  }
  // 2) body layer below Y_CUT, horizontal feather band (same as the prototype composite.mjs)
  const out = Buffer.from(head);
  if (bodyName !== 'ref') {
    const body = load(bodyName);
    for (let y = Y_CUT - FEATHER / 2; y < H; y++) {
      const t = Math.min(1, (y - (Y_CUT - FEATHER / 2)) / FEATHER);   // weight of the body layer
      for (let x = 0; x < W; x++) {
        const i = (y * W + x) * 4;
        if (t === 1) { out.set(body.subarray(i, i + 4), i); continue; }
        const aA = head[i + 3] / 255 * (1 - t), aB = body[i + 3] / 255 * t, a = aA + aB;
        for (let c = 0; c < 3; c++) out[i + c] = a > 0 ? Math.round((head[i + c] * aA + body[i + c] * aB) / a) : 0;
        out[i + 3] = Math.round(a * 255);
      }
    }
  }
  fs.writeFileSync(`${dir}/comp${id}.rgba`, out);
  console.log(`surface${id}: face=${exp ?? '(reference)'} shift=(${dx},${dy}) body=${bodyName}`);
}
```

`e4b` は泣きの作り直し。サーフェス 1（通常）は設定画像を何も変えずに縮小したもの。

### 男の子

（タスク 2.2 で記入）

## 7. 切り抜きと縮小・最適化

切り抜きはしていない（設定画像とポーズの絵は透過 PNG で出力された。表情の絵はアルファを使わない）。

raw RGBA への展開（貼り合わせの入力）:

```text
ffmpeg -i <入力>.png -f rawvideo -pix_fmt rgba <名前>.rgba
```

縮小（9 枚とも同じ。貼り合わせの出力 `compN.rgba` から。290×435 に縮めて 333×500 の下寄せ中央に置く）:

```text
ffmpeg -f rawvideo -pix_fmt rgba -s 1024x1536 -i compN.rgba \
  -vf "format=gbrap,premultiply=inplace=1,scale=290:435:flags=lanczos,unpremultiply=inplace=1,format=rgba,pad=333:500:21:64:color=0x00000000" \
  -frames:v 1 -compression_level 9 -pred mixed surfaceN.png
```

ffmpeg は ShareX 同梱の `C:\Program Files\ShareX\ffmpeg.exe`。縮小前の行 y は、出力の round(y × 0.2832) + 64 行目になる。

減色はしていない（9 枚とも 250 KB を下回った）。出力は RGBA 8 ビットの PNG で、テキストのチャンクは無い。

1 枚の大きさ（女の子）:

| ファイル | 表情 | ポーズ | バイト |
|---|---|---|---|
| `surface0.png` | 笑顔 | 基本 | 107,469 |
| `surface1.png` | 通常 | 基本 | 108,317 |
| `surface2.png` | 照れ | わくわく | 106,198 |
| `surface3.png` | 驚き | びっくり | 111,273 |
| `surface4.png` | 泣き | わくわく | 106,539 |
| `surface5.png` | 困惑 | 基本 | 108,667 |
| `surface6.png` | キラキラ | わくわく | 106,965 |
| `surface7.png` | 眠い | 基本 | 107,870 |
| `surface8.png` | 怒り | びっくり | 110,753 |
| 合計（9 枚） | | | 974,051（951.2 KB） |

## 8. 試作の実測

設計の段階の試作（女の子の 3 ポーズ）の実測は、タスク 6.2 で記入する。

### 女の子の 9 枚の検証（2026-10-10・出力した 333×500 の 9 枚に対する一回限りの比較）

- 9 枚とも 333×500・RGBA 8 ビット・四隅のアルファ 0・1 枚 104〜109 KB（106,198〜111,273 バイト）。
- (a) 出力の 0〜197 行目で、顔の矩形（出力の座標）を `MARGIN`=8 だけ広げた範囲（x 112〜217・y 117〜212）の外の画素が、9 枚すべてでバイト単位に一致した。
- (b) 同じポーズの絵どうし（基本 = 1・0・7・5、びっくり = 3・8、わくわく = 6・2・4）で、広げた顔の矩形の外の画素が、キャンバス全体で一致した。
- 頭の位置ずれ: 表情 8 種のうち 7 種が 0 px、照れが横に 1 px（1024 px 幅）。
- 目で見た確認（中間の灰色の背景に並べた一覧、顔と首の 3 倍拡大）: 9 種の表情が名前どおりに読めて互いに区別できる。顔の矩形の縁・色の段差、首・襟・おさげの継ぎ目、縁の色残り、透かし・文字は見えない。手は顔・髪に重なっていない。

### 男の子

（タスク 2.2 で記入）

## 9. 費用

| 段階 | 生成の回数 | 額 |
|---|---|---|
| 設計の段階（試作を含む） | （タスク 6.2 で記入） | （タスク 6.2 で記入） |
| 実装: 女の子の表情（タスク 2.1） | 9 回（表情 8 + 泣きの作り直し 1） | 約 0.51 ドル（推定） |
| 実装: 男の子（タスク 2.2） | （タスク 2.2 で記入） | （タスク 2.2 で記入） |

タスク 2.1 の額は推定。fal の MCP（`check_account_status`）は残高を返さないので、残高の差は読めなかった。回数 9 × 設計の段階の実測（10 枚で 0.57 ドル＝ 1 枚およそ 0.057 ドル）で計算した。ダッシュボードの値では確かめていない。

## 10. The Unlicense（原文）

（タスク 6.2 で記入）
