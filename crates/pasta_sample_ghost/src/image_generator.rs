//! ピクトグラム画像生成
//!
//! トイレマーク風のシンプルな人型アイコンを生成します。

use crate::GhostError;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};
use std::path::Path;

/// キャラクター種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    /// 女の子（sakura）
    Sakura,
    /// 男の子（kero）
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

/// 画像サイズ
const WIDTH: u32 = 128;
const HEIGHT: u32 = 256;

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
pub fn generate_surface(character: Character, expression: Expression) -> RgbaImage {
    let mut img = RgbaImage::new(WIDTH, HEIGHT);
    let color = character_color(character);

    // 頭部（円）
    draw_filled_circle_mut(&mut img, (64, 50), 30, color);

    // 胴体（四角形を塗りつぶし - 台形の代用）
    draw_body(&mut img, character, color);

    // 手（線）
    draw_arms(&mut img, color);

    // 足（線）
    draw_legs(&mut img, color);

    // 表情を描画
    draw_expression(&mut img, expression);

    // kero の場合は耳を追加
    if character == Character::Kero {
        draw_ears(&mut img, color);
    }

    img
}

/// 胴体を描画（台形の代用として四角形）
fn draw_body(img: &mut RgbaImage, character: Character, color: Rgba<u8>) {
    let top_y = 90;
    let bottom_y = 180;

    // キャラクターによって幅を変える
    let (top_left, top_right, bottom_left, bottom_right) = match character {
        Character::Sakura => (44, 84, 34, 94), // スカート風に下が広い
        Character::Kero => (44, 84, 44, 84),   // ズボン風にストレート
    };

    // 単純な四角形で塗りつぶし
    for y in top_y..bottom_y {
        let ratio = (y - top_y) as f32 / (bottom_y - top_y) as f32;
        let left = top_left as f32 + (bottom_left - top_left) as f32 * ratio;
        let right = top_right as f32 + (bottom_right - top_right) as f32 * ratio;

        for x in left as u32..right as u32 {
            if x < WIDTH {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// 腕を描画
fn draw_arms(img: &mut RgbaImage, color: Rgba<u8>) {
    // 左腕
    draw_line_segment_mut(img, (44.0, 100.0), (20.0, 140.0), color);
    // 右腕
    draw_line_segment_mut(img, (84.0, 100.0), (108.0, 140.0), color);
}

/// 足を描画
fn draw_legs(img: &mut RgbaImage, color: Rgba<u8>) {
    // 左足
    draw_line_segment_mut(img, (54.0, 180.0), (44.0, 240.0), color);
    // 右足
    draw_line_segment_mut(img, (74.0, 180.0), (84.0, 240.0), color);
}

/// 耳を描画（kero専用）
fn draw_ears(img: &mut RgbaImage, color: Rgba<u8>) {
    // 左耳（三角形）
    draw_line_segment_mut(img, (40.0, 40.0), (30.0, 20.0), color);
    draw_line_segment_mut(img, (30.0, 20.0), (50.0, 30.0), color);

    // 右耳（三角形）
    draw_line_segment_mut(img, (88.0, 40.0), (98.0, 20.0), color);
    draw_line_segment_mut(img, (98.0, 20.0), (78.0, 30.0), color);
}

/// 表情を描画（線描画、フォント不使用）
fn draw_expression(img: &mut RgbaImage, expression: Expression) {
    let black = Rgba([0, 0, 0, 255]);
    let eye_y = 45.0;
    let left_eye_x = 52.0;
    let right_eye_x = 76.0;

    match expression {
        Expression::Happy => {
            // ^ ^ 笑顔
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y + 3.0),
                (left_eye_x, eye_y - 3.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x, eye_y - 3.0),
                (left_eye_x + 5.0, eye_y + 3.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y + 3.0),
                (right_eye_x, eye_y - 3.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x, eye_y - 3.0),
                (right_eye_x + 5.0, eye_y + 3.0),
                black,
            );
        }
        Expression::Normal => {
            // - - 通常
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y),
                (left_eye_x + 5.0, eye_y),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y),
                (right_eye_x + 5.0, eye_y),
                black,
            );
        }
        Expression::Shy => {
            // > < 照れ
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y - 3.0),
                (left_eye_x, eye_y),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x, eye_y),
                (left_eye_x - 5.0, eye_y + 3.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x + 5.0, eye_y - 3.0),
                (right_eye_x, eye_y),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x, eye_y),
                (right_eye_x + 5.0, eye_y + 3.0),
                black,
            );
        }
        Expression::Surprised => {
            // o o 驚き
            draw_filled_circle_mut(img, (left_eye_x as i32, eye_y as i32), 4, black);
            draw_filled_circle_mut(img, (right_eye_x as i32, eye_y as i32), 4, black);
        }
        Expression::Crying => {
            // ; ; 泣き
            draw_line_segment_mut(
                img,
                (left_eye_x, eye_y - 3.0),
                (left_eye_x, eye_y + 8.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 3.0, eye_y + 2.0),
                (left_eye_x - 3.0, eye_y + 10.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x, eye_y - 3.0),
                (right_eye_x, eye_y + 8.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x + 3.0, eye_y + 2.0),
                (right_eye_x + 3.0, eye_y + 10.0),
                black,
            );
        }
        Expression::Confused => {
            // @ @ 困惑（渦巻き風）
            draw_filled_circle_mut(img, (left_eye_x as i32, eye_y as i32), 5, black);
            draw_filled_circle_mut(
                img,
                (left_eye_x as i32, eye_y as i32),
                2,
                Rgba([255, 255, 255, 255]),
            );
            draw_filled_circle_mut(img, (right_eye_x as i32, eye_y as i32), 5, black);
            draw_filled_circle_mut(
                img,
                (right_eye_x as i32, eye_y as i32),
                2,
                Rgba([255, 255, 255, 255]),
            );
        }
        Expression::Sparkle => {
            // * * キラキラ
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y),
                (left_eye_x + 5.0, eye_y),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x, eye_y - 5.0),
                (left_eye_x, eye_y + 5.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 4.0, eye_y - 4.0),
                (left_eye_x + 4.0, eye_y + 4.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 4.0, eye_y + 4.0),
                (left_eye_x + 4.0, eye_y - 4.0),
                black,
            );

            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y),
                (right_eye_x + 5.0, eye_y),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x, eye_y - 5.0),
                (right_eye_x, eye_y + 5.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 4.0, eye_y - 4.0),
                (right_eye_x + 4.0, eye_y + 4.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 4.0, eye_y + 4.0),
                (right_eye_x + 4.0, eye_y - 4.0),
                black,
            );
        }
        Expression::Sleepy => {
            // = = 眠い
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y - 2.0),
                (left_eye_x + 5.0, eye_y - 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y + 2.0),
                (left_eye_x + 5.0, eye_y + 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y - 2.0),
                (right_eye_x + 5.0, eye_y - 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y + 2.0),
                (right_eye_x + 5.0, eye_y + 2.0),
                black,
            );
        }
        Expression::Angry => {
            // # # 怒り（ハッシュマーク風）
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y - 2.0),
                (left_eye_x + 5.0, eye_y - 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 5.0, eye_y + 2.0),
                (left_eye_x + 5.0, eye_y + 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x - 2.0, eye_y - 5.0),
                (left_eye_x - 2.0, eye_y + 5.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (left_eye_x + 2.0, eye_y - 5.0),
                (left_eye_x + 2.0, eye_y + 5.0),
                black,
            );

            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y - 2.0),
                (right_eye_x + 5.0, eye_y - 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 5.0, eye_y + 2.0),
                (right_eye_x + 5.0, eye_y + 2.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x - 2.0, eye_y - 5.0),
                (right_eye_x - 2.0, eye_y + 5.0),
                black,
            );
            draw_line_segment_mut(
                img,
                (right_eye_x + 2.0, eye_y - 5.0),
                (right_eye_x + 2.0, eye_y + 5.0),
                black,
            );
        }
    }
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
    }

    #[test]
    fn test_all_expressions() {
        let expressions = Expression::all();
        assert_eq!(expressions.len(), 9);
    }
}
