// WasmBridge - Bridge between TypeScript and pasta_lsp.wasm
//
// Loads the WASM module and exposes analyzeDocument() for
// semantic token analysis and diagnostics.

import * as vscode from 'vscode';

/** Semantic token data from WASM analysis */
export interface SemanticTokenData {
  deltaLine: number;
  deltaStartCharacter: number;
  length: number;
  tokenType: number;
  tokenModifiers: number;
}

/** Diagnostic data from WASM analysis */
export interface DiagnosticData {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  message: string;
  severity: number;
}

/** Result of WASM analysis */
export interface WasmAnalysisResult {
  tokens: ReadonlyArray<SemanticTokenData>;
  diagnostics: ReadonlyArray<DiagnosticData>;
}

// --- ゴースト全体の検査（pasta_check::validate の serde 形と同じ） ---

export type TomlSource =
  | { kind: 'missing' }
  | { kind: 'text'; text: string }
  | { kind: 'unreadable'; reason: string };
export type FileText =
  | { kind: 'text'; text: string }
  | { kind: 'notUtf8' }
  | { kind: 'unreadable'; reason: string };
export interface GhostInput {
  pastaToml: TomlSource;
  entries: ReadonlyArray<string>;
  /** ghost/master の配下のフォルダの相対パス（'/' 区切り・末尾の '/' なし） */
  dirs: ReadonlyArray<string>;
}
/** 飛び先の検査を省いた理由（核の SkipReason と同じ綴り） */
export type SkipReason = 'syntax' | 'unreadable-file' | 'config';
const SKIP_REASONS: ReadonlyArray<unknown> = ['syntax', 'unreadable-file', 'config'] satisfies SkipReason[];
export type GhostSeverity = 'error' | 'warning';
export interface GhostFinding {
  code: string;
  severity: GhostSeverity;
  path: string;
  line: number | null;
  column: number | null;
  message: string;
}
export interface GhostStats {
  dictionaryFiles: number;
  errors: number;
  warnings: number;
  uncheckedDynamicCalls: number;
  skipReasons: ReadonlyArray<SkipReason>;
}
export interface GhostReport {
  findings: ReadonlyArray<GhostFinding>;
  stats: GhostStats;
}

/** Shape of the wasm-bindgen CJS bindings module loaded by {@link WasmBridge}. */
interface WasmBindings {
  wasm_analyze(text: string): unknown;
  wasm_select_ghost_files(input: unknown): unknown;
  wasm_validate_ghost(input: unknown, files: unknown): unknown;
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((v) => typeof v === 'string');
}

function isNumberOrNull(value: unknown): value is number | null {
  return value === null || typeof value === 'number';
}

function isGhostFinding(value: unknown): value is GhostFinding {
  if (typeof value !== 'object' || value === null) return false;
  const f = value as Record<string, unknown>;
  return (
    typeof f.code === 'string' &&
    (f.severity === 'error' || f.severity === 'warning') &&
    typeof f.path === 'string' &&
    isNumberOrNull(f.line) &&
    isNumberOrNull(f.column) &&
    typeof f.message === 'string'
  );
}

function isGhostReport(value: unknown): value is GhostReport {
  if (typeof value !== 'object' || value === null) return false;
  const r = value as { findings?: unknown; stats?: unknown };
  if (!Array.isArray(r.findings) || !r.findings.every(isGhostFinding)) return false;
  if (typeof r.stats !== 'object' || r.stats === null) return false;
  const s = r.stats as Record<string, unknown>;
  return (
    typeof s.dictionaryFiles === 'number' &&
    typeof s.errors === 'number' &&
    typeof s.warnings === 'number' &&
    typeof s.uncheckedDynamicCalls === 'number' &&
    Array.isArray(s.skipReasons) &&
    s.skipReasons.every((r) => SKIP_REASONS.includes(r))
  );
}

/**
 * Boundary validation for the untyped `wasm_analyze` result (cell 3.56 / G3):
 * the result must carry `tokens` and `diagnostics` arrays. Rejecting a
 * malformed shape HERE keeps the failure on the callers' established
 * `WASM analysis failed` exception path instead of escaping as a distant
 * TypeError when the result is spread/iterated later.
 */
function isWasmAnalysisResult(value: unknown): value is WasmAnalysisResult {
  return (
    typeof value === 'object' &&
    value !== null &&
    Array.isArray((value as { tokens?: unknown }).tokens) &&
    Array.isArray((value as { diagnostics?: unknown }).diagnostics)
  );
}

/**
 * Bridge to pasta_lsp WASM module.
 *
 * Loads and initializes the WASM binary, then provides
 * synchronous document analysis via wasm_analyze().
 */
export class WasmBridge {
  private wasmModule: WasmBindings | null = null;
  private ready = false;
  private readonly outputChannel: vscode.OutputChannel;

  constructor(outputChannel: vscode.OutputChannel) {
    this.outputChannel = outputChannel;
  }

  /**
   * Initialize the WASM module from the given URI.
   * @throws Error if WASM loading or initialization fails
   */
  async initialize(wasmUri: vscode.Uri): Promise<void> {
    const startTime = Date.now();

    try {
      // The wasm-pack --target nodejs output uses __dirname to find
      // the .wasm file, and auto-initializes via require('fs').readFileSync.
      // We need to resolve the JS bindings path relative to the extension.
      const wasmDir = vscode.Uri.joinPath(wasmUri, '..');
      const jsBindingsPath = vscode.Uri.joinPath(wasmDir, 'pasta_lsp_wasm.js').fsPath;

      // Use require() to load the CJS wasm-bindgen bindings.
      // The module self-initializes by reading the .wasm file from __dirname.
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const wasmBindings = require(jsBindingsPath);

      this.wasmModule = wasmBindings;
      this.ready = true;

      const elapsed = Date.now() - startTime;
      this.outputChannel.appendLine(`WASM module loaded in ${elapsed}ms`);
    } catch (error) {
      this.ready = false;
      throw new Error(`Failed to load WASM module: ${error}`);
    }
  }

  /**
   * Analyze a document and return semantic tokens + diagnostics.
   * @throws Error if WASM is not initialized or analysis fails
   */
  analyzeDocument(text: string): WasmAnalysisResult {
    if (!this.ready || !this.wasmModule) {
      throw new Error('WASM module is not initialized');
    }

    try {
      const result = this.wasmModule.wasm_analyze(text);
      if (!isWasmAnalysisResult(result)) {
        throw new Error('malformed result (expected { tokens: [...], diagnostics: [...] })');
      }
      return result;
    } catch (error) {
      throw new Error(`WASM analysis failed: ${error}`);
    }
  }

  /**
   * pasta.toml の pasta_patterns に従って検査する辞書ファイルを選ぶ（11.8）。
   * @throws Error if WASM is not initialized or the result is not a string[]
   */
  selectGhostFiles(input: GhostInput): string[] {
    const wasm = this.requireModule();
    const result = wasm.wasm_select_ghost_files(input);
    if (!isStringArray(result)) {
      throw new Error('WASM selectGhostFiles failed: malformed result (expected string[])');
    }
    return result;
  }

  /**
   * ゴースト全体を検査し、核の報告をそのまま返す（11.2）。
   * @throws Error if WASM is not initialized or the result is not a GhostReport
   */
  validateGhost(input: GhostInput, files: Readonly<Record<string, FileText>>): GhostReport {
    const wasm = this.requireModule();
    const result = wasm.wasm_validate_ghost(input, files);
    if (!isGhostReport(result)) {
      throw new Error('WASM validateGhost failed: malformed result (expected GhostReport)');
    }
    return result;
  }

  private requireModule(): WasmBindings {
    if (!this.ready || !this.wasmModule) {
      throw new Error('WASM module is not initialized');
    }
    return this.wasmModule;
  }

  /** Check if the WASM module is ready for use */
  isReady(): boolean {
    return this.ready;
  }

  /** Dispose WASM resources */
  dispose(): void {
    this.wasmModule = null;
    this.ready = false;
  }
}
