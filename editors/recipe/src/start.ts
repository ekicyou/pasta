// 開始の画面（1.5）と、ツールバーの「最近開いたもの」メニュー（3.3）。開くのは自分でせず onOpen に渡す（7.1 の開く流れ・3.5 の消す処理はそちら）。
// 名前・パスはディスクから来るので textContent だけで書く。
import { open as pickFolder } from "@tauri-apps/plugin-dialog";
import { recentList, type RecentEntry } from "./ipc.ts";

export type OpenOrigin = "recent" | "dialog";

/** ボタンの読み上げの名前。名前とフルパスの両方（3.4） */
export const recentLabel = (e: RecentEntry): string => `${e.name}（${e.master}）`;

export class StartView {
  #el: HTMLElement;
  #onOpen: (path: string, origin: OpenOrigin) => void;
  #menuButton: HTMLButtonElement;
  #menu: HTMLElement | null = null;

  constructor(el: HTMLElement, toolbar: HTMLElement, onOpen: (path: string, origin: OpenOrigin) => void) {
    this.#el = el;
    this.#onOpen = onOpen;
    this.#menuButton = document.createElement("button");
    this.#menuButton.textContent = "最近開いたもの";
    this.#menuButton.setAttribute("aria-haspopup", "menu");
    this.#menuButton.setAttribute("aria-expanded", "false");
    this.#menuButton.addEventListener("click", () => (this.#menu ? this.#closeMenu(true) : void this.#openMenu()));
    this.#menuButton.addEventListener("keydown", (e) => {
      if (e.key !== "ArrowDown" || this.#menu) return;
      e.preventDefault();
      void this.#openMenu();
    });
    toolbar.append(this.#pickButton("フォルダを開く…", true), this.#pickButton("ファイルを開く… (Ctrl+O)", false), this.#menuButton);
  }

  /** ゴーストを開いていない間の画面を出す。記録は出すたびに読み直す（開いた後に順が変わる） */
  async show(): Promise<void> {
    const entries = await list();
    const heading = document.createElement("h1");
    heading.textContent = "ゴーストを開く";
    const pick = this.#pickButton("フォルダを選ぶ…", true);
    pick.className = "primary";
    const pickFile = this.#pickButton("ファイルを選ぶ…", false);
    const drop = document.createElement("p");
    drop.textContent = "ここにゴーストのフォルダをドロップしても開けます。";
    const recentHeading = document.createElement("h2");
    recentHeading.id = "start-recent-heading";
    recentHeading.textContent = "最近開いたもの";
    const body: HTMLElement[] = [heading, pick, pickFile, drop, recentHeading];
    if (entries.length === 0) {
      const none = document.createElement("p");
      none.textContent = "まだありません。";
      body.push(none);
    } else {
      const ul = document.createElement("ul");
      ul.setAttribute("aria-labelledby", recentHeading.id);
      for (const entry of entries) {
        const b = document.createElement("button");
        b.className = "recent-entry";
        b.setAttribute("aria-label", recentLabel(entry));
        const name = document.createElement("span");
        name.className = "recent-name";
        name.textContent = entry.name;
        const path = document.createElement("span");
        path.className = "recent-path";
        path.textContent = entry.master;
        b.append(name, path);
        b.addEventListener("click", () => this.#onOpen(entry.master, "recent"));
        const li = document.createElement("li");
        li.append(b);
        ul.append(li);
      }
      body.push(ul);
    }
    this.#el.replaceChildren(...body);
    this.#el.hidden = false;
    pick.focus();
  }

  /** ゴーストを開いた後に呼ぶ */
  hide(): void {
    this.#el.hidden = true;
  }

  #pickButton(label: string, directory: boolean): HTMLButtonElement {
    const b = document.createElement("button");
    b.textContent = label;
    b.addEventListener("click", () => void this.pick(directory));
    return b;
  }

  /** フォルダか、ゴーストの中のファイルを選んで開く。ファイルを選ぶと、そのゴーストを開いてからそのファイルを開く（窓へのドロップと同じ） */
  async pick(directory: boolean): Promise<void> {
    const path = directory
      ? await pickFolder({ directory: true, title: "ゴーストのフォルダを選ぶ" })
      : await pickFolder({ title: "ゴーストのファイルを選ぶ", filters: [{ name: "ゴーストのファイル", extensions: ["pasta", "lua", "toml", "txt"] }] });
    if (typeof path === "string") this.#onOpen(path, "dialog");
  }

  async #openMenu(): Promise<void> {
    const entries = await list();
    if (this.#menu) return; // 読む間に 2 度押された
    const menu = document.createElement("div");
    menu.className = "tree-menu";
    menu.setAttribute("role", "menu");
    menu.setAttribute("aria-label", "最近開いたもの");
    const r = this.#menuButton.getBoundingClientRect();
    menu.style.left = `${r.left}px`;
    menu.style.top = `${r.bottom}px`;
    const items = entries.map((entry) => {
      const b = document.createElement("button");
      b.setAttribute("role", "menuitem");
      b.tabIndex = -1;
      b.textContent = recentLabel(entry);
      b.addEventListener("click", () => {
        this.#closeMenu(true);
        this.#onOpen(entry.master, "recent");
      });
      return b;
    });
    if (items.length === 0) {
      const b = document.createElement("button");
      b.setAttribute("role", "menuitem");
      b.setAttribute("aria-disabled", "true");
      b.tabIndex = -1;
      b.textContent = "まだありません";
      items.push(b);
    }
    menu.append(...items);
    menu.addEventListener("keydown", (e) => {
      const j = items.indexOf(document.activeElement as HTMLButtonElement);
      if (e.key === "ArrowDown") items[(j + 1) % items.length].focus();
      else if (e.key === "ArrowUp") items[(j - 1 + items.length) % items.length].focus();
      else if (e.key === "Escape" || e.key === "Tab") this.#closeMenu(true);
      else return;
      e.preventDefault();
    });
    // 外を押したら閉じる。ボタンへ戻る押下は click の側が閉じる
    menu.addEventListener("focusout", (e) => {
      const to = e.relatedTarget as Node | null;
      if (!menu.contains(to) && to !== this.#menuButton) this.#closeMenu(false);
    });
    document.body.append(menu);
    this.#menu = menu;
    this.#menuButton.setAttribute("aria-expanded", "true");
    items[0].focus();
  }

  #closeMenu(refocus: boolean): void {
    if (!this.#menu) return;
    const menu = this.#menu;
    // 先に外す。焦点のある項目を外すと focusout がこの中へ戻ってくる
    this.#menu = null;
    menu.remove();
    this.#menuButton.setAttribute("aria-expanded", "false");
    if (refocus) this.#menuButton.focus();
  }
}

/** 記録が読めないときも画面は止めない（Rust 側も壊れた記録は空として返す・3.8） */
const list = (): Promise<RecentEntry[]> => recentList().catch(() => []);
