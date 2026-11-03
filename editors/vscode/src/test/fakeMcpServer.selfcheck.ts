// Self-check for the fake MCP endpoint (task 2.1). Run under plain node:
//   npx esbuild src/test/fakeMcpServer.selfcheck.ts --bundle --platform=node --format=cjs --outfile=out/test/fakeMcpServer.selfcheck.js && node out/test/fakeMcpServer.selfcheck.js

import * as assert from 'assert';
import * as fs from 'fs';
import * as http from 'http';
import * as os from 'os';
import * as path from 'path';
import { startFakeMcpServer } from './fakeMcpServer';

const ACCEPT = 'application/json, text/event-stream';

interface Reply { status: number; body: string }

function post(port: number, payload: unknown, headers: Record<string, string> = {}, timeoutMs = 2000): Promise<Reply | 'closed' | 'timeout'> {
  return new Promise((resolve) => {
    const req = http.request({
      host: '127.0.0.1', port, path: '/api/mcp/v1', method: 'POST', agent: false,
      headers: { 'Content-Type': 'application/json', Accept: ACCEPT, ...headers },
    }, (res) => {
      const chunks: Buffer[] = [];
      res.on('data', (c: Buffer) => chunks.push(c));
      res.on('end', () => resolve({ status: res.statusCode ?? 0, body: Buffer.concat(chunks).toString('utf8') }));
    });
    req.setTimeout(timeoutMs, () => { req.destroy(); resolve('timeout'); });
    req.on('error', () => resolve('closed'));
    req.end(JSON.stringify(payload));
  });
}

let id = 0;
const call = (name: string, args: Record<string, unknown>) =>
  ({ jsonrpc: '2.0', id: ++id, method: 'tools/call', params: { name, arguments: args } });
const text = (r: Reply | 'closed' | 'timeout') => {
  assert.ok(typeof r === 'object', `expected a reply, got ${String(r)}`);
  return JSON.parse(r.body).result.content[0].text as string;
};

async function main() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'fake-mcp-'));
  const root = path.join(tmp, 'ghostA');
  const master = path.join(root, 'ghost', 'master');
  const srv = await startFakeMcpServer({ basewareName: 'SSP', runningRoots: [root] });
  try {
    // initialize → serverInfo
    const init = await post(srv.port, { jsonrpc: '2.0', id: ++id, method: 'initialize', params: {} });
    assert.ok(typeof init === 'object' && init.status === 200);
    assert.ok(JSON.parse(init.body).result.serverInfo.name);

    // Accept lacking both types → 406; wrong Host → 403; wrong path → 404
    const r406 = await post(srv.port, call('get_property', {}), { Accept: 'application/json' });
    assert.ok(typeof r406 === 'object' && r406.status === 406);
    const r403 = await post(srv.port, call('get_property', {}), { Host: 'example.com' });
    assert.ok(typeof r403 === 'object' && r403.status === 403);

    // get_property returns name only for running roots (case/separator-insensitive)
    const alt = root.toUpperCase().replace(/\\/g, '/') + '/';
    assert.strictEqual(text(await post(srv.port, call('get_property', { property_name: 'baseware.name', ghost_name: alt }))), 'SSP');
    assert.match(text(await post(srv.port, call('get_property', { property_name: 'baseware.name', ghost_name: path.join(tmp, 'other') }))), /^NG:Cannot find active ghost/);

    // reload writes the marker into pasta.log
    srv.state.reloadLog = ['error', 'loadedFalse'];
    assert.match(text(await post(srv.port, call('reload', { target: 'shiori', ghost_name: root }))), /^OK:reload requested/);
    await new Promise((r) => setTimeout(r, 50));
    const log = fs.readFileSync(path.join(master, 'profile', 'pasta', 'logs', 'pasta.log'), 'utf8');
    assert.match(log, / ERROR /);
    assert.match(log, /SHIORI load entry completed entry="loadu" loaded=false\n$/);

    // raise_event: script / noScript / drop / silent
    const ev = (event: string) => post(srv.port, call('raise_event', { event, references: [], ghost_name: root }), {}, 300);
    srv.state.raiseEvent = { mode: 'script', script: '\\0hello\\e' };
    assert.strictEqual(text(await ev('OnPastaSceneAt')), 'OK:the ghost returned a script and it is being played.\r\nScript: \\0hello\\e');
    srv.state.raiseEvent = { mode: 'noScript' };
    assert.strictEqual(text(await ev('OnPastaSceneAt')), 'OK:the ghost returned no script.');
    srv.state.raiseEvent = { mode: 'drop', script: 'x' };
    assert.strictEqual(await ev('OnPastaSceneAt'), 'closed');
    srv.state.raiseEvent = { mode: 'silent' };
    assert.strictEqual(await ev('OnPastaSceneAt'), 'timeout');

    // OnPastaDumpVariables writes variables.toml only when asked
    const vars = path.join(master, 'profile', 'pasta', 'debug', 'variables.toml');
    srv.state.raiseEvent = { mode: 'noScript' };
    srv.state.writeVariables = false;
    await ev('OnPastaDumpVariables');
    assert.ok(!fs.existsSync(vars));
    srv.state.writeVariables = true;
    await ev('OnPastaDumpVariables');
    assert.strictEqual(fs.readFileSync(vars, 'utf8'), srv.state.variablesText);

    // get_log: sequential ids, since_id, max_count, continuation lines, empty
    assert.strictEqual(text(await post(srv.port, call('get_log', { log_type: 'error', ghost_name: root }))), '(no log entries)');
    const e1 = srv.addLog('error', 'Error', 'line1\nline2');
    const e2 = srv.addLog('error', 'Warning', 'w');
    assert.strictEqual(e2, e1 + 1);
    const all = text(await post(srv.port, call('get_log', { log_type: 'error', ghost_name: root })));
    assert.match(all, new RegExp(`^#${e1} \\d{4}/\\d\\d/\\d\\d \\d\\d:\\d\\d \\[Error\\] ghostA : line1\\n\\tline2\\n#${e2} `));
    assert.match(text(await post(srv.port, call('get_log', { log_type: 'error', ghost_name: root, max_count: 1 }))), new RegExp(`^#${e2} [^\\n]*$`));
    assert.strictEqual(text(await post(srv.port, call('get_log', { log_type: 'error', ghost_name: root, since_id: e2 }))), '(no log entries)');
    // the script played earlier (script + drop) were logged under Ghost:<event>
    assert.match(text(await post(srv.port, call('get_log', { log_type: 'script', ghost_name: root }))), /\[Ghost:OnPastaSceneAt\] ghostA : x$/);

    // get_log(script) with unescaped backslashes → body is not valid JSON
    srv.state.unescapedBackslashInScriptLog = true;
    const bad = await post(srv.port, call('get_log', { log_type: 'script', ghost_name: root }));
    assert.ok(typeof bad === 'object');
    assert.throws(() => JSON.parse(bad.body));
    assert.ok(bad.body.includes('\\0hello\\e'));
    srv.state.unescapedBackslashInScriptLog = false;

    // tools/call records and request headers
    assert.ok(srv.state.calls.some((c) => c.name === 'reload' && c.arguments.target === 'shiori'));
    assert.strictEqual(srv.state.requests.at(-1)?.accept, ACCEPT);
  } finally {
    await srv.close();
  }

  // accept-and-stay-silent server: no response head at all
  const quiet = await startFakeMcpServer({ silent: true });
  try {
    assert.strictEqual(await post(quiet.port, { jsonrpc: '2.0', id: 1, method: 'initialize', params: {} }, {}, 300), 'timeout');
  } finally {
    await quiet.close();
  }
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log('fakeMcpServer self-check: ok');
}

main().catch((e) => { console.error(e); process.exit(1); });
