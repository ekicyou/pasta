/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { afterChoice, needsConfirm, retargetTabs, saveAll, type TabRef } from "./tabs.ts";

const t = (id: number, rel: string, dirty = false): TabRef => ({ id, rel, dirty });

test("ファイルの名前の変更はそのタブだけを付け替える（4.15）", () => {
  const got = retargetTabs([t(1, "dic/a.pasta"), t(2, "dic/b.pasta")], "dic/a.pasta", "dic/c.pasta");
  assert.deepEqual([...got], [[1, "dic/c.pasta"]]);
});

test("フォルダの名前の変更は中のタブを付け替え、dict は取り違えない（4.15）", () => {
  const tabs = [t(1, "dic/a.pasta"), t(2, "dic/sub/b.lua"), t(3, "dict/c.pasta"), t(4, "dic.pasta")];
  const got = retargetTabs(tabs, "dic", "words");
  assert.deepEqual([...got], [
    [1, "words/a.pasta"],
    [2, "words/sub/b.lua"],
  ]);
});

test("大文字・小文字の違うタブも付け替える（4.15）", () => {
  const got = retargetTabs([t(1, "DIC/A.pasta"), t(2, "Dic/b.pasta")], "dic", "Words");
  assert.deepEqual([...got], [
    [1, "Words/A.pasta"],
    [2, "Words/b.pasta"],
  ]);
});

test("未保存のタブがあるときだけ確かめる（2.8・5.9・5.10）", () => {
  assert.equal(needsConfirm([]), false);
  assert.equal(needsConfirm([t(1, "a", false)]), false);
  assert.equal(needsConfirm([t(1, "a", false), t(2, "b", true)]), true);
});

test("全部保存は 2 つ目の失敗で止め、3 つ目を呼ばない（2.8・5.10）", async () => {
  const called: number[] = [];
  const got = await saveAll([t(1, "a"), t(2, "b"), t(3, "c")], async (tab) => {
    called.push(tab.id);
    return tab.id !== 2;
  });
  assert.equal(got, "failed");
  assert.deepEqual(called, [1, 2]);
});

test("全部保存がすべて成功すれば ok", async () => {
  const called: number[] = [];
  assert.equal(await saveAll([t(1, "a"), t(2, "b")], async (tab) => (called.push(tab.id), true)), "ok");
  assert.deepEqual(called, [1, 2]);
});

test("3 択の後: 保存の成功と保存しないは進み、失敗と取り消しは留まる（2.8・5.9・5.10）", () => {
  assert.equal(afterChoice("save", "ok"), "proceed");
  assert.equal(afterChoice("discard", null), "proceed");
  assert.equal(afterChoice("save", "failed"), "stay");
  assert.equal(afterChoice("cancel", null), "stay");
});
