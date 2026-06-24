// Contribution-shape characterization tests for package.json (Task 6.4).
//
// These tests pin the *config* (command/menu) SHAPE that the behavior tests in
// 5.1/5.2 cannot reach: the "right-click top display (.pasta)" + "command
// registration / menu contribution" acceptance items. They read package.json
// directly (no vscode mock needed) and assert the contribution surface:
//   - contributes.commands has pasta.runSceneAtCursor + pasta.reloadShiori.
//   - editor/context: runSceneAtCursor ALWAYS shown for .pasta (NOT debug-gated)
//     and ordered first in the navigation group (top display) — R1.1 / R6.1.
//   - editor/context: reloadShiori gated on debugType == 'pasta' — R9.1.
//   - debug/toolBar: reloadShiori gated on debugType == 'pasta' — R9.1.
//   - the OLD pasta.debug.playScene is ABSENT from commands and all menus
//     (sole position-based 動線) — R5.1 / R5.2.
//
// Plain node test: no mock-vscode, no extension import. Mirrors the
// async-aware sequential runner used by the sibling command tests.

import * as assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';

const ROOT = path.resolve(__dirname, '..', '..');

interface Contribution {
  command: string;
  title?: string;
  group?: string;
  when?: string;
}
interface PackageJson {
  contributes: {
    commands: Contribution[];
    menus: Record<string, Contribution[]>;
  };
}

const pkg: PackageJson = JSON.parse(
  fs.readFileSync(path.join(ROOT, 'package.json'), 'utf8'),
);

const RUN_COMMAND = 'pasta.runSceneAtCursor';
const RELOAD_COMMAND = 'pasta.reloadShiori';
const OLD_COMMAND = 'pasta.debug.playScene';

// =============================================================================
// Sequential test runner (mirrors runSceneAtCursorCommand.test.ts)
// =============================================================================

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void }> = [];

function test(name: string, fn: () => void): void {
  queue.push({ name, fn });
}

function commandEntry(cmd: string): Contribution | undefined {
  return pkg.contributes.commands.find((c) => c.command === cmd);
}
function menu(id: string): Contribution[] {
  return pkg.contributes.menus[id] ?? [];
}
function menuEntry(id: string, cmd: string): Contribution | undefined {
  return menu(id).find((c) => c.command === cmd);
}

// =============================================================================
// contributes.commands
// =============================================================================

test('contributes.commands registers pasta.runSceneAtCursor with a title (R1.1)', () => {
  const e = commandEntry(RUN_COMMAND);
  assert.ok(e, 'pasta.runSceneAtCursor must be a registered command');
  assert.ok(typeof e!.title === 'string' && e!.title.length > 0, 'it must have a title');
});

test('contributes.commands registers pasta.reloadShiori with a title (R9.1)', () => {
  const e = commandEntry(RELOAD_COMMAND);
  assert.ok(e, 'pasta.reloadShiori must be a registered command');
  assert.ok(typeof e!.title === 'string' && e!.title.length > 0, 'it must have a title');
});

// =============================================================================
// editor/context — runSceneAtCursor: top display, ALWAYS shown for .pasta
// =============================================================================

test('editor/context shows runSceneAtCursor in the navigation group ordered first (top display) (R1.1)', () => {
  const e = menuEntry('editor/context', RUN_COMMAND);
  assert.ok(e, 'runSceneAtCursor must contribute to editor/context');
  assert.ok(
    typeof e!.group === 'string' && e!.group!.startsWith('navigation'),
    `group must start with "navigation" (top group); got ${JSON.stringify(e!.group)}`,
  );
  // Ordered first within navigation (e.g. navigation@1).
  assert.strictEqual(
    e!.group,
    'navigation@1',
    'must be navigation@1 — ordered first (top of the context menu)',
  );
});

test('editor/context runSceneAtCursor is shown for .pasta and is NOT debug-gated (always available) (R1.1/R6.1)', () => {
  const e = menuEntry('editor/context', RUN_COMMAND);
  assert.ok(e, 'runSceneAtCursor must contribute to editor/context');
  assert.ok(
    typeof e!.when === 'string' && e!.when!.includes('resourceLangId == pasta'),
    `when must require resourceLangId == pasta; got ${JSON.stringify(e!.when)}`,
  );
  assert.ok(
    !/debugType/.test(e!.when ?? ''),
    'runSceneAtCursor must NOT be debug-gated — it is always shown for .pasta (R6.1)',
  );
});

// =============================================================================
// editor/context + debug/toolBar — reloadShiori: connected-only (R9.1)
// =============================================================================

test('editor/context reloadShiori is gated on debugType == pasta (connected-only) (R9.1)', () => {
  const e = menuEntry('editor/context', RELOAD_COMMAND);
  assert.ok(e, 'reloadShiori must contribute to editor/context');
  assert.ok(
    typeof e!.when === 'string' && e!.when!.includes("debugType == 'pasta'"),
    `when must gate on debugType == 'pasta'; got ${JSON.stringify(e!.when)}`,
  );
});

test('debug/toolBar exposes reloadShiori gated on debugType == pasta (R9.1)', () => {
  const e = menuEntry('debug/toolBar', RELOAD_COMMAND);
  assert.ok(e, 'reloadShiori must contribute to debug/toolBar');
  assert.ok(
    typeof e!.when === 'string' && e!.when!.includes("debugType == 'pasta'"),
    `when must gate on debugType == 'pasta'; got ${JSON.stringify(e!.when)}`,
  );
});

// =============================================================================
// Old scene-name-input 動線 fully removed (R5.1 / R5.2)
// =============================================================================

test('old pasta.debug.playScene is ABSENT from contributes.commands (R5.1)', () => {
  assert.strictEqual(
    commandEntry(OLD_COMMAND),
    undefined,
    'pasta.debug.playScene must not be a registered command',
  );
});

test('old pasta.debug.playScene is ABSENT from every menu contribution (R5.1/R5.2)', () => {
  const offending: string[] = [];
  for (const [menuId, entries] of Object.entries(pkg.contributes.menus)) {
    if (entries.some((c) => c.command === OLD_COMMAND)) {
      offending.push(menuId);
    }
  }
  assert.deepStrictEqual(
    offending,
    [],
    `pasta.debug.playScene must not appear in any menu; found in: ${offending.join(', ')}`,
  );
});

// =============================================================================
// Runner
// =============================================================================

(() => {
  console.log('\n[package.json contribution-shape Tests]');
  for (const { name, fn } of queue) {
    try {
      fn();
      console.log(`  ✓ ${name}`);
      passed++;
    } catch (e: unknown) {
      console.error(`  ✗ ${name}`);
      console.error(`    ${e instanceof Error ? e.stack ?? e.message : String(e)}`);
      failed++;
    }
  }
  console.log(`\ncontributions: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
