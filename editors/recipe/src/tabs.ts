// 純粋: タブを閉じる・開き直す・名前を変える・全部保存するときの判断（Monaco・DOM・Tauri を import しない）

export interface TabRef {
  id: number;
  /** master からの相対・'/' 区切り */
  rel: string;
  dirty: boolean;
}

/**
 * 名前の変更で付け替えるタブ: id → 新しい rel。ファイルなら同じ rel、フォルダならその中（区切りごとの前方一致なので dic と dict は別）。
 * 同じものかは Rust の to_uppercase と同じく大文字に揃えて比べる。区切りの数で切るので、大文字にして長さが変わる文字でも残りがずれない。
 */
export function retargetTabs(tabs: readonly TabRef[], oldRel: string, newRel: string): Map<number, string> {
  const old = oldRel.split("/").map((s) => s.toUpperCase());
  const out = new Map<number, string>();
  for (const tab of tabs) {
    const parts = tab.rel.split("/");
    if (parts.length < old.length || old.some((s, i) => parts[i].toUpperCase() !== s)) continue;
    out.set(tab.id, [newRel, ...parts.slice(old.length)].join("/"));
  }
  return out;
}

/** 未保存のタブが 1 つでもあれば 3 択で確かめる（2.8・5.9・5.10） */
export function needsConfirm(tabs: readonly TabRef[]): boolean {
  return tabs.some((t) => t.dirty);
}

/** 順に保存し、失敗した所で止める（残りは呼ばない・2.8・5.10） */
export async function saveAll(tabs: readonly TabRef[], save: (t: TabRef) => Promise<boolean>): Promise<"ok" | "failed"> {
  for (const tab of tabs) {
    if (!(await save(tab))) return "failed";
  }
  return "ok";
}

/** 3 択の後: 保存して成功・保存しない は進む。保存の失敗・取り消しは留まる（2.8・5.9・5.10） */
export function afterChoice(choice: "save" | "discard" | "cancel", saved: "ok" | "failed" | null): "proceed" | "stay" {
  if (choice === "discard") return "proceed";
  return choice === "save" && saved === "ok" ? "proceed" : "stay";
}

/** 同じファイルのタブ（Rust の to_uppercase と同じく大文字に揃えて比べる・5.4） */
export function findSame<T extends { rel: string }>(tabs: readonly T[], rel: string): T | undefined {
  const key = rel.toUpperCase();
  return tabs.find((t) => t.rel.toUpperCase() === key);
}
