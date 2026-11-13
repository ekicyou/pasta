// Unit tests for the MCP client (task 3.1). Plain node against the fake endpoint:
//   npx esbuild src/test/mcpClient.test.ts --bundle --platform=node --format=cjs --outfile=out/test/mcpClient.test.js && node out/test/mcpClient.test.js
//
// Requirements: 2.2 (ghost_name is the root path), 3.9 (127.0.0.1 only), 7.3 (errors come back typed),
// 3.1 / 3.2 / 3.6 / 3.8 (finding and re-checking the baseware, task 3.2),
// 4.7 / 4.8 (reading get_log and the raise_event answer, task 3.3).

import * as assert from 'assert';
import * as http from 'http';
import * as os from 'os';
import * as path from 'path';
import {
  callTool, HOST, MCP_PATH, CALL_TIMEOUT_MS, CANDIDATE_PORTS, SCAN_TIMEOUT_MS, probe, identify, findBasewares,
  parseLogEntries, lastLogId, readRaiseEvent,
} from '../../../shared/mcpClient.ts';
import { nodeGhostIO as io } from '../nodeGhostIO';
import { createFakeGhostIO } from '../../../shared/fakeGhostIO.ts';
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
      await callTool(io, srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
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
      const r = await callTool(io, srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
      assert.deepStrictEqual(r, { kind: 'ok', text: 'areka' });
      assert.strictEqual(srv.state.calls.at(-1)?.arguments.ghost_name, root);
    });

    await test('ng: isError true keeps the NG text', async () => {
      const r = await callTool(io, srv.port, 'get_property', { property_name: 'baseware.name', ghost_name: path.join(root, 'other') });
      assert.deepStrictEqual(r, { kind: 'ng', text: 'NG:Cannot find active ghost from specified name' });
    });

    await test('rpcError: JSON-RPC error code and message', async () => {
      const r = await callTool(io, srv.port, 'no_such_tool' as any, { ghost_name: root } as any);
      assert.deepStrictEqual(r, { kind: 'rpcError', code: -32602, message: 'Unknown tool: no_such_tool' });
    });

    await test('unreachable: nothing listening on the port', async () => {
      const gone = await rawServer(200, '');
      const port = gone.port;
      await gone.close();
      const r = await callTool(io, port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
      assert.strictEqual(r.kind, 'unreachable', JSON.stringify(r));
    });

    await test('noResponse: connection dropped after the request was written (no retry)', async () => {
      srv.state.raiseEvent = { mode: 'drop', script: '\\0x\\e' };
      const calls = srv.state.calls.length;
      const r = await callTool(io, srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root });
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
      assert.strictEqual(srv.state.calls.length, calls + 1, 'sent exactly once');
    });

    await test('noResponse: 200 with an empty body', async () => {
      srv.state.raiseEvent = { mode: 'empty' };
      const r = await callTool(io, srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root });
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
    });

    await test('noResponse: timeout destroys the request and finishes', async () => {
      srv.state.raiseEvent = { mode: 'silent' };
      const t0 = Date.now();
      const r = await callTool(io, srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: root }, 200);
      assert.strictEqual(r.kind, 'noResponse', JSON.stringify(r));
      assert.ok(Date.now() - t0 < 2000, 'finished near the timeout');
    });

    await test('re-parse: get_log(script) with raw backslashes is read after fixing undefined escapes', async () => {
      srv.state.unescapedBackslashInScriptLog = true;
      srv.addLog('script', 'Ghost:OnPastaSceneAt', '\\0\\s[0]hello\\e', 'ghost');
      const r = await callTool(io, srv.port, 'get_log', { log_type: 'script', ghost_name: root, max_count: 1 });
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
          const r = await callTool(io, raw.port, 'get_property', { property_name: 'baseware.name', ghost_name: root });
          assert.strictEqual(r.kind, 'badShape', JSON.stringify(r));
        } finally {
          await raw.close();
        }
      });
    }
  } finally {
    await srv.close();
  }

  await discoveryTests(root);
  await logAndEventTests(root);

  console.log(`\nmcpClient: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) process.exit(1);
}

async function discoveryTests(root: string) {
  await test('candidate ports: 9801, 9821, 9802, 9822 … 9810, 9830; scan cut-off 2 s', async () => {
    assert.strictEqual(CANDIDATE_PORTS.length, 20);
    assert.deepStrictEqual(CANDIDATE_PORTS.slice(0, 4), [9801, 9821, 9802, 9822]);
    assert.deepStrictEqual(CANDIDATE_PORTS.slice(-2), [9810, 9830]);
    assert.strictEqual(SCAN_TIMEOUT_MS, 2000);
  });

  await test('probe: initialize names the sender from the io (the extension stays pasta-vscode)', async () => {
    assert.strictEqual(io.clientName, 'pasta-vscode');
    const fake = createFakeGhostIO('pasta-recipe');
    const ok = (result: unknown) => ({ kind: 'response', status: 200, body: JSON.stringify({ jsonrpc: '2.0', id: 1, result }) } as const);
    fake.queuePost(ok({}), ok({ content: [{ text: 'SSP' }] }));
    assert.deepStrictEqual(await probe(fake, 9801, root), { port: 9801, basewareName: 'SSP' });
    assert.strictEqual(JSON.parse(fake.posts[0].body).params.clientInfo.name, 'pasta-recipe');
  });

  const ssp = await startFakeMcpServer({ basewareName: 'SSP', runningRoots: [root] });
  const areka = await startFakeMcpServer({ basewareName: 'areka', runningRoots: [root] });
  const sspTwin = await startFakeMcpServer({ basewareName: 'SSP', runningRoots: [root] });
  const silent = await startFakeMcpServer({ basewareName: 'SSP', runningRoots: [root], silent: true });
  const idle = await startFakeMcpServer({ basewareName: 'areka', runningRoots: [] });
  try {
    await test('probe: initialize then get_property(baseware.name, ghost_name=root)', async () => {
      const req = ssp.state.requests.length;
      const calls = ssp.state.calls.length;
      assert.deepStrictEqual(await probe(io, ssp.port, root), { port: ssp.port, basewareName: 'SSP' });
      assert.strictEqual(ssp.state.requests.length, req + 2, 'two round trips');
      assert.deepStrictEqual(ssp.state.calls.slice(calls), [
        { name: 'get_property', arguments: { property_name: 'baseware.name', ghost_name: root } },
      ]);
    });

    await test('identify: a single get_property, no initialize', async () => {
      const req = areka.state.requests.length;
      assert.deepStrictEqual(await identify(io, areka.port, root), { port: areka.port, basewareName: 'areka' });
      assert.strictEqual(areka.state.requests.length, req + 1);
    });

    await test('a root that is not running there is no endpoint (probe, identify, scan)', async () => {
      assert.strictEqual(await probe(io, idle.port, root), undefined);
      assert.strictEqual(await identify(io, idle.port, root), undefined);
      assert.deepStrictEqual(await findBasewares(io, root, [idle.port], 500), []);
    });

    await test('an empty baseware.name is no endpoint', async () => {
      ssp.state.basewareName = '';
      try {
        assert.strictEqual(await identify(io, ssp.port, root), undefined);
      } finally {
        ssp.state.basewareName = 'SSP';
      }
    });

    await test('scan keeps candidate order and skips silent / closed / idle ports within the cut-off', async () => {
      const gone = await rawServer(200, '');
      await gone.close();
      const t0 = Date.now();
      const found = await findBasewares(io, root, [silent.port, areka.port, gone.port, idle.port, ssp.port], 300);
      const took = Date.now() - t0;
      assert.deepStrictEqual(found, [
        { port: areka.port, basewareName: 'areka' },
        { port: ssp.port, basewareName: 'SSP' },
      ]);
      assert.ok(took < 1000, `finished near the 300 ms cut-off (took ${took} ms)`);
    });

    await test('scan merges endpoints with the same baseware name, keeping the first port', async () => {
      const found = await findBasewares(io, root, [sspTwin.port, areka.port, ssp.port], 500);
      assert.deepStrictEqual(found, [
        { port: sspTwin.port, basewareName: 'SSP' },
        { port: areka.port, basewareName: 'areka' },
      ]);
    });
  } finally {
    await Promise.all([ssp, areka, sspTwin, silent, idle].map((s) => s.close()));
  }
}

async function logAndEventTests(root: string) {
  await test('parseLogEntries: (no log entries) is []', async () => {
    assert.deepStrictEqual(parseLogEntries('(no log entries)'), []);
  });

  await test('trailing newline is not kept in the last record or the script', async () => {
    assert.deepStrictEqual(parseLogEntries('#1 2026/11/03 10:00 [Ghost:x] g : a\r\n').map((x) => x.text), ['a']);
    assert.deepStrictEqual(readRaiseEvent({ kind: 'ok', text: 'OK:played.\r\nScript: \\0hi\\e\r\n' }), { kind: 'script', script: '\\0hi\\e' });
  });

  await test('parseLogEntries: header fields, tab continuation lines, CRLF, " : " inside the text', async () => {
    const text = '#3 2026/11/03 10:29 [Ghost:OnPastaSceneAt] えも2 DEBUG : \\0first\r\n\tsecond\n\tthird\n#7 2026/11/03 10:30 [SHIORI] emo : x : y';
    assert.deepStrictEqual(parseLogEntries(text), [
      { id: 3, label: 'Ghost:OnPastaSceneAt', name: 'えも2 DEBUG', text: '\\0first\nsecond\nthird' },
      { id: 7, label: 'SHIORI', name: 'emo', text: 'x : y' },
    ]);
  });

  const srv = await startFakeMcpServer({ runningRoots: [root] });
  const raise = (ghost = root) =>
    callTool(io, srv.port, 'raise_event', { event: 'OnPastaSceneAt', references: ['a.pasta', '1'], ghost_name: ghost });
  try {
    await test('lastLogId: 0 with no records, else the newest id of that type (max_count=1)', async () => {
      assert.strictEqual(await lastLogId(io, srv.port, 'script', root), 0);
      srv.addLog('script', 'Ghost:OnBoot', 'a\nb');
      const e = srv.addLog('error', 'SHIORI', 'err');
      const s = srv.addLog('script', 'Ghost:OnPastaSceneAt', 'c');
      assert.strictEqual(await lastLogId(io, srv.port, 'script', root), s);
      assert.strictEqual(await lastLogId(io, srv.port, 'error', root), e);
      assert.deepStrictEqual(srv.state.calls[srv.state.calls.length - 1].arguments,
        { log_type: 'error', ghost_name: root, max_count: 1 });
    });

    await test('lastLogId: a failed call comes back as the CallResult', async () => {
      const gone = await rawServer(200, '');
      await gone.close();
      const r = await lastLogId(io, gone.port, 'error', root);
      assert.ok(typeof r !== 'number' && r.kind === 'unreachable', JSON.stringify(r));
      const odd = await rawServer(200, JSON.stringify({ jsonrpc: '2.0', id: 1, result: { content: [{ type: 'text', text: 'garbage' }] } }));
      try {
        const b = await lastLogId(io, odd.port, 'error', root);
        assert.ok(typeof b !== 'number' && b.kind === 'badShape', JSON.stringify(b));
      } finally {
        await odd.close();
      }
    });

    await test('parseLogEntries reads multi-line records from the fake get_log', async () => {
      const r = await callTool(io, srv.port, 'get_log', { log_type: 'script', ghost_name: root });
      assert.ok(r.kind === 'ok', JSON.stringify(r));
      assert.deepStrictEqual(parseLogEntries(r.text).map((x) => x.text), ['a\nb', 'c']);
    });

    await test('readRaiseEvent: script (text after the Script: line)', async () => {
      srv.state.raiseEvent = { mode: 'script', script: '\\0\\s[0]hi\\e' };
      assert.deepStrictEqual(readRaiseEvent(await raise()), { kind: 'script', script: '\\0\\s[0]hi\\e' });
    });

    await test('readRaiseEvent: no script', async () => {
      srv.state.raiseEvent = { mode: 'noScript' };
      assert.deepStrictEqual(readRaiseEvent(await raise()), { kind: 'noScript' });
    });

    for (const mode of ['drop', 'empty'] as const) {
      await test(`readRaiseEvent: no response (${mode})`, async () => {
        srv.state.raiseEvent = { mode };
        assert.strictEqual(readRaiseEvent(await raise()).kind, 'noResponse');
      });
    }

    await test('readRaiseEvent: "No response from server" / "did not respond within" in ok or ng are no response', async () => {
      assert.strictEqual(readRaiseEvent({ kind: 'ok', text: 'No response from server' }).kind, 'noResponse');
      assert.strictEqual(readRaiseEvent({ kind: 'ng', text: 'NG:areka did not respond within 10 seconds' }).kind, 'noResponse');
    });

    await test('readRaiseEvent: failure keeps the CallResult', async () => {
      const r = await raise(path.join(root, 'x'));
      assert.deepStrictEqual(readRaiseEvent(r), { kind: 'failed', result: r });
      const u = { kind: 'unreachable', detail: 'ECONNREFUSED' } as const;
      assert.deepStrictEqual(readRaiseEvent(u), { kind: 'failed', result: u });
      const odd = { kind: 'ok', text: 'OK:something else' } as const;
      assert.deepStrictEqual(readRaiseEvent(odd), { kind: 'failed', result: odd });
    });
  } finally {
    await srv.close();
  }
}

main().catch((e) => { console.error(e); process.exit(1); });
