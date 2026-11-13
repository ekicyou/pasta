/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import lightPlus from "@shikijs/themes/light-plus";
import darkPlus from "@shikijs/themes/dark-plus";
import vscodePackage from "../../vscode/package.json" with { type: "json" };
import { keepTokens, resolveScopeStyle, withSemanticRules, type TmTheme } from "./theme.ts";

const scopes = vscodePackage.contributes.semanticTokenScopes.find(
  (entry) => entry.language === "pasta",
)!.scopes as Record<string, string[]>;

const sample: TmTheme = {
  name: "sample",
  type: "light",
  tokenColors: [
    { settings: { foreground: "#111111" } },
    { scope: "keyword", settings: { foreground: "#000001" } },
    { scope: "keyword.control, string", settings: { foreground: "#000002" } },
    { scope: ["keyword.control.scene"], settings: { fontStyle: "bold" } },
    { scope: "keyword.control.scene.pasta", settings: { foreground: "#000004" } },
    { scope: "meta.talk keyword.control", settings: { foreground: "#999999" } },
    { scope: "keywordx", settings: { foreground: "#999998" } },
  ],
};

test("選択子は点で区切った先頭一致の最も長いものを使い、色と字形は別々に解く", () => {
  assert.deepEqual(resolveScopeStyle(sample, "keyword.control.scene.pasta"), {
    foreground: "#000004",
    fontStyle: "bold",
  });
  assert.deepEqual(resolveScopeStyle(sample, "keyword.control.pasta"), { foreground: "#000002" });
  assert.deepEqual(resolveScopeStyle(sample, "keyword.other"), { foreground: "#000001" });
  assert.deepEqual(resolveScopeStyle(sample, "string.quoted"), { foreground: "#000002" });
  // 子孫の選択子・語の途中の一致・スコープの無い規則は使わない
  assert.equal(resolveScopeStyle(sample, "keywordx.y")?.foreground, "#999998");
  assert.equal(resolveScopeStyle(sample, "meta.talk"), undefined);
  assert.equal(resolveScopeStyle(sample, "comment"), undefined);
});

test("写し先が色を持たない種類には規則を足さず、元のテーマは変えない", () => {
  const before = JSON.stringify(sample);
  const out = withSemanticRules(sample, { scene: ["keyword.control.scene.pasta"], comment: ["comment.line"] });
  assert.equal(JSON.stringify(sample), before);
  const added = out.tokenColors.slice(sample.tokenColors.length);
  assert.deepEqual(added, [
    { scope: "scene", settings: { foreground: "#000004", fontStyle: "bold" } },
  ]);
  assert.equal(out.name, "sample");
});

for (const theme of [lightPlus, darkPlus] as unknown as TmTheme[]) {
  test(`${theme.name}: 写し先が色を持つ種類すべてに規則が付き、同じスコープの種類は同じ色`, () => {
    const out = withSemanticRules(theme, scopes);
    const rules = new Map(
      out.tokenColors.slice(theme.tokenColors.length).map((r) => [r.scope as string, r.settings]),
    );
    let colored = 0;
    for (const [kind, kindScopes] of Object.entries(scopes)) {
      const style = resolveScopeStyle(theme, kindScopes[0]!);
      if (style?.foreground) {
        colored++;
        assert.equal(rules.get(kind)?.foreground, style.foreground, kind);
      } else {
        assert.equal(rules.get(kind)?.foreground, undefined, kind);
      }
    }
    assert.ok(colored >= 10, `色の付いた種類が少なすぎる: ${colored}`);
    assert.ok(rules.get("scene")?.foreground);
    assert.equal(rules.get("namespace")?.foreground, rules.get("scene")?.foreground);
    assert.equal(
      rules.get("comment")?.foreground,
      resolveScopeStyle(theme, "comment")?.foreground,
    );
  });
}

const tok = (deltaLine: number, deltaStartCharacter: number, tokenType: number) => ({
  deltaLine,
  deltaStartCharacter,
  length: 1,
  tokenType,
  tokenModifiers: 0,
});

test("落としたトークンの差分を、次に残るトークンへ足して付け直す", () => {
  // 絶対の位置: (0,1)a (0,4)x (0,9)a (2,3)x (2,6)a (5,0)x
  const tokens = [tok(0, 1, 0), tok(0, 3, 9), tok(0, 5, 0), tok(2, 3, 9), tok(0, 3, 0), tok(3, 0, 9)];
  const kept = keepTokens(tokens, (t) => t !== 9);
  // 同じ行の真ん中を落とす: (0,9) は (0,1) から 8。行をまたぐ: (2,6) は行の頭から 6
  assert.deepEqual(kept, [tok(0, 1, 0), tok(0, 8, 0), tok(2, 6, 0)]);
  // 先頭を落とすと、最初に残るトークンの位置は文書の頭からになる
  assert.deepEqual(keepTokens([tok(1, 2, 9), tok(0, 3, 0)], (t) => t !== 9), [tok(1, 5, 0)]);
  assert.deepEqual(keepTokens(tokens, () => true), tokens);
});
