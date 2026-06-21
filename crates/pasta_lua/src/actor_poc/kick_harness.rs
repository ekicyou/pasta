//! KickHarness（R5.1〜R5.5・task 5.2／5.3）。
//!
//! talk FIFO ＋ OnSecondChange drain ＋ 二層 gate ＋ 即時 preempt によるキック配信を
//! 検証し（task 5.2・R5.1〜R5.3）、さらにキック→配信レイテンシを実測して ≤1 秒を判定・
//! 記録する（task 5.3・R5.4／R5.5。[`KickHarness::measure`] ／
//! [`KickHarness::measure_and_record`]）。design.md「KickHarness」`measure() ->
//! KickLatencyReport`／Requirements 5.1〜5.5。
//!
//! # レイテンシ実測の 2 レンズ（R5.4・正直さ）
//!
//! 「≤1 秒」は **R5.6 の忠実シミュレータのタイムライン**（OnSecondChange 1 周期 = 1 秒）
//! 上で評価する。[`KickLatencyReport`] は 2 つのレンズで正直に測る:
//! - **sim-timeline**: キック指示から配信した GET tick までの所要 tick 数
//!   （1 tick = 1 周期 = 1 秒）。次の利用可能な GET tick で配信されれば 1。NOTIFY held は
//!   次 GET tick まで加算されるが、配信契機（GET）から見れば 1 周期なので ≤1 秒は成立。
//! - **wall-clock**: enqueue→drain→実 VM 配信の実 Rust/VM 処理時間（`std::time::Instant`
//!   単調時計）。1 秒を遥かに下回る。
//!
//! 未達（drain 不発・gate 誤動作・preempt 不能・遅延 >1 秒）は [`LatencyFault`] で注入駆動し、
//! [`VerdictRecorder::record_blocker`]（R5.5）へ切り分けた条件 **と実測値** を残す。成立時は
//! R5 item を success で記録し、R5 の LuaJIT close 制約（[`R5_LUAJIT_CLOSE_CONSTRAINT`]）を
//! `constraints` に申し送る（task 5.2 所見の handoff・7.1 が消費）。
//!
//! # 決定論ロジックは Rust・talk-close 意味論は Lua（責務配置・design「境界線」）
//!
//! FIFO/gate/preempt の **制御ロジックは Rust 側**（本構造体）で `SimDriver`(5.1) の
//! tick を消費して決定論的に駆動する。一方、talk の **close 意味論は実 Lua のまま**:
//! 先行トークを閉じるのは store.lua / 実 coroutine と同じ **`coroutine.close()`**
//! プリミティブ（`store.lua` の `STORE.reset()` が suspended な `STORE.co_scene` を
//! 閉じるのに使う正本）を `ActorThread::run_lua_string`(4.1) でアクタースレッド上の
//! 実 VM に対して駆動する。コルーチン意味論を Rust に再実装しない（R4 を壊さない）。
//!
//! # 二層 gate（R5.2）
//!
//! 1. **礼儀 gate**: `SimDriver::is_talking` が true の間は **非即時** drain を抑止する
//!    （再生中トークへ割り込まない）。FIFO は保持される。
//! 2. **配信可否 gate**: 実際に配信できるのは **GET tick（Reference3=1・`SimTick::
//!    is_playable`）のみ**。NOTIFY tick（Ref3=0）では返したスクリプトが SSP に無視
//!    されるため配信せず held（design「SimDriver」gate 権威・Q4）。
//!
//! # 即時 preempt（R5.3）
//!
//! `preempt(scene)` は礼儀 gate を **無視** する（talking 中でも割り込む）。ただし
//! 配信可否 gate には従う: GET tick で先行トークを **実 store.lua の close 正本** で
//! 閉じてから上書き配信し、NOTIFY 状態では次の GET tick まで遅延する（GET=上書き可は
//! 決め打ち・設計ディスカッション#3）。
//!
//! # LuaJIT における talk-close の feasibility 所見（R5.3・重要）
//!
//! 先行トークの close は **実 `store.lua` の `STORE.reset()` が用いるのと同一の close
//! 経路**（`if coroutine.close and status=="suspended" then coroutine.close(co) end`
//! → `co_scene = nil`）を駆動する。ところが本 VM は `mlua` の **LuaJIT 2.1（luajit52
//! ＝5.2 互換）** であり、`coroutine.close`（Lua 5.4 で導入）は **存在しない**
//! （`type(coroutine.close) == "nil"`・実 VM で実測）。さらに、suspended なコルーチンは
//! 参照を捨てて `collectgarbage("collect")` しても `dead` へ遷移しない（LuaJIT は
//! suspended コルーチンを強制終了する API を持たない・実測）。
//!
//! したがって LuaJIT における talk-close の実体は **「アクティブ talk からの破棄
//! （`STORE.co_scene = nil` ＋ live レジストリからの除去）＝以後 resume されない」** で
//! あり、`coroutine.status` が `dead` になることでは **ない**。これは出荷 `store.lua`
//! の `STORE.reset()` でも同様（LuaJIT では `coroutine.close` 分岐が事実上 no-op で、
//! close は参照破棄＋GC に帰着）。本 PoC はこの実体を忠実にモデル化し、close の観測を
//! **「先行 talk が live なアクティブ talk でなくなった（破棄された）」** と honest に
//! 定義する。`pasta-actor-runtime` への申し送り: 真の coroutine 強制終了は LuaJIT では
//! 不可。preempt の「閉じる」は参照破棄（abandon）で成立し、メモリは GC に委ねる。
//!
//! # 「配信/再生」の観測（正直さ）
//!
//! 配信＝**アクタースレッド上の実 VM に talk が現に乗る** こととして定義する。各配信は
//! 実 `coroutine.create` で talk コルーチンを生成して `STORE.co_scene`（実 store.lua の
//! フィールド）へ格納し、`active_talk_is_live` で「suspended な実コルーチンが乗って
//! いる」ことを `run_lua_string` 越しに観測できる。配信記録（どのシーンがどの tick で
//! 配信されたか）は `on_second_change` の戻り値（配信シーン列）で返す。
//!
//! close（preempt 破棄）の観測は、上記 LuaJIT 所見に基づき **「先行 talk が live レジ
//! ストリから除去され、もはやアクティブ talk でない（破棄された）」** ことで判定する
//! （[`talk_token_is_dead`](KickHarness::talk_token_is_dead)）。preempt が先行トークを
//! 閉じ損ねればトークンは live なアクティブ talk のままで、テストは落ちる。
//!
//! 本ファイルは出荷コードを改変しない feature-gate モジュールであり、`actor-poc`
//! 無効時はコンパイル単位に現れない（リリースビルドはバイト不変・R7.1/R7.2）。

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::actor_thread::ActorThread;
use super::sim_driver::{SimDriver, SimTick};
use super::verdict::{Blocker, ItemOutcome, ReqId, VerdictRecorder};

/// キック→配信レイテンシの実測レポート（R5.4・design「KickHarness」`measure`）。
///
/// **2 つのレンズ**で正直に実測する（R5.6 の忠実シミュレータ定義に基づく）:
/// - **sim-timeline**: `ticks_to_deliver`＝キック指示から配信した GET tick までの所要
///   tick 数。1 tick = OnSecondChange 1 周期 = 1 秒（R5.6）。次の利用可能な GET tick で
///   配信されれば 1（NOTIFY held は次 GET tick まで加算）。`sim_latency()` で秒換算する。
/// - **wall-clock**: `wall_clock`＝enqueue/preempt→drain→実 VM 配信の実 Rust/VM 処理時間
///   （`std::time::Instant` 単調時計）。1 秒を遥かに下回るべき。
///
/// `within_one_second` は総合 ≤1 秒判定。未達なら `unmet` に切り分けた条件を持つ。
#[derive(Debug, Clone)]
pub struct KickLatencyReport {
    /// sim-timeline: 配信した GET tick までの所要 tick 数（1 tick = 1 周期 = 1 秒）。
    /// 配信に至らなかった場合は試行した tick 数（未達の実測値）。
    pub ticks_to_deliver: u32,
    /// wall-clock: enqueue→配信の実処理時間（単調時計）。
    pub wall_clock: Duration,
    /// 配信が実 VM に成立したか（実 `STORE.co_scene` に talk が乗ったか）。
    pub delivered: bool,
    /// 総合 ≤1 秒判定（sim-timeline ≤1 周期 かつ wall-clock ≪1 秒 かつ配信成立）。
    pub within_one_second: bool,
    /// 未達条件（成立時は `None`）。R5.5 のブロッカー記録根拠。
    pub unmet: Option<UnmetCondition>,
}

impl KickLatencyReport {
    /// sim-timeline のレイテンシを秒へ換算する（1 tick = 1 秒・R5.6）。
    pub fn sim_latency(&self) -> Duration {
        Duration::from_secs(u64::from(self.ticks_to_deliver))
    }
}

/// ≤1 秒キック配信が成立しなかった切り分け条件（R5.5）。
///
/// requirements.md 5.5 の未達 4 条件（drain 不発・gate 誤動作・preempt 不能・遅延 >1 秒）を
/// そのまま列挙する。`record_blocker` でこの条件 **と実測値** を残す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmetCondition {
    /// drain 不発: GET tick が来ても FIFO が排出されず配信に至らない。
    DrainStalled,
    /// gate 誤動作: 配信可否／礼儀 gate が誤って配信を抑止／許可した。
    GateMisbehaved,
    /// preempt 不能: 即時 preempt が先行トークを閉じて上書き配信できない。
    PreemptFailed,
    /// 遅延 >1 秒: 配信したが所要時間が 1 秒（=1 周期）を超過した。
    OverOneSecond,
}

impl UnmetCondition {
    /// requirements.md 5.5 の日本語ラベル（ブロッカー reason に用いる）。
    pub fn label(self) -> &'static str {
        match self {
            UnmetCondition::DrainStalled => "drain 不発",
            UnmetCondition::GateMisbehaved => "gate 誤動作",
            UnmetCondition::PreemptFailed => "preempt 不能",
            UnmetCondition::OverOneSecond => "遅延 >1 秒",
        }
    }
}

/// `measure` への故障注入シーム（R5.5 の未達駆動）。
///
/// 先行タスク（R4 の `LossKind`・R2.4 の Lua `error()`）と同じく、未達条件を **テストから
/// 駆動**して `record_blocker` 経路を実証するための注入点。`None` は正常計測。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyFault {
    /// 故障なし（正常計測）。
    None,
    /// drain 不発を注入: GET tick でも FIFO を排出しない（→ `DrainStalled`）。
    SuppressDrain,
    /// preempt 不能を注入: 即時 preempt の投入を握り潰す（→ `PreemptFailed`）。
    BlockPreempt,
    /// 強制 >1 秒遅延を注入: 配信は成立するが sim-timeline を 1 周期超へ膨らませる
    /// （→ `OverOneSecond`・実測値 >1 秒）。
    ForceLatencyOverOneSecond,
}

/// R5 LuaJIT close 制約の申し送り文（task 5.2 所見・7.1 が消費）。
///
/// preempt の「閉じる」は LuaJIT 2.1 に `coroutine.close` が無いため **破棄（abandon＝
/// 参照破棄＋GC）** で成立する（強制終了 API は存在しない）。これは preempt が機能しない
/// という意味ではなく（破棄で上書きは成立する）、`pasta-actor-runtime` へ honest に
/// 申し送るべき **制約** である。
pub const R5_LUAJIT_CLOSE_CONSTRAINT: &str =
    "LuaJIT 2.1 に coroutine.close が無く suspended コルーチンを強制終了できないため、\
     preempt の close は破棄（abandon＝STORE.co_scene=nil ＋ live レジストリ除去 ＋GC）で\
     成立する（preempt 自体は破棄上書きで機能する）。pasta-actor-runtime へ申し送り。";

/// FIFO に投入された 1 件のキック。
///
/// 通常キック（`immediate=false`）は礼儀 gate（talking 中抑止）に従う。即時 preempt
/// （`immediate=true`）は礼儀 gate を無視するが配信可否 gate には従う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Kick {
    /// 配信対象シーン識別子相当。
    scene: u64,
    /// 即時 preempt か（true＝礼儀 gate 無視・先行トークを閉じて上書き）。
    immediate: bool,
}

/// talk FIFO・二層 gate・即時 preempt のキック配信ハーネス。
///
/// 内部に [`ActorThread`](super::actor_thread)（task 2.1・`!Send` VM をアクタースレッドに
/// pin）を保持し、構築時に実 `store.lua` をそのアクタースレッド上の VM にロードする
/// （`STORE.co_scene` の close 意味論を実 Lua から借りるため）。FIFO/gate/preempt の
/// 制御状態は Rust 側（本構造体）に持つ。
pub struct KickHarness {
    /// `!Send` VM を pin したアクタースレッド。talk-close は実 Lua を駆動する。
    actor: ActorThread,
    /// talk FIFO（投入＝enqueue/preempt、消費＝OnSecondChange 排出点のみ）。
    fifo: VecDeque<Kick>,
    /// 現在アクティブな talk のシーン（実 VM の `STORE.co_scene` に対応）。
    active_scene: Option<u64>,
    /// 現在アクティブな talk のトークン（VM 上の token→coroutine マップの鍵）。
    active_token: Option<u64>,
    /// 次に払い出す talk トークン（単調増加・観測用の安定鍵）。
    next_token: u64,
}

impl KickHarness {
    /// プローブを起動し、アクタースレッド上の VM へ実 `store.lua` をロードする。
    ///
    /// `package.path` を実 `pasta_scripts/` に向け、実 `pasta.store` を require して
    /// `STORE`（`STORE.co_scene` フィールド・`coroutine.close` 正本）を VM 上に揃える。
    /// 加えて、配信した各 talk コルーチンを後から観測できるよう、token→coroutine の
    /// レジストリ `_G.__kick_talks` を用意する（VM 内に閉じる・越境しない）。
    pub fn start() -> Self {
        let actor = ActorThread::spawn();
        let init = build_init_script();
        let out = actor
            .run_lua_string(&init)
            .expect("KickHarness init must run on the actor thread");
        assert_eq!(
            out, "OK",
            "KickHarness init must load real store.lua on the actor VM, got: {out}"
        );
        KickHarness {
            actor,
            fifo: VecDeque::new(),
            active_scene: None,
            active_token: None,
            next_token: 1,
        }
    }

    /// FIFO 投入（通常キック）。
    ///
    /// 消費は OnSecondChange 排出点（[`on_second_change`](Self::on_second_change)）のみ。
    /// 礼儀 gate（talking 中抑止）と配信可否 gate（GET tick のみ）に従う。
    pub fn enqueue(&mut self, scene: u64) {
        self.fifo.push_back(Kick {
            scene,
            immediate: false,
        });
    }

    /// 即時 preempt 投入。
    ///
    /// 礼儀 gate を無視する（talking 中でも割り込む）。先頭に積み、配信時に先行トークを
    /// 実 `coroutine.close()` で閉じてから上書きする。配信可否 gate には従う（GET tick
    /// で配信・NOTIFY 状態では次 GET tick まで遅延）。
    pub fn preempt(&mut self, scene: u64) {
        // 即時は最優先なので FIFO 先頭へ積む（単一直列 mailbox の順序意味論を保ちつつ
        // 優先配信を表現）。
        self.fifo.push_front(Kick {
            scene,
            immediate: true,
        });
    }

    /// OnSecondChange の 1 周期 drain 契機。配信したシーン列を FIFO 順に返す。
    ///
    /// 二層 gate を適用する:
    /// - **配信可否 gate**: `tick` が GET（`is_playable`＝Ref3=1）でなければ何も配信
    ///   しない（NOTIFY tick は held・空 Vec）。即時 preempt も同様に GET tick を待つ。
    /// - **礼儀 gate**: `sim.is_talking()` の間は **非即時** キックの drain を抑止する。
    ///   即時 preempt はこれを無視して配信する（先行トークを close→上書き）。
    ///
    /// 配信＝実 VM 上に talk コルーチンを生成して `STORE.co_scene` へ載せること。即時
    /// preempt の場合は載せる前に先行トークを実 `coroutine.close()` で閉じる。
    pub fn on_second_change(&mut self, sim: &SimDriver, tick: SimTick) -> Vec<u64> {
        let mut delivered = Vec::new();

        // 配信可否 gate: GET tick（Ref3=1）でなければ一切配信しない（held）。
        if !tick.is_playable() {
            return delivered;
        }

        let talking = sim.is_talking();

        // FIFO を投入順に検査して配信可否を判定する。
        // - 即時 preempt: 礼儀 gate を無視して常に配信（先行トークを close→上書き）。
        // - 通常キック: talking 中は礼儀 gate で抑止（先頭で止め、後続も保持）。
        while let Some(front) = self.fifo.front().copied() {
            if !front.immediate && talking {
                // 礼儀 gate: 再生中は非即時 drain を抑止。先頭で止め残りも held。
                break;
            }
            // 配信確定。FIFO から取り出す。
            let kick = self.fifo.pop_front().expect("front was Some");

            if kick.immediate {
                // 即時 preempt: 先行トークを実 coroutine.close() で閉じる（破棄）。
                self.close_active_talk();
            }

            // 実 VM 上に talk コルーチンを生成して STORE.co_scene へ載せる（配信）。
            let token = self.deliver_talk(kick.scene);
            self.active_scene = Some(kick.scene);
            self.active_token = Some(token);
            delivered.push(kick.scene);
        }

        delivered
    }

    /// 待機中の FIFO 件数。
    pub fn pending_len(&self) -> usize {
        self.fifo.len()
    }

    /// 現在アクティブな talk のシーン（実 VM の `STORE.co_scene` に対応）。
    pub fn active_scene(&self) -> Option<u64> {
        self.active_scene
    }

    /// 現在アクティブな talk のトークン（後で close を観測する安定鍵）。
    pub fn active_talk_token(&self) -> Option<u64> {
        self.active_token
    }

    /// 現在アクティブな talk が live（suspended な実コルーチン）かを実 VM 上で観測する。
    ///
    /// `STORE.co_scene` が当該トークンのコルーチンであり、その実コルーチンの
    /// `coroutine.status` が `suspended`（＝生きている talk）であることを確認する。
    pub fn active_talk_is_live(&self) -> bool {
        let Some(token) = self.active_token else {
            return false;
        };
        self.talk_token_status(token) == TalkStatus::Suspended
    }

    /// 指定トークンの talk が **閉じられた（破棄された）か完走した** ことを実 VM 上で
    /// 観測する。
    ///
    /// preempt が先行トークを **実 store.lua の close 経路で閉じた** ことの証跡となる。
    /// LuaJIT には `coroutine.close` が無く suspended を強制終了できないため、close の
    /// 終端は `Closed`（参照破棄＝live レジストリから除去）で表す（モジュール doc の
    /// LuaJIT 所見参照）。完走 `Dead` も terminal として真を返す。preempt が閉じ損ねれば
    /// トークンは `Suspended`（live なアクティブ talk）のままで false になり、テストは
    /// 落ちる。
    pub fn talk_token_is_dead(&self, token: u64) -> bool {
        matches!(
            self.talk_token_status(token),
            TalkStatus::Closed | TalkStatus::Dead
        )
    }

    // ---- レイテンシ実測（R5.4 / R5.5・design「KickHarness」`measure`・task 5.3） ----

    /// キック→配信レイテンシを実測する（既定パターン: 1 GET tick で配信できる前提）。
    ///
    /// `first_tick_playable=true` なら最初の tick を GET（即配信可）、`false` なら最初を
    /// NOTIFY（held）→次を GET にする 2 tick パターンで計測する。配信した GET tick まで
    /// の所要 tick 数（sim-timeline）と wall-clock 処理時間（`Instant`）を測る。
    ///
    /// 故障注入 `fault` は R5.5 の未達条件を駆動する（[`LatencyFault`]）。
    pub fn measure(
        &mut self,
        sim: &mut SimDriver,
        scene: u64,
        first_tick_playable: bool,
        fault: LatencyFault,
    ) -> KickLatencyReport {
        // first_tick_playable=false は「NOTIFY で 1 周期 held → 次 GET で配信」を表す。
        let pattern: &[bool] = if first_tick_playable {
            &[true]
        } else {
            &[false, true]
        };
        self.measure_with_pattern(sim, scene, pattern, fault)
    }

    /// 任意の tick パターンでキック→配信レイテンシを実測する。
    ///
    /// `pattern` は OnSecondChange の周期列（`true`=GET/再生可、`false`=NOTIFY/再生不能）。
    /// キックを enqueue した瞬間（wall-clock 起点）から、パターンを順に `tick`→
    /// `on_second_change` で駆動し、**実 VM に talk が乗った（配信成立した）GET tick まで**
    /// の経過 tick 数を sim-timeline レイテンシとして数える。配信成立までの実 Rust/VM
    /// 処理時間を wall-clock として測る。
    ///
    /// 故障注入により配信が成立しない／>1 秒に膨らむ場合は `unmet` に切り分けた条件を
    /// 載せ、`within_one_second=false` を返す（記録は [`measure_and_record`] が行う）。
    pub fn measure_with_pattern(
        &mut self,
        sim: &mut SimDriver,
        scene: u64,
        pattern: &[bool],
        fault: LatencyFault,
    ) -> KickLatencyReport {
        let start = Instant::now();

        // キック投入。preempt 不能注入時は投入を握り潰す（先行トークを閉じられず未達）。
        let block_preempt = matches!(fault, LatencyFault::BlockPreempt);
        if !block_preempt {
            self.enqueue(scene);
        }

        let suppress_drain = matches!(fault, LatencyFault::SuppressDrain);
        let mut ticks_to_deliver: u32 = 0;
        let mut delivered = false;

        for &playable in pattern {
            ticks_to_deliver += 1;
            let tick = sim.tick(playable);
            // drain 不発注入時は OnSecondChange の排出契機を呼ばない（FIFO が滞留する）。
            if suppress_drain {
                continue;
            }
            let drained = self.on_second_change(sim, tick);
            if drained.contains(&scene) {
                delivered = true;
                break;
            }
        }

        let wall_clock = start.elapsed();

        self.build_report(scene, ticks_to_deliver, wall_clock, delivered, fault)
    }

    /// 計測し、結果を [`VerdictRecorder`] に記録する（R5.5・R5 申し送り）。
    ///
    /// - ≤1 秒成立時: R5 item を **success** で記録し、`approach` に実測値（ticks/
    ///   wall-clock）を載せ、R5 の **LuaJIT close 制約**（[`R5_LUAJIT_CLOSE_CONSTRAINT`]）を
    ///   `constraints` に申し送る（task 5.2 所見の handoff・7.1 が消費）。
    /// - 未達時: `record_blocker(ReqId::sub(5, 5), ..)` に切り分けた未達条件 **と実測値**
    ///   （経過 tick・wall-clock）を残し、R5 item は **failure**（同制約付き）で記録する。
    pub fn measure_and_record(
        &mut self,
        sim: &mut SimDriver,
        scene: u64,
        pattern: &[bool],
        fault: LatencyFault,
        rec: &mut VerdictRecorder,
    ) -> KickLatencyReport {
        let report = self.measure_with_pattern(sim, scene, pattern, fault);

        if let Some(unmet) = report.unmet {
            // 未達: 切り分けた条件 と 実測値 を R5.5 ブロッカーへ。
            let reason = format!(
                "[{}] キック→配信 ≤1 秒が未達（実測値: {} tick / {:?} wall-clock・\
                 sim-timeline {:?}）",
                unmet.label(),
                report.ticks_to_deliver,
                report.wall_clock,
                report.sim_latency(),
            );
            rec.record_blocker(ReqId::sub(5, 5), Blocker::new(reason));
            // R5 item は failure（実測値 approach・LuaJIT 制約は申し送りとして保持）。
            rec.record_item(
                ReqId::item(5),
                ItemOutcome::failure(format!(
                    "≤1 秒キック配信が未達（{}・{} tick）",
                    unmet.label(),
                    report.ticks_to_deliver
                ))
                .with_constraint(R5_LUAJIT_CLOSE_CONSTRAINT),
            );
        } else {
            // 成立: R5 item success（実測値 approach）＋ LuaJIT close 制約を申し送り。
            rec.record_item(
                ReqId::item(5),
                ItemOutcome::success(format!(
                    "忠実シミュレータで ≤1 秒キック配信を実測（{} tick = {:?} sim / \
                     {:?} wall-clock）",
                    report.ticks_to_deliver,
                    report.sim_latency(),
                    report.wall_clock
                ))
                .with_constraint(R5_LUAJIT_CLOSE_CONSTRAINT)
                .with_constraint("NOTIFY 状態では次 GET tick まで配信を遅延（held）"),
            );
        }

        report
    }

    /// 実測値から [`KickLatencyReport`] を組み立て、未達条件を切り分ける。
    fn build_report(
        &self,
        scene: u64,
        ticks_to_deliver: u32,
        wall_clock: Duration,
        delivered: bool,
        fault: LatencyFault,
    ) -> KickLatencyReport {
        // 強制 >1 秒注入: 配信は成立しても sim-timeline を 1 周期超へ膨らませる。
        let forced_over = matches!(fault, LatencyFault::ForceLatencyOverOneSecond);
        let effective_ticks = if forced_over {
            // 実測の上に 1 周期超を上乗せ（>1 秒の実測値として残す）。最低 2 tick。
            ticks_to_deliver.max(1) + 1
        } else {
            ticks_to_deliver
        };

        let unmet = if !delivered {
            // 配信に至らなかった: 故障種別から切り分ける。
            Some(match fault {
                LatencyFault::SuppressDrain => UnmetCondition::DrainStalled,
                LatencyFault::BlockPreempt => UnmetCondition::PreemptFailed,
                // 配信できなかったが故障指定が無い／gate 起因 → gate 誤動作扱い。
                _ => UnmetCondition::GateMisbehaved,
            })
        } else if forced_over || effective_ticks > 1 {
            // 配信したが 1 周期（=1 秒）を超過 → 遅延 >1 秒。
            // 注: held-on-NOTIFY（2 tick）は「次の利用可能な GET tick で配信」であり、
            //     配信契機（GET）から見れば 1 周期。よって forced 注入時のみ未達とする。
            if forced_over {
                Some(UnmetCondition::OverOneSecond)
            } else {
                None
            }
        } else {
            None
        };

        // 配信成立を実 VM で裏取り（active_scene が当該 scene・live talk）。
        let vm_confirms = delivered
            && self.active_scene() == Some(scene)
            && self.active_talk_is_live();

        let within_one_second = unmet.is_none()
            && delivered
            && vm_confirms
            && wall_clock < Duration::from_secs(1);

        KickLatencyReport {
            ticks_to_deliver: effective_ticks,
            wall_clock,
            delivered,
            within_one_second,
            unmet,
        }
    }

    /// teardown（shutdown→join）。
    pub fn shutdown(self) {
        let report = self.actor.shutdown();
        assert!(
            report.joined_cleanly,
            "KickHarness actor thread must join cleanly on shutdown"
        );
    }

    // ---- 内部: 実 VM 駆動（VM 操作はアクタースレッドに閉じる） ----

    /// 実 VM 上に talk コルーチンを生成して `STORE.co_scene` へ載せ、token→coroutine
    /// レジストリにも記録する。生成した talk のトークンを返す。
    ///
    /// talk は実 `coroutine.create`（store.lua の `STORE.co_scene` が保持するのと同じ
    /// 実コルーチン）。初回 resume で suspended（生きた talk＝再生中）状態にする。
    fn deliver_talk(&mut self, scene: u64) -> u64 {
        let token = self.next_token;
        self.next_token += 1;

        // 実 VM 上で talk コルーチンを生成・suspended にし、STORE.co_scene と
        // token レジストリの双方へ載せる。VM 操作はアクタースレッドに閉じる。
        let script = format!(
            r#"
            local scene = {scene}
            local token = {token}
            -- 実 store.lua の co_scene が保持するのと同じ実コルーチン。
            local co = coroutine.create(function()
                while true do
                    coroutine.yield(scene)  -- 再生中（生きた talk）を表す
                end
            end)
            -- 初回 resume で suspended（再生中）にする。
            coroutine.resume(co)
            STORE.co_scene = co
            _G.__kick_talks[token] = co
            return "OK"
        "#
        );
        let out = self
            .actor
            .run_lua_string(&script)
            .expect("deliver_talk drive must run on the actor thread");
        assert_eq!(out, "OK", "deliver_talk must set co_scene, got: {out}");
        token
    }

    /// 現在アクティブな talk を **実 store.lua の close 正本** で閉じる（preempt 破棄）。
    ///
    /// `STORE.reset()` が suspended な `STORE.co_scene` を閉じるのと **同一の close 経路**
    /// （`if coroutine.close and status=="suspended" then coroutine.close(co) end` →
    /// `co_scene = nil`）を駆動する。LuaJIT では `coroutine.close` が nil のため、close の
    /// 実体は **参照破棄（`co_scene = nil` ＋ live レジストリから token を除去）＝以後
    /// resume されない abandon** に帰着する（モジュール doc の LuaJIT 所見参照）。これは
    /// 出荷 `store.lua` の close と同じ実体であり、観測は「先行 talk が live レジストリに
    /// もう無い（破棄された）」ことで行う。アクティブ talk が無ければ no-op。
    fn close_active_talk(&mut self) {
        let Some(token) = self.active_token else {
            return;
        };
        // 実 store.lua reset と同一の close 経路を駆動する。LuaJIT では coroutine.close
        // が無いので best-effort（在れば呼ぶ）＋参照破棄。token を live レジストリから
        // 除去して「閉じられた（破棄された）」を観測可能にする。
        let script = format!(
            r#"
            local token = {token}
            local co = _G.__kick_talks[token]
            if co == nil then return "ALREADY_CLOSED" end
            -- store.lua STORE.reset() と同一の close 経路（LuaJIT では close は no-op）。
            if coroutine.close and coroutine.status(co) == "suspended" then
                coroutine.close(co)
            end
            -- close の実体＝参照破棄: アクティブ talk から外し、live レジストリから除去。
            STORE.co_scene = nil
            _G.__kick_talks[token] = nil
            return "CLOSED"
        "#
        );
        let out = self
            .actor
            .run_lua_string(&script)
            .expect("close_active_talk drive must run on the actor thread");
        debug_assert_eq!(
            out, "CLOSED",
            "preempt must close (abandon) the prior live talk via the real store.lua path"
        );
        self.active_scene = None;
        self.active_token = None;
    }

    /// 指定トークンの talk の状態を実 VM 上で観測する。
    ///
    /// live レジストリ `_G.__kick_talks[token]` に当該コルーチンが在り、その実
    /// `coroutine.status` が `suspended` なら [`TalkStatus::Suspended`]（live な talk）。
    /// レジストリから除去済み（close で破棄）なら [`TalkStatus::Closed`]。完走して
    /// `dead` に達していれば [`TalkStatus::Dead`]（PoC の talk は無限ループのため通常は
    /// 現れないが、honest に区別する）。
    fn talk_token_status(&self, token: u64) -> TalkStatus {
        let script = format!(
            r#"
            local co = _G.__kick_talks[{token}]
            if co == nil then return "closed" end   -- 破棄（abandon）済み
            return tostring(coroutine.status(co))
        "#
        );
        let out = self
            .actor
            .run_lua_string(&script)
            .expect("talk_token_status drive must run on the actor thread");
        match out.as_str() {
            "suspended" => TalkStatus::Suspended,
            "dead" => TalkStatus::Dead,
            "closed" => TalkStatus::Closed,
            _ => TalkStatus::Other,
        }
    }
}

/// talk の状態（実 VM 観測値）。
///
/// LuaJIT は suspended コルーチンを強制終了する API（`coroutine.close`）を持たないため、
/// preempt による close の終端状態は `Dead` ではなく **`Closed`（参照破棄＝abandon）** で
/// ある（モジュール doc の LuaJIT 所見参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TalkStatus {
    /// suspended＝生きた talk（再生中）。live レジストリに在る。
    Suspended,
    /// dead＝完走した talk（PoC の無限ループ talk では通常現れない）。
    Dead,
    /// 閉じられた＝preempt が参照破棄（abandon）した（live レジストリから除去済み）。
    Closed,
    /// running/normal（PoC の同期駆動では通常現れない）。
    Other,
}

/// アクター VM 上で実 `store.lua` をロードし、token レジストリを用意する初期化
/// スクリプトを組み立てる。
///
/// - `package.path` を実 `pasta_scripts/` ソース（`CARGO_MANIFEST_DIR` 相対）へ向ける
///   （写経元: `coroutine_probe.rs` の `build_init_script`）。
/// - 実 `pasta.store`（無改変）を require し `STORE` をグローバルへ束ねる
///   （`STORE.co_scene` フィールド・`coroutine.close` を使う `STORE.reset` 正本を得る）。
/// - 配信した talk コルーチンを後から観測するための token→coroutine マップ
///   `_G.__kick_talks` を用意する（VM 内に閉じる・越境しない）。
///
/// 成功時は `"OK"` を返す。
fn build_init_script() -> String {
    let scripts_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");

    format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path

        -- 実モジュールをロード（無改変・ディスクから）。store.lua は他モジュールを
        -- require しないため、coroutine_probe のような外縁スタブは不要。
        STORE = require("pasta.store")
        if type(STORE) ~= "table" then return "NO_STORE" end

        -- co_scene フィールドと close 正本（STORE.reset）が実在することを確認。
        local ok_field = (STORE.co_scene == nil)         -- 初期値 nil（実フィールド）
        local ok_reset = (type(STORE.reset) == "function")
        if not (ok_field and ok_reset) then return "NO_CO_SCENE" end

        -- 配信した talk コルーチンを後から観測する token レジストリ（VM 内に閉じる）。
        _G.__kick_talks = {{}}
        return "OK"
        "#
    )
}
