// TextMate Grammar Tests for Pasta DSL
//
// Validates that the TextMate grammar correctly tokenizes
// all Pasta DSL syntax elements with both full-width and half-width markers.

import * as fs from 'fs';
import * as path from 'path';
import * as vsctm from 'vscode-textmate';
import * as oniguruma from 'vscode-oniguruma';

// ============================================================================
// Test Infrastructure
// ============================================================================

const WASM_PATH = path.join(
  path.dirname(require.resolve('vscode-oniguruma')),
  'onig.wasm'
);

let registry: vsctm.Registry;
let grammar: vsctm.IGrammar;

async function initGrammar(): Promise<vsctm.IGrammar> {
  const wasmBin = fs.readFileSync(WASM_PATH).buffer;
  const vscodeOnigurumaLib = oniguruma.loadWASM({
    data: wasmBin,
    print: () => {},
  }).then(() => {
    return {
      createOnigScanner(patterns: string[]) {
        return new oniguruma.OnigScanner(patterns);
      },
      createOnigString(s: string) {
        return new oniguruma.OnigString(s);
      },
    };
  });

  registry = new vsctm.Registry({
    onigLib: vscodeOnigurumaLib,
    loadGrammar: async (scopeName: string) => {
      if (scopeName === 'source.pasta') {
        const grammarPath = path.resolve(__dirname, '..', '..', 'syntaxes', 'pasta.tmLanguage.json');
        const content = fs.readFileSync(grammarPath, 'utf-8');
        return vsctm.parseRawGrammar(content, grammarPath);
      }
      return null;
    },
  });

  const g = await registry.loadGrammar('source.pasta');
  if (!g) {
    throw new Error('Failed to load pasta grammar');
  }
  return g;
}

function tokenizeLine(line: string, prevState?: vsctm.StateStack): vsctm.ITokenizeLineResult {
  return grammar.tokenizeLine(line, prevState ?? vsctm.INITIAL);
}

function hasScope(tokens: vsctm.IToken[], scope: string): boolean {
  return tokens.some((t) => t.scopes.includes(scope));
}

function findTokenWithScope(tokens: vsctm.IToken[], scope: string): vsctm.IToken | undefined {
  return tokens.find((t) => t.scopes.includes(scope));
}

// ============================================================================
// Test Runner
// ============================================================================

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

const results: TestResult[] = [];

function test(name: string, fn: () => void): void {
  try {
    fn();
    results.push({ name, passed: true });
  } catch (e) {
    results.push({ name, passed: false, error: String(e) });
  }
}

function expect(value: boolean, message: string): void {
  if (!value) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

// ============================================================================
// Tests
// ============================================================================

async function runTests(): Promise<void> {
  grammar = await initGrammar();

  // --- Comment Tests ---
  test('全角コメントマーカーの認識', () => {
    const result = tokenizeLine('＃全角コメント');
    expect(hasScope(result.tokens, 'comment.line.pasta'), 'should have comment.line.pasta scope');
  });

  test('半角コメントマーカーの認識', () => {
    const result = tokenizeLine('#半角コメント');
    expect(hasScope(result.tokens, 'comment.line.pasta'), 'should have comment.line.pasta scope');
  });

  // --- Global Scene Tests ---
  test('全角グローバルシーンマーカーの認識（空白あり）', () => {
    const result = tokenizeLine('＊ 挨拶');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('全角グローバルシーンマーカーの認識（空白なし）', () => {
    const result = tokenizeLine('＊メイン');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope for ＊メイン');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('半角グローバルシーンマーカーの認識', () => {
    const result = tokenizeLine('* greeting');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope');
  });

  // --- Local Scene Tests ---
  test('全角ローカルシーンマーカーの認識（空白あり）', () => {
    const result = tokenizeLine('  ・ 次の会話');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('全角ローカルシーンマーカーの認識（空白なし）', () => {
    const result = tokenizeLine('　・グローバル単語呼び出し');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope for ・直後テキスト');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('半角ローカルシーンマーカーの認識', () => {
    const result = tokenizeLine('  - next_scene');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope');
  });

  // --- Attribute Tests ---
  test('全角属性マーカーの認識', () => {
    const result = tokenizeLine('  ＆priority：10');
    expect(hasScope(result.tokens, 'entity.other.attribute-name.pasta'), 'should have attribute scope');
  });

  test('半角属性マーカーの認識', () => {
    const result = tokenizeLine('  &priority:10');
    expect(hasScope(result.tokens, 'entity.other.attribute-name.pasta'), 'should have attribute scope');
  });

  // --- Word Tests ---
  test('全角単語マーカーの認識', () => {
    const result = tokenizeLine('  ＠greeting：こんにちは おはよう');
    expect(hasScope(result.tokens, 'string.key.word.pasta'), 'should have string.key.word scope');
  });

  test('半角単語マーカーの認識', () => {
    const result = tokenizeLine('  @greeting:hello hi');
    expect(hasScope(result.tokens, 'string.key.word.pasta'), 'should have string.key.word scope');
  });

  // --- Variable Tests ---
  test('全角変数マーカーの認識', () => {
    const result = tokenizeLine('  ＄count：1');
    expect(hasScope(result.tokens, 'variable.other.pasta'), 'should have variable scope');
  });

  test('半角変数マーカーの認識', () => {
    const result = tokenizeLine('  $count:1');
    expect(hasScope(result.tokens, 'variable.other.pasta'), 'should have variable scope');
  });

  // --- Call Tests ---
  test('全角Callマーカーの認識', () => {
    const result = tokenizeLine('  ＞次の会話');
    expect(hasScope(result.tokens, 'keyword.control.pasta'), 'should have keyword scope');
  });

  test('半角Callマーカーの認識', () => {
    const result = tokenizeLine('  >next_scene');
    expect(hasScope(result.tokens, 'keyword.control.pasta'), 'should have keyword scope');
  });

  // --- Actor Tests ---
  test('全角アクターマーカーの認識', () => {
    const result = tokenizeLine('％Alice');
    expect(hasScope(result.tokens, 'entity.name.class.pasta'), 'should have class scope');
  });

  test('半角アクターマーカーの認識', () => {
    const result = tokenizeLine('%Bob');
    expect(hasScope(result.tokens, 'entity.name.class.pasta'), 'should have class scope');
  });

  test('インデント付きアクターマーカーの認識（シーン内使用宣言）', () => {
    const result = tokenizeLine('\u3000％さくら、うにゅう、ぱすた');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
    expect(hasScope(result.tokens, 'entity.name.class.pasta'), 'should have class scope for actor names');
  });

  // --- Lua Code Block Tests ---
  test('Luaコードブロック開始の認識', () => {
    const result = tokenizeLine('```lua');
    expect(
      hasScope(result.tokens, 'meta.embedded.block.lua') ||
      hasScope(result.tokens, 'punctuation.definition.code.begin.pasta') ||
      result.tokens.some((t) => t.scopes.some((s) => s.includes('meta.embedded'))),
      'should have lua block or begin punctuation scope'
    );
  });

  test('Luaコードブロック内容の認識', () => {
    const open = tokenizeLine('```lua');
    const content = tokenizeLine('print("hello")', open.ruleStack);
    // Inside a lua block, should be within meta.embedded.block.lua scope
    // Note: source.lua grammar is not loaded in test env, so we check for meta.embedded
    expect(
      content.tokens.some((t) => t.scopes.some((s) =>
        s.includes('meta.embedded') || s.includes('source.lua')
      )),
      'should be inside lua block (meta.embedded or source.lua scope)'
    );
  });

  // --- Action Line Tests ---
  test('アクション行のアクター名認識', () => {
    const result = tokenizeLine('  Alice：こんにちは');
    expect(
      hasScope(result.tokens, 'entity.name.type.actor.pasta'),
      'should have actor name scope'
    );
  });

  test('アクション行のコロン区切り認識', () => {
    const result = tokenizeLine('  Alice：こんにちは');
    expect(
      hasScope(result.tokens, 'punctuation.separator.pasta'),
      'should have separator scope'
    );
  });

  test('アクター名とコロン間の空白許容（全角スペース）', () => {
    const result = tokenizeLine('　　　さくら　：＠笑顔　＠挨拶！');
    expect(
      hasScope(result.tokens, 'entity.name.type.actor.pasta'),
      'should have actor name scope even with fullwidth space before colon'
    );
    const actorToken = findTokenWithScope(result.tokens, 'entity.name.type.actor.pasta');
    expect(!!actorToken, 'actor token must exist');
    const text = '　　　さくら　：＠笑顔　＠挨拶！';
    const actorName = text.substring(actorToken!.startIndex, actorToken!.endIndex);
    expect(actorName === 'さくら', `actor name should be "さくら" but got "${actorName}"`);
  });

  test('アクター名とコロン間の空白許容（半角スペース）', () => {
    const result = tokenizeLine('  sakura ：hello');
    expect(
      hasScope(result.tokens, 'entity.name.type.actor.pasta'),
      'should have actor name scope with half-width space before colon'
    );
  });

  test('アクター名とコロン間に空白なしも引き続き動作', () => {
    const result = tokenizeLine('　　　うにゅう：＠通常　やふぅ。');
    expect(
      hasScope(result.tokens, 'entity.name.type.actor.pasta'),
      'should have actor name scope without space before colon'
    );
  });

  // --- Results ---
  console.log('\n=== TextMate Grammar Test Results ===\n');
  let passed = 0;
  let failed = 0;
  for (const r of results) {
    if (r.passed) {
      console.log(`  ✅ ${r.name}`);
      passed++;
    } else {
      console.log(`  ❌ ${r.name}: ${r.error}`);
      failed++;
    }
  }
  console.log(`\n  Total: ${results.length} | Passed: ${passed} | Failed: ${failed}\n`);

  if (failed > 0) {
    process.exit(1);
  }
}

runTests().catch((err) => {
  console.error('Test runner failed:', err);
  process.exit(1);
});
