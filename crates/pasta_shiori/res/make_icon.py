"""img/pasta.svg（正典）から pasta.dll のアイコン pasta.ico を作る。

台紙（角丸の四角）は外し、麺に囲まれた内側だけをクリーム色で塗る。16/24/32px は文字が
潰れるので「pasta」の代わりに点 3 つ（…）を置く。Windows の Edge（SVG の描画）と Pillow を使う。

    python make_icon.py <repo>/img/pasta.svg <作業フォルダ>
    → <作業フォルダ>/pasta.ico と pasta-{16..256}.png をこのフォルダへコピーする（preview.png は目視確認用）
"""
import re, subprocess, sys, time
from pathlib import Path
from PIL import Image, ImageDraw

src, out_dir = Path(sys.argv[1]), Path(sys.argv[2])
out_dir.mkdir(parents=True, exist_ok=True)
EDGE = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
CREAM = (0xF6, 0xE5, 0xC9, 255)
SMALL = [16, 24, 32]          # 文字を消すサイズ
LARGE = [48, 64, 128, 256]

svg = src.read_text(encoding="utf-8")
svg = re.sub(r"<rect [^>]*/>", "", svg)                      # 角丸の台紙を外す
paths = re.findall(r"<path [^>]*/>", svg)                    # [影, 麺, 文字]
noodle_svg = svg.replace(paths[2], "")
text_svg = svg.replace(paths[0], "").replace(paths[1], "")


def render(text, name):
    p = out_dir / f"{name}.svg"
    p.write_text(text, encoding="utf-8")
    png = out_dir / f"{name}.png"
    subprocess.run([EDGE, "--headless=new", f"--user-data-dir={out_dir / 'prof'}",
                    "--disable-gpu", "--hide-scrollbars", "--default-background-color=00000000",
                    "--window-size=1024,1024", f"--screenshot={png}", p.as_uri()],
                   check=True, capture_output=True, timeout=60)
    for _ in range(300):  # msedge.exe はランチャーで、子が書き終える前に戻る
        if png.exists() and png.stat().st_size > 0:
            time.sleep(0.5)
            break
        time.sleep(0.1)
    return Image.open(png).convert("RGBA")


noodle = render(noodle_svg, "noodle")
text = render(text_svg, "text")

# 内側 = 外周から届かない領域（麺そのもの＋吹き出し本体＋結び目の輪）。外縁へははみ出さない。
a = noodle.getchannel("A").point(lambda v: 255 if v >= 128 else 0)
ImageDraw.floodfill(a, (0, 0), 128)
mask = a.point(lambda v: 0 if v == 128 else 255)
assert mask.getpixel((512, 420)) == 255, "bubble interior not enclosed"

base = Image.new("RGBA", noodle.size, (0, 0, 0, 0))
base.paste(CREAM, mask=mask)
base.alpha_composite(noodle)
with_text = base.copy()
with_text.alpha_composite(text)

# 小サイズ用: 文字の代わりに「…」。文字の外接矩形の中央に、文字色の点を 3 つ並べる。
l, t, r, b = text.getchannel("A").getbbox()
cx, cy = (l + r) / 2, (t + b) / 2
def with_dots(dot_r, pitch):
    im = base.copy()
    d = ImageDraw.Draw(im)
    for i in (-1, 0, 1):
        x = cx + i * pitch
        d.ellipse((x - dot_r, cy - dot_r, x + dot_r, cy + dot_r), fill=(0x7E, 0x62, 0x46, 255))
    return im


# 16px は点を大きくしないと潰れる。24/32px は小さい点のほうが吹き出しとの釣り合いが良い。
DOTS = {16: (72, 200), 24: (52, 170), 32: (52, 170)}
imgs = {s: with_dots(*DOTS[s]).resize((s, s), Image.LANCZOS) for s in SMALL}
imgs |= {s: with_text.resize((s, s), Image.LANCZOS) for s in LARGE}
for s, im in imgs.items():
    im.save(out_dir / f"pasta-{s}.png")
imgs[256].save(out_dir / "pasta.ico", sizes=[(s, s) for s in imgs],
               append_images=[imgs[s] for s in sorted(imgs) if s != 256])

# 確認用シート（白・黒背景で 4 倍拡大）
sizes = sorted(imgs)
w = sum(min(s * 4, 512) for s in sizes) + 10 * len(sizes)
sheet = Image.new("RGBA", (w, 2 * 512 + 10), (255, 255, 255, 255))
sheet.paste((32, 32, 32, 255), (0, 522, w, 1034))
x = 0
for s in sizes:
    big = imgs[s].resize((min(s * 4, 512),) * 2, Image.NEAREST)
    sheet.alpha_composite(big, (x, 0))
    sheet.alpha_composite(big, (x, 522))
    x += big.width + 10
sheet.crop((0, 0, x, 1034)).save(out_dir / "preview.png")
print("ok", sorted(imgs))
