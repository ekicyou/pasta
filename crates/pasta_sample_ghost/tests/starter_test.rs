//! 入門の出発点 pasta-starter.nar（my-ghost）の、リポジトリに置く starter/ の素材の検証
//!
//! nar は release.ps1 が starter/ に hello-pasta の dic/01-boot.pasta・shell/・pasta.dll・
//! THIRD_PARTY_LICENSES.txt を足して作る。ここでは starter/ が決まりの 3 ファイルだけを持つことを確かめる。

use std::path::{Path, PathBuf};

fn starter_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("starter")
}

fn files(dir: &Path, base: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read starter dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            files(&path, base, out);
        } else {
            let rel = path.strip_prefix(base).expect("relative");
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// 辞書・シェルは hello-pasta が正本なので starter/ に写しを持たない
#[test]
fn starter_holds_only_its_own_files() {
    let mut got = Vec::new();
    files(&starter_dir(), &starter_dir(), &mut got);
    got.sort();
    assert_eq!(
        got,
        [
            "ghost/master/descript.txt",
            "ghost/master/pasta.toml",
            "install.txt"
        ]
    );
}

#[test]
fn starter_install_and_descript() {
    let install = std::fs::read_to_string(starter_dir().join("install.txt")).unwrap();
    assert_eq!(
        install.lines().collect::<Vec<_>>(),
        [
            "charset,UTF-8",
            "type,ghost",
            "name,my-ghost",
            "directory,my-ghost",
            "accept,"
        ]
    );

    let descript =
        std::fs::read_to_string(starter_dir().join("ghost/master/descript.txt")).unwrap();
    assert_eq!(descript.lines().next(), Some("charset,UTF-8"));
    for line in [
        "type,ghost",
        "name,my-ghost",
        "sakura.name,女の子",
        "kero.name,男の子",
        "craftman,your-name",
        "craftmanw,あなたの名前",
        "shiori,pasta.dll",
    ] {
        assert!(
            descript.lines().any(|l| l == line),
            "descript.txt に {line} が無い"
        );
    }
    // ネットワーク更新しない
    assert!(
        !descript.contains("homeurl"),
        "descript.txt に homeurl が在る"
    );
}

#[test]
fn starter_pasta_toml() {
    // autocrlf で CRLF に変わっても同じに読む
    let toml = std::fs::read_to_string(starter_dir().join("ghost/master/pasta.toml"))
        .unwrap()
        .replace("\r\n", "\n");
    let sections: Vec<_> = toml.lines().filter(|l| l.starts_with('[')).collect();
    assert_eq!(
        sections,
        [r#"[actor."女の子"]"#, r#"[actor."男の子"]"#, "[debug]"]
    );
    assert!(toml.contains("[actor.\"女の子\"]\nspot = 0"));
    assert!(toml.contains("[actor.\"男の子\"]\nspot = 1"));
    assert!(toml.contains("enabled = true"));
}
