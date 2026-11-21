// 索引・クエリ共有の正準 bigram tokenize（単一ソース）。依存ゼロの純関数。
//
//   - 日本語（空白で区切られない CJK 等）は隣接 2 文字を全位置で bigram 化する。索引・クエリの双方で同一規則を使うので、
//     連続 2 文字以上の日本語語句が、その語を含むページにヒットする。
//   - ASCII 語（英数）は空白・ハイフン区切りの単語トークンを維持する。
//
// ブラウザは ESM import 不可のため、theme/head.hbs に本関数のミラー（ES5 化した同一規則のコピー）を
// インライン同梱し、defineProperty フックで elasticlunr.tokenizer を差し替えている。
// 本ファイルを変更したら head.hbs の BEGIN/END canonical bigram tokenize ブロックも同期すること
// （verify-search.mjs が規則一致を検証する）。

// 正規化: 小文字化 + 全角英数を半角へ（NFKC は CJK 互換まで畳むため使わず、英数記号のみ畳む）。
function normalize(text) {
  return text
    .toLowerCase()
    .replace(/[Ａ-Ｚａ-ｚ０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0));
}

// 1 文字が「CJK 等の語中連結対象（bigram 化する文字）」かどうか。
// ひらがな・カタカナ・CJK 統合漢字・CJK 拡張 A・長音記号などを対象にする。
function isCjk(ch) {
  const cp = ch.codePointAt(0);
  return (
    (cp >= 0x3040 && cp <= 0x30ff) || // ひらがな・カタカナ
    (cp >= 0x3400 && cp <= 0x4dbf) || // CJK 拡張A
    (cp >= 0x4e00 && cp <= 0x9fff) || // CJK 統合漢字
    (cp >= 0xf900 && cp <= 0xfaff) || // CJK 互換漢字
    (cp >= 0xff66 && cp <= 0xff9f) || // 半角カタカナ
    cp === 0x30fc // 長音記号 ー
  );
}

// 入力文字列をトークン列へ。
//   - 連続する CJK 文字列は隣接 2-gram に分割（"構造体" -> ["構造","造体"]）。
//     単独 CJK 文字（前後が非 CJK で 1 文字だけ）はその 1 文字をトークンとして残す。
//   - ASCII/その他は空白・ハイフンで分割した単語トークン。
export function tokenize(input) {
  if (input === null || input === undefined) return [];
  const text = normalize(String(input));
  const tokens = [];

  let i = 0;
  const n = text.length;
  while (i < n) {
    const ch = text[i];
    if (isCjk(ch)) {
      // CJK ランを収集
      let j = i;
      const run = [];
      while (j < n && isCjk(text[j])) {
        run.push(text[j]);
        j++;
      }
      if (run.length === 1) {
        tokens.push(run[0]);
      } else {
        for (let k = 0; k + 1 < run.length; k++) {
          tokens.push(run[k] + run[k + 1]);
        }
      }
      i = j;
    } else if (/\s/.test(ch) || ch === '-') {
      i++;
    } else {
      // 非 CJK・非区切り（ASCII 英数記号など）のラン
      let j = i;
      const run = [];
      while (j < n && !isCjk(text[j]) && !/\s/.test(text[j]) && text[j] !== '-') {
        run.push(text[j]);
        j++;
      }
      tokens.push(run.join(''));
      i = j;
    }
  }
  return tokens.filter((t) => t.length > 0);
}
