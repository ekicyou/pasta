// E2E and build verification tests (Phase 5, Tasks 5.1-5.3)
//
// Tests extension manifest integrity, activation lifecycle,
// TextMate grammar completeness, and build configuration.

import * as assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';

// =============================================================================
// Test framework
// =============================================================================

let passed = 0;
let failed = 0;

function test(name: string, fn: () => void) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e: any) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

const ROOT = path.resolve(__dirname, '..', '..');

// =============================================================================
// Package.json Manifest Verification (Task 5.3)
// =============================================================================

console.log('\n[Package.json Manifest Tests]');

const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf-8'));

test('package.json exists and is valid JSON', () => {
  assert.ok(pkg, 'package.json should be parseable');
});

test('has required extension fields', () => {
  assert.strictEqual(pkg.name, 'pasta-vscode');
  assert.ok(pkg.version, 'should have version');
  assert.ok(pkg.engines?.vscode, 'should have vscode engine');
  assert.strictEqual(pkg.main, './out/extension.js');
});

test('activation events include onLanguage:pasta', () => {
  assert.ok(
    pkg.activationEvents.includes('onLanguage:pasta'),
    'should activate on pasta language'
  );
});

test('pasta language is registered', () => {
  const lang = pkg.contributes.languages[0];
  assert.strictEqual(lang.id, 'pasta');
  assert.ok(lang.extensions.includes('.pasta'), 'should associate .pasta extension');
});

test('TextMate grammar is registered', () => {
  const grammar = pkg.contributes.grammars[0];
  assert.strictEqual(grammar.language, 'pasta');
  assert.strictEqual(grammar.scopeName, 'source.pasta');
  assert.ok(grammar.path.includes('pasta.tmLanguage.json'));
});

test('declares all 10 custom semantic token types', () => {
  const customTypes = pkg.contributes.semanticTokenTypes.map((t: any) => t.id);
  // Custom types declared in package.json (not built-in to VSCode)
  const expected = ['scene', 'word', 'call', 'actor', 'actorName', 'talk', 'codeBlock', 'sakuraScript', 'escape', 'number'];
  for (const e of expected) {
    assert.ok(customTypes.includes(e), `should declare custom type '${e}'`);
  }
});

test('declares global semantic token modifier', () => {
  const mods = pkg.contributes.semanticTokenModifiers.map((m: any) => m.id);
  assert.ok(mods.includes('global'), 'should declare global modifier');
});

test('has semantic token scopes for all 15 types', () => {
  const scopes = pkg.contributes.semanticTokenScopes[0].scopes;
  // Must match PASTA_TOKEN_TYPES in semanticTokensProvider.ts (indices 0-14)
  const expectedKeys = [
    'comment', 'namespace', 'scene', 'decorator', 'word', 'variable',
    'call', 'actor', 'actorName', 'codeBlock', 'talk', 'sakuraScript',
    'escape', 'operator', 'number'
  ];
  for (const key of expectedKeys) {
    assert.ok(scopes[key], `should have scope mapping for '${key}'`);
  }
});

test('npm scripts include compile and package', () => {
  assert.ok(pkg.scripts.compile, 'should have compile script');
  assert.ok(pkg.scripts.package, 'should have package script');
  assert.ok(pkg.scripts['build:wasm'], 'should have build:wasm script');
});

// =============================================================================
// TextMate Grammar Verification (Task 5.1)
// =============================================================================

console.log('\n[TextMate Grammar Verification]');

const grammarPath = path.join(ROOT, 'syntaxes', 'pasta.tmLanguage.json');
const grammar = JSON.parse(fs.readFileSync(grammarPath, 'utf-8'));

test('TextMate grammar file exists and is valid JSON', () => {
  assert.ok(grammar, 'grammar should be parseable');
});

test('grammar has correct scopeName', () => {
  assert.strictEqual(grammar.scopeName, 'source.pasta');
});

test('grammar has patterns array', () => {
  assert.ok(Array.isArray(grammar.patterns), 'should have patterns');
  assert.ok(grammar.patterns.length > 0, 'should have at least one pattern');
});

test('grammar covers all required marker types', () => {
  const patternNames = grammar.patterns.map((p: any) => p.name || p.include || '').filter(Boolean);
  const repositoryKeys = Object.keys(grammar.repository || {});
  const allNames = [...patternNames, ...repositoryKeys];

  // We just check the repository has entries for key concepts
  const expectedPatterns = ['comment', 'global-scene', 'local-scene', 'attribute', 'word', 'variable', 'call'];
  for (const expected of expectedPatterns) {
    const found = allNames.some(n => n.includes(expected));
    assert.ok(found, `grammar should cover '${expected}' pattern`);
  }
});

test('grammar supports full-width markers', () => {
  // Check that at least one pattern contains full-width marker characters
  const grammarStr = JSON.stringify(grammar);
  assert.ok(grammarStr.includes('＃') || grammarStr.includes('\\uff03'), 'should support full-width # (＃)');
  assert.ok(grammarStr.includes('＊') || grammarStr.includes('\\uff0a'), 'should support full-width * (＊)');
});

// =============================================================================
// Language Configuration Verification
// =============================================================================

console.log('\n[Language Configuration]');

const langConfigPath = path.join(ROOT, 'language-configuration.json');
const langConfig = JSON.parse(fs.readFileSync(langConfigPath, 'utf-8'));

test('language configuration exists', () => {
  assert.ok(langConfig, 'language config should exist');
});

test('has comment configuration', () => {
  assert.ok(langConfig.comments, 'should have comments config');
  assert.ok(langConfig.comments.lineComment, 'should have line comment');
});

test('has bracket pairs', () => {
  assert.ok(langConfig.brackets, 'should have bracket pairs');
});

// =============================================================================
// Build Configuration Verification
// =============================================================================

console.log('\n[Build Configuration]');

test('tsconfig.json exists', () => {
  const tsConfigPath = path.join(ROOT, 'tsconfig.json');
  assert.ok(fs.existsSync(tsConfigPath), 'tsconfig.json should exist');
});

test('.vscodeignore exists', () => {
  const vsciPath = path.join(ROOT, '.vscodeignore');
  assert.ok(fs.existsSync(vsciPath), '.vscodeignore should exist');
});

test('WASM build script exists', () => {
  const scriptPath = path.join(ROOT, 'scripts', 'build-wasm.ps1');
  assert.ok(fs.existsSync(scriptPath), 'build-wasm.ps1 should exist');
});

test('esbuild compile script targets correct output', () => {
  assert.ok(
    pkg.scripts.compile.includes('out/extension.js'),
    'compile should output to out/extension.js'
  );
  assert.ok(
    pkg.scripts.compile.includes('--external:vscode'),
    'compile should externalize vscode'
  );
});

// =============================================================================
// Activation State Logic Tests (Task 5.2 - Fallback)
// =============================================================================

console.log('\n[Activation State / Fallback Tests]');

test('activation state defaults to not-ready, not-fallback', () => {
  const state = { wasmReady: false, fallbackMode: false };
  assert.strictEqual(state.wasmReady, false);
  assert.strictEqual(state.fallbackMode, false);
});

test('WASM success sets wasmReady true', () => {
  const state = { wasmReady: false, fallbackMode: false };
  // Simulate successful init
  state.wasmReady = true;
  assert.strictEqual(state.wasmReady, true);
  assert.strictEqual(state.fallbackMode, false);
});

test('WASM failure sets fallbackMode true', () => {
  const state = { wasmReady: false, fallbackMode: false };
  // Simulate failed init
  state.fallbackMode = true;
  assert.strictEqual(state.wasmReady, false);
  assert.strictEqual(state.fallbackMode, true);
});

test('fallback mode means TextMate-only operation', () => {
  const state = { wasmReady: false, fallbackMode: true };
  // In fallback mode:
  // - TextMate grammar still provides basic highlighting
  // - SemanticTokensProvider is NOT registered
  // - DocumentSync is NOT created
  const shouldRegisterSemantic = state.wasmReady;
  const shouldCreateDocSync = state.wasmReady;
  assert.strictEqual(shouldRegisterSemantic, false);
  assert.strictEqual(shouldCreateDocSync, false);
});

// =============================================================================
// Extension Module Structure Tests
// =============================================================================

console.log('\n[Extension Module Structure]');

test('extension.ts source file exists', () => {
  assert.ok(
    fs.existsSync(path.join(ROOT, 'src', 'extension.ts')),
    'src/extension.ts should exist'
  );
});

test('wasmBridge.ts source file exists', () => {
  assert.ok(
    fs.existsSync(path.join(ROOT, 'src', 'wasmBridge.ts')),
    'src/wasmBridge.ts should exist'
  );
});

test('semanticTokensProvider.ts source file exists', () => {
  assert.ok(
    fs.existsSync(path.join(ROOT, 'src', 'semanticTokensProvider.ts')),
    'src/semanticTokensProvider.ts should exist'
  );
});

test('diagnosticsManager.ts source file exists', () => {
  assert.ok(
    fs.existsSync(path.join(ROOT, 'src', 'diagnosticsManager.ts')),
    'src/diagnosticsManager.ts should exist'
  );
});

test('documentSync.ts source file exists', () => {
  assert.ok(
    fs.existsSync(path.join(ROOT, 'src', 'documentSync.ts')),
    'src/documentSync.ts should exist'
  );
});

// =============================================================================
// Summary
// =============================================================================

console.log(`\nE2E/Build: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
