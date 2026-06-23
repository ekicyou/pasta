//! finalize シーン突合（task 2.2・requirements 3.1/3.2/3.3/7.1）。
//!
//! ビルド側が蓄積した `(join_key, .pasta 宣言行)` 記録（[`super::SourceMap::scene_records`]）
//! と、ランタイム実シーン識別子（`collect_scenes` の `(global_name, local_name)`）を、
//! **同一の突合キー**で突き合わせ、`.pasta` (ファイル, 行範囲) → (scene_id, parent) の
//! [`SceneIdentityIndex`] を確定する。
//!
//! # join_key 契約（task 2.1 が emit・本モジュールが consume）
//!
//! - global: `G:{base}#{counter}`（例 `G:会話#1`）
//! - local:  `L:{parent_base}#{parent_counter}:{fn_name}`（例 `L:会話#1:挨拶_1`）
//!
//! `base`/`parent_base` = `SceneRegistry::sanitize_name(name)`、`counter`/`parent_counter`
//! = per-base 出現順（= ランタイム `create_scene` 連番）。`fn_name` = `{sanitize}_{counter}`
//! または `__start__`。
//!
//! # runtime identity（SSOT）
//!
//! `collect_scenes` が返す `(global_name, local_name)`（例 `(会話1, 挨拶_1)`）が
//! 解決対象の SSOT。global_name は counter 連結形（`会話1`・アンダースコア無し）。
//! `SceneRegistry` の別形式（`会話_1`）は SSOT にしない（task 2.2 / kick 決定）。
//!
//! # join（global / local）
//!
//! 1. runtime global_name `会話1` を **(base, counter)** へ分解（末尾連番を剥がす）。
//!    `G:会話#1` の (base=会話, counter=1) と突合 → scene_id=会話1, parent=None, level=0。
//! 2. local join_key `L:会話#1:挨拶_1` の親参照 `会話#1` を runtime global `会話1` へ
//!    対応付け、`collect_scenes` の `(会話1, 挨拶_1)` と fn_name 一致で突合 →
//!    scene_id=挨拶_1, parent=会話1, level=1。
//!
//! # end_line + level（builder 側算出）
//!
//! `level` は join_key 接頭辞（`G`=0 / `L`=1）から。`end_line` は同一 `.pasta` ファイル内で
//! 宣言行昇順に並べ、各シーンにつき「次の同レベル以上宣言の開始行 − 1」（無ければ
//! ファイル末尾相当 = `u32::MAX`）を inclusive な終端として投入する（requirements 2.1 /
//! Implementation Notes 1.1「次宣言行 − 1」）。

use std::collections::HashMap;

use mlua::Lua;

use super::{SceneIdentityIndex, SourceMap};
use crate::runtime::finalize::collect_scenes;

/// 1 シーン記録のパース結果（join 用の中間表現）。
struct ParsedRecord {
    /// 入れ子レベル（global=0 / local=1）。
    level: u32,
    /// 宣言行（`.pasta`・1 始まり）。
    start_line: u32,
    /// 突合済み scene_id（runtime 実 identity）。突合失敗時は `None`（索引へ入れない）。
    scene_id: Option<String>,
    /// 親グローバル（local のみ `Some`）。
    parent: Option<String>,
}

/// runtime global_name `会話1` を `(base, counter)` へ分解する。
///
/// 末尾の連続する ASCII 数字を counter とし、その手前を base とする（`会話1` →
/// `("会話", 1)`、`メイン12` → `("メイン", 12)`）。末尾が数字でなければ `None`。
/// base は非空でなければならない（純数字名は想定外として `None`）。
fn split_runtime_global(global_name: &str) -> Option<(&str, u32)> {
    let digits_start = global_name
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i)?;
    let (base, digits) = global_name.split_at(digits_start);
    if base.is_empty() {
        return None;
    }
    let counter: u32 = digits.parse().ok()?;
    Some((base, counter))
}

/// finalize 後に呼び、runtime 実 identity と蓄積記録を突合して [`SceneIdentityIndex`] を
/// 構築する（task 2.2 のコア）。
///
/// 突合できなかった記録（runtime に対応 identity が無い等）は索引へ入れない（誤解決防止）。
/// `collect_scenes` が空でも空索引を返す（`scene_at` は常に未検出）。
pub(crate) fn build_scene_index(lua: &Lua, source_map: &SourceMap) -> mlua::Result<SceneIdentityIndex> {
    // runtime 実シーン: (global_name, local_name) の列。
    let runtime_scenes = collect_scenes(lua)?;

    // (base, counter) → runtime global_name（例 (会話,1) → 会話1）。
    let mut global_by_base_counter: HashMap<(String, u32), String> = HashMap::new();
    // global_name → set of local_name（例 会話1 → {__start__, 挨拶_1}）。
    let mut locals_by_global: HashMap<String, Vec<String>> = HashMap::new();
    for (global_name, local_name) in &runtime_scenes {
        if let Some((base, counter)) = split_runtime_global(global_name) {
            global_by_base_counter
                .entry((base.to_string(), counter))
                .or_insert_with(|| global_name.clone());
        }
        locals_by_global
            .entry(global_name.clone())
            .or_default()
            .push(local_name.clone());
    }

    let mut builder = SceneIdentityIndex::builder();

    for (pasta_file, records) in source_map.scene_records() {
        // このファイルは（シーンが空でも）索引に存在させる（未検出を正しく返すため）。
        builder.insert_file(pasta_file);

        // 各記録をパース＋突合し、(start_line, level, scene_id, parent) を集める。
        let mut parsed: Vec<ParsedRecord> = Vec::with_capacity(records.len());
        for (join_key, start_line) in records {
            parsed.push(parse_and_join(
                join_key,
                *start_line,
                &global_by_base_counter,
                &locals_by_global,
            ));
        }

        // end_line 算出のため宣言行昇順に並べる（安定: 同一行は元順を保つ）。
        // start_line をキーに昇順ソート（同一 start_line は global→local の元順を維持）。
        let mut order: Vec<usize> = (0..parsed.len()).collect();
        order.sort_by_key(|&i| parsed[i].start_line);

        // end_line = 「次の同レベル以上宣言の開始行 − 1」。下方を走査して最初に
        // level <= 自分 の宣言を探す。無ければ u32::MAX（ファイル末尾相当・inclusive）。
        for pos in 0..order.len() {
            let i = order[pos];
            let my_level = parsed[i].level;
            let my_start = parsed[i].start_line;
            let mut end_line = u32::MAX;
            for &j in &order[pos + 1..] {
                if parsed[j].start_line <= my_start {
                    // 同一開始行（global ヘッダ ⊃ __start__ など）は終端境界にしない。
                    continue;
                }
                if parsed[j].level <= my_level {
                    end_line = parsed[j].start_line.saturating_sub(1);
                    break;
                }
            }

            if let Some(scene_id) = parsed[i].scene_id.clone() {
                builder.add_scene(
                    pasta_file,
                    &scene_id,
                    parsed[i].parent.as_deref(),
                    my_start,
                    end_line,
                    my_level,
                );
            }
        }
    }

    Ok(builder.finish())
}

/// 1 つの `(join_key, start_line)` を level/scene_id/parent へパース＋突合する。
fn parse_and_join(
    join_key: &str,
    start_line: u32,
    global_by_base_counter: &HashMap<(String, u32), String>,
    locals_by_global: &HashMap<String, Vec<String>>,
) -> ParsedRecord {
    if let Some(rest) = join_key.strip_prefix("G:") {
        // global: "G:{base}#{counter}"
        let (base, counter) = parse_base_counter(rest);
        let scene_id = counter
            .and_then(|c| global_by_base_counter.get(&(base.to_string(), c)).cloned());
        ParsedRecord {
            level: 0,
            start_line,
            scene_id,
            parent: None,
        }
    } else if let Some(rest) = join_key.strip_prefix("L:") {
        // local: "L:{parent_base}#{parent_counter}:{fn_name}"
        // 親参照は最初の ':' まで（fn_name に ':' は含まれない契約）。
        let (parent_ref, fn_name) = match rest.split_once(':') {
            Some((p, f)) => (p, f),
            None => (rest, ""),
        };
        let (pbase, pcounter) = parse_base_counter(parent_ref);
        let parent = pcounter
            .and_then(|c| global_by_base_counter.get(&(pbase.to_string(), c)).cloned());

        // `__start__` はグローバル本体そのもの（無名 start シーン）であり、その宣言行は
        // グローバルヘッダ行と一致する。索引には **別 level-1 エントリとして入れない**:
        // グローバル本体領域へのクリックはグローバル identity `(会話1, None)` へ解決され、
        // global kick（parent=None・global 分岐 `SCENE.search(name, nil)` が `__start__`
        // を強制）でちょうど `__start__` が再生される（kick 決定ノート）。よって
        // `__start__` を level-1 で持つとグローバル領域が誤って local 扱いされる。
        if fn_name == "__start__" {
            return ParsedRecord {
                level: 1,
                start_line,
                scene_id: None, // 索引へ入れない（global エントリが本体領域を覆う）。
                parent,
            };
        }

        // 名前付き local: scene_id = fn_name（runtime local_name と一致する想定）。突合
        // 確認: 親グローバル配下に当該 fn_name が存在するときのみ採用する。
        let scene_id = match &parent {
            Some(g) => locals_by_global
                .get(g)
                .filter(|locals| locals.iter().any(|l| l == fn_name))
                .map(|_| fn_name.to_string()),
            None => None,
        };
        ParsedRecord {
            level: 1,
            start_line,
            scene_id,
            parent,
        }
    } else {
        // 未知接頭辞: 突合不可（索引へ入れない）。
        ParsedRecord {
            level: 0,
            start_line,
            scene_id: None,
            parent: None,
        }
    }
}

/// `"{base}#{counter}"` を `(base, Some(counter))` へ分解する。`#` が無い／counter が
/// 数値でないときは `(全体, None)`。
fn parse_base_counter(s: &str) -> (&str, Option<u32>) {
    match s.rsplit_once('#') {
        Some((base, num)) => (base, num.parse::<u32>().ok()),
        None => (s, None),
    }
}

#[cfg(test)]
#[path = "scene_join_tests.rs"]
mod tests;
