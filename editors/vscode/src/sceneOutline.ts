// SceneOutlineProvider - 解析の結果のシーンの並びを、アウトライン（DocumentSymbol）と
// 見出しの上の「▶ シーンを実行」（CodeLens）に写す。並びの意味は解析器に任せ、ここでは写すだけ。

import * as vscode from 'vscode';
import { WasmBridge, SceneNode } from './wasmBridge';

/** package.json の contributes.commands と同じ id・題（試験で照らす） */
const RUN_COMMAND = 'pasta.runSceneAtCursor';
const RUN_TITLE = '▶ シーンを実行';

function toRange(r: SceneNode['range']): vscode.Range {
  return new vscode.Range(
    new vscode.Position(r.start.line, r.start.character),
    new vscode.Position(r.end.line, r.end.character)
  );
}

function toSymbol(scene: SceneNode): vscode.DocumentSymbol {
  const kind = scene.kind === 'global' ? vscode.SymbolKind.Namespace : vscode.SymbolKind.Function;
  const symbol = new vscode.DocumentSymbol(scene.name, '', kind, toRange(scene.range), toRange(scene.heading));
  symbol.children = scene.children.map(toSymbol);
  return symbol;
}

export class SceneOutlineProvider implements vscode.DocumentSymbolProvider, vscode.CodeLensProvider {
  // shortcut: 1 枠の控え。DocumentSync・SemanticTokensProvider と解析を共有しない。大きな辞書で遅さが見えたら控えを結果ごとに共有する
  private cached: { uri: string; version: number; scenes: ReadonlyArray<SceneNode> } | undefined;

  constructor(private readonly bridge: Pick<WasmBridge, 'analyzeDocument'>) {}

  provideDocumentSymbols(document: vscode.TextDocument, _token: vscode.CancellationToken): vscode.DocumentSymbol[] {
    return this.scenesOf(document).map(toSymbol);
  }

  provideCodeLenses(document: vscode.TextDocument, _token: vscode.CancellationToken): vscode.CodeLens[] {
    return this.scenesOf(document)
      .flatMap((g) => [g, ...g.children])
      .map((s) => new vscode.CodeLens(toRange(s.heading), {
        title: RUN_TITLE,
        command: RUN_COMMAND,
        arguments: [document.uri, s.heading.start.line],
      }));
  }

  private scenesOf(document: vscode.TextDocument): ReadonlyArray<SceneNode> {
    const uri = document.uri.toString();
    if (this.cached?.uri === uri && this.cached.version === document.version) {
      return this.cached.scenes;
    }
    try {
      const { scenes } = this.bridge.analyzeDocument(document.getText());
      this.cached = { uri, version: document.version, scenes };
      return scenes;
    } catch {
      // 解析に失敗した文書は出さず、通知もしない（SemanticTokensProvider と同じ扱い）
      this.cached = undefined;
      return [];
    }
  }
}
