/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  DEFAULT_LAYOUT,
  KEY_STEP,
  LAYOUT_MIN,
  dragLayout,
  fitLayout,
  nudgeLayout,
  sameLayout,
  separatorValue,
  swapLayout,
  type LayoutPrefs,
} from "./layout.ts";

const wide = { width: 1200, height: 700 };
const narrow = { width: 400, height: 220 };
const prefs: LayoutPrefs = { treeWidth: 400, panelsHeight: 300, leftShare: 0.3, swapped: false };

test("既定の並びは今の見た目（260・180・半分ずつ・入れ替えなし）", () => {
  assert.deepEqual(DEFAULT_LAYOUT, { treeWidth: 260, panelsHeight: 180, leftShare: 0.5, swapped: false });
  assert.equal(KEY_STEP, 16);
  assert.deepEqual(LAYOUT_MIN, { tree: 120, editorWidth: 240, editorHeight: 120, panels: 60, panel: 160 });
});

test("広い窓では覚えた並びをそのまま返す（11.8）", () => {
  assert.deepEqual(fitLayout(prefs, wide), prefs);
});

test("狭い窓では最小を守って収め、覚えた値は書き換えない（11.6・11.8）", () => {
  const remembered = { ...prefs, leftShare: 0.1 };
  const got = fitLayout(remembered, narrow);
  assert.equal(got.treeWidth, 400 - 240); // ツリーはエディタの最小を残す
  assert.equal(got.panelsHeight, 220 - 120);
  assert.equal(got.leftShare, 160 / 400); // 左の欄は 160 px 以上
  assert.deepEqual(remembered, { ...prefs, leftShare: 0.1 });
  // 窓を広げ直すと覚えた並びに戻る
  assert.deepEqual(fitLayout(remembered, wide), { ...remembered, leftShare: 160 / 1200 });
  assert.deepEqual(fitLayout(prefs, wide), prefs);
});

test("窓の最小より小さい範囲では、ツリー・下の欄・2 つの欄の最小を先に守る", () => {
  const got = fitLayout(prefs, { width: 300, height: 150 });
  assert.equal(got.treeWidth, LAYOUT_MIN.tree);
  assert.equal(got.panelsHeight, LAYOUT_MIN.panels);
  assert.equal(got.leftShare, 0.5); // 320 px 未満では 2 つとも 160 にできないので半分ずつ
});

test("ツリーのドラッグは最小・最大で止まり、ほかのキーは覚えた値のまま（11.1・11.6・11.8）", () => {
  assert.equal(dragLayout(prefs, "tree", 10, wide).treeWidth, 120);
  assert.equal(dragLayout(prefs, "tree", 5000, wide).treeWidth, 1200 - 240);
  assert.equal(dragLayout(prefs, "tree", 333.6, wide).treeWidth, 334);
  // 狭い窓で 1 本を動かしても、ほかの 2 本は収めた値に書き換わらない
  assert.deepEqual(dragLayout(prefs, "tree", 150, narrow), { ...prefs, treeWidth: 150 });
});

test("下の欄のドラッグは最小・最大で止まる（11.2・11.6）", () => {
  assert.equal(dragLayout(prefs, "panels", 0, wide).panelsHeight, 60);
  assert.equal(dragLayout(prefs, "panels", 5000, wide).panelsHeight, 700 - 120);
  assert.deepEqual(dragLayout(prefs, "panels", 250, wide), { ...prefs, panelsHeight: 250 });
});

test("左右のドラッグは 2 つの欄を 160 px 以上に保ち、入れ替えに依らない（11.3・11.4・11.6）", () => {
  assert.equal(dragLayout(prefs, "split", 0, wide).leftShare, 160 / 1200);
  assert.equal(dragLayout(prefs, "split", 1200, wide).leftShare, 1 - 160 / 1200);
  const plain = dragLayout(prefs, "split", 480, wide);
  const swapped = dragLayout({ ...prefs, swapped: true }, "split", 480, wide);
  assert.equal(plain.leftShare, 0.4);
  assert.equal(swapped.leftShare, plain.leftShare);
  assert.equal(swapped.swapped, true);
});

test("矢印のキーは見た目の向きへ 16 px 動かす。下の欄は −1 で高くなる（11.5）", () => {
  assert.equal(nudgeLayout(prefs, "tree", 1, wide).treeWidth, 400 + KEY_STEP);
  assert.equal(nudgeLayout(prefs, "tree", -1, wide).treeWidth, 400 - KEY_STEP);
  assert.equal(nudgeLayout(prefs, "panels", -1, wide).panelsHeight, 300 + KEY_STEP);
  assert.equal(nudgeLayout(prefs, "panels", 1, wide).panelsHeight, 300 - KEY_STEP);
  assert.equal(nudgeLayout(prefs, "split", 1, wide).leftShare, (0.3 * 1200 + KEY_STEP) / 1200);
  assert.equal(nudgeLayout({ ...prefs, treeWidth: 125 }, "tree", -1, wide).treeWidth, 120);
  // 狭い窓では収めた値から動かし、ほかのキーは覚えた値のまま
  assert.deepEqual(nudgeLayout(prefs, "tree", -1, narrow), { ...prefs, treeWidth: 160 - KEY_STEP });
});

test("入れ替えは swapped だけを替え、2 回で元に戻る（11.4）", () => {
  const once = swapLayout(prefs);
  assert.deepEqual(once, { ...prefs, swapped: true });
  assert.deepEqual(swapLayout(once), prefs);
  assert.deepEqual(prefs.swapped, false);
});

test("読み上げの値: ツリーと下の欄は px、左右は左の欄の %（11.5）", () => {
  assert.deepEqual(separatorValue(prefs, "tree", wide), { now: 400, min: 120, max: 960 });
  assert.deepEqual(separatorValue(prefs, "panels", wide), { now: 300, min: 60, max: 580 });
  assert.deepEqual(separatorValue({ ...prefs, leftShare: 0.5 }, "split", { width: 1600, height: 700 }), { now: 50, min: 10, max: 90 });
});

test("数でない値は既定に替え、大きさ 0 でも落ちない", () => {
  const bad = { treeWidth: Number.NaN, panelsHeight: Number.POSITIVE_INFINITY, leftShare: Number.NaN, swapped: false };
  assert.deepEqual(fitLayout(bad, wide), DEFAULT_LAYOUT);
  assert.deepEqual(fitLayout({ ...prefs, leftShare: 1.5 }, wide).leftShare, 0.5);
  assert.deepEqual(fitLayout({ ...prefs, treeWidth: -5, panelsHeight: -5 }, wide), { ...prefs, treeWidth: 120, panelsHeight: 60 });
  const zero = { width: 0, height: 0 };
  assert.deepEqual(fitLayout(prefs, zero), { treeWidth: 120, panelsHeight: 60, leftShare: 0.5, swapped: false });
  assert.deepEqual(fitLayout(prefs, { width: Number.NaN, height: -1 }), fitLayout(prefs, zero));
  for (const id of ["tree", "panels", "split"] as const) {
    for (const v of Object.values(dragLayout(prefs, id, Number.NaN, zero))) assert.ok(typeof v === "boolean" || Number.isFinite(v));
    for (const v of Object.values(nudgeLayout(prefs, id, 1, zero))) assert.ok(typeof v === "boolean" || Number.isFinite(v));
    for (const v of Object.values(separatorValue(prefs, id, zero))) assert.ok(Number.isFinite(v));
  }
});

test("同じ並びかを 4 つのキーで比べる（値が変わらなければ書かない。11.7）", () => {
  assert.equal(sameLayout(prefs, { ...prefs }), true);
  for (const diff of [{ treeWidth: 401 }, { panelsHeight: 301 }, { leftShare: 0.31 }, { swapped: true }]) {
    assert.equal(sameLayout(prefs, { ...prefs, ...diff }), false);
  }
});
