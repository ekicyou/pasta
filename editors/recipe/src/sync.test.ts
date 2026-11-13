/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { decideExternal, failedRead, planGhostChanged, type DiskState, type TabSnapshot } from "./sync.ts";

const tab = (over: Partial<TabSnapshot> = {}): TabSnapshot => ({
  diskText: "a",
  diskBom: false,
  dirty: false,
  deleted: false,
  editorText: "a",
  ...over,
});
const text = (t: string, bom = false): DiskState => ({ kind: "text", text: t, bom });
const missing: DiskState = { kind: "missing" };
const unreadable: DiskState = { kind: "unreadable", reason: "使用中" };

test("中身と BOM が同じなら何もしない（自分の保存・10.6）", () => {
  assert.equal(decideExternal(tab(), text("a")), "none");
  assert.equal(decideExternal(tab({ dirty: true, editorText: "b" }), text("a")), "none");
  assert.equal(decideExternal(tab({ diskBom: true }), text("a", true)), "none");
});

test("未保存でなく中身が違えば読み直す（10.2）", () => {
  assert.equal(decideExternal(tab(), text("b")), "reload");
  // BOM だけ違っても読み直す
  assert.equal(decideExternal(tab(), text("a", true)), "reload");
  assert.equal(decideExternal(tab({ diskBom: true }), text("a")), "reload");
});

test("未保存で中身が違えば尋ねる（10.3）", () => {
  assert.equal(decideExternal(tab({ dirty: true, editorText: "x" }), text("b")), "ask");
  assert.equal(decideExternal(tab({ dirty: true, editorText: "x" }), text("a", true)), "ask");
});

test("消えたら消えたと決める・既に消えていれば何もしない（10.4）", () => {
  assert.equal(decideExternal(tab(), missing), "deleted");
  assert.equal(decideExternal(tab({ dirty: true }), missing), "deleted");
  assert.equal(
    decideExternal(tab({ diskText: null, deleted: true, dirty: true }), missing),
    "none",
  );
});

test("読めないときは次の知らせで決める", () => {
  assert.equal(decideExternal(tab(), unreadable), "none");
  assert.equal(decideExternal(tab({ dirty: true }), unreadable), "none");
  assert.equal(decideExternal(tab({ diskText: null, deleted: true, dirty: true }), unreadable), "none");
});

test("消えた後に戻ってきた: エディタと同じ中身なら何もしない・違えば尋ねる", () => {
  const gone = tab({ diskText: null, deleted: true, dirty: true, editorText: "a" });
  assert.equal(decideExternal(gone, text("a")), "none");
  assert.equal(decideExternal(gone, text("a", true)), "none");
  assert.equal(decideExternal(gone, text("b")), "ask");
  // 未保存の印が無くても、戻ってきた違う中身は黙って読み直さない
  assert.equal(decideExternal({ ...gone, dirty: false }, text("b")), "ask");
});

test("読めないときはツリーに無ければ消えた・あれば読めない（大文字・小文字を区別しない）", () => {
  assert.deepEqual(failedRead("dic/a.pasta", ["dic/b.pasta"], "x"), missing);
  assert.deepEqual(failedRead("dic/a.pasta", ["DIC/A.pasta"], "使用中"), unreadable);
});

test("見張りの知らせ: 消えた・知らせの・新しく出た .pasta のシーンを読み直す（10.1・10.5）", () => {
  const plan = planGhostChanged(["dic/a.pasta", "dic/b.pasta", "x.txt"], ["dic/B.pasta", "dic/c.pasta", "x.txt"], ["dic/b.pasta", "dic", "x.txt"]);
  assert.deepEqual(plan.scenes, ["dic/a.pasta", "dic/b.pasta", "dic/c.pasta"]);
});

test("見張りの知らせ: .pasta・.lua・pasta.toml が含まれるか、その並びが変われば検査する（9.3）", () => {
  const files = ["dic/a.pasta", "scripts/m.lua", "x.txt"];
  assert.equal(planGhostChanged(files, files, ["x.txt"]).check, false);
  assert.equal(planGhostChanged(files, files, ["dic"]).check, false);
  assert.equal(planGhostChanged(files, files, ["dic/A.PASTA"]).check, true);
  assert.equal(planGhostChanged(files, files, ["scripts/m.lua"]).check, true);
  assert.equal(planGhostChanged(files, files, ["pasta.toml"]).check, true);
  // フォルダの名前の変更は知らせにフォルダしか来なくても、並びが変わる
  assert.equal(planGhostChanged(files, ["dic2/a.pasta", "scripts/m.lua", "x.txt"], ["dic", "dic2"]).check, true);
  assert.equal(planGhostChanged(files, ["dic/a.pasta", "scripts/m.lua", "x.txt", "y.txt"], ["y.txt"]).check, false);
});
