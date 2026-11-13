// 開いているファイルの状態の正本（タブ・モデル・ディスクとの揃い）。判断は tabs.ts、ここは Monaco とタブに当てるだけ
import { monaco, semanticSource } from "./editor.ts";
import { analyze, listTree, readText, writeText, type AnalysisResult, type TextFile } from "./ipc.ts";
import { diagnosticsToMarkers, languageOf } from "./mapping.ts";
import { CoalescingRunner } from "./scheduler.ts";
import { afterChoice, findSame, isDirty, needsConfirm, retargetTabs, saveAll } from "./tabs.ts";
import { decideExternal, failedRead, type DiskState } from "./sync.ts";
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
  /** 最後に当てた解析の結果（.pasta だけ）。解析の失敗では前のものを残す（6.5） */
  analysis: AnalysisResult | null;
  /** analysis を求めた時の model.getVersionId() */
  analysisVersion: number;
  /** 外で変わって「読み直す／今の中身を残す」を待っているディスクの中身（10.3）。帯はタブごとに 1 つ */
  pending: TextFile | null;
}

export interface DocumentsHooks {
  /** 保存の成功の後（9.2 の検査の求め） */
  onSaved?(tab: Tab): void;
  /** タブを閉じた後（7.6 の診断の一覧の片付け・閉じたファイルのシーンの読み直し） */
  onClosed?(tab: Tab): void;
  /** 新しい解析の結果を当てた後（ツリーのシーン 4.10・診断の一覧 7.3） */
  onAnalyzed?(tab: Tab): void;
  /** 名前の変更でタブの rel・言語を付け替えた後（診断の一覧の名前と 9.8 の除外を今の rel に揃える） */
  onRenamed?(): void;
}

/** 書き換えが止まってから解析するまで（7.2） */
const ANALYZE_DELAY_MS = 200;

const baseName = (rel: string) => rel.slice(rel.lastIndexOf("/") + 1);

export class Documents {
  readonly tabs: Tab[] = [];
  active: Tab | null = null;
  #nextId = 1;
  /** beginSwitch（別のゴーストへ移る）のたびに進める。読む前と後で違えば、その結果は前のゴーストのもの */
  #generation = 0;
  /** beginSwitch から endSwitch まで。Rust の master が替わる途中なので、どちらの master を読むか決まらず、タブを開かない */
  #switching = false;
  #editor: monaco.editor.IStandaloneCodeEditor;
  #bar: HTMLElement;
  #notices: HTMLElement;
  /** 帯は差分で足し引きする（描き直すと読み上げが繰り返され、ボタンの焦点も外れる） */
  #noticeEls = new Map<Tab, HTMLElement>();
  #hooks: DocumentsHooks;
  /** 外の変更の適用を 1 つずつ（重なると同じタブを 2 度読み直す） */
  #external: Promise<void> = Promise.resolve();

  constructor(editor: monaco.editor.IStandaloneCodeEditor, bar: HTMLElement, notices: HTMLElement, hooks: DocumentsHooks = {}) {
    this.#editor = editor;
    this.#bar = bar;
    this.#notices = notices;
    this.#hooks = hooks;
    // 提供者は今の版の結果だけを返す。古い結果の位置で塗らない
    semanticSource.tokensOf = (model) => {
      const tab = this.tabs.find((t) => t.model === model);
      return tab?.analysis && tab.analysisVersion === model.getVersionId() ? tab.analysis.tokens : null;
    };
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
    if (this.#switching) return null;
    let tab = findSame(this.tabs, rel);
    if (!tab) {
      const generation = this.#generation;
      let file;
      try {
        file = await readText(rel);
      } catch (reason) {
        if (generation === this.#generation) notify(String(reason));
        return null;
      }
      if (generation !== this.#generation || this.#switching) return null; // 読む間に別のゴーストへ移った。前のゴーストの中身でタブを作らない
      // 読んでいる間に同じファイルが開かれていたらそちらを使う
      tab = findSame(this.tabs, rel);
      if (!tab) {
        const id = this.#nextId++;
        // Monaco は中身から改行（CRLF・LF）を決め、getValue はその改行で返す（5.7）
        const model = monaco.editor.createModel(file.text, languageOf(rel), monaco.Uri.from({ scheme: "ghost-tab", path: `/${id}` }));
        const opened: Tab = { id, rel, model, diskText: file.text, diskBom: file.bom, deleted: false, savedVersionId: model.getAlternativeVersionId(), viewState: null, saving: Promise.resolve(true), analysis: null, analysisVersion: -1, pending: null };
        const runner = new CoalescingRunner(() => this.#analyze(opened), ANALYZE_DELAY_MS, globalThis);
        // 解析は .pasta だけ（8.3）。言語は名前の変更（6.4）で変わりうるので、そのつど見る
        const request = () => {
          if (model.getLanguageId() === "pasta") runner.request();
        };
        model.onDidChangeContent(() => {
          this.#render();
          request();
        });
        // 名前の変更で .pasta になれば解析し、.pasta でなくなれば波線を消す（トークンの提供者は pasta だけなので色は消える）
        model.onDidChangeLanguage(() => {
          if (model.getLanguageId() === "pasta") return request();
          opened.analysis = null;
          monaco.editor.setModelMarkers(model, "pasta", []);
        });
        request();
        tab = opened;
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
    // 移る途中の保存は、書く時にどちらの master かが決まらないので受けない（3 択で保存しないを選んだ後の Ctrl+S など）
    if (this.#switching) return Promise.resolve(false);
    // 前の保存が終わってから始め、そこで今の中身を読むので最新の中身が最後に書かれる。前が投げても（onSaved など）鎖は切らない
    const write = () => this.#write(tab);
    tab.saving = tab.saving.then(write, write);
    return tab.saving;
  }

  async #write(tab: Tab): Promise<boolean> {
    if (tab.model.isDisposed()) return false; // 鎖で待つ間にタブが閉じられた（保存しないを選んだ）。書かず、投げない
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
    tab.pending = null; // 保存で外の中身を上書きしたので、帯の選択はもう要らない
    // 書いている間の書き換えは未保存のまま残す
    tab.savedVersionId = versionId;
    this.#render();
    this.#renderNotices();
    this.#hooks.onSaved?.(tab);
    return true;
  }

  /** 送った時の版と同じ結果だけを当てる。失敗は前の結果と波線を残す（6.5・7.2・7.7） */
  async #analyze(tab: Tab): Promise<void> {
    if (tab.model.isDisposed()) return; // 閉じた後に残った求め
    const version = tab.model.getVersionId();
    let result: AnalysisResult;
    try {
      result = await analyze(tab.model.getValue());
    } catch {
      return;
    }
    if (tab.model.isDisposed() || tab.model.getVersionId() !== version) return; // 古い。次の求めが新しい版を解析する
    tab.analysis = result;
    tab.analysisVersion = version;
    monaco.editor.setModelMarkers(tab.model, "pasta", diagnosticsToMarkers(result.diagnostics));
    semanticSource.changed.fire();
    this.#hooks.onAnalyzed?.(tab);
  }

  /** rename_entry の結果でタブの rel を付け替える。モデルはそのままなので未保存の書き換えと取り消しの履歴が残る（4.15） */
  renamed(oldRel: string, newRel: string): void {
    const moves = retargetTabs(this.tabs.map((t) => ({ id: t.id, rel: t.rel, dirty: isDirty(t) })), oldRel, newRel);
    for (const tab of this.tabs) {
      const rel = moves.get(tab.id);
      if (rel === undefined) continue;
      tab.rel = rel;
      const lang = languageOf(rel);
      if (tab.model.getLanguageId() !== lang) monaco.editor.setModelLanguage(tab.model, lang);
    }
    this.#render();
    this.#renderNotices();
    this.#hooks.onRenamed?.();
  }

  /** 全タブをディスクと比べて 5.2 の判定を当てる（10.2・10.3・10.4・10.6）。呼びが重なっても 1 つずつ走る */
  applyExternal(): Promise<void> {
    const run = () => this.#applyExternal();
    this.#external = this.#external.then(run, run);
    return this.#external;
  }

  async #applyExternal(): Promise<void> {
    let files: Promise<string[]> | null = null;
    let changed = false; // 関係の無い知らせではタブと帯を描き直さない（焦点を落とさない）
    for (const tab of [...this.tabs]) {
      await tab.saving.catch(() => {}); // 自分の保存の途中の中身を外の変更と取り違えない（10.6）
      if (tab.model.isDisposed()) continue;
      let disk: DiskState;
      try {
        disk = { kind: "text", ...(await readText(tab.rel)) };
      } catch (reason) {
        files ??= listTree().then((t) => t.files);
        disk = failedRead(tab.rel, await files.catch(() => [tab.rel]), String(reason)); // 一覧も読めなければ消えたと決めない
      }
      if (tab.model.isDisposed()) continue; // 読んでいる間に閉じた
      const action = decideExternal({ diskText: tab.diskText, diskBom: tab.diskBom, dirty: isDirty(tab), deleted: tab.deleted, editorText: tab.model.getValue() }, disk);
      if (disk.kind !== "text") {
        if (action === "deleted") {
          changed = true;
          tab.deleted = true;
          tab.diskText = null;
          tab.pending = null;
        }
      } else if (action === "reload") {
        changed = true;
        this.#reload(tab, disk);
      } else if (action === "ask") {
        changed ||= !tab.pending;
        tab.pending = { text: disk.text, bom: disk.bom };
      } else {
        changed ||= !!tab.pending || tab.deleted;
        tab.pending = null;
        if (tab.deleted) {
          // 消えた後に同じ中身で戻ってきた。今のディスクの BOM で揃えて未保存でなくする（5.7）
          tab.diskText = disk.text;
          tab.diskBom = disk.bom;
          tab.deleted = false;
          tab.savedVersionId = tab.model.getAlternativeVersionId();
        }
      }
    }
    if (!changed) return;
    this.#render();
    this.#renderNotices();
  }

  /** ディスクの中身へ取り消せる形で置き換え、未保存でなくする（10.2） */
  #reload(tab: Tab, disk: TextFile): void {
    const model = tab.model;
    model.pushStackElement();
    model.pushEditOperations([], [{ range: model.getFullModelRange(), text: disk.text }], () => null);
    // 外で改行が変わったら改行も揃える（pushEOL も取り消せる・5.7）
    const eol = disk.text.includes("\r\n") ? monaco.editor.EndOfLineSequence.CRLF : monaco.editor.EndOfLineSequence.LF;
    if (model.getEndOfLineSequence() !== eol) model.pushEOL(eol);
    model.pushStackElement();
    tab.diskText = disk.text;
    tab.diskBom = disk.bom;
    tab.deleted = false;
    tab.pending = null;
    tab.savedVersionId = model.getAlternativeVersionId();
    this.#render();
  }

  /** 帯の選択。残すときはディスクを今の中身として覚え、未保存のまま（取り消しで戻しても保存済みに見せない） */
  #resolvePending(tab: Tab, reload: boolean): void {
    const disk = tab.pending;
    if (!disk || tab.model.isDisposed()) return;
    if (reload) this.#reload(tab, disk);
    else {
      tab.diskText = disk.text;
      tab.diskBom = disk.bom;
      tab.deleted = false;
      tab.pending = null;
      tab.savedVersionId = -1;
      this.#render();
    }
    this.#renderNotices();
    if (tab === this.active) this.#editor.focus();
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
    monaco.editor.setModelMarkers(tab.model, "pasta", []); // 波線を消す（7.6）
    tab.model.dispose();
    tab.pending = null;
    this.#render();
    this.#renderNotices();
    this.#hooks.onClosed?.(tab);
    return true;
  }

  /** 別のゴーストを開く・窓を閉じる前の 1 回の 3 択（保存する＝未保存を全部保存し、失敗した所で止める）。進めてよければ true（2.8・5.10） */
  async confirmUnsaved(): Promise<boolean> {
    const refs = this.tabs.map((t) => ({ id: t.id, rel: t.rel, dirty: isDirty(t) }));
    if (!needsConfirm(refs)) return true;
    const choice = await confirm3("保存していない変更があります。保存しますか？");
    const saved = choice === "save" ? await saveAll(refs.filter((r) => r.dirty), (r) => this.save(this.tabs.find((t) => t.id === r.id)!)) : null;
    return afterChoice(choice, saved) === "proceed";
  }

  /**
   * 別のゴーストへ移り始める（confirmUnsaved の後、open_ghost の前）。保存の途中を待ってから、確かめずに全部のタブを閉じ、
   * endSwitch までタブを開かない（替わった後に前の中身を新しい master へ書かない）。onClosed は呼ばない（前のゴーストのシーンを読み直さない）
   */
  async beginSwitch(): Promise<void> {
    this.#switching = true;
    await Promise.all(this.tabs.map((t) => t.saving.catch(() => {})));
    this.#generation++;
    for (const tab of this.tabs) {
      monaco.editor.setModelMarkers(tab.model, "pasta", []);
      tab.model.dispose();
      tab.pending = null;
    }
    this.tabs.length = 0;
    this.active = null;
    this.#editor.setModel(null);
    this.#render();
    this.#renderNotices();
  }

  /** open_ghost が終わったら（成功でも失敗でも）呼ぶ */
  endSwitch(): void {
    this.#switching = false;
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
        const gone = tab.deleted ? "（外で消えました）" : "";
        name.textContent = (dirty ? "● " : "") + baseName(tab.rel) + gone;
        name.setAttribute("aria-label", `${tab.rel}${gone}${dirty ? "（未保存）" : ""}`);
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

  /** 外で変わった未保存のタブごとの帯（10.3）。名前はディスクから来るので textContent だけで書く */
  #renderNotices(): void {
    for (const [tab, el] of this.#noticeEls) {
      if (tab.pending && this.tabs.includes(tab)) continue;
      el.remove();
      this.#noticeEls.delete(tab);
    }
    for (const tab of this.tabs) {
      if (!tab.pending) continue;
      const text = `${tab.rel} は外で変わりました:`;
      const el = this.#noticeEls.get(tab);
      if (el) {
        // 名前の変更のときだけ文が変わる。同じなら触らない（読み上げを繰り返さない）
        const span = el.firstElementChild!;
        if (span.textContent !== text) span.textContent = text;
        continue;
      }
      const bar = document.createElement("div");
      bar.className = "notice";
      bar.setAttribute("role", "alert");
      const span = document.createElement("span");
      span.textContent = text;
      const buttons = ([[true, "読み直す"], [false, "今の中身を残す"]] as const).map(([reload, label]) => {
        const b = document.createElement("button");
        b.textContent = label;
        b.addEventListener("click", () => this.#resolvePending(tab, reload));
        return b;
      });
      bar.append(span, ...buttons);
      this.#notices.append(bar);
      this.#noticeEls.set(tab, bar);
    }
  }
}
