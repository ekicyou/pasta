// 開いているファイルの状態の正本（タブ・モデル・ディスクとの揃い）。判断は tabs.ts、ここは Monaco とタブに当てるだけ
import { monaco } from "./editor.ts";
import { readText, writeText } from "./ipc.ts";
import { languageOf } from "./mapping.ts";
import { afterChoice, findSame, needsConfirm } from "./tabs.ts";
import { confirm3, notify } from "./dialog.ts";

export interface Tab {
  id: number;
  /** master からの相対・'/' 区切り。名前の変更で書き換える（6.4） */
  rel: string;
  /** URI は ghost-tab:/<id>（パスに依らない。名前の変更でモデルを作り直さない） */
  model: monaco.editor.ITextModel;
  /** 最後にディスクと揃えた中身（消えたら null） */
  diskText: string | null;
  diskBom: boolean;
  deleted: boolean;
  savedVersionId: number;
  viewState: monaco.editor.ICodeEditorViewState | null;
  /** 最後に求めた保存。保存はこの後ろへつなぎ、同じファイルへの書き込みを重ねない */
  saving: Promise<boolean>;
}

export interface DocumentsHooks {
  /** 保存の成功の後（4.9 のシーンの作り直し・9.2 の検査の求めは 6.3・7.1 でここへつなぐ） */
  onSaved?(tab: Tab): void;
  /** タブを閉じた後（7.6 の波線・診断の一覧の片付けは 6.3・6.6 でここへつなぐ） */
  onClosed?(tab: Tab): void;
}

export const isDirty = (tab: Tab) => tab.deleted || tab.model.getAlternativeVersionId() !== tab.savedVersionId;
const baseName = (rel: string) => rel.slice(rel.lastIndexOf("/") + 1);

export class Documents {
  readonly tabs: Tab[] = [];
  active: Tab | null = null;
  #nextId = 1;
  #editor: monaco.editor.IStandaloneCodeEditor;
  #bar: HTMLElement;
  #hooks: DocumentsHooks;

  constructor(editor: monaco.editor.IStandaloneCodeEditor, bar: HTMLElement, hooks: DocumentsHooks = {}) {
    this.#editor = editor;
    this.#bar = bar;
    this.#hooks = hooks;
    // Ctrl+S はエディタの外に焦点があっても今のタブを保存する（5.6）。自動では保存しない（5.11）
    document.addEventListener("keydown", (e) => {
      if (!e.ctrlKey || e.altKey || e.shiftKey || e.key.toLowerCase() !== "s") return;
      e.preventDefault();
      if (e.repeat) return; // 押しっぱなしで保存を重ねない
      if (document.querySelector("dialog[open]")) return; // 確かめの窓の裏では保存しない
      if (this.active) void this.save(this.active);
    });
  }

  /** 同じファイルのタブがあればそこへ（5.4）。無ければ読んで開く。UTF-8 でない等は理由を出して開かない（5.3） */
  async open(rel: string, pos?: monaco.IPosition): Promise<Tab | null> {
    let tab = findSame(this.tabs, rel);
    if (!tab) {
      let file;
      try {
        file = await readText(rel);
      } catch (reason) {
        notify(String(reason));
        return null;
      }
      // 読んでいる間に同じファイルが開かれていたらそちらを使う
      tab = findSame(this.tabs, rel);
      if (!tab) {
        const id = this.#nextId++;
        // Monaco は中身から改行（CRLF・LF）を決め、getValue はその改行で返す（5.7）
        const model = monaco.editor.createModel(file.text, languageOf(rel), monaco.Uri.from({ scheme: "ghost-tab", path: `/${id}` }));
        tab = { id, rel, model, diskText: file.text, diskBom: file.bom, deleted: false, savedVersionId: model.getAlternativeVersionId(), viewState: null, saving: Promise.resolve(true) };
        model.onDidChangeContent(() => this.#render());
        this.tabs.push(tab);
      }
    }
    this.activate(tab);
    if (pos) {
      this.#editor.setPosition(pos);
      this.#editor.revealPositionInCenter(pos);
    }
    this.#editor.focus();
    return tab;
  }

  activate(tab: Tab): void {
    if (this.active && this.active !== tab) this.active.viewState = this.#editor.saveViewState();
    this.active = tab;
    this.#editor.setModel(tab.model);
    if (tab.viewState) this.#editor.restoreViewState(tab.viewState);
    this.#render();
  }

  /** モデルの改行のまま・開いたときの BOM のまま書く（5.7）。失敗は理由を出し、中身と印を残す（5.8） */
  save(tab: Tab): Promise<boolean> {
    // 前の保存が終わってから始め、そこで今の中身を読むので最新の中身が最後に書かれる。前が投げても（onSaved など）鎖は切らない
    const write = () => this.#write(tab);
    tab.saving = tab.saving.then(write, write);
    return tab.saving;
  }

  async #write(tab: Tab): Promise<boolean> {
    const text = tab.model.getValue();
    const versionId = tab.model.getAlternativeVersionId();
    try {
      await writeText(tab.rel, text, tab.diskBom);
    } catch (reason) {
      notify(String(reason));
      return false;
    }
    tab.diskText = text;
    tab.deleted = false;
    // 書いている間の書き換えは未保存のまま残す
    tab.savedVersionId = versionId;
    this.#render();
    this.#hooks.onSaved?.(tab);
    return true;
  }

  /** 未保存なら 3 択（5.9）。閉じたら true */
  async close(tab: Tab): Promise<boolean> {
    if (needsConfirm([{ id: tab.id, rel: tab.rel, dirty: isDirty(tab) }])) {
      const choice = await confirm3(`${baseName(tab.rel)} の変更を保存しますか？`);
      const saved = choice === "save" ? ((await this.save(tab)) ? "ok" : "failed") : null;
      if (afterChoice(choice, saved) === "stay") return false;
    }
    const i = this.tabs.indexOf(tab);
    if (i < 0) return true;
    this.tabs.splice(i, 1);
    if (this.active === tab) {
      const next = this.tabs[Math.min(i, this.tabs.length - 1)];
      this.active = null;
      if (next) this.activate(next);
      else this.#editor.setModel(null);
    }
    tab.model.dispose();
    this.#render();
    this.#hooks.onClosed?.(tab);
    return true;
  }

  #render(): void {
    this.#bar.replaceChildren(
      ...this.tabs.map((tab) => {
        const dirty = isDirty(tab);
        const el = document.createElement("div");
        el.className = "tab";
        el.title = tab.rel;
        const name = document.createElement("button");
        name.className = "tab-name";
        // 名前はディスクから来るので textContent だけで書く
        name.textContent = (dirty ? "● " : "") + baseName(tab.rel);
        name.setAttribute("aria-label", `${tab.rel}${dirty ? "（未保存）" : ""}`);
        name.setAttribute("aria-pressed", String(tab === this.active));
        name.addEventListener("click", () => this.activate(tab));
        const x = document.createElement("button");
        x.className = "tab-close";
        x.textContent = "×";
        x.setAttribute("aria-label", `${baseName(tab.rel)} を閉じる`);
        x.addEventListener("click", () => void this.close(tab));
        el.append(name, x);
        return el;
      }),
    );
  }
}
