// MCP client for the SSP / areka endpoint (POST http://127.0.0.1:{port}/api/mcp/v1).
// Pure node: no vscode import. Wire shapes: .kiro/specs/vscode-mcp-client/research.md.

import * as http from 'http';

/** The only destination host. There is no setting to change host or port (requirement 3.9). */
export const HOST = '127.0.0.1';
export const MCP_PATH = '/api/mcp/v1';
export const CALL_TIMEOUT_MS = 15000;

export interface ToolArgsMap {
  get_property: { property_name: 'baseware.name'; ghost_name: string };
  get_log: { log_type: 'error' | 'script'; ghost_name: string; since_id?: number; max_count?: number };
  reload: { target: 'shiori'; ghost_name: string };
  raise_event: {
    event: 'OnPastaSceneAt' | 'OnPastaDumpVariables';
    references: readonly string[];
    ghost_name: string;
  };
}
export type ToolName = keyof ToolArgsMap;

export type CallResult =
  | { kind: 'ok'; text: string }                         // isError: false (empty text included)
  | { kind: 'ng'; text: string }                         // isError: true (NG:…)
  | { kind: 'rpcError'; code: number; message: string }  // JSON-RPC error
  | { kind: 'unreachable'; detail: string }              // could not connect before the request was written
  | { kind: 'noResponse'; detail: string }               // dropped after writing, empty body, or timeout
  | { kind: 'badShape'; detail: string };                // non-200, not JSON, or wrong result shape

let nextId = 1;

/** The parts of a JSON-RPC answer this client reads. */
interface RpcMessage {
  error?: { code?: unknown; message?: unknown };
  result?: { content?: { text?: unknown }[]; isError?: unknown };
}
type RpcOutcome = CallResult | { kind: 'result'; result: NonNullable<RpcMessage['result']> };

/** Turns `\x` escapes JSON does not define into `\\x` (SSP 2.9.05–2.9.07 get_log(script) leaves `\` raw). */
const fixEscapes = (json: string) =>
  json.replace(/\\([\s\S])/g, (m, c: string) => ('"\\/bfnrtu'.includes(c) ? m : '\\\\' + c));

function parseLenient(body: string): unknown {
  try {
    return JSON.parse(body);
  } catch {
    return JSON.parse(fixEscapes(body)); // throws again when still broken
  }
}

/** One JSON-RPC request. Never throws; never retries (a replay would play a script twice). */
function rpc(port: number, method: string, params: unknown, timeoutMs: number): Promise<RpcOutcome> {
  return new Promise((resolve) => {
    let done = false;
    const finish = (r: RpcOutcome) => { if (!done) { done = true; resolve(r); } };
    let written = false;

    const body = JSON.stringify({ jsonrpc: '2.0', id: nextId++, method, params });
    const req = http.request({
      host: HOST, port, path: MCP_PATH, method: 'POST',
      agent: false, // no connection reuse
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json, text/event-stream', // areka answers 406 without both
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      const chunks: Buffer[] = [];
      res.on('data', (c: Buffer) => chunks.push(c));
      res.on('error', (e) => finish({ kind: 'noResponse', detail: e.message }));
      res.on('end', () => {
        const text = Buffer.concat(chunks).toString('utf8');
        if (res.statusCode !== 200) { finish({ kind: 'badShape', detail: `HTTP ${res.statusCode}: ${text}` }); return; }
        if (text.trim() === '') { finish({ kind: 'noResponse', detail: 'empty response body' }); return; }
        let msg: RpcMessage | null;
        try { msg = parseLenient(text) as RpcMessage | null; } catch (e) { finish({ kind: 'badShape', detail: `invalid JSON: ${(e as Error).message}` }); return; }
        if (msg?.error && typeof msg.error.code === 'number') {
          finish({ kind: 'rpcError', code: msg.error.code, message: String(msg.error.message ?? '') });
          return;
        }
        if (msg?.result == null || typeof msg.result !== 'object') { finish({ kind: 'badShape', detail: `no result: ${text}` }); return; }
        finish({ kind: 'result', result: msg.result });
      });
    });
    req.on('finish', () => { written = true; });
    req.setTimeout(timeoutMs, () => {
      finish({ kind: 'noResponse', detail: `no response within ${timeoutMs} ms` });
      req.destroy();
    });
    req.on('error', (e) => finish(written
      ? { kind: 'noResponse', detail: e.message }
      : { kind: 'unreachable', detail: e.message }));
    req.end(body);
  });
}

export async function callTool<N extends ToolName>(
  port: number, name: N, args: ToolArgsMap[N], timeoutMs = CALL_TIMEOUT_MS,
): Promise<CallResult> {
  const r = await rpc(port, 'tools/call', { name, arguments: args }, timeoutMs);
  if (r.kind !== 'result') return r;
  const text = r.result.content?.[0]?.text;
  if (typeof text !== 'string') return { kind: 'badShape', detail: `unexpected result: ${JSON.stringify(r.result)}` };
  return r.result.isError === true ? { kind: 'ng', text } : { kind: 'ok', text };
}
