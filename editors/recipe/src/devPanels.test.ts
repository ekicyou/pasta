/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { kindLabel, noticeLine, outputHeading, rowActionLabels } from "./devPanels.ts";

test("誤りと警告の行は頭の文字で見分けられる（4.3）", () => {
  assert.equal(noticeLine("error", "見つかりません"), "誤り: 見つかりません");
  assert.equal(noticeLine("warning", "読めません"), "警告: 読めません");
});

test("進んでいる間は出力の見出しに操作の名前を出す（4.6）", () => {
  assert.equal(outputHeading(null), "出力");
  assert.equal(outputHeading("シーンを実行"), "出力（実行中: シーンを実行）");
});

test("型は 文字列・数・真偽・表 で出す（6.2）", () => {
  assert.deepEqual(
    (["string", "number", "boolean", "table"] as const).map(kindLabel),
    ["文字列", "数", "真偽", "表"],
  );
});

test("表の行には操作を出さず、ほかは読み上げの名前を持つ（6.3・11.4）", () => {
  assert.equal(rowActionLabels({ name: "t", kind: "table", value: "{}" }), null);
  assert.deepEqual(rowActionLabels({ name: "好感度", kind: "number", value: "3" }), {
    edit: "＄＊好感度 を書き換える",
    del: "＄＊好感度 を消す",
  });
});
