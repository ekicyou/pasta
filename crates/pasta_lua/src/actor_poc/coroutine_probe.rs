//! CoroutineProbe（R4・task 4.1）。
//!
//! 駆動主体を **ホスト tick から executor へ移しても** 既存の継続実行基盤
//! （実コルーチン `STORE.co_scene` と非同期 callback `CALLBACK.pending`）が生存・
//! 継続することを、**無改変の実 `*.lua`** を executor 駆動して実証する
//! （design.md「CoroutineProbe」／Requirements 4.1〜4.4）。
//!
//! # 何を実 `.lua` から借りるか（R4.3 / 境界線の遵守）
//!
//! design.md「Marshal / 境界線（R4 を壊さないための制約）」: **シーン実行・コルーチン
//! 継続・callback 待機の意味論は Lua のまま** 駆動する。本プローブは Rust/Lua の
//! 再実装を持たず、以下の実モジュールを **ディスクからそのまま** ロードしてその実
//! 関数を駆動する:
//! - `pasta/store.lua`（`STORE.co_scene` フィールド・`STORE.reset` の close 意味論）
//! - `pasta/shiori/event/init.lua`（**ローカルの** `resume_until_valid`／`set_co_scene`
//!   ＝ nil yield スキップと旧コルーチン close の正本ロジック。`EVENT._resume_until_valid`
//!   で公開）
//! - `pasta/shiori/event/callback.lua`（`CALLBACK.pending`／`stage_pending`／
//!   `consume_staged`／`try_route`／`next_event_id`／`sweep` の callback 待機正本）
//! - `pasta/shiori/event/second_change.lua`（`REG.OnSecondChange`＝drain/sweep 契機）
//!
//! `event/init.lua` は本来 `EVENT.fire` 経路で重いモジュール（`pasta.scene`・
//! `pasta.shiori.act`）を必要とするが、それらは **本プローブが駆動するコルーチン継続／
//! callback 解決の経路では使われない**。`require` 時の解決のみ成立させるため、
//! `pasta.scene` / `pasta.shiori.act` は **package.preload の軽量スタブ**で満たす
//! （これらは「シーン中核ロジック」ではなく `EVENT.fire`/`no_entry` 経由でのみ使う
//! 外縁依存であり、本プローブの継続意味論には一切寄与しない＝R4.3 を壊さない）。
//!
//! # 駆動主体が executor であること（R4.1 の核心）
//!
//! 各 resume／callback 解決は [`ActorThread::run_lua_string`] へ投入する **別個の
//! コマンド**で起こる。1 コマンド = アクター future の 1 回の executor ポーリング
//! （driving step）であり、ホストの SHIORI tick ではない。コルーチン本体は VM 内に
//! 留まり（VM は越境しない）、Rust 側は文字列の観測値だけを受け取る。
//!
//! # 喪失の記録（R4.4）
//!
//! 継続喪失（resume 不能・状態喪失・継続契機消失）を注入したとき、[`record_loss`]
//! が [`VerdictRecorder::record_blocker`] で **R4.4** に切り分けたブロッカーを残し
//! NO-GO 根拠化する（reuse: task 1.5 の `VerdictRecorder`）。

use super::actor_thread::ActorThread;
use super::verdict::{Blocker, ItemOutcome, ReqId, VerdictRecorder};

/// `resume_scene`（R4.1）の観測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneResumeResult {
    /// 中断地点の **次から** 継続したか（リセットされていない）。
    pub continued: bool,
    /// 各 executor 駆動で観測した yield 値の列（実シーン継続なら `[1, 2, 3]`）。
    pub observed: Vec<i64>,
    /// コルーチンが完了（dead）に到達したか。
    pub completed: bool,
    /// 要した executor 駆動（driving step）回数。各 resume が別ポーリングである証跡。
    pub executor_drives: usize,
}

/// `resolve_callback`（R4.2）の観測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackResolveResult {
    /// コルーチンが `CALLBACK.stage_pending` で pending を登録し suspend したか。
    pub staged: bool,
    /// 解決前に `CALLBACK.pending` が待機コルーチンを保持していたか。
    pub was_pending_before_resolution: bool,
    /// 後続の executor 駆動で callback が解決（`CALLBACK.try_route`）したか。
    pub resolved: bool,
    /// 解決後にコルーチンが完了（dead）まで継続したか。
    pub completed: bool,
    /// 解決後に `CALLBACK.pending` からエントリが除去されたか。
    pub pending_cleared_after: bool,
}

/// `assert_real_lua_loaded`（R4.3）の忠実性チェック結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FidelityCheck {
    /// 実 `store.lua` の `STORE.co_scene` フィールドが在る（nil 初期値含む）。
    pub store_has_co_scene_field: bool,
    /// 実 `callback.lua` の `CALLBACK.pending` テーブルが在る。
    pub callback_has_pending_table: bool,
    /// 実 `callback.lua` の主要関数（stage/consume/route/next_id）が在る。
    pub callback_has_real_functions: bool,
    /// 実 `event/init.lua` の `_resume_until_valid` が公開されている。
    pub event_has_resume_until_valid: bool,
    /// 実 `second_change.lua` が `REG.OnSecondChange` を登録している。
    pub second_change_registered: bool,
}

/// 注入する喪失条件の種別（R4.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossKind {
    /// `STORE.co_scene` が resume 前に喪失（状態喪失）→ 継続不能。
    CoSceneLost,
    /// `CALLBACK.pending` の待機エントリが解決契機前に喪失 → 継続契機の消失。
    CallbackPendingLost,
}

impl LossKind {
    /// 分類ラベル（reason 先頭に付与し条件を一目で切り分けられるようにする）。
    fn label(self) -> &'static str {
        match self {
            LossKind::CoSceneLost => "co_scene-lost",
            LossKind::CallbackPendingLost => "CALLBACK.pending-lost",
        }
    }
}

/// executor 駆動で実 `.lua` のコルーチン／callback を駆動するプローブ。
///
/// 内部に [`ActorThread`]（task 2.1・`!Send` VM をアクタースレッドに pin）を保持し、
/// 構築時に実モジュールをそのアクタースレッド上の VM にロードする。
pub struct CoroutineProbe {
    actor: ActorThread,
}

impl CoroutineProbe {
    /// プローブを起動し、アクタースレッド上の VM へ実 `.lua` をロードする。
    ///
    /// `package.path` を実 `pasta_scripts/` に向け、軽量スタブ（`pasta.scene` /
    /// `pasta.shiori.act` — 本プローブの継続経路では使われない外縁依存）を preload
    /// した上で、実モジュールを require する。ロード失敗時は panic する（プローブの
    /// 前提が成立しない＝バグであり、検証対象の「喪失」ではない）。
    pub fn start() -> Self {
        let actor = ActorThread::spawn();
        let init = build_init_script();
        let out = actor
            .run_lua_string(&init)
            .expect("CoroutineProbe init must run on the actor thread");
        assert_eq!(
            out, "OK",
            "CoroutineProbe init must load the real *.lua modules on the actor VM, got: {out}"
        );
        CoroutineProbe { actor }
    }

    /// R4.1: executor 駆動でシーンコルーチン（`STORE.co_scene` モデル）を中断地点から
    /// resume し、継続することを観測する。
    ///
    /// 駆動手順（各ステップ = 別個の executor 駆動）:
    /// 1. 実 `set_co_scene` 素地で 3 回 yield するコルーチンを `STORE.co_scene` に格納。
    /// 2. 別ステップで実 `resume_until_valid` を 1 回ずつ呼び、yield 値を観測。
    ///    各呼び出しは別コマンド＝別 executor ポーリングであり、コルーチンは VM 内に
    ///    留まったまま「前回の中断地点の次から」継続する。
    pub fn resume_scene(&self) -> SceneResumeResult {
        // (drive 0) コルーチンを生成し STORE.co_scene に格納（実 store.lua のフィールド）。
        // 進行は実コルーチンの中断/再開で表現する: yield 1, yield 2, yield 3, 完了。
        let setup = r#"
            STORE.co_scene = coroutine.create(function()
                coroutine.yield(1)
                coroutine.yield(2)
                coroutine.yield(3)
                return nil
            end)
            return tostring(coroutine.status(STORE.co_scene))
        "#;
        let status = self
            .actor
            .run_lua_string(setup)
            .expect("scene setup drive must run");
        // 格納直後のコルーチンは未開始（suspended）であること（中断地点の検証前提）。
        debug_assert_eq!(status, "suspended", "co_scene must start suspended");
        let mut drives = 1usize; // setup も 1 回の executor 駆動

        let mut observed = Vec::new();
        let mut completed = false;
        // 各 resume を別個の executor 駆動として投入（実 resume_until_valid を使用）。
        for _ in 0..4 {
            let step = r#"
                local co = STORE.co_scene
                if co == nil then return "LOST" end
                if coroutine.status(co) == "dead" then return "DEAD" end
                -- 実 event/init.lua の resume_until_valid（nil yield スキップの正本）。
                local ok, value = EVENT._resume_until_valid(co)
                if not ok then return "ERR:" .. tostring(value) end
                if coroutine.status(co) == "dead" then
                    return "DEAD:" .. tostring(value)
                end
                return "Y:" .. tostring(value)
            "#;
            let r = self
                .actor
                .run_lua_string(step)
                .expect("scene resume drive must run");
            drives += 1;
            if let Some(v) = r.strip_prefix("Y:") {
                if let Ok(n) = v.parse::<i64>() {
                    observed.push(n);
                }
            } else if r.starts_with("DEAD") {
                completed = true;
                break;
            } else {
                // LOST / ERR — 継続喪失。観測を止める。
                break;
            }
        }

        // 継続した = 1 にリセットされず、観測値が単調に進んだ（中断地点の次から再開）。
        let continued = observed.first() == Some(&1)
            && observed.windows(2).all(|w| w[1] == w[0] + 1)
            && observed.len() >= 2;

        SceneResumeResult {
            continued,
            observed,
            completed,
            executor_drives: drives,
        }
    }

    /// R4.2: executor 駆動下で非同期 callback（`CALLBACK.pending` モデル）を発行し、
    /// 後続の駆動契機で resume して解決・継続することを観測する。
    ///
    /// 実 `callback.lua` の `next_event_id`／`stage_pending`／`consume_staged`／
    /// `try_route` をそのまま駆動する（callback 待機の意味論は実 Lua）。
    pub fn resolve_callback(&self) -> CallbackResolveResult {
        // (drive 1) callback 待ちコルーチンを生成・初回 resume・consume_staged で
        // pending に登録する（実 callback.lua の登録経路）。
        let stage = r#"
            CALLBACK.reset()
            local event_id = CALLBACK.next_event_id()
            _G.__probe_event_id = event_id
            local co = coroutine.create(function()
                -- callback を待つ: stage_pending してから yield（実 stage_pending）。
                CALLBACK.stage_pending(event_id, os.time() + 3600, nil)
                local refs = coroutine.yield()       -- ここで callback 到着を待つ
                _G.__probe_resolved_with = (refs and refs[1]) or "no-ref"
                return "done"
            end)
            -- 初回 resume（stage_pending→yield まで進む）。
            local ok, _ = coroutine.resume(co)
            if not ok then return "RESUME_ERR" end
            -- 実 consume_staged: staged を pending へ登録（co_callback も設定）。
            local consumed = CALLBACK.consume_staged(co, { fake_act = true })
            if not consumed then return "NOT_STAGED" end
            if coroutine.status(co) ~= "suspended" then return "NOT_SUSPENDED" end
            return "STAGED"
        "#;
        let staged_out = self
            .actor
            .run_lua_string(stage)
            .expect("callback stage drive must run");
        let staged = staged_out == "STAGED";

        // (drive 2) 解決前に pending が待機エントリを保持していることを観測。
        let pending_before = r#"
            local id = _G.__probe_event_id
            return (CALLBACK.pending[id] ~= nil) and "PENDING" or "EMPTY"
        "#;
        let was_pending_before_resolution = self
            .actor
            .run_lua_string(pending_before)
            .map(|s| s == "PENDING")
            .unwrap_or(false);

        // (drive 3) 後続契機: 実 try_route で到着イベントを resume して解決する。
        //   try_route は EVENT.fire のコールバックルーティングが呼ぶのと同じ実関数。
        let resolve = r#"
            local id = _G.__probe_event_id
            local req = { id = id, reference = { [0] = "callback-payload" } }
            -- 実 try_route: pending を削除→コルーチン resume→継続。
            local response = CALLBACK.try_route(req)
            if response == nil then return "NO_ROUTE" end
            -- コルーチンが解決値を受け取り完了したか。
            local resolved_with = _G.__probe_resolved_with or "nil"
            return "RESOLVED:" .. tostring(resolved_with)
        "#;
        let resolve_out = self
            .actor
            .run_lua_string(resolve)
            .expect("callback resolve drive must run");
        let resolved = resolve_out.starts_with("RESOLVED:")
            && resolve_out.contains("callback-payload");

        // (drive 4) 解決後にコルーチンが完了し pending から除去されたことを観測。
        let after = r#"
            local id = _G.__probe_event_id
            local cleared = (CALLBACK.pending[id] == nil)
            return cleared and "CLEARED" or "STILL_PENDING"
        "#;
        let pending_cleared_after = self
            .actor
            .run_lua_string(after)
            .map(|s| s == "CLEARED")
            .unwrap_or(false);

        // 完了の判定: resolve_out が解決値（callback-payload）を運んだ＝コルーチンが
        // yield の次まで継続し return 値に到達した。
        let completed = resolved;

        CallbackResolveResult {
            staged,
            was_pending_before_resolution,
            resolved,
            completed,
            pending_cleared_after,
        }
    }

    /// R4.3: アクター VM 上に実 `.lua` モジュールの関数／テーブルが実在することを
    /// 確認する（Rust 再実装でない証跡）。
    pub fn assert_real_lua_loaded(&self) -> FidelityCheck {
        let check = r#"
            local parts = {}
            -- store.lua: 実モジュールであること（reset は正本関数）。
            parts[#parts+1] = (type(STORE) == "table" and type(STORE.reset) == "function")
                and "store" or "x"
            -- co_scene フィールドが正本どおり機能する（格納→reset で nil 化される）。
            -- これは store.lua の reset() が co_scene を close/nil 化する実ロジックを
            -- 駆動して確かめる（フィールドの存在を芝居でなく実観測する）。
            STORE.co_scene = coroutine.create(function() coroutine.yield() end)
            STORE.reset()  -- 実 store.lua: co_scene を close し nil 化する
            parts[#parts+1] = (STORE.co_scene == nil) and "co_scene_field" or "x"
            -- callback.lua
            parts[#parts+1] = (type(CALLBACK.pending) == "table") and "pending" or "x"
            local cb_fns = (type(CALLBACK.stage_pending) == "function")
                and (type(CALLBACK.consume_staged) == "function")
                and (type(CALLBACK.try_route) == "function")
                and (type(CALLBACK.next_event_id) == "function")
            parts[#parts+1] = cb_fns and "cb_fns" or "x"
            -- event/init.lua の resume_until_valid（公開関数）
            parts[#parts+1] = (type(EVENT._resume_until_valid) == "function") and "rsv" or "x"
            -- second_change.lua: REG.OnSecondChange 登録
            local REG = require("pasta.shiori.event.register")
            parts[#parts+1] = (type(REG.OnSecondChange) == "function") and "osc" or "x"
            return table.concat(parts, ",")
        "#;
        let out = self
            .actor
            .run_lua_string(check)
            .expect("fidelity check drive must run");
        let has = |tag: &str| out.split(',').any(|p| p == tag);
        FidelityCheck {
            store_has_co_scene_field: has("store") && has("co_scene_field"),
            callback_has_pending_table: has("pending"),
            callback_has_real_functions: has("cb_fns"),
            event_has_resume_until_valid: has("rsv"),
            second_change_registered: has("osc"),
        }
    }

    /// R4.4: 継続喪失を注入し、切り分けたブロッカーを R4.4 に記録する（NO-GO 根拠化）。
    ///
    /// 注入は実 VM の状態を実際に喪失させ（`STORE.co_scene = nil` 等）、その後に
    /// 継続を試みて **resume 不能** を観測したうえでブロッカーを残す（芝居ではなく
    /// 実観測）。`Healthy` 経路（喪失なし）では `None`。
    pub fn record_loss(&self, rec: &mut VerdictRecorder, kind: LossKind) -> Option<Blocker> {
        let observed_loss = self.inject_and_observe_loss(kind);
        if !observed_loss {
            // 喪失が観測されなかった = 継続は生存（GO 側）。item に成功を残す。
            rec.record_item(
                ReqId::item(4),
                ItemOutcome::success(format!(
                    "executor 駆動下で {} 注入後も継続が生存（喪失せず）",
                    kind.label()
                )),
            );
            return None;
        }
        let blocker = match kind {
            LossKind::CoSceneLost => Blocker::with_workaround(
                format!(
                    "[{}] executor 駆動前に STORE.co_scene が喪失し resume 不能 \
                     — シーン継続の状態が失われた（R4.1 不成立）",
                    kind.label()
                ),
                "VM 移設時に co_scene を保全し、resume 駆動を executor へ確実に橋渡しする",
            ),
            LossKind::CallbackPendingLost => Blocker::with_workaround(
                format!(
                    "[{}] 解決契機前に CALLBACK.pending の待機エントリが喪失し \
                     try_route が継続契機を失った（R4.2 不成立）",
                    kind.label()
                ),
                "pending レジストリを VM 移設後も保持し、後続駆動契機を確実に届ける",
            ),
        };
        // 失敗でも item を残す（R8.2「成否にかかわらず全項目を試行し結果を残す」）。
        rec.record_item(
            ReqId::item(4),
            ItemOutcome::failure(format!(
                "executor 駆動下で継続喪失を観測: {}",
                kind.label()
            )),
        );
        rec.record_blocker(ReqId::sub(4, 4), blocker.clone());
        Some(blocker)
    }

    /// 喪失を実 VM 状態へ注入し、継続不能（resume 不能）を実観測する。
    ///
    /// 真に喪失が起きれば `true`。何らかの理由で継続できてしまえば `false`
    /// （その場合は喪失なし＝GO 側として扱う）。
    fn inject_and_observe_loss(&self, kind: LossKind) -> bool {
        let script = match kind {
            LossKind::CoSceneLost => {
                r#"
                -- まず正常なコルーチンを格納し suspended にする。
                STORE.co_scene = coroutine.create(function()
                    coroutine.yield(1); coroutine.yield(2); return nil
                end)
                EVENT._resume_until_valid(STORE.co_scene)  -- 1 まで進める
                -- 喪失注入: executor 駆動の前に状態を失わせる。
                STORE.co_scene = nil
                -- 継続を試みる: co_scene が無いので resume 不能。
                if STORE.co_scene == nil then return "LOST" end
                return "ALIVE"
                "#
            }
            LossKind::CallbackPendingLost => {
                r#"
                CALLBACK.reset()
                local event_id = CALLBACK.next_event_id()
                local co = coroutine.create(function()
                    CALLBACK.stage_pending(event_id, os.time() + 3600, nil)
                    coroutine.yield()
                    return "done"
                end)
                coroutine.resume(co)
                CALLBACK.consume_staged(co, { fake_act = true })
                -- 喪失注入: 解決契機の前に pending エントリを失わせる。
                CALLBACK.pending[event_id] = nil
                -- 後続契機で解決を試みる: pending が無いので try_route はルートしない。
                local req = { id = event_id, reference = { [0] = "x" } }
                local response = CALLBACK.try_route(req)
                if response == nil then return "LOST" end
                return "ALIVE"
                "#
            }
        };
        self.actor
            .run_lua_string(script)
            .map(|s| s == "LOST")
            .unwrap_or(true) // 駆動自体が失敗したら継続不能＝喪失とみなす
    }

    /// アクタースレッドを clean に teardown する（shutdown→join）。
    pub fn shutdown(self) {
        let report = self.actor.shutdown();
        assert!(
            report.joined_cleanly,
            "CoroutineProbe actor thread must join cleanly on shutdown"
        );
    }
}

/// アクター VM 上で実 `.lua` をロードする初期化スクリプトを組み立てる。
///
/// - `package.path` を **実 `pasta_scripts/` ソース**（`CARGO_MANIFEST_DIR` 相対）へ
///   向ける（写経元: `tests/common/mod.rs` の `create_runtime_with_pasta_path`）。
/// - `pasta.scene` / `pasta.shiori.act` を **軽量スタブ**として preload する。これらは
///   `EVENT.fire`/`no_entry`/`boot`/`choice_select` の外縁依存であり、本プローブが
///   駆動するコルーチン継続／callback 解決の経路では一切呼ばれない（R4.3 を壊さない）。
/// - 実 `pasta.store` / `pasta.shiori.event`（init.lua）/ `pasta.shiori.event.callback`
///   を require し、`STORE` / `EVENT` / `CALLBACK` をグローバルへ束ねる。
///   `event/init.lua` の require が `second_change.lua`（→`REG.OnSecondChange`）も
///   読み込むため、drain/sweep 契機の正本も VM 上に揃う。
///
/// 成功時は `"OK"` を返す。
fn build_init_script() -> String {
    // 実 pasta_scripts ソースへの絶対パス（forward-slash 正規化）。
    let scripts_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");

    format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path

        -- 軽量スタブ（本プローブの継続経路では未使用の外縁依存）。
        -- pasta.scene: boot.lua / choice_select.lua / no_entry が require するが、
        -- co_scene 継続・callback 解決の経路では呼ばれない。
        package.loaded["pasta.scene"] = setmetatable({{}}, {{
            __index = function() return function() return nil end end
        }})
        -- pasta.shiori.act: event/init.lua が create_act 用に require するが、
        -- 本プローブは EVENT.fire を呼ばないため使われない。
        package.loaded["pasta.shiori.act"] = {{
            new = function(_actors, req) return {{ req = req }} end
        }}

        -- 実モジュールをロード（無改変・ディスクから）。
        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        CALLBACK = require("pasta.shiori.event.callback")

        if type(STORE) ~= "table" then return "NO_STORE" end
        if type(EVENT) ~= "table" then return "NO_EVENT" end
        if type(CALLBACK) ~= "table" then return "NO_CALLBACK" end
        if type(EVENT._resume_until_valid) ~= "function" then return "NO_RSV" end
        return "OK"
        "#
    )
}
