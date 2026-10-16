//! モジュール名を作り、辞書ファイル・辞書モジュールの重なりを分類する段（4.1〜4.4）。

use std::collections::BTreeMap;
use std::path::Path;

use super::discovery::{DictFile, Discovered};
use super::{Finding, FindingKind, msg};

/// 判定はモジュール名だけで行う（4.4）。報告のパスは実物の綴り
pub(crate) fn check(discovered: &Discovered) -> Vec<Finding> {
    let pasta = by_module(&discovered.pasta);
    let lua = by_module(&discovered.lua);
    let mut findings = Vec::new();
    for (module, files) in &pasta {
        collisions(
            &mut findings,
            module,
            files,
            FindingKind::ModulePastaCollision,
            msg::module_pasta_collision,
        );
    }
    for (module, files) in &lua {
        collisions(
            &mut findings,
            module,
            files,
            FindingKind::ModuleLuaCollision,
            msg::module_lua_collision,
        );
        if let Some(pastas) = pasta.get(module) {
            for file in files {
                findings.push(Finding::new(
                    FindingKind::ModuleLuaShadowed,
                    *file,
                    None,
                    None,
                    msg::module_lua_shadowed(module, pastas),
                ));
            }
        }
    }
    findings
}

/// モジュール名ごとの実物のパス
fn by_module(files: &[DictFile]) -> BTreeMap<String, Vec<&str>> {
    let mut map: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for f in files {
        map.entry(module_name_of(&f.dll_path))
            .or_default()
            .push(&f.path);
    }
    map
}

/// 2 つ以上なら関わる各ファイルに、自分以外のパスを示して付ける
fn collisions(
    findings: &mut Vec<Finding>,
    module: &str,
    files: &[&str],
    code: FindingKind,
    message: fn(&str, &[&str]) -> String,
) {
    if files.len() < 2 {
        return;
    }
    for file in files {
        let others: Vec<&str> = files.iter().copied().filter(|o| o != file).collect();
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
