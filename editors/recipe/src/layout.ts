// 窓の中の並び（仕切り 3 本と出力・保存変数の左右）。上は数を決める純粋な決まり（11.1〜11.6・11.8）、
// 下の mountLayout はそれを DOM に当てる取り付け（11.1〜11.5・11.7・11.8）。どちらも何も import しない（Panels は形で受ける）。
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

/** 同じ並びか（値が変わらなければ書かない） */
export function sameLayout(a: LayoutPrefs, b: LayoutPrefs): boolean {
  return a.treeWidth === b.treeWidth && a.panelsHeight === b.panelsHeight && a.leftShare === b.leftShare && a.swapped === b.swapped;
}

/** Panels のうち並びが使う所（panels.ts を読み込まず、形で受ける） */
export interface PanelsSplit {
  /** 2 つの面それぞれの、左右の欄の間の仕切り（role="separator"。出ている面のものだけが見える） */
  readonly splitters: readonly HTMLElement[];
  /** 下の欄（#panels。幅と左の端を測る） */
  readonly area: HTMLElement;
  /** 左の欄の幅の割合を 2 つの面に当て、動かす面の 2 つの欄の順を当てる */
  setSplit(leftShare: number, swapped: boolean): void;
  /** 入れ替えのボタンが押されたときに呼ばれる */
  onSwap: () => void;
}

export interface LayoutTargets {
  /** CSS の変数を当てる要素（document.documentElement） */
  root: HTMLElement;
  /** #workspace（範囲を測る・見張る） */
  workspace: HTMLElement;
  treeSplitter: HTMLElement;
  panelsSplitter: HTMLElement;
  panels: PanelsSplit;
  /** 操作の終わりに呼ぶ。失敗しても投げない */
  save(prefs: LayoutPrefs): void;
}

/** 仕切り 3 本の操作を受け、並びを当て、操作の終わりに覚えさせる。呼んだ直後に initial を窓に収めた並びが当たっている */
export function mountLayout(t: LayoutTargets, initial: LayoutPrefs): void {
  /** 覚えた並び（作者の操作でだけ変わる） */
  let prefs = initial;
  /** 最後に書いた（か読んだ）並び */
  let saved = initial;
  /** 今当てている下の欄の面の高さ（CSS の既定と同じ値から始める） */
  let face = DEFAULT_LAYOUT.panelsHeight;
  const parts: [HTMLElement, SplitterId][] = [
    [t.treeSplitter, "tree"],
    [t.panelsSplitter, "panels"],
    ...t.panels.splitters.map((el): [HTMLElement, SplitterId] => [el, "split"]),
  ];
  // #workspace の高さ＋面の高さ（仕切りを動かしても変わらない和）
  const space = (): LayoutSpace => {
    const r = t.workspace.getBoundingClientRect();
    return { width: r.width, height: r.height + face };
  };
  const apply = () => {
    const s = space();
    const fit = fitLayout(prefs, s);
    t.root.style.setProperty("--tree-width", `${fit.treeWidth}px`);
    t.root.style.setProperty("--panels-height", `${fit.panelsHeight}px`);
    face = fit.panelsHeight;
    t.panels.setSplit(fit.leftShare, fit.swapped);
    for (const [el, id] of parts) {
      const v = separatorValue(prefs, id, s);
      el.setAttribute("aria-valuenow", String(v.now));
      el.setAttribute("aria-valuemin", String(v.min));
      el.setAttribute("aria-valuemax", String(v.max));
    }
  };
  const commit = () => {
    if (sameLayout(prefs, saved)) return;
    saved = prefs;
    t.save(prefs);
  };
  /** ポインタの位置を仕切りの offset に直す（tree＝範囲の左から、panels＝面の高さ、split＝下の欄の左から） */
  const offset = (id: SplitterId, e: PointerEvent): number => {
    if (id === "tree") return e.clientX - t.workspace.getBoundingClientRect().left;
    const r = t.panels.area.getBoundingClientRect();
    if (id === "split") return e.clientX - r.left;
    // 下の欄の下の端からの距離から、帯と枠線（下の欄の高さ − 面の高さ）を引く
    return r.bottom - e.clientY - (r.height - face);
  };

  for (const [el, id] of parts) {
    el.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      e.preventDefault(); // 文字の選択を始めない
      el.focus();
      el.setPointerCapture(e.pointerId);
      el.classList.add("dragging");
      t.root.style.userSelect = "none";
    });
    el.addEventListener("pointermove", (e) => {
      if (!el.hasPointerCapture(e.pointerId)) return;
      prefs = dragLayout(prefs, id, offset(id, e), space());
      apply();
    });
    // 離した・捕まえを失った（どちらも操作の終わり）
    el.addEventListener("lostpointercapture", () => {
      el.classList.remove("dragging");
      t.root.style.userSelect = "";
      commit();
    });
    const vertical = id !== "panels";
    const direction = (key: string): -1 | 1 | 0 =>
      key === (vertical ? "ArrowLeft" : "ArrowUp") ? -1 : key === (vertical ? "ArrowRight" : "ArrowDown") ? 1 : 0;
    el.addEventListener("keydown", (e) => {
      const d = direction(e.key);
      if (d === 0 || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
      e.preventDefault();
      prefs = nudgeLayout(prefs, id, d, space());
      apply();
    });
    el.addEventListener("keyup", (e) => {
      if (direction(e.key) !== 0) commit();
    });
  }
  t.panels.onSwap = () => {
    prefs = swapLayout(prefs);
    apply();
    commit();
  };
  // 窓の大きさが変わった・開始の画面から部品が出たとき。覚えた値は変えず、書かない（11.8）
  new ResizeObserver(() => apply()).observe(t.workspace);
  apply();
}
