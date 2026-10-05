//! finalize シーン突合（task 2.2・requirements 3.1/3.2/3.3/7.1）。
//!
//! ビルド側が蓄積した `(join_key, .pasta 宣言行)` 記録（[`super::SourceMap::scene_records`]）
//! と、ランタイム実シーン識別子（シーン表の登録名を定義元 `.pasta` ファイルごとに分けたもの）を、
//! **同一の突合キー**で突き合わせ、`.pasta` (ファイル, 行範囲) → (scene_id, parent) の
//! [`SceneIdentityIndex`] を確定する。
//!
//! # join_key 契約（task 2.1 が emit・本モジュールが consume）
//!
//! - global: `G:{base}#{counter}`（例 `G:会話#1`）
//! - local:  `L:{parent_base}#{parent_counter}:{fn_name}`（例 `L:会話#1:挨拶_1`）
//!
//! `base`/`parent_base` = `SceneRegistry::sanitize_name(name)`、`counter`/`parent_counter`
//! = その `.pasta` ファイルの中での per-base 出現順（ファイルごとに 1 から）。`fn_name` =
//! `{sanitize}_{counter}` または `__start__`。
//!
//! # runtime identity（SSOT）
//!
//! シーン表（`STORE.scenes`）の登録名 `{名前}_{通し番号}`（例 `会話_1`）と、その配下の
//! 関数名（例 `挨拶_1`）が解決対象の SSOT。実行時の通し番号は全ファイルを通して数える
//! ため、記録の `counter` とは一致しない（scene-identity-format・4.5）。
//!
//! # join（定義元ファイルごと・順位）
//!
//! 1. 実行時のグローバルシーンを、シーン表の関数（`__start__` を優先し、無ければ任意の
//!    関数）の定義元チャンク（`Function::info().source`）から
//!    [`SourceMap::pasta_file_for_chunk`] で `.pasta` ファイルに分ける。引けないシーン
//!    （利用者の `.lua` で作ったシーン・関数を持たないシーン）は対象にしない。
//! 2. ファイルの中で登録名を `SceneRegistry::split_registered_name` で（名前, 通し番号）に
//!    分け、名前ごとに通し番号の昇順に並べる。`G:会話#k` は、そのファイルの `会話` の
//!    k 番目の登録名（例 `会話_3`）へ突合 → scene_id=会話_3, parent=None, level=0。
//!    k 番目が無い記録・分けられない登録名は索引へ入れない。
//! 3. local join_key `L:会話#k:挨拶_1` は親を 2. と同じ方法で登録名にし、その配下に
//!    `挨拶_1` があるときだけ採る → scene_id=挨拶_1, parent=会話_3, level=1。
//!
//! # end_line + level（builder 側算出）
//!
//! `level` は join_key 接頭辞（`G`=0 / `L`=1）から。`end_line` は同一 `.pasta` ファイル内で
//! 宣言行昇順に並べ、各シーンにつき「次の同レベル以上宣言の開始行 − 1」（無ければ
//! ファイル末尾相当 = `u32::MAX`）を inclusive な終端として投入する（requirements 2.1 /
//! Implementation Notes 1.1「次宣言行 − 1」）。

use std::collections::HashMap;

use mlua::{Function, Lua, Table, Value};
use pasta_core::registry::SceneRegistry;

use super::{SceneIdentityIndex, SourceMap};

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

/// finalize 後に呼び、runtime 実 identity と蓄積記録を突合して [`SceneIdentityIndex`] を
/// 構築する（task 2.2 のコア）。
///
/// シーン表から（登録名 → 定義元 `.pasta` ファイル）と（登録名 → 配下の関数名）を集め、
/// [`join_records`] で突き合わせる。シーン表が空でも空索引を返す（`scene_at` は常に未検出）。
pub(crate) fn build_scene_index(
    lua: &Lua,
    source_map: &SourceMap,
) -> mlua::Result<SceneIdentityIndex> {
    // SAFETY(injection): モジュール名はコンパイル時の文字列リテラル（collect_scenes と同じ）。
    let scene_module: Table = lua.load("return require('pasta.scene')").eval()?;
    let get_all_scenes: Function = scene_module.get("get_all_scenes")?;
    let registry: Table = get_all_scenes.call(())?;

    // 定義元 `.pasta` ファイル → そのファイルの実行時グローバル登録名。
    let mut globals_by_file: HashMap<String, Vec<String>> = HashMap::new();
    // global_name → local_name の列（例 会話_1 → {__start__, 挨拶_1}）。
    let mut locals_by_global: HashMap<String, Vec<String>> = HashMap::new();
    for pair in registry.pairs::<String, Table>() {
        let (global_name, scene_table) = pair?;
        let mut locals = Vec::new();
        // 定義元を引く関数: `__start__` を優先し、無ければ任意の関数。
        let mut probe: Option<Function> = None;
        for entry in scene_table.pairs::<String, Value>() {
            let (local_name, value) = entry?;
            if local_name == "__global_name__" {
                continue;
            }
            if let Value::Function(f) = value
                && (probe.is_none() || local_name == "__start__")
            {
                probe = Some(f);
            }
            locals.push(local_name);
        }
        let pasta_file = probe
            .and_then(|f| f.info().source)
            .and_then(|chunk| source_map.pasta_file_for_chunk(&chunk));
        if let Some(pasta_file) = pasta_file {
            globals_by_file
                .entry(pasta_file.to_string())
                .or_default()
                .push(global_name.clone());
        }
        locals_by_global.insert(global_name, locals);
    }

    Ok(join_records(
        source_map.scene_records(),
        &globals_by_file,
        &locals_by_global,
    ))
}

/// 記録（`.pasta` ファイル → `(join_key, 宣言行)`）と、ファイルごとの実行時グローバル
/// 登録名・グローバル配下の関数名を突き合わせて索引を作る（Lua に依存しない純関数）。
///
/// 突合できなかった記録（そのファイルの実行時に対応する登録名が無い等）は索引へ入れない
/// （誤解決防止）。
fn join_records(
    scene_records: &HashMap<String, Vec<(String, u32)>>,
    globals_by_file: &HashMap<String, Vec<String>>,
    locals_by_global: &HashMap<String, Vec<String>>,
) -> SceneIdentityIndex {
    let mut builder = SceneIdentityIndex::builder();

    for (pasta_file, records) in scene_records {
        // このファイルは（シーンが空でも）索引に存在させる（未検出を正しく返すため）。
        builder.insert_file(pasta_file);

        // (base, 順位) → このファイルの実行時グローバル登録名。
        let globals = globals_by_file
            .get(pasta_file)
            .map(|names| rank_globals(names))
            .unwrap_or_default();

        // 各記録をパース＋突合し、(start_line, level, scene_id, parent) を集める。
        let mut parsed: Vec<ParsedRecord> = Vec::with_capacity(records.len());
        for (join_key, start_line) in records {
            parsed.push(parse_and_join(
                join_key,
                *start_line,
                &globals,
                locals_by_global,
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

    builder.finish()
}

/// 1 つの `(join_key, start_line)` を level/scene_id/parent へパース＋突合する。
fn parse_and_join(
    join_key: &str,
    start_line: u32,
    globals: &HashMap<(String, usize), String>,
    locals_by_global: &HashMap<String, Vec<String>>,
) -> ParsedRecord {
    if let Some(rest) = join_key.strip_prefix("G:") {
        // global: "G:{base}#{counter}"
        let (base, counter) = parse_base_counter(rest);
        let scene_id = counter.and_then(|c| globals.get(&(base.to_string(), c)).cloned());
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
        let parent = pcounter.and_then(|c| globals.get(&(pbase.to_string(), c)).cloned());

        // `__start__` はグローバル本体そのもの（無名 start シーン）であり、その宣言行は
        // グローバルヘッダ行と一致する。索引には **別 level-1 エントリとして入れない**:
        // グローバル本体領域へのクリックはグローバル identity `(会話_1, None)` へ解決され、
        // global kick（parent=None・global 分岐がシーン表から開始関数 `__start__`
        // を引く）でちょうど `__start__` が再生される（kick 決定ノート）。よって
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

/// 1 ファイルの実行時グローバル登録名を `(名前, 順位)` → 登録名にする。
///
/// 登録名は `SceneRegistry::split_registered_name` だけで分ける（末尾の数字を推測しない）。
/// 分けられない名前は入れない。順位は名前ごとの通し番号の昇順で 1 から数える
/// （ファイルの中の定義順と実行時の通し番号の大小が一致することだけを使う）。
fn rank_globals(registered: &[String]) -> HashMap<(String, usize), String> {
    let mut by_name: HashMap<&str, Vec<(usize, &String)>> = HashMap::new();
    for name in registered {
        if let (base, Some(counter)) = SceneRegistry::split_registered_name(name) {
            by_name.entry(base).or_default().push((counter, name));
        }
    }
    let mut ranked = HashMap::new();
    for (base, mut list) in by_name {
        list.sort();
        for (rank, (_, name)) in list.into_iter().enumerate() {
            ranked.insert((base.to_string(), rank + 1), name.clone());
        }
    }
    ranked
}

/// `"{base}#{counter}"` を `(base, Some(counter))` へ分解する。`#` が無い／counter が
/// 数値でないときは `(全体, None)`。
fn parse_base_counter(s: &str) -> (&str, Option<usize>) {
    match s.rsplit_once('#') {
        Some((base, num)) => (base, num.parse::<usize>().ok()),
        None => (s, None),
    }
}

#[cfg(test)]
#[path = "scene_join_tests.rs"]
mod tests;
