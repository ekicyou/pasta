import { test } from "node:test";
import assert from "node:assert/strict";
import { packageRoot } from "./js-licenses.mjs";

test("ふつうのパッケージの根", () => {
  assert.equal(
    packageRoot("C:/a/node_modules/monaco-editor/esm/vs/editor/editor.main.js"),
    "C:/a/node_modules/monaco-editor",
  );
});

test("@scope/name の根", () => {
  assert.equal(
    packageRoot("/a/node_modules/@shikijs/langs/dist/rust.mjs?worker"),
    "/a/node_modules/@shikijs/langs",
  );
});

test("入れ子の node_modules は内側の根", () => {
  assert.equal(
    packageRoot("C:\\a\\node_modules\\x\\node_modules\\@s\\y\\index.js"),
    "C:/a/node_modules/x/node_modules/@s/y",
  );
});

test("仮想の id と自前のファイルは根を持たない", () => {
  assert.equal(packageRoot("\0vite/preload-helper.js"), null);
  assert.equal(packageRoot("\0/a/node_modules/x/index.js?commonjs-proxy"), null);
  assert.equal(packageRoot("C:/a/editors/recipe/src/main.ts"), null);
});
