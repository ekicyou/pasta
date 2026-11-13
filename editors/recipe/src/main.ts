import "./style.css";
import { createEditor } from "./editor.ts";
import { Documents } from "./documents.ts";
import { checkGhost, initialPath, listTree, onGhostChanged, openGhost, resolveGhost } from "./ipc.ts";

const editor = await createEditor(document.getElementById("editor")!);
const docs = new Documents(editor, document.getElementById("tabs")!, document.getElementById("notices")!);

const saveButton = document.createElement("button");
saveButton.textContent = "保存 (Ctrl+S)";
saveButton.addEventListener("click", () => {
  if (docs.active) void docs.save(docs.active);
});
document.getElementById("toolbar")!.append(saveButton);

// shortcut: ツリー（6.5）が無い間、開発の窓で CDP からほかのファイルを開くための口。開発のビルドだけ
if (import.meta.env.DEV) Object.assign(globalThis, { recipeDocs: docs });
// shortcut: 見張りの知らせを外の変更の適用へ直につなぐ仮の口。開発のビルドだけ。7.2 の ghost-changed の流れ（ツリー・シーン・検査・今の master の知らせだけ）が置き換える
if (import.meta.env.DEV) await onGhostChanged(() => void docs.applyExternal());

// shortcut: 開発の窓で部品を確かめるための仮の起動（引数のファイルか最初の .pasta をタブで開く）。7.1 の openPath と StartView が置き換える
const path = await initialPath();
if (path) {
  const { master, file } = await resolveGhost(path);
  await openGhost(master);
  const tree = await listTree();
  const rel = file ?? tree.files.find((f) => f.endsWith(".pasta"));
  if (rel) await docs.open(rel);
  await checkGhost();
} else {
  const start = document.getElementById("start")!;
  start.textContent = "ゴーストを開いていません";
  start.hidden = false;
}
