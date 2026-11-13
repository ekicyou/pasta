// 下の 2 つの一覧（診断 7.3・7.4・7.6／ゴースト全体の検査 9.6〜9.8・9.10）。行と件数は mapping.ts、ここは描いて押下を Documents へ渡すだけ。
// 名前・メッセージはディスクから来るので textContent だけで書く。描き直しは行の位置ごとにボタンを使い回し、焦点と読み上げを保つ。
import type { AnalysisResult, Report } from "./ipc.ts";
import { checkRows, diagnosticRows, type PanelRow } from "./mapping.ts";

/** Documents のうち一覧が使う所 */
export interface PanelsDocs {
  readonly tabs: readonly { rel: string; analysis: AnalysisResult | null; model: { getLanguageId(): string } }[];
  open(rel: string, pos?: { lineNumber: number; column: number }): Promise<unknown>;
}

interface Section {
  heading: HTMLElement;
  list: HTMLUListElement;
}

export class Panels {
  #docs: PanelsDocs;
  #diag: Section;
  #check: Section;
  #reason: HTMLParagraphElement;
  /** 最後の検査の結果。文字列は検査できない理由（9.10）。まだ検査していなければ null */
  #report: Report | string | null = null;

  constructor(el: HTMLElement, docs: PanelsDocs) {
    this.#docs = docs;
    this.#diag = this.#section(el, "panel-diagnostics");
    this.#check = this.#section(el, "panel-check");
    this.#reason = document.createElement("p");
    this.#reason.hidden = true;
    this.#check.list.before(this.#reason);
    this.update();
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
      return;
    }
    const { rows, errors, warnings } = checkRows(report, new Set(tabs.map((t) => t.rel)));
    setText(this.#check.heading, `ゴースト全体の検査（誤り ${errors} 件・警告 ${warnings} 件）`);
    this.#fill(this.#check.list, rows, (r) => void this.#docs.open(r.rel, r.pos));
  }

  #section(parent: HTMLElement, id: string): Section {
    const section = document.createElement("section");
    section.className = "panel";
    section.setAttribute("aria-labelledby", `${id}-heading`);
    const heading = document.createElement("h2");
    heading.id = `${id}-heading`;
    const list = document.createElement("ul");
    section.append(heading, list);
    parent.append(section);
    return { heading, list };
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

/** 同じ文なら触らない（読み上げを繰り返さない） */
function setText(el: HTMLElement, text: string): void {
  if (el.textContent !== text) el.textContent = text;
}
