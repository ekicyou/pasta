// tokenizer.mjs — PastaTokenizer（タスク 2.2 / 要件 1.1, 1.3, 5.1, 5.2, 5.3, 6.2）。
//
// pasta 文法（source.pasta・SSOT・読み取り専用）と vendor lua 文法（source.lua）を
// oniguruma WASM 上の vscode-textmate でロードし、テキストを行単位トークナイズする。
//
// 二段トークナイズ（核心）:
//   pasta 文法は lua-code-block に `include: source.lua` を持たない（改変不可・5.2）。
//   よって pasta パスで `meta.embedded.block.lua.content` 区間を検出し、その内部テキストを
//   vendor lua 文法で再トークナイズして lua スコープ付きトークンへ差し替える（1.3）。
//   lua 側も行間 ruleStack を引き回す（複数行 lua 文字列の跨ぎを正しく着色）。
//   差し替え時は lua 区間トークンの行内オフセットを元行オフセットへ正規化して返す。
//
// 設計根拠: design.md「PastaTokenizer」Service Interface / Implementation Notes、
//          research §8.1（vscode-textmate/oniguruma 利用法）・§8.6（二段トークナイズ採用）。
//
// 契約（design Service Interface）:
//   Pre : load() resolve 済み。text は改行 `\n`（`\r` を行文字列に含めない）。
//   Post: 行数と返り配列長が一致。各 TokenSpan は行を被覆。入れ子 lua 区間は lua スコープを持つ。
//   Inv : 同一 text → 同一トークン列（決定論・6.4 を支える）。文法ファイルを変更しない（5.2）。
//
// 失敗（6.2）: WASM init／文法ロード／パース失敗・文法ファイル不在は例外を送出する。
//   process.exit はしない（CLI 層＝後続 3.1 が exit 1 する設計）。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import vsctm from 'vscode-textmate';
import oniguruma from 'vscode-oniguruma';

const here = path.dirname(fileURLToPath(import.meta.url));
const bookDir = path.resolve(here, '../..'); // book/

// pasta 文法が入れ子 lua 本体に付与する contentName。この区間を二段化対象とする。
const LUA_CONTENT_SCOPE = 'meta.embedded.block.lua.content';

// oniguruma WASM は build-time のみロードする（プロセス内で一度きり初期化）。
let onigLibPromise = null;
function getOnigLib() {
  if (onigLibPromise) return onigLibPromise;
  const wasmPath = path.join(
    bookDir,
    'node_modules/vscode-oniguruma/release/onig.wasm',
  );
  // 不在時はここで例外（6.2: 必須 build-time 依存欠落）。
  const wasmBin = fs.readFileSync(wasmPath).buffer;
  onigLibPromise = oniguruma.loadWASM(wasmBin).then(() => ({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (s) => new oniguruma.OnigString(s),
  }));
  return onigLibPromise;
}

// *.json 文法を必ず *.json パス付きで parseRawGrammar に渡す（plist 誤判定回避）。
// 不在・パース失敗はここで例外（6.2）。
function loadRawGrammar(filePath) {
  const content = fs.readFileSync(filePath, 'utf8'); // 不在で ENOENT 例外
  const raw = vsctm.parseRawGrammar(content, filePath);
  if (!raw || !raw.scopeName) {
    throw new Error(`Invalid TextMate grammar (no scopeName): ${filePath}`);
  }
  return raw;
}

/**
 * 1 つの TextMate 文法でテキスト全体を行単位トークナイズする（ruleStack を行間で引き回す）。
 * textmate は各行末に仮想 `\n` を付けてマッチするため、トークン終端が行長を超え得る。
 * 呼び出し側で行境界へクランプする前提の生トークンを返す。
 *
 * @param {object} grammar vscode-textmate IGrammar
 * @param {string[]} lines 改行を含まない行配列
 * @returns {{ tokens: {startIndex:number,endIndex:number,scopes:string[]}[] }[]}
 */
function tokenizeLinesRaw(grammar, lines) {
  let ruleStack = vsctm.INITIAL;
  const out = [];
  for (const line of lines) {
    const r = grammar.tokenizeLine(line, ruleStack);
    ruleStack = r.ruleStack;
    out.push({ tokens: r.tokens });
  }
  return out;
}

// 生トークンを行境界 [0, line.length] にクランプし、隙間なく被覆する TokenSpan 列へ整える。
// textmate の仮想改行由来の超過終端を吸収し、Post 条件（各行被覆・行内オフセット）を満たす。
function clampToLine(tokens, line) {
  const spans = [];
  let cursor = 0;
  for (const t of tokens) {
    const start = Math.max(0, Math.min(t.startIndex, line.length));
    const end = Math.max(start, Math.min(t.endIndex, line.length));
    if (end <= cursor) continue; // 仮想改行の空トークン等は捨てる
    const s = Math.max(start, cursor);
    spans.push({ startIndex: s, endIndex: end, scopes: t.scopes.slice() });
    cursor = end;
  }
  // 末尾が行長に満たない場合（理論上ほぼ起きないが）プレーン区間で埋めて被覆を保証。
  if (cursor < line.length) {
    spans.push({ startIndex: cursor, endIndex: line.length, scopes: ['source.pasta'] });
  }
  // 空行（line.length === 0）は空配列を返す（coversLine と整合）。
  return spans;
}

/**
 * PastaTokenizer を生成する factory。
 * `load(paths)` → `tokenizeText(text)` の契約（design Service Interface）を満たす。
 *
 * @returns {{
 *   load: (paths: {pasta:string, lua:string}) => Promise<void>,
 *   tokenizeText: (text: string) => {startIndex:number,endIndex:number,scopes:string[]}[][],
 *   tokenizeTextSinglePass: (text: string) => {startIndex:number,endIndex:number,scopes:string[]}[][],
 * }}
 */
export function createPastaTokenizer() {
  let pastaGrammar = null;
  let luaGrammar = null;
  let registry = null;

  async function load(paths) {
    if (!paths || typeof paths.pasta !== 'string' || typeof paths.lua !== 'string') {
      throw new Error('load(paths): paths.pasta と paths.lua（*.json）が必要です');
    }
    // 不在・不正は loadRawGrammar 内で例外（6.2）。WASM init 失敗も例外。
    const onigLib = Promise.resolve(getOnigLib());
    const rawByScope = new Map();
    const pastaRaw = loadRawGrammar(paths.pasta);
    const luaRaw = loadRawGrammar(paths.lua);
    rawByScope.set(pastaRaw.scopeName, pastaRaw); // source.pasta
    rawByScope.set(luaRaw.scopeName, luaRaw); // source.lua

    registry = new vsctm.Registry({
      onigLib,
      loadGrammar: async (scopeName) => rawByScope.get(scopeName) || null,
    });

    pastaGrammar = await registry.loadGrammar(pastaRaw.scopeName);
    luaGrammar = await registry.loadGrammar(luaRaw.scopeName);
    if (!pastaGrammar) throw new Error(`Failed to load grammar: ${pastaRaw.scopeName}`);
    if (!luaGrammar) throw new Error(`Failed to load grammar: ${luaRaw.scopeName}`);
  }

  // pasta 文法のみで行トークナイズ（二段化なし）。lua 本体は content スコープ止まり。
  // テスト対比用・内部の二段化前段としても使用。
  function tokenizeTextSinglePass(text) {
    ensureLoaded();
    const lines = text.split('\n');
    const rawLines = tokenizeLinesRaw(pastaGrammar, lines);
    return rawLines.map((rl, i) => clampToLine(rl.tokens, lines[i]));
  }

  function ensureLoaded() {
    if (!pastaGrammar || !luaGrammar) {
      throw new Error('PastaTokenizer.tokenizeText: load() を先に await してください');
    }
  }

  // 二段トークナイズ本体。
  // 1) pasta 文法で全行をトークナイズ（lua ブロック begin/end の複数行 state を ruleStack で維持）。
  // 2) meta.embedded.block.lua.content を含むトークン区間 = lua 本体を行ごとに収集。
  // 3) lua 本体テキストを行配列として vendor lua 文法で再トークナイズ（lua 側 ruleStack 継続）。
  // 4) 再トークナイズ結果を、元行内の lua 区間オフセットへ正規化して pasta 行へ差し戻す。
  function tokenizeText(text) {
    ensureLoaded();
    const lines = text.split('\n');
    const pastaRaw = tokenizeLinesRaw(pastaGrammar, lines);

    // 各 pasta 行について、lua content 区間 [contentStart, contentEnd) を抽出する。
    // 行内に lua content トークンが無ければ null。pasta 文法上、lua 本体は 1 行=1 区間。
    const luaRegions = lines.map((line, i) => {
      const toks = pastaRaw[i].tokens;
      let start = null;
      let end = null;
      for (const t of toks) {
        if (t.scopes.includes(LUA_CONTENT_SCOPE)) {
          const s = Math.max(0, Math.min(t.startIndex, line.length));
          const e = Math.max(s, Math.min(t.endIndex, line.length));
          if (start === null) start = s;
          end = e;
        }
      }
      return start === null ? null : { start, end };
    });

    // lua 本体行の連なり（ブロック）を、lua 文法へ「行配列」として渡し ruleStack を継続させる。
    // 連続する lua content 行を 1 ブロックとしてまとめ、ブロック単位で lua トークナイズする。
    // 各 lua 行テキストは元行の [start, end) 部分文字列（通常は行全体だが厳密に切り出す）。
    const luaTokensByLine = new Array(lines.length).fill(null);
    let block = []; // { lineIndex, text, start }
    const flush = () => {
      if (block.length === 0) return;
      const luaLineTexts = block.map((b) => b.text);
      const luaRaw = tokenizeLinesRaw(luaGrammar, luaLineTexts);
      for (let j = 0; j < block.length; j++) {
        const { lineIndex, text: luaLine, start } = block[j];
        // lua トークンを lua 行内でクランプし、元行オフセット start を加算して正規化。
        const clamped = clampLuaToLine(luaRaw[j].tokens, luaLine);
        luaTokensByLine[lineIndex] = clamped.map((s) => ({
          startIndex: s.startIndex + start,
          endIndex: s.endIndex + start,
          scopes: s.scopes,
        }));
      }
      block = [];
    };
    for (let i = 0; i < lines.length; i++) {
      const region = luaRegions[i];
      if (region) {
        block.push({
          lineIndex: i,
          text: lines[i].slice(region.start, region.end),
          start: region.start,
        });
      } else {
        flush();
      }
    }
    flush();

    // 各 pasta 行を組み立てる。lua content 区間は lua トークンへ差し替え、
    // 区間外（フェンス・インデント等の pasta トークン）はそのまま残す。
    const result = [];
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const region = luaRegions[i];
      if (!region || !luaTokensByLine[i]) {
        result.push(clampToLine(pastaRaw[i].tokens, line));
        continue;
      }
      result.push(mergeLuaIntoLine(pastaRaw[i].tokens, line, region, luaTokensByLine[i]));
    }
    return result;
  }

  return { load, tokenizeText, tokenizeTextSinglePass };
}

// lua 行トークンを [0, luaLine.length] にクランプし隙間なく被覆（lua 行ローカル座標）。
function clampLuaToLine(tokens, luaLine) {
  const spans = [];
  let cursor = 0;
  for (const t of tokens) {
    const start = Math.max(0, Math.min(t.startIndex, luaLine.length));
    const end = Math.max(start, Math.min(t.endIndex, luaLine.length));
    if (end <= cursor) continue;
    const s = Math.max(start, cursor);
    spans.push({ startIndex: s, endIndex: end, scopes: t.scopes.slice() });
    cursor = end;
  }
  if (cursor < luaLine.length) {
    spans.push({ startIndex: cursor, endIndex: luaLine.length, scopes: ['source.lua'] });
  }
  return spans;
}

// pasta 行のトークンのうち lua content 区間 [region.start, region.end) を
// 正規化済み lua トークン列で置換する。区間前後の pasta トークンは保持し、
// 行全体 [0, line.length] を隙間なく被覆させる。
function mergeLuaIntoLine(pastaTokens, line, region, luaSpans) {
  const out = [];
  let cursor = 0;

  // 区間前の pasta トークン（[0, region.start)）を行内クランプで取り込む。
  for (const t of pastaTokens) {
    const start = Math.max(0, Math.min(t.startIndex, line.length));
    const end = Math.max(start, Math.min(t.endIndex, line.length));
    if (end <= region.start) {
      if (end <= cursor) continue;
      const s = Math.max(start, cursor);
      if (s < end) {
        out.push({ startIndex: s, endIndex: end, scopes: t.scopes.slice() });
        cursor = end;
      }
    }
  }
  // 区間前に隙間があればプレーンで埋める（通常はインデント等が pasta トークンで埋まる）。
  if (cursor < region.start) {
    out.push({ startIndex: cursor, endIndex: region.start, scopes: ['source.pasta'] });
    cursor = region.start;
  }

  // lua 区間: 正規化済み lua トークンをそのまま採用（既に元行オフセット）。
  for (const s of luaSpans) {
    const start = Math.max(region.start, Math.min(s.startIndex, region.end));
    const end = Math.max(start, Math.min(s.endIndex, region.end));
    if (end <= cursor) continue;
    out.push({ startIndex: Math.max(start, cursor), endIndex: end, scopes: s.scopes });
    cursor = end;
  }
  if (cursor < region.end) {
    out.push({ startIndex: cursor, endIndex: region.end, scopes: ['source.lua'] });
    cursor = region.end;
  }

  // 区間後の pasta トークン（[region.end, line.length)）。pasta 文法上 lua 本体行は
  // 行末まで content なので通常は空だが、堅牢性のため保持する。
  for (const t of pastaTokens) {
    const start = Math.max(0, Math.min(t.startIndex, line.length));
    const end = Math.max(start, Math.min(t.endIndex, line.length));
    if (start >= region.end) {
      if (end <= cursor) continue;
      const s = Math.max(start, cursor);
      if (s < end) {
        out.push({ startIndex: s, endIndex: end, scopes: t.scopes.slice() });
        cursor = end;
      }
    }
  }
  if (cursor < line.length) {
    out.push({ startIndex: cursor, endIndex: line.length, scopes: ['source.pasta'] });
  }
  return out;
}
