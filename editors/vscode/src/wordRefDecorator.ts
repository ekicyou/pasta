// wordRefDecorator.ts
// Applies background-box decoration to @word-reference tokens in pasta files.
//
// 規則は TextMate の文法（pasta.tmLanguage.json）から読む。自前の正規表現の写しを持たない。

import * as vscode from 'vscode';
import grammar from '../syntaxes/pasta.tmLanguage.json';

const rules = grammar.repository;
// s: Oniguruma の `.` は U+2028・U+2029 にも当たる（行は分けてから合わせる）
const LUA_BEGIN = new RegExp(rules['lua-code-block'].begin, 'us');
const LUA_END_SOURCE = rules['lua-code-block'].end;
// アクション行・継続行の台詞は、どちらも最後の捕まえ（行末まで）
const TALK_LINES = [rules['action-line'].match, rules['continue-action-line'].match].map((s) => new RegExp(s, 'us'));
// TextMate の inline の並び（いちばん左で当たり、同じ位置なら先の規則）を 1 つの選択に並べる
// 箱にするのは動的な単語参照と単語参照（box の捕まえ）だけ
const INLINE = new RegExp(
    `(?:${rules['inline-escape'].match})|(?:${rules['inline-sakura-script'].match})` +
        `|(?<box>(?:${rules['inline-dynamic-ref'].match})|(?:${rules['inline-word-ref'].match}))`,
    'gus',
);

/** 文書全体の文字から、箱を付ける範囲（UTF-16 の位置・半開区間）を返す */
export function findWordRefRanges(text: string): ReadonlyArray<readonly [start: number, end: number]> {
    const ranges: Array<readonly [number, number]> = [];
    let luaEnd: RegExp | undefined;
    let offset = 0;
    for (const raw of text.split('\n')) {
        const lineStart = offset;
        offset += raw.length + 1;
        const line = raw.endsWith('\r') ? raw.slice(0, -1) : raw;
        if (luaEnd) {
            if (luaEnd.test(line)) luaEnd = undefined;
            continue;
        }
        const begin = LUA_BEGIN.exec(line);
        if (begin) {
            // 閉じの \1 は開きの印（バッククォートだけなので正規表現のエスケープは要らない）
            luaEnd = new RegExp(LUA_END_SOURCE.replace('\\1', begin[1]), 'us');
            continue;
        }
        const talkLine = TALK_LINES.map((re) => re.exec(line)).find((m) => m);
        if (!talkLine) continue;
        const talk = talkLine[talkLine.length - 1];
        const talkStart = lineStart + line.length - talk.length;
        INLINE.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = INLINE.exec(talk)) !== null) {
            if (m.groups?.box !== undefined) {
                ranges.push([talkStart + m.index, talkStart + m.index + m[0].length]);
            }
        }
    }
    return ranges;
}

const decorationType = vscode.window.createTextEditorDecorationType({
    border: '1px solid',
    borderColor: new vscode.ThemeColor('editorWidget.border'),
    borderRadius: '3px',
});

function updateDecorations(editor: vscode.TextEditor | undefined): void {
    if (!editor || editor.document.languageId !== 'pasta') return;

    const doc = editor.document;
    const ranges = findWordRefRanges(doc.getText()).map(
        ([start, end]) => new vscode.Range(doc.positionAt(start), doc.positionAt(end)),
    );
    editor.setDecorations(decorationType, ranges);
}

export function activateWordRefDecorator(context: vscode.ExtensionContext): void {
    updateDecorations(vscode.window.activeTextEditor);

    context.subscriptions.push(
        decorationType,
        vscode.window.onDidChangeActiveTextEditor(updateDecorations),
        vscode.workspace.onDidChangeTextDocument(event => {
            if (vscode.window.activeTextEditor?.document === event.document) {
                updateDecorations(vscode.window.activeTextEditor);
            }
        }),
    );
}
