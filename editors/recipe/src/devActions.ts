// DevActions - アプリの開発の操作の糊（pasta-recipe-dev-actions の設計の DevActions。拡張の devCommands.ts と同じ立場）
//
// DevHost を画面の部品（出力の欄・相手を選ぶ窓・タブの保存）で満たし、共有の DevSession へ渡す。
// 操作はアプリ全体で 1 つずつ（5.1）。始めたときのゴーストのまま、devBegin → execute → devEnd で包む（5.2）。
import { CANDIDATE_PORTS } from "../../shared/mcpClient.ts";
import { ghostTarget } from "../../shared/ghostFiles.ts";
import { DEFAULT_TIMING, type FlowContext, formatReload, reloadAndWait } from "../../shared/reloadShiori.ts";
import { BUSY, type DevCommand, type DevHost, DevSession, type Report, clock } from "../../shared/devSession.ts";
import type { GhostTarget } from "../../shared/ghostFiles.ts";
import { formatScene, runScene } from "../../shared/runSceneAtCursor.ts";
import { dumpVariables, formatDump, formatSet, setSaveVariable, type SaveVariableChange } from "../../shared/dumpVariables.ts";
import { analyze, devBegin, devEnd, readText, tauriGhostIO as io, variableRows, type GhostInfo, type LocalPair, type VarRow } from "./ipc.ts";
import { confirmDelete, editVariable, localsDialog, pickEndpoint } from "./devDialogs.ts";
import { findSame, isDevSource, isDirty } from "./tabs.ts";
import { sceneLine, type SceneTarget } from "./mapping.ts";
import type { Documents } from "./documents.ts";
import type { OutputPanel, VariablesPanel } from "./devPanels.ts";
import type { Panels } from "./panels.ts";

export const noDescript = (master: string): string =>
  `ghost/master に descript.txt が無いので、ゴーストのフォルダとして扱えません（${master}）`;

export type { SceneTarget };

export const notFoundInSaved = (label: string, rel: string): string => `シーン「${label}」が保存した中身に見つかりません（${rel}）`;
export const switchedMidOp = "操作の途中で別のゴーストを開いたので、保存した中身を読めません";
export const notToml =(path: string, reason: string): string => `${path} を TOML として読めませんでした: ${reason}`;

export interface DevActionsDeps {
  readonly docs: Pick<Documents, "tabs" | "save">;
  readonly output: Pick<OutputPanel, "append" | "notify" | "progress">;
  readonly vars: Pick<VariablesPanel, "state" | "clear" | "setRows" | "setUnreadable">;
  readonly panels: Pick<Panels, "show">;
  /** 窓（試験で差し替える） */
  readonly dialogs?: { editVariable: typeof editVariable; confirmDelete: typeof confirmDelete; localsDialog: typeof localsDialog };
}

/** 2 つの報告を 1 つの操作の行に。知らせは先のものを採る。ただし後だけが失敗なら、誤りを警告へ下げないよう後を採る */
export const merge = (a: Report, b: Report): Report =>
  a.notice !== undefined && (a.failure !== undefined || b.failure === undefined)
    ? { ...a, lines: [...a.lines, ...b.lines] }
    : { ...b, lines: [...a.lines, ...b.lines] };

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
  #dialogs: NonNullable<DevActionsDeps["dialogs"]>;
  /** シーンごとに最後に鳴らした組（鍵は ghost.key|rel を大文字|道。ファイルに残さない 8.10） */
  #locals = new Map<string, LocalPair[]>();

  constructor(deps: DevActionsDeps) {
    this.#deps = deps;
    this.#dialogs = deps.dialogs ?? { editVariable, confirmDelete, localsDialog };
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

  get current(): GhostInfo | null {
    return this.#current;
  }

  reload(): Promise<void> {
    return this.#execute(RELOAD);
  }

  /** ▶: 保存の後の中身を解析し直し、道から今の行を引いて鳴らす（1.1・1.2・1.4）。ローカル変数は送らない（8.11） */
  run(target: SceneTarget): Promise<void> {
    return this.#play("▶ シーンを実行", target);
  }

  /** 値を入れて実行: 覚えた組で窓を開き、決めた組を覚えて鳴らす。組が 0 なら Reference2 を付けない（8.1・8.3・8.5・8.10） */
  async runWithLocals(target: SceneTarget): Promise<void> {
    const info = this.#current;
    if (info === null) return;
    const key = `${info.root.toLowerCase()}|${target.rel.toUpperCase()}|${JSON.stringify(target.path)}`;
    const r = await this.#dialogs.localsDialog(target.label, this.#locals.get(key) ?? []);
    if (r === null || this.#current?.master !== info.master) return; // 窓の間に別のゴーストを開いたら鳴らさない
    this.#locals.set(key, r.pairs);
    return this.#play("値を入れて実行", target, r.table);
  }

  #play(title: string, target: SceneTarget, locals?: string): Promise<void> {
    return this.#execute({
      title,
      saves: true,
      run: async (ctx) => {
        // 保存は済んでいる。タブがあれば保存の後の中身、無ければディスクの中身。
        // タブも read_text も今開いているゴーストのものなので、操作のゴーストでなくなったら読まない（5.2）
        const switched = () => this.#current?.master !== ctx.ghost.master;
        const stop = { lines: [switchedMidOp], notice: switchedMidOp };
        if (switched()) return stop;
        const text = findSame(this.#deps.docs.tabs, target.rel)?.diskText ?? (await readText(target.rel)).text;
        if (switched()) return stop;
        const line = sceneLine((await analyze(text)).scenes, target.path);
        if (line === undefined) {
          const msg = notFoundInSaved(target.label, target.rel);
          return { lines: [msg], notice: msg };
        }
        const o = await runScene(ctx, io.join(ctx.ghost.master, ...target.rel.split("/")), line, locals);
        const failure = o.kind === "failed" ? o.result : o.reload.kind === "refused" ? o.reload.result : undefined;
        const report = { ...formatScene(o), failure };
        // 表を読めている間は、鳴らした後に読み直す（6.9）
        return o.kind !== "notPlayed" && this.#deps.vars.state.kind === "rows" ? merge(report, await this.#readTable(ctx)) : report;
      },
    });
  }

  /** 書き換え（7.1・7.2） */
  async editVariable(row: VarRow): Promise<void> {
    if (row.kind === "table") return; // 表の行には操作が出ない（6.3）
    const master = this.#current?.master;
    const v = await this.#dialogs.editVariable({ ...row, kind: row.kind });
    if (v) await this.#change(master, v.name, { type: v.kind, value: v.value });
  }

  /** 削除: 確かめてから型 delete で送る（7.3） */
  async deleteVariable(row: VarRow): Promise<void> {
    const master = this.#current?.master;
    if (await this.#dialogs.confirmDelete(row.name)) await this.#change(master, row.name, { type: "delete" });
  }

  /** 追加: 入れた名前で送る（7.4・7.5 は窓の中） */
  async addVariable(): Promise<void> {
    const master = this.#current?.master;
    const v = await this.#dialogs.editVariable(null);
    if (v) await this.#change(master, v.name, { type: v.kind, value: v.value });
  }

  /** 書き換えを頼み、成功でも断りでも届かなくても同じ操作の中で表を読み直す（7.6〜7.8）。ファイルは書かない（7.9） */
  async #change(master: string | undefined, name: string, change: SaveVariableChange): Promise<void> {
    if (this.#current?.master !== master) return; // 窓の間に別のゴーストを開いたら、前の表の名前を送らない
    return this.#execute({
      title: "保存変数を書き換え",
      saves: false,
      run: async (ctx) => merge(formatSet(name, change, await setSaveVariable(ctx, name, change)), await this.#readTable(ctx)),
    });
  }

  /** 保存変数を表示: 書き出しを待って表へ当てる（6.1・6.4・6.5）。操作の間にゴーストが替わっていれば当てない（5.3） */
  showVariables(): Promise<void> {
    return this.#execute({
      title: "保存変数を表示",
      saves: false,
      run: (ctx) => this.#readTable(ctx),
    });
  }

  /** 書き出しを待って表へ当てる。操作のゴーストが今のものでなければ当てない（5.3） */
  async #readTable(ctx: FlowContext): Promise<Report> {
    const o = await dumpVariables(ctx);
    const current = () => this.#current?.master === ctx.ghost.master;
    const unreadable = () => current() && this.#deps.vars.setUnreadable();
    if (o.kind !== "shown") {
      unreadable();
      return { ...formatDump(o), failure: o.kind === "failed" ? o.result : undefined };
    }
    let rows;
    try {
      rows = await variableRows(o.text);
    } catch (reason) {
      unreadable();
      const msg = notToml(o.path, String(reason));
      return { lines: [msg], notice: msg };
    }
    if (current()) this.#deps.vars.setRows(rows);
    return { lines: [`保存変数（${o.path}）を表に出しました`] };
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
