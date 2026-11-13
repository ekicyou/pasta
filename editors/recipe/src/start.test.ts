import { test } from "node:test";
import assert from "node:assert/strict";
import { recentLabel } from "./start.ts";

test("最近開いたものの読み上げの名前は、ゴーストの名前とフルパスの両方を含む（3.4）", () => {
  const master = "C:\\ghosts\\hello-pasta\\ghost\\master";
  assert.equal(recentLabel({ master, name: "hello-pasta" }), `hello-pasta（${master}）`);
});
