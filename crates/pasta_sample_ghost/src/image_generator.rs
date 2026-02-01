//! ピクトグラム画像生成
//!
//! トイレマーク風のシンプルな人型アイコンを生成します。
//! 構成：○（頭）+ △/▽（胴体）のみ。手足なし。

use crate::GhostError;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use std::path::Path;

/// キャラクター種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    /// 女の子（sakura）- スカート（△ 上向き三角形）
    Sakura,
    /// 男の子（kero）- ズボン（▽ 下向き三角形）
    Kero,
}

/// 表情種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expression {
    /// 笑顔 ^ ^
    Happy,
    /// 通常 - -
    Normal,
    /// 照れ > <
    Shy,
    /// 驚き o o
    Surprised,
    /// 泣き ; ;
    Crying,
    /// 困惑 @ @
    Confused,
    /// キラキラ * *
    Sparkle,
    /// 眠い = =
    Sleepy,
    /// 怒り # #
    Angry,
}

impl Expression {
    /// 全表情を取得
    pub fn all() -> [Expression; 9] {
        [
            Expression::Happy,
            Expression::Normal,
            Expression::Shy,
            Expression::Surprised,
            Expression::Crying,
            Expression::Confused,
            Expression::Sparkle,
            Expression::Sleepy,
            Expression::Angry,
        ]
    }

    /// サーフェス番号オフセット取得
    pub fn surface_offset(&self) -> u32 {
        match self {
            Expression::Happy => 0,
            Expression::Normal => 1,
            Expression::Shy => 2,
            Expression::Surprised => 3,
            Expression::Crying => 4,
            Expression::Confused => 5,
            Expression::Sparkle => 6,
            Expression::Sleepy => 7,
            Expression::Angry => 8,
        }
    }
}

/// 画像サイズ（3頭身バランス: 128x256）
const WIDTH: u32 = 128;
const HEIGHT: u32 = 256;

/// 3頭身の1頭分（256/3 ≈ 85px、半径は42px）
const HEAD_RADIUS: i32 = 42;
/// 頭の中心Y座標（頭の直径分の余白を上に）
const HEAD_CENTER_Y: i32 = HEAD_RADIUS + 2;
/// 胴体開始Y座標（頭の下端）
const BODY_TOP_Y: i32 = HEAD_CENTER_Y + HEAD_RADIUS;
/// 胴体高さ（残り2頭分）
const BODY_HEIGHT: i32 = HEAD_RADIUS * 4; // 2頭分 = 直径×2

/// キャラクター色（トイレマーク標準色）
fn character_color(character: Character) -> Rgba<u8> {
    match character {
        Character::Sakura => Rgba([220, 53, 69, 255]), // 赤 #DC3545
        Character::Kero => Rgba([0, 123, 255, 255]),   // 青 #007BFF
    }
}

/// 全サーフェス画像を生成
pub fn generate_surfaces(output_dir: &Path) -> Result<(), GhostError> {
    std::fs::create_dir_all(output_dir)?;

    // sakura サーフェス (0-8)
    for expr in Expression::all() {
        let img = generate_surface(Character::Sakura, expr);
        let path = output_dir.join(format!("surface{}.png", expr.surface_offset()));
        img.save(&path)?;
    }

    // kero サーフェス (10-18)
    for expr in Expression::all() {
        let img = generate_surface(Character::Kero, expr);
        let path = output_dir.join(format!("surface{}.png", 10 + expr.surface_offset()));
        img.save(&path)?;
    }

    Ok(())
}

/// 個別サーフェス画像を生成
///
/// トイレピクトグラム風のシンプル構成:
/// - 頭部: 塗りつぶし円
/// - 胴体: 女の子=△（上向き三角形）、男の子=▽（下向き三角形）
/// - 手足: なし
pub fn generate_surface(character: Character, expression: Expression) -> RgbaImage {
    let mut img = RgbaImage::new(WIDTH, HEIGHT);
    let color = character_color(character);

    // 頭部（円）
    let center_x = (WIDTH / 2) as i32;
    draw_filled_circle_mut(&mut img, (center_x, HEAD_CENTER_Y), HEAD_RADIUS, color);

    // 胴体
    draw_body(&mut img, character, color);

    // 表情を描画
    draw_expression(&mut img, expression, character);

    img
}

/// 胴体を描画
///
/// - 女の子: △（上向き三角形）- 頂点が上、底辺が下（スカート）
/// - 男の子: ▽（下向き三角形）- 底辺が上、頂点が下（ズボン）
fn draw_body(img: &mut RgbaImage, character: Character, color: Rgba<u8>) {
    let center_x = WIDTH as i32 / 2;
    let body_bottom_y = (BODY_TOP_Y + BODY_HEIGHT).min(HEIGHT as i32 - 1);

    match character {
        Character::Sakura => {
            // △ 上向き三角形（スカート）: 頂点が上、底辺が下
            // 頂点から始まり、下に向かって広がる
            let bottom_half_width = 55; // 底辺の半幅

            for y in BODY_TOP_Y..body_bottom_y {
                let ratio = (y - BODY_TOP_Y) as f32 / BODY_HEIGHT as f32;
                let half_width = bottom_half_width as f32 * ratio;
                let left = (center_x as f32 - half_width).max(0.0) as u32;
                let right = (center_x as f32 + half_width).min(WIDTH as f32) as u32;

                for x in left..right {
                    img.put_pixel(x, y as u32, color);
                }
            }
        }
        Character::Kero => {
            // ▽ 下向き三角形（ズボン）: 底辺が上、頂点が下
            // 上から始まり、下に向かって狭まる
            let top_half_width = 55; // 上辺の半幅

            for y in BODY_TOP_Y..body_bottom_y {
                let ratio = (y - BODY_TOP_Y) as f32 / BODY_HEIGHT as f32;
                let half_width = top_half_width as f32 * (1.0 - ratio);
                let left = (center_x as f32 - half_width).max(0.0) as u32;
                let right = (center_x as f32 + half_width).min(WIDTH as f32) as u32;

                for x in left..right {
                    img.put_pixel(x, y as u32, color);
                }
            }
        }
    }
}

/// 表情を描画（線描画で顔に重ねる）
fn draw_expression(img: &mut RgbaImage, expression: Expression, character: Character) {
    let white = Rgba([255, 255, 255, 255]);
    let center_x = WIDTH as i32 / 2;
    let eye_y = HEAD_CENTER_Y; // 顔の中央
    let left_eye_x = center_x - 18; // 目の間隔を広く
    let right_eye_x = center_x + 18;

    // @マーク用に背景色を取得
    let bg_color = character_color(character);

    match expression {
        Expression::Happy => {
            // ^ ^ 笑顔（山形）
            draw_caret(img, left_eye_x, eye_y, white);
            draw_caret(img, right_eye_x, eye_y, white);
        }
        Expression::Normal => {
            // - - 通常（横線）
            draw_thick_horizontal_line(img, left_eye_x, eye_y, 10, 3, white);
            draw_thick_horizontal_line(img, right_eye_x, eye_y, 10, 3, white);
        }
        Expression::Shy => {
            // > < 照れ
            draw_greater_than(img, left_eye_x, eye_y, white);
            draw_less_than(img, right_eye_x, eye_y, white);
        }
        Expression::Surprised => {
            // o o 驚き（円）
            draw_filled_circle_mut(img, (left_eye_x, eye_y), 8, white);
            draw_filled_circle_mut(img, (right_eye_x, eye_y), 8, white);
        }
        Expression::Crying => {
            // ; ; 泣き（セミコロン）
            draw_semicolon(img, left_eye_x, eye_y, white);
            draw_semicolon(img, right_eye_x, eye_y, white);
        }
        Expression::Confused => {
            // @ @ 困惑（渦）
            draw_at_sign(img, left_eye_x, eye_y, white, bg_color);
            draw_at_sign(img, right_eye_x, eye_y, white, bg_color);
        }
        Expression::Sparkle => {
            // * * キラキラ（星）
            draw_star(img, left_eye_x, eye_y, white);
            draw_star(img, right_eye_x, eye_y, white);
        }
        Expression::Sleepy => {
            // = = 眠い（二重線）
            draw_thick_horizontal_line(img, left_eye_x, eye_y - 5, 10, 3, white);
            draw_thick_horizontal_line(img, left_eye_x, eye_y + 5, 10, 3, white);
            draw_thick_horizontal_line(img, right_eye_x, eye_y - 5, 10, 3, white);
            draw_thick_horizontal_line(img, right_eye_x, eye_y + 5, 10, 3, white);
        }
        Expression::Angry => {
            // # # 怒り（ハッシュ）
            draw_hash(img, left_eye_x, eye_y, white);
            draw_hash(img, right_eye_x, eye_y, white);
        }
    }
}

// === 表情用ヘルパー関数 ===

/// 太い横線を描画
fn draw_thick_horizontal_line(
    img: &mut RgbaImage,
    cx: i32,
    cy: i32,
    half_len: i32,
    thickness: i32,
    color: Rgba<u8>,
) {
    let half_thick = thickness / 2;
    for dy in -half_thick..=half_thick {
        for dx in -half_len..=half_len {
            let x = (cx + dx) as u32;
            let y = (cy + dy) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// 太い縦線を描画
fn draw_thick_vertical_line(
    img: &mut RgbaImage,
    cx: i32,
    cy: i32,
    half_len: i32,
    thickness: i32,
    color: Rgba<u8>,
) {
    let half_thick = thickness / 2;
    for dx in -half_thick..=half_thick {
        for dy in -half_len..=half_len {
            let x = (cx + dx) as u32;
            let y = (cy + dy) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// ^ 山形を描画（太い線）
fn draw_caret(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    let size = 10; // 大きさ
    let thickness = 3; // 太さ

    // 左斜線 (左下から中央上へ)
    for i in 0..=size {
        let base_x = cx - size / 2 + i;
        let base_y = cy + size / 2 - i;
        // 太さを出すために周囲も塗る
        for t in 0..thickness {
            let x = (base_x + t - thickness / 2) as u32;
            let y = base_y as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
    // 右斜線 (中央上から右下へ)
    for i in 0..=size {
        let base_x = cx + i;
        let base_y = cy - size / 2 + i;
        for t in 0..thickness {
            let x = (base_x + t - thickness / 2) as u32;
            let y = base_y as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// > を描画（太い線）
fn draw_greater_than(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    let size = 8;
    let thickness = 3;

    // 上半分 (左上から中央へ)
    for i in 0..=size {
        let base_x = cx - size / 2 + i;
        let base_y = cy - size + i;
        for t in 0..thickness {
            let x = base_x as u32;
            let y = (base_y + t - thickness / 2) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
    // 下半分 (中央から左下へ)
    for i in 0..=size {
        let base_x = cx + size / 2 - i;
        let base_y = cy + i;
        for t in 0..thickness {
            let x = base_x as u32;
            let y = (base_y + t - thickness / 2) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// < を描画（太い線）
fn draw_less_than(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    let size = 8;
    let thickness = 3;

    // 上半分 (右上から中央へ)
    for i in 0..=size {
        let base_x = cx + size / 2 - i;
        let base_y = cy - size + i;
        for t in 0..thickness {
            let x = base_x as u32;
            let y = (base_y + t - thickness / 2) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
    // 下半分 (中央から右下へ)
    for i in 0..=size {
        let base_x = cx - size / 2 + i;
        let base_y = cy + i;
        for t in 0..thickness {
            let x = base_x as u32;
            let y = (base_y + t - thickness / 2) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// ; セミコロンを描画（大きく）
fn draw_semicolon(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    // 上の点（大きく）
    draw_filled_circle_mut(img, (cx, cy - 6), 4, color);
    // 下の点
    draw_filled_circle_mut(img, (cx, cy + 4), 4, color);
    // 涙のしずく（太く）
    for i in 0..8 {
        for t in -2..=2 {
            let x = (cx + t) as u32;
            let y = (cy + 8 + i) as u32;
            if x < WIDTH && y < HEIGHT {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// @ 渦巻きを描画（大きく）
fn draw_at_sign(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>, bg_color: Rgba<u8>) {
    // 外円（大きく）
    draw_filled_circle_mut(img, (cx, cy), 10, color);
    // 内円（背景色で抜く）→ドーナツ状
    draw_filled_circle_mut(img, (cx, cy), 5, bg_color);
    // 中央に点
    draw_filled_circle_mut(img, (cx, cy), 2, color);
}

/// * 星を描画（大きく太く）
fn draw_star(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    let size = 8;
    let thickness = 3;

    // 十字
    draw_thick_horizontal_line(img, cx, cy, size, thickness, color);
    draw_thick_vertical_line(img, cx, cy, size, thickness, color);

    // 斜め線（太く）
    for i in -size..=size {
        for t in -1..=1 {
            // 右下がり
            let x1 = (cx + i) as u32;
            let y1 = (cy + i + t) as u32;
            if x1 < WIDTH && y1 < HEIGHT {
                img.put_pixel(x1, y1, color);
            }
            // 右上がり
            let x2 = (cx + i) as u32;
            let y2 = (cy - i + t) as u32;
            if x2 < WIDTH && y2 < HEIGHT {
                img.put_pixel(x2, y2, color);
            }
        }
    }
}

/// # ハッシュを描画（大きく太く）
fn draw_hash(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    let size = 8;
    let thickness = 3;
    let spacing = 5;

    // 横線2本
    draw_thick_horizontal_line(img, cx, cy - spacing, size, thickness, color);
    draw_thick_horizontal_line(img, cx, cy + spacing, size, thickness, color);
    // 縦線2本
    draw_thick_vertical_line(img, cx - spacing, cy, size, thickness, color);
    draw_thick_vertical_line(img, cx + spacing, cy, size, thickness, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_color() {
        // 女の子=赤、男の子=青（トイレマーク標準色）
        let sakura_color = character_color(Character::Sakura);
        assert_eq!(sakura_color, Rgba([220, 53, 69, 255])); // 赤

        let kero_color = character_color(Character::Kero);
        assert_eq!(kero_color, Rgba([0, 123, 255, 255])); // 青
    }

    #[test]
    fn test_expression_offset() {
        assert_eq!(Expression::Happy.surface_offset(), 0);
        assert_eq!(Expression::Normal.surface_offset(), 1);
        assert_eq!(Expression::Angry.surface_offset(), 8);
    }

    #[test]
    fn test_generate_surface() {
        let img = generate_surface(Character::Sakura, Expression::Happy);
        assert_eq!(img.width(), WIDTH);
        assert_eq!(img.height(), HEIGHT);

        // 3頭身の確認：頭の直径 = 84px (42*2), 全体高さ = 256px
        // 1頭分 ≈ 256/3 ≈ 85px なので正しい3頭身
        assert_eq!(HEAD_RADIUS * 2, 84); // 頭の直径
    }

    #[test]
    fn test_all_expressions() {
        let expressions = Expression::all();
        assert_eq!(expressions.len(), 9);
    }

    #[test]
    fn test_sakura_triangle_up_kero_triangle_down() {
        // 女の子: △（上向き）= 下端が広い
        // 男の子: ▽（下向き）= 上端が広い
        let sakura = generate_surface(Character::Sakura, Expression::Normal);
        let kero = generate_surface(Character::Kero, Expression::Normal);

        // 胴体上端と下端の幅を比較
        let top_y = (BODY_TOP_Y + 10) as u32; // 上端付近
        let bottom_y = (BODY_TOP_Y + BODY_HEIGHT - 10).min(HEIGHT as i32 - 1) as u32; // 下端付近

        // sakura: 下端 > 上端（△）
        let sakura_top_width = measure_width(&sakura, top_y);
        let sakura_bottom_width = measure_width(&sakura, bottom_y);
        assert!(
            sakura_bottom_width > sakura_top_width,
            "Sakura △: bottom ({}) should be wider than top ({})",
            sakura_bottom_width,
            sakura_top_width
        );

        // kero: 上端 > 下端（▽）
        let kero_top_width = measure_width(&kero, top_y);
        let kero_bottom_width = measure_width(&kero, bottom_y);
        assert!(
            kero_top_width > kero_bottom_width,
            "Kero ▽: top ({}) should be wider than bottom ({})",
            kero_top_width,
            kero_bottom_width
        );
    }

    /// 指定Y座標での不透明ピクセル幅を測定
    fn measure_width(img: &RgbaImage, y: u32) -> u32 {
        let left = (0..WIDTH).find(|&x| img.get_pixel(x, y)[3] > 0);
        let right = (0..WIDTH).rev().find(|&x| img.get_pixel(x, y)[3] > 0);
        match (left, right) {
            (Some(l), Some(r)) => r - l,
            _ => 0,
        }
    }
}
