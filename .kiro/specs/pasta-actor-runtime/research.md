# ギャップ分析: pasta-actor-runtime

本ドキュメントは、生成済み要件（`requirements.md`）と既存コードベースの間のギャップを分析し、設計フェーズの実装戦略を導く。先行 PoC 仕様 `pasta-actor-feasibility`（判定 GO+・`.kiro/specs/completed/pasta-actor-feasibility/`）の結論を着手前提とする。

## 1. 現状調査（Current State Investigation）

### 主要資産とディレクトリ
- **SHIORI アダプタ層**: `crates/pasta_shiori/src/`
  - `windows.rs` — FFI 入口（`DllMain`／`load`／`unload`／`request`）。`OnceLock<RawShiori<PastaShiori>>` がプロセスグローバル。`RawShiori` 各 dispatch は `catch_unwind` で panic を SHIORI エラー契約へ変換（リリースは `panic=abort` で catch 到達不能・dev/test/rlib 向け）。
  - `shiori.rs` — `PastaShiori`（`Shiori` trait 実装）。`PastaLuaRuntime` を `Option` で保持。**`unsafe impl Send` / `unsafe impl Sync`**（`shiori.rs:51-52`）と `windows.rs:148` の `Arc<Mutex<Option<PastaShiori>>>` で VM を SHIORI スレッドへ束縛。SHIORI.load/request/unload を `Function` キャッシュ。
  - `lua_request.rs` — pest プロトコルパーサ。`req.method`（get/notify）は VM 投入**前**に確定（`parse1`、`lua_request.rs:86-87`）。marshaling 分岐点として再利用可能。
- **エンジンコア層**: `crates/pasta_lua/src/`
  - `runtime/mod.rs` — `PastaLuaRuntime{ lua: mlua::Lua(!Send), ... }`。公開アクセサ `lua()`／`exec*()`。VM 構築は `runtime/factory.rs`。**型自体に Send/Sync 実装なし**（unsafe ハックは `PastaShiori` 側にのみ存在）。
  - `sakura_script/`（`mod.rs`・`tokenizer.rs`・`wait_inserter.rs`・`line_breaker.rs`）— **さくらスクリプト描画**。公開: `register(lua, config) -> Table`、Lua 側 `SAKURA.talk_to_script(actor, text)`／`SAKURA.break_lines(text, widths)`。
  - `runtime/module_registry.rs:125-137` — `register_sakura_script_module` が `@pasta_sakura_script` を `package.loaded` に登録。`factory.rs:172` から初期化時に呼ばれる。
- **Lua ランタイムスクリプト**: `crates/pasta_lua/pasta_scripts/pasta/`
  - `store.lua`（`STORE.co_scene`／`co_callback`／`actor_spots`）、`shiori/event/init.lua`（`set_co_scene`／`resume_until_valid`／`EVENT.fire`）、`shiori/event/callback.lua`（`CALLBACK.pending`／`stage_pending`／`consume_staged`／`sweep`）、`shiori/event/second_change.lua`（`OnSecondChange` が sweep／dispatch を駆動）。
  - `shiori/act.lua`・`shiori/sakura_builder.lua`（`BUILDER.build`）— **presentation/talk 出力の縫い目**。`act:talk` でトークン蓄積→`act:build`→`BUILDER.build(grouped_tokens, ...)`→`SAKURA.talk_to_script()` 呼び出し→さくらスクリプト文字列。
- **PoC 参照資産（feature-gated・default off）**: `crates/pasta_lua/src/actor_poc/`（`actor_thread.rs`・`mailbox.rs`・`responder.rs`・`teardown.rs`・`coroutine_probe.rs`・`sim_driver.rs`・`latency.rs`・`verdict.rs` ほか）。本番化の**実証済みテンプレート**。`pasta_shiori/Cargo.toml` は `actor-poc = ["pasta_lua/actor-poc"]` のみ（executor 依存は `pasta_lua` 側に閉じる）。

### 規約・パターン
- レイヤー依存方向: `pasta_dsl → pasta_core → pasta_lua → pasta_shiori`。コアは宿主非依存、アダプタが SHIORI 固有。
- ファイルサイズ目安 < 600 行（`oversized-file-decomposition` 適用済み）。分解は振る舞い不変が不変条件。
- テスト配置: 統合 `crates/*/tests/<feature>_test.rs`（単数）、src 内 `<feature>_tests.rs`（複数）。`#[ctor]` で `PASTA_DEBUG` 中和（固定ポート枯渇回避）。
- 既存 teardown idiom: `pasta_lua/src/debug/`（thread＋mpsc＋`Arc<AtomicBool>` shutdown＋`Drop` teardown＋socket2 エフェメラル）。`DebugHandle::Drop` の二重 join 回避（`take()`）。

### 統合面（Integration Surfaces）
- FFI 境界: `windows.rs` の `load`/`unload`/`request`（HGLOBAL 所有権移譲・ANSI/UTF-8 変換）。**この入口バイト列が外部観測点**＝R1 バイト不変の検証点。
- Rust↔Lua 境界: `@pasta_sakura_script` 登録（`module_registry.rs`）と `SHIORI.request` Function 呼び出し（`shiori.rs:call_lua_request`）。
- コア↔アダプタ境界（現状）: **唯一の描画関数 `SAKURA.talk_to_script()`** が `sakura_builder.lua` から呼ばれる。これが presentation event stream の縫い目候補（要件 R2/R3）。

## 2. 要件→資産マップ（Requirement-to-Asset Map）

| 要件 | 関連既存資産 | ギャップ区分 | 内容 |
|------|--------------|--------------|------|
| R1 バイト不変 | `windows.rs` FFI 入口・全既存テスト | **Constraint** | FFI 入口応答バイト列＝検証点。特性化テスト先行で全リファクタを縛る。差分検出は既存テスト＋応答バイト比較。 |
| R2 presentation event stream | `sakura_builder.lua`（`BUILDER.build`→`talk_to_script`）・Lua トークン`{type,actor,text}` | **Missing** | 宿主非依存マーカー契約は未定義。現状はトークン→直接さくらスクリプト文字列。マーカー粒度/スキーマ要設計（Q1）。 |
| R3 さくらスクリプト移設 | `pasta_lua/src/sakura_script/`・`@pasta_sakura_script` 登録・`sakura_builder.lua` | **Constraint/Missing** | 実装は存在するがコア側に配置。アダプタ責務への再配置（物理移動 or 論理隔離）が未決（Q2）。バイト不変が制約。 |
| R4 アクタースレッド＋VM pin | `actor_poc/actor_thread.rs`（実証済み）・`PastaLuaRuntime`（`!Send`） | **Missing（本番）** | PoC が実証。本番では `pasta_shiori` がアクタースレッドを所有し executor 依存を閉じ込める。出荷経路への接合が新規。 |
| R5 CH marshaling（GET/NOTIFY/drop→204） | `lua_request.rs`（method 確定）・`actor_poc/{mailbox,responder}.rs`・`shiori.rs:request` | **Missing（本番）** | PoC は再パースで出荷経路非干渉。本番は実 `request()` 同期経路を marshaling へ置換。GET timeout→204 閾値（6.68ms）要確定（Q3）。 |
| R6 単一直列キュー | `actor_poc/mailbox.rs`（実証済み） | **Missing（本番）** | PoC が直列順序を構造保証。本番 mailbox を出荷経路へ。 |
| R7 reload teardown 本番化 | `actor_poc/teardown.rs`・`debug/`(idiom)・`shiori.rs:load`(既存 reload) | **Missing（本番）** | 既存 reload は VM を `None` に落とすのみ。アクタースレッドの shutdown→join＋リーク不在を本番化。 |
| R8 `unsafe impl Send` 解消 | `shiori.rs:51-52`・`windows.rs:148`(`Arc<Mutex>`) | **Constraint→解消対象** | VM pin で構造的に充足し unsafe を撤去。`OnceLock`/`Arc<Mutex>` の所有モデル再設計が必要。 |
| R9 コルーチン/callback 意味論維持 | `store.lua`・`event/init.lua`・`callback.lua`・`second_change.lua`・`actor_poc/coroutine_probe.rs`(実証済み) | **Constraint** | PoC が executor 駆動下の生存を実証。Lua 意味論は無改変維持（Rust 化は marshaling の殻のみ）。 |

**複雑性シグナル**: 単純 CRUD ではなく、**スレッドモデル転換＋FFI 同期契約＋Lua コルーチン生存＋バイト不変**の複合。外部統合（`wintf_winmsg_executor`・Windows メッセージループ）あり。

## 3. 実装アプローチ選択肢

### Option A: 出荷経路を直接アクター化（in-place 置換）
`shiori.rs`/`windows.rs` の既存同期 `request()` 経路を、PoC の `ActorThread`/`Mailbox`/`Responder` を本番モジュールへ昇格して直接置換し、`Arc<Mutex>+unsafe impl Send` をアクタースレッド所有モデルへ差し替える。さくらスクリプトは登録経路をアダプタ起点へ移す。

- **対象**: `pasta_shiori/src/{windows.rs, shiori.rs}`、`pasta_lua` のアクタープリミティブ昇格、`module_registry` 登録経路。
- **トレードオフ**: ✅ 新規ファイル最小・PoC 資産を最大活用・最短経路。❌ 出荷の中核（FFI 入口・VM 所有）を一度に触るためバイト不変リスクが集中。`oversized-file` 方針との両立に注意。

### Option B: 新規アクターランタイムモジュール＋段階接合
`pasta_shiori` に新規アクターランタイムモジュール（PoC 由来を昇格・整理）を作り、まず presentation event stream 契約とさくらスクリプトアダプタを確立、次にアクタースレッド＋marshaling、最後に `unsafe` 撤去、という独立コンポーネントを段階接合する。

- **対象**: `pasta_shiori/src/actor/`（新規）、`pasta_shiori/src/sakura/`（移設先・新規）、`pasta_lua` 側マーカー出力 API。
- **トレードオフ**: ✅ 責務分離が明快・テスト容易・1 抽出=1 検証=1 コミットに乗せやすい。❌ ファイル増・境界 API 設計コスト・移行期に二経路併存の一貫性管理。

### Option C: ハイブリッド（特性化テスト先行の段階移行）— **推奨**
Constraints（制約）に従い、**特性化テスト（FFI 入口の応答バイト固定）を最初に敷設**したうえで、(1) presentation event stream 契約 + さくらスクリプトアダプタ移設（R2/R3、バイト不変を局所検証）、(2) アクタースレッド＋mailbox＋marshaling 本番化（R4/R5/R6、PoC 昇格）、(3) reload teardown 本番化（R7）、(4) `unsafe impl Send` 撤去（R8）の順で、各抽出を 1 検証 1 コミット（revert 可能）で進める。R1/R9 は全段階を貫く不変条件として常時検証。

- **段階方針**: 各段で「特性化テスト緑→抽出→再検証緑→コミット」を守る。さくらスクリプト移設を marshaling より先に置くのは、描画は純関数寄りでバイト差分の局所検証が容易なため。
- **リスク軽減**: 段階ごとに revert 境界を持つ。PoC の `actor_poc/` を参照テンプレートに保持（撤去は本番接合完了後）。
- **トレードオフ**: ✅ バイト不変リスクを分散・検証先行・記憶（`refactoring-safe-reversible`）と整合。❌ 計画コストと中間状態管理。プロジェクト方針「完全達成かリジェクト」に最適合。

## 4. Research Needed（設計フェーズ繰り延べ）
- **RN1**: `wintf_winmsg_executor` の `block_on` がメッセージループを回す前提、`spawn_local`／`JoinHandle`／メッセージ専用ウィンドウの Drop 解放挙動の本番確認（PoC Q2 申し送り。R1 で部分実証済みだが本番統合形は要確認）。
- **RN2**: presentation event のマーカー種別・粒度・データ表現（Q1）。現 Lua トークン＋既存さくらスクリプト出力からバイト不変で逆算する最小集合の確定。
- **RN3**: さくらスクリプト実装の物理移設範囲（Q2）。Rust 実装を `pasta_shiori` へ移すか、`pasta_lua` 内に残し登録経路のみアダプタ起点化するか。`line_breaker`（budouy 依存）・`TalkConfig` 依存の移送可否。
- **RN4**: GET timeout→204 フォールバック閾値（Q3、初期値 6.68ms）の実機実測調整方針。
- **RN5**: リリース `panic=abort` 下の drop→204 ガード保証範囲（Q4）。unwind 前提の成立範囲と、abort プロファイルでの代替防御要否。
- **RN6**: `OnceLock<RawShiori>`＋`Arc<Mutex>` の所有モデルを、アクタースレッドハンドル保持型へ再設計する形（R8）。`DllMain` の attach/detach ライフサイクルとの整合。

## 5. 実装複雑性・リスク

| 区分 | 評価 | 根拠（1 行） |
|------|------|--------------|
| 全体工数 | **L（1〜2 週間）** | スレッドモデル転換＋FFI 同期契約＋さくらスクリプト移設＋unsafe 撤去の複合。PoC 資産で短縮されるが出荷経路接合と特性化テスト敷設が重い。 |
| 全体リスク | **Medium** | 中核未知は PoC（GO+）で解消済み。残りは「実証済み方式の本番接合＋バイト不変維持」で、ガイダンス（PoC 設計／verdict）が明確。 |
| R2/R3（event stream＋移設） | 工数 M / リスク Medium | 縫い目は `talk_to_script` 単一で狭いが、マーカー契約設計とバイト不変逆算が新規。 |
| R4/R5/R6（アクター＋marshaling） | 工数 M / リスク Medium | PoC 昇格だが出荷 `request()` 同期経路の置換は High 寄り。直列 mailbox で順序は構造保証。 |
| R7（teardown） | 工数 S / リスク Low | debug idiom＋PoC `teardown.rs` 実証済み。 |
| R8（unsafe 撤去） | 工数 S〜M / リスク Medium | 所有モデル再設計が `DllMain` ライフサイクルと絡む。 |
| R1/R9（バイト不変・意味論維持） | 工数 M / リスク Medium | 全段横断の不変条件。特性化テスト品質に依存。 |

## 6. 設計フェーズへの推奨

- **推奨アプローチ**: Option C（特性化テスト先行のハイブリッド段階移行）。プロジェクト方針「リファクタは安全かつ可逆／1 抽出=1 検証=1 コミット」「完全達成かリジェクト」に最適合。
- **主要な設計判断（design フェーズで Boundary Commitments 化）**:
  1. presentation event stream のマーカースキーマ確定（RN2/Q1）と縫い目位置（`talk_to_script` 置換点）。
  2. さくらスクリプト移設の物理形態（RN3/Q2）。
  3. アクタースレッド所有・executor 閉じ込めの本番型（RN1/RN6）と `OnceLock`/`Arc<Mutex>`→アクターハンドルの所有モデル（R8）。
  4. GET timeout→204 閾値（RN4/Q3）と panic=abort 下のガード範囲（RN5/Q4）。
- **持ち越し研究項目**: RN1〜RN6（上記）。とりわけ RN1（executor 本番統合形）と RN2（マーカー契約）が設計の中核。
- **特性化テスト戦略**: FFI 入口（`request`）の応答バイト列を、代表的 SHIORI イベント列（OnBoot／OnSecondChange／GET property／コルーチン継続を含む）で固定するゴールデンテストを最初に敷設し、全段階で緑を維持する。

## OPEN QUESTIONS（要件ディスカッションへの申し送り／解決状況）

> **ディスカッション横断の確定事項（2026-06-22）**: **デバッグ容易性を本仕様の最優先関心事**と位置づけ、Requirement 10「デバッグ容易性の保全（作者デバッグ＋開発デバッグ）」を新設。対象は2層 — (A) ゴースト作者の `.pasta`/`.lua` DAP デバッグ（Phase 5 資産）を劣化させない、(B) 本リファクタが導入する並行機構（アクタースレッド・marshaling・VM pin・teardown）を開発者自身が実装中にデバッグできる環境（観測可能なログ点・決定論的テストハーネス・特性化テスト先行）を確保する。

1. **Q1（マーカー粒度）— ✅ 解決（ディスカッション #1）**: 「実装は最小・設計は将来対応可能」（B 寄りの C）に確定。最小集合（talk/アクター切替/wait/choice）のみ実装し、将来宿主マーカーを破壊的変更なしに追加できる拡張アーキテクチャとして設計。R2-AC6/AC7/AC8 化。具体スキーマは設計（RN2）へ。
2. **Q2（移設の物理形態）— ✅ 解決（ディスカッション #2）**: 「物理移設」案を**撤回**。**論理デカップリング（レンダラのアダプタ注入・VM 内レンダリング維持・物理移動なし・Lua/Rust とも `pasta_lua` 集約）**に確定。理由 = デバッグ容易性最優先（VM 外レンダリングは `.pasta` デバッグ可視範囲を損なう）＋ Lua 分散はソースマップ／DAP を崩壊させる＋ Rust デッドコード除去により物理同居は汚染とならない。宿主非依存は「レンダラ注入の差し替え可能性」で達成。R3 全面改訂。レンダラ注入 IF・`TalkConfig` 受け渡し・登録経路のアダプタ起点化は設計（RN3）へ。
3. **Q3（GET タイムアウト閾値）— ✅ 解決（ディスカッション #3）**: GET timeout→204 を**本仕様で実装**・初期閾値 **6.68ms 候補**採用。**デバッガ停止中も抑止しない**（停止中の 204 は次 `OnSecondChange` の `resume_until_valid` で回復・work 不失。LuaJIT プリエンプション不可ゆえタイムアウトは SHIORI 待機打ち切りのみ）。閾値は「通常処理では発火せず停止／異常時のみ発動する安全網」とし通常経路バイト不変（R1）を担保。実機チューニングは設計以降。R5-AC7〜9 化。
4. **Q4（panic=abort 下のガード）— ✅ 解決（ディスカッション #4）**: 「dev/test/unwind 限定の安全網」と明示受容＋正常経路の**構造的 panic-free 化を受入基準化**（R5-AC10/AC11）。リリースの `panic=abort` プロファイルは不変（横断ビルド変更はスコープ外）。既存 `windows.rs` の `catch_unwind` 姿勢と一貫・既存コードの panic 回避を新経路へ継続。dev=unwind/release=abort の分割はデバッグ容易性に資する（R10 整合）。
5. **Q5（PoC `actor_poc/` の扱い）— ✅ 解決（ディスカッション #5）**: 「昇格してから撤去」を本仕様の責務とする。デバッグ資産（`sim_driver`／`mailbox`／`responder`／`coroutine_probe`）は **R10-AC5 の本番テスト基盤へ昇格**、feature-gated 使い捨て足場（`verdict.rs`・scaffold・`actor-poc` gate）は**最終タスクで撤去**し、出荷 `pasta.dll` バイト不変を正規化 sha で確認（actor_poc は default-off ゆえ出荷に元から不含・feasibility 実証済み）。実装中は参照テンプレートとして保持。R10-AC8 化。

---

# 設計フェーズ Discovery / Synthesis（2026-06-22）

設計フェーズでの light＋統合フォーカス discovery（既存コード精査）の結果と、設計シンセシス（一般化・build vs adopt・簡素化）の結論を記録する。design.md はこの結論で自己完結している。

## 1. Discovery 確認事実（コード精査・design.md の根拠）

### 所有モデル現状（RN6 の前提）
- `static SHIORI: OnceLock<RawShiori<PastaShiori>>`（`windows.rs:14`）。`RawShiori(isize, Arc<Mutex<Option<PastaShiori>>>)`（`windows.rs:148`）。
- `PastaShiori` は `runtime: Option<PastaLuaRuntime>`（`!Send`）と `SHIORI.load/request/unload` の `Function` キャッシュを保持。`unsafe impl Send/Sync`（`shiori.rs:51-52`）の健全性は「`OnceLock` 単一＋`Mutex` 直列化＋メインスレッドのみ呼出し」の運用仮定依存。
- FFI dispatch は `catch_unwind(AssertUnwindSafe(..))` で panic→`MyError`→SHIORI 応答。リリース `panic=abort` で catch 到達不能（dev/test 保険）。
- **`HGLOBAL` 所有移譲**: `load`/`request` は受領 `HGLOBAL` を `ShioriString::capture`（drop で free）、応答は `clone_from_str_nofree`（呼出側 free）。`ShioriString` にも `unsafe impl Send/Sync`（排他所有・別 unsafe）が存在するが本仕様の解消対象外（VM 束縛 unsafe のみが対象）。
- **`req.method` 確定点**: `lua_request.rs:86-87` の pest `Rule::get|Rule::notify` で VM 投入前に確定。marshaling 分岐入力に再利用可能。

### 描画縫い目現状（RN3 の前提）
- 唯一の描画接点 = Lua `SAKURA.talk_to_script(actor, text)`。`@pasta_sakura_script` を `register_sakura_script_module`（`module_registry.rs:128-137`）がコア初期化時（`factory.rs:172`）に `package.loaded` へ無条件登録。
- `register(lua, config: Option<&TalkConfig>) -> LuaResult<Table>`（`sakura_script/mod.rs:50`）。`config` は `PastaConfig::talk()` 由来。budouy/unicode-width で改行処理。
- `BUILDER.build(grouped_tokens, config, actor_spots)`（`sakura_builder.lua`）が grouped token を走査、`emit_inner_token` が `talk/surface/wait/newline/clear/choice/yield` を分類処理。**マーカー最小集合（talk/actor 切替/wait/choice）の逆算出発点**。`act.lua:build` が `STORE.actor_spots` を in-place 更新。

### アクター PoC 実証テンプレート（RN1 の前提）
- `actor_poc/actor_thread.rs`: `ActorThread::spawn()` が `thread::spawn` 内で `wintf_winmsg_executor::block_on(actor_future(..))` を回し、`actor_future` 内で `PastaLuaRuntime::new(..)` を生成し `!Send` VM を pin。`MailboxSender`/`shutdown: Arc<AtomicBool>`/`join_handle: Option<JoinHandle>`/`actor_thread_id`。
- `block_on<'a, T: 'a>(future: impl Future<Output=T> + 'a) -> T`。再 poll は内部 `MSG_ID_WAKE`＋`Waker::wake_by_ref()`。メッセージ専用ウィンドウは executor 内部生成（ユーザコード非関与）。
- `mailbox.rs`: `enum ActorMsg{ Get{payload,script,responder}, Notify{payload,script}, Kick{scene} }`、`mpsc` FIFO、`MailboxReceiver: !Sync`（drain スレッド pin）。
- `responder.rs`: `enum PocResponse{ Value(u64), NoContent204 }`、`Responder{ tx: Option<Sender> }`、`reply()` XOR `drop()→204` exactly-once（`take()`）。**drop→204 は unwind 限定**（release `panic=abort` で非発火）。
- `teardown.rs`: `ReloadProbe::run_cycles(n)` が warmup 3＋N サイクルでハンドル（`GetProcessHandleCount`）／USER オブジェクト（`GetGuiResources`）成長を計測。teardown idiom = shutdown フラグ→wake→join、`take()` 二重 join 回避。
- feature gate: `actor-poc = ["dep:wintf-winmsg-executor", "windows-sys/Win32_System_Threading"]`（`pasta_lua/Cargo.toml`）。`wintf-winmsg-executor = { version = "0.0.3", optional = true }`。`pasta_shiori` は `actor-poc = ["pasta_lua/actor-poc"]` のみ（executor 直接依存なし）。

### debug backend スレッドモデル（R10 の前提）
- `set_global_hook(EVERY_LINE)` は **VM 実行スレッド上で同期発火**（`debug/hook.rs`）。ブレーク停止もフックループ内（同スレッド）で処理。LuaJIT は `lua_sethook` がメインステート全体に作用＝全コルーチン横断発火。
- socket bridge（`Transport` 単独所有・`!Sync`）／event encoder／transport listener は VM スレッドから `mpsc`（`Send` のみ）で分離。`mlua::Lua` はスレッドを越えない。
- `enable(lua, cfg, source_map) -> Result<Option<DebugHandle>, DebugError>`。無効時 `Ok(None)`（ゼロコスト）。`DebugHandle::Drop` = Terminated emit→30ms flush→shutdown フラグ→socket_handle join（port 解放）→encoder detach。`take()` 二重 join 回避。
- **設計含意**: VM がアクタースレッドへ移ると `set_global_hook`／`enable()` はアクタースレッドで呼ぶ必要がある。debug teardown はアクタースレッド join **前**に完了させ port 残留を防ぐ。

## 2. Synthesis 結論

### 一般化（Generalization）
- R2（マーカー契約）と R3（レンダラ注入）は「コア↔アダプタ境界を contract 化する」同一問題の二側面。マーカー型体系を拡張可能な境界 API（未知マーカーをレンダラが既定動作で受容）として一般化し、SHIORI さくらレンダラを最初の実装として注入する形に統一。実装は最小集合のみ（インタフェースのみ一般化、実装は一般化しない）。
- R5（marshaling）と R6（直列キュー）と R7（teardown）は `actor_poc` の `{mailbox,responder,teardown}` で既に統合実証済み。本番化は「同一プリミティブを出荷経路へ昇格」する単一の移行。

### Build vs Adopt
- **Adopt**: `wintf-winmsg-executor` 0.0.3（`block_on` メッセージループ・`!Send` future 駆動・内部メッセージウィンドウ）。PoC（GO+）で適合実証済み。自作の Win メッセージループは再発明。
- **Adopt**: `actor_poc/` の `ActorThread`/`Mailbox`/`Responder`/`Teardown` をテンプレートとして昇格（再設計しない）。
- **Adopt**: 既存 teardown idiom（`Arc<AtomicBool>`＋`take()`＋socket2 SO_REUSEADDR）と debug backend の VM スレッド同期フックモデル（変更せず VM=アクタースレッドへ移すのみ）。
- **Build（薄く）**: presentation マーカー層は新規だが「現状トークンを宿主非依存名で表現する薄い層」に留める（バイト不変逆算を崩さない）。

### 簡素化（Simplification）
- マーカー型を Rust 側へ厚く持たない。Lua トークン→さくらスクリプトのバイト不変経路を維持し、マーカー契約は薄い命名層。
- レンダラ注入 IF を過度に抽象化しない（単一実装の不要な間接化回避）。最小の差し替え点（注入なし＝既存挙動バイト不変）に留める。
- `actor_poc` の PoC 専用フィールド（`payload: u64`・`Kick{scene}`・`PocResponse::Value(u64)`）は本番昇格時に本番応答型（SHIORI 応答 `String`）へ整理。`verdict.rs`/`latency.rs`/`sim_driver` の使い捨て計測足場は撤去（`sim_driver` 等のデバッグ資産はテストへ昇格）。

## 3. 設計フェーズ繰り越し（OPEN QUESTIONS — design-discussion へ）

design.md の Open Questions と一致。要約:
1. **RN6 所有モデル**: `OnceLock<RawShiori>`＋`Arc<Mutex<Option<PastaShiori>>>`→`ActorHandle` 保持形。`OnceLock` 再代入不可ゆえ reload 再 spawn を許す所有形（`Mutex<Option<ActorHandle>>` 等）と `DllMain` attach/detach × `load`/`unload` 二重ライフサイクル整合。
2. **RN1 executor 本番統合形**: `wintf` 0.0.3 のメッセージ専用ウィンドウ Drop 解放・`JoinHandle` の `pasta_shiori` 側所有での本番統合形。executor 依存を `pasta_lua` 留置／`pasta_shiori` 移設のいずれにするか。
3. **RN4 GET timeout 閾値**: 6.68ms 初期値の実機実測調整と通常経路非発火の実証手段。
4. **RN2 マーカースキーマ**: Rust enum 形・Lua テーブル表現・両者整合の具体データ表現。
5. **RN3 レンダラ注入 IF 形**: trait object／関数ポインタ／登録フラグのいずれか。`TalkConfig` 受け渡しと `@pasta_sakura_script` アダプタ起点化の最小実装。
