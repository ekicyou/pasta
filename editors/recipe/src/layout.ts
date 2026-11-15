// 純粋: 窓の中の並び（仕切り 3 本と出力・保存変数の左右）の数を決める（11.1〜11.6・11.8）。DOM・Tauri を import しない。
// 覚えた並びは作者の操作でだけ変わる。窓に収めるときは新しい値を返すだけで、覚えた値は書き換えない。

/** 覚える並び（layout.toml と 1 対 1。layout.ts に置き、ipc.ts は型だけを借りる） */
export interface LayoutPrefs {
  /** ツリーの幅（CSS px・整数） */
  treeWidth: number;
  /** 下の欄の面の高さ（帯を除く。CSS px・整数） */
  panelsHeight: number;
  /** 下の欄の左の欄の幅の割合（0〜1）。書く面（診断｜検査）と動かす面（出力｜保存変数）で共通 */
  leftShare: number;
  /** true で保存変数が左・出力が右 */
  swapped: boolean;
}

/** 大きさを変えられる範囲の今の大きさ（CSS px） */
export interface LayoutSpace {
  /** #workspace の幅（下の欄の幅と同じ） */
  width: number;
  /** #workspace の高さ＋下の欄の面の高さ（仕切りを動かしても変わらない和） */
  height: number;
}

export type SplitterId = "tree" | "panels" | "split";

export const DEFAULT_LAYOUT: LayoutPrefs = { treeWidth: 260, panelsHeight: 180, leftShare: 0.5, swapped: false };
export const LAYOUT_MIN = {
  tree: 120,
  editorWidth: 240,
  /** タブの列を含むエディタの側の高さ */
  editorHeight: 120,
  panels: 60,
  /** 下の欄の左右の欄のそれぞれの幅 */
  panel: 160,
} as const;
/** 矢印のキー 1 回の px */
export const KEY_STEP = 16;

// lo が hi より大きい（範囲が窓の最小より小さい）ときは lo を守り、エディタが残りを取る
const clamp = (v: number, lo: number, hi: number): number => Math.max(lo, Math.min(v, hi));
const size = (v: number): number => (Number.isFinite(v) && v > 0 ? v : 0);
const num = (v: number, fallback: number): number => (Number.isFinite(v) ? v : fallback);

const fitTree = (v: number, width: number): number => clamp(Math.round(v), LAYOUT_MIN.tree, width - LAYOUT_MIN.editorWidth);
const fitPanels = (v: number, height: number): number =>
  clamp(Math.round(v), LAYOUT_MIN.panels, height - LAYOUT_MIN.editorHeight);
function fitShare(v: number, width: number): number {
  if (width < 2 * LAYOUT_MIN.panel) return 0.5; // 2 つとも最小にはできないので半分ずつ
  const lo = LAYOUT_MIN.panel / width;
  return clamp(v, lo, 1 - lo);
}

/** 窓に収めた並び（最小を守る）。prefs は書き換えない。数でない値は既定に替える */
export function fitLayout(prefs: LayoutPrefs, space: LayoutSpace): LayoutPrefs {
  const width = size(space.width);
  const share = Number.isFinite(prefs.leftShare) && prefs.leftShare >= 0 && prefs.leftShare <= 1 ? prefs.leftShare : DEFAULT_LAYOUT.leftShare;
  return {
    treeWidth: fitTree(num(prefs.treeWidth, DEFAULT_LAYOUT.treeWidth), width),
    panelsHeight: fitPanels(num(prefs.panelsHeight, DEFAULT_LAYOUT.panelsHeight), size(space.height)),
    leftShare: fitShare(share, width),
    swapped: prefs.swapped === true,
  };
}

/** ドラッグ: 仕切りを offset へ動かした並び。tree＝範囲の左の端から、panels＝範囲の下の端から、split＝下の欄の左の端からの px */
export function dragLayout(prefs: LayoutPrefs, id: SplitterId, offset: number, space: LayoutSpace): LayoutPrefs {
  const fit = fitLayout(prefs, space);
  const width = size(space.width);
  // 動かした仕切りのキーだけを替え、ほかは覚えた値のまま返す（11.8）
  switch (id) {
    case "tree":
      return { ...prefs, treeWidth: fitTree(num(offset, fit.treeWidth), width) };
    case "panels":
      return { ...prefs, panelsHeight: fitPanels(num(offset, fit.panelsHeight), size(space.height)) };
    case "split":
      return { ...prefs, leftShare: width > 0 ? fitShare(num(offset / width, fit.leftShare), width) : fit.leftShare };
  }
}

/** 矢印: 見た目の向きへ KEY_STEP だけ動かした並び。-1＝左か上、1＝右か下 */
export function nudgeLayout(prefs: LayoutPrefs, id: SplitterId, direction: -1 | 1, space: LayoutSpace): LayoutPrefs {
  const fit = fitLayout(prefs, space);
  const step = direction * KEY_STEP;
  const offset = id === "tree" ? fit.treeWidth + step : id === "panels" ? fit.panelsHeight - step : fit.leftShare * size(space.width) + step;
  return dragLayout(prefs, id, offset, space);
}

/** 動かす面の左右を入れ替える（swapped だけを替える）。境は動かない */
export function swapLayout(prefs: LayoutPrefs): LayoutPrefs {
  return { ...prefs, swapped: !prefs.swapped };
}

/** 読み上げに渡す今の値と範囲（tree・panels は px、split は左の欄の %） */
export function separatorValue(prefs: LayoutPrefs, id: SplitterId, space: LayoutSpace): { now: number; min: number; max: number } {
  const fit = fitLayout(prefs, space);
  const width = size(space.width);
  switch (id) {
    case "tree":
      return { now: fit.treeWidth, min: LAYOUT_MIN.tree, max: Math.max(LAYOUT_MIN.tree, width - LAYOUT_MIN.editorWidth) };
    case "panels":
      return {
        now: fit.panelsHeight,
        min: LAYOUT_MIN.panels,
        max: Math.max(LAYOUT_MIN.panels, size(space.height) - LAYOUT_MIN.editorHeight),
      };
    case "split": {
      const lo = fitShare(0, width);
      return { now: Math.round(fit.leftShare * 100), min: Math.round(lo * 100), max: Math.round((1 - lo) * 100) };
    }
  }
}
