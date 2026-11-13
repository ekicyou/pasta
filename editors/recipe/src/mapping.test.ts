/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import type { Finding, Report, SceneNode } from "./ipc.ts";
import {
  buildTree,
  diagnosticsToMarkers,
  findingPosition,
  MARKER_SEVERITY_ERROR,
  scenesToItems,
  tokensToData,
  visibleFindings,
  type TreeNode,
} from "./mapping.ts";

const range = (line: number, start: number, end: number) => ({
  start: { line, character: start },
  end: { line, character: end },
});

test("トークンを 5 つずつ並べる（6.2）", () => {
  const data = tokensToData([
    { deltaLine: 0, deltaStartCharacter: 1, length: 3, tokenType: 4, tokenModifiers: 0 },
    { deltaLine: 2, deltaStartCharacter: 0, length: 5, tokenType: 1, tokenModifiers: 2 },
  ]);
  assert.ok(data instanceof Uint32Array);
  assert.deepEqual([...data], [0, 1, 3, 4, 0, 2, 0, 5, 1, 2]);
  assert.equal(tokensToData([]).length, 0);
});

test("診断は行・列を +1 した重大度 Error の波線になり、全角・サロゲートペアの列も UTF-16 のまま +1（7.1・7.5）", () => {
  // 「＊あ😀x」の x: 全角 2 文字で 2、サロゲートペアで 2 → UTF-16 の 4 列目（0 始まり）
  const markers = diagnosticsToMarkers([
    { range: { start: { line: 0, character: 4 }, end: { line: 0, character: 5 } }, message: "誤り", severity: 1 },
    { range: { start: { line: 3, character: 0 }, end: { line: 4, character: 2 } }, message: "警告", severity: 2 },
  ]);
  assert.deepEqual(markers, [
    { severity: MARKER_SEVERITY_ERROR, message: "誤り", startLineNumber: 1, startColumn: 5, endLineNumber: 1, endColumn: 6 },
    { severity: MARKER_SEVERITY_ERROR, message: "警告", startLineNumber: 4, startColumn: 1, endLineNumber: 5, endColumn: 3 },
  ]);
  assert.equal("＊あ😀".length, 4);
});

test("シーンはグローバル ＞ ローカルの 2 階層・文書の順・同名も別項目、行は見出しの 1 始まり（4.4・4.5・4.13）", () => {
  const local = (name: string, line: number): SceneNode => ({
    name,
    kind: "local",
    heading: range(line, 0, 3),
    range: range(line, 0, 9),
    children: [],
  });
  const scenes: SceneNode[] = [
    { name: "挨拶", kind: "global", heading: range(0, 0, 3), range: range(0, 0, 9), children: [local("朝", 2), local("朝", 5)] },
    { name: "挨拶", kind: "global", heading: range(8, 0, 3), range: range(8, 0, 9), children: [] },
    { name: "別れ", kind: "global", heading: range(10, 0, 3), range: range(10, 0, 9), children: [] },
  ];
  const items = scenesToItems("dic/a.pasta", scenes);
  assert.deepEqual(items, [
    {
      kind: "scene",
      rel: "dic/a.pasta",
      name: "挨拶",
      sceneKind: "global",
      line: 1,
      children: [
        { kind: "scene", rel: "dic/a.pasta", name: "朝", sceneKind: "local", line: 3, children: [] },
        { kind: "scene", rel: "dic/a.pasta", name: "朝", sceneKind: "local", line: 6, children: [] },
      ],
    },
    { kind: "scene", rel: "dic/a.pasta", name: "挨拶", sceneKind: "global", line: 9, children: [] },
    { kind: "scene", rel: "dic/a.pasta", name: "別れ", sceneKind: "global", line: 11, children: [] },
  ]);
});

test("構文の誤りのあるファイルでは解析器が返した解ける部分だけを写し、自分で足さない（4.11・4.12）", () => {
  assert.deepEqual(scenesToItems("dic/b.pasta", []), []);
  const one: SceneNode[] = [{ name: "解けた", kind: "global", heading: range(4, 0, 3), range: range(4, 0, 6), children: [] }];
  assert.deepEqual(scenesToItems("dic/b.pasta", one).map((i) => [i.name, i.line]), [["解けた", 5]]);
});

const names = (node: TreeNode): unknown => [node.name, node.children.map(names)];

test("ツリーは同じ階層でフォルダが先、それぞれ大文字・小文字を区別しない名前の順（4.1・4.2）", () => {
  const root = buildTree({
    files: ["b.txt", "A.pasta", "dic/z.pasta", "dic/B.pasta", "dic/a.pasta", "Readme.txt", "c.lua"],
    dirs: ["dic", "Zeta", "alpha", "dic/Sub"],
  });
  assert.equal(root.kind, "dir");
  assert.equal(root.rel, "");
  assert.deepEqual(names(root), [
    "",
    [
      ["alpha", []],
      ["dic", [["Sub", []], ["a.pasta", []], ["B.pasta", []], ["z.pasta", []]]],
      ["Zeta", []],
      ["A.pasta", []],
      ["b.txt", []],
      ["c.lua", []],
      ["Readme.txt", []],
    ],
  ]);
  const dic = root.children.find((c) => c.name === "dic")!;
  assert.deepEqual(
    dic.children.map((c) => [c.kind, c.rel]),
    [
      ["dir", "dic/Sub"],
      ["file", "dic/a.pasta"],
      ["file", "dic/B.pasta"],
      ["file", "dic/z.pasta"],
    ],
  );
});

test("ツリーは dirs に無い途中のフォルダも files から作る", () => {
  const root = buildTree({ files: ["x/y/z.pasta"], dirs: [] });
  assert.deepEqual(names(root), ["", [["x", [["y", [["z.pasta", []]]]]]]]);
  assert.equal(root.children[0]!.children[0]!.rel, "x/y");
});

const finding = (path: string, code: string, line: number | null = 1, column: number | null = 1): Finding => ({
  code,
  severity: "error",
  path,
  line,
  column,
  message: code,
});

test("開いているファイルの構文の誤りだけを検査の一覧から除く（大文字・小文字を区別しない・9.8）", () => {
  const report: Report = {
    findings: [
      finding("dic/A.pasta", "syntax"),
      finding("dic/a.pasta", "undefined-scene"),
      finding("dic/b.pasta", "syntax"),
      finding("pasta.toml", "config"),
    ],
    stats: { dictionaryFiles: 2, errors: 4, warnings: 0, uncheckedDynamicCalls: 0, skipReasons: [] },
  };
  assert.deepEqual(
    visibleFindings(report, new Set(["dic/a.pasta"])).map((f) => [f.path, f.code]),
    [
      ["dic/a.pasta", "undefined-scene"],
      ["dic/b.pasta", "syntax"],
      ["pasta.toml", "config"],
    ],
  );
  assert.equal(visibleFindings(report, new Set()).length, 4);
});

test("同じファイルかは Rust の to_uppercase と同じく大文字に揃えて比べる（ı・ſ）", () => {
  const report: Report = {
    findings: [finding("dic/I.pasta", "syntax"), finding("dic/S.pasta", "syntax")],
    stats: { dictionaryFiles: 2, errors: 2, warnings: 0, uncheckedDynamicCalls: 0, skipReasons: [] },
  };
  assert.deepEqual(visibleFindings(report, new Set(["dic/ı.pasta", "dic/ſ.pasta"])), []);
});

test("検査の項目の位置は 1 始まりのまま、行の無い項目はファイルの先頭（9.6・9.7）", () => {
  assert.deepEqual(findingPosition(finding("a.pasta", "x", 3, 5)), { lineNumber: 3, column: 5 });
  assert.deepEqual(findingPosition(finding("a.pasta", "x", null, null)), { lineNumber: 1, column: 1 });
  assert.deepEqual(findingPosition(finding("a.pasta", "x", 7, null)), { lineNumber: 7, column: 1 });
});
