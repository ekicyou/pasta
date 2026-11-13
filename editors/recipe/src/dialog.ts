// HTML の小さな窓（3 択の確かめ・名前の入力・知らせ）。<dialog> の showModal が焦点を窓の中に閉じ、Esc で取り消す。
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

function open(message: string, buttons: [value: string, label: string][], input?: HTMLInputElement): Promise<string> {
  const dialog = document.createElement("dialog");
  const form = document.createElement("form");
  form.method = "dialog";
  const p = document.createElement("p");
  p.textContent = message;
  // 焦点はすぐボタンへ行くので、窓の名前として文を結び、読み上げで文が聞こえるようにする
  dialog.setAttribute("aria-label", message);
  form.append(p);
  if (input) form.append(input);
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
  // showModal は外を inert にするが、Tab は端で窓の外（文書そのもの）へ一度抜けるので、端で反対の端へ回す
  const focusables = [...(input ? [input] : []), ...row.querySelectorAll("button")];
  dialog.addEventListener("keydown", (e) => {
    if (e.key !== "Tab") return;
    const edge = e.shiftKey ? focusables[0] : focusables[focusables.length - 1];
    if (document.activeElement !== edge) return;
    e.preventDefault();
    (e.shiftKey ? focusables[focusables.length - 1] : focusables[0]).focus();
  });
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      dialog.remove();
      resolve(dialog.returnValue);
    });
    dialog.showModal();
    (input ?? row.querySelector("button"))!.focus();
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
  const rv = await open(label, [["ok", "決める"], ["cancel", "取り消す"]], input);
  return promptResult(rv, input.value);
}

export function notify(message: string): void {
  void open(message, [["ok", "閉じる"]]);
}
