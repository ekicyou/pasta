//! 最近開いた ghost/master の記録（TOML・新しい順に 10 件）。
//! ファイルが正本。毎回読み、変えたら書く（メモリに写しを持たない）。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILE: &str = "recent.toml";
const MAX: usize = 10;

/// name はゴーストのフォルダの名前（master の 2 つ上）
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecentEntry {
    pub master: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct RecentFile {
    ghosts: Vec<String>,
}

/// NTFS は ASCII の外も大文字・小文字を区別しないので、大文字へ揃えて比べる（paths::check_name と同じ）
fn same(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
}

/// 壊れていれば空（3.8）
pub fn parse(text: &str) -> Vec<String> {
    toml::from_str::<RecentFile>(text)
        .map(|f| f.ghosts)
        .unwrap_or_default()
}

/// 先頭へ・大文字小文字を区別しない重なりを消す・10 件（3.1・3.2）
pub fn push(list: Vec<String>, master: &str) -> Vec<String> {
    let mut out = vec![master.to_string()];
    out.extend(remove(list, master));
    out.truncate(MAX);
    out
}

pub fn remove(list: Vec<String>, master: &str) -> Vec<String> {
    list.into_iter().filter(|m| !same(m, master)).collect()
}

/// 人が読める TOML（3.7）
pub fn render(list: &[String]) -> String {
    let body = toml::to_string(&RecentFile {
        ghosts: list.to_vec(),
    })
    .expect("文字列の配列は必ず TOML にできる");
    format!("# pasta-recipe が最近開いたゴースト（新しい順・最大 10 件）\n{body}")
}

/// 読めなければ空（3.8）
pub fn load(dir: &Path) -> Vec<String> {
    fs::read_to_string(dir.join(FILE))
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// フォルダが無ければ作る
pub fn save(dir: &Path, list: &[String]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("{} を作れません: {e}", dir.display()))?;
    let path = dir.join(FILE);
    fs::write(&path, render(list)).map_err(|e| format!("{} に書けません: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn push_puts_new_one_first() {
        let l = push(list(&[r"C:\a\ghost\master"]), r"C:\b\ghost\master");
        assert_eq!(l, list(&[r"C:\b\ghost\master", r"C:\a\ghost\master"]));
    }

    #[test]
    fn push_moves_existing_to_front_ignoring_case() {
        let l = push(
            list(&[r"C:\a\ghost\master", r"C:\Ｂ\ghost\master"]),
            r"c:\ｂ\GHOST\master",
        );
        assert_eq!(l, list(&[r"c:\ｂ\GHOST\master", r"C:\a\ghost\master"]));
    }

    #[test]
    fn push_drops_the_eleventh() {
        let mut l = Vec::new();
        for i in 0..11 {
            l = push(l, &format!(r"C:\g{i}\ghost\master"));
        }
        assert_eq!(l.len(), 10);
        assert_eq!(l[0], r"C:\g10\ghost\master");
        assert_eq!(l[9], r"C:\g1\ghost\master");
    }

    #[test]
    fn remove_ignores_case() {
        let l = remove(
            list(&[r"C:\a\ghost\master", r"C:\b\ghost\master"]),
            r"c:\A\ghost\master",
        );
        assert_eq!(l, list(&[r"C:\b\ghost\master"]));
    }

    #[test]
    fn broken_toml_is_empty() {
        assert!(parse("ghosts = [").is_empty());
        assert!(parse("ghosts = 3").is_empty());
        assert!(parse("").is_empty());
    }

    #[test]
    fn render_then_parse_round_trips() {
        let l = list(&[r"C:\ghosts\hello\ghost\master", r"D:\dev\emo2\ghost\master"]);
        let text = render(&l);
        assert!(text.starts_with("# "), "{text}");
        assert!(text.contains(r"'C:\ghosts\hello\ghost\master'"), "{text}");
        assert_eq!(parse(&text), l);
    }

    #[test]
    fn save_then_load_round_trips_and_makes_the_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("io.github.ekicyou.pasta-recipe");
        assert!(load(&dir).is_empty());
        let l = list(&[r"C:\a\ghost\master"]);
        save(&dir, &l).unwrap();
        assert!(dir.join("recent.toml").is_file());
        assert_eq!(load(&dir), l);
    }
}
