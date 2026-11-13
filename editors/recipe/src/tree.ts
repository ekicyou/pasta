// ファイルとシーンのツリー（4.1〜4.7・4.14〜4.17）。並びは mapping.ts の treeRows、ここは描いて操作を Documents と Rust へ渡すだけ。
// 名前はディスクから来るので textContent だけで書く。描き直しは行のキーで差分にし、焦点と開いた所を保つ。
import { createEntry, listTree, renameEntry, type EntryKind, type TreeListing } from "./ipc.ts";
import { buildTree, treeRows, type TreeItem, type TreeNode, type TreeRow } from "./mapping.ts";
import { retargetTabs } from "./tabs.ts";
import { notify, prompt } from "./dialog.ts";

/** Documents のうちツリーが使う所 */
export interface TreeDocs {
  open(rel: string, pos?: { lineNumber: number; column: number }): Promise<unknown>;
  renamed(oldRel: string, newRel: string): void;
}

export class TreeView {
  #docs: TreeDocs;
  #list: HTMLUListElement;
  #root: TreeNode | null = null;
  /** rel を大文字にしたキー（Rust の to_uppercase と同じ揃え）→ シーンの項目 */
  #scenes = new Map<string, TreeItem[]>();
  /** 開いた行のキー。最上位は始めから開く */
  #expanded = new Set<string>([""]);
  #rows: TreeRow[] = [];
  #els = new Map<string, HTMLLIElement>();
  /** Tab で入る 1 行（roving tabindex） */
  #current = "";
  #menu: HTMLElement | null = null;

  constructor(el: HTMLElement, docs: TreeDocs) {
    this.#docs = docs;
    this.#list = document.createElement("ul");
    this.#list.setAttribute("role", "tree");
    this.#list.setAttribute("aria-label", "ファイルとシーン");
    el.replaceChildren(this.#list);
    this.#list.addEventListener("click", (e) => {
      const row = this.#rowOf(e.target);
      if (!row) return;
      this.#focus(row.key);
      if ((e.target as Element).closest(".twisty") && row.expanded !== null) this.#toggle(row);
      else this.#activate(row);
    });
    this.#list.addEventListener("contextmenu", (e) => {
      const row = this.#rowOf(e.target);
      if (!row) return;
      e.preventDefault();
      this.#focus(row.key);
      this.#openMenu(row, e.clientX, e.clientY);
    });
    this.#list.addEventListener("keydown", (e) => this.#onKey(e));
    // 焦点の来た行を今の行にする（キーの操作はこの行に当てる）
    this.#list.addEventListener("focusin", (e) => {
      const row = this.#rowOf(e.target);
      if (row && row.key !== this.#current) this.#focus(row.key);
    });
  }

  /** ツリーを読み直す（作成・名前の変更の後と見張りの知らせ）。読めた一覧を返し、読めなければ理由を出して null */
  async refresh(): Promise<TreeListing | null> {
    let listing;
    try {
      listing = await listTree();
    } catch (reason) {
      notify(String(reason));
      return null;
    }
    this.#root = buildTree(listing);
    this.#render();
    return listing;
  }

  /** 別のゴーストへ移るとき、前のゴーストのシーンと開いた所を捨てて読み直す */
  async reset(): Promise<TreeListing | null> {
    this.#scenes.clear();
    this.#expanded = new Set([""]);
    this.#current = "";
    return this.refresh();
  }

  /** そのファイルのシーンの項目を置き換える（書き換え中は 6.3 の onAnalyzed、ほかは開く流れ 7.1 が読んで渡す） */
  setScenes(rel: string, items: TreeItem[]): void {
    this.#scenes.set(rel.toUpperCase(), items);
    this.#render();
  }

  #render(): void {
    if (!this.#root) return;
    const prev = this.#rows;
    this.#rows = treeRows(this.#root, this.#scenes, this.#expanded);
    // 焦点の行が見えなくなったら、前に見えていた親をたどる（無ければ最上位）
    while (!this.#rows.some((r) => r.key === this.#current)) this.#current = prev.find((r) => r.key === this.#current)?.parent ?? "";
    const hadFocus = this.#list.contains(document.activeElement);
    const keys = new Set(this.#rows.map((r) => r.key));
    for (const [key, li] of this.#els) {
      if (keys.has(key)) continue;
      li.remove();
      this.#els.delete(key);
    }
    // 今ある行は動かさず、足りない行だけ差し込む（焦点のある行を外して付け直すと焦点が落ちる）
    this.#rows.forEach((row, i) => {
      let li = this.#els.get(row.key);
      if (!li) {
        li = document.createElement("li");
        li.setAttribute("role", "treeitem");
        const twisty = document.createElement("span");
        twisty.className = "twisty";
        twisty.setAttribute("aria-hidden", "true");
        const label = document.createElement("span");
        li.append(twisty, label);
        this.#els.set(row.key, li);
      }
      if (this.#list.children[i] !== li) this.#list.insertBefore(li, this.#list.children[i] ?? null);
      const label = row.kind === "scene" ? this.#sceneMark(row) + row.name : row.key === "" ? "ghost/master" : row.name;
      set(li.lastElementChild!, "textContent", label);
      set(li.firstElementChild!, "textContent", row.expanded === null ? "" : row.expanded ? "▾" : "▸");
      li.className = `tree-${row.kind}`;
      li.style.paddingLeft = `${(row.level - 1) * 12 + 4}px`;
      attr(li, "aria-level", String(row.level));
      attr(li, "aria-expanded", row.expanded === null ? null : String(row.expanded));
      attr(li, "tabindex", row.key === this.#current ? "0" : "-1");
      attr(li, "title", row.kind === "scene" ? `${row.rel} の ${row.line} 行目` : row.rel || "ghost/master");
    });
    if (hadFocus && !this.#list.contains(document.activeElement)) this.#els.get(this.#current)?.focus();
  }

  /** 見出しの印（親がファイルならグローバル＝＊、シーンならローカル＝・） */
  #sceneMark(row: TreeRow): string {
    return row.parent!.includes("|") ? "・" : "＊";
  }

  #rowOf(target: EventTarget | null): TreeRow | undefined {
    const li = (target as Element | null)?.closest?.("li[role=treeitem]");
    for (const [key, el] of this.#els) if (el === li) return this.#rows.find((r) => r.key === key);
    return undefined;
  }

  #focus(key: string): void {
    this.#current = key;
    for (const [k, li] of this.#els) attr(li, "tabindex", k === key ? "0" : "-1");
    this.#els.get(key)?.focus();
  }

  #toggle(row: TreeRow): void {
    if (row.expanded) this.#expanded.delete(row.key);
    else this.#expanded.add(row.key);
    this.#render();
  }

  /** ファイルはタブで、シーンは見出しの行で開く（4.6・4.7）。フォルダは開け閉め */
  #activate(row: TreeRow): void {
    if (row.kind === "dir") return this.#toggle(row);
    void this.#docs.open(row.rel, row.line === null ? undefined : { lineNumber: row.line, column: 1 });
  }

  #onKey(e: KeyboardEvent): void {
    const i = this.#rows.findIndex((r) => r.key === this.#current);
    const row = this.#rows[i];
    if (!row) return;
    const go = (j: number) => this.#rows[j] && this.#focus(this.#rows[j].key);
    if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
      const r = this.#els.get(row.key)!.getBoundingClientRect();
      this.#openMenu(row, r.left + 16, r.bottom);
    } else if (e.key === "ArrowDown") go(i + 1);
    else if (e.key === "ArrowUp") go(i - 1);
    else if (e.key === "Home") go(0);
    else if (e.key === "End") go(this.#rows.length - 1);
    else if (e.key === "ArrowRight") {
      if (row.expanded === false) this.#toggle(row);
      else if (row.expanded) go(i + 1);
    } else if (e.key === "ArrowLeft") {
      if (row.expanded) this.#toggle(row);
      else if (row.parent !== null) this.#focus(row.parent);
    } else if (e.key === "Enter") this.#activate(row);
    else if (e.key === "F2" && row.kind !== "scene" && row.key !== "") void this.#rename(row);
    else return;
    e.preventDefault();
  }

  /** フォルダ（最上位を含む）は作成、ファイル・フォルダは名前の変更。削除は置かない（4.17）。シーンには無い */
  #openMenu(row: TreeRow, x: number, y: number): void {
    this.#closeMenu(false);
    const actions: [string, () => Promise<void>][] = [];
    if (row.kind === "dir") {
      actions.push(["新しいファイル", () => this.#create(row, "file")], ["新しいフォルダ", () => this.#create(row, "dir")]);
    }
    if (row.kind !== "scene" && row.key !== "") actions.push(["名前の変更", () => this.#rename(row)]);
    if (actions.length === 0) return;
    const menu = document.createElement("div");
    menu.className = "tree-menu";
    menu.setAttribute("role", "menu");
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    const items = actions.map(([label, run]) => {
      const b = document.createElement("button");
      b.setAttribute("role", "menuitem");
      b.tabIndex = -1;
      b.textContent = label;
      b.addEventListener("click", () => {
        // 先に行へ焦点を戻す。窓（prompt・notify）を閉じた後、焦点がそこへ戻る
        this.#closeMenu(true);
        void run();
      });
      return b;
    });
    menu.append(...items);
    // メニューのキーで開くと、続く contextmenu は焦点の移ったこのメニューへ来る。既定のメニューを重ねない
    menu.addEventListener("contextmenu", (e) => e.preventDefault());
    menu.addEventListener("keydown", (e) => {
      const j = items.indexOf(document.activeElement as HTMLButtonElement);
      if (e.key === "ArrowDown") items[(j + 1) % items.length].focus();
      else if (e.key === "ArrowUp") items[(j - 1 + items.length) % items.length].focus();
      else if (e.key === "Escape" || e.key === "Tab") this.#closeMenu(true);
      else return;
      e.preventDefault();
    });
    // 外を押したら閉じる（中のボタンの click より先に来ないよう、menu の中は除く）
    menu.addEventListener("focusout", (e) => {
      if (!menu.contains(e.relatedTarget as Node | null)) this.#closeMenu(false);
    });
    document.body.append(menu);
    this.#menu = menu;
    items[0].focus();
  }

  #closeMenu(refocus: boolean): void {
    if (!this.#menu) return;
    const menu = this.#menu;
    this.#menu = null;
    menu.remove();
    if (refocus) this.#els.get(this.#current)?.focus();
  }

  /** 空のファイル・フォルダを作り、ツリーに出し、ファイルならタブで開く（4.14）。名前の誤りは Rust の理由を出す（4.16） */
  async #create(row: TreeRow, kind: EntryKind): Promise<void> {
    const name = await prompt(kind === "file" ? "新しいファイルの名前" : "新しいフォルダの名前", "");
    if (name === null) return this.#focus(row.key);
    let rel: string;
    try {
      rel = await createEntry(row.rel, name, kind);
    } catch (reason) {
      notify(String(reason));
      return;
    }
    this.#expanded.add(row.key);
    await this.refresh();
    if (kind === "file") void this.#docs.open(rel);
    else this.#focus(rel);
  }

  /** 名前を変え、タブ・シーン・開いた所を新しい rel へ付け替える（4.15・4.16） */
  async #rename(row: TreeRow): Promise<void> {
    const name = await prompt(`${row.name} の新しい名前`, row.name);
    if (name === null || name === row.name) return this.#focus(row.key);
    let rel: string;
    try {
      rel = await renameEntry(row.rel, name);
    } catch (reason) {
      notify(String(reason));
      return;
    }
    this.#docs.renamed(row.rel, rel);
    // キーは rel か rel|シーン。rel の所だけ付け替える（フォルダならその中も・大文字小文字を区別しない）
    const moved = (key: string) => {
      const cut = key.includes("|") ? key.indexOf("|") : key.length;
      const to = retargetTabs([{ id: 0, rel: key.slice(0, cut), dirty: false }], row.rel, rel).get(0);
      return to === undefined ? key : to + key.slice(cut);
    };
    // .pasta でなくなったファイルのシーンは捨てる（4.4）
    this.#scenes = new Map(
      [...this.#scenes].map(([k, v]) => [moved(k).toUpperCase(), v] as const).filter(([k]) => k.endsWith(".PASTA")),
    );
    this.#expanded = new Set([...this.#expanded].map(moved));
    this.#current = moved(this.#current);
    await this.refresh();
    this.#focus(this.#current);
  }
}

/** 同じなら触らない（読み上げ直しを起こさない） */
function set(el: Element, prop: "textContent", value: string): void {
  if (el[prop] !== value) el[prop] = value;
}
function attr(el: Element, name: string, value: string | null): void {
  if (value === null) el.removeAttribute(name);
  else if (el.getAttribute(name) !== value) el.setAttribute(name, value);
}
