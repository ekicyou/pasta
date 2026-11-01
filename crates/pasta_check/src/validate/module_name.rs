//! モジュール名を作り、辞書ファイル・辞書モジュールの重なりを分類する段（4.1〜4.4）。

use std::collections::BTreeMap;
use std::path::Path;

use super::discovery::{DictFile, Discovered};
use super::{Finding, FindingKind, msg};

/// 判定はモジュール名だけで行う（4.4）。比べるのは大文字と小文字を畳んだ名前、報告は実際の名前と実物の綴り
pub(crate) fn check(discovered: &Discovered) -> Vec<Finding> {
    let pasta = by_module(&discovered.pasta);
    let lua = by_module(&discovered.lua);
    let mut findings = Vec::new();
    for files in pasta.values() {
        collisions(
            &mut findings,
            files,
            FindingKind::ModulePastaCollision,
            msg::module_pasta_collision,
        );
    }
    for (folded, files) in &lua {
        collisions(
            &mut findings,
            files,
            FindingKind::ModuleLuaCollision,
            msg::module_lua_collision,
        );
        if let Some(pastas) = pasta.get(folded) {
            let pasta_paths: Vec<&str> = pastas.iter().map(|(_, p)| *p).collect();
            for (module, file) in files {
                findings.push(Finding::new(
                    FindingKind::ModuleLuaShadowed,
                    *file,
                    None,
                    None,
                    msg::module_lua_shadowed(module, &pasta_paths),
                ));
            }
        }
    }
    findings
}

/// 畳んだモジュール名ごとの (実際のモジュール名, 実物のパス)
fn by_module(files: &[DictFile]) -> BTreeMap<String, Vec<(String, &str)>> {
    let mut map: BTreeMap<String, Vec<(String, &str)>> = BTreeMap::new();
    for f in files {
        let module = module_name_of(&f.dll_path);
        map.entry(module.to_lowercase())
            .or_default()
            .push((module, &f.path));
    }
    map
}

/// 2 つ以上なら関わる各ファイルに、自分のモジュール名と自分以外のパスを示して付ける
fn collisions(
    findings: &mut Vec<Finding>,
    files: &[(String, &str)],
    code: FindingKind,
    message: fn(&str, &[&str]) -> String,
) {
    if files.len() < 2 {
        return;
    }
    for (module, file) in files {
        let others: Vec<&str> = files
            .iter()
            .map(|(_, p)| *p)
            .filter(|o| o != file)
            .collect();
        findings.push(Finding::new(
            code,
            *file,
            None,
            None,
            message(module, &others),
        ));
    }
}

/// pasta.dll が見るパスからモジュール名を作る（pasta_lua の `CacheManager::source_to_module_name` と同じ規則）。
/// 先頭の成分がちょうど `dic` なら外す → 最後の拡張子を外す → `.` と `-` を `_` → 区切りを `.` → `pasta.scene.` を前に付ける
pub fn module_name_of(rel_path: &str) -> String {
    let without_prefix = match rel_path.strip_prefix("dic") {
        Some(rest) => match rest.strip_prefix(['/', '\\']) {
            Some(stripped) => stripped,
            None if rest.is_empty() => rest,
            None => rel_path,
        },
        None => rel_path,
    };
    let stem = Path::new(without_prefix).with_extension("");
    let module_path = stem
        .to_string_lossy()
        .replace(['.', '-'], "_")
        .replace(['/', '\\'], ".");
    format!("pasta.scene.{module_path}")
}
