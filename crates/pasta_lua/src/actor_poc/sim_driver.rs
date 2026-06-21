//! SimDriver 忠実シミュレータ（R5.6）。
//!
//! 「実 SSP 相当」を、OnSecondChange の周期と `Status: talking` 遷移を忠実に
//! 再現する自前ドライバとして定義する（design.md「SimDriver（忠実シミュレータ）」）。
//!
//! # 役割（生成器であって再パーサではない）
//!
//! SimDriver は OnSecondChange 周期で tick を発行し、talk FIFO の drain 契機を
//! 駆動する**独立した生成器**である。各 tick を GET（Reference3=1・再生可/上書き可）
//! ／NOTIFY（Reference3=0・再生不能）として発火し、**自身が method タグ付け**する。
//! これは議題2の Marshal pest 再パース（受信リクエスト文字列の解析）とは異なり、
//! `tick(playable: bool)` の bool 引数のみから決定論的に method/Reference3 を
//! 決める——request 文字列を一切受け取らないため、Marshal の再パースには構造的に
//! 依存し得ない。
//!
//! # gate 権威（Q4）
//!
//! gate の権威は OnSecondChange の method（GET/NOTIFY ≡ Reference3）である。
//! ukadoc 明記: 再生不能（NOTIFY/Ref3=0）時は返したスクリプトが無視される。
//! **GET tick（Ref3=1）が来ている＝トーク上書き可**を決め打ちする
//! （設計ディスカッション#3で確定）。SimDriver は GET tick と NOTIFY tick の双方、
//! および `Status: talking` 状態を produce し、KickHarness(5.2)・Latency(6.1) が
//! これを consume する。
//!
//! 本ファイルは出荷コードを改変しない feature-gate モジュールであり、`actor-poc`
//! 無効時はコンパイル単位に現れない（リリースビルドはバイト不変・R7.1/R7.2）。

/// OnSecondChange tick の method タグ。GET/NOTIFY ≡ Reference3 の権威を表す。
///
/// - `Get`: Reference3=1。再生可・トーク上書き可。配信可否 gate を通す。
/// - `Notify`: Reference3=0。再生不能。返したスクリプトは SSP に無視される。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimMethod {
    /// GET tick。Reference3=1・再生可/上書き可。
    Get,
    /// NOTIFY tick。Reference3=0・再生不能。
    Notify,
}

impl SimMethod {
    /// この method に対応する OnSecondChange の Reference3 値（GET=1／NOTIFY=0）。
    ///
    /// method タグと Reference3 が常に連動する（自己整合）ことを型レベルで担保する
    /// ための導出関数。tick 生成時にこの値を `SimTick::reference3` へ載せる。
    pub fn reference3(self) -> u8 {
        match self {
            SimMethod::Get => 1,
            SimMethod::Notify => 0,
        }
    }
}

/// OnSecondChange 1 周期が発火する 1 tick。
///
/// `method`（GET/NOTIFY）と `reference3`（1/0）を併せ持ち、両者は常に連動する
/// （`SimMethod::reference3` で導出）。KickHarness(5.2) はこの tick を drain 契機
/// ＋配信可否 gate（GET tick のみ配信）として消費し、Latency(6.1) は GET tick の
/// 反復で往復レイテンシを実測する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimTick {
    /// この tick の method タグ（GET/NOTIFY ≡ Reference3 の権威）。
    pub method: SimMethod,
    /// OnSecondChange の Reference3 値（GET=1・再生可/上書き可／NOTIFY=0・再生不能）。
    pub reference3: u8,
}

impl SimTick {
    /// この tick で配信可能か（GET tick＝Ref3=1 のみ true）。
    ///
    /// 配信可否 gate の権威判定。NOTIFY tick（Ref3=0）では返したスクリプトが
    /// SSP に無視されるため `false` を返す。
    pub fn is_playable(self) -> bool {
        matches!(self.method, SimMethod::Get)
    }
}

/// OnSecondChange の周期と `Status: talking` 遷移を忠実に再現する自前ドライバ。
///
/// `tick(playable)` で 1 周期を発火（自身が method タグ付け）し、`set_talking` で
/// `Status: talking` 遷移（礼儀 gate 用）を制御する。talking 状態は tick 発火を
/// またいで保持され、`is_talking` で観測できる。
#[derive(Debug, Default)]
pub struct SimDriver {
    /// `Status: talking`（再生中）状態。`set_talking` で遷移し、KickHarness(5.2)
    /// の礼儀 gate（talking 中は非即時 drain を抑止）が参照する。
    talking: bool,
}

impl SimDriver {
    /// 新規 SimDriver を生成する。初期状態は非 talking（再生していない）。
    pub fn new() -> Self {
        SimDriver { talking: false }
    }

    /// OnSecondChange 1 周期を発火する。
    ///
    /// `playable=true` → GET tick（Reference3=1・再生可/上書き可）、
    /// `playable=false` → NOTIFY tick（Reference3=0・再生不能）。
    ///
    /// method タグ付けはこの bool 引数のみから決定論的に行われ、受信リクエスト
    /// 文字列のパース（Marshal の pest 再パース）には依存しない独立した生成器で
    /// ある。`reference3` は `SimMethod::reference3` で method から導出するため、
    /// method と Reference3 は構造的に連動する。
    pub fn tick(&mut self, playable: bool) -> SimTick {
        let method = if playable {
            SimMethod::Get
        } else {
            SimMethod::Notify
        };
        SimTick {
            method,
            reference3: method.reference3(),
        }
    }

    /// `Status: talking`（再生中）遷移を制御する（礼儀 gate 用）。
    ///
    /// `true` で talking（再生中）、`false` で非 talking へ遷移する。設定した
    /// 状態は後続の `tick` 発火をまたいで保持される。
    pub fn set_talking(&mut self, talking: bool) {
        self.talking = talking;
    }

    /// 現在 `Status: talking`（再生中）かを返す。
    ///
    /// KickHarness(5.2) の礼儀 gate（talking 中は非即時 drain を抑止）が参照する
    /// 観測点。
    pub fn is_talking(&self) -> bool {
        self.talking
    }
}
