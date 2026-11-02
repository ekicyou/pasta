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

// ============================================================================
// Injection-aware Test Infrastructure (task 1.2)
//
// The existing single-grammar path above (initGrammar / tokenizeLine) is kept
// intact for the existing tests. The wiring below adds a SECOND, injection-aware
// registry so that a ```lua block body actually receives source.lua scopes via
// a TextMate injection grammar — the prerequisite for tasks 2.1/2.2 to validate
// real Lua colouring.
//
// vscode-textmate v9 only consults an injection grammar's `injectionSelector`
// once the registry knows the injection EXISTS for the host scope. That linkage
// is supplied through `RegistryOptions.getInjections(scopeName)`. Without it the
// injection is never exercised and the block body stays
// `meta.embedded.block.lua.content` with NO source.lua scopes (the RED state).
// ============================================================================

const PASTA_SCOPE = 'source.pasta';
const INJECTION_SCOPE = 'pasta-lua.injection';
const LUA_SCOPE = 'source.lua';

// Vendored Lua fixture from task 1.1 (read-only). Registered as `source.lua`
// to stand in for VS Code's built-in Lua grammar, which is absent in the
// esbuild+node test environment.
// At runtime __dirname is the bundled `out/test`; the fixture lives in the source
// tree under `src/test/fixtures`, so resolve from the editors/vscode root.
const LUA_FIXTURE_PATH = path.resolve(__dirname, '..', '..', 'src', 'test', 'fixtures', 'lua.tmLanguage.json');

// Resolve the REAL shipped injection grammar so the tests exercise the actual
// grammar that ships in the VSIX, not an inline stand-in.
//
// Task 2.1 created `editors/vscode/syntaxes/pasta-lua-injection.tmLanguage.json`
// (scopeName `pasta-lua.injection`, injectionSelector
// `L:meta.embedded.block.lua.content`, include source.lua). At runtime __dirname is
// the esbuild bundle dir `out/test`, so resolve from the editors/vscode root the
// same way the SSOT / fixture grammars are resolved above.
const INJECTION_GRAMMAR_PATH = path.resolve(__dirname, '..', '..', 'syntaxes', 'pasta-lua-injection.tmLanguage.json');

// Single resolution point for where the injection grammar comes from: the real
// shipped file. parsed with the same parseRawGrammar path-resolution pattern.
function loadInjectionGrammar(): vsctm.IRawGrammar {
  const content = fs.readFileSync(INJECTION_GRAMMAR_PATH, 'utf-8');
  return vsctm.parseRawGrammar(content, INJECTION_GRAMMAR_PATH);
}

let injectionRegistry: vsctm.Registry;
let injectedPastaGrammar: vsctm.IGrammar;

async function initInjectedGrammar(): Promise<vsctm.IGrammar> {
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

  injectionRegistry = new vsctm.Registry({
    onigLib: vscodeOnigurumaLib,
    // Wire the injection so vscode-textmate actually exercises it for source.pasta.
    getInjections: (scopeName: string) => {
      if (scopeName === PASTA_SCOPE) {
        return [INJECTION_SCOPE];
      }
      return undefined;
    },
    loadGrammar: async (scopeName: string) => {
      if (scopeName === PASTA_SCOPE) {
        const grammarPath = path.resolve(__dirname, '..', '..', 'syntaxes', 'pasta.tmLanguage.json');
        const content = fs.readFileSync(grammarPath, 'utf-8');
        return vsctm.parseRawGrammar(content, grammarPath);
      }
      if (scopeName === INJECTION_SCOPE) {
        return loadInjectionGrammar();
      }
      if (scopeName === LUA_SCOPE) {
        const content = fs.readFileSync(LUA_FIXTURE_PATH, 'utf-8');
        return vsctm.parseRawGrammar(content, LUA_FIXTURE_PATH);
      }
      return null;
    },
  });

  const g = await injectionRegistry.loadGrammar(PASTA_SCOPE);
  if (!g) {
    throw new Error('Failed to load injected pasta grammar');
  }
  return g;
}

// Tokenize multiple lines through the injection-aware grammar, threading the
// rule stack so block state (inside/outside a ```lua fence) is preserved.
// Returns the flattened token list of every line, each tagged with its line index.
interface InjectedToken extends vsctm.IToken {
  line: number;
}

function tokenizeLinesInjected(lines: string[]): InjectedToken[] {
  let ruleStack = vsctm.INITIAL;
  const all: InjectedToken[] = [];
  for (let i = 0; i < lines.length; i++) {
    const result = injectedPastaGrammar.tokenizeLine(lines[i], ruleStack);
    for (const t of result.tokens) {
      all.push({ ...t, line: i });
    }
    ruleStack = result.ruleStack;
  }
  return all;
}

// True iff a token carries a scope produced by the injected Lua grammar.
// The pasta host scope `meta.embedded.block.lua` / `.content` also ends in
// `.lua`/`.content`, so those must be excluded — they are present with OR
// without injection and would mask the contrast.
function hasLuaScope(tokens: vsctm.IToken[]): boolean {
  return tokens.some((t) =>
    t.scopes.some(
      (s) =>
        (s === LUA_SCOPE || s.endsWith('.lua')) &&
        !s.startsWith('meta.embedded.block.lua')
    )
  );
}

// True when every character of `text` (first occurrence at/after `from`) in
// `line` carries `scope`, and the characters just outside it do not — i.e.
// `scope` covers exactly that range.
function scopeCoversExactly(line: string, text: string, scope: string, from = 0): boolean {
  const start = line.indexOf(text, from);
  if (start < 0) return false;
  const end = start + text.length;
  const tokens = tokenizeLine(line).tokens;
  const has = (i: number) =>
    tokens.some((t) => t.startIndex <= i && i < t.endIndex && t.scopes.includes(scope));
  for (let i = start; i < end; i++) {
    if (!has(i)) return false;
  }
  return !(start > 0 && has(start - 1)) && !(end < line.length && has(end));
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
  // 文法は印と名前の間に空白を許さない（`＊ 挨拶` は誤りの行）。名前の色は付けない。
  test('全角グローバルシーンマーカーの認識（空白あり）', () => {
    const result = tokenizeLine('＊ 挨拶');
    expect(!hasScope(result.tokens, 'keyword.control.scene.pasta'), 'name after a space is not a scene name');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('全角グローバルシーンマーカーの認識（空白なし）', () => {
    const result = tokenizeLine('＊メイン');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope for ＊メイン');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('半角グローバルシーンマーカーの認識', () => {
    const result = tokenizeLine('*greeting');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope');
  });

  // --- Local Scene Tests ---
  test('全角ローカルシーンマーカーの認識（空白あり）', () => {
    const result = tokenizeLine('  ・ 次の会話');
    expect(!hasScope(result.tokens, 'keyword.control.scene.pasta'), 'name after a space is not a scene name');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('全角ローカルシーンマーカーの認識（空白なし）', () => {
    const result = tokenizeLine('　・グローバル単語呼び出し');
    expect(hasScope(result.tokens, 'keyword.control.scene.pasta'), 'should have keyword.control.scene.pasta scope for ・直後テキスト');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('半角ローカルシーンマーカーの認識', () => {
    const result = tokenizeLine('  -next_scene');
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
  // NOTE: commit 0ccf429 (word-ref box decoration) intentionally changed the
  // word definition/reference scope from string.key.word.pasta /
  // entity.name.function.reference.pasta to markup.inline.raw.string.pasta.
  // These expectations follow the grammar's current committed behaviour.
  test('全角単語マーカーの認識', () => {
    const result = tokenizeLine('  ＠greeting：こんにちは おはよう');
    expect(hasScope(result.tokens, 'markup.inline.raw.string.pasta'), 'should have markup.inline.raw.string scope');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('半角単語マーカーの認識', () => {
    const result = tokenizeLine('  @greeting:hello hi');
    expect(hasScope(result.tokens, 'markup.inline.raw.string.pasta'), 'should have markup.inline.raw.string scope');
    expect(hasScope(result.tokens, 'keyword.other.marker.pasta'), 'marker should have keyword.other.marker.pasta scope');
  });

  test('アクション行内の単語参照の認識（inline-word-ref）', () => {
    const result = tokenizeLine('　さくら：＠笑顔　こんにちは');
    expect(
      hasScope(result.tokens, 'markup.inline.raw.string.pasta'),
      'inline word reference should have markup.inline.raw.string scope'
    );
  });

  // --- Dynamic Word Reference Tests (dynamic-word-reference 6.1, 6.2, 7.10) ---
  const WORD_SCOPE = 'markup.inline.raw.string.pasta';
  const VAR_REF_SCOPE = 'variable.other.reference.pasta';

  test('動的単語参照 ＠＄x の ＠ を含む範囲が単語参照スコープ', () => {
    expect(scopeCoversExactly('　さくら：＠＄x　です', '＠＄x', WORD_SCOPE), '＠＄x should be one word-ref range');
  });

  test('動的単語参照 ＠＄＊x の ＠ を含む範囲が単語参照スコープ', () => {
    expect(scopeCoversExactly('　さくら：＠＄＊x　です', '＠＄＊x', WORD_SCOPE), '＠＄＊x should be one word-ref range');
  });

  test('動的関数呼び出し ＠＄f（１） は名前の ＠＄f だけが単語参照スコープ', () => {
    expect(scopeCoversExactly('　さくら：＠＄f（１）　です', '＠＄f', WORD_SCOPE), 'only ＠＄f should be word-ref');
  });

  test('動的単語参照の全角半角混在（@$x・＠$＊x）が単語参照スコープ', () => {
    expect(scopeCoversExactly('　さくら：@$x　です', '@$x', WORD_SCOPE), '@$x should be one word-ref range');
    expect(scopeCoversExactly('　さくら：＠$＊x　です', '＠$＊x', WORD_SCOPE), '＠$＊x should be one word-ref range');
  });

  test('エスケープ＋動的単語参照 ＠＠＠＄x は ＠＄x だけが単語参照スコープ', () => {
    const line = '　さくら：＠＠＠＄x';
    expect(scopeCoversExactly(line, '＠＄x', WORD_SCOPE, line.indexOf('＠＠＠') + 2), '＠＄x after escape should be word-ref');
  });

  test('既存: 静的単語参照 ＠笑顔 の範囲は不変', () => {
    expect(scopeCoversExactly('　さくら：＠笑顔　こんにちは', '＠笑顔', WORD_SCOPE), '＠笑顔 should stay one word-ref range');
  });

  test('既存: 変数参照 ＄x・＄＊x の範囲は不変', () => {
    expect(scopeCoversExactly('　さくら：＄x　です', '＄x', VAR_REF_SCOPE), '＄x should stay variable reference');
    expect(scopeCoversExactly('　さくら：＄＊x　です', '＄＊x', VAR_REF_SCOPE), '＄＊x should stay variable reference');
  });

  test('既存: エスケープ ＠＠＄x はエスケープ＋変数参照のまま', () => {
    const line = '　さくら：＠＠＄x';
    expect(scopeCoversExactly(line, '＄x', VAR_REF_SCOPE), '＄x should stay variable reference');
    expect(!hasScope(tokenizeLine(line).tokens, WORD_SCOPE), '＠＠＄x must not get word-ref scope');
  });

  // --- 台詞の中の単語参照・エスケープ・変数参照の範囲（ponytail-editor-tooling 5.2・5.4〜5.6） ---
  const SAKURA_SCOPE = 'string.other.sakura-script.pasta';
  const ESCAPE_SCOPE = 'constant.character.escape.pasta';

  test('単語参照は ＠ と名前の文字が続く所まで（＠挨拶。こんにちは）', () => {
    expect(scopeCoversExactly('　さくら：＠挨拶。こんにちは', '＠挨拶', WORD_SCOPE), 'only ＠挨拶 is word-ref');
    expect(scopeCoversExactly('　さくら：＠関数（１）です', '＠関数', WORD_SCOPE), 'function call: name only');
    expect(scopeCoversExactly('　さくら：＠＊関数（）です', '＠＊関数', WORD_SCOPE), 'global function call: name only');
    expect(scopeCoversExactly('　さくら：@greet!', '@greet', WORD_SCOPE), 'half-width word-ref');
    // 文法の XID_Continue（Unicode 15.1 以降）は ・ を含む。Oniguruma の表は古いので名前の文字に足してある
    expect(scopeCoversExactly('　さくら：＠ほげ・ふが。', '＠ほげ・ふが', WORD_SCOPE), '・ is a name character');
  });

  test('エスケープ ＠＠・＄＄ の後ろの文字を単語参照・変数参照にしない', () => {
    const at = '　さくら：＠＠メール';
    expect(scopeCoversExactly(at, '＠＠', ESCAPE_SCOPE), '＠＠ is escape');
    expect(!hasScope(tokenizeLine(at).tokens, WORD_SCOPE), '＠＠メール has no word-ref');
    const at3 = '　さくら：＠＠＠メール';
    expect(scopeCoversExactly(at3, '＠＠', ESCAPE_SCOPE), '＠＠＠メール: leading ＠＠ is escape');
    expect(scopeCoversExactly(at3, '＠メール', WORD_SCOPE, at3.indexOf('＠＠＠') + 2), '＠＠＠メール: only ＠メール is word-ref');
    const dollar = '　さくら：＄＄x';
    expect(scopeCoversExactly(dollar, '＄＄', ESCAPE_SCOPE), '＄＄ is escape');
    expect(!hasScope(tokenizeLine(dollar).tokens, VAR_REF_SCOPE), '＄＄x has no variable reference');
    expect(scopeCoversExactly('　さくら：a＠@b', '＠@', ESCAPE_SCOPE), 'mixed-width ＠@ is escape');
  });

  test('文法のエスケープ（\\\\・\\%）だけをエスケープの色にする', () => {
    expect(scopeCoversExactly('　さくら：a\\\\b', '\\\\', ESCAPE_SCOPE), '\\\\ is escape');
    for (const ch of ['＠', '@', '＄', '$', '＞', '>']) {
      const line = `　さくら：a\\${ch}b`;
      expect(!hasScope(tokenizeLine(line).tokens, ESCAPE_SCOPE), `\\${ch} must not be escape`);
    }
  });

  test('動的な単語参照・変数参照の名前は文法の名前（id・数字・＊id・プロパティ）', () => {
    expect(scopeCoversExactly('　さくら：＠＄１です', '＠＄１', WORD_SCOPE), '＠＄１ digit var_id');
    expect(scopeCoversExactly('　さくら：＠＄x。です', '＠＄x', WORD_SCOPE), '＠＄x stops at 。');
    expect(scopeCoversExactly('　さくら：＄x。です', '＄x', VAR_REF_SCOPE), '＄x stops at 。');
    expect(scopeCoversExactly('　さくら：＄２番', '＄２', VAR_REF_SCOPE), '＄２ digit var_id');
    expect(scopeCoversExactly('　さくら：＄＊g、です', '＄＊g', VAR_REF_SCOPE), '＄＊g global');
    expect(scopeCoversExactly('　さくら：＄％user.name です', '＄％user.name', VAR_REF_SCOPE), '＄％ property');
  });

  test('継続行の台詞にアクション行と同じ色を付ける', () => {
    const c = '　：＠挨拶。＄x\\n＠＠';
    expect(scopeCoversExactly(c, '：', 'punctuation.separator.pasta'), 'continuation colon');
    expect(scopeCoversExactly(c, '＠挨拶', WORD_SCOPE), 'continuation word-ref');
    expect(scopeCoversExactly(c, '＄x', VAR_REF_SCOPE), 'continuation var-ref');
    expect(scopeCoversExactly(c, '\\n', SAKURA_SCOPE), 'continuation sakura-script');
    expect(scopeCoversExactly(c, '＠＠', ESCAPE_SCOPE), 'continuation escape');
    expect(hasScope(tokenizeLine('  :hello').tokens, 'meta.talk.pasta'), 'half-width continuation talk');
  });

  test('決して当たらない規則 inline-fn-call を持たない', () => {
    const json = JSON.parse(fs.readFileSync(path.resolve(__dirname, '..', '..', 'syntaxes', 'pasta.tmLanguage.json'), 'utf-8'));
    expect(!('inline-fn-call' in json.repository), 'inline-fn-call must be removed');
  });

  // --- Sakura Script / Escape Tests (paragraph-break-tag-only-talk 10.6) ---

  test('さくらスクリプト \\nHello は \\n だけがタグ', () => {
    expect(scopeCoversExactly('　さくら：\\nHello', '\\n', SAKURA_SCOPE), '\\n only should be sakura-script');
  });

  test('エスケープ \\% はエスケープスコープ', () => {
    expect(scopeCoversExactly('　さくら：a\\%b', '\\%', ESCAPE_SCOPE), '\\% should be escape');
  });

  test('囲み \\_?＠w\\_? は全体がさくらスクリプト', () => {
    const line = '　さくら：a\\_?＠w\\_?b';
    expect(scopeCoversExactly(line, '\\_?＠w\\_?', SAKURA_SCOPE), 'literal region should be sakura-script');
    expect(!hasScope(tokenizeLine(line).tokens, WORD_SCOPE), '＠w inside literal region must not be word-ref');
  });

  test('引数の中の \\] を含む \\s[a\\]b] は 1 つのトークン', () => {
    const line = '　さくら：\\s[a\\]b]x';
    const text = '\\s[a\\]b]';
    const start = line.indexOf(text);
    const end = start + text.length;
    expect(
      tokenizeLine(line).tokens.some(
        (t) => t.startIndex === start && t.endIndex === end && t.scopes.includes(SAKURA_SCOPE)
      ),
      '\\s[a\\]b] should be one sakura-script token'
    );
  });

  test('既存: 代入行の右辺の動的参照は行全体の変数スコープのまま', () => {
    const line = '＄x＝＠＄y';
    expect(scopeCoversExactly(line, 'x＝＠＄y', 'variable.other.pasta'), 'assignment body stays variable.other.pasta');
    expect(!hasScope(tokenizeLine(line).tokens, WORD_SCOPE), 'assignment line must not get inline word-ref scope');
  });

  // --- String Concat Operator Tests (string-concat-operator 7.6, 7.7) ---
  // The grammar has no operator scopes and is NOT changed: the expression `＆`
  // must get exactly the scopes `＋` gets in the same position, and the
  // line-head attribute `＆` / talk-line `＆` keep their pre-existing scopes.
  const scopesAt = (line: string, ch: string): string[] => {
    const i = line.indexOf(ch);
    const t = tokenizeLine(line).tokens.find((tok) => tok.startIndex <= i && i < tok.endIndex);
    return t ? t.scopes : [];
  };
  const sameScopes = (a: string[], b: string[]) => a.join(' ') === b.join(' ');

  test('式の ＆ は同じ位置の ＋ と同じスコープ（全角・半角）', () => {
    for (const [cat, plus, op, pl] of [
      ['＄x＝「a」＆＄b', '＄x＝１＋＄b', '＆', '＋'],
      ['$x=「a」&$b', '$x=1+$b', '&', '+'],
      ['＞「挨拶」＆＄種類', '＞「挨拶」＋＄種類', '＆', '＋'],
    ]) {
      const c = scopesAt(cat, op);
      expect(c.length > 0 && sameScopes(c, scopesAt(plus, pl)), `${cat}: ${c.join(' ')}`);
      expect(!c.includes('meta.attribute.pasta'), `${cat}: ＆ must not be shown as attribute marker`);
    }
  });

  test('行頭の属性マーカー ＆ の表示は不変（全角・半角）', () => {
    const attr = ['source.pasta', 'meta.attribute.pasta', 'keyword.other.marker.pasta'];
    expect(sameScopes(scopesAt('  ＆author：Test', '＆'), attr), 'full-width attribute marker');
    expect(sameScopes(scopesAt('  &author:Test', '&'), attr), 'half-width attribute marker');
  });

  test('台詞の中の ＆ の表示は不変（全角・半角）', () => {
    const talk = ['source.pasta', 'meta.talk.pasta'];
    expect(sameScopes(scopesAt('　さくら：A＆B', '＆'), talk), 'full-width talk ＆');
    expect(sameScopes(scopesAt('　さくら：A&B', '&'), talk), 'half-width talk &');
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

  // --- 宣言・属性・アクター・単語定義・変数・Call・キュー・選択肢の行（ponytail-editor-tooling 4.6・5.7） ---
  const SCENE_SCOPE = 'keyword.control.scene.pasta';
  const ATTR_SCOPE = 'entity.other.attribute-name.pasta';
  const COMMENT_SCOPE = 'comment.line.pasta';

  test('シーンの宣言の行: 名前だけが名前の色・行末のコメントはコメントの色', () => {
    const g = '＊OnTest ＃ メモ';
    expect(scopeCoversExactly(g, 'OnTest', SCENE_SCOPE), 'global scene name only');
    expect(scopeCoversExactly(g, '＃ メモ', COMMENT_SCOPE), 'global scene trailing comment');
    const l = '　・ローカル ＃c';
    expect(scopeCoversExactly(l, 'ローカル', SCENE_SCOPE), 'local scene name only');
    expect(scopeCoversExactly(l, '＃c', COMMENT_SCOPE), 'local scene trailing comment');
    expect(scopeCoversExactly('＊ #c', '#c', COMMENT_SCOPE), 'unnamed ＊ trailing comment');
  });

  test('シーンの宣言の行: 属性は属性の色で名前の色に含めない', () => {
    const g = '＊挨拶 ＆朝：1';
    expect(scopeCoversExactly(g, '挨拶', SCENE_SCOPE), 'global scene name stops before the attribute');
    expect(scopeCoversExactly(g, '朝：1', ATTR_SCOPE), 'global scene attribute');
    expect(sameScopes(scopesAt(g, '＆'), ['source.pasta', 'meta.scene.global.pasta', 'meta.attribute.pasta', 'keyword.other.marker.pasta']), `＆: ${scopesAt(g, '＆').join(' ')}`);
    const l = '　・ローカル ＆朝：「#」 ＆夜：2 ＃c';
    expect(scopeCoversExactly(l, 'ローカル', SCENE_SCOPE), 'local scene name only');
    expect(scopeCoversExactly(l, '朝：「#」', ATTR_SCOPE), 'quoted attribute value keeps its #');
    expect(scopeCoversExactly(l, '夜：2', ATTR_SCOPE), 'second attribute');
    expect(scopeCoversExactly(l, '＃c', COMMENT_SCOPE), 'trailing comment after attributes');
  });

  test('アクターの行: 名前と行末のコメントを分ける', () => {
    expect(scopeCoversExactly('％さくら ＃c', 'さくら', 'entity.name.class.pasta'), 'actor dictionary name');
    expect(scopeCoversExactly('％さくら ＃c', '＃c', COMMENT_SCOPE), 'actor dictionary trailing comment');
    const a = '　％さくら、うにゅう ＃c';
    expect(scopeCoversExactly(a, 'さくら、うにゅう', 'entity.name.class.pasta'), 'scene actors body');
    expect(scopeCoversExactly(a, '＃c', COMMENT_SCOPE), 'scene actors trailing comment');
  });

  test('属性の行: 引用の外の ＃ から行末までがコメント', () => {
    const a = '　＆朝：「#」 ＃c';
    expect(scopeCoversExactly(a, '朝：「#」', ATTR_SCOPE), 'attribute body keeps quoted #');
    expect(scopeCoversExactly(a, '＃c', COMMENT_SCOPE), 'attribute trailing comment');
    const f = '＆a：1＆b：x「y #c';
    expect(scopeCoversExactly(f, 'a：1', ATTR_SCOPE), 'first attribute');
    expect(scopeCoversExactly(f, 'b：x「y', ATTR_SCOPE), 'mid-value 「 is ordinary');
    expect(scopeCoversExactly(f, '#c', COMMENT_SCOPE), 'comment after mid-value 「');
  });

  test('変数・Call・キューの行: 行末のコメントを本体から分ける', () => {
    const v = '　＄x＝「#」 ＃c';
    expect(scopeCoversExactly(v, 'x＝「#」', 'variable.other.pasta'), 'variable body');
    expect(scopeCoversExactly(v, '＃c', COMMENT_SCOPE), 'variable trailing comment');
    const c = '　＞次の会話 #c';
    expect(scopeCoversExactly(c, '次の会話', 'keyword.control.pasta'), 'call body');
    expect(scopeCoversExactly(c, '#c', COMMENT_SCOPE), 'call trailing comment');
    const q = '　！wait（「）」） ＃c';
    expect(scopeCoversExactly(q, '「）」', 'string.quoted.other.pasta'), 'cue string arg with ）');
    expect(scopeCoversExactly(q, '＃c', COMMENT_SCOPE), 'cue trailing comment');
    const n = '　！名前＠x ＃c';
    expect(scopeCoversExactly(n, '名前', 'entity.name.function.cue.pasta'), 'cue name is an identifier');
    expect(scopeCoversExactly(n, '＃c', COMMENT_SCOPE), 'cue trailing comment after scope');
  });

  // 文法の XID_Continue は ・ を含む（台詞の単語参照と同じ名前の文字。Oniguruma の表が古い分を足してある）
  test('宣言・属性・選択肢・キューの名前も ・ を名前の文字に含める', () => {
    expect(scopeCoversExactly('＊挨拶・朝', '挨拶・朝', SCENE_SCOPE), 'global scene name with ・');
    expect(scopeCoversExactly('　・a・b', 'a・b', SCENE_SCOPE), 'local scene name with ・');
    expect(scopeCoversExactly('　＆朝・夜：1', '朝・夜：1', ATTR_SCOPE), 'attribute name with ・');
    expect(scopeCoversExactly('＊挨拶・朝 ＆季節・春：1', '季節・春：1', ATTR_SCOPE), 'scene-line attribute name with ・');
    expect(scopeCoversExactly('　＠？はい・いいえ「はい」', 'はい・いいえ', 'keyword.control.pasta'), 'choice target with ・');
    expect(scopeCoversExactly('　！待つ・長く（1）', '待つ・長く', 'entity.name.function.cue.pasta'), 'cue name with ・');
  });

  test('単語定義の行: 囲んだ値の後ろだけが行末のコメント', () => {
    const q = '　＠w：あ、「い」 ＃c';
    expect(scopeCoversExactly(q, 'w：あ、「い」', WORD_SCOPE), 'word definition body');
    expect(scopeCoversExactly(q, '＃c', COMMENT_SCOPE), 'comment after a quoted last value');
    const u = '　＠w：あ、い ＃c';
    expect(!hasScope(tokenizeLine(u).tokens, COMMENT_SCOPE), 'unquoted last value keeps its ＃');
    // 囲みの本文は閉じの並びを含まない。後ろの値の 」・" まで伸ばさない
    expect(!hasScope(tokenizeLine('　＠w：「x」、y「z」 ＃c').tokens, COMMENT_SCOPE), 'unquoted last value y「z」 keeps its ＃');
    expect(!hasScope(tokenizeLine('　＠w："x"、y"z" ＃c').tokens, COMMENT_SCOPE), 'unquoted last value y"z" keeps its ＃');
    expect(scopeCoversExactly('　＠w：""a"b"" ＃c', '＃c', COMMENT_SCOPE), 'comment after a "" fenced value containing "');
    // 値は頭から 1 つずつ読む。囲みの中の区切り・閉じを、値の区切りや最後の値の頭と取り違えない
    expect(!hasScope(tokenizeLine('　＠w：「「a、「b」 ＃c」」').tokens, COMMENT_SCOPE), '「「…」」 is one value; its ＃ is body');
    expect(!hasScope(tokenizeLine('　＠w：「p、"q" ＃d」、r').tokens, COMMENT_SCOPE), 'separator inside 「…」 is body; last value r is unquoted');
    expect(!hasScope(tokenizeLine('　＠w：「「a」 ＃b」」').tokens, COMMENT_SCOPE), '「「 fence closes only at 」」');
    expect(!hasScope(tokenizeLine('　＠w：「a」、 ＃c').tokens, COMMENT_SCOPE), 'after a trailing separator ＃c is an unquoted value');
    expect(scopeCoversExactly('　＠w：「a」、「b」 ＃c', '＃c', COMMENT_SCOPE), 'comment after the second quoted value');
    expect(scopeCoversExactly('　＠w："a、b" ＃c', '＃c', COMMENT_SCOPE), 'comment after a "…" value containing a separator');
    expect(scopeCoversExactly('　＠a、b：「x」 ＃c', '＃c', COMMENT_SCOPE), 'comment after a key list');
  });

  test('単語定義の行: さくらスクリプトの値は 1 つの値として読み、中の ＃ を取り違えない', () => {
    // 文法の word_value は「囲み → さくらスクリプト → 囲まない値」の順に試す
    for (const line of [
      '　＠w：\\h[a,「b」 ＃c]',
      '　＠w：\\h["x","y" ＃c]',
      '　＠w：\\_?a,「b」 ＃c\\_?',
      '　＠w：\\h[a、「b」 ＃c]',
      '　＠w：\\h、x ＃c',
      '　＠w：\\「 ＃c',
      '　＠w：「「a」 ＃c', // pest では誤りの行
    ]) {
      expect(!hasScope(tokenizeLine(line).tokens, COMMENT_SCOPE), `no trailing comment: ${line}`);
    }
    expect(scopeCoversExactly('　＠w：\\h ＃c', 'w：\\h', WORD_SCOPE), 'word body stops after \\h');
    for (const line of ['　＠w：\\h ＃c', '　＠w：「a」、\\h ＃c', '　＠w：\\f[color,"red"] ＃c']) {
      expect(scopeCoversExactly(line, '＃c', COMMENT_SCOPE), `trailing comment after a sakura value: ${line}`);
    }
  });

  test('キューの行: 囲みの引数の中の ）・＃ を本体の終わりやコメントと取り違えない', () => {
    const f = '　！c（「「a」）」」） ＃d';
    expect(scopeCoversExactly(f, '「「a」）」」', 'string.quoted.other.pasta'), '「「…」」 arg is one string');
    expect(scopeCoversExactly(f, '＃d', COMMENT_SCOPE), 'comment after 「「…」」 arg');
    const d = '　！c（""a)""） ＃d';
    expect(scopeCoversExactly(d, '""a)""', 'string.quoted.other.pasta'), '""…"" arg is one string');
    expect(scopeCoversExactly(d, '＃d', COMMENT_SCOPE), 'comment after ""…"" arg');
    for (const line of ['　！c（「a」、"b"） ＃d', '　！c（「abc） ＃d', '　！c（＠a、1、x） ＃d']) {
      expect(scopeCoversExactly(line, '＃d', COMMENT_SCOPE), `cue trailing comment: ${line}`);
    }
    expect(scopeCoversExactly('　！c（「「 ） ＃」', '＃」', COMMENT_SCOPE), 'unclosed 「「 does not swallow the comment');
    expect(scopeCoversExactly('　！c（a"""） ＃"', '＃"', COMMENT_SCOPE), 'unclosed """ does not swallow the comment');
  });

  test('選択肢の行: 単語定義の色にせず、行末のコメントだけを分ける', () => {
    const c = '　＠？はい「はい」';
    expect(!hasScope(tokenizeLine(c).tokens, WORD_SCOPE), 'choice line is not a word definition');
    expect(!hasScope(tokenizeLine('　@?no').tokens, WORD_SCOPE), 'half-width choice line is not a word definition');
    expect(scopeCoversExactly('　＠？はい「はい」 ＃c', '＃c', COMMENT_SCOPE), 'choice trailing comment');
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

  // --- Injection Wiring Tests (task 1.2) ---
  // Minimal confirmation that the getInjections + 3-grammar registry resolves the
  // injection and vendored lua, producing source.lua scopes inside a ```lua body
  // that the non-injected path never yields. This is the GREEN counterpart to the
  // RED state described at the top of the injection infrastructure block.
  injectedPastaGrammar = await initInjectedGrammar();

  test('注入結線: ```lua ブロック本文に source.lua 系スコープが注入される', () => {
    const tokens = tokenizeLinesInjected(['```lua', 'print("hello")', '```']);
    const bodyTokens = tokens.filter((t) => t.line === 1);
    expect(
      bodyTokens.length > 0,
      'lua block body line should produce tokens'
    );
    expect(
      bodyTokens.some((t) => t.scopes.includes('meta.embedded.block.lua.content')),
      'body should still carry pasta content scope (host scope preserved)'
    );
    expect(
      hasLuaScope(bodyTokens),
      'body should carry source.lua-injected scopes (e.g. support.function.lua / string.quoted.double.lua)'
    );
  });

  test('注入結線: 非注入 registry では本文に source.lua 系スコープが出ない（RED 対照）', () => {
    // The existing single-grammar `grammar` (loaded WITHOUT getInjections / lua
    // fixture) must NOT yield any source.lua scope in the block body. This pins
    // the contrast: source.lua appears ONLY because of the injection wiring.
    const open = tokenizeLine('```lua');
    const body = tokenizeLine('print("hello")', open.ruleStack);
    expect(
      hasScope(body.tokens, 'meta.embedded.block.lua.content'),
      'non-injected body should carry the pasta content scope'
    );
    expect(
      !hasLuaScope(body.tokens),
      'non-injected body must NOT carry any source.lua scope'
    );
  });

  // --- Injection Observation Points (task 2.2) ---
  // Four observation points validated through the injection-aware harness, now
  // resolving the REAL shipped grammar (editors/vscode/syntaxes/
  // pasta-lua-injection.tmLanguage.json) via loadInjectionGrammar(). State is
  // threaded across lines with tokenizeLinesInjected() (the 1.2 multi-line path).

  // (1) ```lua ブロック本文の print("hello") に source.lua 系スコープが付与される。
  test('観点1: ```lua ブロック本文の print("hello") に source.lua 系スコープが注入される', () => {
    const tokens = tokenizeLinesInjected(['```lua', 'print("hello")', '```']);
    const bodyTokens = tokens.filter((t) => t.line === 1);
    expect(bodyTokens.length > 0, 'lua block body line should produce tokens');
    // host scope preserved AND lua injected scope present on the body.
    expect(
      bodyTokens.some((t) => t.scopes.includes('meta.embedded.block.lua.content')),
      'body should still carry pasta host content scope'
    );
    expect(
      hasLuaScope(bodyTokens),
      'print("hello") body should carry source.lua-injected scopes'
    );
  });

  // (2) 言語名なしフェンス（```）で開始するブロック本文にも Lua 着色が注入される。
  test('観点2: 言語名なしフェンス（```）本文にも Lua 着色が注入される', () => {
    const tokens = tokenizeLinesInjected(['```', 'print("hello")', '```']);
    const bodyTokens = tokens.filter((t) => t.line === 1);
    expect(bodyTokens.length > 0, 'no-language fence body line should produce tokens');
    expect(
      bodyTokens.some((t) => t.scopes.includes('meta.embedded.block.lua.content')),
      'no-language fence body should still carry pasta host content scope'
    );
    expect(
      hasLuaScope(bodyTokens),
      'no-language fence body should carry source.lua-injected scopes'
    );
  });

  // (3) 開始/終了フェンス行に pasta スコープ（フェンス句読点・言語名）が保持される。
  //
  // Begin fence (``` + language name) is asserted through the injected path: it
  // proves the injection preserves the host fence scopes while colouring the body.
  //
  // End fence is asserted through the NON-injected single-grammar path
  // (tokenizeLine). The read-only Lua fixture from task 1.1 treats a backtick as
  // a `string.quoted.double.lua` delimiter, so under injection the closing ```
  // is swallowed as Lua string content and the host `end` rule never fires —
  // a quirk of the vendored fixture, not of the shipped grammars (real VS Code
  // Lua closes its constructs). The end-fence scope is a property of the host
  // SSOT grammar, which the injection (selector L:meta.embedded.block.lua.content)
  // never overrides; the single-grammar path verifies that retention directly.
  test('観点3: 開始/終了フェンス行に pasta スコープ（フェンス句読点・言語名）が保持される', () => {
    const injected = tokenizeLinesInjected(['```lua', 'print("hello")', '```']);
    const beginTokens = injected.filter((t) => t.line === 0);
    // begin fence: ``` punctuation + lua language name retained under injection.
    expect(
      beginTokens.some((t) => t.scopes.includes('punctuation.definition.code.begin.pasta')),
      'begin fence should retain punctuation.definition.code.begin.pasta under injection'
    );
    expect(
      beginTokens.some((t) => t.scopes.includes('entity.name.type.language.pasta')),
      'begin fence should retain entity.name.type.language.pasta for the lua language name'
    );
    // The injection must NOT bleed Lua scopes onto the begin fence line itself.
    expect(
      !hasLuaScope(beginTokens),
      'begin fence line must NOT carry source.lua scopes'
    );

    // end fence: ``` punctuation retained at the host SSOT level.
    const open = tokenizeLine('```lua');
    const body = tokenizeLine('print("hello")', open.ruleStack);
    const close = tokenizeLine('```', body.ruleStack);
    expect(
      hasScope(close.tokens, 'punctuation.definition.code.end.pasta'),
      'end fence should retain punctuation.definition.code.end.pasta (host SSOT grammar)'
    );
  });

  // (4) アクション行内のインライン Lua（＠func()）に source.lua が注入されない。
  //
  // The injection selector is L:meta.embedded.block.lua.content — block-only — so
  // an inline ＠func() in an action line must receive NO source.lua scope. Its
  // pasta scope here is markup.inline.raw.string.pasta (inline-word-ref matches the
  // function name ＠func); the load-bearing assertion is the ABSENCE of
  // any Lua injection, with a pasta inline scope present to confirm it tokenized.
  test('観点4: アクション行内のインライン Lua（＠func()）に source.lua が注入されない', () => {
    const tokens = tokenizeLinesInjected(['　さくら：＠func()　こんにちは']);
    const lineTokens = tokens.filter((t) => t.line === 0);
    expect(lineTokens.length > 0, 'action line should produce tokens');
    // The inline token keeps a pasta scope (no Lua injection occurs)...
    expect(
      lineTokens.some((t) => t.scopes.includes('markup.inline.raw.string.pasta')),
      'inline ＠func() should carry a pasta inline scope (markup.inline.raw.string.pasta)'
    );
    // ...and must NOT receive any source.lua injection (selector is block-only).
    expect(
      !hasLuaScope(lineTokens),
      'inline ＠func() must NOT carry any source.lua scope'
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
