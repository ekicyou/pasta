// HTML の小さな窓（3 択の確かめ・名前の入力・知らせ・選ぶ・確かめ）。<dialog> の showModal が焦点を窓の中に閉じ、Esc で取り消す。
// 文言はディスクから来るファイル名や理由を含むので textContent だけで書く（innerHTML は使わない）。

export type Choice = "save" | "discard" | "cancel";

/** 閉じた理由（returnValue）を 3 択へ。Esc・それ以外は取り消し */
export function choiceOf(returnValue: string): Choice {
  return returnValue === "save" || returnValue === "discard" ? returnValue : "cancel";
}

/** 閉じた理由を名前の入力の結果へ。決める以外（Esc・取り消し）は null */
export function promptResult(returnValue: string, value: string): string | null {
  return returnValue === "ok" ? value : null;
}

/** 閉じた理由を選ぶ窓の結果へ。並べたボタンの番号だけを返し、Esc・取り消し・知らない値は null */
export function chooseResult(returnValue: string, count: number): number | null {
  const i = Number(returnValue);
  return returnValue !== "" && Number.isInteger(i) && i >= 0 && i < count ? i : null;
}

export interface OpenOptions {
  /** 文とボタンの間に並べる部品（入力・選択・組の行など） */
  parts?: HTMLElement[];
  /** 決める（value "ok"）で閉じる前の確かめ。true で閉じる・false で閉じない・文字列は閉じずに窓に理由として出す */
  accept?: () => boolean | string;
}

export function open(message: string, buttons: [value: string, label: string][], options: OpenOptions = {}): Promise<string> {
  const { parts = [], accept } = options;
  const dialog = document.createElement("dialog");
  const form = document.createElement("form");
  form.method = "dialog";
  const p = document.createElement("p");
  p.textContent = message;
  // 焦点はすぐボタンへ行くので、窓の名前として文を結び、読み上げで文が聞こえるようにする
  dialog.setAttribute("aria-label", message);
  form.append(p, ...parts);
  let reason: HTMLParagraphElement | undefined;
  if (accept) {
    reason = document.createElement("p");
    reason.setAttribute("role", "alert");
    form.append(reason);
    // Enter での送りも submitter は最初のボタンになる
    form.addEventListener("submit", (e) => {
      if ((e.submitter as HTMLButtonElement | null)?.value !== "ok") return;
      const r = accept();
      if (r === true) return;
      e.preventDefault();
      if (typeof r === "string") reason!.textContent = r;
    });
  }
  const row = document.createElement("div");
  row.className = "dialog-buttons";
  for (const [value, label] of buttons) {
    const b = document.createElement("button");
    b.value = value;
    b.textContent = label;
    row.append(b);
  }
  form.append(row);
  dialog.append(form);
  (document.getElementById("dialog-root") ?? document.body).append(dialog);
  // 部品は開いた後に増え減りする（組の行）ので、焦点の行き先はその都度数える
  const focusables = () =>
    [...form.querySelectorAll<HTMLElement>("input, select, textarea, button")].filter((el) => !(el as HTMLButtonElement).disabled);
  // showModal は外を inert にするが、Tab は端で窓の外（文書そのもの）へ一度抜けるので、端で反対の端へ回す
  dialog.addEventListener("keydown", (e) => {
    if (e.key !== "Tab") return;
    const all = focusables();
    const edge = e.shiftKey ? all[0] : all[all.length - 1];
    if (document.activeElement !== edge) return;
    e.preventDefault();
    (e.shiftKey ? all[all.length - 1] : all[0]).focus();
  });
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      dialog.remove();
      resolve(dialog.returnValue);
    });
    dialog.showModal();
    focusables()[0].focus();
  });
}

export async function confirm3(message: string): Promise<Choice> {
  return choiceOf(await open(message, [["save", "保存する"], ["discard", "保存しない"], ["cancel", "取り消す"]]));
}

export async function prompt(label: string, initial: string): Promise<string | null> {
  const input = document.createElement("input");
  input.value = initial;
  input.setAttribute("aria-label", label);
  // Enter は最初のボタン（決める）で送る
  const rv = await open(label, [["ok", "決める"], ["cancel", "取り消す"]], { parts: [input] });
  return promptResult(rv, input.value);
}

export function notify(message: string): void {
  void open(message, [["ok", "閉じる"]]);
}

/** ボタンを並べて 1 つを選ぶ。選んだ番号、Esc・取り消しは null */
export async function choose(message: string, labels: string[]): Promise<number | null> {
  const buttons: [string, string][] = labels.map((label, i) => [String(i), label]);
  return chooseResult(await open(message, [...buttons, ["cancel", "取り消す"]]), labels.length);
}

/** 決めるボタンの名前を持つ確かめ。決めたときだけ true */
export async function confirm(message: string, okLabel: string): Promise<boolean> {
  return (await open(message, [["ok", okLabel], ["cancel", "取り消す"]])) === "ok";
}
