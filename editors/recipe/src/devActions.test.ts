// アプリの開発の操作の糊（DevActions）を、偽の invoke（中身は偽の入出力）と偽の画面で確かめる（1.5・1.6・2.1・3.6・5.1・5.2）
import { test } from "node:test";
import assert from "node:assert/strict";
import { createFakeGhostIO, type FakeGhostIO, type PostRequest } from "../../shared/fakeGhostIO.ts";
import type { PostOutcome } from "../../shared/ghostIO.ts";
import { BUSY, SAVE_FAILED } from "../../shared/devSession.ts";
import { DevActions, noDescript } from "./devActions.ts";
import { isDevSource } from "./tabs.ts";

const info = { master: "C:\\g\\ghost\\master", root: "C:\\g", name: "g", version: "1" };
const LOG = "C:\\g\\ghost\\master\\profile\\pasta\\logs\\pasta.log";
const DONE = "2026-11-13T00:00:00Z  INFO pasta::windows: SHIORI load entry completed entry=\"loadu\" loaded=true\n";
const reply = (result: unknown): PostOutcome => ({ kind: "response", status: 200, body: JSON.stringify({ jsonrpc: "2.0", id: 1, result }) });
const text = (t: string) => reply({ content: [{ type: "text", text: t }], isError: false });

/** 9801 で SSP がこのゴーストを動かしている。reload で pasta.log に読み込みの終わりの行が出る */
function serve(fio: FakeGhostIO) {
  return (req: PostRequest): PostOutcome => {
    if (req.port !== 9801) return { kind: "unreachable", detail: "ECONNREFUSED" };
    const { method, params } = JSON.parse(req.body);
    if (method === "initialize") return reply({});
    if (params.name === "get_property") return text("SSP");
    if (params.name === "get_log") return text("(no log entries)");
    if (params.name === "reload") {
      fio.appendFile(LOG, DONE);
      return text("OK");
    }
    throw new Error(`筋書きに無い頼み: ${req.body}`);
  };
}

function setup(opts: { descript?: boolean; begin?: Promise<void> } = {}) {
  const fio = createFakeGhostIO("pasta-recipe");
  if (opts.descript !== false) fio.setFile(`${info.master}\\descript.txt`, "charset,UTF-8\r\nname,ゴースト\r\n");
  fio.queuePost(...Array.from({ length: 100 }, () => serve(fio)));
  const cmds: string[] = [];
  const orThrow = <T>(v: T | undefined): T => {
    if (v === undefined) throw "無い";
    return v;
  };
  (globalThis as unknown as { window: unknown }).window = {
    __TAURI_INTERNALS__: {
      invoke: async (cmd: string, a: { path: string; start: number; port: number; body: string; timeoutMs: number }) => {
        cmds.push(cmd);
        switch (cmd) {
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
  const tabs: { rel: string; deleted: boolean; savedVersionId: number; model: { getAlternativeVersionId(): number } }[] = [];
  const saved: string[] = [];
  const state = { saveOk: true as boolean | "throw", progressThrows: false };
  const actions = new DevActions({
    docs: {
      tabs: tabs as never,
      save: async (tab) => {
        saved.push(tab.rel);
        if (state.saveOk === "throw") throw new Error("壊れた");
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
    vars: { clear: () => void cleared++ },
    panels: { show: (f) => void shown.push(f) },
  });
  actions.setGhost(info);
  const dirty = (rel: string) => tabs.push({ rel, deleted: false, savedVersionId: 1, model: { getAlternativeVersionId: () => 2 } });
  const count = (cmd: string) => cmds.filter((c) => c === cmd).length;
  const tools = () => fio.posts.map((p) => JSON.parse(p.body).params?.name).filter(Boolean);
  return { actions, lines, notices, shown, saved, state, dirty, count, tools, cleared: () => cleared };
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
