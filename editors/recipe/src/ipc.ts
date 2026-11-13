// Rust のコマンドの表を 1 対 1 で包む（型は Rust の serde の名前と同じ）。残りのコマンドと事件は 4.2 で足す
import { invoke } from "@tauri-apps/api/core";

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

// 失敗は Rust の Err(String) の文のまま reject される
export const analyze = (source: string) => invoke<AnalysisResult>("analyze", { source });
export const legend = () => invoke<Legend>("legend");
