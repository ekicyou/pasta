// DevSession - 拡張とアプリの「1 つの開発の操作」の共通の手順（pasta-recipe-dev-actions の設計の DevSession。要件 1.5・1.6・3.1・3.3〜3.5・4.1・4.3・4.6・5.1・10.1）
//
// 相手の覚えと確かめ・探し直し・2 つ以上の選び・同じゴーストの重なりの止め・見出しの行・保存と失敗での止め・
// 誤りと警告の分け・文言を持つ。画面と保存は DevHost で差し込む（vscode にも Tauri にも触らない）。

import { type CallResult, type Endpoint, HOST, findBasewares, identify } from './mcpClient.ts';
import type { GhostIO } from './ghostIO.ts';
import type { GhostTarget } from './ghostFiles.ts';
import type { FlowContext, Timing } from './reloadShiori.ts';

export const BUSY = 'このゴーストへのコマンドが進んでいます';
export const NOT_FOUND = 'このゴーストが動いている SSP・areka が見つかりません';
export const NOT_FOUND_CHECKS = '確かめること: ベースウェアでこのゴーストを起動しているか・ベースウェアの MCP の窓口が有効か';
export const SAVE_FAILED = '保存できなかったファイルがあるので止めました';
export const pickPlaceHolder = (ghost: GhostTarget) => `${ghost.displayName} へ送る相手を選んでください`;

/** 誤りとして知らせる失敗（それ以外の止まり方は警告） */
export const ERROR_KINDS: ReadonlySet<CallResult['kind']> = new Set(['unreachable', 'badShape', 'rpcError', 'ng']);

export type Report = { lines: string[]; notice?: string; failure?: CallResult };

export interface DevCommand {
  readonly title: string;          // '▶ シーンを実行' など（見出しと進み具合に出る）
  readonly saves: boolean;         // 読み込み直す操作だけが保存する
  readonly run: (ctx: FlowContext) => Promise<Report>;
}

export interface PickItem { readonly label: string; readonly endpoint: Endpoint } // label は `${名前}（127.0.0.1:${port}）`

export interface DevHost {
  readonly io: GhostIO;
  readonly ports: readonly number[];   // 既定 CANDIDATE_PORTS
  readonly timing: Timing;             // 既定 DEFAULT_TIMING
  readonly sleep: (ms: number) => Promise<void>;
  write(lines: readonly string[]): void;                       // 出力へ足す・見せる・焦点を奪わない
  notify(level: 'error' | 'warning', text: string): void;      // 拡張は通知、アプリは出力の目立つ行
  pick(ghost: GhostTarget, items: readonly PickItem[], placeHolder: string): Promise<Endpoint | undefined>;
  progress<T>(title: string, work: () => Promise<T>): Promise<T>;
  save(ghost: GhostTarget): Promise<readonly string[]>;        // 保存できなかったファイル（絶対パス）
}

const pad = (n: number) => String(n).padStart(2, '0');
export function clock(d = new Date()): string {
  return `[${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}]`;
}

export class DevSession {
  // この窓・アプリの間だけ（保存しない。3.5）
  private readonly remembered = new Map<string, Endpoint>();
  private readonly busy = new Map<string, GhostTarget>();
  private readonly host: DevHost;

  constructor(host: DevHost) {
    this.host = host;
  }

  /** 進んでいる操作があるか（アプリはアプリ全体で 1 つずつにするために見る） */
  get running(): GhostTarget | undefined {
    return this.busy.values().next().value;
  }

  /** 重なりの止めの行と知らせを出す */
  refuseBusy(title: string, ghost: GhostTarget): void {
    this.stop([`${clock()} ${title} — ${BUSY}（${ghost.displayName}）`], BUSY);
  }

  /** 見出しの行と知らせを出して止める */
  stop(lines: readonly string[], notice: string, asError = false): void {
    this.host.write(lines);
    this.host.notify(asError ? 'error' : 'warning', notice);
  }

  /** 1 つの操作。相手が無い・選ばずに閉じた・保存の失敗では cmd.run を呼ばない */
  async execute(cmd: DevCommand, ghost: GhostTarget): Promise<void> {
    const host = this.host;
    // 確かめと足し込みの間に await を挟まないので、同じゴーストの 2 本目はここで止まる（5.1）
    if (this.busy.has(ghost.key)) {
      this.refuseBusy(cmd.title, ghost);
      return;
    }
    this.busy.set(ghost.key, ghost);
    try {
      const endpoint = await this.chooseEndpoint(cmd, ghost);
      if (!endpoint) {
        return;
      }
      host.write([`${clock()} ${cmd.title} — ${ghost.displayName}（${ghost.root}）→ ${endpoint.basewareName} ${HOST}:${endpoint.port}`]);
      // 古い辞書を読ませない（1.5）。確かめずに保存し、1 つでも保存できなければ送らない（1.6）
      const unsaved = cmd.saves ? await host.save(ghost) : [];
      if (unsaved.length > 0) {
        this.stop([SAVE_FAILED, ...unsaved.map((f) => `  ${f}`)], `${SAVE_FAILED}: ${unsaved.join('、')}`);
        return;
      }
      const ctx: FlowContext = { io: host.io, endpoint, ghost, timing: host.timing, sleep: host.sleep };
      const report = await host.progress(cmd.title, () => cmd.run(ctx));
      if (report.notice) {
        this.stop(report.lines, report.notice, report.failure !== undefined && ERROR_KINDS.has(report.failure.kind));
      } else {
        host.write(report.lines); // 成功は知らせない
      }
    } finally {
      this.busy.delete(ghost.key);
    }
  }

  /** 覚えた相手を 1 回で確かめ、外れたら探し直す（3.5）。0 個・閉じたら undefined */
  private async chooseEndpoint(cmd: DevCommand, ghost: GhostTarget): Promise<Endpoint | undefined> {
    const host = this.host;
    const head = `${clock()} ${cmd.title} — ${ghost.displayName}（${ghost.root}）`;
    const found = await host.progress(cmd.title, async () => {
      const known = this.remembered.get(ghost.key);
      if (known) {
        const now = await identify(host.io, known.port, ghost.root);
        if (now?.basewareName === known.basewareName) {
          return [known];
        }
        this.remembered.delete(ghost.key);
      }
      return findBasewares(host.io, ghost.root, host.ports);
    });
    if (found.length === 0) {
      this.stop([head, `${NOT_FOUND}（${ghost.displayName}・${ghost.root}）`, NOT_FOUND_CHECKS],
        `${NOT_FOUND}: ${ghost.displayName}（${ghost.root}）。${NOT_FOUND_CHECKS}`);
      return undefined;
    }
    let chosen = found[0];
    if (found.length > 1) {
      const items = found.map((e) => ({ label: `${e.basewareName}（${HOST}:${e.port}）`, endpoint: e }));
      const picked = await host.pick(ghost, items, pickPlaceHolder(ghost));
      if (!picked) {
        return undefined; // 閉じたら何も送らない（3.4）
      }
      chosen = picked;
    }
    this.remembered.set(ghost.key, chosen);
    return chosen;
  }
}
