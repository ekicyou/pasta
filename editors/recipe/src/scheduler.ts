// 純粋: 続けて来る求めを 1 回の実行にまとめる（DOM・Monaco・Tauri を import しない）

export interface Clock {
  setTimeout(fn: () => void, ms: number): unknown;
  clearTimeout(h: unknown): void;
}

export class CoalescingRunner {
  #run: () => Promise<void>;
  #delayMs: number;
  #clock: Clock;
  #timer: unknown = undefined;
  #running = false;
  #pending = false;

  constructor(run: () => Promise<void>, delayMs: number, clock: Clock) {
    this.#run = run;
    this.#delayMs = delayMs;
    this.#clock = clock;
  }

  /** delayMs 以内の求めは 1 回に。走っている間の求めは、終わった後に 1 回だけ */
  request(): void {
    if (this.#timer !== undefined) this.#clock.clearTimeout(this.#timer);
    this.#timer = this.#clock.setTimeout(() => this.#fire(), this.#delayMs);
  }

  #fire(): void {
    this.#timer = undefined;
    if (this.#running) {
      this.#pending = true;
      return;
    }
    this.#running = true;
    // 失敗の知らせは run の側の持ち分。ここでは握りつぶして止まらないことだけを守る
    Promise.resolve()
      .then(this.#run)
      .catch(() => {})
      .finally(() => {
        this.#running = false;
        if (this.#pending) {
          this.#pending = false;
          this.#fire();
        }
      });
  }
}
