// 下の欄。帯の 2 つのボタンで、書く面（診断｜検査）と動かす面（出力｜保存変数）を切り替える。
// 書く面の 2 つの一覧（診断／ゴースト全体の検査）。行と件数は mapping.ts、ここは描いて押下を Documents へ渡すだけ。
// 名前・メッセージはディスクから来るので textContent だけで書く。描き直しは行の位置ごとにボタンを使い回し、焦点と読み上げを保つ。
import type { AnalysisResult, Report } from "./ipc.ts";
import { checkRows, diagnosticRows, writeFaceLabel, type PanelRow } from "./mapping.ts";

/** Documents のうち一覧が使う所 */
export interface PanelsDocs {
  readonly tabs: readonly { rel: string; analysis: AnalysisResult | null; model: { getLanguageId(): string } }[];
  open(rel: string, pos?: { lineNumber: number; column: number }): Promise<unknown>;
}

/** 見出しの h2 を持つ `.panel` の 1 つ */
export interface PanelSection {
  section: HTMLElement;
  heading: HTMLElement;
}

interface Section extends PanelSection {
  list: HTMLUListElement;
}

/** 下の欄の面: 書く面＝診断｜検査、動かす面＝出力｜保存変数 */
export type Face = "write" | "run";

export class Panels {
  #docs: PanelsDocs;
  #diag: Section;
  #check: Section;
  #reason: HTMLParagraphElement;
  #faces: Record<Face, { button: HTMLButtonElement; body: HTMLElement }>;
  #swap: HTMLButtonElement;
  /** 動かす面の 2 つの `.panel`。中身は devPanels.ts が満たす */
  readonly output: PanelSection;
  readonly variables: PanelSection;
  /** 最後の検査の結果。文字列は検査できない理由。まだ検査していなければ null */
  #report: Report | string | null = null;
  /** 2 つの面の左右の欄の間の仕切り（書く面・動かす面の順）。ドラッグとキーは layoutMount が受ける */
  readonly splitters: readonly HTMLElement[];
  /** 下の欄（#panels） */
  readonly area: HTMLElement;
  /** 入れ替えのボタンが押されたとき。layoutMount が差し替える */
  onSwap: () => void = () => {};

  constructor(el: HTMLElement, docs: PanelsDocs) {
    this.#docs = docs;
    this.area = el;
    const bar = document.createElement("div");
    bar.className = "panels-bar";
    el.append(bar);
    const face = (label: string): { button: HTMLButtonElement; body: HTMLElement } => {
      const button = document.createElement("button");
      button.textContent = label;
      const body = document.createElement("div");
      body.className = "panels-face";
      bar.append(button);
      el.append(body);
      return { button, body };
    };
    this.#faces = { write: face(""), run: face("出力・保存変数") };
    for (const f of ["write", "run"] as const) this.#faces[f].button.onclick = () => this.show(f);
    // 面のタブのすぐ右（帯の右の端は画面の右下のゴーストに隠れる）。作り直さないので押した後も焦点が残る
    this.#swap = document.createElement("button");
    this.#swap.textContent = "左右を入れ替える";
    this.#swap.setAttribute("aria-label", "出力と保存変数の左右を入れ替える");
    this.#swap.onclick = () => this.onSwap();
    bar.append(this.#swap);

    const write = this.#faces.write.body;
    this.#diag = withList(this.#section(write, "panel-diagnostics"));
    this.#check = withList(this.#section(write, "panel-check"));
    this.#reason = document.createElement("p");
    this.#reason.hidden = true;
    this.#check.list.before(this.#reason);
    this.output = this.#section(this.#faces.run.body, "panel-output", "出力");
    this.variables = this.#section(this.#faces.run.body, "panel-variables", "保存変数");
    this.splitters = [splitter(this.#diag.section, "診断と検査の境"), splitter(this.output.section, "出力と保存変数の境")];
    this.show("write");
    this.update();
  }

  /** 面を切り替える。焦点は動かさない */
  show(face: Face): void {
    for (const f of ["write", "run"] as const) {
      const { button, body } = this.#faces[f];
      button.setAttribute("aria-pressed", String(f === face));
      // 隠す面の中の焦点は body へ落ちるので、その面へ戻るボタンへ置く（エディタの焦点は触らない）
      const lost = f !== face && !body.hidden && body.contains(document.activeElement);
      body.hidden = f !== face;
      if (lost) button.focus();
    }
    this.#swap.hidden = face !== "run";
  }

  /** 左の欄の幅の割合を 2 つの面に当て、動かす面の 2 つの欄の順を見た目の順にする */
  setSplit(leftShare: number, swapped: boolean): void {
    const run = this.#faces.run.body;
    const [left, right] = swapped ? [this.variables.section, this.output.section] : [this.output.section, this.variables.section];
    // DOM の順＝Tab と読み上げの順。変わらないときは触らない。動かすとスクロールが戻るので保つ
    if (run.firstElementChild !== left) {
      const tops = [left.scrollTop, right.scrollTop];
      run.prepend(left);
      run.append(right);
      [left.scrollTop, right.scrollTop] = tops;
    }
    // 境は位置で決まる（入れ替えても境は動かず、中身だけが入れ替わる）
    for (const [l, r] of [[this.#diag.section, this.#check.section], [left, right]]) {
      l.style.flexGrow = String(leftShare);
      r.style.flexGrow = String(1 - leftShare);
    }
  }

  /** 検査の結果か、検査できない理由を出す。理由のときは古い結果を残さない */
  setReport(report: Report | string): void {
    this.#report = report;
    this.update();
  }

  /** タブの解析・開け閉めの後に呼ぶ（onAnalyzed・onClosed）。閉じたタブの診断はここで消える */
  update(): void {
    const tabs = this.#docs.tabs;
    // 言語が pasta でなくなったタブ（名前の変更）は波線と同じく出さない
    const diag = tabs.flatMap((t) =>
      t.analysis && t.model.getLanguageId() === "pasta" ? diagnosticRows(t.rel, t.analysis.diagnostics).map((r) => ({ ...r, tab: t })) : [],
    );
    setText(this.#diag.heading, `診断（${diag.length} 件）`);
    // 押した時の rel で開く（名前の変更で古くならない）
    this.#fill(this.#diag.list, diag, (r) => void this.#docs.open(r.tab.rel, r.pos));

    const report = this.#report;
    const reason = typeof report === "string" ? report : "";
    setText(this.#reason, reason ? `検査できません: ${reason}` : "");
    this.#reason.hidden = !reason;
    if (typeof report === "string" || report === null) {
      setText(this.#check.heading, "ゴースト全体の検査");
      this.#fill(this.#check.list, [], () => {});
      setText(this.#faces.write.button, writeFaceLabel(diag.length, null));
      return;
    }
    const { rows, errors, warnings } = checkRows(report, new Set(tabs.map((t) => t.rel)));
    setText(this.#check.heading, `ゴースト全体の検査（誤り ${errors} 件・警告 ${warnings} 件）`);
    this.#fill(this.#check.list, rows, (r) => void this.#docs.open(r.rel, r.pos));
    // 動かす面を見ている間も件数は帯に出る（書き間違いに気づける）
    setText(this.#faces.write.button, writeFaceLabel(diag.length, { errors, warnings }));
  }

  #section(parent: HTMLElement, id: string, title = ""): PanelSection {
    const section = document.createElement("section");
    section.className = "panel";
    section.setAttribute("aria-labelledby", `${id}-heading`);
    const heading = document.createElement("h2");
    heading.id = `${id}-heading`;
    heading.textContent = title;
    section.append(heading);
    parent.append(section);
    return { section, heading };
  }

  /** 行の数だけボタンを使い回す。焦点の行が消えたら残る最後の行へ焦点を移す */
  #fill<R extends PanelRow>(list: HTMLUListElement, rows: R[], open: (r: R) => void): void {
    const focused = list.contains(document.activeElement);
    while (list.children.length > rows.length) list.lastElementChild!.remove();
    rows.forEach((r, i) => {
      let button = list.children[i]?.firstElementChild as HTMLButtonElement | undefined;
      if (!button) {
        const li = document.createElement("li");
        button = document.createElement("button");
        li.append(button);
        list.append(li);
      }
      setText(button, r.label);
      button.onclick = () => open(r);
    });
    if (focused && !list.contains(document.activeElement)) (list.lastElementChild?.firstElementChild as HTMLElement | null)?.focus();
  }
}

/** 面の左の欄のすぐ後に仕切りを置く（左の欄・仕切り・右の欄） */
function splitter(left: HTMLElement, label: string): HTMLElement {
  const el = document.createElement("div");
  el.className = "splitter vertical";
  el.setAttribute("role", "separator");
  el.tabIndex = 0;
  el.setAttribute("aria-orientation", "vertical");
  el.setAttribute("aria-label", label);
  left.after(el);
  return el;
}

function withList(s: PanelSection): Section {
  const list = document.createElement("ul");
  s.section.append(list);
  return { ...s, list };
}

/** 同じ文なら触らない（読み上げを繰り返さない） */
function setText(el: HTMLElement, text: string): void {
  if (el.textContent !== text) el.textContent = text;
}
