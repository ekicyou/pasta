// 純粋: 解析と検査の結果を、エディタ・ツリー・一覧の形へ写す（DOM・Monaco を import しない）
import type { Diagnostic, Finding, Report, SceneNode, SemanticToken, TreeListing } from "./ipc.ts";

/** Monaco の MarkerSeverity.Error の値 */
export const MARKER_SEVERITY_ERROR = 8;

/** Monaco の IMarkerData と同じ欄（1 始まり） */
export interface MarkerData {
  severity: number;
  message: string;
  startLineNumber: number;
  startColumn: number;
  endLineNumber: number;
  endColumn: number;
}

export interface TreeItem {
  kind: "scene";
  rel: string;
  name: string;
  sceneKind: SceneNode["kind"];
  /** 見出しの行（1 始まり） */
  line: number;
  children: TreeItem[];
}

export interface TreeNode {
  kind: "dir" | "file";
  /** 最上位（master そのもの）は空 */
  name: string;
  /** master からの相対・'/' 区切り。最上位は空 */
  rel: string;
  children: TreeNode[];
}

/** 5 つずつ並べるだけ（6.2） */
export function tokensToData(tokens: readonly SemanticToken[]): Uint32Array {
  const data = new Uint32Array(tokens.length * 5);
  tokens.forEach((t, i) =>
    data.set([t.deltaLine, t.deltaStartCharacter, t.length, t.tokenType, t.tokenModifiers], i * 5),
  );
  return data;
}

/** 行・列（どちらも UTF-16 の 0 始まり）を +1 するだけ。拡張と同じ位置になる（7.1・7.5） */
export function diagnosticsToMarkers(d: readonly Diagnostic[]): MarkerData[] {
  return d.map(({ range: { start, end }, message }) => ({
    severity: MARKER_SEVERITY_ERROR,
    message,
    startLineNumber: start.line + 1,
    startColumn: start.character + 1,
    endLineNumber: end.line + 1,
    endColumn: end.character + 1,
  }));
}

/** 解析器の返す並びをそのまま 2 階層に写す（4.4・4.5・4.11〜4.13） */
export function scenesToItems(rel: string, scenes: readonly SceneNode[]): TreeItem[] {
  const item = (s: SceneNode, children: TreeItem[]): TreeItem => ({
    kind: "scene",
    rel,
    name: s.name,
    sceneKind: s.kind,
    line: s.heading.start.line + 1,
    children,
  });
  return scenes.map((g) => item(g, g.children.map((l) => item(l, []))));
}

// 小文字にした UTF-16 の並び（ロケールに依らない）。同じなら元の文字で決める
const byName = (a: TreeNode, b: TreeNode): number => {
  const x = a.name.toLowerCase();
  const y = b.name.toLowerCase();
  return x < y ? -1 : x > y ? 1 : a.name < b.name ? -1 : a.name > b.name ? 1 : 0;
};

/** フォルダが先・それぞれ大文字・小文字を区別しない名前の順（4.1・4.2） */
export function buildTree(listing: TreeListing): TreeNode {
  const root: TreeNode = { kind: "dir", name: "", rel: "", children: [] };
  const dirs = new Map<string, TreeNode>([["", root]]);
  const dir = (rel: string): TreeNode => {
    let node = dirs.get(rel);
    if (!node) {
      const cut = rel.lastIndexOf("/");
      node = { kind: "dir", name: rel.slice(cut + 1), rel, children: [] };
      dirs.set(rel, node);
      dir(cut < 0 ? "" : rel.slice(0, cut)).children.push(node);
    }
    return node;
  };
  listing.dirs.forEach(dir);
  for (const rel of listing.files) {
    const cut = rel.lastIndexOf("/");
    dir(cut < 0 ? "" : rel.slice(0, cut)).children.push({ kind: "file", name: rel.slice(cut + 1), rel, children: [] });
  }
  for (const node of dirs.values()) {
    node.children.sort((a, b) => (a.kind === b.kind ? byName(a, b) : a.kind === "dir" ? -1 : 1));
  }
  return root;
}

/**
 * 開いているファイルの code 'syntax' を除く（9.8・パスは大文字・小文字を区別しない）。
 * 同じファイルかの判定なので Rust の to_uppercase と同じく大文字に揃える（ı と I・ſ と S も同じに。小文字では揃わない）
 */
export function visibleFindings(report: Report, openRels: ReadonlySet<string>): Finding[] {
  const open = new Set([...openRels].map((r) => r.toUpperCase()));
  return report.findings.filter((f) => !(f.code === "syntax" && open.has(f.path.toUpperCase())));
}

/** 行・列は既に 1 始まり。行の無い項目はファイルの先頭（9.7） */
export function findingPosition(f: Finding): { lineNumber: number; column: number } {
  return { lineNumber: f.line ?? 1, column: f.line === null ? 1 : (f.column ?? 1) };
}

/** 一覧の 1 行。label はそのまま textContent に書く（名前・メッセージはディスクから来る） */
export interface PanelRow {
  label: string;
  rel: string;
  pos: { lineNumber: number; column: number };
}

/** 診断の一覧の行。行・列は diagnosticsToMarkers と同じ 1 始まり（7.3・7.5） */
export function diagnosticRows(rel: string, d: readonly Diagnostic[]): PanelRow[] {
  return diagnosticsToMarkers(d).map((m) => ({
    label: `${rel}:${m.startLineNumber}:${m.startColumn} ${m.message}`,
    rel,
    pos: { lineNumber: m.startLineNumber, column: m.startColumn },
  }));
}

/** 検査の一覧の行と件数。件数は一覧に見える項目だけを数える（9.6・9.8）。色だけに頼らず「誤り」「警告」を文字で出す */
export function checkRows(report: Report, openRels: ReadonlySet<string>): { rows: PanelRow[]; errors: number; warnings: number } {
  const shown = visibleFindings(report, openRels);
  const rows = shown.map((f) => {
    const at = f.line === null ? "" : `:${f.line}` + (f.column === null ? "" : `:${f.column}`);
    return { label: `${f.severity === "error" ? "誤り" : "警告"} ${f.path}${at} ${f.message}`, rel: f.path, pos: findingPosition(f) };
  });
  const errors = shown.filter((f) => f.severity === "error").length;
  return { rows, errors, warnings: shown.length - errors };
}

/** 下の欄の帯の、書く面のボタンの名前。検査がまだ・できないときは診断の件数だけ（4.2・11.4） */
export function writeFaceLabel(diagnostics: number, check: { errors: number; warnings: number } | null): string {
  const tail = check ? `・誤り ${check.errors} 件・警告 ${check.warnings} 件` : "";
  return `診断・検査（${diagnostics} 件${tail}）`;
}

/** タブの言語は拡張子で決める。.pasta・.lua 以外は色なし（5.2） */
export function languageOf(rel: string): "pasta" | "lua" | "plaintext" {
  const ext = rel.slice(rel.lastIndexOf("/") + 1).match(/\.([^.]+)$/)?.[1].toLowerCase();
  return ext === "pasta" || ext === "lua" ? ext : "plaintext";
}

/** シーンの道: グローバルなら 1 つ、ローカルなら 2 つ。n は同じ名前の中の順（ツリーの行のキーと同じ数え方・1.4） */
export type ScenePath = readonly { readonly name: string; readonly n: number }[];

/** 見出しの行（1 始まり）と道の組を文書の順に平らに並べる（1.4） */
export function scenePaths(scenes: readonly SceneNode[]): { line: number; path: ScenePath }[] {
  const out: { line: number; path: ScenePath }[] = [];
  const walk = (list: readonly SceneNode[], parent: ScenePath): void => {
    const seen = new Map<string, number>();
    for (const s of list) {
      const n = seen.get(s.name) ?? 0;
      seen.set(s.name, n + 1);
      const path = [...parent, { name: s.name, n }];
      out.push({ line: s.heading.start.line + 1, path });
      walk(s.children, path);
    }
  };
  walk(scenes, []);
  return out;
}

/** 保存した中身の解析から道の今の行（1 始まり）を引く。無ければ undefined（1.4） */
export function sceneLine(scenes: readonly SceneNode[], path: ScenePath): number | undefined {
  return scenePaths(scenes).find(
    (p) => p.path.length === path.length && p.path.every((s, i) => s.name === path[i].name && s.n === path[i].n),
  )?.line;
}

/** ツリーの見える 1 行（aria-level で階層を表す平らな並び。キーで差分を描き、上下のキーはこの順に動く） */
export interface TreeRow {
  /** フォルダ・ファイルは rel（最上位は空）、シーンは親のキー＋`|名前|同じ名前の中の順`（`|` はファイル名に使えない。前にシーンを足しても開いた所がずれない） */
  key: string;
  /** 1 始まり */
  level: number;
  kind: "dir" | "file" | "scene";
  name: string;
  /** 開くファイル。シーンもツリーの今の rel（項目が覚えた rel は名前の変更で古くなる） */
  rel: string;
  /** シーンの見出しの行（1 始まり）。それ以外は null */
  line: number | null;
  /** シーンの道。それ以外は null */
  scene: ScenePath | null;
  parent: string | null;
  /** 子が無ければ null */
  expanded: boolean | null;
}

/** scenes は rel を大文字にしたキー（Rust の to_uppercase と同じ揃え）。expanded は開いた行のキー */
export function treeRows(root: TreeNode, scenes: ReadonlyMap<string, readonly TreeItem[]>, expanded: ReadonlySet<string>): TreeRow[] {
  const rows: TreeRow[] = [];
  const add = (row: Omit<TreeRow, "expanded">, hasChildren: boolean): boolean => {
    const open = hasChildren && expanded.has(row.key);
    rows.push({ ...row, expanded: hasChildren ? open : null });
    return open;
  };
  const sceneRows = (rel: string, items: readonly TreeItem[], level: number, parent: string, path: ScenePath) => {
    const seen = new Map<string, number>();
    for (const item of items) {
      const n = seen.get(item.name) ?? 0;
      seen.set(item.name, n + 1);
      const key = `${parent}|${item.name}|${n}`;
      const scene = [...path, { name: item.name, n }];
      if (add({ key, level, kind: "scene", name: item.name, rel, line: item.line, scene, parent }, item.children.length > 0)) {
        sceneRows(rel, item.children, level + 1, key, scene);
      }
    }
  };
  const walk = (node: TreeNode, level: number, parent: string | null): void => {
    const items = node.kind === "file" ? (scenes.get(node.rel.toUpperCase()) ?? []) : [];
    const kids = node.kind === "dir" ? node.children.length : items.length;
    if (!add({ key: node.rel, level, kind: node.kind, name: node.name, rel: node.rel, line: null, scene: null, parent }, kids > 0)) return;
    if (node.kind === "dir") node.children.forEach((c) => walk(c, level + 1, node.rel));
    else sceneRows(node.rel, items, level + 1, node.rel, []);
  };
  walk(root, 1, null);
  return rows;
}
