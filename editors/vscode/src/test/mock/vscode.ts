// Minimal VSCode API mock for unit testing
// Provides just enough structure to test our modules without real VSCode.

export class Uri {
  readonly scheme: string;
  readonly authority: string;
  readonly path: string;
  readonly fsPath: string;

  private constructor(scheme: string, authority: string, path: string) {
    this.scheme = scheme;
    this.authority = authority;
    this.path = path;
    this.fsPath = path;
  }

  static parse(value: string): Uri {
    return new Uri('file', '', value);
  }

  static file(path: string): Uri {
    return new Uri('file', '', path);
  }

  static joinPath(base: Uri, ...pathSegments: string[]): Uri {
    return new Uri(base.scheme, base.authority, base.path + '/' + pathSegments.join('/'));
  }

  toString(): string {
    return `${this.scheme}://${this.path}`;
  }
}

export class Position {
  readonly line: number;
  readonly character: number;
  constructor(line: number, character: number) {
    this.line = line;
    this.character = character;
  }
}

export class Range {
  readonly start: Position;
  readonly end: Position;
  constructor(start: Position, end: Position) {
    this.start = start;
    this.end = end;
  }
}

export enum DiagnosticSeverity {
  Error = 0,
  Warning = 1,
  Information = 2,
  Hint = 3,
}

export class Diagnostic {
  readonly range: Range;
  readonly message: string;
  readonly severity: DiagnosticSeverity;
  constructor(range: Range, message: string, severity: DiagnosticSeverity) {
    this.range = range;
    this.message = message;
    this.severity = severity;
  }
}

export class EventEmitter<T> {
  private listeners: Array<(e: T) => void> = [];
  readonly event = (listener: (e: T) => void) => {
    this.listeners.push(listener);
    return { dispose: () => { this.listeners = this.listeners.filter(l => l !== listener); } };
  };
  fire(data: T): void {
    for (const l of this.listeners) {
      l(data);
    }
  }
  dispose(): void {
    this.listeners = [];
  }
}

export class SemanticTokensLegend {
  readonly tokenTypes: string[];
  readonly tokenModifiers: string[];
  constructor(tokenTypes: string[], tokenModifiers: string[] = []) {
    this.tokenTypes = tokenTypes;
    this.tokenModifiers = tokenModifiers;
  }
}

export class SemanticTokensBuilder {
  private data: number[] = [];
  constructor(private legend: SemanticTokensLegend) {}

  push(deltaLine: number, deltaStartChar: number, length: number, tokenType: number, tokenModifiers: number): void {
    this.data.push(deltaLine, deltaStartChar, length, tokenType, tokenModifiers);
  }

  build(): SemanticTokens {
    return new SemanticTokens(new Uint32Array(this.data));
  }
}

export class SemanticTokens {
  readonly data: Uint32Array;
  constructor(data: Uint32Array) {
    this.data = data;
  }
}

// Stub for DiagnosticCollection
export class MockDiagnosticCollection {
  readonly name: string;
  private store = new Map<string, Diagnostic[]>();

  constructor(name: string) {
    this.name = name;
  }

  set(uri: Uri, diagnostics: Diagnostic[]): void {
    this.store.set(uri.toString(), diagnostics);
  }

  delete(uri: Uri): void {
    this.store.delete(uri.toString());
  }

  clear(): void {
    this.store.clear();
  }

  get(uri: Uri): Diagnostic[] | undefined {
    return this.store.get(uri.toString());
  }

  get size(): number {
    return this.store.size;
  }

  dispose(): void {
    this.store.clear();
  }
}

// Stubs for vscode.languages / vscode.workspace / vscode.window
export const languages = {
  createDiagnosticCollection: (name: string) => new MockDiagnosticCollection(name),
  registerDocumentSemanticTokensProvider: () => ({ dispose: () => {} }),
};

export const workspace = {
  textDocuments: [] as any[],
  onDidOpenTextDocument: () => ({ dispose: () => {} }),
  onDidChangeTextDocument: () => ({ dispose: () => {} }),
  onDidCloseTextDocument: () => ({ dispose: () => {} }),
  fs: {
    readFile: async () => new Uint8Array(0),
  },
};

export const window = {
  createOutputChannel: (name: string) => ({
    name,
    appendLine: (_msg: string) => {},
    dispose: () => {},
  }),
  showErrorMessage: (_msg: string) => {},
};
