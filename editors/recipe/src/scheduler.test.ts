/// <reference types="node" />
import { test } from "node:test";
import assert from "node:assert/strict";
import { CoalescingRunner, eachLimit, type Clock } from "./scheduler.ts";

/** 手で進める時計 */
function fakeClock() {
  let now = 0;
  let timers: { at: number; fn: () => void }[] = [];
  const clock: Clock = {
    setTimeout(fn, ms) {
      const t = { at: now + ms, fn };
      timers.push(t);
      return t;
    },
    clearTimeout(h) {
      timers = timers.filter((t) => t !== h);
    },
  };
  const advance = (ms: number) => {
    now += ms;
    for (const t of timers.filter((t) => t.at <= now)) {
      timers = timers.filter((x) => x !== t);
      t.fn();
    }
  };
  return { clock, advance };
}

/** 終わりを手で決められる run */
function manualRun(fail = false) {
  let calls = 0;
  let finish: () => void = () => {};
  const run = () => {
    calls++;
    return new Promise<void>((ok, ng) => {
      finish = () => (fail ? ng(new Error("失敗")) : ok());
    });
  };
  return { run, calls: () => calls, finish: () => finish() };
}
const flush = () => new Promise((r) => setImmediate(r));

test("300 ms 以内の 3 回の求めは 1 回にまとまる", async () => {
  const { clock, advance } = fakeClock();
  const r = manualRun();
  const runner = new CoalescingRunner(r.run, 300, clock);
  runner.request();
  advance(100);
  runner.request();
  advance(100);
  runner.request();
  advance(299);
  await flush();
  assert.equal(r.calls(), 0);
  advance(1);
  await flush();
  assert.equal(r.calls(), 1);
  r.finish();
  await flush();
  advance(1000);
  await flush();
  assert.equal(r.calls(), 1);
});

test("走っている間の 2 回の求めは、終わった後の 1 回になる", async () => {
  const { clock, advance } = fakeClock();
  const r = manualRun();
  const runner = new CoalescingRunner(r.run, 300, clock);
  runner.request();
  advance(300);
  await flush();
  assert.equal(r.calls(), 1);
  runner.request();
  advance(300);
  runner.request();
  advance(300);
  await flush();
  assert.equal(r.calls(), 1, "走っている間は重ねて走らせない");
  r.finish();
  await flush();
  advance(1000);
  await flush();
  assert.equal(r.calls(), 2);
  r.finish();
  await flush();
  advance(1000);
  await flush();
  assert.equal(r.calls(), 2);
});

test("run が失敗しても止まらず、待っていた求めを走らせる", async () => {
  const { clock, advance } = fakeClock();
  const r = manualRun(true);
  const runner = new CoalescingRunner(r.run, 300, clock);
  runner.request();
  advance(300);
  await flush();
  runner.request();
  advance(300);
  r.finish();
  await flush();
  advance(1000);
  await flush();
  assert.equal(r.calls(), 2);
  r.finish();
  await flush();
  runner.request();
  advance(300);
  await flush();
  assert.equal(r.calls(), 3);
});

test("run がその場で投げても止まらない", async () => {
  const { clock, advance } = fakeClock();
  let calls = 0;
  const runner = new CoalescingRunner(
    () => {
      calls++;
      throw new Error("同期の失敗");
    },
    300,
    clock,
  );
  runner.request();
  advance(300);
  await flush();
  runner.request();
  advance(300);
  await flush();
  assert.equal(calls, 2);
});

test("eachLimit はすべてを 1 回ずつ、同時に limit 個までで走らせる（Performance）", async () => {
  let running = 0;
  let peak = 0;
  const seen: number[] = [];
  await eachLimit([1, 2, 3, 4, 5, 6, 7], 3, async (n) => {
    running++;
    peak = Math.max(peak, running);
    await new Promise((r) => setTimeout(r, 1));
    seen.push(n);
    running--;
  });
  assert.deepEqual(seen.sort(), [1, 2, 3, 4, 5, 6, 7]);
  assert.equal(peak, 3);
});
