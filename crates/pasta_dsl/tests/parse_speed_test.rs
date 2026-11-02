//! 解析の速さのテスト（要件 1.1・1.2）。
//!
//! 時間を測るので、ほかのテストと並んで走らないよう 1 本だけの別のファイルに置く。

use pasta_dsl::parser::parse_str;
use std::time::{Duration, Instant};

/// 1 シーン 4 行の辞書を `lines` 行ぶん作る。
fn dict(lines: usize) -> String {
    let scene = "＊挨拶\n  さくら：こんにちは。\n  うにゅう：やあ、元気？\n  ＄x＝（１＋２）＊３\n";
    scene.repeat(lines / 4)
}

/// 3 回解析した最短の時間（機械の揺れを抑える）。
fn best_of_3(src: &str) -> Duration {
    (0..3)
        .map(|_| {
            let t = Instant::now();
            let file = parse_str(src, "speed.pasta").expect("解析に成功する");
            let elapsed = t.elapsed();
            assert!(!file.items.is_empty());
            elapsed
        })
        .min()
        .unwrap()
}

fn ratio(small: usize, large: usize) -> (Duration, Duration, f64) {
    let a = best_of_3(&dict(small));
    let b = best_of_3(&dict(large));
    (a, b, b.as_secs_f64() / a.as_secs_f64())
}

#[test]
fn parse_time_is_linear_in_lines() {
    let (a, b, r) = ratio(2_000, 8_000);
    assert!(r <= 8.0, "2,000 行 {a:?}・8,000 行 {b:?}・比 {r:.1} 倍");
}

/// `cargo test -p pasta_dsl --release --test parse_speed_test -- --ignored`
#[test]
#[ignore]
fn release_parse_8000_lines_under_1s_and_linear() {
    let (a, b, r) = ratio(8_000, 32_000);
    assert!(a < Duration::from_secs(1), "8,000 行 {a:?}");
    assert!(r <= 8.0, "8,000 行 {a:?}・32,000 行 {b:?}・比 {r:.1} 倍");
}
