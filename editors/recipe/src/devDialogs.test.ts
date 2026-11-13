/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { boolValue, deleteMessage, nameRequired, pickedEndpoint, problemsByRow } from "./devDialogs.ts";

test("相手は押した番号の相手、閉じたら何も選ばない（3.4）", () => {
  const items = [
    { label: "SSP（127.0.0.1:9801）", endpoint: { port: 9801, basewareName: "SSP" } },
    { label: "areka（127.0.0.1:9821）", endpoint: { port: 9821, basewareName: "areka" } },
  ];
  assert.deepEqual(pickedEndpoint(items, 1), { port: 9821, basewareName: "areka" });
  assert.equal(pickedEndpoint(items, null), undefined);
});

test("追加の名前が空なら閉じずに名前が要ることを示す（7.5）", () => {
  assert.equal(nameRequired(""), "名前が要ります");
  assert.equal(nameRequired("pasta_talk_interval_min"), true);
});

test("削除の確かめは名前と保存変数から消えることを示す（7.3）", () => {
  assert.equal(deleteMessage("好感度"), "＄＊好感度 をゴーストの保存変数から消します");
});

test("真偽の初めの値は false のときだけ false、ほかは true（7.2・8.2）", () => {
  assert.equal(boolValue("false"), "false");
  assert.equal(boolValue("true"), "true");
  assert.equal(boolValue("3"), "true");
});

test("弾きの理由は 0 始まりの番号の組へ寄せ、同じ組の理由は並べ、並びの外は捨てる（8.6〜8.8）", () => {
  const byRow = problemsByRow(
    [
      { index: 0, message: "名前にできません" },
      { index: 2, message: "名前が重なっています" },
      { index: 2, message: "数として読めません" },
      { index: 5, message: "外" },
    ],
    3,
  );
  assert.deepEqual(byRow, ["名前にできません", "", "名前が重なっています／数として読めません"]);
});
