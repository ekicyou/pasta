// Unit tests for the MCP client (task 3.1). Plain node against the fake endpoint:
//   npx esbuild src/test/mcpClient.test.ts --bundle --platform=node --format=cjs --outfile=out/test/mcpClient.test.js && node out/test/mcpClient.test.js
//
// Requirements: 2.2 (ghost_name is the root path), 3.9 (127.0.0.1 only), 7.3 (errors come back typed).

import * as assert from 'assert';
import * as http from 'http';
import * as os from 'os';
import * as path from 'path';
import { callTool, HOST, MCP_PATH, CALL_TIMEOUT_MS } from '../mcpClient';
import { startFakeMcpServer } from './fakeMcpServer';

let passed = 0;
let failed = 0;

async function test(name: string, fn: () => Promise<void>) {
  try {
    await fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e: any) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

/** A raw server that answers every request with `status` and `body`. */
async function rawServer(status: number, body: string): Promise<{ port: number; close: () => Promise<void> }> {
  const srv = http.createServer((req, res) => { req.resume(); req.on('end', () => res.writeHead(status).end(body)); });
  await new Promise<void>((r) => srv.listen(0, '127.0.0.1', r));
  return {
    port: (srv.address() as { port: number }).port,
    close: () => new Promise<void>((r) => { srv.closeAllConnections(); srv.close(() => r()); }),
  };
}

async function main() {
  console.log('\n[mcpClient Unit Tests]');
  const root = path.join(os.tmpdir(), 'mcp-client-ghost');
  const srv = await startFakeMcpServer({ basewareName: 'areka', runningRoots: [root] });
  try {
    await test('host is the constant 127.0.0.1, path /api/mcp/v1, call timeout 15 s', async () => {
      assert.strictEqual(HOST, '127.0.0.1');
      assert.strictEqual(MCP_PATH, '/api/mcp/v1');
      assert.strictEqual(CALL_TIMEOUT_MS, 15000);
    });

    await test('sends Content-Type and Accept (both types), no Origin / Mcp-Session-Id, no keep-alive', async () => {
      const before = srv.state.requests.length;
      await callTool(srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
      const h = srv.state.requests[before];
      assert.ok(h, 'request reached the server');
      assert.strictEqual(h['content-type'], 'application/json');
      assert.ok(String(h.accept).includes('application/json') && String(h.accept).includes('text/event-stream'));
      assert.strictEqual(h.origin, undefined);
      assert.strictEqual(h['mcp-session-id'], undefined);
      assert.strictEqual(h['mcp-protocol-version'], undefined);
      assert.strictEqual(h.connection, 'close');
      assert.strictEqual(h.host, `127.0.0.1:${srv.port}`);
    });

    await test('ok: tool text, with ghost_name passed through as the root path', async () => {
      const r = await callTool(srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
      assert.deepStrictEqual(r, { kind: 'ok', text: 'areka' });
      assert.strictEqual(srv.state.calls.at(-1)?.arguments.ghost_name, root);
    });

    await test('ng: isError true keeps the NG text', async () => {
      const r = await callTool(srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: path.join(root, 'other') });
      assert.deepStrictEqual(r, { kind: 'ng', text: 'NG:Cannot find active ghost from specified name' });
    });

    await test('rpcError: JSON-RPC error code and message', async () => {
      const r = await callTool(srv.port, 'no_such_tool' as any, { ghost_name: root } as any);
      assert.deepStrictEqual(r, { kind: 'rpcError', code: -32602, message: 'Unknown tool: no_such_tool' });
    });

    await test('unreachable: nothing listening on the port', async () => {
      const gone = await rawServer(200, '');
      const port = gone.port;
      await gone.close();
      const r = await callTool(port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
      assert.strictEqual(r.kind, 'unreachable', JSON.stringify(r));
    });

    await test('noResponse: connection dropped after the request was written (no retry)', async () => {
      srv.state.raiseEvent = { mode: 'drop', script: '\\0x\\e' };
      const calls = srv.state.calls.length;
      const r = await callTool(srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root });
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
      assert.strictEqual(srv.state.calls.length, calls + 1, 'sent exactly once');
    });

    await test('noResponse: 200 with an empty body', async () => {
      srv.state.raiseEvent = { mode: 'empty' };
      const r = await callTool(srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root });
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
    });

    await test('noResponse: timeout destroys the request and finishes', async () => {
      srv.state.raiseEvent = { mode: 'silent' };
      const t0 = Date.now();
      const r = await callTool(srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root }, 200);
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
      assert.ok(Date.now() - t0 < 2000, 'finished near the timeout');
    });

    await test('re-parse: get_log(script) with raw backslashes is read after fixing undefined escapes', async () => {
      srv.state.unescapedBackslashInScriptLog = true;
      srv.addLog('script', 'Ghost:OnPastaSceneAt', '\\0\\s[0]hello\\e', 'ghost');
      const r = await callTool(srv.port, 'get_log', { log_type: 'script', ghost_name: root, max_count: 1 });
      srv.state.unescapedBackslashInScriptLog = false;
      assert.strictEqual(r.kind, 'ok', JSON.stringify(r));
      assert.ok(r.kind === 'ok' && r.text.endsWith(': \\0\\s[0]hello\\e'), r.kind === 'ok' ? r.text : '');
    });

    for (const [label, status, body] of [
      ['non-200', 500, 'boom'],
      ['not JSON even after the escape fix', 200, '{"jsonrpc":"2.0",'],
      ['result without content text', 200, '{"jsonrpc":"2.0","id":1,"result":{"isError":false}}'],
      ['null result', 200, '{"jsonrpc":"2.0","id":1,"result":null}'],
    ] as const) {
      await test(`badShape: ${label}`, async () => {
        const raw = await rawServer(status, body);
        try {
          const r = await callTool(raw.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
          assert.strictEqual(r.kind, 'badShape', JSON.stringify(r));
        } finally {
          await raw.close();
        }
      });
    }
  } finally {
    await srv.close();
  }

  console.log(`\nmcpClient: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) process.exit(1);
}

main().catch((e) => { console.error(e); process.exit(1); });
