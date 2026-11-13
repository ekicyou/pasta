// Rust のコマンドの表と事件を 1 対 1 で包む（型は Rust の serde の名前と同じ）
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Position {
  line: number;
  character: number;
}
export interface Range {
  start: Position;
  end: Position;
}
export interface SemanticToken {
  deltaLine: number;
  deltaStartCharacter: number;
  length: number;
  tokenType: number;
  tokenModifiers: number;
}
export interface Diagnostic {
  range: Range;
  message: string;
  severity: number;
}
export interface SceneNode {
  name: string;
  kind: "global" | "local";
  heading: Range;
  range: Range;
  children: SceneNode[];
}
export interface AnalysisResult {
  tokens: SemanticToken[];
  diagnostics: Diagnostic[];
  scenes: SceneNode[];
}
export interface Legend {
  tokenTypes: string[];
  tokenModifiers: string[];
}

// pasta_check::validate::report の serde の名前（Severity・Stats は camelCase、code・SkipReason は kebab-case）
export type Severity = "error" | "warning";
export type SkipReason = "syntax" | "unreadable-file" | "config";
/** line は 1 始まり・column は UTF-16 の 1 始まり */
export interface Finding {
  code: string;
  severity: Severity;
  path: string;
  line: number | null;
  column: number | null;
  message: string;
}
export interface Stats {
  dictionaryFiles: number;
  errors: number;
  warnings: number;
  uncheckedDynamicCalls: number;
  skipReasons: SkipReason[];
}
export interface Report {
  findings: Finding[];
  stats: Stats;
}
/** master からの相対・'/' 区切り */
export interface TreeListing {
  files: string[];
  dirs: string[];
}
export interface TextFile {
  text: string;
  bom: boolean;
}
export interface Resolved {
  master: string;
  file: string | null;
}
export interface GhostInfo {
  master: string;
  name: string;
  version: string;
}
export interface RecentEntry {
  master: string;
  name: string;
}
export type EntryKind = "file" | "dir";
export interface GhostChanged {
  paths: string[];
}
export interface OpenRequest {
  path: string | null;
}

/** 失敗は Rust の Err(String) の文。万一それ以外が来ても理由の文にして投げ直す */
const call = <T>(cmd: string, args?: Record<string, unknown>): Promise<T> =>
  invoke<T>(cmd, args).catch((e: unknown) => {
    throw typeof e === "string" ? e : String(e);
  });

// 引数の名前は Tauri が camelCase から Rust の snake_case へ写す
export const initialPath = () => call<string | null>("initial_path");
export const resolveGhost = (path: string) => call<Resolved>("resolve_ghost", { path });
export const openGhost = (master: string) => call<GhostInfo>("open_ghost", { master });
export const listTree = () => call<TreeListing>("list_tree");
export const readText = (rel: string) => call<TextFile>("read_text", { rel });
export const writeText = (rel: string, text: string, bom: boolean) => call<null>("write_text", { rel, text, bom });
export const createEntry = (parentRel: string, name: string, kind: EntryKind) =>
  call<string>("create_entry", { parentRel, name, kind });
export const renameEntry = (rel: string, newName: string) => call<string>("rename_entry", { rel, newName });
export const analyze = (source: string) => call<AnalysisResult>("analyze", { source });
export const legend = () => call<Legend>("legend");
export const checkGhost = () => call<Report>("check_ghost");
export const recentList = () => call<RecentEntry[]>("recent_list");
export const recentRemove = (master: string) => call<null>("recent_remove", { master });

export const onGhostChanged = (handler: (e: GhostChanged) => void): Promise<UnlistenFn> =>
  listen<GhostChanged>("ghost-changed", (e) => handler(e.payload));
export const onOpenRequest = (handler: (e: OpenRequest) => void): Promise<UnlistenFn> =>
  listen<OpenRequest>("open-request", (e) => handler(e.payload));
