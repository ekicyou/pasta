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

### 男の子（`reference-boy.png`）

- 1024×1536 px・RGBA。設計の段階で 3 段階（体の釣り合いを決める → 帽子なしで描き直す → 帽子を載せる）で生成・承認し、ffmpeg で可逆に再エンコードしてコミットしたもの（設定画像になった最後の生成の request_id は `01a125a8-92e0-7e90-a326-ccba87020b0e`）。
- 表情とポーズの編集に渡した入力は、fal の CDN に残っていた生成時の原本 `https://v3b.fal.media/files/b/0aadd0b4/y33LSiAs-4uNANiZgVclN_tNukyhgS.png`。コミットした `reference-boy.png` と画素（RGBA）が完全に一致することを、両方を raw RGBA に展開して `cmp` で確かめた（2026-10-10）。
- 不透明な部分のアルファは最大 254（255 の画素は無い）。背景はほぼアルファ 0 で、アルファ 1〜15 の点が 28,344 画素残っている。不透明な範囲（アルファ 128 超）は x 294〜730・y 40〜1500。
- 指示文の全文と不採用の試行は、設計の段階の記録としてタスク 6.2 で記入する。

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

モデル・呼んだ日付・共通のパラメータは女の子と同じ（`image_urls` は男の子の設定画像＝4 節の URL）。seed は無い。

#### 男の子のポーズ（2 種。実装で生成した）

指示文（設定画像を入力に。`{POSE}` を差し替える。改行なしの 1 段落で渡した。女の子の指示文の `hair, braids, ribbons, dress` を `hair, chef's hat, neckerchief, jacket, apron` に読み替えたもの）:

```text
Edit this character image. Change ONLY the pose of the arms: {POSE}. Keep everything else exactly identical: the same character, same art style, same head, face, facial expression, hair, chef's hat, neckerchief, jacket, apron, colors and line weight. Keep the head at exactly the same position and size, keep the feet at exactly the same place and keep the character exactly the same height. Hands must not touch or overlap the face or hair. Transparent background, no shadow, no text.
```

| ポーズ | `{POSE}` | request_id | 採否 | 使うサーフェス |
|---|---|---|---|---|
| 基本 | （設定画像そのもの） | — | — | 11・10・17・14 |
| びっくり | both arms are raised and spread outward at shoulder height with open palms facing the viewer, as if startled in surprise | `01a125e6-65d7-7eb1-ad5f-7b7a4d5ddaed` | 不採用 | — |
| びっくり | both arms are raised and spread outward at shoulder height with open palms facing the viewer, as if startled in surprise. Hands must stay below the chin; even the highest fingertip is clearly lower than the collar of the jacket | `01a125ec-783a-7712-9231-0bc95ec03488` | 採用 | 13・16 |
| 腕組み | arms folded across the chest, an exasperated 'oh dear' pose | `01a125e7-246b-7331-902c-e48d1009eb89` | 採用 | 15・18・12 |

びっくりの 1 回目を不採用にした理由: 指先が y 543 まで上がり、切り線（顎の下。6 節）より上に出る。切り線より上は頭の層（設定画像）から取るので、指先が水平に欠ける。人物・服・頭の位置は合っていた。作り直しでは「手は顎より下、いちばん高い指先も襟より下」を足し、指先の上端が y 558〜559 に下がった。

ポーズの絵について測ったこと（採用した 2 枚。不採用の 1 枚も頭と背丈は同じ結果）:

- 背景は正しく透明（アルファ 0）。不透明な範囲の上端は 40・下端は 1501（設定画像は 40・1500）で、背丈と足元は 1 px 以内。出力（333×500）では 3 つのポーズとも不透明な範囲が y 75〜488（高さ 414 px）。
- 頭の位置ずれ（設定画像に対する最良の平行移動。±8 px を総当たり。顔の中央を除いた帽子・横の髪・耳の領域で、色の差の平均が最小になる移動量）: 2 枚とも (0, 0)。
- 切り線の付近（550〜562 行）で、首と襟の不透明な幅の左右の端は設定画像と 3 px 以内で一致した。
- 服・色・体つきは目で見て同じ。頭の領域の色の差の平均は 1 チャンネルあたり 1.6 階調以内。
- 腕組みの手は胸の前、びっくりの手は肩の横にあり、顔・髪・帽子に重なっていない。

#### 男の子の表情（8 種。`通常` は設定画像そのもの）

指示文は女の子と同じ（`{expression}` を差し替える。改行なしの 1 段落）。泣きは、女の子で作り直したときの指示文（涙が口の高さより上に収まる言い方）を最初から使った。

| サーフェス | 表情 | `{expression}` | request_id | 採否 |
|---|---|---|---|---|
| 10 | 笑顔 | a big happy smile, eyes curved into happy arcs | `01a125e7-76e5-7700-87af-5fb5d4e72eed` | 採用 |
| 12 | 照れ | shy smile, pink blush on both cheeks, eyes glancing slightly aside | `01a125e7-c9fe-72f3-8b1f-a92614650d2b` | 採用 |
| 13 | 驚き | wide-open round eyes, raised eyebrows, small open mouth | `01a125e8-3673-77d0-803d-530ff1691314` | 採用 |
| 14 | 泣き | （女の子の泣きの作り直しと同じ全文） | `01a125e8-913f-7190-8828-725a94c297db` | 採用 |
| 15 | 困惑 | troubled slanted eyebrows, wavy mouth, a single sweat drop on the temple | `01a125e8-f3d7-7280-aa00-d2a44f111b2c` | 採用 |
| 16 | キラキラ | sparkling star highlights in the eyes, excited open smile | `01a125e9-58e8-7922-947b-0ed3dfd1dda4` | 採用 |
| 17 | 眠い | half-closed droopy eyes, small sleepy mouth | `01a125e9-b72c-79f1-9061-9d451df8eeaf` | 採用 |
| 18 | 怒り | angry eyebrows pulled down, puffed cheeks, frowning mouth | `01a125ea-08f1-77f3-aec1-bb68cf7c0a86` | 採用 |

表情の編集の結果について分かったこと:

- 女の子と同じく、8 枚とも背景が透明にならず、半透明の暗い色で返ってきた（アルファは全面で最小 1〜21・最大 252〜253）。顔の中はほぼ不透明（アルファ 241〜253。眠いだけ、顎の縁にアルファが 181 まで下がる画素が 81 個あり、そこは貼り替えていない）。表情の絵のアルファは使わず、色だけを顔の矩形に使う（6 節）。
- 頭の位置ずれ（ポーズと同じ測り方）: 8 枚とも (0, 0)。平行移動はしていない。
- 色味: 設定画像との差の平均は 1 チャンネルあたり 1 階調未満。補正はしていない。
- 困惑の汗の粒は右の横髪の上（1024×1536 で x≈648〜666・y≈380〜415）に描かれた。帽子にはかかっていない。顔の矩形の中に入っている。
- 泣きの涙は目の下から頬の途中までで、顎の線に届いていない。
- 怒りは頬を膨らませたぶん、頬から顎の輪郭が設定画像より外へ出た。困惑・驚きも顎の線が少しずれた。輪郭の線は設定画像のものを保つようにした（6 節の「輪郭の保護」）。
- 驚き・怒りは前髪の線が少し描き変わった（y 298〜330 のあたり）。矩形の上端のぼかしの中でなじみ、境目は見えなかった。

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

層の一覧:

| 層 | 元 | 枚数 |
|---|---|---|
| 頭（通常の顔） | `reference-boy.png` の切り線より上 | 1 |
| 顔の矩形 | 表情の編集の結果（8 種）から、矩形の中の色だけ | 8 |
| 体 | 基本 = `reference-boy.png`、びっくり・腕組み = ポーズの編集の結果 | 3 |

定数:

| 定数 | 値 | 意味 |
|---|---|---|
| `Y_CUT` | 556 | 頭と体の切り線（顎の下・ネッカチーフの上） |
| `FEATHER` | 12 | ぼかし帯の幅。切り線では 550〜562 行で頭と体を混ぜる |
| 顔の矩形 | x 366〜679・y 322〜549（左上 (366, 322)・右下 (680, 550) の手前まで。幅 314・高さ 228） | 9 枚で共通 |
| `EDGE0`・`EDGE1` | 5・11 | 輪郭の保護。設定画像の輪郭（不透明でない画素）から 5 px 以内は貼り替えず、11 px 離れたら重み 1 |

切り線の決め方: 設定画像で、顎の線の下端は中央で y 552〜556、襟とネッカチーフの上端は y≈560。不透明な幅が首だけ（x 483〜542）になるのは 553〜559 行。その中の 556 を選んだ。558 でも合成して比べ、びっくりの指先（上端 y 558〜559）がぼかし帯で薄くなる量が少ないほうを採った（ポーズの絵だけを縮小したものと比べて、出力でアルファが違う手の画素は 558 で 15 個・最大 58/255、556 で 5 個・最大 26/255。6 倍に拡大しても見分けられなかった）。出力では切り線は 221.5 行目（round(556 × 0.2832) + 64 = 221）。

顔の矩形の決め方: 8 種の表情が設定画像と強く違う範囲（眉・目・口・汗の粒。x 380〜666・y 344〜537。前髪の線の描き変わりは数えない。左右は笑顔・眠い・怒りの目尻と困惑の汗の粒、上端は驚きの上がった眉、下端はキラキラの開いた口）に、左右はぼかし幅 12 px と 1〜2 px、上は 22 px を足した。帽子（帯の下端は矩形の幅の中で y 293〜317）は矩形の外にある。前髪は眉と目に重なるので矩形に入る。下端 550 は切り線のぼかし帯の上端と同じ行で、矩形と帯は重ならない（重み 1 で貼るのは 537 行目まで）。

貼り替えの方式は女の子と同じ（アルファは設定画像のまま・色だけをアルファ乗算で混ぜる・両方が不透明な画素だけ・ぼかしは矩形の内向き）。矩形の外の画素は、表情によらず設定画像のまま変わらない。

輪郭の保護（男の子で足した）: 男の子は顎の輪郭が矩形の中を通り、表情の絵の顎が設定画像と少しずれる。女の子と同じ重みのままで合成すると、設定画像の輪郭線（暗い画素）の上に表情の絵の肌色が載って、輪郭線が欠けた（矩形の中の顎の輪郭線 428 画素のうち、明るくなった画素が怒り 198・困惑 74・驚き 72・照れ 58・笑顔 47・キラキラ 29・眠い 18・泣き 15）。そこで、設定画像の不透明でない画素からの距離が 5 px 以内の画素は貼り替えず、11 px 離れたところで重み 1 になるようにし、矩形のぼかしの重みと小さいほうを使った。保護したあとは 8 種とも 0 画素。重みを下げるだけなので、矩形の外が変わらない性質はそのまま。怒りの膨らんだ頬は、耳に重なる上のほう（輪郭の内側）に残り、顎の輪郭は設定画像のまま。

出力（333×500）の座標では、顔の矩形は x 124〜213・y 155〜219（左上 (124, 155)・右下 (214, 220) の手前まで）。同じポーズの絵どうしで実際に画素が違った範囲の外接矩形は x 122〜215・y 153〜221。体のポーズが基本と違い始めるのは 217 行目から（頭を通常の顔にそろえ、体だけを替えた合成どうしの比較。切り線は 221.5 行目）。

一回限りのスクリプト（Node。依存なし。入出力は ffmpeg で展開した raw RGBA。保守の対象ではなく記録）:

```js
// One-off compositing for the boy (surface10..18). Raw RGBA 1024x1536 in/out, no npm deps.
//   node composite.mjs <dir>
// in : ref.rgba (art/reference-boy.png), e10,e12,e13,e14,e15,e16,e17,e18.rgba (expression edits),
//      pose-surprise2.rgba, pose-fold.rgba (pose edits)
// out: comp10.rgba .. comp18.rgba
import fs from 'node:fs';
const W = 1024, H = 1536, dir = process.argv[2];
const Y_CUT = 556, FEATHER = 12;
const RECT = { L: 366, T: 322, R: 680, B: 550 };            // face rectangle, R/B exclusive
// surface -> [expression layer, head shift (dx,dy) measured by analyze.mjs, body layer]
const SURFACES = {
  10: ['e10', 0, 0, 'ref'], 11: [null, 0, 0, 'ref'], 12: ['e12', 0, 0, 'pose-fold'],
  13: ['e13', 0, 0, 'pose-surprise2'], 14: ['e14', 0, 0, 'ref'], 15: ['e15', 0, 0, 'pose-fold'],
  16: ['e16', 0, 0, 'pose-surprise2'], 17: ['e17', 0, 0, 'ref'], 18: ['e18', 0, 0, 'pose-fold'],
};
const load = n => fs.readFileSync(`${dir}/${n}.rgba`);
const ref = load('ref');
// distance (px, capped at EDGE1) from each face-rectangle pixel to the nearest non-opaque reference pixel:
// the silhouette outline of the jaw stays the reference's, so an expression whose jaw is 1-2 px off
// (or puffed cheeks) cannot erase or double the outline.
const EDGE0 = 5, EDGE1 = 11;                               // weight 0 within 5 px of the silhouette edge, 1 from 11 px
const edgeDist = new Float32Array((RECT.R - RECT.L) * (RECT.B - RECT.T)).fill(EDGE1);
for (let y = RECT.T - EDGE1; y < RECT.B + EDGE1; y++) for (let x = RECT.L - EDGE1; x < RECT.R + EDGE1; x++) {
  if (ref[(y * W + x) * 4 + 3] >= 250) continue;
  for (let v = -EDGE1; v <= EDGE1; v++) for (let u = -EDGE1; u <= EDGE1; u++) {
    const yy = y + v, xx = x + u; if (yy < RECT.T || yy >= RECT.B || xx < RECT.L || xx >= RECT.R) continue;
    const k = (yy - RECT.T) * (RECT.R - RECT.L) + xx - RECT.L, d = Math.hypot(u, v); if (d < edgeDist[k]) edgeDist[k] = d;
  }
}

for (const [id, [exp, dx, dy, bodyName]] of Object.entries(SURFACES)) {
  // 1) head layer = reference, with the face rectangle replaced by the expression's.
  //    The weight ramps from 0 at the rectangle edge to 1 at FEATHER px inside, so nothing
  //    outside RECT is ever touched. Alpha stays the reference's (the expression edits come back
  //    with a semi-opaque background, so their alpha is not usable); colour is blended
  //    alpha-premultiplied, and only where both layers are opaque. Boy only: the weight is also
  //    capped by the distance to the silhouette edge (edgeDist), see above.
  const head = Buffer.from(ref);
  if (exp) {
    const e = load(exp);
    for (let y = RECT.T; y < RECT.B; y++) for (let x = RECT.L; x < RECT.R; x++) {
      const i = (y * W + x) * 4, j = ((y + dy) * W + (x + dx)) * 4;
      if (ref[i + 3] < 250 || e[j + 3] < 240) continue;
      const d = Math.min(x - RECT.L, RECT.R - 1 - x, y - RECT.T, RECT.B - 1 - y) + 0.5;
      const we = (edgeDist[(y - RECT.T) * (RECT.R - RECT.L) + x - RECT.L] - EDGE0) / (EDGE1 - EDGE0);
      const w = Math.max(0, Math.min(1, d / FEATHER, we));
      if (w === 0) continue;
      const aR = ref[i + 3] * (1 - w), aE = e[j + 3] * w;
      for (let c = 0; c < 3; c++) head[i + c] = Math.round((ref[i + c] * aR + e[j + c] * aE) / (aR + aE));
    }
  }
  // 2) body layer below Y_CUT, horizontal feather band (same as the girl's)
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

`pose-surprise2` はびっくりの作り直し。サーフェス 11（通常）は設定画像を何も変えずに縮小したもの。

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

1 枚の大きさ（男の子。縮小のコマンドは女の子と同じ。減色はしていない）:

| ファイル | 表情 | ポーズ | バイト |
|---|---|---|---|
| `surface10.png` | 笑顔 | 基本 | 84,577 |
| `surface11.png` | 通常 | 基本 | 85,018 |
| `surface12.png` | 照れ | 腕組み | 78,945 |
| `surface13.png` | 驚き | びっくり | 90,305 |
| `surface14.png` | 泣き | 基本 | 85,491 |
| `surface15.png` | 困惑 | 腕組み | 79,367 |
| `surface16.png` | キラキラ | びっくり | 90,901 |
| `surface17.png` | 眠い | 基本 | 84,886 |
| `surface18.png` | 怒り | 腕組み | 79,050 |
| 合計（9 枚） | | | 758,540（740.8 KB） |

シェルの絵 18 枚の合計は 1,732,591 バイト（1.65 MB。上限は 4.5 MB ＝ 4,718,592 バイト）。

## 8. 試作の実測

設計の段階の試作（女の子の 3 ポーズ）の実測は、タスク 6.2 で記入する。

### 女の子の 9 枚の検証（2026-10-10・出力した 333×500 の 9 枚に対する一回限りの比較）

- 9 枚とも 333×500・RGBA 8 ビット・四隅のアルファ 0・1 枚 104〜109 KB（106,198〜111,273 バイト）。
- (a) 出力の 0〜197 行目で、顔の矩形（出力の座標）を `MARGIN`=8 だけ広げた範囲（x 112〜217・y 117〜212）の外の画素が、9 枚すべてでバイト単位に一致した。
- (b) 同じポーズの絵どうし（基本 = 1・0・7・5、びっくり = 3・8、わくわく = 6・2・4）で、広げた顔の矩形の外の画素が、キャンバス全体で一致した。
- 頭の位置ずれ: 表情 8 種のうち 7 種が 0 px、照れが横に 1 px（1024 px 幅）。
- 目で見た確認（中間の灰色の背景に並べた一覧、顔と首の 3 倍拡大）: 9 種の表情が名前どおりに読めて互いに区別できる。顔の矩形の縁・色の段差、首・襟・おさげの継ぎ目、縁の色残り、透かし・文字は見えない。手は顔・髪に重なっていない。

### 男の子の 9 枚の検証（2026-10-10・出力した 333×500 の 9 枚に対する一回限りの比較）

- 判断（ポーズの数）: 男の子もポーズ 3 種で進めた（基本 1 種へ戻す形は使っていない）。びっくり・腕組みとも、頭の位置ずれ 0 px・背丈と足元 1 px 以内・首と襟の輪郭 3 px 以内で設定画像と揃い、継ぎ目は見えなかった。
- 9 枚とも 333×500・RGBA 8 ビット・四隅のアルファ 0・テキストのチャンクなし・1 枚 77〜89 KB（78,945〜90,901 バイト）。
- (a) 出力の 0〜213 行目（切り線 221.5 − `MARGIN`）で、顔の矩形（出力の座標）を `MARGIN`=8 だけ広げた範囲（x 116〜221・y 147〜227）の外の画素が、9 枚すべてでバイト単位に一致した。実際には 0〜216 行目まで一致する（体が違い始めるのは 217 行目）。
- (b) 同じポーズの絵どうし（基本 = 11・10・17・14、びっくり = 13・16、腕組み = 15・18・12）で、広げた顔の矩形の外の画素が、キャンバス全体で一致した。
- 頭の位置ずれ: 表情 8 種・ポーズ 2 種とも 0 px。
- 目で見た確認（中間の灰色の背景に並べた一覧、顔と首の 3 倍拡大、暗い色・白・マゼンタの背景）: 9 種の表情が名前どおりに読めて互いに区別できる。顔の矩形の縁・色の段差、首・襟・ネッカチーフの継ぎ目や二重の線、縁の色残り、透かし・文字は見えない。帽子は 9 枚で同じ。手は顔・髪・帽子に重なっていない。
- 女の子と並べた確認（`surface1` と `surface11`、`surface0` と `surface10`）: 2 人とも同じ変換で縮小しているので、背丈の釣り合いは承認済みの設定画像どうしと同じ（出力での人物の高さは女の子 404 px・男の子は帽子込みで 414 px）。画風（線の太さ・塗り・目の描き方）も揃って見える。

## 9. 費用

| 段階 | 生成の回数 | 額 |
|---|---|---|
| 設計の段階（試作を含む） | （タスク 6.2 で記入） | （タスク 6.2 で記入） |
| 実装: 女の子の表情（タスク 2.1） | 9 回（表情 8 + 泣きの作り直し 1） | 約 0.51 ドル（推定） |
| 実装: 男の子（タスク 2.2） | 11 回（ポーズ 3 ＝ 採用 2・不採用 1、表情 8） | 約 0.63 ドル（推定） |
| 実装の累計（タスク 2.1 + 2.2） | 20 回（女の子 9 + 男の子 11） | 約 1.14 ドル（推定） |

タスク 2.1 の額は推定。fal の MCP（`check_account_status`）は残高を返さないので、残高の差は読めなかった。回数 9 × 設計の段階の実測（10 枚で 0.57 ドル＝ 1 枚およそ 0.057 ドル）で計算した。ダッシュボードの値では確かめていない。

タスク 2.2 の額も同じ計算の推定（11 × 0.057 = 0.63 ドル）。残高の差は使っていない（fal の MCP が残高を返さないのはタスク 2.1 で確かめたとおり）。ダッシュボードの値では確かめていない。実装の累計 20 回は、見込みの 20 枚・1.2 ドル前後に収まっている。

## 10. The Unlicense（原文）

（タスク 6.2 で記入）
