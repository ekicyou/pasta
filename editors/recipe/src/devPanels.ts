// 動かす面の 2 つの欄（出力 4.2〜4.6／保存変数の表 6.2・6.3・6.6・6.7・7.11）。panels.ts の `.panel` を満たすだけで、押したことは DevActions へ渡す。
// 台本・ログ・名前・値はディスクやゴーストから来るので textContent だけで書く（4.5）。
import type { VarRow } from "./ipc.ts";
import type { PanelSection } from "./panels.ts";

/** 誤り・警告の行。色だけに頼らず頭の文字で見分ける（4.3） */
export const noticeLine = (level: "error" | "warning", text: string): string => `${level === "error" ? "誤り" : "警告"}: ${text}`;

/** 進んでいる間は操作の名前を出す（4.6） */
export const outputHeading = (running: string | null): string => (running === null ? "出力" : `出力（実行中: ${running}）`);

const KIND_LABELS: Record<VarRow["kind"], string> = { string: "文字列", number: "数", boolean: "真偽", table: "表" };
export const kindLabel = (kind: VarRow["kind"]): string => KIND_LABELS[kind];

/** 操作のボタンの読み上げの名前（11.4）。表の行は操作を出さない（6.3） */
export function rowActionLabels(row: VarRow): { edit: string; del: string } | null {
  return row.kind === "table" ? null : { edit: `＄＊${row.name} を書き換える`, del: `＄＊${row.name} を消す` };
}

export class OutputPanel {
  #slot: PanelSection;
  #log: HTMLDivElement;

  constructor(slot: PanelSection) {
    this.#slot = slot;
    this.#log = document.createElement("div");
    this.#log.className = "output-log";
    this.#log.setAttribute("role", "log");
    this.#log.setAttribute("aria-labelledby", slot.heading.id);
    slot.section.append(this.#log);
  }

  /** 後ろへ足し、前の行は残す。足したら一番下へ送る（4.4） */
  append(lines: readonly string[], className?: string): void {
    for (const line of lines) {
      const div = document.createElement("div");
      div.textContent = line;
      if (className) div.className = className;
      this.#log.append(div);
    }
    this.#slot.section.scrollTop = this.#slot.section.scrollHeight;
  }

  notify(level: "error" | "warning", text: string): void {
    this.append([noticeLine(level, text)], `output-notice output-${level}`);
  }

  /** 操作は DevActions の印で 1 つずつしか走らないので、見出しは 1 つの名前だけ持つ */
  async progress<T>(title: string, work: () => Promise<T>): Promise<T> {
    this.#slot.heading.textContent = outputHeading(title);
    try {
      return await work();
    } finally {
      this.#slot.heading.textContent = outputHeading(null);
    }
  }
}

export interface VariablesCallbacks {
  onRefresh(): void;
  onEdit(row: VarRow): void;
  onDelete(row: VarRow): void;
  onAdd(): void;
}

/** none＝まだ読んでいない・rows＝読めた・unreadable＝読めなかった */
export type VariablesState = { kind: "none" } | { kind: "rows"; rows: readonly VarRow[] } | { kind: "unreadable" };

export class VariablesPanel {
  #cb: VariablesCallbacks;
  #refresh: HTMLButtonElement;
  #body: HTMLDivElement;
  #state: VariablesState = { kind: "none" };

  constructor(slot: PanelSection, callbacks: VariablesCallbacks) {
    this.#cb = callbacks;
    this.#refresh = button("保存変数を表示", () => this.#cb.onRefresh());
    const note = document.createElement("p");
    note.className = "variables-note";
    note.textContent =
      "書き換えはランタイムのメモリの上で効き、ファイル（profile/pasta/save/ の保存先）へはランタイムの終了時（ゴーストの終了・SHIORI の読み込み直し）に書かれます。";
    const bar = document.createElement("p");
    bar.append(this.#refresh);
    this.#body = document.createElement("div");
    slot.section.append(bar, note, this.#body);
  }

  get state(): VariablesState {
    return this.#state;
  }

  setRows(rows: readonly VarRow[]): void {
    this.#render({ kind: "rows", rows });
  }

  setUnreadable(): void {
    this.#render({ kind: "unreadable" });
  }

  /** ゴーストの切り替えで前の表を消す（5.3） */
  clear(): void {
    this.#render({ kind: "none" });
  }

  #render(state: VariablesState): void {
    this.#state = state;
    // 書き換えた後の描き直しで焦点を body へ落とさない
    const focused = this.#body.contains(document.activeElement);
    this.#body.replaceChildren();
    // 読めていない間は表も操作も出さない（7.11）
    if (state.kind === "rows") {
      if (state.rows.length === 0) {
        const p = document.createElement("p");
        p.textContent = "保存変数はありません";
        this.#body.append(p);
      } else {
        this.#body.append(this.#table(state.rows));
      }
      const add = document.createElement("p");
      add.append(button("追加…", () => this.#cb.onAdd()));
      this.#body.append(add);
    }
    if (focused) this.#refresh.focus();
  }

  #table(rows: readonly VarRow[]): HTMLTableElement {
    const table = document.createElement("table");
    table.className = "variables-table";
    const head = table.createTHead().insertRow();
    for (const h of ["名前", "型", "値", "操作"]) {
      const th = document.createElement("th");
      th.scope = "col";
      th.textContent = h;
      head.append(th);
    }
    const body = table.createTBody();
    for (const row of rows) {
      const tr = body.insertRow();
      for (const text of [row.name, kindLabel(row.kind), row.value]) tr.insertCell().textContent = text;
      const ops = tr.insertCell();
      const labels = rowActionLabels(row);
      if (labels) {
        const edit = button("書き換え…", () => this.#cb.onEdit(row));
        edit.setAttribute("aria-label", labels.edit);
        const del = button("消す…", () => this.#cb.onDelete(row));
        del.setAttribute("aria-label", labels.del);
        ops.append(edit, del);
      }
    }
    return table;
  }
}

function button(label: string, onclick: () => void): HTMLButtonElement {
  const b = document.createElement("button");
  b.textContent = label;
  b.onclick = onclick;
  return b;
}
