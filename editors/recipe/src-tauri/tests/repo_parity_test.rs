//! リポジトリのファイル同士の一致を固定する（1.6・6.3・13.4）。
//! アプリの lock の版・拡張の凡例と色の写し・版の置き場がルートとずれたら落とす。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pasta_lsp::analysis::{TOKEN_MODIFIERS, TOKEN_TYPES};

fn app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    app_dir().join("../../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()))
}

fn read_toml(path: &Path) -> toml::Table {
    read(path)
        .parse()
        .unwrap_or_else(|e| panic!("{} を TOML として読めない: {e}", path.display()))
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|e| panic!("{} を JSON として読めない: {e}", path.display()))
}

/// lock の 1 包み（名前・版・path かどうか・依存の書き方）。
struct LockPackage {
    name: String,
    version: String,
    is_path: bool,
    deps: Vec<String>,
}

fn lock_packages(path: &Path) -> Vec<LockPackage> {
    let lock = read_toml(path);
    let packages = lock["package"].as_array().expect("[[package]] が無い");
    packages
        .iter()
        .map(|p| LockPackage {
            name: p["name"].as_str().unwrap().to_owned(),
            version: p["version"].as_str().unwrap().to_owned(),
            // source の無い包みは path のもの
            is_path: p.get("source").is_none(),
            deps: p
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|d| d.iter().map(|s| s.as_str().unwrap().to_owned()).collect())
                .unwrap_or_default(),
        })
        .collect()
}

/// lock の依存の書き方（"名前" か "名前 版" か "名前 版 (出どころ)"）から包みを引く。
fn resolve<'a>(packages: &'a [LockPackage], dep: &str) -> &'a LockPackage {
    let mut parts = dep.split(' ');
    let name = parts.next().unwrap();
    let version = parts.next();
    packages
        .iter()
        .find(|p| p.name == name && version.is_none_or(|v| p.version == v))
        .unwrap_or_else(|| panic!("lock に {dep} が無い"))
}

fn versions_by_name<'a>(
    packages: impl Iterator<Item = &'a LockPackage>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for p in packages {
        map.entry(p.name.clone())
            .or_default()
            .insert(p.version.clone());
    }
    map
}

/// (1) アプリの lock で pasta_lsp・pasta_check からたどれるクレートの版が、ルートの lock と同じ。
/// アプリの lock では両者はワークスペースの外なので、lock に載る依存は通常（と build）の依存だけ。
#[test]
fn app_lock_matches_root_lock_for_analyzer_and_check_core() {
    let app = lock_packages(&app_dir().join("Cargo.lock"));
    let root = lock_packages(&repo_root().join("Cargo.lock"));

    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut stack: Vec<&LockPackage> = ["pasta_lsp", "pasta_check"]
        .iter()
        .map(|n| resolve(&app, n))
        .collect();
    let mut reachable: Vec<&LockPackage> = Vec::new();
    while let Some(p) = stack.pop() {
        if !seen.insert((p.name.clone(), p.version.clone())) {
            continue;
        }
        reachable.push(p);
        stack.extend(p.deps.iter().map(|d| resolve(&app, d)));
    }

    let app_versions = versions_by_name(reachable.into_iter().filter(|p| !p.is_path));
    let root_versions = versions_by_name(root.iter());

    let mismatches: Vec<String> = app_versions
        .iter()
        .filter_map(|(name, app_vs)| {
            // ルートに無い名前は比べない。lock の依存は木全体で機能を合わせた結果なので、
            // アプリ側だけで有効な機能（例: Tauri が入れる futures-util の io）の分が混ざる
            let root_vs = root_versions.get(name)?;
            // ルートは木全体の lock なので、pasta_lsp・pasta_check の外で使う版（例: syn 2）も載る。
            // アプリの版がどれもルートに在れば揃っている
            if app_vs.is_subset(root_vs) {
                return None;
            }
            let join = |vs: &BTreeSet<String>| vs.iter().cloned().collect::<Vec<_>>().join(", ");
            let precise = root_vs.iter().next_back().unwrap();
            Some(format!(
                "  {name}: アプリ {} / ルート {}\n    直し方: cargo update --manifest-path editors/recipe/src-tauri/Cargo.toml -p {name} --precise {precise}",
                join(app_vs),
                join(root_vs)
            ))
        })
        .collect();

    assert!(
        mismatches.is_empty(),
        "アプリの Cargo.lock の版がルートの Cargo.lock とずれている:\n{}",
        mismatches.join("\n")
    );
}

/// TS の `export const {name}` の配列の引用符の中の名前を順に取り出す（行末の // の注釈は除く）。
fn ts_string_array(source: &str, name: &str) -> Vec<String> {
    let start = source
        .find(&format!("export const {name}"))
        .unwrap_or_else(|| panic!("{name} が見つからない"));
    let body = &source[start..];
    // 型の注釈 `readonly string[]` の [] を飛ばし、`=` の後の配列を読む
    let body = &body[body.find('=').unwrap()..];
    let body = &body[body.find('[').unwrap() + 1..];
    let body = &body[..body.find(']').unwrap()];
    body.lines()
        .map(|line| line.split("//").next().unwrap())
        .flat_map(|line| line.split('\'').skip(1).step_by(2).map(str::to_owned))
        .collect()
}

/// (2) 拡張の凡例の並びが解析器の凡例と同じ（6.3）。
#[test]
fn extension_legend_matches_analyzer_legend() {
    let ts = read(&repo_root().join("editors/vscode/src/semanticTokensProvider.ts"));
    assert_eq!(ts_string_array(&ts, "PASTA_TOKEN_TYPES"), TOKEN_TYPES);
    assert_eq!(
        ts_string_array(&ts, "PASTA_TOKEN_MODIFIERS"),
        TOKEN_MODIFIERS
    );
}

/// (3) 拡張の `semanticTokenScopes`（pasta）の鍵の集合が解析器の凡例の種類と同じ（色の写しに漏れが無い）。
#[test]
fn extension_scope_keys_match_analyzer_token_types() {
    let pkg = read_json(&repo_root().join("editors/vscode/package.json"));
    let entry = pkg["contributes"]["semanticTokenScopes"]
        .as_array()
        .expect("semanticTokenScopes が無い")
        .iter()
        .find(|e| e["language"] == "pasta")
        .expect("language = pasta の semanticTokenScopes が無い");
    let keys: BTreeSet<&str> = entry["scopes"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let types: BTreeSet<&str> = TOKEN_TYPES.iter().copied().collect();
    assert_eq!(keys, types);
}

/// (4) tauri.conf.json の version が指す package.json の版がルートの [workspace.package].version と同じ（1.6）。
#[test]
fn app_version_source_matches_workspace_version() {
    let conf = read_json(&app_dir().join("tauri.conf.json"));
    let version_path = conf["version"]
        .as_str()
        .expect("tauri.conf.json の version が無い");
    let pkg = read_json(&app_dir().join(version_path));
    let root = read_toml(&repo_root().join("Cargo.toml"));
    assert_eq!(
        pkg["version"].as_str().unwrap(),
        root["workspace"]["package"]["version"].as_str().unwrap()
    );
}
