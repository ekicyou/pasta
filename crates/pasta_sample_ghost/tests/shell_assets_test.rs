//! シェルの素材の検証（hello-pasta-shell-art）。
//!
//! コミットした素材 `ghosts/hello-pasta/shell/master/`（立ち絵 18 枚と `surfaces.txt`）を
//! 実ファイルのまま読み、枚数・寸法・透過・大きさ・サーフェス定義・当たり判定・画素の一致を検査する。
//! 入門ガイドに載せる見本（`book/src/img/hello-pasta/` の写し 2 枚）がシェルの絵と同じかも見る。
//! ネットワーク・一時ディレクトリ・DLL は使わず、素材を書き換えない。
//!
//! `surfaces.txt` は小さなパーサで読む。行が足りない・順序が違う・座標が整数でないなど
//! 形が崩れていれば、0 件として素通りせず panic（テストの失敗）にする
//! （`tutorial_stages_test.rs` と同じ流儀）。

// このファイルが使うのは `workspace_root` だけ（DLL を写すヘルパーは使わない）
#[allow(dead_code)]
mod common;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// 立ち絵のキャンバス（幅・高さ）
const CANVAS: (u32, u32) = (333, 500);
/// 立ち絵 1 枚の上限（250 KB）
const MAX_PNG_BYTES: u64 = 250 * 1024;
/// 立ち絵 18 枚の合計の上限（4.5 MB。KB・MB は 1024 進）
const MAX_TOTAL_BYTES: u64 = 4_718_592;
/// 顔の矩形を外へ広げる余白（ぼかし帯 + 縮小フィルタ）
const MARGIN: u32 = 8;
/// 女の子のサーフェス番号
const SAKURA: [u32; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
/// 男の子のサーフェス番号
const KERO: [u32; 9] = [10, 11, 12, 13, 14, 15, 16, 17, 18];
/// ポーズの割り当て（基本・びっくり・決めポーズの順。同じポーズを使う表情のサーフェス番号の組）
const SAKURA_POSES: [&[u32]; 3] = [&[1, 0, 7, 5], &[3, 8], &[6, 2, 4]];
const KERO_POSES: [&[u32]; 3] = [&[11, 10, 17, 14], &[13, 16], &[15, 18, 12]];

/// 復号した立ち絵（`data` は 1 画素 4 バイトの RGBA）
struct Rgba8 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// 当たり判定の矩形。終点 `(x2, y2)` は矩形に含む画素座標
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

impl Rect {
    fn contains(&self, inner: &Rect) -> bool {
        self.x1 <= inner.x1 && inner.x2 <= self.x2 && self.y1 <= inner.y1 && inner.y2 <= self.y2
    }

    fn intersects(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && other.x1 <= self.x2 && self.y1 <= other.y2 && other.y1 <= self.y2
    }
}

/// `surfaces.txt` のサーフェス定義 1 つ（`element` は element0 が参照するファイル名）
struct SurfaceDef {
    id: u32,
    element: String,
    head: Rect,
    face: Rect,
    body: Rect,
}

fn shell_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta/shell/master")
}

/// 立ち絵のサーフェス番号（0〜8・10〜18）
fn surface_ids() -> impl Iterator<Item = u32> {
    SAKURA.into_iter().chain(KERO)
}

fn png_name(id: u32) -> String {
    format!("surface{id}.png")
}

/// PNG を RGBA8 に復号する。読めない・アルファチャンネルを持たないなら panic。
///
/// パレットと tRNS は展開して読むので、RGBA8 でも tRNS 付きパレットでも通る。
fn load_rgba(path: &Path) -> Rgba8 {
    let name = path.display();
    let file = File::open(path).unwrap_or_else(|e| panic!("{name}: 開けない: {e}"));
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .unwrap_or_else(|e| panic!("{name}: PNG として読めない: {e}"));
    let size = reader
        .output_buffer_size()
        .unwrap_or_else(|| panic!("{name}: 復号後の大きさが求まらない"));
    let mut buf = vec![0; size];
    let info = reader
        .next_frame(&mut buf)
        .unwrap_or_else(|e| panic!("{name}: PNG として読めない: {e}"));
    buf.truncate(info.buffer_size());
    let data = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::GrayscaleAlpha => buf
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[gray, alpha]| [gray, gray, gray, alpha])
            .collect(),
        other => panic!("{name}: アルファチャンネルを持たない（復号後の色型 {other:?}）"),
    };
    Rgba8 {
        width: info.width,
        height: info.height,
        data,
    }
}

/// `collisionexK,ID,rect,X1,Y1,X2,Y2` の 1 行を読む。`K`・`ID`・形が期待と違えば panic。
fn parse_collision(line: &str, index: u32, name: &str, at: &str) -> Rect {
    let shape = format!("collisionex{index},{name},rect,");
    let coords: Vec<u32> = line
        .strip_prefix(&shape)
        .unwrap_or_else(|| panic!("{at}: `{shape}X1,Y1,X2,Y2` でない: `{line}`"))
        .split(',')
        .map(|c| {
            c.parse()
                .unwrap_or_else(|_| panic!("{at}: 座標 `{c}` が整数でない: `{line}`"))
        })
        .collect();
    let [x1, y1, x2, y2] = coords[..] else {
        panic!("{at}: 座標が 4 つでない: `{line}`")
    };
    Rect { x1, y1, x2, y2 }
}

/// `surfaces.txt` を読む。先頭行は `charset,UTF-8`、続く各ブロックは次の 7 行（空行は読み飛ばす）。
///
/// ```text
/// surfaceN
/// {
/// element0,overlay,ファイル名,0,0
/// collisionex0,Face,rect,X1,Y1,X2,Y2
/// collisionex1,Head,rect,X1,Y1,X2,Y2
/// collisionex2,Body,rect,X1,Y1,X2,Y2
/// }
/// ```
///
/// 当たり判定は `Face`・`Head`・`Body` がこの順に 1 つずつ（先に書いた方が手前になる）。
/// 形が崩れていれば panic（テストの失敗）。
fn parse_surfaces_txt(text: &str) -> Vec<SurfaceDef> {
    assert_eq!(
        text.lines().next(),
        Some("charset,UTF-8"),
        "surfaces.txt: 先頭行が `charset,UTF-8` でない"
    );
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l))
        .skip(1)
        .filter(|(_, l)| !l.is_empty());
    let mut defs = Vec::new();
    while let Some((n, line)) = lines.next() {
        let id: u32 = line
            .strip_prefix("surface")
            .and_then(|s| s.parse().ok())
            .filter(|id| line == format!("surface{id}"))
            .unwrap_or_else(|| panic!("surfaces.txt {n} 行目: `surfaceN` でない: `{line}`"));
        let mut next = || {
            let (n, line) = lines.next().unwrap_or_else(|| {
                panic!("surfaces.txt: surface{id} の定義が `}}` の前で終わっている")
            });
            (format!("surfaces.txt {n} 行目（surface{id}）"), line)
        };
        let (at, line) = next();
        assert_eq!(line, "{", "{at}: `{{` でない");
        let (at, line) = next();
        let element = line
            .strip_prefix("element0,overlay,")
            .and_then(|s| s.strip_suffix(",0,0"))
            .unwrap_or_else(|| panic!("{at}: `element0,overlay,ファイル名,0,0` でない: `{line}`"))
            .to_string();
        let mut collision = |index, name| {
            let (at, line) = next();
            parse_collision(line, index, name, &at)
        };
        let face = collision(0, "Face");
        let head = collision(1, "Head");
        let body = collision(2, "Body");
        let (at, line) = next();
        assert_eq!(line, "}", "{at}: `}}` でない（当たり判定は 3 行だけ）");
        defs.push(SurfaceDef {
            id,
            element,
            head,
            face,
            body,
        });
    }
    assert!(
        !defs.is_empty(),
        "surfaces.txt: サーフェス定義が 1 つも無い"
    );
    defs
}

fn load_defs() -> Vec<SurfaceDef> {
    let path = shell_dir().join("surfaces.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: 読めない: {e}", path.display()));
    parse_surfaces_txt(&text)
}

/// 番号 `id` のサーフェス定義を引く。無ければ panic。
fn def(defs: &[SurfaceDef], id: u32) -> &SurfaceDef {
    defs.iter()
        .find(|d| d.id == id)
        .unwrap_or_else(|| panic!("surfaces.txt: surface{id} の定義が無い"))
}

/// 顔の領域（`Face` を `MARGIN` だけ外へ広げ、キャンバスに収めた矩形）
fn face_area(face: &Rect) -> Rect {
    Rect {
        x1: face.x1.saturating_sub(MARGIN),
        y1: face.y1.saturating_sub(MARGIN),
        x2: (face.x2 + MARGIN).min(CANVAS.0 - 1),
        y2: (face.y2 + MARGIN).min(CANVAS.1 - 1),
    }
}

/// `within` の中で `except` のどれにも入らない画素（色とアルファ）を `a` と `b` で比べる。
/// 違えば、最初に食い違った座標 `(x, y)` を返す。
fn pixels_equal_outside(
    a: &Rgba8,
    b: &Rgba8,
    within: &Rect,
    except: &[Rect],
) -> Result<(), (u32, u32)> {
    for y in within.y1..=within.y2 {
        for x in within.x1..=within.x2 {
            let point = Rect {
                x1: x,
                y1: y,
                x2: x,
                y2: y,
            };
            if except.iter().any(|r| r.contains(&point)) {
                continue;
            }
            let at = |img: &Rgba8| ((y * img.width + x) * 4) as usize;
            if a.data[at(a)..at(a) + 4] != b.data[at(b)..at(b) + 4] {
                return Err((x, y));
            }
        }
    }
    Ok(())
}

/// 立ち絵は 0〜8・10〜18 の 18 枚だけ。欠番の `surface9.png` は「余分」として捕まえる。
#[test]
fn shell_has_exactly_18_surface_pngs_and_no_surface9() {
    let expected: BTreeSet<String> = surface_ids().map(png_name).collect();
    let actual: BTreeSet<String> = std::fs::read_dir(shell_dir())
        .expect("shell/master を読めない")
        .map(|e| {
            e.expect("shell/master のエントリを読めない")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.ends_with(".png"))
        .collect();
    let missing: Vec<_> = expected.difference(&actual).collect();
    let extra: Vec<_> = actual.difference(&expected).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "shell/master の PNG が surface0〜8・10〜18 の 18 枚でない。欠け: {missing:?}・余分: {extra:?}"
    );
}

#[test]
fn every_surface_png_is_333x500_rgba_with_transparent_corners() {
    for id in surface_ids() {
        let name = png_name(id);
        // アルファチャンネルの有無は load_rgba が確かめる
        let img = load_rgba(&shell_dir().join(&name));
        assert_eq!(
            (img.width, img.height),
            CANVAS,
            "{name}: 寸法が 333×500 でない"
        );
        let (right, bottom) = (img.width - 1, img.height - 1);
        for (x, y) in [(0, 0), (right, 0), (0, bottom), (right, bottom)] {
            let alpha = img.data[((y * img.width + x) * 4 + 3) as usize];
            assert_eq!(
                alpha, 0,
                "{name}: 隅 ({x},{y}) が透明でない（アルファ {alpha}）"
            );
        }
    }
}

#[test]
fn every_surface_png_is_within_size_caps() {
    let mut total = 0;
    for id in surface_ids() {
        let name = png_name(id);
        let bytes = std::fs::metadata(shell_dir().join(&name))
            .unwrap_or_else(|e| panic!("{name}: 大きさを読めない: {e}"))
            .len();
        assert!(
            bytes <= MAX_PNG_BYTES,
            "{name}: {bytes} バイトで、1 枚の上限 {MAX_PNG_BYTES} バイトを超える"
        );
        total += bytes;
    }
    assert!(
        total <= MAX_TOTAL_BYTES,
        "立ち絵 18 枚の合計が {total} バイトで、上限 {MAX_TOTAL_BYTES} バイトを超える"
    );
}

/// 先頭行と、各定義の当たり判定（`Face`→`Head`→`Body` が 1 つずつ）は `parse_surfaces_txt` が確かめる。
#[test]
fn surfaces_txt_has_18_defs_matching_pngs_with_head_face_body() {
    let defs = load_defs();
    let mut ids: Vec<u32> = defs.iter().map(|d| d.id).collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        surface_ids().collect::<Vec<_>>(),
        "surfaces.txt: サーフェス定義が 0〜8・10〜18 の 18 個でない"
    );
    for d in &defs {
        assert_eq!(
            d.element,
            png_name(d.id),
            "surfaces.txt: surface{} の element0 が同じ番号の PNG でない",
            d.id
        );
    }
}

#[test]
fn head_and_face_rects_are_identical_within_a_character() {
    let defs = load_defs();
    for ids in [SAKURA, KERO] {
        let first = def(&defs, ids[0]);
        for id in ids {
            let d = def(&defs, id);
            assert_eq!(
                d.head, first.head,
                "surface{id} の Head が同じ人物の surface{} と違う",
                first.id
            );
            assert_eq!(
                d.face, first.face,
                "surface{id} の Face が同じ人物の surface{} と違う",
                first.id
            );
        }
    }
}

#[test]
fn body_rect_is_identical_within_a_pose() {
    let defs = load_defs();
    for pose in SAKURA_POSES.into_iter().chain(KERO_POSES) {
        let first = def(&defs, pose[0]);
        for &id in pose {
            assert_eq!(
                def(&defs, id).body,
                first.body,
                "surface{id} の Body が同じポーズの surface{} と違う",
                first.id
            );
        }
    }
}

/// 矩形はキャンバス内（0 ≤ X1 < X2 ≤ 332、0 ≤ Y1 < Y2 ≤ 499。終点も画素の添字に使える）。`Face` は `Head` に含まれ、
/// `Body` はどちらとも交わらない。
#[test]
fn collision_rects_are_face_inside_head_and_body_apart() {
    for d in load_defs() {
        let id = d.id;
        for (name, r) in [("Face", d.face), ("Head", d.head), ("Body", d.body)] {
            assert!(
                r.x1 < r.x2 && r.x2 < CANVAS.0 && r.y1 < r.y2 && r.y2 < CANVAS.1,
                "surface{id} の {name} がキャンバス内の矩形でない: {r:?}"
            );
        }
        assert!(
            d.head.contains(&d.face),
            "surface{id} の Face が Head に含まれない: Face {:?}・Head {:?}",
            d.face,
            d.head
        );
        assert!(
            !d.body.intersects(&d.head),
            "surface{id} の Body が Head と交わる: Body {:?}・Head {:?}",
            d.body,
            d.head
        );
        assert!(
            !d.body.intersects(&d.face),
            "surface{id} の Body が Face と交わる: Body {:?}・Face {:?}",
            d.body,
            d.face
        );
    }
}

/// 同じ人物の 9 枚は、`Head` の中で顔の領域の外の画素が「通常」（1・11）と同じ。
#[test]
fn head_pixels_outside_face_are_identical_across_nine_surfaces() {
    let defs = load_defs();
    for (ids, normal) in [(SAKURA, 1), (KERO, 11)] {
        let d = def(&defs, normal);
        let base = load_rgba(&shell_dir().join(png_name(normal)));
        for id in ids {
            let img = load_rgba(&shell_dir().join(png_name(id)));
            if let Err((x, y)) = pixels_equal_outside(&img, &base, &d.head, &[face_area(&d.face)]) {
                panic!(
                    "surface{id} と surface{normal} で、頭（顔の領域の外）の画素が ({x},{y}) で食い違う"
                );
            }
        }
    }
}

/// 同じポーズの絵は、キャンバス全体で顔の領域の外の画素が、組の先頭の絵と同じ。
#[test]
fn pixels_outside_face_are_identical_within_a_pose() {
    let defs = load_defs();
    let canvas = Rect {
        x1: 0,
        y1: 0,
        x2: CANVAS.0 - 1,
        y2: CANVAS.1 - 1,
    };
    for pose in SAKURA_POSES.into_iter().chain(KERO_POSES) {
        let first = pose[0];
        let face = face_area(&def(&defs, first).face);
        let base = load_rgba(&shell_dir().join(png_name(first)));
        for &id in pose {
            let img = load_rgba(&shell_dir().join(png_name(id)));
            if let Err((x, y)) = pixels_equal_outside(&img, &base, &canvas, &[face]) {
                panic!(
                    "surface{id} と surface{first} で、顔の領域の外の画素が ({x},{y}) で食い違う"
                );
            }
        }
    }
}

/// 入門ガイドの見本（`book/src/img/hello-pasta/`）は、シェルの同じ名前の絵とバイト単位で同じ。
#[test]
fn book_sample_images_are_byte_identical_to_shell() {
    let book_dir = common::workspace_root().join("book/src/img/hello-pasta");
    for id in [0, 10] {
        let name = png_name(id);
        let read = |path: PathBuf| {
            std::fs::read(&path).unwrap_or_else(|e| panic!("{}: 読めない: {e}", path.display()))
        };
        let book = read(book_dir.join(&name));
        let shell = read(shell_dir().join(&name));
        assert!(
            book == shell,
            "{name}: 本の写し（{} バイト）がシェルの絵（{} バイト）とバイト単位で一致しない。shell/master から写し直す",
            book.len(),
            shell.len()
        );
    }
}
