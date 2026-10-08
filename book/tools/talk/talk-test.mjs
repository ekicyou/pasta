// talk-test.mjs — 台詞部品の登録簿とタグ解析の自己テスト（manual-claudia-theme タスク 1.1 / 要件 3.2, 3.3, 3.4, 3.8, 4.2, 4.8, 10.7）。
//
// 観測する完了条件（design「TalkCore」Validation・「Testing Strategy / Unit Tests」）:
//   - 登録簿の不変条件: 各話し手が「素」を持つ／話し手名と表情名が重ならない／
//     画像名が全話し手で一意／id が英小文字だけ／話し手名がカタカナだけ。
//   - 登録簿の初期値（design「登録簿（初期値）」）と一致する（Claudia 14 表情・アンソニー 2 表情）。
//   - 全 16 表情を解析できる。話し手省略（表情だけ）・表情省略（話し手だけ）も解析できる。
//   - 失敗の種類（malformed-tag・unknown-speaker・unknown-expression）が正しく返り、未知の名前には候補が添わる。
//   - 顔画像の相対パスを組み立てる。
//   - TALK_LINE_RE が台詞の開始行を見分ける。

import {
  SPEAKERS,
  DEFAULT_SPEAKER_ID,
  DEFAULT_FACE,
  FACE_DIR,
  MAX_FACE_BYTES,
  TALK_LINE_RE,
  parseTalkTag,
  faceFilePath,
} from './talk.mjs';

// --- 最小 assert ハーネス（依存ゼロ） ---
let passed = 0;
let failed = 0;
const log = (...a) => console.log(...a);
function check(name, cond, detail) {
  if (cond) {
    passed++;
    log(`  PASS  ${name}`);
  } else {
    failed++;
    log(`  FAIL  ${name}${detail ? '  -- ' + detail : ''}`);
  }
}
const byId = (id) => SPEAKERS.find((s) => s.id === id);

// ============================================================
log('\n== (A) 登録簿の不変条件と初期値 ==');
{
  check('A-1 話し手は claudia・anthony の 2 名',
    JSON.stringify(SPEAKERS.map((s) => s.id)) === '["claudia","anthony"]');
  check('A-2 既定の話し手は claudia・既定の表情は素',
    DEFAULT_SPEAKER_ID === 'claudia' && DEFAULT_FACE === '素' && !!byId(DEFAULT_SPEAKER_ID));
  for (const s of SPEAKERS) {
    check(`A-3 ${s.id} は表情「素」を持つ`, Object.hasOwn(s.faces, DEFAULT_FACE));
    check(`A-4 ${s.id} の id は英小文字だけ`, /^[a-z]+$/.test(s.id));
    check(`A-5 ${s.id} の名前はカタカナだけ`, /^[ァ-ヺー]+$/.test(s.name), s.name);
    check(`A-6 ${s.id} の配置は left か right`, s.side === 'left' || s.side === 'right');
    check(`A-7 ${s.id} の画像名は f<数字>`, Object.values(s.faces).every((f) => /^f\d+$/.test(f)));
  }
  const allFaces = SPEAKERS.flatMap((s) => Object.keys(s.faces));
  const clash = SPEAKERS.filter((s) => allFaces.includes(s.name)).map((s) => s.name);
  check('A-8 話し手名と表情名が重ならない', clash.length === 0, clash.join(','));
  const files = SPEAKERS.flatMap((s) => Object.values(s.faces));
  check('A-9 画像名が全話し手で一意', new Set(files).size === files.length, files.join(','));
  check('A-10 名前も一意', new Set(SPEAKERS.map((s) => s.name)).size === SPEAKERS.length);

  const c = byId('claudia');
  const a = byId('anthony');
  check('A-11 Claudia の名前・名札・配置',
    c.name === 'クローディア' && c.latin === 'Claudia' && c.side === 'left');
  check('A-12 アンソニーの名前・名札・配置',
    a.name === 'アンソニー' && a.latin === 'Anthony' && a.side === 'right');
  const expectC = {
    素: 'f0', 照れ: 'f1', 驚き: 'f2', 不安: 'f3', 落胆: 'f4', 高笑い: 'f5', 目閉じ: 'f6',
    不機嫌: 'f7', 冷笑: 'f8', 照れ怒り: 'f9', にっこり: 'f25', したり顔: 'f26', 考え中: 'f27', お辞儀: 'f28',
  };
  check('A-13 Claudia の表情 14 種と画像の対応', JSON.stringify(c.faces) === JSON.stringify(expectC),
    JSON.stringify(c.faces));
  check('A-14 アンソニーの表情 2 種と画像の対応',
    JSON.stringify(a.faces) === JSON.stringify({ 素: 'f10', 刮目: 'f11' }), JSON.stringify(a.faces));
  check('A-15 表情は計 16 種', files.length === 16);
  check('A-16 登録簿は凍結されている',
    Object.isFrozen(SPEAKERS) && SPEAKERS.every((s) => Object.isFrozen(s) && Object.isFrozen(s.faces)));
  check('A-17 FACE_DIR・MAX_FACE_BYTES', FACE_DIR === 'img/claudia' && MAX_FACE_BYTES === 20480);
}

// ============================================================
log('\n== (B) タグの解析（正常系） ==');
{
  // 全 16 表情を「話し手：表情」で解析できる。
  for (const s of SPEAKERS) {
    for (const [face, file] of Object.entries(s.faces)) {
      const r = parseTalkTag(`${s.name}：${face}`);
      check(`B-1 ${s.name}：${face}`, r.ok && r.tag.speaker === s && r.tag.face === face && r.tag.file === file,
        JSON.stringify(r));
    }
  }
  // 話し手省略 → 既定の話し手（Claudia）の表情。
  const c = byId('claudia');
  for (const [face, file] of Object.entries(c.faces)) {
    const r = parseTalkTag(face);
    check(`B-2 表情だけ ${face}`, r.ok && r.tag.speaker === c && r.tag.face === face && r.tag.file === file,
      JSON.stringify(r));
  }
  // 表情省略 → その話し手の「素」。
  for (const s of SPEAKERS) {
    const r = parseTalkTag(s.name);
    check(`B-3 話し手だけ ${s.name}`,
      r.ok && r.tag.speaker === s && r.tag.face === '素' && r.tag.file === s.faces['素'], JSON.stringify(r));
  }
}

// ============================================================
log('\n== (C) タグの解析（失敗の種類） ==');
{
  const expectFail = (id, inner, kind, ...mustInclude) => {
    const r = parseTalkTag(inner);
    const ok = !r.ok && r.kind === kind && typeof r.detail === 'string' && r.detail.length > 0
      && mustInclude.every((m) => r.detail.includes(m));
    check(`${id} ${JSON.stringify(inner)} → ${kind}`, ok, JSON.stringify(r));
  };
  expectFail('C-1', '', 'malformed-tag');
  expectFail('C-2', 'アンソニー:刮目', 'malformed-tag');            // 半角コロン
  expectFail('C-3', 'アンソニー： 刮目', 'malformed-tag');          // 空白入り
  expectFail('C-4', ' 高笑い', 'malformed-tag');
  expectFail('C-5', '高笑い　', 'malformed-tag');               // 全角空白
  expectFail('C-6', 'アンソニー：素：刮目', 'malformed-tag');        // 区切り 2 つ
  expectFail('C-7', '：照れ', 'malformed-tag');                     // 話し手が空
  expectFail('C-8', 'クローディア：', 'malformed-tag');              // 表情が空
  expectFail('C-9', '高笑い】さあ【', 'malformed-tag');              // 括弧の混入（】欠落時に行の残りを渡された場合など）
  expectFail('C-10', 'エミリー：素', 'unknown-speaker', 'エミリー', 'クローディア', 'アンソニー');
  expectFail('C-11', 'Claudia：照れ', 'unknown-speaker', 'Claudia', 'クローディア'); // 欧文名は受け付けない（カタカナのみ）
  expectFail('C-12', 'アンソニー：高笑い', 'unknown-expression', '高笑い', '素', '刮目'); // アンソニーに無い表情
  expectFail('C-13', '刮目', 'unknown-expression', '刮目', 'にっこり');                  // 既定話し手に無い表情
  expectFail('C-14', 'クローディア：刮目', 'unknown-expression', '刮目', 'お辞儀');
  expectFail('C-15', 'クロ', 'unknown-expression', 'クロ', 'クローディア', 'アンソニー'); // 単独の未知名は話し手名も候補に
}

// ============================================================
log('\n== (D) 顔画像の相対パス ==');
{
  const c = byId('claudia');
  const a = byId('anthony');
  check('D-1 クローディア・高笑い', faceFilePath(c, '高笑い') === 'img/claudia/f5.png', faceFilePath(c, '高笑い'));
  check('D-2 アンソニー・刮目', faceFilePath(a, '刮目') === 'img/claudia/f11.png');
  check('D-3 解析結果からの組み立て', (() => {
    const r = parseTalkTag('アンソニー');
    return r.ok && faceFilePath(r.tag.speaker, r.tag.face) === 'img/claudia/f10.png';
  })());
  let threw = false;
  try { faceFilePath(a, '高笑い'); } catch { threw = true; }
  check('D-4 話し手に無い表情は例外', threw);
}

// ============================================================
log('\n== (E) 台詞の開始行 ==');
{
  const yes = ['> 【高笑い】さあ', '>【アンソニー】', '  > 【素】字下げ', '>  【素】'];
  const no = ['> 普通の引用', '【素】引用でない', '>> 【素】', '- > 【素】', '> > 【素】'];
  for (const l of yes) check(`E-1 開始行 ${JSON.stringify(l)}`, TALK_LINE_RE.test(l));
  // 入れ子の引用・リスト内は TALK_LINE_RE では拾わない（走査側=タスク 1.2 で nested-talk として扱う）。
  for (const l of no) check(`E-2 非開始行 ${JSON.stringify(l)}`, !TALK_LINE_RE.test(l));
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
