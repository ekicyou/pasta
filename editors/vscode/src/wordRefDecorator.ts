// wordRefDecorator.ts
// Applies background-box decoration to @word-reference tokens in pasta files.

import * as vscode from 'vscode';

const WORD_REF_PATTERN = /[＠@][^\s＠@＄$\\！!）)（(\r\n]+/g;

const decorationType = vscode.window.createTextEditorDecorationType({
    border: '1px solid',
    borderColor: new vscode.ThemeColor('editorWidget.border'),
    borderRadius: '3px',
});

function updateDecorations(editor: vscode.TextEditor | undefined): void {
    if (!editor || editor.document.languageId !== 'pasta') return;

    const text = editor.document.getText();
    const ranges: vscode.Range[] = [];
    WORD_REF_PATTERN.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = WORD_REF_PATTERN.exec(text)) !== null) {
        const startPos = editor.document.positionAt(match.index);
        const endPos = editor.document.positionAt(match.index + match[0].length);
        ranges.push(new vscode.Range(startPos, endPos));
    }
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
