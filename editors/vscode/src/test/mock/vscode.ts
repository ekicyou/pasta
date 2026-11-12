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

  with(change: { path?: string }): Uri {
    return new Uri(this.scheme, this.authority, change.path ?? this.path);
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
  source?: string;
  code?: string | number;
  constructor(range: Range, message: string, severity: DiagnosticSeverity) {
    this.range = range;
    this.message = message;
    this.severity = severity;
  }
}

export enum SymbolKind {
  Namespace = 2,
  Function = 11,
}

export class DocumentSymbol {
  children: DocumentSymbol[] = [];
  constructor(
    public name: string,
    public detail: string,
    public kind: SymbolKind,
    public range: Range,
    public selectionRange: Range,
  ) {}
}

export class CodeLens {
  constructor(
    public range: Range,
    public command?: { title: string; command: string; arguments?: unknown[] },
  ) {}
  get isResolved(): boolean { return this.command !== undefined; }
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

// ---------------------------------------------------------------------------
// Additional API surface so REAL extension modules can run against this mock
// (bundled via esbuild --alias:vscode=./src/test/mock/vscode.ts).
// Test-only helpers are exported alongside; the module is a singleton within
// a test bundle, so tests importing './mock/vscode' directly observe the same
// state the module under test mutates.
// ---------------------------------------------------------------------------

export class ThemeColor {
  constructor(readonly id: string) {}
}

export enum StatusBarAlignment {
  Left = 1,
  Right = 2,
}

export class MockStatusBarItem {
  text = '';
  tooltip: string | undefined;
  command: string | undefined;
  visible = false;
  show(): void { this.visible = true; }
  hide(): void { this.visible = false; }
  dispose(): void { /* no-op */ }
}

export class MockTextEditorDecorationType {
  disposed = false;
  constructor(readonly options: unknown) {}
  dispose(): void { this.disposed = true; }
}

// --- test-observable state ---------------------------------------------------

/** All status bar items created via window.createStatusBarItem (in order). */
export const createdStatusBarItems: MockStatusBarItem[] = [];

/**
 * All user-facing messages shown via showErrorMessage / showWarningMessage.
 * `items` records the action button labels passed to the message (if any), so
 * tests can assert which actions were offered.
 */
export const shownMessages: Array<{ kind: 'error' | 'warning'; message: string; items?: string[] }> = [];

/**
 * Programmable return value for the NEXT showWarningMessage call that passes
 * action items (the label the "author clicked", or `undefined` to dismiss).
 * Consumed once per call.
 */
export let nextWarningSelection: string | undefined = undefined;
export function setNextWarningSelection(value: string | undefined): void {
  nextWarningSelection = value;
}

/**
 * Programmable response for the next window.showInputBox call. Tests set this
 * to a string (entered name) or `undefined` (cancel). Each call records its
 * options in `inputBoxCalls` for assertion.
 */
export let nextInputBoxResult: string | undefined = undefined;
export function setNextInputBoxResult(value: string | undefined): void {
  nextInputBoxResult = value;
}
export const inputBoxCalls: Array<Record<string, unknown> | undefined> = [];

// --- event emitters (exported so tests can fire events) ----------------------

export const didOpenTextDocumentEmitter = new EventEmitter<any>();
export const didChangeTextDocumentEmitter = new EventEmitter<any>();
export const didCloseTextDocumentEmitter = new EventEmitter<any>();
export const didSaveTextDocumentEmitter = new EventEmitter<any>();
export const didChangeActiveTextEditorEmitter = new EventEmitter<any>();

// --- in-memory fake file system (workspace.fs) --------------------------------

export enum FileType {
  Unknown = 0,
  File = 1,
  Directory = 2,
  SymbolicLink = 64,
}

/**
 * Fake files keyed by Uri.path. Directories are implied by the paths below
 * them, or placed explicitly with setFakeDir (empty ones). A symlink entry reports `type | FileType.SymbolicLink`. An unreadable
 * entry lists/stats as a file but its readFile rejects.
 */
const fakeEntries = new Map<string, { type: FileType; data: Uint8Array; unreadable?: boolean }>();

export function resetFakeFs(): void {
  fakeEntries.clear();
}
export function setFakeFile(path: string, content: string | Uint8Array): void {
  const data = typeof content === 'string' ? new TextEncoder().encode(content) : content;
  fakeEntries.set(path, { type: FileType.File, data });
}
export function setFakeUnreadable(path: string): void {
  fakeEntries.set(path, { type: FileType.File, data: new Uint8Array(0), unreadable: true });
}
export function setFakeSymlink(path: string, target: FileType = FileType.File): void {
  fakeEntries.set(path, { type: target | FileType.SymbolicLink, data: new Uint8Array(0) });
}
/** An explicit (possibly empty) directory. */
export function setFakeDir(path: string): void {
  fakeEntries.set(path, { type: FileType.Directory, data: new Uint8Array(0) });
}
export function deleteFakeFile(path: string): void {
  fakeEntries.delete(path);
}

function fakeNotFound(uri: Uri): Error {
  return Object.assign(new Error(`EntryNotFound (FileSystemError): ${uri.toString()}`), { code: 'FileNotFound' });
}

function fakeTypeOf(path: string): FileType | undefined {
  const entry = fakeEntries.get(path);
  if (entry) {
    return entry.type;
  }
  const prefix = path.endsWith('/') ? path : path + '/';
  for (const key of fakeEntries.keys()) {
    if (key.startsWith(prefix)) {
      return FileType.Directory;
    }
  }
  return undefined;
}

const fakeFs = {
  readFile: async (uri: Uri): Promise<Uint8Array> => {
    const entry = fakeEntries.get(uri.path);
    if (entry?.unreadable) {
      throw Object.assign(new Error(`NoPermissions (FileSystemError): ${uri.toString()}`), { code: 'NoPermissions' });
    }
    if (!entry || (entry.type & FileType.File) === 0) {
      throw fakeNotFound(uri);
    }
    return entry.data;
  },
  readDirectory: async (uri: Uri): Promise<Array<[string, FileType]>> => {
    if (fakeTypeOf(uri.path) !== FileType.Directory) {
      throw fakeNotFound(uri);
    }
    const prefix = uri.path.endsWith('/') ? uri.path : uri.path + '/';
    const children = new Map<string, FileType>();
    for (const key of fakeEntries.keys()) {
      if (key.startsWith(prefix)) {
        const name = key.slice(prefix.length).split('/')[0];
        children.set(name, fakeTypeOf(prefix + name)!);
      }
    }
    return [...children.entries()].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
  },
  stat: async (uri: Uri): Promise<{ type: FileType; ctime: number; mtime: number; size: number }> => {
    const type = fakeTypeOf(uri.path);
    if (type === undefined) {
      throw fakeNotFound(uri);
    }
    return { type, ctime: 0, mtime: 0, size: fakeEntries.get(uri.path)?.data.length ?? 0 };
  },
};

// --- file system watchers -----------------------------------------------------

export class MockFileSystemWatcher {
  readonly createEmitter = new EventEmitter<Uri>();
  readonly changeEmitter = new EventEmitter<Uri>();
  readonly deleteEmitter = new EventEmitter<Uri>();
  readonly onDidCreate = this.createEmitter.event;
  readonly onDidChange = this.changeEmitter.event;
  readonly onDidDelete = this.deleteEmitter.event;
  disposed = false;
  constructor(readonly globPattern: unknown) {}
  dispose(): void {
    this.disposed = true;
    this.createEmitter.dispose();
    this.changeEmitter.dispose();
    this.deleteEmitter.dispose();
  }
}

/** All watchers created via workspace.createFileSystemWatcher (in order). */
export const createdFileSystemWatchers: MockFileSystemWatcher[] = [];

/** Every line written to an output channel created via window.createOutputChannel. */
export const outputLines: Array<{ channel: string; message: string }> = [];

/** Names of output channels on which show() was called (in order). */
export const shownOutputChannels: string[] = [];

// --- quick pick / progress / document save -------------------------------------

/**
 * Programmable result for the NEXT window.showQuickPick call: the label of the
 * item the "author picked", or `undefined` to close the pick. Consumed once.
 */
export let nextQuickPickSelection: string | undefined = undefined;
export function setNextQuickPickSelection(label: string | undefined): void {
  nextQuickPickSelection = label;
}
/** Every showQuickPick call: the offered labels and the options passed. */
export const quickPickCalls: Array<{ items: string[]; options: unknown }> = [];

export enum ProgressLocation {
  SourceControl = 1,
  Window = 10,
  Notification = 15,
}
/** Options of every window.withProgress call (in order). */
export const progressCalls: unknown[] = [];

/** Every TextDocument.save() call on documents made by createTextDocument. */
export const savedDocuments: Array<{ path: string; ok: boolean }> = [];

/**
 * A fake open document for workspace.textDocuments. save() is recorded in
 * savedDocuments and resolves `saveResult` (default true; true also clears isDirty).
 */
export function createTextDocument(path: string, opts: { isDirty?: boolean; languageId?: string; saveResult?: boolean } = {}) {
  const uri = Uri.file(path);
  const doc = {
    uri,
    fileName: uri.fsPath,
    languageId: opts.languageId ?? 'pasta',
    isDirty: opts.isDirty ?? true,
    save: (): Promise<boolean> => {
      const ok = opts.saveResult ?? true;
      savedDocuments.push({ path: uri.path, ok });
      if (ok) {
        doc.isDirty = false;
      }
      return Promise.resolve(ok);
    },
  };
  return doc;
}

// --- namespaces ---------------------------------------------------------------

/** All diagnostic collections created via languages.createDiagnosticCollection (in order). */
export const createdDiagnosticCollections: MockDiagnosticCollection[] = [];

/** Providers registered via languages.registerDocumentSymbolProvider / registerCodeLensProvider (in order). */
export const registeredDocumentSymbolProviders: { selector: any; provider: any }[] = [];
export const registeredCodeLensProviders: { selector: any; provider: any }[] = [];

export const languages = {
  createDiagnosticCollection: (name: string) => {
    const collection = new MockDiagnosticCollection(name);
    createdDiagnosticCollections.push(collection);
    return collection;
  },
  registerDocumentSemanticTokensProvider: () => ({ dispose: () => {} }),
  registerDocumentSymbolProvider: (selector: any, provider: any) => {
    registeredDocumentSymbolProviders.push({ selector, provider });
    return { dispose: () => {} };
  },
  registerCodeLensProvider: (selector: any, provider: any) => {
    registeredCodeLensProviders.push({ selector, provider });
    return { dispose: () => {} };
  },
};

export const workspace = {
  textDocuments: [] as any[],
  workspaceFolders: undefined as any,
  onDidOpenTextDocument: didOpenTextDocumentEmitter.event,
  onDidChangeTextDocument: didChangeTextDocumentEmitter.event,
  onDidCloseTextDocument: didCloseTextDocumentEmitter.event,
  onDidSaveTextDocument: didSaveTextDocumentEmitter.event,
  fs: fakeFs,
  // shortcut: only documents already in textDocuments open (no load from disk); anything else rejects like a missing file
  openTextDocument: async (uri: Uri): Promise<any> => {
    const doc = workspace.textDocuments.find((d) => d.uri.path === uri.path);
    if (!doc) {
      throw fakeNotFound(uri);
    }
    return doc;
  },
  createFileSystemWatcher: (globPattern: unknown) => {
    const watcher = new MockFileSystemWatcher(globPattern);
    createdFileSystemWatchers.push(watcher);
    return watcher;
  },
};

export const window = {
  activeTextEditor: undefined as any,
  createOutputChannel: (name: string) => ({
    name,
    appendLine: (message: string) => { outputLines.push({ channel: name, message }); },
    show: (_preserveFocus?: boolean) => { shownOutputChannels.push(name); },
    dispose: () => {},
  }),
  showQuickPick: async <T extends string | { label: string }>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: unknown,
  ): Promise<T | undefined> => {
    const resolved = await items;
    const labels = resolved.map((i) => (typeof i === 'string' ? i : i.label));
    quickPickCalls.push({ items: labels, options });
    const selection = nextQuickPickSelection;
    nextQuickPickSelection = undefined;
    const index = selection === undefined ? -1 : labels.indexOf(selection);
    return index < 0 ? undefined : resolved[index];
  },
  withProgress: <R>(
    options: unknown,
    task: (progress: { report(value: { message?: string; increment?: number }): void }, token: unknown) => Thenable<R>,
  ): Thenable<R> => {
    progressCalls.push(options);
    return task({ report: () => {} }, { isCancellationRequested: false, onCancellationRequested: new EventEmitter<unknown>().event });
  },
  showErrorMessage: (message: string, ...items: string[]) => {
    shownMessages.push({ kind: 'error', message, items: items.length ? items : undefined });
    return Promise.resolve(undefined);
  },
  showWarningMessage: (message: string, ...items: string[]) => {
    shownMessages.push({ kind: 'warning', message, items: items.length ? items : undefined });
    // When the message offered action items, return the programmed selection
    // once (the label the "author clicked"); otherwise resolve undefined.
    if (items.length > 0) {
      const selection = nextWarningSelection;
      nextWarningSelection = undefined;
      return Promise.resolve(selection);
    }
    return Promise.resolve(undefined);
  },
  showInputBox: (options?: Record<string, unknown>) => {
    inputBoxCalls.push(options);
    return Promise.resolve(nextInputBoxResult);
  },
  createStatusBarItem: (_alignment?: StatusBarAlignment) => {
    const item = new MockStatusBarItem();
    createdStatusBarItems.push(item);
    return item;
  },
  createTextEditorDecorationType: (options: unknown) => new MockTextEditorDecorationType(options),
  onDidChangeActiveTextEditor: didChangeActiveTextEditorEmitter.event,
};

export const registeredCommands = new Map<string, (...args: any[]) => any>();

export const commands = {
  registerCommand: (id: string, fn: (...args: any[]) => any) => {
    registeredCommands.set(id, fn);
    return { dispose: () => { registeredCommands.delete(id); } };
  },
  executeCommand: (id: string, ...args: any[]) => {
    const fn = registeredCommands.get(id);
    return Promise.resolve(fn ? fn(...args) : undefined);
  },
};
