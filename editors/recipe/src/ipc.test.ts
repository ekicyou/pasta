// tauriGhostIO が GhostIO を Rust のコマンドへ写すこと（3.9・9.4）。invoke は window.__TAURI_INTERNALS__ を差し替えて受ける
import { test } from "node:test";
import assert from "node:assert/strict";
import { tauriGhostIO } from "./ipc.ts";

type Handler = (cmd: string, args: Record<string, unknown>) => unknown;
const calls: [string, Record<string, unknown>][] = [];
const fake = (h: Handler) => {
  calls.length = 0;
  (globalThis as unknown as { window: unknown }).window = {
    __TAURI_INTERNALS__: {
      invoke: async (cmd: string, args: Record<string, unknown>) => {
        calls.push([cmd, args]);
        return h(cmd, args);
      },
    },
  };
};
const refuse: Handler = () => {
  throw "開発の操作が進んでいない";
};

test("送り手の名前は pasta-recipe、パスは \\ で結ぶ（1.3）", () => {
  assert.equal(tauriGhostIO.clientName, "pasta-recipe");
  assert.equal(tauriGhostIO.join("C:\\g\\ghost\\master", "profile", "pasta", "x.toml"), "C:\\g\\ghost\\master\\profile\\pasta\\x.toml");
});

test("post は dev_post へ port・body・timeoutMs を渡し、結果をそのまま返す（3.9）", async () => {
  fake(() => ({ kind: "unreachable", detail: "port 80 is not a baseware candidate" }));
  assert.deepEqual(await tauriGhostIO.post(80, "{}", 500), { kind: "unreachable", detail: "port 80 is not a baseware candidate" });
  assert.deepEqual(calls, [["dev_post", { port: 80, body: "{}", timeoutMs: 500 }]]);
});

test("list・stat・read は dev_ のコマンドへ写し、read の ArrayBuffer を Uint8Array にする", async () => {
  fake((cmd) => (cmd === "dev_list" ? ["a"] : cmd === "dev_stat" ? { size: 3, mtimeMs: 1, isFile: true } : new Uint8Array([1, 2]).buffer));
  assert.deepEqual(await tauriGhostIO.list("d"), ["a"]);
  assert.deepEqual(await tauriGhostIO.stat("f"), { size: 3, mtimeMs: 1, isFile: true });
  const bytes = await tauriGhostIO.read("f", 4);
  assert.ok(bytes instanceof Uint8Array);
  assert.deepEqual([...bytes], [1, 2]);
  assert.deepEqual(calls, [
    ["dev_list", { path: "d" }],
    ["dev_stat", { path: "f" }],
    ["dev_read", { path: "f", start: 4 }],
  ]);
});

test("門で断られた・無いは投げずに undefined（9.4）", async () => {
  fake(refuse);
  assert.equal(await tauriGhostIO.list("d"), undefined);
  assert.equal(await tauriGhostIO.stat("f"), undefined);
  assert.equal(await tauriGhostIO.read("f", 0), undefined);
});

test("post の invoke が投げても unreachable にする（GhostIO は投げない）", async () => {
  fake(refuse);
  assert.deepEqual(await tauriGhostIO.post(9801, "{}", 1), { kind: "unreachable", detail: "開発の操作が進んでいない" });
});
