//! 派生ゴーストの homeurl の判定（8.3〜8.6）。descript.txt の bytes を受ける純関数で、ファイルは読まない。

use super::report::{Finding, FindingKind, msg};

/// hello-pasta の更新の置き場所（descript.txt の homeurl の SSOT と同じ値。一致テストで守る）
pub const HELLO_PASTA_UPDATE_URL: &str = "https://ekicyou.github.io/pasta/hello-pasta/";

/// `descript.txt` の bytes から、派生ゴーストの homeurl の注意を作る。
/// 出さないとき（name が hello-pasta、homeurl が無い、置き場所を指さない）は None。
pub fn derived_homeurl_finding(descript: &[u8]) -> Option<Finding> {
    let descript = descript.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(descript);
    // charset が UTF-8 でなくても、キーと URL は ASCII なので読める
    let text = String::from_utf8_lossy(descript);
    let mut name = None;
    let mut homeurl = None;
    for (i, line) in text.lines().enumerate() {
        let Some((key, value)) = line.split_once(',') else {
            continue;
        };
        match key.trim().to_lowercase().as_str() {
            "name" if name.is_none() => name = Some(value.trim()),
            "homeurl" if homeurl.is_none() => homeurl = Some((i + 1, value.trim())),
            _ => {}
        }
    }
    if name == Some("hello-pasta") {
        return None;
    }
    let (line, url) = homeurl?;
    let normalize = |u: &str| {
        let u = u.to_lowercase();
        let u = u
            .strip_prefix("https://")
            .or_else(|| u.strip_prefix("http://"))
            .unwrap_or(&u);
        u.trim_end_matches('/').to_string()
    };
    let site = normalize(HELLO_PASTA_UPDATE_URL);
    let url = normalize(url);
    let points = url == site || url.starts_with(&format!("{site}/"));
    points.then(|| {
        Finding::new(
            FindingKind::HelloPastaHomeurl,
            "descript.txt",
            Some(line as u32),
            None,
            msg::hello_pasta_homeurl(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(text: &str) -> Option<Finding> {
        derived_homeurl_finding(text.as_bytes())
    }

    fn warns(homeurl: &str) -> bool {
        check(&format!("name,my-ghost\nhomeurl,{homeurl}\n")).is_some()
    }

    #[test]
    fn finding_fields() {
        let f = check(
            "charset,UTF-8\nname,my-ghost\nhomeurl,https://ekicyou.github.io/pasta/hello-pasta/\n",
        )
        .unwrap();
        assert_eq!(f.code, FindingKind::HelloPastaHomeurl);
        assert_eq!(f.path, "descript.txt");
        assert_eq!(f.line, Some(3));
        assert_eq!(f.column, None);
        assert_eq!(f.message, msg::hello_pasta_homeurl());
    }

    #[test]
    fn homeurl_normalization() {
        assert!(warns(HELLO_PASTA_UPDATE_URL));
        assert!(warns("http://ekicyou.github.io/pasta/hello-pasta/"));
        assert!(warns("HTTPS://EkiCyou.GitHub.io/Pasta/Hello-Pasta"));
        assert!(warns("https://ekicyou.github.io/pasta/hello-pasta"));
        assert!(warns("https://ekicyou.github.io/pasta/hello-pasta//"));
        assert!(warns(
            "https://ekicyou.github.io/pasta/hello-pasta/sub/dir/"
        ));
        assert!(!warns("https://ekicyou.github.io/pasta/hello-pasta-foo/"));
        assert!(!warns("https://ekicyou.github.io/pasta/"));
        assert!(!warns("https://example.com/my-ghost/"));
    }

    #[test]
    fn name_rules() {
        let url = HELLO_PASTA_UPDATE_URL;
        assert!(check(&format!("name,hello-pasta\nhomeurl,{url}\n")).is_none());
        assert!(check(&format!("name , hello-pasta \nhomeurl,{url}\n")).is_none());
        // 大文字小文字の違いは別名
        assert!(check(&format!("name,Hello-Pasta\nhomeurl,{url}\n")).is_some());
        // name が無いときは hello-pasta でないものとして続ける
        assert!(check(&format!("homeurl,{url}\n")).is_some());
    }

    #[test]
    fn no_homeurl_is_none() {
        assert!(check("name,my-ghost\ncraftman,x\n").is_none());
        assert!(check("").is_none());
    }

    #[test]
    fn bom_crlf_and_non_utf8() {
        let url = HELLO_PASTA_UPDATE_URL;
        let mut bytes = b"\xEF\xBB\xBFname,my-ghost\r\n".to_vec();
        bytes.extend(format!("HomeURL , {url} \r\n").as_bytes());
        assert_eq!(derived_homeurl_finding(&bytes).unwrap().line, Some(2));
        // charset が Shift_JIS の name（UTF-8 でない bytes）でも読める
        let mut sjis = b"charset,Shift_JIS\r\nname,\x82\xA0\x82\xA2\r\n".to_vec();
        sjis.extend(format!("homeurl,{url}\r\n").as_bytes());
        assert_eq!(derived_homeurl_finding(&sjis).unwrap().line, Some(3));
        // BOM 付きの先頭行の name が hello-pasta なら出さない
        let mut hp = b"\xEF\xBB\xBFname,hello-pasta\r\n".to_vec();
        hp.extend(format!("homeurl,{url}\r\n").as_bytes());
        assert!(derived_homeurl_finding(&hp).is_none());
    }

    #[test]
    fn duplicate_keys_first_wins() {
        let url = HELLO_PASTA_UPDATE_URL;
        assert!(check(&format!("name,hello-pasta\nname,my-ghost\nhomeurl,{url}\n")).is_none());
        assert!(check(&format!("name,my-ghost\nname,hello-pasta\nhomeurl,{url}\n")).is_some());
        assert!(
            check(&format!(
                "name,my-ghost\nhomeurl,https://example.com/\nhomeurl,{url}\n"
            ))
            .is_none()
        );
        let f = check(&format!(
            "name,my-ghost\nhomeurl,{url}\nhomeurl,https://example.com/\n"
        ))
        .unwrap();
        assert_eq!(f.line, Some(2));
    }

    /// SSOT は hello-pasta の descript.txt。定数がそれと同じ値であること
    #[test]
    fn constant_matches_hello_pasta_descript() {
        let text = include_str!(
            "../../../pasta_sample_ghost/ghosts/hello-pasta/ghost/master/descript.txt"
        );
        let homeurl = text
            .lines()
            .find_map(|l| l.trim_start_matches('\u{FEFF}').strip_prefix("homeurl,"))
            .map(str::trim);
        assert_eq!(homeurl, Some(HELLO_PASTA_UPDATE_URL));
        // hello-pasta 自身には出さない
        assert!(derived_homeurl_finding(text.as_bytes()).is_none());
    }
}
