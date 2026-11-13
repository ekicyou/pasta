import "./style.css";
import { createEditor } from "./editor.ts";
import { Documents } from "./documents.ts";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { analyze, checkGhost, initialPath, onGhostChanged, onOpenRequest, openGhost, readText, recentRemove, resolveGhost } from "./ipc.ts";
import { scenesToItems } from "./mapping.ts";
import { Panels } from "./panels.ts";
import { CoalescingRunner, eachLimit } from "./scheduler.ts";
import { StartView, type OpenOrigin } from "./start.ts";
import { findSame, sameMaster } from "./tabs.ts";
import { planGhostChanged } from "./sync.ts";
import { notify } from "./dialog.ts";
import { TreeView } from "./tree.ts";
import { DevActions } from "./devActions.ts";
import { OutputPanel, VariablesPanel } from "./devPanels.ts";

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
  // .pasta・直下の pasta.toml の保存で検査し直す（9.2）。見張りの知らせの求めと同じ checker で 1 回にまとまる（9.9）
  onSaved: (tab) => {
    if (tab.rel.toLowerCase().endsWith(".pasta") || tab.rel.toLowerCase() === "pasta.toml") checker.request();
  },
});
const tree = new TreeView(document.getElementById("tree")!, docs);
const panels = new Panels(document.getElementById("panels")!, docs);
// 開発の操作の糊を 1 つ組み、下の欄の動かす面とつなぐ。保存変数の表の操作は 5.2・5.3 でつなぐ
const devActions = new DevActions({
  docs,
  output: new OutputPanel(panels.output),
  vars: new VariablesPanel(panels.variables, { onRefresh() {}, onEdit() {}, onDelete() {}, onAdd() {} }),
  panels,
});
// 保存の後と見張りの知らせの検査の求めを 300 ms で 1 回にまとめる（9.9）
const checker = new CoalescingRunner(async () => panels.setReport(await checkGhost().catch((reason: unknown) => String(reason))), 300, globalThis);

/** ディスクの中身からシーンを作る。開いているタブはその解析が正なので触らない。読めない（UTF-8 でない等）ものはシーン無し（4.8） */
async function loadScenes(rel: string): Promise<void> {
  if (!rel.toLowerCase().endsWith(".pasta") || findSame(docs.tabs, rel)) return;
  const gen = generation;
  let items: ReturnType<typeof scenesToItems> = [];
  try {
    items = scenesToItems(rel, (await analyze((await readText(rel)).text)).scenes);
  } catch {
    // 消えた・読めない
  }
  if (gen === generation && !findSame(docs.tabs, rel)) tree.setScenes(rel, items); // 読む間に別のゴーストへ移ったら捨てる
}

const saveButton = document.createElement("button");
saveButton.textContent = "保存 (Ctrl+S)";
saveButton.className = "primary";
saveButton.addEventListener("click", () => {
  if (docs.active) void docs.save(docs.active);
});
// 開始の画面では今の CSS がツールバーごと隠す（2.5）
const reloadButton = document.createElement("button");
reloadButton.textContent = "SHIORI をリロード";
reloadButton.addEventListener("click", () => void devActions.reload());
document.getElementById("toolbar")!.append(saveButton, reloadButton);

const start = new StartView(document.getElementById("start")!, document.getElementById("toolbar")!, (path, origin) => void openPath(path, origin));

// Ctrl+O はどこに焦点があってもファイルを選ぶ窓を出す（開始の画面でも）
document.addEventListener("keydown", (e) => {
  if (!e.ctrlKey || e.altKey || e.shiftKey || e.key.toLowerCase() !== "o") return;
  e.preventDefault();
  if (e.repeat || document.querySelector("dialog[open]")) return;
  void start.pick(false);
});

/** 開いているゴーストの master（まだ開いていなければ null） */
let current: string | null = null;
/** 開く流れを 1 つずつ（2 つの入口が重なっても確かめと開き直しを交ぜない） */
let opening: Promise<void> = Promise.resolve();
/** 別のゴーストへ移り始めるたびに進める。前のゴーストの知らせ・読みの結果を捨てる */
let generation = 0;
/** beginSwitch から open_ghost が終わるまで。この間の見張りの知らせは前のゴーストのものなので捨てる */
let switching = false;
/** 最後に見たツリーの一覧のファイル（見張りの知らせで、消えた・新しく出たものと並びの変わりを見る） */
let files: string[] = [];
/** 見張りの知らせを 1 つずつ */
let changes: Promise<void> = Promise.resolve();

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
    switching = true;
    generation++;
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
      switching = false;
    }
    current = info.master;
    devActions.setGhost(info);
    panels.show("write"); // 開いた直後は書く面
    start.hide();
    // すべての .pasta のシーンを作る（4.8）。一覧が読めなければ tree.reset が理由を出している
    files = (await tree.reset())?.files ?? [];
    await eachLimit(files, 8, loadScenes);
    checker.request(); // 9.1
  }
  if (found.file) await docs.open(found.file); // 2.6
}

// shortcut: 開発の窓で CDP からタブの状態を見るための口。開発のビルドだけ
if (import.meta.env.DEV) Object.assign(globalThis, { recipeDocs: docs, recipePanels: panels });

/** 見張りの知らせ: ツリー → シーン（10.1・10.5）→ 開いているタブ（10.2〜10.4）→ 検査（9.3） */
async function ghostChanged(paths: string[], gen: number): Promise<void> {
  const listing = await tree.refresh();
  if (gen !== generation) return;
  const now = listing?.files ?? files; // 一覧が読めなければ前の一覧のまま（理由は tree.refresh が出した）
  const plan = planGhostChanged(files, now, paths);
  files = now;
  await eachLimit(plan.scenes, 8, loadScenes);
  if (gen !== generation) return;
  await docs.applyExternal();
  if (plan.check && gen === generation) checker.request();
}
// Rust は今の master の知らせだけを送るが、移る前に送られて届くのが遅れたものはここで捨てる
await onGhostChanged((e) => {
  if (switching || current === null) return;
  const gen = generation;
  const run = () => (gen === generation ? ghostChanged(e.paths, gen) : undefined);
  changes = changes.then(run, run);
});

// 窓を閉じる: 未保存があれば 1 回の 3 択。取り消し・保存の失敗では閉じない（5.10）。
// handler の後で preventDefault が無ければ @tauri-apps/api が destroy() する（2.12 の onCloseRequested）
let closing = false;
await getCurrentWindow().onCloseRequested(async (e) => {
  if (closing) return e.preventDefault(); // 3 択の途中にもう一度閉じようとしても 2 つ目の 3 択は出さない
  closing = true;
  try {
    if (!(await docs.confirmUnsaved())) e.preventDefault();
    // 書いている途中で閉じない（fs::write は空にしてから書くので、途中で終わると中身を失う）
    else await Promise.all(docs.tabs.map((t) => t.saving.catch(() => {})));
  } finally {
    closing = false;
  }
});

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
