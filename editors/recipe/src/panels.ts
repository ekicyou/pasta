// 下の欄。帯の 2 つのボタンで、書く面（診断｜検査）と動かす面（出力｜保存変数）を切り替える（4.2・11.1）。
// 書く面の 2 つの一覧（診断 7.3・7.4・7.6／ゴースト全体の検査 9.6〜9.8・9.10）。行と件数は mapping.ts、ここは描いて押下を Documents へ渡すだけ。
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
  /** 動かす面の 2 つの `.panel`。中身は devPanels.ts が満たす */
  readonly output: PanelSection;
  readonly variables: PanelSection;
  /** 最後の検査の結果。文字列は検査できない理由（9.10）。まだ検査していなければ null */
  #report: Report | string | null = null;

  constructor(el: HTMLElement, docs: PanelsDocs) {
    this.#docs = docs;
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

    const write = this.#faces.write.body;
    this.#diag = withList(this.#section(write, "panel-diagnostics"));
    this.#check = withList(this.#section(write, "panel-check"));
    this.#reason = document.createElement("p");
    this.#reason.hidden = true;
    this.#check.list.before(this.#reason);
    this.output = this.#section(this.#faces.run.body, "panel-output", "出力");
    this.variables = this.#section(this.#faces.run.body, "panel-variables", "保存変数");
    this.show("write");
    this.update();
  }

  /** 面を切り替える。焦点は動かさない（4.2） */
  show(face: Face): void {
    for (const f of ["write", "run"] as const) {
      this.#faces[f].button.setAttribute("aria-pressed", String(f === face));
      this.#faces[f].body.hidden = f !== face;
    }
  }

  /** 検査の結果か、検査できない理由を出す。理由のときは古い結果を残さない（9.10） */
  setReport(report: Report | string): void {
    this.#report = report;
    this.update();
  }

  /** タブの解析・開け閉めの後に呼ぶ（6.3 の onAnalyzed・onClosed）。閉じたタブの診断はここで消える（7.6） */
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

function withList(s: PanelSection): Section {
  const list = document.createElement("ul");
  s.section.append(list);
  return { ...s, list };
}

/** 同じ文なら触らない（読み上げを繰り返さない） */
function setText(el: HTMLElement, text: string): void {
  if (el.textContent !== text) el.textContent = text;
}
