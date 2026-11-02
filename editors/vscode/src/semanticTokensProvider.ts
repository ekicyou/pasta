// SemanticTokensProvider - Provides semantic tokens from WASM analysis
//
// Implements vscode.DocumentSemanticTokensProvider to deliver
// pasta_lsp's 17 semantic token types + 3 modifiers.

import * as vscode from 'vscode';
import { WasmBridge, SemanticTokenData } from './wasmBridge';

/** URI ごとの色の控え。版が文書の版と同じときだけ使う */
interface CachedTokens {
  readonly version: number;
  readonly tokens: ReadonlyArray<SemanticTokenData>;
}

/**
 * Token types matching pasta_lsp token_types.rs TOKEN_TYPES (same order).
 */
export const PASTA_TOKEN_TYPES: readonly string[] = [
  'comment',       // 0
  'namespace',     // 1: グローバルシーン
  'scene',         // 2: ローカルシーン (custom)
  'decorator',     // 3: 属性
  'word',          // 4: 単語 (custom)
  'variable',      // 5
  'call',          // 6: Call文 (custom)
  'actor',         // 7: アクター辞書 (custom)
  'actorName',     // 8: アクター名 (custom)
  'codeBlock',     // 9: Luaコードブロック (custom)
  'talk',          // 10: Talk文 (custom)
  'sakuraScript',  // 11: さくらスクリプト (custom)
  'escape',        // 12: エスケープ (custom)
  'operator',      // 13
  'number',        // 14: 数値リテラル
  'cueMarker',     // 15: キューコマンドマーカー (custom)
  'cueCommand',    // 16: キューコマンド名 (custom)
] as const;

/**
 * Token modifiers matching pasta_lsp token_types.rs TOKEN_MODIFIERS (same order).
 */
export const PASTA_TOKEN_MODIFIERS: readonly string[] = [
  'declaration',   // 0
  'definition',    // 1
  'global',        // 2 (custom)
] as const;

/** SemanticTokensLegend for registration */
export const PASTA_TOKENS_LEGEND = new vscode.SemanticTokensLegend(
  [...PASTA_TOKEN_TYPES],
  [...PASTA_TOKEN_MODIFIERS]
);

/**
 * Provides semantic tokens for *.pasta documents.
 *
 * Delegates analysis to WasmBridge, then converts the result
 * into VSCode SemanticTokens using SemanticTokensBuilder.
 */
export class SemanticTokensProvider implements vscode.DocumentSemanticTokensProvider {
  private readonly wasmBridge: WasmBridge;
  private cachedTokens: Map<string, CachedTokens> = new Map();

  private readonly _onDidChangeSemanticTokens = new vscode.EventEmitter<void>();
  readonly onDidChangeSemanticTokens = this._onDidChangeSemanticTokens.event;

  constructor(wasmBridge: WasmBridge) {
    this.wasmBridge = wasmBridge;
  }

  provideDocumentSemanticTokens(
    document: vscode.TextDocument,
    _token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.SemanticTokens> {
    if (!this.wasmBridge.isReady()) {
      return null;
    }

    const uri = document.uri.toString();
    const cached = this.cachedTokens.get(uri);

    if (cached && cached.version === document.version) {
      return this.buildSemanticTokens(cached.tokens);
    }

    // 控えが無いか版が違えば、その場で解いて今の版として控える
    try {
      const result = this.wasmBridge.analyzeDocument(document.getText());
      this.cachedTokens.set(uri, { version: document.version, tokens: [...result.tokens] });
      return this.buildSemanticTokens(result.tokens);
    } catch {
      // 前の版の色は返さない（TextMate に任せる）
      this.cachedTokens.delete(uri);
      return null;
    }
  }

  /** DocumentSync が解析した結果を、その版と一緒に控える */
  updateTokens(uri: string, version: number, tokens: ReadonlyArray<SemanticTokenData>): void {
    this.cachedTokens.set(uri, { version, tokens: [...tokens] });
    this._onDidChangeSemanticTokens.fire();
  }

  /** Clear cached tokens for a document */
  clearTokens(uri: string): void {
    this.cachedTokens.delete(uri);
  }

  private buildSemanticTokens(tokens: ReadonlyArray<SemanticTokenData>): vscode.SemanticTokens {
    const builder = new vscode.SemanticTokensBuilder(PASTA_TOKENS_LEGEND);

    // pasta_lsp returns delta-encoded values, but SemanticTokensBuilder.push()
    // expects ABSOLUTE line/character positions (it encodes deltas internally).
    // Convert delta → absolute before pushing.
    let currentLine = 0;
    let currentChar = 0;

    for (const token of tokens) {
      currentLine += token.deltaLine;
      if (token.deltaLine > 0) {
        currentChar = token.deltaStartCharacter;
      } else {
        currentChar += token.deltaStartCharacter;
      }

      builder.push(
        currentLine,
        currentChar,
        token.length,
        token.tokenType,
        token.tokenModifiers
      );
    }

    return builder.build();
  }
}
