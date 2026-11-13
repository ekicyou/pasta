// 開発の操作の 4 つの窓（相手を選ぶ 3.4／値の書き換え・追加 7.2・7.4・7.5・7.10／削除の確かめ 7.3／ローカル変数 8.1・8.2・8.6〜8.8）。
// 名前・値・理由はディスクやゴーストから来るので textContent だけで書く。部品には読み上げの名前を付ける（11.4）。
import type { Endpoint } from "../../shared/mcpClient.ts";
import type { PickItem } from "../../shared/devSession.ts";
import { choose, confirm, open } from "./dialog.ts";
import { buildLocals, type LocalPair, type LocalsOutcome, type VarRow } from "./ipc.ts";

type Kind = LocalPair["kind"];
const KINDS: [Kind, string][] = [["string", "文字列"], ["number", "数"], ["boolean", "真偽"]];

/** 選ぶ窓の番号を相手へ。閉じた（null）なら何も選ばない（3.4） */
export const pickedEndpoint = (items: readonly PickItem[], index: number | null): Endpoint | undefined =>
  index === null ? undefined : items[index]?.endpoint;

/** 追加の名前の確かめ（7.5）。空なら閉じずに理由を出す */
export const nameRequired = (name: string): true | string => (name === "" ? "名前が要ります" : true);

export const deleteMessage = (name: string): string => `＄＊${name} をゴーストの保存変数から消します`;

/** 真偽の select の初めの値。今の値が false のときだけ false */
export const boolValue = (value: string): "true" | "false" => (value === "false" ? "false" : "true");

/** 弾きの理由（index は 0 始まり）を組の行ごとの文へ。並びの外は捨てる（8.6〜8.8） */
export function problemsByRow(problems: (LocalsOutcome & { kind: "invalid" })["problems"], count: number): string[] {
  const rows: string[][] = Array.from({ length: count }, () => []);
  for (const p of problems) rows[p.index]?.push(p.message);
  return rows.map((m) => m.join("／"));
}

export async function pickEndpoint(placeHolder: string, items: readonly PickItem[]): Promise<Endpoint | undefined> {
  return pickedEndpoint(items, await choose(placeHolder, items.map((i) => i.label)));
}

export const confirmDelete = (name: string): Promise<boolean> => confirm(deleteMessage(name), "消す");

/** 型の 3 択と値（真偽は true・false の 2 択、ほかは 1 行の入力 7.10）。型を変えると値の部品を差し替える */
function kindValue(kind: Kind, value: string) {
  const kindSel = document.createElement("select");
  kindSel.setAttribute("aria-label", "型");
  for (const [v, label] of KINDS) kindSel.add(new Option(label, v));
  kindSel.value = kind;
  const text = document.createElement("input");
  text.type = "text";
  text.setAttribute("aria-label", "値");
  const bool = document.createElement("select");
  bool.setAttribute("aria-label", "値");
  bool.add(new Option("true", "true"));
  bool.add(new Option("false", "false"));
  text.value = kind === "boolean" ? "" : value;
  bool.value = boolValue(value);
  // 隠した方は disabled にして、窓の焦点の回し（dialog.ts）から外す
  const sync = () => {
    const isBool = kindSel.value === "boolean";
    text.hidden = text.disabled = isBool;
    bool.hidden = bool.disabled = !isBool;
  };
  kindSel.addEventListener("change", sync);
  sync();
  const read = (): { kind: Kind; value: string } => {
    const k = kindSel.value as Kind;
    return { kind: k, value: k === "boolean" ? bool.value : text.value };
  };
  return { parts: [kindSel, text, bool], read };
}

/** 書き換え（row）・追加（null）の窓。決めたら送る名前・型・値、閉じたら null（7.2・7.4・7.5） */
export async function editVariable(
  row: (VarRow & { kind: Kind }) | null,
): Promise<{ name: string; kind: Kind; value: string } | null> {
  const name = document.createElement("input");
  name.type = "text";
  name.setAttribute("aria-label", "名前");
  const kv = kindValue(row?.kind ?? "string", row?.value ?? "");
  const message = row === null ? "保存変数を足します" : `＄＊${row.name} を書き換えます`;
  const rv = await open(message, [["ok", row === null ? "足す" : "書き換える"], ["cancel", "取り消す"]], {
    parts: row === null ? [name, ...kv.parts] : kv.parts,
    accept: () => (row === null ? nameRequired(name.value) : true),
  });
  return rv === "ok" ? { name: row?.name ?? name.value, ...kv.read() } : null;
}

/** ローカル変数を入れて鳴らす窓（8.1・8.2）。鳴らすなら組とインライン表（組が 0 なら表なし 8.5）、閉じたら null */
export async function localsDialog(
  label: string,
  initial: readonly LocalPair[],
): Promise<{ pairs: LocalPair[]; table: string | undefined } | null> {
  const list = document.createElement("div");
  list.className = "locals-rows";
  const rows: { el: HTMLElement; problem: HTMLElement; read: () => LocalPair }[] = [];
  const add = document.createElement("button");
  add.type = "button"; // 送りのボタンにしない（Enter は「鳴らす」で送る）
  add.textContent = "組を足す";
  // 組の番号は消すたびに詰まるので、読み上げの名前を付け直す
  const relabel = () => rows.forEach((r, i) => r.el.setAttribute("aria-label", `組 ${i + 1}`));
  const addRow = (pair: LocalPair) => {
    const el = document.createElement("div");
    el.className = "locals-row";
    el.setAttribute("role", "group");
    const name = document.createElement("input");
    name.type = "text";
    name.setAttribute("aria-label", "名前");
    name.value = pair.name;
    const kv = kindValue(pair.kind, pair.value);
    const del = document.createElement("button");
    del.type = "button";
    del.textContent = "この組を消す";
    const problem = document.createElement("p");
    problem.className = "locals-problem";
    problem.setAttribute("role", "alert");
    el.append(name, ...kv.parts, del, problem);
    const row = { el, problem, read: (): LocalPair => ({ name: name.value, ...kv.read() }) };
    del.addEventListener("click", () => {
      rows.splice(rows.indexOf(row), 1);
      el.remove();
      relabel();
      add.focus(); // 押したボタンが消えるので焦点を窓の中に残す
    });
    rows.push(row);
    list.append(el);
    relabel();
    return name;
  };
  initial.forEach(addRow);
  add.addEventListener("click", () => addRow({ name: "", kind: "string", value: "" }).focus());
  let result: { pairs: LocalPair[]; table: string | undefined } | undefined;
  const rv = await open(`値を入れて実行：${label}`, [["ok", "鳴らす"], ["cancel", "取り消す"]], {
    parts: [list, add],
    accept: async () => {
      // 待つ間に組を足し消ししても、尋ねたときの組へ理由を寄せる
      const asked = [...rows];
      const pairs = asked.map((r) => r.read());
      if (pairs.length === 0) {
        result = { pairs, table: undefined };
        return true;
      }
      let outcome: LocalsOutcome;
      try {
        outcome = await buildLocals(pairs);
      } catch (e) {
        return String(e);
      }
      if (outcome.kind === "ok") {
        result = { pairs, table: outcome.table };
        return true;
      }
      for (const r of rows) r.problem.textContent = "";
      problemsByRow(outcome.problems, pairs.length).forEach((m, i) => (asked[i].problem.textContent = m));
      return false;
    },
  });
  return rv === "ok" && result ? result : null;
}
