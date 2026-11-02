// 単語参照の箱の表（要件 5.1〜5.3）。vscodeModules.test.ts（箱の範囲）と
// tmGrammar.test.ts（箱と TextMate の単語参照の scope の範囲が同じ）の両方が読む。
export const WORD_REF_CASES: ReadonlyArray<{ name: string; text: string; boxes: string[] }> = [
  { name: '単語参照は名前まで', text: '　さくら：＠挨拶。こんにちは', boxes: ['＠挨拶'] },
  { name: '＠＠ はエスケープ', text: '　さくら：＠＠メール', boxes: [] },
  { name: '＠＠＠ はエスケープの後ろだけ', text: '　さくら：＠＠＠メール', boxes: ['＠メール'] },
  { name: '単語定義の行', text: '　＠挨拶：こんにちは、＠やあ', boxes: [] },
  { name: '行全体のコメント', text: '＃ ＠メモ', boxes: [] },
  { name: 'Lua ブロックの中', text: '```lua\n---@param x number\n　さくら：＠中\n```\n　さくら：＠後', boxes: ['＠後'] },
  { name: '長いフェンスの中の短いフェンス', text: '````lua\n```\n　さくら：＠中\n````\n　さくら：＠後', boxes: ['＠後'] },
  { name: 'キューのスコープ', text: '　！cmd＠scope', boxes: [] },
  { name: '関数呼び出しは名前だけ', text: '　さくら：＠関数（１）', boxes: ['＠関数'] },
  { name: '動的な単語参照', text: '　さくら：＠＄x です', boxes: ['＠＄x'] },
  { name: '継続行', text: '　さくら：はい\n　：＠続き', boxes: ['＠続き'] },
  { name: 'さくらスクリプトの引数', text: '　さくら：\\![open,＠x]', boxes: [] },
  { name: 'CRLF の行', text: '　さくら：＠一\r\n　さくら：＠二', boxes: ['＠一', '＠二'] },
];
