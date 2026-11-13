// Vite の plugin: 束ねに入ったパッケージの name・version・license と根の直下の許諾文を
// build/js-licenses.txt へ書く（dist/ には入れない。exe に余計なものを埋め込まない）。
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const OUT = fileURLToPath(new URL("../build/js-licenses.txt", import.meta.url));
const TEXT_FILE = /^(LICEN[CS]E|NOTICE|ThirdPartyNotices)/i;

// worker の束ね（別のビルド）で集めた根も、本体のビルドの最後にまとめて書く
const roots = new Set();
// パッケージの中に写し込まれた部品（monaco の中の DOMPurify など）の注記。縮めると消えるので元のファイルから拾う
const banners = new Set();
const BANNER = /\/\*![^]*?\*\//g;

/** モジュールの id → node_modules の中のパッケージの根（無ければ null） */
export function packageRoot(id) {
  if (id.startsWith("\0")) return null;
  const p = id.split("?")[0].replaceAll("\\", "/");
  const i = p.lastIndexOf("/node_modules/");
  if (i < 0) return null;
  const head = i + "/node_modules/".length;
  const parts = p.slice(head).split("/");
  const n = parts[0].startsWith("@") ? 2 : 1;
  return p.slice(0, head) + parts.slice(0, n).join("/");
}

function readPackage(root) {
  const pkg = JSON.parse(readFileSync(`${root}/package.json`, "utf8"));
  const texts = readdirSync(root)
    .filter((f) => TEXT_FILE.test(f))
    .sort()
    .map((f) => `--- ${f} ---\n${readFileSync(`${root}/${f}`, "utf8").trimEnd()}`);
  return { name: pkg.name, version: pkg.version, license: typeof pkg.license === "string" ? pkg.license : "", texts };
}

/** write: false は worker の束ねで集めるだけ */
export default function jsLicenses({ write = true } = {}) {
  return {
    name: "js-licenses",
    apply: "build",
    generateBundle(_options, bundle) {
      for (const out of Object.values(bundle)) {
        if (out.type !== "chunk") continue;
        for (const id of out.moduleIds) {
          const root = packageRoot(id);
          if (!root) continue;
          roots.add(root);
          for (const b of readFileSync(id.split("?")[0], "utf8").match(BANNER) ?? []) {
            if (b.includes("@license")) banners.add(b);
          }
        }
      }
      if (!write) return;
      const pkgs = new Map();
      for (const root of roots) {
        const pkg = readPackage(root);
        if (!pkg.license && pkg.texts.length === 0) {
          this.error(`${root} に license も許諾文も無い。第三者の表示を作れないので止める`);
        }
        pkgs.set(`${pkg.name}@${pkg.version}`, pkg);
      }
      const sorted = [...pkgs.values()].sort((a, b) =>
        a.name < b.name ? -1 : a.name > b.name ? 1 : a.version < b.version ? -1 : 1,
      );
      const body = sorted
        .map((p) => [`${"=".repeat(80)}\n${p.name} ${p.version}\nLicense: ${p.license || "(本文を参照)"}`, ...p.texts].join("\n\n"))
        .join("\n\n");
      const vendored = `${"=".repeat(80)}\nパッケージの中に写し込まれた部品の注記\n\n${[...banners].sort().join("\n")}`;
      mkdirSync(fileURLToPath(new URL("../build/", import.meta.url)), { recursive: true });
      writeFileSync(OUT, `${body}\n\n${vendored}\n`);
    },
  };
}
