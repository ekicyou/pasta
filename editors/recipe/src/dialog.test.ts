/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { choiceOf, chooseResult, promptResult } from "./dialog.ts";

test("3 択は押したボタンを返し、Esc（空）や知らない値は取り消し（2.8・5.9・5.10）", () => {
  assert.equal(choiceOf("save"), "save");
  assert.equal(choiceOf("discard"), "discard");
  assert.equal(choiceOf("cancel"), "cancel");
  assert.equal(choiceOf(""), "cancel");
  assert.equal(choiceOf("ok"), "cancel");
});

test("名前の入力は決めたときだけ入れた値（空も Rust へ渡す）、Esc・取り消しは null（4.16）", () => {
  assert.equal(promptResult("ok", "朝.pasta"), "朝.pasta");
  assert.equal(promptResult("ok", ""), "");
  assert.equal(promptResult("cancel", "朝.pasta"), null);
  assert.equal(promptResult("", "朝.pasta"), null);
});

test("選ぶ窓は押したボタンの番号、Esc（空）・取り消し・並びの外は null（3.4）", () => {
  assert.equal(chooseResult("0", 2), 0);
  assert.equal(chooseResult("1", 2), 1);
  assert.equal(chooseResult("2", 2), null);
  assert.equal(chooseResult("", 2), null);
  assert.equal(chooseResult("cancel", 2), null);
  assert.equal(chooseResult("-1", 2), null);
  assert.equal(chooseResult("1.5", 2), null);
});
