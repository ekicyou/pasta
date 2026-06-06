// scope-map.mjs — ScopeClassMapper（タスク 2.1 / 要件 1.2, 1.3, 2.1, 2.2, 2.3, 6.1）。
//
// TextMate スコープスタックを highlight.js 互換クラスへ写像する文法非依存の純関数。
// pasta 文法（source.pasta）も vendor lua 文法（source.lua）も、前方一致ゆえ同一テーブルで
// 6色グループへ写る（入れ子 lua の着色を追加コードなしで担保）。
//
// 写像の根拠: research §8.2（実 CSS 色グループ検証済み）・design.md ScopeClassMapper（L292）。
// 採用は light/navy 両テーマで確実に着色され色が衝突しない6スロットのみ:
//   gray=hljs-comment / purple=hljs-keyword / blue,aqua=hljs-title /
//   orange=hljs-built_in,hljs-literal,hljs-number / green=hljs-string /
//   red=hljs-variable,hljs-attr,hljs-tag
// 禁止: hljs-symbol（navy 無色）・hljs-section/hljs-name（同色衝突）は使わない。
// 色は焼き込まず（2.3）クラス名のみ返す。配色は mdBook テーマ CSS に委ねる。

/**
 * スコープ前方一致テーブル（SSOT・一点集約）。
 *
 * キーは TextMate スコープの `.` 区切り前方一致プレフィックス、値は hljs 互換クラス。
 * 各スコープは末尾（最特定）から `.` 区切りで段階短縮しながら本テーブルを探索し、
 * 最初にヒットしたクラスを採用する（より長い＝より特定的なプレフィックスが優先）。
 *
 * 例:
 *   'keyword.control.scene' は 'keyword.control.scene' で先にヒット → hljs-title。
 *   'keyword.control' は 'keyword.control.scene' に一致せず 'keyword.control'→'keyword'
 *   の順で短縮し 'keyword' でヒット → hljs-keyword。
 *
 * 末尾の言語サフィックス（`.pasta`/`.lua` 等）は段階短縮の過程で自然に脱落するため、
 * 文法非依存になる（例 `comment.line.double-dash.lua` は `comment` でヒット）。
 *
 * @type {Readonly<Record<string, string>>}
 */
const SCOPE_CLASS_TABLE = Object.freeze({
  // gray — コメント
  'comment': 'hljs-comment',

  // blue/aqua — 名前系（シーン・アクター）。keyword.control より特定的に先判定。
  'keyword.control.scene': 'hljs-title',
  'entity.name.class': 'hljs-title',
  'entity.name.type.actor': 'hljs-title',

  // purple — キーワード（マーカー・呼び出し制御）。scene 以外の keyword.* を畳む。
  'keyword': 'hljs-keyword',

  // orange — 呼出・定数
  'entity.name.function': 'hljs-built_in',
  'constant.character.escape': 'hljs-literal',
  'constant.numeric': 'hljs-number',

  // green — 文字列系（単語＠・さくらスクリプト・引用）
  'markup.inline.raw.string': 'hljs-string',
  'string': 'hljs-string',

  // red — 参照系（変数・属性・タグ）
  'variable': 'hljs-variable',
  'entity.other.attribute-name': 'hljs-attr',
  'entity.name.tag': 'hljs-tag',
});

/**
 * 単一スコープを `.` 区切りで長い順に短縮しながらテーブル探索し、最初にヒットした
 * クラスを返す。どのプレフィックスもヒットしなければ null。
 *
 * @param {string} scope 単一の TextMate スコープ（例 'keyword.control.scene.pasta'）
 * @returns {string | null}
 */
function classForScope(scope) {
  if (typeof scope !== 'string' || scope.length === 0) return null;
  const parts = scope.split('.');
  // 長い前方一致を優先: フルスコープ → 末尾セグメントを1つずつ落とす。
  for (let len = parts.length; len > 0; len--) {
    const prefix = parts.slice(0, len).join('.');
    const cls = SCOPE_CLASS_TABLE[prefix];
    if (cls !== undefined) return cls;
  }
  return null;
}

/**
 * スコープスタックを hljs 互換クラスへ写像する（純関数・副作用なし・決定論的）。
 *
 * `scopes` の末尾（最特定）から先頭へ走査し、各スコープを `.` 区切り前方一致で
 * テーブル探索する。最初にヒットしたクラスを返す。空配列・全て未マッチは null
 * （要件 6.1 のプレーン継続）。文法非依存（pasta も lua も同一テーブル）。
 *
 * @param {string[]} scopes tokenizeLine 由来のスコープスタック（外→内・末尾が最特定）。空配列可。
 * @returns {string | null} 確定マッピング表のいずれか、または null。
 */
export function scopesToClass(scopes) {
  if (!Array.isArray(scopes)) return null;
  // 末尾（最特定）→先頭の順に走査。最初に着色対象が見つかった時点で確定。
  for (let i = scopes.length - 1; i >= 0; i--) {
    const cls = classForScope(scopes[i]);
    if (cls !== null) return cls;
  }
  return null;
}

// テーブルはテスト（禁止クラス非使用・採用範囲封じ込め検証）から参照される。
export { SCOPE_CLASS_TABLE };
