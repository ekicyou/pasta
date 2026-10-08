// talk.mjs — 台詞部品（Claudia とアンソニーの掛け合い）の登録簿と記法の解析。
// manual-claudia-theme タスク 1.1（要件 3.2, 3.3, 3.4, 3.8, 4.2, 4.8, 10.7 / design「TalkCore」）。
//
// 登録簿（SPEAKERS）は話し手と表情の唯一の定義。話し手や表情の追加はこの配列の変更と
// 顔画像（book/src/img/claudia/）の追加で完結する。
//
// 記法（design「データモデル / 記法の文法」）:
//   > 【表情】本文            … 既定の話し手（Claudia）の表情
//   > 【話し手】本文          … その話し手の「素」
//   > 【話し手：表情】本文    … 区切りは全角コロン「：」のみ
//   話し手名はカタカナの日本語名だけ（別名・欧文名なし）。
//
// 共用 export（talk-html.mjs・gen-skill-refs.mjs・verify-content.mjs などが import する）:
//   SPEAKERS / DEFAULT_SPEAKER_ID / DEFAULT_FACE / FACE_DIR / MAX_FACE_BYTES / TALK_LINE_RE
//   parseTalkTag(inner) … 【】の内側を解析
//   faceFilePath(speaker, face) … 顔画像への相対パス（book/src と出力根から）
//
// 純関数のみ・入出力なし。

const speaker = (id, name, latin, side, faces) =>
  Object.freeze({ id, name, latin, side, faces: Object.freeze(faces) });

export const SPEAKERS = Object.freeze([
  speaker('claudia', 'クローディア', 'Claudia', 'left', {
    素: 'f0', 照れ: 'f1', 驚き: 'f2', 不安: 'f3', 落胆: 'f4', 高笑い: 'f5', 目閉じ: 'f6',
    不機嫌: 'f7', 冷笑: 'f8', 照れ怒り: 'f9', にっこり: 'f25', したり顔: 'f26', 考え中: 'f27', お辞儀: 'f28',
  }),
  speaker('anthony', 'アンソニー', 'Anthony', 'right', { 素: 'f10', 刮目: 'f11' }),
]);

export const DEFAULT_SPEAKER_ID = 'claudia';
export const DEFAULT_FACE = '素';
export const FACE_DIR = 'img/claudia';
export const MAX_FACE_BYTES = 20480;

// 台詞の開始行（字下げも拾う。入れ子の引用・リスト内の判定は走査側で行う）。
export const TALK_LINE_RE = /^\s*>\s*【/;

const SEP = '：';
const DEFAULT_SPEAKER = SPEAKERS.find((s) => s.id === DEFAULT_SPEAKER_ID);
const byName = (name) => SPEAKERS.find((s) => s.name === name);
const candidates = (names) => `（候補: ${names.join('・')}）`;

const fail = (kind, detail) => ({ ok: false, kind, detail });

function resolveFace(sp, face) {
  if (!Object.hasOwn(sp.faces, face)) {
    return fail('unknown-expression',
      `${sp.name} に表情「${face}」はありません${candidates(Object.keys(sp.faces))}`);
  }
  return { ok: true, tag: { speaker: sp, face, file: sp.faces[face] } };
}

// 【】の内側を解析する。成功なら { ok: true, tag }、失敗なら { ok: false, kind, detail }。
export function parseTalkTag(inner) {
  if (inner === '') return fail('malformed-tag', '空のタグです');
  if (/[\s【】:]/.test(inner)) {
    return fail('malformed-tag',
      `タグ「${inner}」に空白・括弧・半角コロンは使えません（区切りは全角コロン「${SEP}」）`);
  }
  const parts = inner.split(SEP);
  if (parts.length > 2) return fail('malformed-tag', `タグ「${inner}」の区切り「${SEP}」は 1 つまでです`);
  if (parts.some((p) => p === '')) return fail('malformed-tag', `タグ「${inner}」の話し手か表情が空です`);

  if (parts.length === 2) {
    const sp = byName(parts[0]);
    if (!sp) {
      return fail('unknown-speaker',
        `話し手「${parts[0]}」はいません${candidates(SPEAKERS.map((s) => s.name))}`);
    }
    return resolveFace(sp, parts[1]);
  }

  // 単独の名前: 話し手名ならその話し手の「素」、そうでなければ既定の話し手の表情。
  const sp = byName(inner);
  if (sp) return resolveFace(sp, DEFAULT_FACE);
  if (Object.hasOwn(DEFAULT_SPEAKER.faces, inner)) return resolveFace(DEFAULT_SPEAKER, inner);
  return fail('unknown-expression',
    `「${inner}」は ${DEFAULT_SPEAKER.name} の表情にも話し手にもありません`
    + candidates([...Object.keys(DEFAULT_SPEAKER.faces), ...SPEAKERS.map((s) => s.name)]));
}

// 話し手と表情から顔画像への相対パスを組み立てる（例: 'img/claudia/f5.png'）。
export function faceFilePath(sp, face) {
  if (!Object.hasOwn(sp.faces, face)) throw new Error(`${sp.name} に表情「${face}」はありません`);
  return `${FACE_DIR}/${sp.faces[face]}.png`;
}
