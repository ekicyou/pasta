import "./style.css";
import { createEditor } from "./editor.ts";
import { Documents } from "./documents.ts";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { analyze, checkGhost, initialPath, listTree, onGhostChanged, onOpenRequest, openGhost, readText, recentRemove, resolveGhost } from "./ipc.ts";
import { scenesToItems } from "./mapping.ts";
import { Panels } from "./panels.ts";
import { CoalescingRunner, eachLimit } from "./scheduler.ts";
import { StartView, type OpenOrigin } from "./start.ts";
import { findSame, sameMaster } from "./tabs.ts";
import { notify } from "./dialog.ts";
import { TreeView } from "./tree.ts";

const editor = await createEditor(document.getElementById("editor")!);
const docs = new Documents(editor, document.getElementById("tabs")!, document.getElementById("notices")!, {
  // 書き換え中の .pasta はその解析にシーンを追わせる（4.10）
  onAnalyzed: (tab) => {
    tree.setScenes(tab.rel, scenesToItems(tab.rel, tab.analysis!.scenes));
    panels.update(); // 診断の一覧（7.3）と、開いたファイルの syntax を検査の一覧から除く（9.8）
  },
  // 閉じたら未保存の書き換えのシーンを捨て、ディスクの中身から作り直す。診断の一覧からも消す（7.6）
  onClosed: (tab) => {
    void loadScenes(tab.rel);
    panels.update();
  },
  onRenamed: () => panels.update(),
  // shortcut: .pasta・直下の pasta.toml の保存で検査し直す仮の口（9.2）。7.2 の保存の後の検査が置き換える
  onSaved: (tab) => {
    if (tab.rel.toLowerCase().endsWith(".pasta") || tab.rel.toLowerCase() === "pasta.toml") checker.request();
  },
});
const tree = new TreeView(document.getElementById("tree")!, docs);
const panels = new Panels(document.getElementById("panels")!, docs);
// shortcut: 検査の求めをまとめる仮の口（9.9）。7.2 が見張りの知らせと保存の後の検査でこれを持つ
const checker = new CoalescingRunner(async () => panels.setReport(await checkGhost().catch((reason: unknown) => String(reason))), 300, globalThis);

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

const start = new StartView(document.getElementById("start")!, document.getElementById("toolbar")!, (path, origin) => void openPath(path, origin));

/** 開いているゴーストの master（まだ開いていなければ null） */
let current: string | null = null;
/** 開く流れを 1 つずつ（2 つの入口が重なっても確かめと開き直しを交ぜない） */
let opening: Promise<void> = Promise.resolve();

/** 5 つの入口が入る 1 本の開く流れ（2.1・2.2・2.3・2.6〜2.8・2.11） */
function openPath(path: string, origin: OpenOrigin | "arg" | "second" | "drop"): Promise<void> {
  const run = () => openNow(path, origin);
  opening = opening.then(run, run);
  return opening;
}

async function openNow(path: string, origin: string): Promise<void> {
  let found;
  try {
    found = await resolveGhost(path);
  } catch (reason) {
    // 見つからなければ理由を出し、開いているゴーストは変えない（2.7）。最近開いたものからなら一覧から消す（3.5）
    if (origin === "recent") {
      await recentRemove(path).catch(() => {});
      notify(`${reason}（最近開いたものから消しました）`);
      if (current === null) await start.show();
    } else notify(String(reason));
    return;
  }
  if (!sameMaster(current, found.master)) {
    if (!(await docs.confirmUnsaved())) return;
    // 前のゴーストのタブは Rust の master が替わる前に片付け、替わる間はタブを開かない。
    // 開けなかったときはタブを閉じたまま今のゴーストに留まる（保存する／しないは選び終えている）
    await docs.beginSwitch();
    panels.update();
    let info;
    try {
      info = await openGhost(found.master);
    } catch (reason) {
      notify(String(reason));
      return;
    } finally {
      docs.endSwitch();
    }
    current = info.master;
    start.hide();
    await tree.reset();
    // すべての .pasta のシーンを作る（4.8）。一覧が読めなければ tree.reset が理由を出している
    const files = await listTree().then((t) => t.files, () => []);
    await eachLimit(files, 8, loadScenes);
    checker.request(); // 9.1
  }
  if (found.file) await docs.open(found.file); // 2.6
}

// shortcut: 開発の窓で CDP からタブの状態を見るための口。開発のビルドだけ
if (import.meta.env.DEV) Object.assign(globalThis, { recipeDocs: docs, recipePanels: panels });
// shortcut: 見張りの知らせを外の変更の適用へ直につなぐ仮の口。開発のビルドだけ。7.2 の ghost-changed の流れ（ツリー・シーン・検査・今の master の知らせだけ）が置き換える
if (import.meta.env.DEV) await onGhostChanged(() => void (docs.applyExternal(), tree.refresh()));

// 2 つ目の起動（2.11。パスが無ければ Rust が窓を前に出すだけ）・窓へのドロップ（2.2。複数なら最初の 1 つ）
await onOpenRequest((e) => {
  if (e.path) void openPath(e.path, "second");
});
await getCurrentWebview().onDragDropEvent((e) => {
  if (e.payload.type === "drop" && e.payload.paths.length > 0) void openPath(e.payload.paths[0], "drop");
});

// 起動の引数（2.1）。無いか開けなければ開始の画面（1.5）
const path = await initialPath();
if (path) await openPath(path, "arg");
if (current === null) await start.show();
