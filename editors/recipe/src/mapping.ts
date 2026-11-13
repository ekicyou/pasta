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
