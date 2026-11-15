//! 窓の中の並び（TOML）。ファイルが正本。毎回読み、変えたら書く。
//! 窓の大きさに合わせた丸めはここではしない（画面側 layout.ts の持ち分）。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILE: &str = "layout.toml";

/// 画面との受け渡し（camelCase）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    /// ツリーの幅（CSS px）
    pub tree_width: u32,
    /// 下の欄の面の高さ（帯を除く。CSS px）
    pub panels_height: u32,
    /// 下の欄の左の欄の幅の割合（0.0〜1.0。常に有限）
    pub left_share: f64,
    /// true で保存変数が左・出力が右
    pub swapped: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            tree_width: 260,
            panels_height: 180,
            left_share: 0.5,
            swapped: false,
        }
    }
}

/// ファイルの型（snake_case・欠けたキーは None）
#[derive(Serialize, Deserialize)]
struct LayoutFile {
    tree_width: Option<u32>,
    panels_height: Option<u32>,
    left_share: Option<f64>,
    swapped: Option<bool>,
}

/// 読めない・型が合わないなら全部既定。欠けたキー・範囲の外の割合はそのキーだけ既定
pub fn parse(text: &str) -> Layout {
    let d = Layout::default();
    let Ok(f) = toml::from_str::<LayoutFile>(text) else {
        return d;
    };
    Layout {
        tree_width: f.tree_width.unwrap_or(d.tree_width),
        panels_height: f.panels_height.unwrap_or(d.panels_height),
        left_share: f
            .left_share
            .filter(|s| (0.0..=1.0).contains(s))
            .unwrap_or(d.left_share),
        swapped: f.swapped.unwrap_or(d.swapped),
    }
}

/// 人が読める TOML
pub fn render(layout: &Layout) -> String {
    let body = toml::to_string(&LayoutFile {
        tree_width: Some(layout.tree_width),
        panels_height: Some(layout.panels_height),
        // 非有限は書けない（読むと既定になる値を書かない）
        left_share: Some(if (0.0..=1.0).contains(&layout.left_share) {
            layout.left_share
        } else {
            Layout::default().left_share
        }),
        swapped: Some(layout.swapped),
    })
    .expect("数と真偽値は必ず TOML にできる");
    format!("# pasta-recipe の窓の中の並び（仕切りの位置と、出力と保存変数の左右）\n{body}")
}

/// 読めなければ既定
pub fn load(dir: &Path) -> Layout {
    fs::read_to_string(dir.join(FILE))
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// フォルダが無ければ作る
pub fn save(dir: &Path, layout: &Layout) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("{} を作れません: {e}", dir.display()))?;
    let path = dir.join(FILE);
    fs::write(&path, render(layout)).map_err(|e| format!("{} に書けません: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared(l: f64) -> Layout {
        Layout {
            left_share: l,
            ..Layout::default()
        }
    }

    #[test]
    fn missing_file_is_default() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(load(&tmp.path().join("none")), Layout::default());
    }

    #[test]
    fn broken_or_wrong_type_is_all_default() {
        assert_eq!(parse("tree_width = ["), Layout::default());
        assert_eq!(parse(""), Layout::default());
        assert_eq!(
            parse("tree_width = \"wide\"\npanels_height = 300\nswapped = true"),
            Layout::default()
        );
        assert_eq!(parse("tree_width = -5\nswapped = true"), Layout::default());
        assert_eq!(parse("swapped = 1\ntree_width = 300"), Layout::default());
    }

    #[test]
    fn missing_key_is_default_only_for_that_key() {
        let l = parse("panels_height = 300\nswapped = true");
        assert_eq!(
            l,
            Layout {
                tree_width: 260,
                panels_height: 300,
                left_share: 0.5,
                swapped: true
            }
        );
    }

    #[test]
    fn bad_left_share_is_default_only_for_that_key() {
        for s in ["1.5", "-0.1", "nan", "inf", "-inf", "2"] {
            let l = parse(&format!("tree_width = 300\nleft_share = {s}"));
            assert_eq!(l.left_share, 0.5, "{s}");
            assert_eq!(l.tree_width, 300, "{s}");
        }
        assert_eq!(parse("left_share = 0").left_share, 0.0);
        assert_eq!(parse("left_share = 1").left_share, 1.0);
    }

    #[test]
    fn render_has_header_comment_and_round_trips() {
        let l = Layout {
            tree_width: 321,
            panels_height: 222,
            left_share: 0.37,
            swapped: true,
        };
        let text = render(&l);
        assert!(
            text.starts_with(
                "# pasta-recipe の窓の中の並び（仕切りの位置と、出力と保存変数の左右）\n"
            ),
            "{text}"
        );
        assert_eq!(parse(&text), l);
        assert_eq!(parse(&render(&shared(f64::NAN))).left_share, 0.5);
    }

    #[test]
    fn save_then_load_round_trips_and_makes_the_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("io.github.ekicyou.pasta-recipe");
        let l = Layout {
            tree_width: 300,
            panels_height: 240,
            left_share: 0.25,
            swapped: true,
        };
        save(&dir, &l).unwrap();
        assert!(dir.join("layout.toml").is_file());
        assert_eq!(load(&dir), l);
    }
}
