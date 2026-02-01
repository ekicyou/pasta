//! ピクトグラム画像生成
//!
//! トイレマーク風のシンプルな人型アイコンを生成します。
//! 構成：○（頭）+ △/□（胴体）のみ。手足なし。

use crate::GhostError;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use std::path::Path;

/// キャラクター種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    /// 女の子（sakura）- スカート（三角形）
    Sakura,
    /// 男の子（kero）- ズボン（四角形）
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

/// 頭の半径（約40px、3頭身の1頭分）
const HEAD_RADIUS: i32 = 35;
/// 頭の中心Y座標
const HEAD_CENTER_Y: i32 = 45;
/// 胴体開始Y座標（頭の下端 + 少し隙間）
const BODY_TOP_Y: i32 = HEAD_CENTER_Y + HEAD_RADIUS + 5;
/// 胴体高さ（画像下端まで）
const BODY_HEIGHT: i32 = HEIGHT as i32 - BODY_TOP_Y - 5;

/// キャラクター色
fn character_color(character: Character) -> Rgba<u8> {
    match character {
        Character::Sakura => Rgba([74, 144, 217, 255]), // ライトブルー #4A90D9
        Character::Kero => Rgba([74, 217, 138, 255]),   // ライトグリーン #4AD98A
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
/// - 胴体: 女の子=三角形（スカート）、男の子=四角形
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
/// - 女の子: 三角形（スカート）- 上が細く下が広い
/// - 男の子: 四角形（ズボン）- ストレート
fn draw_body(img: &mut RgbaImage, character: Character, color: Rgba<u8>) {
    let center_x = WIDTH as i32 / 2;
    let body_bottom_y = BODY_TOP_Y + BODY_HEIGHT;

    match character {
        Character::Sakura => {
            // 三角形（スカート）: 頂点が上、底辺が下
            // 上端は狭く（肩幅程度）、下端は広く
            let top_half_width = 20; // 上端の半幅
            let bottom_half_width = 55; // 下端の半幅

            for y in BODY_TOP_Y..body_bottom_y {
                let ratio = (y - BODY_TOP_Y) as f32 / BODY_HEIGHT as f32;
                let half_width =
                    top_half_width as f32 + (bottom_half_width - top_half_width) as f32 * ratio;
                let left = (center_x as f32 - half_width) as u32;
                let right = (center_x as f32 + half_width) as u32;

                for x in left..right.min(WIDTH) {
                    img.put_pixel(x, y as u32, color);
                }
            }
        }
        Character::Kero => {
            // 四角形（ズボン）: 一定幅のストレート
            let half_width = 30; // 固定幅
            let left = (center_x - half_width) as u32;
            let right = (center_x + half_width) as u32;

            for y in BODY_TOP_Y..body_bottom_y {
                for x in left..right.min(WIDTH) {
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
    let eye_y = HEAD_CENTER_Y - 5; // 顔の中央やや上
    let left_eye_x = center_x - 12;
    let right_eye_x = center_x + 12;

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
            draw_horizontal_line(img, left_eye_x, eye_y, 8, white);
            draw_horizontal_line(img, right_eye_x, eye_y, 8, white);
        }
        Expression::Shy => {
            // > < 照れ
            draw_greater_than(img, left_eye_x, eye_y, white);
            draw_less_than(img, right_eye_x, eye_y, white);
        }
        Expression::Surprised => {
            // o o 驚き（円）
            draw_filled_circle_mut(img, (left_eye_x, eye_y), 5, white);
            draw_filled_circle_mut(img, (right_eye_x, eye_y), 5, white);
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
            draw_horizontal_line(img, left_eye_x, eye_y - 3, 8, white);
            draw_horizontal_line(img, left_eye_x, eye_y + 3, 8, white);
            draw_horizontal_line(img, right_eye_x, eye_y - 3, 8, white);
            draw_horizontal_line(img, right_eye_x, eye_y + 3, 8, white);
        }
        Expression::Angry => {
            // # # 怒り（ハッシュ）
            draw_hash(img, left_eye_x, eye_y, white);
            draw_hash(img, right_eye_x, eye_y, white);
        }
    }
}

// === 表情用ヘルパー関数 ===

/// 横線を描画
fn draw_horizontal_line(img: &mut RgbaImage, cx: i32, cy: i32, half_len: i32, color: Rgba<u8>) {
    for dx in -half_len..=half_len {
        let x = (cx + dx) as u32;
        let y = cy as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// 縦線を描画
fn draw_vertical_line(img: &mut RgbaImage, cx: i32, cy: i32, half_len: i32, color: Rgba<u8>) {
    for dy in -half_len..=half_len {
        let x = cx as u32;
        let y = (cy + dy) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// ^ 山形を描画
fn draw_caret(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    // 左斜線
    for i in 0..6 {
        let x = (cx - 5 + i) as u32;
        let y = (cy + 4 - i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
    // 右斜線
    for i in 0..6 {
        let x = (cx + i) as u32;
        let y = (cy - 1 + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// > を描画
fn draw_greater_than(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    for i in 0..5 {
        let x = (cx - 3 + i) as u32;
        let y = (cy - 4 + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
    for i in 0..5 {
        let x = (cx + 1 - i) as u32;
        let y = (cy + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// < を描画
fn draw_less_than(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    for i in 0..5 {
        let x = (cx + 3 - i) as u32;
        let y = (cy - 4 + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
    for i in 0..5 {
        let x = (cx - 1 + i) as u32;
        let y = (cy + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// ; セミコロンを描画
fn draw_semicolon(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    // 上の点
    draw_filled_circle_mut(img, (cx, cy - 4), 2, color);
    // 下の点（少し下に流れる）
    draw_filled_circle_mut(img, (cx, cy + 2), 2, color);
    // 涙のしずく
    for i in 0..4 {
        let x = cx as u32;
        let y = (cy + 4 + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// @ 渦巻きを描画（簡略版：二重円）
fn draw_at_sign(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>, bg_color: Rgba<u8>) {
    // 外円
    draw_filled_circle_mut(img, (cx, cy), 6, color);
    // 内円（背景色で抜く）→ドーナツ状
    draw_filled_circle_mut(img, (cx, cy), 3, bg_color);
    // 中央に点
    draw_filled_circle_mut(img, (cx, cy), 1, color);
}

/// * 星を描画（十字+斜め十字）
fn draw_star(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    draw_horizontal_line(img, cx, cy, 5, color);
    draw_vertical_line(img, cx, cy, 5, color);
    // 斜め線
    for i in -4..=4 {
        let x = (cx + i) as u32;
        let y = (cy + i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
    for i in -4..=4 {
        let x = (cx + i) as u32;
        let y = (cy - i) as u32;
        if x < WIDTH && y < HEIGHT {
            img.put_pixel(x, y, color);
        }
    }
}

/// # ハッシュを描画
fn draw_hash(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    // 横線2本
    draw_horizontal_line(img, cx, cy - 3, 6, color);
    draw_horizontal_line(img, cx, cy + 3, 6, color);
    // 縦線2本
    draw_vertical_line(img, cx - 3, cy, 6, color);
    draw_vertical_line(img, cx + 3, cy, 6, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_color() {
        let sakura_color = character_color(Character::Sakura);
        assert_eq!(sakura_color, Rgba([74, 144, 217, 255]));

        let kero_color = character_color(Character::Kero);
        assert_eq!(kero_color, Rgba([74, 217, 138, 255]));
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

        // 3頭身の確認：頭が全体の1/3程度
        // 頭の直径 = 80px, 全体高さ = 256px → 約31%
    }

    #[test]
    fn test_all_expressions() {
        let expressions = Expression::all();
        assert_eq!(expressions.len(), 9);
    }

    #[test]
    fn test_sakura_is_triangle_kero_is_rectangle() {
        // 女の子のサーフェス生成
        let sakura = generate_surface(Character::Sakura, Expression::Normal);
        // 男の子のサーフェス生成
        let kero = generate_surface(Character::Kero, Expression::Normal);

        // 胴体下端の幅をチェック
        let bottom_y = (BODY_TOP_Y + BODY_HEIGHT - 1) as u32;

        // sakura: 下端が広い（三角形）- 左端から右端までの幅を測定
        let sakura_left = (0..WIDTH).find(|&x| sakura.get_pixel(x, bottom_y)[3] > 0);
        let sakura_right = (0..WIDTH).rev().find(|&x| sakura.get_pixel(x, bottom_y)[3] > 0);

        // kero: 一定幅（四角形）
        let kero_left = (0..WIDTH).find(|&x| kero.get_pixel(x, bottom_y)[3] > 0);
        let kero_right = (0..WIDTH).rev().find(|&x| kero.get_pixel(x, bottom_y)[3] > 0);

        // sakura の幅 > kero の幅（下端で）
        if let (Some(sl), Some(sr), Some(kl), Some(kr)) =
            (sakura_left, sakura_right, kero_left, kero_right)
        {
            let sakura_width = sr - sl;
            let kero_width = kr - kl;
            assert!(
                sakura_width > kero_width,
                "Sakura should have wider bottom (triangle: {}) than Kero (rectangle: {})",
                sakura_width,
                kero_width
            );
        }
    }
}
