// Test-only stand-in for the SSP / areka MCP endpoint (POST /api/mcp/v1).
// Wire shapes follow the real SSP 2.9.28 / areka answers recorded in
// .kiro/specs/vscode-mcp-client/research.md. Node built-ins only.

import * as fs from 'fs';
import * as http from 'http';
import * as path from 'path';

export type ReloadLogLine = 'loadedTrue' | 'loadedFalse' | 'error' | 'warn';
export type RaiseEventMode = 'script' | 'noScript' | 'drop' | 'silent';

export interface FakeLogRecord {
  readonly id: number;
  readonly type: 'error' | 'script';
  readonly label: string;
  readonly name: string;
  readonly text: string;
  readonly time: Date;
}

export interface FakeState {
  basewareName: string;
  /** Ghost roots that count as running (compared case- and separator-insensitively). */
  runningRoots: string[];
  /** Lines appended to pasta.log on reload, in order. [] writes nothing. */
  reloadLog: ReloadLogLine[];
  /** Delay before the reload lines are written (reload itself answers at once, like SSP). */
  reloadDelayMs: number;
  /** pasta.log path; default <ghost_name>/ghost/master/profile/pasta/logs/pasta.log. */
  pastaLogPath?: string;
  /** raise_event answer. script/drop also record a script log under `Ghost:<event>`. */
  raiseEvent: { mode: RaiseEventMode; script?: string };
  /** OnPastaDumpVariables writes <ghost_name>/ghost/master/profile/pasta/debug/variables.toml. */
  writeVariables: boolean;
  variablesText: string;
  /** Emit get_log(script) bodies with `\` left unescaped (the SSP bug). */
  unescapedBackslashInScriptLog: boolean;
  /** Accept connections and never send a response head. */
  silent: boolean;
  readonly logs: FakeLogRecord[];
  readonly calls: { name: string; arguments: Record<string, any> }[];
  readonly requests: http.IncomingHttpHeaders[];
}

export interface FakeMcpServer {
  readonly port: number;
  readonly state: FakeState;
  /** Adds a baseware log record and returns its id (ids are shared across log types). */
  addLog(type: 'error' | 'script', label: string, text: string, name?: string): number;
  close(): Promise<void>;
}

const LOOPBACK = /^(127\.0\.0\.1|localhost|\[::1\])(:\d+)?$/i;

const normRoot = (p: string) => p.replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();

const pad = (n: number) => String(n).padStart(2, '0');
const logTime = (d: Date) =>
  `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;

function pastaLogLine(kind: ReloadLogLine): string {
  const ts = new Date().toISOString();
  switch (kind) {
    case 'loadedTrue': return `${ts}  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=true`;
    case 'loadedFalse': return `${ts}  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=false`;
    case 'error': return `${ts} ERROR pasta_lua::loader: fake load error`;
    case 'warn': return `${ts}  WARN pasta_lua::loader::cache: fake load warning`;
  }
}

function append(file: string, lines: string[]) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.appendFileSync(file, lines.map((l) => l + '\n').join(''));
}

export async function startFakeMcpServer(init: Partial<FakeState> = {}): Promise<FakeMcpServer> {
  const state: FakeState = {
    basewareName: 'SSP',
    runningRoots: [],
    reloadLog: ['loadedTrue'],
    reloadDelayMs: 0,
    raiseEvent: { mode: 'script', script: '\\0\\s[0]fake script\\e' },
    writeVariables: true,
    variablesText: '[fake]\nvalue = 1\n',
    unescapedBackslashInScriptLog: false,
    silent: false,
    ...init,
    logs: [],
    calls: [],
    requests: [],
  };
  let nextId = 1;

  const addLog = (type: 'error' | 'script', label: string, text: string, name?: string) => {
    const id = nextId++;
    const ghost = name ?? (state.runningRoots[0] ? path.basename(state.runningRoots[0]) : 'ghost');
    state.logs.push({ id, type, label, name: ghost, text, time: new Date() });
    return id;
  };

  const isRunning = (ghost: unknown) =>
    typeof ghost === 'string' && state.runningRoots.some((r) => normRoot(r) === normRoot(ghost));
  const masterOf = (ghost: string) => path.join(ghost, 'ghost', 'master');

  type ToolAnswer = { text: string; isError?: boolean } | 'drop' | 'silent';

  function runTool(name: string, a: Record<string, any>): ToolAnswer | undefined {
    const ng = (text: string) => ({ text, isError: true });
    const notRunning = () => ng('NG:Cannot find active ghost from specified name');
    switch (name) {
      case 'get_property':
        if (a.property_name !== 'baseware.name') return ng('NG:Cannot find such property name.');
        if (a.ghost_name !== undefined && !isRunning(a.ghost_name)) return notRunning();
        return { text: state.basewareName };
      case 'get_log': {
        if (a.log_type !== 'error' && a.log_type !== 'script') return ng(`NG:Unknown log_type ${String(a.log_type)}`);
        let recs = state.logs.filter((r) => r.type === a.log_type && r.id > (a.since_id ?? 0));
        if (typeof a.max_count === 'number') recs = recs.slice(-a.max_count); // newest N, kept oldest-first
        if (recs.length === 0) return { text: '(no log entries)' };
        return {
          text: recs
            .map((r) => `#${r.id} ${logTime(r.time)} [${r.label}] ${r.name} : ${r.text.split('\n').join('\n\t')}`)
            .join('\n'),
        };
      }
      case 'reload': {
        if (a.target !== 'shiori') return ng(`NG:Unknown target ${String(a.target)}`);
        if (!isRunning(a.ghost_name)) return notRunning();
        const lines = state.reloadLog.map(pastaLogLine);
        const file = state.pastaLogPath ?? path.join(masterOf(a.ghost_name), 'profile', 'pasta', 'logs', 'pasta.log');
        if (lines.length > 0) setTimeout(() => append(file, lines), state.reloadDelayMs);
        return {
          text: 'OK:reload requested. It runs asynchronously; errors while loading (dictionary errors etc.) will be recorded to the error log. Wait until get_status answers again, then check get_log tool (log_type=error, since_id=0)',
        };
      }
      case 'raise_event': {
        if (!isRunning(a.ghost_name)) return notRunning();
        if (a.event === 'OnPastaDumpVariables' && state.writeVariables) {
          const file = path.join(masterOf(a.ghost_name), 'profile', 'pasta', 'debug', 'variables.toml');
          fs.mkdirSync(path.dirname(file), { recursive: true });
          fs.writeFileSync(file, state.variablesText);
        }
        const { mode, script = '' } = state.raiseEvent;
        if (mode === 'script' || mode === 'drop') addLog('script', `Ghost:${a.event}`, script, path.basename(a.ghost_name));
        if (mode === 'drop' || mode === 'silent') return mode;
        if (mode === 'noScript') return { text: 'OK:the ghost returned no script.' };
        return { text: `OK:the ghost returned a script and it is being played.\r\nScript: ${script}` };
      }
      default:
        return undefined;
    }
  }

  const server = http.createServer((req, res) => {
    if (state.silent) return; // keep the socket open, never answer
    state.requests.push(req.headers);
    if (req.method !== 'POST' || req.url !== '/api/mcp/v1') { res.writeHead(404).end(); return; }
    if (!LOOPBACK.test(req.headers.host ?? '')) { res.writeHead(403).end(); return; }
    if (req.headers.origin !== undefined && !/^https?:\/\/(127\.0\.0\.1|localhost|\[::1\])(:\d+)?$/i.test(req.headers.origin)) {
      res.writeHead(403).end();
      return;
    }
    const accept = req.headers.accept ?? '';
    if (!accept.includes('application/json') || !accept.includes('text/event-stream')) { res.writeHead(406).end(); return; }

    const chunks: Buffer[] = [];
    req.on('data', (c: Buffer) => chunks.push(c));
    req.on('end', () => {
      let msg: any;
      try { msg = JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch {
        send({ jsonrpc: '2.0', id: null, error: { code: -32700, message: 'Parse error' } });
        return;
      }
      if (msg.id === undefined) { res.writeHead(202).end(); return; } // notification
      const reply = (result: unknown) => send({ jsonrpc: '2.0', id: msg.id, result });
      const fail = (code: number, message: string) => send({ jsonrpc: '2.0', id: msg.id, error: { code, message } });

      if (msg.method === 'initialize') {
        reply({ protocolVersion: '2025-06-18', capabilities: { tools: {} }, serverInfo: { name: 'fake-mcp-server', version: '0' } });
        return;
      }
      if (msg.method !== 'tools/call') { fail(-32601, `Method not found: ${String(msg.method)}`); return; }
      const name = String(msg.params?.name);
      const args = (msg.params?.arguments ?? {}) as Record<string, any>;
      state.calls.push({ name, arguments: args });
      const answer = runTool(name, args);
      if (answer === undefined) { fail(-32602, `Unknown tool: ${name}`); return; }
      if (answer === 'silent') return;
      if (answer === 'drop') { req.socket.destroy(); return; }
      const body = { content: [{ type: 'text', text: answer.text }], isError: answer.isError ?? false };
      if (name === 'get_log' && args.log_type === 'script' && state.unescapedBackslashInScriptLog) {
        // stringify with a placeholder, then put the raw `\` back so the JSON is invalid like SSP's
        const marked = { ...body, content: [{ type: 'text', text: answer.text.replace(/\\/g, '') }] };
        sendRaw(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result: marked }).replace(//g, '\\'));
        return;
      }
      reply(body);
    });

    function send(obj: unknown) { sendRaw(JSON.stringify(obj)); }
    function sendRaw(json: string) {
      res.writeHead(200, { 'Content-Type': 'application/json', Connection: 'close' }).end(json);
    }
  });

  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const port = (server.address() as { port: number }).port;

  return {
    port,
    state,
    addLog,
    close: () => new Promise<void>((resolve) => {
      server.closeAllConnections();
      server.close(() => resolve());
    }),
  };
}
