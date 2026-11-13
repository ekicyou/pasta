// タブの .pasta の見出しの上に「▶ シーンを実行」「値を入れて実行…」を出す（1.2・8.1・11.4）。
// 並びはそのタブの今の解析から mapping.ts の sceneTargets で作る。キーボードでは F1 の「Show CodeLens Commands For Current Line」で届く。
import { monaco, semanticSource } from "./editor.ts";
import { sceneTargets, type SceneTarget } from "./mapping.ts";
import type { DevActions } from "./devActions.ts";
import type { Documents } from "./documents.ts";

const RUN = "pasta.recipe.runScene";
const WITH_LOCALS = "pasta.recipe.runSceneWithLocals";

export function registerSceneLens(docs: Pick<Documents, "tabs">, dev: Pick<DevActions, "run" | "runWithLocals">): void {
  monaco.editor.registerCommand(RUN, (_a, t: SceneTarget) => void dev.run(t));
  monaco.editor.registerCommand(WITH_LOCALS, (_a, t: SceneTarget) => void dev.runWithLocals(t));
  monaco.languages.registerCodeLensProvider("pasta", {
    // 解析を当てるたびに Documents が鳴らす合図（色の塗り直しと同じ）で出し直す。Monaco は引数を見ない
    onDidChange: semanticSource.changed.event as never,
    provideCodeLenses(model) {
      const tab = docs.tabs.find((t) => t.model === model);
      const lenses = (tab?.analysis ? sceneTargets(tab.rel, tab.analysis.scenes) : []).flatMap(({ line, target }) => {
        const range = { startLineNumber: line, startColumn: 1, endLineNumber: line, endColumn: 1 };
        return [
          { range, command: { id: RUN, title: "▶ シーンを実行", arguments: [target] } },
          { range, command: { id: WITH_LOCALS, title: "値を入れて実行…", arguments: [target] } },
        ];
      });
      return { lenses, dispose() {} };
    },
  });
}
