// DevActions - アプリの開発の操作の糊（pasta-recipe-dev-actions の設計の DevActions。拡張の devCommands.ts と同じ立場）
//
// DevHost を画面の部品（出力の欄・相手を選ぶ窓・タブの保存）で満たし、共有の DevSession へ渡す。
// 操作はアプリ全体で 1 つずつ（5.1）。始めたときのゴーストのまま、devBegin → execute → devEnd で包む（5.2）。
import { CANDIDATE_PORTS } from "../../shared/mcpClient.ts";
import { ghostTarget } from "../../shared/ghostFiles.ts";
import { DEFAULT_TIMING, formatReload, reloadAndWait } from "../../shared/reloadShiori.ts";
import { BUSY, type DevCommand, type DevHost, DevSession, clock } from "../../shared/devSession.ts";
import type { GhostTarget } from "../../shared/ghostFiles.ts";
import { devBegin, devEnd, tauriGhostIO as io, type GhostInfo } from "./ipc.ts";
import { pickEndpoint } from "./devDialogs.ts";
import { isDevSource, isDirty } from "./tabs.ts";
import type { Documents } from "./documents.ts";
import type { OutputPanel, VariablesPanel } from "./devPanels.ts";
import type { Panels } from "./panels.ts";

export const noDescript = (master: string): string =>
  `ghost/master に descript.txt が無いので、ゴーストのフォルダとして扱えません（${master}）`;

export interface DevActionsDeps {
  readonly docs: Pick<Documents, "tabs" | "save">;
  readonly output: Pick<OutputPanel, "append" | "notify" | "progress">;
  readonly vars: Pick<VariablesPanel, "clear">;
  readonly panels: Pick<Panels, "show">;
}

const RELOAD: DevCommand = {
  title: "SHIORI をリロード",
  saves: true,
  run: async (ctx) => {
    const o = await reloadAndWait(ctx);
    return { ...formatReload(o), failure: o.kind === "refused" ? o.result : undefined };
  },
};

export class DevActions {
  #deps: DevActionsDeps;
  #session: DevSession;
  #current: GhostInfo | null = null;
  /** 進んでいる操作のゴーストの名前。確かめと同期に立てる（間に await がある。tasks の 2.3 の注） */
  #busy: string | null = null;

  constructor(deps: DevActionsDeps) {
    this.#deps = deps;
    const { output, panels } = deps;
    const host: DevHost = {
      io,
      ports: CANDIDATE_PORTS,
      timing: DEFAULT_TIMING,
      sleep: (ms) => new Promise<void>((r) => setTimeout(r, ms)),
      write: (lines) => {
        output.append(lines);
        panels.show("run"); // 焦点は動かさない（4.2）
      },
      notify: (level, text) => {
        output.notify(level, text);
        panels.show("run");
      },
      pick: (_ghost, items, placeHolder) => pickEndpoint(placeHolder, items),
      progress: (title, work) => output.progress(title, work),
      save: (ghost) => this.#save(ghost),
    };
    this.#session = new DevSession(host);
  }

  /** ゴーストを開いた後に渡す。替わったら前の表を消す（5.3） */
  setGhost(info: GhostInfo | null): void {
    if (info?.master !== this.#current?.master) this.#deps.vars.clear();
    this.#current = info;
  }

  reload(): Promise<void> {
    return this.#execute(RELOAD);
  }

  async #execute(cmd: DevCommand): Promise<void> {
    const info = this.#current;
    if (info === null) return; // 開始の画面ではボタンが出ない（2.5）
    if (this.#busy !== null) {
      this.#session.stop([`${clock()} ${cmd.title} — ${BUSY}（${this.#busy}）`], BUSY);
      return;
    }
    this.#busy = info.name;
    try {
      await devBegin();
      try {
        // open_ghost の綴りの master・root で結ぶ（門は綴りで比べる。tasks の 3.2 の注）
        const ghost = await ghostTarget(io, info.root, info.master, info.name);
        if (!ghost) {
          this.#session.stop([`${clock()} ${cmd.title} — ${noDescript(info.master)}`], noDescript(info.master));
          return;
        }
        this.#busy = ghost.displayName;
        await this.#session.execute(cmd, ghost);
      } finally {
        await devEnd().catch(() => {});
      }
    } catch (reason) {
      this.#session.stop([`${clock()} ${cmd.title} — ${reason}`], String(reason), true);
    } finally {
      this.#busy = null;
    }
  }

  /** 開いているゴーストの未保存の .pasta・.lua・pasta.toml を順に保存し、保存できなかったものの絶対パスを返す（1.5・1.6） */
  async #save(ghost: GhostTarget): Promise<string[]> {
    // 操作の途中で別のゴーストへ移っていれば、前のゴーストのタブは切り替えの 3 択で片付いている
    if (this.#current?.master !== ghost.master) return [];
    const failed: string[] = [];
    for (const tab of this.#deps.docs.tabs.filter((t) => isDirty(t) && isDevSource(t.rel))) {
      if (!(await this.#deps.docs.save(tab).catch(() => false))) failed.push(io.join(ghost.master, ...tab.rel.split("/")));
    }
    return failed;
  }
}
