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
//   タスク 1.2（要件 3.7, 4.1, 4.3, 4.6, 4.7, 10.3, 10.7）:
//   - scanTalk がフェンス外の台詞ブロック（開始行・終了行・タグ・本文）を列挙する。
//   - 9 種のエラーがそれぞれ正しい行番号で出る（最初の 1 件で止めない）。
//   - フェンス内の台詞のような行は無視する。CRLF 入力を LF と同じに扱う。
//   - checkRegion が台詞だけ・両話し手を検査し、表紙の設定では扉の内側と HTML ブロックを許す。
//   - faceStats と --stats CLI が表情の出現数と章ごとの導入・締めの組み合わせを出し、exit 0。

import {
  SPEAKERS,
  DEFAULT_SPEAKER_ID,
  DEFAULT_FACE,
  FACE_DIR,
  MAX_FACE_BYTES,
  TALK_LINE_RE,
  parseTalkTag,
  faceFilePath,
  scanTalk,
  checkRegion,
  faceStats,
} from './talk.mjs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

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
// タスク 1.2: 走査・領域検査・集計
const md = (...lines) => lines.join('\n');
const has = (errors, kind, line) => errors.some((e) => e.kind === kind && e.line === line);
const kinds = (errors) => errors.map((e) => `${e.kind}@${e.line}`).join(', ');

// 正常な章（導入・本文・締め）。行番号は 1 始まり。
const GOOD = md(
  '# 題', //                                        1
  '', //                                            2
  '> 【高笑い】さあ、**熱く**参りましょう！', //     3
  '>', //                                           4
  '> 二段落目ですわ。', //                           5
  '', //                                            6
  '> 【アンソニー】執事のアンソニーでございます。', // 7
  '', //                                            8
  '---', //                                         9
  '', //                                            10
  '本文。', //                                      11
  '', //                                            12
  '> 将来変更あり: 通常の引用', //                   13
  '', //                                            14
  '```text', //                                     15
  '> 【不明】フェンスの中', //                       16
  '> 【素】', //                                    17
  '```', //                                         18
  '', //                                            19
  '~~~~', //                                        20
  '- > 【素】チルダのフェンスの中', //                21
  '~~~~', //                                        22
  '', //                                            23
  '---', //                                         24
  '', //                                            25
  '> 【クローディア：にっこり】では。', //           26
  '', //                                            27
  '> 【アンソニー：刮目】おや。', //                 28
);
const GOOD_INTRO = { start: 1, end: 8 }; // 0 始まり・end 排他（2〜8 行目）
const GOOD_BODY = { start: 9, end: 23 };
const GOOD_OUTRO = { start: 24, end: 28 };

log('\n== (F) scanTalk 正常系・フェンス ==');
{
  const r = scanTalk(GOOD);
  check('F-1 正常な章はエラー 0', r.errors.length === 0, kinds(r.errors));
  check('F-2 台詞ブロックは 4 つ（フェンス内・通常の引用は数えない）', r.blocks.length === 4, String(r.blocks.length));
  const [b0, b1, b2, b3] = r.blocks;
  check('F-3 1 つ目: 開始 3・終了 5', b0 && b0.line === 3 && b0.endLine === 5, b0 && `${b0.line}-${b0.endLine}`);
  check('F-4 1 つ目: タグ（クローディア・高笑い・f5）',
    b0 && b0.tag.speaker.id === 'claudia' && b0.tag.face === '高笑い' && b0.tag.file === 'f5');
  check('F-5 1 つ目: 本文は引用記号を除いた Markdown（段落区切りを保つ）',
    b0 && b0.body === 'さあ、**熱く**参りましょう！\n\n二段落目ですわ。', b0 && JSON.stringify(b0.body));
  check('F-6 2 つ目: アンソニーの素・開始 7・終了 7',
    b1 && b1.tag.speaker.id === 'anthony' && b1.tag.face === '素' && b1.line === 7 && b1.endLine === 7);
  check('F-7 3 つ目: 話し手と表情・本文', b2 && b2.line === 26 && b2.tag.face === 'にっこり' && b2.body === 'では。');
  check('F-8 4 つ目: 末尾改行なしでも終了行は 28', b3 && b3.line === 28 && b3.endLine === 28 && b3.body === 'おや。');
  const r2 = scanTalk(GOOD + '\n');
  check('F-9 末尾改行ありでも同じ結果', JSON.stringify(r2) === JSON.stringify(r));
  check('F-10 フェンス内の不正な台詞はエラーにしない（16・17・21 行目）',
    !r.errors.some((e) => [16, 17, 21].includes(e.line)));
  const r3 = scanTalk(md('---', '> 【素】区切りの直後', '', '# 見出し', '> 【素】見出しの直後'));
  check('F-11 区切り・見出しの直後の台詞は空行なしでも可', r3.errors.length === 0 && r3.blocks.length === 2,
    kinds(r3.errors));
  const r4 = scanTalk(md('> 普通の引用', '> > 普通の入れ子', '', '- 項目', '  > 項目内の引用'));
  check('F-12 台詞でない引用・入れ子・リストはエラーにしない', r4.errors.length === 0 && r4.blocks.length === 0,
    kinds(r4.errors));
}

log('\n== (G) scanTalk 不正系（種類と行番号） ==');
{
  const tagErr = scanTalk(md(
    '> 【アンソニー:刮目】半角コロン', //  1 malformed-tag
    '', //                                2
    '> 【素 本文', //                      3 malformed-tag（】欠落）
    '', //                                4
    '> 【エミリ：素】未知の話し手', //      5 unknown-speaker
    '', //                                6
    '> 【アンソニー：高笑い】無い表情', //  7 unknown-expression
    '', //                                8
    '> 【素】', //                         9 empty-body
    '', //                                10
    '> 【アンソニー】', //                 11 empty-body（空の > だけ続く）
    '>', //                               12
  ));
  const e = tagErr.errors;
  check('G-1 malformed-tag: 半角コロン @1', has(e, 'malformed-tag', 1), kinds(e));
  check('G-2 malformed-tag: 】欠落 @3', has(e, 'malformed-tag', 3), kinds(e));
  check('G-3 unknown-speaker @5', has(e, 'unknown-speaker', 5), kinds(e));
  check('G-4 unknown-expression（アンソニーに無い表情）@7', has(e, 'unknown-expression', 7), kinds(e));
  check('G-5 empty-body @9', has(e, 'empty-body', 9), kinds(e));
  check('G-6 empty-body（空の > だけ）@11', has(e, 'empty-body', 11), kinds(e));
  check('G-7 全件列挙（6 件ちょうど）', e.length === 6, kinds(e));
  check('G-8 不正なブロックは blocks に入らない', tagErr.blocks.length === 0, String(tagErr.blocks.length));
  check('G-9 各エラーは行番号と詳細を持つ',
    e.every((x) => Number.isInteger(x.line) && typeof x.detail === 'string' && x.detail !== ''));
  check('G-10 未知の話し手の詳細に候補が添わる',
    e.some((x) => x.kind === 'unknown-speaker' && x.detail.includes('クローディア')));

  const blank = scanTalk(md(
    '# 題', //                       1
    '', //                           2
    '> 【素】一行目', //              3
    '> 【アンソニー】連続', //        4 missing-blank-line（台詞同士）
    '', //                           5
    '> 【素】続き', //                6
    '遅延継続の行', //                7 missing-blank-line（遅延継続）
    '次の行', //                      8（同じ段落なので重ねて出さない）
    '', //                           9
    '段落', //                       10
    '> 【アンソニー】段落の直後', //  11 missing-blank-line（段落と台詞）
  ));
  const b = blank.errors;
  check('G-11 missing-blank-line: 台詞同士 @4', has(b, 'missing-blank-line', 4), kinds(b));
  check('G-12 missing-blank-line: 遅延継続 @7', has(b, 'missing-blank-line', 7), kinds(b));
  check('G-13 missing-blank-line: 段落の直後 @11', has(b, 'missing-blank-line', 11), kinds(b));
  check('G-14 missing-blank-line は 3 件ちょうど', b.length === 3, kinds(b));
  check('G-15 連続した台詞も両方ブロックとして解析される', blank.blocks.length === 4, String(blank.blocks.length));

  const nest = scanTalk(md(
    '- > 【素】リスト内', //          1
    '', //                            2
    '> > 【素】引用内', //            3
    '', //                            4
    '  > 【素】字下げ', //            5
    '', //                            6
    '1. > 【アンソニー】番号付き', //  7
  ));
  const n = nest.errors;
  for (const line of [1, 3, 5, 7]) check(`G-16 nested-talk @${line}`, has(n, 'nested-talk', line), kinds(n));
  check('G-17 nested-talk だけ 4 件・ブロックなし', n.length === 4 && nest.blocks.length === 0, kinds(n));

  // 完了時の棚卸しで見つかった取りこぼし: 引用記号の間の空白 2 つ・引用の中のリスト・リスト 2 段・引用の中のリスト項目の台詞。
  const nest2 = scanTalk(md(
    '>  > 【素】空白 2 つの入れ子', //   1
    '', //                                2
    '> - > 【素】引用の中のリスト', //    3
    '', //                                4
    '1. - > 【素】リスト 2 段', //        5
    '', //                                6
    '> - 【素】引用の中のリスト項目', //  7
  ));
  const n2 = nest2.errors;
  for (const line of [1, 3, 5, 7]) check(`G-22 nested-talk @${line}（取りこぼしの形）`, has(n2, 'nested-talk', line), kinds(n2));
  check('G-23 取りこぼしの形は nested-talk だけ 4 件・ブロックなし', n2.length === 4 && nest2.blocks.length === 0, kinds(n2));

  // 台詞の中のリストと、引用でないリスト項目の【は、これまでどおり台詞の入れ子として扱わない。
  const ok2 = scanTalk(md(
    '> 【素】手順は次のとおりですわ。', // 1
    '> - 一つ目', //                       2
    '> - 二つ目', //                       3
    '', //                                 4
    '- 【参考】ただのリスト項目', //       5
  ));
  check('G-24 台詞の中のリストと、引用でないリスト項目はエラーにしない', ok2.errors.length === 0 && ok2.blocks.length === 1, kinds(ok2.errors));

  const unsup = scanTalk(md(
    '> 【素】説明します。', //  1
    '> ```text', //            2 unsupported-content（フェンス）
    '> code', //               3
    '> ```', //                4
    '', //                     5
    '> 【素】表です。', //      6
    '> | a | b |', //          7 unsupported-content（表）
    '> | - | - |', //          8
    '', //                     9
    '> 【素】引用です。', //    10
    '> > 入れ子', //           11 unsupported-content（入れ子の引用）
  ));
  const u = unsup.errors;
  check('G-18 unsupported-content: フェンス @2', has(u, 'unsupported-content', 2), kinds(u));
  check('G-19 unsupported-content: 表 @7', has(u, 'unsupported-content', 7), kinds(u));
  check('G-20 unsupported-content: 入れ子の引用 @11', has(u, 'unsupported-content', 11), kinds(u));
  check('G-21 構造ごとに 1 件（3 件ちょうど）', u.length === 3, kinds(u));
}

log('\n== (H) CRLF ==');
{
  const lf = scanTalk(GOOD);
  const crlf = scanTalk(GOOD.replace(/\n/g, '\r\n'));
  check('H-1 CRLF 入力は LF と同じ結果', JSON.stringify(crlf) === JSON.stringify(lf));
  check('H-2 本文に \\r を含まない', crlf.blocks.every((x) => !x.body.includes('\r')));
  const bad = scanTalk('> 【素】a\r\n> 【エミリ】b\r\n');
  check('H-3 CRLF でも行番号が正しい',
    has(bad.errors, 'missing-blank-line', 2) && has(bad.errors, 'unknown-expression', 2), kinds(bad.errors));
}

log('\n== (I) checkRegion ==');
{
  const lines = GOOD.split('\n');
  const { blocks } = scanTalk(GOOD);
  check('I-1 導入: 台詞だけ・両話し手で合格', checkRegion(blocks, lines, GOOD_INTRO, { cover: false }).length === 0);
  check('I-2 締め: 合格', checkRegion(blocks, lines, GOOD_OUTRO, { cover: false }).length === 0);
  const body = checkRegion(blocks, lines, GOOD_BODY, { cover: false });
  check('I-3 本文の範囲は prose-in-region @11', has(body, 'prose-in-region', 11), kinds(body));

  const prose = md(
    '# 題', //               1
    '', //                   2
    '> 【素】a', //           3
    '', //                   4
    '地の文です。', //        5 prose-in-region
    '続きの行。', //          6（同じ塊なので重ねて出さない）
    '', //                   7
    '> 【アンソニー】b', //   8
    '', //                   9
    '---', //                10
  );
  const pr = checkRegion(scanTalk(prose).blocks, prose.split('\n'), { start: 1, end: 9 }, { cover: false });
  check('I-4 prose-in-region @5（塊ごとに 1 件）', has(pr, 'prose-in-region', 5) && pr.length === 1, kinds(pr));

  const solo = md('# 題', '', '> 【素】a', '', '---');
  const so = checkRegion(scanTalk(solo).blocks, solo.split('\n'), { start: 1, end: 4 }, { cover: false });
  check('I-5 missing-speaker（アンソニー不在）@2', has(so, 'missing-speaker', 2) && so.length === 1, kinds(so));
  check('I-6 missing-speaker の詳細に欠けた話し手名', so.some((x) => x.detail.includes('アンソニー')));
  const none = checkRegion([], ['', ''], { start: 0, end: 2 }, { cover: false });
  check('I-7 台詞の無い範囲は両話し手が missing-speaker',
    none.length === 2 && none.every((x) => x.kind === 'missing-speaker'), kinds(none));

  const cover = md(
    '# pasta マニュアル', //                                                  1
    '', //                                                                    2
    '<section class="claudia-hero" aria-label="扉">', //                      3
    '<div class="hero-faces"><img src="img/claudia/f0.png" alt=""></div>', // 4
    '', //                                                                    5
    '> 【お辞儀】ようこそ。', //                                               6
    '', //                                                                    7
    '> 【アンソニー】ご案内いたします。', //                                   8
    '', //                                                                    9
    '<nav class="hero-toc">', //                                              10
    '', //                                                                    11
    '- [入門](getting-started/index.md)', //                                  12
    '', //                                                                    13
    '</nav>', //                                                              14
    '</section>', //                                                          15
    '', //                                                                    16
    '---', //                                                                 17
    '', //                                                                    18
    '本文。', //                                                              19
    '', //                                                                    20
    '---', //                                                                 21
    '', //                                                                    22
    '> 【にっこり】では。', //                                                 23
    '', //                                                                    24
    '> 【アンソニー】失礼いたします。', //                                     25
    '', //                                                                    26
    '<p class="claudia-credit">顔アイコン: ponapalt/claudia</p>', //           27
  );
  const cs = scanTalk(cover);
  const cl = cover.split('\n');
  check('I-8 表紙の走査はエラー 0', cs.errors.length === 0, kinds(cs.errors));
  const heroOn = checkRegion(cs.blocks, cl, { start: 1, end: 16 }, { cover: true });
  check('I-9 cover: 扉の内側（HTML・案内リスト）を許す', heroOn.length === 0, kinds(heroOn));
  const heroOff = checkRegion(cs.blocks, cl, { start: 1, end: 16 }, { cover: false });
  check('I-10 cover なしでは扉の HTML が prose-in-region @3・案内 @10',
    has(heroOff, 'prose-in-region', 3) && has(heroOff, 'prose-in-region', 10), kinds(heroOff));
  const outOn = checkRegion(cs.blocks, cl, { start: 21, end: 27 }, { cover: true });
  check('I-11 cover: 締めのクレジット（HTML ブロック）を許す', outOn.length === 0, kinds(outOn));
  const outOff = checkRegion(cs.blocks, cl, { start: 21, end: 27 }, { cover: false });
  check('I-12 cover なしではクレジットが prose-in-region @27', has(outOff, 'prose-in-region', 27), kinds(outOff));
  const bodyOn = checkRegion(cs.blocks, cl, { start: 17, end: 20 }, { cover: true });
  check('I-13 cover でも扉の外の地の文は prose-in-region @19', has(bodyOn, 'prose-in-region', 19), kinds(bodyOn));
}

log('\n== (J) 表情の集計 ==');
{
  const { blocks } = scanTalk(GOOD);
  const out = faceStats(new Map([['a.md', blocks], ['b.md', []]]),
    new Map([['a.md', { intro: GOOD_INTRO, outro: GOOD_OUTRO }]]));
  const lines = out.split('\n');
  const row = (...cells) => lines.includes(`| ${cells.join(' | ')} |`);
  check('J-1 出現数: クローディア 高笑い 1', row('クローディア', '高笑い', '1'), out);
  check('J-2 出現数: アンソニー 刮目 1', row('アンソニー', '刮目', '1'));
  check('J-3 出現数: 使われない表情も 0 で並ぶ', row('クローディア', '冷笑', '0'));
  check('J-4 出現数は 16 表情すべて', SPEAKERS.every((s) => Object.keys(s.faces).every((f) =>
    lines.some((l) => l.startsWith(`| ${s.name} | ${f} |`)))));
  check('J-5 章ごと: 導入と締めの組み合わせ',
    row('a.md', 'クローディア：高笑い, アンソニー：素', 'クローディア：にっこり, アンソニー：刮目'), out);
  check('J-6 章ごと: 台詞の無い章も行を持つ', lines.some((l) => l.startsWith('| b.md |')));

  const talkPath = fileURLToPath(new URL('./talk.mjs', import.meta.url));
  const cli = spawnSync(process.execPath, [talkPath, '--stats'], { encoding: 'utf8' });
  check('J-7 --stats は exit 0', cli.status === 0, `status=${cli.status} ${cli.stderr}`);
  check('J-8 --stats は出現数と章ごとの表を出す',
    cli.stdout.includes('表情の出現数') && cli.stdout.includes('| 章 | 導入 | 締め |'), cli.stdout.slice(0, 200));
  const chapterRows = cli.stdout.split('\n').filter((l) => /^\| \S+\.md \|/.test(l));
  check('J-9 --stats は全 60 章（debug を含む）を並べる',
    chapterRows.length === 60 && chapterRows.some((l) => l.startsWith('| debug/')), String(chapterRows.length));
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
