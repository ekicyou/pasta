//! pasta.dll と同じ辞書ファイルと辞書モジュールを選び出す段（2.1〜2.3・2.6〜2.8・11.8）。
//!
//! pasta.dll（`loader/discovery.rs`・`process.rs`）は Windows で `glob::glob` を使う。その振る舞いを
//! ファイルの一覧の上で写す: ワイルドカードの無い成分はパターンの綴りでファイルシステムを引くので
//! 大文字・小文字を区別せず、ある成分は `Pattern::matches`（既定の MatchOptions）で区別する。
//! `glob::glob` はフォルダにも一致し、pasta.dll はそのフォルダを辞書として読もうとして失敗する（2.1）。

use glob::Pattern;

use super::config::{GhostConfig, PATH};
use super::{Finding, FindingKind, GhostInput, msg};

/// 選び出した 1 ファイル
#[derive(Debug, Clone)]
pub(crate) struct DictFile {
    /// 実物の綴り（報告のパス）
    pub path: String,
    /// pasta.dll が見るパス（ワイルドカードの無い成分はパターンの綴り）。モジュール名はこれから作る
    pub dll_path: String,
}

/// 辞書ファイル（.pasta）と辞書モジュール（.lua）と辞書のパターンに一致したフォルダ。
/// どれも実物の相対パスの昇順で、重複なし
#[derive(Debug, Clone, Default)]
pub(crate) struct Discovered {
    pub pasta: Vec<DictFile>,
    pub lua: Vec<DictFile>,
    /// .pasta と .lua のパターンの和に一致したフォルダ。ghost/master そのものはパス ""
    pub folders: Vec<DictFile>,
}

pub(crate) fn discover(input: &GhostInput, config: &GhostConfig) -> (Discovered, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut lua_patterns = Vec::new();
    let mut pasta_comps = Vec::new();
    for pattern in &config.pasta_patterns {
        match pattern.strip_suffix(".pasta") {
            Some(stem) => lua_patterns.push(format!("{stem}.lua")),
            None => findings.push(on_toml(
                FindingKind::PatternNotPasta,
                msg::pattern_not_pasta(pattern),
            )),
        }
        if is_traversal(pattern) {
            findings.push(on_toml(
                FindingKind::PatternIgnored,
                msg::pattern_ignored(pattern),
            ));
            continue;
        }
        match parse(pattern) {
            Ok(comps) => pasta_comps.push(comps),
            Err(e) => findings.push(on_toml(
                FindingKind::ConfigPatternInvalid,
                msg::config_pattern_invalid(pattern, e.msg),
            )),
        }
    }
    // .lua のパターンは読める .pasta のパターンから作るので、読めない・.. を含むものは上で報告済み
    let lua_comps: Vec<_> = lua_patterns
        .iter()
        .filter(|p| !is_traversal(p))
        .filter_map(|p| parse(p).ok())
        .collect();

    // フォルダの候補は ghost/master そのもの（""）と入力のフォルダ
    let dirs: Vec<String> = std::iter::once(String::new())
        .chain(input.dirs.iter().cloned())
        .collect();
    let both: Vec<Parsed> = pasta_comps.iter().chain(&lua_comps).cloned().collect();
    let found = Discovered {
        pasta: collect(&input.entries, &pasta_comps, false),
        lua: collect(&input.entries, &lua_comps, false),
        folders: collect(&dirs, &both, true),
    };

    // 両方のパターンに一致しても 1 件。入れ子はそれぞれ（pasta.dll がそれぞれ読もうとして失敗する）
    for folder in &found.folders {
        findings.push(Finding::new(
            FindingKind::FolderMatchesPattern,
            &folder.path,
            None,
            None,
            msg::folder_matches_pattern(),
        ));
    }

    for file in found.pasta.iter().chain(&found.lua) {
        let name = file.dll_path.rsplit('/').next().unwrap_or_default();
        if name == "init.pasta" || name == "init.lua" {
            findings.push(Finding::new(
                FindingKind::InitFileName,
                &file.path,
                None,
                None,
                msg::init_file_name(name),
            ));
        }
    }
    if found.pasta.is_empty() {
        findings.push(on_toml(
            FindingKind::NoDictionaryFiles,
            msg::no_dictionary_files(),
        ));
    }
    (found, findings)
}

/// pasta.toml に付ける問題（分類は種類で決まる）
fn on_toml(code: FindingKind, message: String) -> Finding {
    Finding::new(code, PATH, None, None, message)
}

/// pasta.dll の `contains_traversal`（Windows の `Path::components`）: `..`・ルート・ドライブを含む
fn is_traversal(pattern: &str) -> bool {
    let b = pattern.as_bytes();
    pattern.starts_with(['/', '\\'])
        || (b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic())
        || pattern.split(['/', '\\']).any(|c| c == "..")
}

#[derive(Clone)]
enum Comp {
    /// ワイルドカードの無い成分（大文字・小文字を区別しない）
    Literal(String),
    /// `**`: 0 個以上のフォルダ
    Recursive,
    Wild(Pattern),
}

/// 成分の並びと、フォルダだけに一致するか
#[derive(Clone)]
struct Parsed {
    comps: Vec<Comp>,
    dir_only: bool,
}

/// パターンを成分に分ける
fn parse(pattern: &str) -> Result<Parsed, glob::PatternError> {
    // Windows では `\` も区切り
    let pattern = pattern.replace('\\', "/");
    Pattern::new(&pattern)?;
    // `glob::glob` は `/` で終わるとフォルダだけ、最後の `.` もフォルダにしか一致しない。
    // 成分が 0 個（`""`・`.`・`./`）は ghost/master そのもの
    let dir_only =
        pattern.is_empty() || pattern.ends_with('/') || pattern.rsplit('/').next() == Some(".");
    let mut comps = Vec::new();
    // 空の成分と `.` はフォルダをそのまま指す（`Path::components` と同じく pasta.dll が見るパスにも残らない）
    // ponytail: `.*` のような点で始まるワイルドカードが `.`・`..` にも一致する glob の特例は写さない
    for c in pattern.split('/').filter(|c| !c.is_empty() && *c != ".") {
        comps.push(if c == "**" {
            Comp::Recursive
        } else if c.contains(['*', '?', '[']) {
            Comp::Wild(Pattern::new(c)?)
        } else {
            Comp::Literal(c.to_string())
        });
    }
    Ok(Parsed { comps, dir_only })
}

/// 一覧のうちどれかのパターンに一致するもの（profile を除き、重複をまとめ、昇順）。
/// `dir` が false ならファイルとして照らし、フォルダだけのパターンを使わない
fn collect(entries: &[String], patterns: &[Parsed], dir: bool) -> Vec<DictFile> {
    let mut files: Vec<DictFile> = Vec::new();
    for entry in entries {
        // ghost/master そのもの（""）は成分 0 個
        let parts: Vec<&str> = entry.split('/').filter(|p| !p.is_empty()).collect();
        let dll_path = patterns
            .iter()
            .filter(|p| dir || !p.dir_only)
            .find_map(|p| {
                let mut seen = Vec::new();
                matches(&p.comps, &parts, dir, &mut seen).then(|| seen.join("/"))
            });
        // pasta.dll は自分が見るパスで profile を判定する（Path::starts_with は大文字・小文字を区別）
        let excluded = |p: &String| p.starts_with("profile/") || (dir && p == "profile");
        if let Some(dll_path) = dll_path.filter(|p| !excluded(p)) {
            files.push(DictFile {
                path: entry.clone(),
                dll_path,
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    files
}

/// 成分の並びが一致すれば true。`seen` に pasta.dll が見る綴りを積む。`dir` はフォルダとして照らす
fn matches(comps: &[Comp], parts: &[&str], dir: bool, seen: &mut Vec<String>) -> bool {
    let Some((first, rest)) = comps.split_first() else {
        return parts.is_empty();
    };
    match first {
        // 最後の `**` はフォルダにしか一致せず、1 段以上（`dic/**` は dic そのものに一致しない）
        Comp::Recursive if rest.is_empty() => {
            let ok = dir && !parts.is_empty();
            if ok {
                seen.extend(parts.iter().map(|s| s.to_string()));
            }
            ok
        }
        // 途中の `**` は 0 段以上。残りの成分に少なくとも 1 つ残す
        Comp::Recursive => (0..parts.len()).any(|n| {
            let len = seen.len();
            seen.extend(parts[..n].iter().map(|s| s.to_string()));
            let ok = matches(rest, &parts[n..], dir, seen);
            if !ok {
                seen.truncate(len);
            }
            ok
        }),
        Comp::Literal(lit) => one(rest, parts, dir, seen, |p| {
            (p.to_lowercase() == lit.to_lowercase()).then(|| lit.clone())
        }),
        Comp::Wild(pat) => one(rest, parts, dir, seen, |p| {
            pat.matches(p).then(|| p.to_string())
        }),
    }
}

/// 先頭の 1 成分を `head` で照らし、残りを続けて照らす
fn one(
    rest: &[Comp],
    parts: &[&str],
    dir: bool,
    seen: &mut Vec<String>,
    head: impl Fn(&str) -> Option<String>,
) -> bool {
    let Some((part, tail)) = parts.split_first() else {
        return false;
    };
    let Some(spelling) = head(part) else {
        return false;
    };
    seen.push(spelling);
    let ok = matches(rest, tail, dir, seen);
    if !ok {
        seen.pop();
    }
    ok
}
