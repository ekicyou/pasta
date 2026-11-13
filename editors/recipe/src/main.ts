import "./style.css";
import { createEditor } from "./editor.ts";
import { Documents } from "./documents.ts";
import { analyze, checkGhost, initialPath, listTree, onGhostChanged, openGhost, readText, resolveGhost } from "./ipc.ts";
import { scenesToItems } from "./mapping.ts";
import { findSame } from "./tabs.ts";
import { TreeView } from "./tree.ts";

const editor = await createEditor(document.getElementById("editor")!);
const docs = new Documents(editor, document.getElementById("tabs")!, document.getElementById("notices")!, {
  // 書き換え中の .pasta はその解析にシーンを追わせる（4.10）
  onAnalyzed: (tab) => tree.setScenes(tab.rel, scenesToItems(tab.rel, tab.analysis!.scenes)),
  // 閉じたら未保存の書き換えのシーンを捨て、ディスクの中身から作り直す
  onClosed: (tab) => void loadScenes(tab.rel),
});
const tree = new TreeView(document.getElementById("tree")!, docs);

/** ディスクの中身からシーンを作る。開いているタブはその解析が正なので触らない。読めない（UTF-8 でない等）ものはシーン無し（4.8） */
async function loadScenes(rel: string): Promise<void> {
  if (!rel.toLowerCase().endsWith(".pasta") || findSame(docs.tabs, rel)) return;
  try {
    const { scenes } = await analyze((await readText(rel)).text);
    if (!findSame(docs.tabs, rel)) tree.setScenes(rel, scenesToItems(rel, scenes));
  } catch {
    tree.setScenes(rel, []);
  }
}

const saveButton = document.createElement("button");
saveButton.textContent = "保存 (Ctrl+S)";
saveButton.addEventListener("click", () => {
  if (docs.active) void docs.save(docs.active);
});
document.getElementById("toolbar")!.append(saveButton);

// shortcut: 開発の窓で CDP からタブの状態を見るための口。開発のビルドだけ
if (import.meta.env.DEV) Object.assign(globalThis, { recipeDocs: docs });
// shortcut: 見張りの知らせを外の変更の適用へ直につなぐ仮の口。開発のビルドだけ。7.2 の ghost-changed の流れ（ツリー・シーン・検査・今の master の知らせだけ）が置き換える
if (import.meta.env.DEV) await onGhostChanged(() => void (docs.applyExternal(), tree.refresh()));

// shortcut: 開発の窓で部品を確かめるための仮の起動（引数のファイルか最初の .pasta をタブで開く）。7.1 の openPath と StartView が置き換える
const path = await initialPath();
if (path) {
  const { master, file } = await resolveGhost(path);
  await openGhost(master);
  const listing = await listTree();
  await tree.refresh();
  // shortcut: 全 .pasta のシーンを 1 回だけ読む。7.1 の開く流れ（4.8）が置き換える
  await Promise.all(listing.files.map(loadScenes));
  const rel = file ?? listing.files.find((f) => f.endsWith(".pasta"));
  if (rel) await docs.open(rel);
  await checkGhost();
} else {
  const start = document.getElementById("start")!;
  start.textContent = "ゴーストを開いていません";
  start.hidden = false;
}
