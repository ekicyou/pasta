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

/**
 * 見張りの知らせ（paths はフォルダの名前や名前の変更の新旧も入る）を、前と今のツリーの一覧と合わせて決める。
 * scenes: 読み直す .pasta（消えた・知らせに含まれる・新しく出た。10.1・10.5）。check: ゴースト全体の検査を求めるか（9.3）
 */
export function planGhostChanged(prev: readonly string[], now: readonly string[], paths: readonly string[]): { scenes: string[]; check: boolean } {
  const keys = (files: readonly string[]) => new Set(files.map((f) => f.toUpperCase()));
  const prevKeys = keys(prev);
  const nowKeys = keys(now);
  const seen = new Set<string>();
  const scenes = [...prev.filter((f) => !nowKeys.has(f.toUpperCase())), ...paths, ...now.filter((f) => !prevKeys.has(f.toUpperCase()))].filter((f) => {
    const key = f.toUpperCase();
    if (!key.endsWith(".PASTA") || seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  const checked = (f: string) => /\.(pasta|lua)$/i.test(f);
  const name = (f: string) => f.slice(f.lastIndexOf("/") + 1).toLowerCase();
  const listed = (files: readonly string[]) => [...keys(files.filter(checked))].sort().join("\n");
  const check = paths.some((f) => checked(f) || name(f) === "pasta.toml") || listed(prev) !== listed(now);
  return { scenes, check };
}
