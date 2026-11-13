// アプリの開発の操作の糊（DevActions）を、偽の invoke（中身は偽の入出力）と偽の画面で確かめる（1.5・1.6・2.1・3.6・5.1・5.2）
import { test } from "node:test";
import assert from "node:assert/strict";
import { createFakeGhostIO, type FakeGhostIO, type PostRequest } from "../../shared/fakeGhostIO.ts";
import type { PostOutcome } from "../../shared/ghostIO.ts";
import { BUSY, SAVE_FAILED } from "../../shared/devSession.ts";
import { DevActions, merge, noDescript, notFoundInSaved, notToml, switchedInDialog, switchedMidOp } from "./devActions.ts";
import { isDevSource } from "./tabs.ts";
import { sceneLabel, sceneTargets } from "./mapping.ts";
import type { LocalPair, SceneNode, VarRow } from "./ipc.ts";

const info = { master: "C:\\g\\ghost\\master", root: "C:\\g", name: "g", version: "1" };
const LOG = "C:\\g\\ghost\\master\\profile\\pasta\\logs\\pasta.log";
const DONE = "2026-11-13T00:00:00Z  INFO pasta::windows: SHIORI load entry completed entry=\"loadu\" loaded=true\n";
const reply = (result: unknown): PostOutcome => ({ kind: "response", status: 200, body: JSON.stringify({ jsonrpc: "2.0", id: 1, result }) });
const text = (t: string) => reply({ content: [{ type: "text", text: t }], isError: false });

const VARS = "C:\\g\\ghost\\master\\profile\\pasta\\debug\\variables.toml";
type RaiseArgs = { event: string; references: string[]; ghost_name: string };

/** 9801 で SSP がこのゴーストを動かしている。reload で pasta.log に読み込みの終わりの行が出る。raise_event は raise に任せる */
function serve(fio: FakeGhostIO, raise?: (a: RaiseArgs) => PostOutcome, refuseReload = () => false) {
  return (req: PostRequest): PostOutcome => {
    if (req.port !== 9801) return { kind: "unreachable", detail: "ECONNREFUSED" };
    const { method, params } = JSON.parse(req.body);
    if (method === "initialize") return reply({});
    if (params.name === "get_property") return text("SSP");
    if (params.name === "get_log") return text("(no log entries)");
    if (params.name === "reload") {
      if (refuseReload()) return { kind: "unreachable", detail: "ECONNRESET" };
      fio.appendFile(LOG, DONE);
      return text("OK");
    }
    if (params.name === "raise_event" && raise) return raise(params.arguments);
    throw new Error(`筋書きに無い頼み: ${req.body}`);
  };
}

/** 見出しの行（1 始まり）だけを持つシーン */
const scene = (name: string, line: number, kind: "global" | "local" = "global", children: SceneNode[] = []): SceneNode => {
  const r = { start: { line: line - 1, character: 0 }, end: { line: line - 1, character: 1 } };
  return { name, kind, heading: r, range: r, children };
};

interface SetupOpts {
  descript?: boolean;
  begin?: Promise<void>;
  raise?: (a: RaiseArgs, fio: FakeGhostIO) => PostOutcome;
  /** 解析: 中身 → シーン */
  analyze?: (source: string) => SceneNode[];
  /** ディスクの中身（read_text） */
  disk?: Record<string, string>;
  /** variable_rows: 文字列なら Err として投げる */
  rows?: (text: string) => VarRow[] | string;
  /** 窓の答え（無ければ閉じた＝null・false） */
  edit?: { name: string; kind: LocalPair["kind"]; value: string } | null;
  confirm?: boolean;
  locals?: (label: string, initial: readonly LocalPair[]) => { pairs: LocalPair[]; table: string | undefined } | null;
}

function setup(opts: SetupOpts = {}) {
  const fio = createFakeGhostIO("pasta-recipe");
  if (opts.descript !== false) fio.setFile(`${info.master}\\descript.txt`, "charset,UTF-8\r\nname,ゴースト\r\n");
  const raise = opts.raise;
  fio.queuePost(...Array.from({ length: 100 }, () => serve(fio, raise && ((a) => raise(a, fio)), () => state.refuseReload)));
  const analyzed: string[] = [];
  const rowTexts: string[] = [];
  const cmds: string[] = [];
  const orThrow = <T>(v: T | undefined): T => {
    if (v === undefined) throw "無い";
    return v;
  };
  (globalThis as unknown as { window: unknown }).window = {
    __TAURI_INTERNALS__: {
      invoke: async (cmd: string, a: { path: string; start: number; port: number; body: string; timeoutMs: number; source: string; rel: string; text: string }) => {
        cmds.push(cmd);
        switch (cmd) {
          case "analyze":
            analyzed.push(a.source);
            return { tokens: [], diagnostics: [], scenes: opts.analyze?.(a.source) ?? [] };
          case "read_text":
            state.during?.();
            return { text: orThrow(opts.disk?.[a.rel]), bom: false };
          case "variable_rows": {
            rowTexts.push(a.text);
            const r = opts.rows?.(a.text) ?? [];
            if (typeof r === "string") throw r;
            return r;
          }
          case "dev_begin":
            return opts.begin ?? null;
          case "dev_end":
            return null;
          case "dev_list":
            return orThrow(await fio.list(a.path));
          case "dev_stat":
            return orThrow(await fio.stat(a.path));
          case "dev_read":
            return orThrow(await fio.read(a.path, a.start)).buffer;
          case "dev_post":
            return fio.post(a.port, a.body, a.timeoutMs);
        }
        throw `知らないコマンド ${cmd}`;
      },
    },
  };
  const lines: string[] = [];
  const notices: string[] = [];
  const shown: string[] = [];
  let cleared = 0;
  const table: (readonly VarRow[] | "unreadable")[] = [];
  type FakeTab = { rel: string; deleted: boolean; savedVersionId: number; diskText: string | null; model: { getAlternativeVersionId(): number }; editorText?: string };
  const tabs: FakeTab[] = [];
  const saved: string[] = [];
  const dialogs: string[] = [];
  /** during: 保存と read_text の途中で呼ぶ（操作の途中でゴーストを開いたときを作る） */
  const state = { saveOk: true as boolean | "throw", progressThrows: false, refuseReload: false, during: undefined as (() => void) | undefined };
  const actions = new DevActions({
    docs: {
      tabs: tabs as never,
      save: async (tab) => {
        saved.push(tab.rel);
        state.during?.();
        if (state.saveOk === "throw") throw new Error("壊れた");
        const t = tab as unknown as FakeTab;
        if (state.saveOk && t.editorText !== undefined) t.diskText = t.editorText;
        return state.saveOk;
      },
    },
    output: {
      append: (ls) => void lines.push(...ls),
      notify: (level, t) => void notices.push(`${level}: ${t}`),
      progress: (_t, work) => {
        if (state.progressThrows) throw new Error("壊れた");
        return work();
      },
    },
    vars: {
      get state() {
        const t = table.at(-1);
        return t === undefined ? { kind: "none" as const } : t === "unreadable" ? { kind: "unreadable" as const } : { kind: "rows" as const, rows: t };
      },
      clear: () => void cleared++,
      setRows: (rows) => void table.push(rows),
      setUnreadable: () => void table.push("unreadable"),
    },
    panels: { show: (f) => void shown.push(f) },
    dialogs: {
      editVariable: async (row) => (dialogs.push(`edit:${row?.name ?? "(新)"}`), opts.edit ?? null),
      confirmDelete: async (name) => (dialogs.push(`delete:${name}`), opts.confirm ?? false),
      localsDialog: async (label, initial) => (dialogs.push(`locals:${label}:${JSON.stringify(initial)}`), opts.locals?.(label, initial) ?? null),
    },
  });
  actions.setGhost(info);
  const dirty = (rel: string, diskText: string | null = null, editorText?: string) =>
    tabs.push({ rel, deleted: false, savedVersionId: 1, diskText, editorText, model: { getAlternativeVersionId: () => 2 } });
  const count = (cmd: string) => cmds.filter((c) => c === cmd).length;
  const tools = () => fio.posts.map((p) => JSON.parse(p.body).params?.name).filter(Boolean);
  const raises = () =>
    fio.posts.map((p) => JSON.parse(p.body).params).filter((p) => p?.name === "raise_event").map((p) => p.arguments as RaiseArgs);
  return { actions, fio, lines, notices, shown, saved, state, dirty, count, tools, raises, analyzed, rowTexts, table, dialogs, cleared: () => cleared };
}

test("リロードは見出しの行と拡張と同じ要約を出し、動かす面を見せ、devBegin・devEnd で包む（2.1・4.2）", async () => {
  const s = setup();
  await s.actions.reload();
  assert.match(s.lines[0], /^\[\d\d:\d\d:\d\d\] SHIORI をリロード — ゴースト（C:\\g）→ SSP 127\.0\.0\.1:9801$/);
  assert.equal(s.lines.at(-1), "読み込みは誤り無く終わりました");
  assert.deepEqual(s.notices, []);
  assert.ok(s.shown.length > 0 && s.shown.every((f) => f === "run"));
  assert.equal(s.count("dev_begin"), 1);
  assert.equal(s.count("dev_end"), 1);
  assert.deepEqual(s.tools().filter((t) => t === "reload"), ["reload"]);
});

test("速い 2 回押しの 2 本目は重なりの文言で止め、2 回目の devBegin を呼ばない（5.1）", async () => {
  let release!: () => void;
  const s = setup({ begin: new Promise<void>((r) => (release = () => r())) });
  const first = s.actions.reload();
  await s.actions.reload();
  assert.match(s.lines[0], new RegExp(`SHIORI をリロード — ${BUSY}（g）$`));
  assert.deepEqual(s.notices, [`warning: ${BUSY}`]);
  release();
  await first;
  assert.equal(s.count("dev_begin"), 1);
  assert.equal(s.count("dev_end"), 1);
  // 終わった後はまた始められる
  await s.actions.reload();
  assert.equal(s.count("dev_begin"), 2);
});

test("descript.txt が無ければ何も送らず、ゴーストのフォルダとして扱えないと出し、devEnd を呼ぶ（3.6）", async () => {
  const s = setup({ descript: false });
  await s.actions.reload();
  assert.match(s.lines[0], new RegExp(`SHIORI をリロード — ${noDescript(info.master).replace(/[\\()（）]/g, "\\$&")}$`));
  assert.deepEqual(s.notices, [`warning: ${noDescript(info.master)}`]);
  assert.deepEqual(s.tools(), []);
  assert.equal(s.count("dev_end"), 1);
});

test("未保存の .pasta・.lua・pasta.toml だけを保存し、失敗があれば絶対パスを並べて読み込み直しを頼まない（1.5・1.6）", async () => {
  const s = setup();
  for (const rel of ["dic/a.pasta", "scripts/b.LUA", "pasta.toml", "readme.txt"]) s.dirty(rel);
  s.state.saveOk = false;
  await s.actions.reload();
  assert.deepEqual(s.saved, ["dic/a.pasta", "scripts/b.LUA", "pasta.toml"]);
  assert.ok(s.lines.includes(SAVE_FAILED));
  assert.ok(s.lines.includes("  C:\\g\\ghost\\master\\dic\\a.pasta"));
  assert.ok(!s.tools().includes("reload"));
  assert.equal(s.count("dev_end"), 1);
});

test("保存が投げたら保存できなかったものに数える（1.6）", async () => {
  const s = setup();
  s.dirty("dic/a.pasta");
  s.state.saveOk = "throw";
  await s.actions.reload();
  assert.ok(s.lines.includes(SAVE_FAILED));
  assert.ok(!s.tools().includes("reload"));
});

test("途中で投げても devEnd を呼び、誤りとして出し、次の操作を始められる（5.2）", async () => {
  const s = setup();
  s.state.progressThrows = true;
  await s.actions.reload();
  assert.deepEqual(s.notices, ["error: Error: 壊れた"]);
  assert.equal(s.count("dev_end"), 1);
  s.state.progressThrows = false;
  await s.actions.reload();
  assert.equal(s.count("dev_begin"), 2);
});

test("ゴーストを開いていなければ何もしない・替わったら表を消す（2.5・5.3）", async () => {
  const s = setup();
  assert.equal(s.cleared(), 1);
  s.actions.setGhost(info);
  assert.equal(s.cleared(), 1);
  s.actions.setGhost(null);
  assert.equal(s.cleared(), 2);
  await s.actions.reload();
  assert.equal(s.count("dev_begin"), 0);
});

test("開発の操作の前に保存するのは .pasta・.lua・名前が pasta.toml（大文字・小文字を問わない）", () => {
  assert.ok(isDevSource("dic/a.PASTA") && isDevSource("x.lua") && isDevSource("PASTA.TOML"));
  assert.ok(!isDevSource("dic/pasta.toml.bak") && !isDevSource("descript.txt"));
});

const SCRIPT = "\\h\\s[0]おはよう\\e";
const OTHER = { master: "C:\\h\\ghost\\master", root: "C:\\h", name: "h", version: "1" };
const played = (a: RaiseArgs) => (a.event === "OnPastaSceneAt" ? text(`Script: ${SCRIPT}`) : text("OK:the ghost returned no script."));
const morning = { rel: "dic/a.pasta", path: [{ name: "挨拶", n: 1 }, { name: "朝", n: 0 }], label: "＊挨拶・朝" };

test("▶ は保存の後の中身を解析し直し、道から今の行を引いて送る。ローカル変数（Reference2）は送らない（1.1・1.4・8.11）", async () => {
  const s = setup({
    raise: played,
    analyze: (src) => (src === "NEW" ? [scene("挨拶", 3), scene("挨拶", 7, "global", [scene("朝", 9, "local")])] : []),
  });
  s.dirty("dic/a.pasta", "OLD", "NEW");
  await s.actions.run(morning);
  assert.deepEqual(s.saved, ["dic/a.pasta"]);
  assert.deepEqual(s.analyzed, ["NEW"]);
  assert.deepEqual(s.raises(), [{ event: "OnPastaSceneAt", references: [`${info.master}\\dic\\a.pasta`, "9"], ghost_name: info.root }]);
  assert.match(s.lines[0], /▶ シーンを実行 — ゴースト/);
  assert.ok(s.lines.includes("台本:") && s.lines.includes(SCRIPT));
  assert.deepEqual(s.notices, []);
  assert.equal(s.count("dev_end"), 1);
});

test("▶ はタブで開いていないファイルならディスクの中身を解析し、道が無ければ読み込み直しも再生も頼まずに止める（1.4）", async () => {
  const s = setup({ raise: played, disk: { "dic/b.pasta": "DISK" }, analyze: () => [scene("挨拶", 2)] });
  const target = { rel: "dic/b.pasta", path: [{ name: "挨拶", n: 1 }], label: "＊挨拶" };
  await s.actions.run(target);
  assert.deepEqual(s.analyzed, ["DISK"]);
  assert.ok(s.lines.includes(notFoundInSaved("＊挨拶", "dic/b.pasta")));
  assert.deepEqual(s.notices, [`warning: ${notFoundInSaved("＊挨拶", "dic/b.pasta")}`]);
  assert.ok(!s.tools().includes("reload") && s.raises().length === 0);
});

test("▶ の保存の途中で別のゴーストを開いたら、中身を読まずに前のゴーストの見出しの下で止める（5.2）", async () => {
  const s = setup({ raise: played, disk: { "dic/a.pasta": "DISK" }, analyze: () => [scene("挨拶", 7, "global", [scene("朝", 9, "local")])] });
  s.dirty("dic/a.pasta", "OLD", "NEW");
  s.state.during = () => s.actions.setGhost(OTHER);
  await s.actions.run(morning);
  assert.equal(s.count("read_text") + s.count("analyze"), 0);
  assert.equal(s.raises().length, 0);
  assert.match(s.lines[0], /▶ シーンを実行 — ゴースト（C:\\g）/);
  assert.ok(s.lines.includes(switchedMidOp));
  assert.deepEqual(s.notices, [`warning: ${switchedMidOp}`]);
  assert.equal(s.count("dev_end"), 1);
});

test("▶ のディスクの読みの途中で別のゴーストを開いたら、読んだ中身を使わずに止める（5.2）", async () => {
  const s = setup({ raise: played, disk: { "dic/a.pasta": "DISK" }, analyze: () => [scene("挨拶", 7, "global", [scene("朝", 9, "local")])] });
  s.state.during = () => s.actions.setGhost(null); // 切り替えの始め（main.ts の openNow）
  await s.actions.run(morning);
  assert.equal(s.count("analyze"), 0);
  assert.equal(s.raises().length, 0);
  assert.deepEqual(s.notices, [`warning: ${switchedMidOp}`]);
});

const dumped =(a: RaiseArgs, fio: FakeGhostIO) => {
  if (a.event === "OnPastaDumpVariables") fio.setFile(VARS, "a = 1\n");
  return text("OK:the ghost returned no script.");
};

test("保存変数を表示は書き出しを待って表の行にし、表へ当てる。保存も読み込み直しもしない（6.1）", async () => {
  const row: VarRow = { name: "a", kind: "number", value: "1" };
  const s = setup({ raise: dumped, rows: () => [row] });
  s.dirty("dic/a.pasta");
  await s.actions.showVariables();
  assert.deepEqual(s.raises(), [{ event: "OnPastaDumpVariables", references: [], ghost_name: info.root }]);
  assert.deepEqual(s.rowTexts, ["a = 1\n"]);
  assert.deepEqual(s.table, [[row]]);
  assert.deepEqual(s.saved, []);
  assert.ok(!s.tools().includes("reload"));
  assert.deepEqual(s.notices, []);
});

test("TOML として読めなければ表を読めなかったにして、場所と理由を出す（6.5）", async () => {
  const s = setup({ raise: dumped, rows: () => "expected `=`" });
  await s.actions.showVariables();
  assert.deepEqual(s.table, ["unreadable"]);
  assert.ok(s.lines.includes(notToml(VARS, "expected `=`")));
  assert.deepEqual(s.notices, [`warning: ${notToml(VARS, "expected `=`")}`]);
});

test("書き出しを頼めなければ表を読めなかったにして、誤りとして出す（6.4）", async () => {
  const s = setup({ raise: () => ({ kind: "unreachable", detail: "ECONNREFUSED" }) });
  await s.actions.showVariables();
  assert.deepEqual(s.table, ["unreadable"]);
  assert.equal(s.notices.length, 1);
  assert.match(s.notices[0], /^error: 保存変数の書き出しを頼めませんでした/);
});

test("操作の間に別のゴーストへ移ったら、前のゴーストの表の行は当てずに捨てる（5.3）", async () => {
  const other = OTHER;
  const holder: { s?: ReturnType<typeof setup> } = {};
  holder.s = setup({
    raise: (a, fio) => {
      holder.s!.actions.setGhost(other);
      return dumped(a, fio);
    },
    rows: () => [{ name: "a", kind: "number", value: "1" }],
  });
  const s = holder.s;
  await s.actions.showVariables();
  assert.deepEqual(s.table, []);
  assert.equal(s.cleared(), 2);
});

test("別のゴーストを渡すと表を消し、表は読みに行かない（5.3・6.8）", () => {
  const s = setup();
  s.actions.setGhost(OTHER);
  assert.equal(s.cleared(), 2);
  assert.equal(s.count("dev_begin") + s.count("dev_read") + s.tools().length, 0);
});

test("見出しの上の操作は解析のシーンごとに、道と見出しの名前を付けて並ぶ（1.2）", () => {
  const scenes = [scene("挨拶", 1, "global", [scene("朝", 2, "local"), scene("朝", 4, "local")]), scene("挨拶", 6)];
  assert.deepEqual(
    sceneTargets("dic/a.pasta", scenes).map((t) => [t.line, t.target.label, t.target.path.map((p) => p.n).join(",")]),
    [
      [1, "＊挨拶", "0"],
      [2, "＊挨拶・朝", "0,0"],
      [4, "＊挨拶・朝", "0,1"],
      [6, "＊挨拶", "1"],
    ],
  );
  assert.equal(sceneLabel([{ name: "a", n: 0 }]), "＊a");
  assert.ok(sceneTargets("dic/a.pasta", scenes).every((t) => t.target.rel === "dic/a.pasta"));
});

const ROW_A: VarRow = { name: "a", kind: "number", value: "1" };
/** 書き換えの答えを set に任せ、書き出しは variables.toml を書く */
const setAnd = (set: () => PostOutcome) => (a: RaiseArgs, fio: FakeGhostIO) => (a.event === "OnPastaSetSaveVariable" ? set() : dumped(a, fio));
const ok = () => text("OK:the ghost returned no script.");
const dumps = (s: ReturnType<typeof setup>) => s.raises().filter((r) => r.event === "OnPastaDumpVariables").length;

test("書き換えは窓の後に名前・値・型を送り、同じ操作の中で表を読み直す。ファイルは書かない（7.1・7.6・7.9）", async () => {
  const s = setup({ raise: setAnd(ok), rows: () => [{ ...ROW_A, value: "2" }], edit: { name: "a", kind: "number", value: "2" } });
  await s.actions.editVariable(ROW_A);
  assert.deepEqual(s.dialogs, ["edit:a"]);
  assert.deepEqual(s.raises(), [
    { event: "OnPastaSetSaveVariable", references: ["a", "2", "number"], ghost_name: info.root },
    { event: "OnPastaDumpVariables", references: [], ghost_name: info.root },
  ]);
  assert.deepEqual(s.table, [[{ ...ROW_A, value: "2" }]]);
  assert.equal(s.lines.filter((l) => /— ゴースト/.test(l)).length, 1);
  assert.equal(s.count("dev_begin"), 1);
  assert.equal(s.count("dev_end"), 1);
  assert.ok(s.lines.includes("＄＊a を 2（number）にしました"));
  assert.deepEqual(s.notices, []);
  assert.deepEqual(s.saved, []);
});

test("ゴーストの断りは警告として出し、表を読み直す（7.7）", async () => {
  const s = setup({ raise: setAnd(() => text("Script: 【数にできない値：x】")), rows: () => [ROW_A], edit: { name: "a", kind: "number", value: "x" } });
  await s.actions.editVariable(ROW_A);
  assert.deepEqual(s.notices, ["warning: 保存変数を書き換えられませんでした"]);
  assert.deepEqual(s.table, [[ROW_A]]);
  assert.equal(s.count("dev_begin"), 1);
});

test("頼みが届かない・形が違うときは誤りとして出し、表の読み直しも同じ操作の中で試みる（7.8）", async () => {
  for (const set of [(): PostOutcome => ({ kind: "unreachable", detail: "ECONNREFUSED" }), () => reply({ nope: 1 })]) {
    const s = setup({ raise: setAnd(set), rows: () => [ROW_A], edit: { name: "a", kind: "number", value: "2" } });
    await s.actions.editVariable(ROW_A);
    assert.equal(s.notices.length, 1);
    assert.match(s.notices[0], /^error: 保存変数の書き換えを頼めませんでした/);
    assert.equal(dumps(s), 1);
    assert.equal(s.count("dev_begin"), 1);
  }
});

test("削除は確かめた後に型 delete で送り、取り消せば何も送らない（7.3）", async () => {
  const s = setup({ raise: setAnd(ok), confirm: true });
  await s.actions.deleteVariable(ROW_A);
  assert.deepEqual(s.dialogs, ["delete:a"]);
  assert.deepEqual(s.raises()[0].references, ["a", "", "delete"]);
  assert.equal(dumps(s), 1);
  const c = setup({ raise: setAnd(ok), confirm: false });
  await c.actions.deleteVariable(ROW_A);
  assert.equal(c.count("dev_begin") + c.raises().length, 0);
});

test("追加は窓で入れた名前で送り、窓を閉じたら何も送らない（7.4）", async () => {
  const s = setup({ raise: setAnd(ok), edit: { name: "pasta_talk_interval_min", kind: "number", value: "5" } });
  await s.actions.addVariable();
  assert.deepEqual(s.dialogs, ["edit:(新)"]);
  assert.deepEqual(s.raises()[0].references, ["pasta_talk_interval_min", "5", "number"]);
  const c = setup({ raise: setAnd(ok) });
  await c.actions.addVariable();
  await c.actions.editVariable(ROW_A);
  assert.equal(c.count("dev_begin") + c.raises().length, 0);
});

const morningScenes = () => [scene("挨拶", 3), scene("挨拶", 7, "global", [scene("朝", 9, "local")])];

test("値を入れて実行はシーンごとに組を覚え、組があればインライン表を Reference2 に、0 なら付けない。▶ は組を送らない（8.3・8.5・8.10・8.11）", async () => {
  const PAIRS: LocalPair[] = [{ name: "x", kind: "number", value: "1" }];
  let answer: { pairs: LocalPair[]; table: string | undefined } = { pairs: PAIRS, table: "{ x = 1 }" };
  const s = setup({ raise: played, disk: { "dic/a.pasta": "DISK" }, analyze: morningScenes, locals: () => answer });
  await s.actions.runWithLocals(morning);
  answer = { pairs: [], table: undefined };
  await s.actions.runWithLocals(morning);
  await s.actions.runWithLocals({ ...morning, path: [{ name: "挨拶", n: 1 }], label: "＊挨拶" });
  assert.deepEqual(s.dialogs, ["locals:＊挨拶・朝:[]", `locals:＊挨拶・朝:${JSON.stringify(PAIRS)}`, "locals:＊挨拶:[]"]);
  const refs = s.raises().map((r) => r.references);
  assert.deepEqual(refs[0], [`${info.master}\\dic\\a.pasta`, "9", "{ x = 1 }"]);
  assert.deepEqual(refs[1], [`${info.master}\\dic\\a.pasta`, "9"]);
  assert.match(s.lines[0], /値を入れて実行 — ゴースト/);
  await s.actions.run(morning);
  assert.deepEqual(s.raises().at(-1)!.references, [`${info.master}\\dic\\a.pasta`, "9"]);
});

test("値を入れて実行の窓を閉じたら何も送らない（8.1）", async () => {
  const s = setup({ raise: played, disk: { "dic/a.pasta": "DISK" }, analyze: morningScenes });
  await s.actions.runWithLocals(morning);
  assert.equal(s.count("dev_begin") + s.raises().length, 0);
});

test("値を入れて実行の窓の間に別のゴーストを開いたら、何も送らずに前のゴーストの名前を添えて止める（5.2）", async () => {
  const holder: { s?: ReturnType<typeof setup> } = {};
  holder.s = setup({
    raise: played,
    disk: { "dic/a.pasta": "DISK" },
    analyze: morningScenes,
    locals: () => (holder.s!.actions.setGhost(OTHER), { pairs: [], table: undefined }),
  });
  const s = holder.s;
  await s.actions.runWithLocals(morning);
  assert.equal(s.count("dev_begin") + s.raises().length, 0);
  assert.match(s.lines[0], new RegExp(`値を入れて実行 — ${switchedInDialog}（g）$`));
  assert.deepEqual(s.notices, [`warning: ${switchedInDialog}`]);
});

test("書き換え・削除・追加の窓の間に別のゴーストを開いたら、前の表の名前を送らずに止める（5.2）", async () => {
  for (const act of [(a: DevActions) => a.editVariable(ROW_A), (a: DevActions) => a.deleteVariable(ROW_A), (a: DevActions) => a.addVariable()]) {
    const s = setup({ raise: setAnd(ok), edit: { name: "a", kind: "number", value: "2" }, confirm: true });
    const p = act(s.actions);
    s.actions.setGhost(OTHER); // 窓の答えが返る前
    await p;
    assert.equal(s.count("dev_begin") + s.raises().length, 0);
    assert.match(s.lines[0], new RegExp(`保存変数を書き換え — ${switchedInDialog}（g）$`));
    assert.deepEqual(s.notices, [`warning: ${switchedInDialog}`]);
  }
});

test("窓を閉じたときに別の操作が進んでいれば、重なりの文言で止めて何も送らない（5.1）", async () => {
  let release!: () => void;
  const s = setup({
    raise: setAnd(ok),
    begin: new Promise<void>((r) => (release = () => r())),
    disk: { "dic/a.pasta": "DISK" },
    analyze: morningScenes,
    locals: () => ({ pairs: [], table: undefined }),
    edit: { name: "a", kind: "number", value: "2" },
  });
  const first = s.actions.reload();
  await s.actions.runWithLocals(morning);
  await s.actions.editVariable(ROW_A);
  assert.deepEqual(s.notices, [`warning: ${BUSY}`, `warning: ${BUSY}`]);
  assert.match(s.lines[0], new RegExp(`値を入れて実行 — ${BUSY}（g）$`));
  assert.match(s.lines[1], new RegExp(`保存変数を書き換え — ${BUSY}（g）$`));
  release();
  await first;
  assert.equal(s.count("dev_begin"), 1);
  assert.equal(s.raises().length, 0);
});

test("表を読めている間は、▶・値を入れて実行で鳴らした後に同じ操作の中で表を読み直す（6.9）", async () => {
  const raise = (a: RaiseArgs, fio: FakeGhostIO) => (a.event === "OnPastaDumpVariables" ? dumped(a, fio) : played(a));
  const s = setup({ raise, disk: { "dic/a.pasta": "DISK" }, analyze: morningScenes, rows: () => [ROW_A], locals: () => ({ pairs: [], table: undefined }) });
  await s.actions.run(morning); // まだ読んでいない間は読み直さない
  assert.equal(dumps(s), 0);
  await s.actions.showVariables();
  await s.actions.run(morning);
  await s.actions.runWithLocals(morning);
  assert.deepEqual(s.raises().map((r) => r.event), ["OnPastaSceneAt", "OnPastaDumpVariables", "OnPastaSceneAt", "OnPastaDumpVariables", "OnPastaSceneAt", "OnPastaDumpVariables"]);
  assert.equal(s.table.length, 3);
  assert.equal(s.count("dev_begin"), 4);
});

test("鳴らせなかったとき（読み込み直しを頼めない）は表を読み直さない（6.9）", async () => {
  const s = setup({ raise: dumped, disk: { "dic/a.pasta": "DISK" }, analyze: morningScenes, rows: () => [ROW_A] });
  await s.actions.showVariables();
  s.state.refuseReload = true;
  await s.actions.run(morning);
  assert.equal(dumps(s), 1);
});

test("merge は先の知らせを採るが、後だけが失敗なら誤りを警告へ下げないよう後を採る", () => {
  const warn = { lines: ["a"], notice: "断り" };
  const err = { lines: ["b"], notice: "届かない", failure: { kind: "unreachable" as const, detail: "x" } };
  assert.deepEqual(merge(warn, err), { ...err, lines: ["a", "b"] });
  assert.deepEqual(merge(warn, { lines: ["b"] }), { ...warn, lines: ["a", "b"] });
  assert.deepEqual(merge({ lines: ["x"] }, warn), { ...warn, lines: ["x", "a"] });
});
