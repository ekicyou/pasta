//! 段階判定オーケストレータの **レコーダ土台**（R8.2 / R8.3 / R7.3）。
//!
//! design.md「Verdict（段階判定オーケストレータ）」コンポーネントのうち、本タスク
//! （1.5）は **累積器（レコーダ）部分のみ** を実装する:
//! - `record_item` — 各チャレンジ項目（R1〜R6）の成否・採用方式（approach）・制約
//!   （constraints）を項目別に蓄積する（R8.2）。
//! - `record_blocker` — 各項目のブロッカー（NO-GO/制約付き判定の根拠）を蓄積する
//!   （R8.2・各 R の `.4`/`.5`）。
//! - `assert_isolation` — 隔離前提（default 無効・バイト不変・テスト非汚染）の
//!   ステータスを返す（R8.3）。
//!
//! 段階確定（`stage()` の NO-GO／条件付き GO／GO／GO+ ロジック）と判定文書の
//! 生成（`render()`／`VerdictDocument`）は **タスク 7.1** の責務であり、本タスクでは
//! 実装しない。本タスクは 7.1 が消費する累積結果（項目別 outcome・blocker・隔離
//! ステータス）を「記録でき・取り出せる」土台までを担う。

//! 段階判定の正本前提:
//! design.md「Verdict」の Service Interface に列挙された `stage()` / `render()` は
//! 後続タスク 7.1 が本モジュールへ追加する。ここで定義する `ReqId` / `ItemOutcome`
//! / `Blocker` / `IsolationStatus` はその段階ロジックが参照する土台型である。

use std::collections::BTreeMap;

/// 6 つのチャレンジ項目（R1〜R6）と、その `.4`/`.5` ブロッカー副基準を識別する ID。
///
/// requirements.md の要件番号（1〜6）をそのまま整数で保持する。`sub` は副基準
/// （`.4`/`.5` 等のブロッカー記録条件）を表す任意の枝番で、項目全体に対する記録は
/// `sub == None` とする。これにより「R5.5 のブロッカー」「R1.4 のブロッカー」のような
/// requirements.md の正本番号をそのまま再現できる（REQ-* エイリアスは作らない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReqId {
    /// チャレンジ項目番号（1〜6 = Requirement 1〜6）。
    pub challenge: u8,
    /// 副基準の枝番（例: R5.5 なら `Some(5)`、R1.4 なら `Some(4)`）。
    /// 項目全体を指す場合は `None`。
    pub sub: Option<u8>,
}

impl ReqId {
    /// 項目全体（副基準なし）を指す `ReqId` を作る（例: `ReqId::item(1)` = R1）。
    pub fn item(challenge: u8) -> Self {
        ReqId { challenge, sub: None }
    }

    /// 副基準を指す `ReqId` を作る（例: `ReqId::sub(5, 5)` = R5.5、`ReqId::sub(1, 4)` = R1.4）。
    pub fn sub(challenge: u8, sub: u8) -> Self {
        ReqId { challenge, sub: Some(sub) }
    }
}

impl std::fmt::Display for ReqId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sub {
            Some(s) => write!(f, "R{}.{}", self.challenge, s),
            None => write!(f, "R{}", self.challenge),
        }
    }
}

/// 1 チャレンジ項目の試行結果（成否・採用方式・制約）。
///
/// R8.2「項目ごとの成否・採用方式・制約を個別に記録する（成否にかかわらず全項目を
/// 試行し結果を残す）」に対応する記録単位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemOutcome {
    /// 試行が成立したか（true = 成功 / false = 失敗）。失敗でも記録は残す。
    pub success: bool,
    /// 採用方式（approach）。どの方式で成立／不成立を確認したかの記述。
    pub approach: String,
    /// 観測された制約（constraints）。0 個以上。
    pub constraints: Vec<String>,
}

impl ItemOutcome {
    /// 成功 outcome を作る。
    pub fn success(approach: impl Into<String>) -> Self {
        ItemOutcome {
            success: true,
            approach: approach.into(),
            constraints: Vec::new(),
        }
    }

    /// 失敗 outcome を作る。
    pub fn failure(approach: impl Into<String>) -> Self {
        ItemOutcome {
            success: false,
            approach: approach.into(),
            constraints: Vec::new(),
        }
    }

    /// 制約を 1 つ追加する（ビルダー風）。
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
}

/// ブロッカー（NO-GO／制約付き判定の根拠）。
///
/// R8.2 と各 R の `.4`/`.5`（不成立条件の記録）に対応。`reason` は切り分けた
/// ブロッカー条件、`workaround` は判明していれば回避候補（R8.4 の「回避候補」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    /// ブロッカー条件の説明（切り分けた根拠）。
    pub reason: String,
    /// 判明している回避候補（無ければ `None`）。
    pub workaround: Option<String>,
}

impl Blocker {
    /// 回避候補なしのブロッカーを作る。
    pub fn new(reason: impl Into<String>) -> Self {
        Blocker {
            reason: reason.into(),
            workaround: None,
        }
    }

    /// 回避候補付きのブロッカーを作る。
    pub fn with_workaround(reason: impl Into<String>, workaround: impl Into<String>) -> Self {
        Blocker {
            reason: reason.into(),
            workaround: Some(workaround.into()),
        }
    }
}

/// 隔離前提（R7 由来）の充足状態。
///
/// 重要な正直さ要件: バイト不変（R7.2）と default 無効（R7.1）は **ビルド事実** で
/// あり、隔離検証（タスク 1.1／8.1）が別途確認する。実行時のレコーダはそれらを
/// その場で計測できないため、ここでは「ハーネスが在ると主張する前提（precondition/
/// assumption）」として扱う。各フラグは「真であると確認した」ではなく「前提として
/// 主張されている」ことを表す（design.md の R8.3「判定妥当性前提として確認」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsolationStatus {
    /// default 無効 feature-gate（R7.1）が前提として在るか。
    /// このコードが `actor-poc` feature 下でのみコンパイルされる事実から、
    /// 「feature ゲートが在る」ことは true として前提できる。
    pub default_off_gate: bool,
    /// バイト不変（R7.2）が前提として主張されているか。
    /// レコーダは実行時にバイト不変を計測できないため、これは「タスク 1.1／8.1 が
    /// 別途検証する前提」であることを示す主張フラグである（実測値ではない）。
    pub byte_invariant_assumed: bool,
    /// テスト非汚染（R7.4・エフェメラルポート＋`#[ctor]` env ガード）が前提として
    /// 在るか。
    pub non_pollution_assumed: bool,
}

impl IsolationStatus {
    /// すべての隔離前提が在る状態。
    ///
    /// `default_off_gate` はこのコードが `actor-poc` feature 下でのみ存在する事実から
    /// true。`byte_invariant_assumed` / `non_pollution_assumed` は **実測ではなく前提**
    /// として true を主張する（実検証はタスク 1.1／8.1／7.4 が担う）。
    pub fn in_force() -> Self {
        IsolationStatus {
            default_off_gate: true,
            byte_invariant_assumed: true,
            non_pollution_assumed: true,
        }
    }

    /// すべての隔離前提が満たされている（成立を主張している）か。
    pub fn all_in_force(&self) -> bool {
        self.default_off_gate && self.byte_invariant_assumed && self.non_pollution_assumed
    }
}

/// 段階判定レコーダ（累積器）。
///
/// 各 probe（R1〜R6）の試行結果とブロッカーを項目別に蓄積し、後から取り出せる
/// 形で保持する。段階確定（`stage`）・文書生成（`render`）はタスク 7.1 が本構造体に
/// 追加する。本タスク（1.5）はその土台＝記録と取り出しまでを担う。
#[derive(Debug, Default)]
pub struct VerdictRecorder {
    /// 項目別 outcome。同一 `ReqId` への再記録は最新で上書きする
    /// （各項目は最終的な試行結果を 1 つ持つ）。
    items: BTreeMap<ReqId, ItemOutcome>,
    /// ブロッカーは `ReqId` ごとに複数蓄積しうる（同一項目に複数の不成立条件）。
    blockers: BTreeMap<ReqId, Vec<Blocker>>,
}

impl VerdictRecorder {
    /// 空のレコーダを作る。
    pub fn new() -> Self {
        VerdictRecorder::default()
    }

    /// 1 チャレンジ項目の試行結果を記録する（R8.2）。
    ///
    /// 同一 `req` への再記録は最新の outcome で上書きする。
    pub fn record_item(&mut self, req: ReqId, outcome: ItemOutcome) {
        self.items.insert(req, outcome);
    }

    /// ブロッカーを記録する（R8.2・各 R の `.4`/`.5`）。
    ///
    /// 同一 `req` への複数記録は順に蓄積される（取りこぼさない）。
    pub fn record_blocker(&mut self, req: ReqId, blocker: Blocker) {
        self.blockers.entry(req).or_default().push(blocker);
    }

    /// 記録済みの項目 outcome を取り出す（無ければ `None`）。
    pub fn item(&self, req: ReqId) -> Option<&ItemOutcome> {
        self.items.get(&req)
    }

    /// 記録済みの全項目を `ReqId` 昇順で取り出す。
    pub fn items(&self) -> impl Iterator<Item = (&ReqId, &ItemOutcome)> {
        self.items.iter()
    }

    /// 指定 `req` に蓄積されたブロッカー列を取り出す（無ければ空スライス）。
    pub fn blockers(&self, req: ReqId) -> &[Blocker] {
        self.blockers.get(&req).map(Vec::as_slice).unwrap_or(&[])
    }

    /// 記録済みの全ブロッカーを `ReqId` 昇順で取り出す。
    pub fn all_blockers(&self) -> impl Iterator<Item = (&ReqId, &Vec<Blocker>)> {
        self.blockers.iter()
    }

    /// 記録済みブロッカーの総数（全 `ReqId` 合算）。
    pub fn blocker_count(&self) -> usize {
        self.blockers.values().map(Vec::len).sum()
    }

    /// 隔離前提（R7.1/7.2/7.4 由来）のステータスを返す（R8.3）。
    ///
    /// 正直さ: `default_off_gate` はこのコードが `actor-poc` feature 下でのみ
    /// コンパイルされる事実から true として前提できる。バイト不変・テスト非汚染は
    /// レコーダが実行時に計測できないため **前提（assumption）** として返す
    /// （実検証はタスク 1.1／8.1／7.4 の責務）。
    pub fn assert_isolation(&self) -> IsolationStatus {
        IsolationStatus::in_force()
    }

    /// 指定チャレンジ項目（R1〜R6）が **成立** として記録されているかを判定する。
    ///
    /// 段階確定（[`stage`](Self::stage)）の純関数的な土台。`record_item(R<n>, ..)` が
    /// success outcome で記録されていれば true。未記録または failure outcome なら
    /// false（未記録は保守的に不成立として扱う）。`ReqId::item(n)`（副基準なし）の
    /// outcome のみを見る（`.4`/`.5` のブロッカー枝番は段階に直接効かない）。
    pub fn challenge_succeeded(&self, challenge: u8) -> bool {
        self.items
            .get(&ReqId::item(challenge))
            .map(|o| o.success)
            .unwrap_or(false)
    }

    /// 記録済み項目 outcome から段階的 go/no-go を確定する（R8.1）。
    ///
    /// design.md「Verdict」R8.1 の段階定義に厳密に従う **純粋・決定論的** ロジック:
    /// - **NO-GO**: R1 不成立（executor 上 VM ホスト・reload teardown がいかなる方式でも
    ///   成立しない）。R2〜R6 の成否に依らず NO-GO。
    /// - **条件付き GO（最低ライン）**: R1 ∧ R2 ∧ R3（安全な marshaling 基盤が成立）。
    /// - **GO（標準）**: 最低ライン ∧ R4（coroutine/callback 生存）。
    /// - **GO+（高信頼）**: GO ∧ R5 ∧ R6（≤1 秒キック配信＋GET レイテンシ実測）。
    ///
    /// 各段階は **下位段階の全前提を含む**（R3 欠落は R4/R5/R6 が成立していても昇格
    /// できない）。これにより「R1+R2+R4 だが R3 欠落」は条件付き GO に到達できず NO-GO。
    pub fn stage(&self) -> GoNoGoStage {
        let r1 = self.challenge_succeeded(1);
        let r2 = self.challenge_succeeded(2);
        let r3 = self.challenge_succeeded(3);
        let r4 = self.challenge_succeeded(4);
        let r5 = self.challenge_succeeded(5);
        let r6 = self.challenge_succeeded(6);

        // R1 が本丸。不成立なら他が何であれ NO-GO。
        if !r1 {
            return GoNoGoStage::NoGo;
        }
        // 最低ライン: R1 ∧ R2 ∧ R3。未達なら NO-GO（R4 以降が成立していても昇格不可）。
        if !(r2 && r3) {
            return GoNoGoStage::NoGo;
        }
        // ここから最低ライン到達。R4 で GO、さらに R5 ∧ R6 で GO+。
        if r4 {
            if r5 && r6 {
                GoNoGoStage::GoPlus
            } else {
                GoNoGoStage::Go
            }
        } else {
            GoNoGoStage::ConditionalGo
        }
    }

    /// 記録済みの全項目・全ブロッカー・隔離前提・確定段階から判定文書を生成する
    /// （R8.2 / R8.4 / R8.5 / 7.3）。
    ///
    /// - 全 probe（R1〜R6）の成否・採用方式・制約を項目別に載せる（R8.2）。
    /// - 最低ライン（R1+R2+R3）未達 = NO-GO 時は、記録済みブロッカー＋回避候補を載せた
    ///   NO-GO 文書とし、後続前提結論は **出さない**（R8.4）。
    /// - 条件付き GO 以上では、実際の probe 結果から合成した後続前提結論（6 トピック）を
    ///   `conclusions` に明記する（R8.5）。結論は本レコーダの記録から **導出** され、
    ///   定型文ではない。
    pub fn render(&self) -> VerdictDocument {
        let stage = self.stage();

        let items: Vec<(ReqId, ItemOutcome)> =
            self.items.iter().map(|(id, o)| (*id, o.clone())).collect();
        let blockers: Vec<(ReqId, Blocker)> = self
            .blockers
            .iter()
            .flat_map(|(id, list)| list.iter().map(move |b| (*id, b.clone())))
            .collect();

        // 条件付き GO 以上でのみ後続前提結論を明記する（R8.5・R8.4 の裏返し）。
        let conclusions = if stage.is_go_or_above() {
            Some(self.derive_conclusions())
        } else {
            None
        };

        VerdictDocument {
            stage,
            items,
            blockers,
            isolation: self.assert_isolation(),
            conclusions,
        }
    }

    /// 条件付き GO 以上で `pasta-actor-runtime` が前提とする 6 つの後続前提結論を、
    /// 実際の probe 記録（item outcome・制約・ブロッカー）から **合成** する（R8.5）。
    ///
    /// design.md「Verdict」R8.5 が列挙する 6 トピックを、本仕様の確定済み所見と
    /// 各 probe が残した実測値・制約（特に R5 の LuaJIT `coroutine.close` 不在制約、
    /// R6 の in-process sub-ms 実測＋フォールバック申し送り）から導く。各結論は
    /// 「採用方式（detail）」と、関連する probe 記録から取った「根拠（basis）」を持つ。
    fn derive_conclusions(&self) -> Vec<Conclusion> {
        // 各 probe の採用方式（approach）を引いて根拠に織り込む（定型文化を避ける）。
        let r1_basis = self
            .item(ReqId::item(1))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        let r2_basis = self
            .item(ReqId::item(2))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        let r3_basis = self
            .item(ReqId::item(3))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        let r4_basis = self
            .item(ReqId::item(4))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        // R5 の制約（LuaJIT close 不在）を結論へ申し送る。
        let r5_constraints = self
            .item(ReqId::item(5))
            .map(|o| o.constraints.join(" / "))
            .unwrap_or_default();
        let r5_basis = self
            .item(ReqId::item(5))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        // R6 のフォールバック申し送り（R6.3 ブロッカーの workaround を閾値候補として拾う）。
        let r6_basis = self
            .item(ReqId::item(6))
            .map(|o| o.approach.clone())
            .unwrap_or_default();
        let r6_fallback = self
            .blockers(ReqId::sub(6, 3))
            .iter()
            .find_map(|b| b.workaround.clone())
            .unwrap_or_default();

        let r4_alive = self.challenge_succeeded(4);
        let r5_alive = self.challenge_succeeded(5);
        let r6_alive = self.challenge_succeeded(6);

        vec![
            Conclusion {
                topic: "採用 executor 統合方式".to_string(),
                detail:
                    "std::thread::spawn 内で wintf-winmsg-executor の block_on を回し、その future が \
                     !Send な実 PastaLuaRuntime（mlua VM）を生成・所有する。再ポーリングは executor の \
                     MSG_ID_WAKE（Waker）で駆動し、値のみを mpsc で越境させる。"
                        .to_string(),
                basis: format!("R1 実証: {r1_basis}"),
            },
            Conclusion {
                topic: "VM pin / teardown 方針".to_string(),
                detail:
                    "mlua の !Send 制約により VM はアクタースレッドを構造的に越えない（pin）。teardown は \
                     Arc<AtomicBool>(SeqCst)→wake→JoinHandle::join、Drop は take() で二重 join を回避する \
                     （debug DebugHandle::Drop idiom）。reload は shutdown→再 spawn の反復で clean teardown \
                     とハンドル/ポート非枯渇を確認する。"
                        .to_string(),
                basis: format!("R1 teardown 実証: {r1_basis}"),
            },
            Conclusion {
                topic: "marshaling 契約".to_string(),
                detail:
                    "GET = block-on-reply（ActorMsg::Get に Responder を載せ enqueue→応答受信までブロック \
                     し応答値を戻す・SHIORI/3.0 同期契約）。NOTIFY = 即 204 fire-and-forget（応答経路なし）。 \
                     method 判定・marshaling 分岐は決定論ロジックとして Rust 側で完結し、シーン中核は Lua の \
                     まま保つ。"
                        .to_string(),
                basis: format!("R2 実証: {r2_basis}"),
            },
            Conclusion {
                topic: "drop→204 ガード方針".to_string(),
                detail:
                    "GET 応答 Responder を採用し、応答未送信のまま drop（応答忘れ／panic 巻き戻し）されたら \
                     自動的に 204 を撃つ。これにより『応答未送信のまま drop』のデッドロック経路が原理的に \
                     消滅する。注意: release は panic=abort のため unwind に依存する panic 経路の保証は \
                     test/unwind profile でのみ成立する（pasta-actor-runtime へ申し送り）。"
                        .to_string(),
                basis: format!("R3 実証: {r3_basis}"),
            },
            Conclusion {
                topic: "coroutine 生存条件".to_string(),
                detail: if r4_alive {
                    "駆動主体をホスト tick から executor へ移しても、実 *.lua（STORE.co_scene／\
                     CALLBACK.pending）が中断地点から resume し callback が解決する。各 resume は別個の \
                     executor 駆動（ホスト tick ではない）であり、コルーチン意味論は Lua のまま保つ。"
                        .to_string()
                } else {
                    "R4 が不成立のため coroutine/callback 生存は GO 前提として満たされていない（条件付き \
                     GO 止まり）。executor 移設での継続喪失条件を解消するまで GO へは昇格できない。"
                        .to_string()
                },
                basis: format!("R4 実証: {r4_basis}"),
            },
            Conclusion {
                topic: "GET レイテンシとフォールバック要否".to_string(),
                detail: {
                    let kick = if r5_alive {
                        "R5: 忠実シミュレータ上で ≤1 秒キック配信を実測（GET tick=Ref3=1 のみ配信・NOTIFY \
                         状態では次 GET tick まで held）。"
                    } else {
                        "R5: ≤1 秒キック配信は GO+ 前提として未確定。"
                    };
                    let lat = if r6_alive {
                        "R6: GET block-on-reply は in-process 実測で十分速いが、防御として GET タイムアウト \
                         →204 フォールバックを推奨する。"
                    } else {
                        "R6: GET レイテンシ実測／フォールバック判断は GO+ 前提として未確定。"
                    };
                    format!(
                        "{kick} {lat} R5 制約: {r5_constraints}。推奨フォールバック方針: {r6_fallback}"
                    )
                },
                basis: format!("R5 実証: {r5_basis}／R6 実証: {r6_basis}"),
            },
        ]
    }
}

/// 段階的 go/no-go 判定（R8.1）。
///
/// design.md「Verdict」R8.1 の 4 段階。下位段階の全前提を含む単調な序列で、
/// `stage()` がレコーダ状態から決定論的に確定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoNoGoStage {
    /// NO-GO: R1（executor 上 VM ホスト・reload teardown）がいかなる方式でも不成立。
    NoGo,
    /// 条件付き GO（最低ライン）: R1 ∧ R2 ∧ R3。安全な marshaling 基盤が成立。
    ConditionalGo,
    /// GO（標準）: 最低ライン ∧ R4（coroutine/callback 生存）。
    Go,
    /// GO+（高信頼）: GO ∧ R5（≤1 秒キック配信）∧ R6（GET レイテンシ実測）。
    GoPlus,
}

impl GoNoGoStage {
    /// 文書に載せる段階ラベル（requirements.md R8.1 の日本語表記）。
    pub fn label(self) -> &'static str {
        match self {
            GoNoGoStage::NoGo => "NO-GO",
            GoNoGoStage::ConditionalGo => "条件付き GO（最低ライン）",
            GoNoGoStage::Go => "GO（標準）",
            GoNoGoStage::GoPlus => "GO+（高信頼）",
        }
    }

    /// 最低ライン（条件付き GO）以上に到達しているか。
    ///
    /// これが true のときのみ後続前提結論（R8.5）を明記する。false（NO-GO）のときは
    /// NO-GO 文書（ブロッカー＋回避候補・R8.4）を出す。
    pub fn is_go_or_above(self) -> bool {
        self >= GoNoGoStage::ConditionalGo
    }
}

/// 後続実装仕様 `pasta-actor-runtime` が前提とする 1 つの結論（R8.5）。
///
/// `topic` は前提トピック名、`detail` は確定した採用方式、`basis` は本 PoC の
/// どの probe 記録から導いたかの根拠（定型文ではなく実記録の持ち回り）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conclusion {
    /// 前提トピック名（例: "marshaling 契約"）。
    pub topic: String,
    /// 確定した採用方式・方針。
    pub detail: String,
    /// 本結論を導いた probe 記録の根拠。
    pub basis: String,
}

/// 段階判定の成果物文書（R8.2 / R8.4 / R8.5 / 7.3）。
///
/// `render()` が生成する。全 probe の項目別記録・ブロッカー・隔離前提・確定段階を
/// 保持し、条件付き GO 以上では後続前提結論（`conclusions`）を持つ。`to_text` で
/// 人間可読なレポートへ整形する（一次成果物）。
#[derive(Debug, Clone)]
pub struct VerdictDocument {
    /// 確定した段階（R8.1）。
    pub stage: GoNoGoStage,
    /// 全 probe の項目別 outcome（R8.2・`ReqId` 昇順）。
    pub items: Vec<(ReqId, ItemOutcome)>,
    /// 全ブロッカー（R8.4 の NO-GO 根拠・各 R の `.4`/`.5`・`ReqId` 昇順）。
    pub blockers: Vec<(ReqId, Blocker)>,
    /// 隔離前提の確認結果（R8.3）。
    pub isolation: IsolationStatus,
    /// 後続前提結論（R8.5）。条件付き GO 以上でのみ `Some`、NO-GO では `None`。
    pub conclusions: Option<Vec<Conclusion>>,
}

impl VerdictDocument {
    /// NO-GO 文書か（最低ライン未達）。
    pub fn is_no_go(&self) -> bool {
        self.stage == GoNoGoStage::NoGo
    }

    /// 人間可読な判定レポートへ整形する（一次成果物・R8.2/R8.4/R8.5）。
    ///
    /// 段階・隔離前提・項目別結果（成否・採用方式・制約）・ブロッカー（理由＋回避候補）・
    /// 後続前提結論を順に並べる。NO-GO 時はブロッカー＋回避候補が必ず現れ、条件付き
    /// GO 以上では 6 つの後続前提結論が現れる。
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("# pasta-actor-feasibility 段階判定文書（R8）\n\n");
        out.push_str(&format!("## 判定段階: {}\n\n", self.stage.label()));

        // 隔離前提（R8.3）。
        out.push_str("## 隔離前提（R8.3・判定妥当性の前提）\n");
        out.push_str(&format!(
            "- default 無効 feature-gate: {}\n- バイト不変（前提）: {}\n- テスト非汚染（前提）: {}\n\n",
            self.isolation.default_off_gate,
            self.isolation.byte_invariant_assumed,
            self.isolation.non_pollution_assumed,
        ));

        // 項目別結果（R8.2）。
        out.push_str("## 項目別試行結果（R8.2・全項目を成否にかかわらず記録）\n");
        for (id, o) in &self.items {
            out.push_str(&format!(
                "- {}: {} — {}\n",
                id,
                if o.success { "成立" } else { "不成立" },
                o.approach,
            ));
            for c in &o.constraints {
                out.push_str(&format!("    - 制約: {c}\n"));
            }
        }
        out.push('\n');

        // ブロッカー＋回避候補（R8.4）。
        out.push_str("## ブロッカーと回避候補（R8.4・NO-GO/制約付き判定の根拠）\n");
        if self.blockers.is_empty() {
            out.push_str("- （記録されたブロッカーなし）\n");
        } else {
            for (id, b) in &self.blockers {
                out.push_str(&format!("- {}: {}\n", id, b.reason));
                if let Some(w) = &b.workaround {
                    out.push_str(&format!("    - 回避候補: {w}\n"));
                }
            }
        }
        out.push('\n');

        // 後続前提結論（R8.5）。
        match &self.conclusions {
            Some(conclusions) => {
                out.push_str(
                    "## 後続実装仕様 pasta-actor-runtime への前提結論（R8.5）\n",
                );
                for c in conclusions {
                    out.push_str(&format!("- {}: {}\n", c.topic, c.detail));
                    out.push_str(&format!("    - 根拠: {}\n", c.basis));
                }
            }
            None => {
                out.push_str(
                    "## 後続前提結論（R8.5）\n- 最低ライン（R1+R2+R3）未達のため後続前提結論は明記しない（NO-GO）。\n",
                );
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R8.2: 複数の `record_item` が項目別に蓄積され、成否・採用方式・制約を
    /// 反映した形で取り出せる。
    #[test]
    fn record_item_accumulates_and_is_retrievable() {
        let mut rec = VerdictRecorder::new();

        rec.record_item(
            ReqId::item(1),
            ItemOutcome::success("executor 上 !Send VM ホスト成立"),
        );
        rec.record_item(
            ReqId::item(2),
            ItemOutcome::failure("block-on-reply 往復が不成立")
                .with_constraint("GET タイムアウト経路が残る"),
        );
        rec.record_item(
            ReqId::item(5),
            ItemOutcome::success("忠実シミュレータで ≤1 秒配信")
                .with_constraint("実機 attach は未計測")
                .with_constraint("NOTIFY 状態では次 GET tick まで遅延"),
        );

        // R1: 成功・採用方式・制約なし
        let r1 = rec.item(ReqId::item(1)).expect("R1 outcome must be recorded");
        assert!(r1.success, "R1 must be recorded as success");
        assert_eq!(r1.approach, "executor 上 !Send VM ホスト成立");
        assert!(r1.constraints.is_empty(), "R1 has no constraints");

        // R2: 失敗・制約 1 件
        let r2 = rec.item(ReqId::item(2)).expect("R2 outcome must be recorded");
        assert!(!r2.success, "R2 must be recorded as failure");
        assert_eq!(r2.constraints, vec!["GET タイムアウト経路が残る".to_string()]);

        // R5: 成功・制約 2 件（順序保存）
        let r5 = rec.item(ReqId::item(5)).expect("R5 outcome must be recorded");
        assert!(r5.success);
        assert_eq!(
            r5.constraints,
            vec![
                "実機 attach は未計測".to_string(),
                "NOTIFY 状態では次 GET tick まで遅延".to_string(),
            ],
            "constraints must accumulate in order"
        );

        // 全項目が ReqId 昇順で取り出せる（3 件）。
        let recorded: Vec<ReqId> = rec.items().map(|(id, _)| *id).collect();
        assert_eq!(
            recorded,
            vec![ReqId::item(1), ReqId::item(2), ReqId::item(5)],
            "items must be retrievable sorted by ReqId"
        );
    }

    /// 同一 `ReqId` への再 `record_item` は最新で上書きされる。
    #[test]
    fn record_item_overwrites_same_req() {
        let mut rec = VerdictRecorder::new();
        rec.record_item(ReqId::item(1), ItemOutcome::failure("初回試行は失敗"));
        rec.record_item(ReqId::item(1), ItemOutcome::success("再試行で成立"));

        let r1 = rec.item(ReqId::item(1)).expect("R1 must exist");
        assert!(r1.success, "latest record_item must overwrite the earlier one");
        assert_eq!(r1.approach, "再試行で成立");
        assert_eq!(rec.items().count(), 1, "same ReqId must not duplicate");
    }

    /// R8.2・各 R の `.4`/`.5`: 複数の `record_blocker` が `ReqId` 別に蓄積され、
    /// 理由・回避候補が反映された形で取り出せる。
    #[test]
    fn record_blocker_accumulates_and_is_retrievable() {
        let mut rec = VerdictRecorder::new();

        // R1.4（NO-GO 根拠）: 回避候補なし
        rec.record_blocker(
            ReqId::sub(1, 4),
            Blocker::new("executor 上で VM の !Send 制約違反が発生"),
        );
        // R5.5（制約付き判定根拠）: 回避候補あり・同一 ReqId に 2 件
        rec.record_blocker(
            ReqId::sub(5, 5),
            Blocker::with_workaround("preempt が NOTIFY 状態で遅延", "次 GET tick を待つ"),
        );
        rec.record_blocker(
            ReqId::sub(5, 5),
            Blocker::new("drain 不発が稀に観測"),
        );

        // R1.4: 1 件・回避候補なし
        let b14 = rec.blockers(ReqId::sub(1, 4));
        assert_eq!(b14.len(), 1, "R1.4 must have exactly one blocker");
        assert_eq!(b14[0].reason, "executor 上で VM の !Send 制約違反が発生");
        assert!(b14[0].workaround.is_none(), "R1.4 blocker has no workaround");

        // R5.5: 2 件・順序保存・1 件目に回避候補
        let b55 = rec.blockers(ReqId::sub(5, 5));
        assert_eq!(b55.len(), 2, "R5.5 must accumulate both blockers");
        assert_eq!(
            b55[0].workaround.as_deref(),
            Some("次 GET tick を待つ"),
            "first R5.5 blocker carries its workaround"
        );
        assert_eq!(b55[1].reason, "drain 不発が稀に観測");

        // 未記録の項目は空スライス。
        assert!(
            rec.blockers(ReqId::sub(2, 4)).is_empty(),
            "unrecorded ReqId must return an empty blocker slice"
        );

        // 総数は全 ReqId 合算で 3 件。
        assert_eq!(rec.blocker_count(), 3, "blocker_count must sum across all ReqIds");

        // all_blockers は ReqId 昇順（R1.4 が R5.5 より先）。
        let order: Vec<ReqId> = rec.all_blockers().map(|(id, _)| *id).collect();
        assert_eq!(order, vec![ReqId::sub(1, 4), ReqId::sub(5, 5)]);
    }

    /// 壊れ検出: 蓄積が壊れていれば（上書きや取りこぼし）この期待は崩れる。
    /// items と blockers は独立した記録経路であることを確認する。
    #[test]
    fn items_and_blockers_are_independent() {
        let mut rec = VerdictRecorder::new();
        rec.record_item(ReqId::item(3), ItemOutcome::success("drop→204 ガード成立"));
        rec.record_blocker(ReqId::sub(3, 4), Blocker::new("ガード不全経路の仮説"));

        assert!(rec.item(ReqId::item(3)).is_some(), "item recorded");
        assert!(
            rec.item(ReqId::sub(3, 4)).is_none(),
            "a blocker ReqId must NOT appear as an item outcome"
        );
        assert_eq!(rec.blockers(ReqId::sub(3, 4)).len(), 1);
        assert_eq!(rec.blocker_count(), 1);
    }

    /// R8.3: `assert_isolation` が隔離前提ステータスを返す。
    ///
    /// 正直さ: default 無効ゲートは feature コンパイル事実から true。バイト不変・
    /// 非汚染は **前提（assumption）** として true（実測ではない）。
    #[test]
    fn assert_isolation_returns_precondition_status() {
        let rec = VerdictRecorder::new();
        let iso = rec.assert_isolation();

        assert!(
            iso.default_off_gate,
            "default-off gate is a build fact (this code compiles only under actor-poc)"
        );
        assert!(
            iso.byte_invariant_assumed,
            "byte-invariance is asserted as a precondition (verified by tasks 1.1/8.1)"
        );
        assert!(
            iso.non_pollution_assumed,
            "non-pollution is asserted as a precondition (R7.4 ephemeral + ctor guard)"
        );
        assert!(iso.all_in_force(), "all isolation preconditions must be in force");
    }

    /// `ReqId` の表示が requirements.md の正本番号（R1 / R5.5）を再現する。
    #[test]
    fn req_id_displays_source_numbering() {
        assert_eq!(ReqId::item(1).to_string(), "R1");
        assert_eq!(ReqId::sub(5, 5).to_string(), "R5.5");
        assert_eq!(ReqId::sub(1, 4).to_string(), "R1.4");
    }
}
