// 純粋: 外の変更をタブにどう当てるかを決める（DOM・Monaco・Tauri を import しない）

export type DiskState =
  | { kind: "text"; text: string; bom: boolean }
  | { kind: "missing" }
  | { kind: "unreadable"; reason: string };

export interface TabSnapshot {
  /** 最後にディスクと揃えた中身（消えたら null） */
  diskText: string | null;
  diskBom: boolean;
  dirty: boolean;
  deleted: boolean;
  /** エディタの今の中身（消えた後に戻ってきたときに比べる） */
  editorText: string;
}

export type ExternalAction = "none" | "reload" | "ask" | "deleted";

export function decideExternal(tab: TabSnapshot, disk: DiskState): ExternalAction {
  if (disk.kind === "missing") return tab.deleted ? "none" : "deleted"; // 10.4
  if (disk.kind === "unreadable") return "none"; // 次の知らせで決める
  if (disk.text === tab.diskText && disk.bom === tab.diskBom) return "none"; // 自分の保存（10.6）
  if (tab.deleted) return disk.text === tab.editorText ? "none" : "ask"; // 消えた後に戻ってきた
  return tab.dirty ? "ask" : "reload"; // 10.3・10.2
}

/** read_text の失敗を DiskState へ。理由の文では消えたか分からないので、ツリーの一覧に無ければ消えた（10.4） */
export function failedRead(rel: string, files: readonly string[], reason: string): DiskState {
  const key = rel.toUpperCase();
  return files.some((f) => f.toUpperCase() === key) ? { kind: "unreadable", reason } : { kind: "missing" };
}
