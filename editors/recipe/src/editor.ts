// Monaco を 1 つ作り、言語・下地の着色（shiki）・テーマ・セマンティックトークンを用意する
import EditorWorker from "monaco-editor/editor/editor.worker?worker";
import * as monaco from "monaco-editor/editor";
import "monaco-editor/features/register.all";
// features/register.all に入っていない（editor.main だけが読む）。無いと文書全体のセマンティックトークンが塗られない
import "monaco-editor/editor/contrib/semanticTokens/browser/documentSemanticTokens";
import { createHighlighterCore, type LanguageRegistration, type ThemeRegistration } from "shiki/core";
import { createOnigurumaEngine } from "shiki/engine/oniguruma";
import { shikiToMonaco } from "@shikijs/monaco";
import lua from "@shikijs/langs/lua";
import lightPlus from "@shikijs/themes/light-plus";
import darkPlus from "@shikijs/themes/dark-plus";
// 拡張の文法と写しが正本（読むだけ）
import pastaGrammar from "../../vscode/syntaxes/pasta.tmLanguage.json";
import injectionGrammar from "../../vscode/syntaxes/pasta-lua-injection.tmLanguage.json";
import vscodePackage from "../../vscode/package.json";
import { analyze, legend } from "./ipc.ts";
import { keepTokens, semanticRules, withSemanticRules, type TmTheme } from "./theme.ts";

self.MonacoEnvironment = { getWorker: () => new EditorWorker() };

const scopes = vscodePackage.contributes.semanticTokenScopes.find((e) => e.language === "pasta")!.scopes;

/** 明暗の設定に合うテーマの名前（shiki のテーマの name がそのまま Monaco のテーマの名前になる） */
const themeFor = (dark: boolean) => (dark ? "dark-plus" : "light-plus");
// JSON の推論の型は文法の型に合わないので、読み込んだ形のまま型だけを付ける
const grammar = (json: object, extra: Partial<LanguageRegistration>) =>
  ({ ...json, ...extra }) as LanguageRegistration;

export async function createEditor(container: HTMLElement): Promise<monaco.editor.IStandaloneCodeEditor> {
  monaco.languages.register({ id: "pasta", extensions: [".pasta"] });
  monaco.languages.register({ id: "lua", extensions: [".lua"] });

  const highlighter = await createHighlighterCore({
    engine: createOnigurumaEngine(import("shiki/wasm")),
    langs: [
      lua,
      grammar(pastaGrammar, { name: "pasta", embeddedLangs: ["lua"] }),
      grammar(injectionGrammar, { name: "pasta-lua-injection", injectTo: ["source.pasta"], embeddedLangs: ["lua"] }),
    ],
    themes: [lightPlus, darkPlus].map((t) => withSemanticRules(t as TmTheme, scopes) as ThemeRegistration),
  });
  shikiToMonaco(highlighter, monaco);

  const { tokenTypes, tokenModifiers } = await legend();
  // テーマごとの「規則のある種類の番号」。規則の無い種類は下地の色を消すので渡さない（6.6）
  const styled = new Map(
    [lightPlus, darkPlus].map((t) => {
      const kinds = new Set(semanticRules(t as TmTheme, scopes).map((r) => r.scope));
      return [t.name!, new Set(tokenTypes.flatMap((k, i) => (kinds.has(k) ? [i] : [])))] as const;
    }),
  );
  const dark = matchMedia("(prefers-color-scheme: dark)");
  let theme = themeFor(dark.matches);
  const themeChanged = new monaco.Emitter<void>();
  // shortcut: 今のモデルを直に解析する仮のつなぎ。6.3 で Documents の最新の結果を読む形へ差し替える
  monaco.languages.registerDocumentSemanticTokensProvider("pasta", {
    getLegend: () => ({ tokenTypes, tokenModifiers }),
    onDidChange: themeChanged.event,
    async provideDocumentSemanticTokens(model) {
      const kept = styled.get(theme)!;
      const tokens = keepTokens((await analyze(model.getValue())).tokens, (t) => kept.has(t));
      const data = new Uint32Array(tokens.length * 5);
      tokens.forEach((t, i) =>
        data.set([t.deltaLine, t.deltaStartCharacter, t.length, t.tokenType, t.tokenModifiers], i * 5),
      );
      return { data };
    },
    releaseDocumentSemanticTokens() {},
  });

  dark.addEventListener("change", (e) => {
    theme = themeFor(e.matches);
    monaco.editor.setTheme(theme);
    themeChanged.fire();
  });
  return monaco.editor.create(container, {
    theme,
    automaticLayout: true,
    "semanticHighlighting.enabled": true,
    // 日本語の字下げの全角空白（U+3000）を枠で囲まない。ほかの紛らわしい文字の知らせは残す
    unicodeHighlight: { allowedCharacters: { "\u3000": true } },
  });
}

export { monaco };
