import "./style.css";
import { createEditor, monaco } from "./editor.ts";
import { checkGhost, initialPath, listTree, openGhost, readText, resolveGhost } from "./ipc.ts";

const editor = await createEditor(document.getElementById("editor")!);

// shortcut: 開発の窓で部品を確かめるための仮の起動（最初の .pasta だけをエディタに出す）。7.1 の openPath と StartView が置き換える
const path = await initialPath();
if (path) {
  const { master, file } = await resolveGhost(path);
  await openGhost(master);
  const tree = await listTree();
  const rel = file ?? tree.files.find((f) => f.endsWith(".pasta"));
  if (rel) {
    const { text } = await readText(rel);
    editor.setModel(monaco.editor.createModel(text, "pasta", monaco.Uri.file(rel)));
  }
  await checkGhost();
} else {
  const start = document.getElementById("start")!;
  start.textContent = "ゴーストを開いていません";
  start.hidden = false;
}
