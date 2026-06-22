//! 宿主非依存 presentation マーカーの最小集合型表現。
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R2.1（出力をマーカー列として表現）／R2.2（さくらスクリプト文字列でなく
//!   宿主非依存マーカーとして載せる）／R2.5（UI 独立: Wait はマーカーのみ）／
//!   R2.6（破壊的変更なしに拡張可能）／R2.7（実装は最小集合に限定）。
//! - 設計: design.md の **PresentationMarker** コンポーネント
//!   （pasta_lua / Core・「最小・薄い導入に留める（simplification）」）。
//!
//! ## 本層の位置づけ（薄い契約層）
//! 本仕様は **外部 SHIORI 挙動バイト不変の純内部リファクタ**である。本層は現状の
//! Lua 側トークン（`{type="talk", actor, text}`・`surface`/`wait`/`newline`/`clear`/
//! `choice` 等）を **宿主非依存の名前**で表現する薄い型層を確立するに留め、レンダリング
//! 経路（`sakura_builder.lua`・`@pasta_sakura_script` 登録）は本タスクでは一切変更しない
//! （レンダラ・デカップリングは task 2.2）。これにより最終さくらスクリプト出力は
//! ゴールデンとバイト不変であることが構造的に保証される。
//!
//! 「真実の源」は VM 内の Lua トークンであり（design.md）、本 Rust 型は **コア↔アダプタ
//! 境界で用いうる最小・拡張可能 enum**である。未知マーカー（将来宿主の push/常駐系）は
//! エラーにせず既定動作（パススルー/無視）で扱える契約とすることで、最小集合外を
//! 実装することなく破壊的変更なしの拡張余地を確保する（R2.6/R2.7）。

/// 宿主非依存 presentation マーカー（最小集合 + 拡張境界）。
///
/// 最小集合（R2.7）= `Talk` / `ActorSwitch` / `Wait` / `Choice` のみを具体実装する。
/// `#[non_exhaustive]` に加え、未知/将来マーカーを表現する `Extension` variant を備え、
/// レンダラ IF が未対応マーカーをエラーでなく既定動作で受容できる契約とする（R2.6）。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PresentationMarker {
    /// talk ライン（発話）。`actor` は発言アクター名（無名なら `None`）。
    Talk { actor: Option<String>, text: String },
    /// アクター切替（発言主体の切り替え）。
    ActorSwitch { actor: Option<String> },
    /// 待機（UI 独立: ms のみを保持し、さくらスクリプト固有のタグ形式は持たない・R2.5）。
    Wait { ms: u64 },
    /// 選択肢。`display` は表示文字列、`target` はジャンプ先。
    Choice { display: String, target: String },
    /// 拡張境界: 最小集合外（将来宿主のマーカー等）を破壊的変更なしに表現する。
    ///
    /// 本仕様では具体実装しない（R2.7）。`kind` に宿主非依存の種別名、`raw` に元トークンの
    /// 型名を保持し、レンダラはこれをエラーでなく既定動作（無視/パススルー）で扱える（R2.6）。
    Extension { kind: String, raw: String },
}

impl PresentationMarker {
    /// 最小集合（実装済み）のマーカーか判定する。
    ///
    /// `Extension` は拡張境界であり最小集合ではない。
    pub fn is_minimal(&self) -> bool {
        !matches!(self, PresentationMarker::Extension { .. })
    }

    /// 既存 Lua トークンの **型名（`token.type`）** を、宿主非依存マーカーへ分類する
    /// 薄い変換契約。
    ///
    /// 現状の `sakura_builder.lua` の `emit_inner_token` 分類・`act.lua` のトークン
    /// 蓄積を出発点に、**バイト不変を崩さない範囲で**型名→マーカーを逆算する。
    /// 最小集合に該当しないトークン型は `Extension`（既定パススルー）として返し、
    /// 未知トークンでも分類が失敗（panic/None でのデータ欠落）しないことを保証する。
    ///
    /// # 引数
    /// - `token_type`: Lua トークンの `type` 文字列（例: `"talk"`, `"wait"`, `"choice"`）
    /// - `fields`: トークンの付随フィールドを引くアクセサ（型名のみでは値が定まらない
    ///   `talk`/`wait`/`choice` 等のためのフィールド供給）
    pub fn classify(token_type: &str, fields: &dyn TokenFields) -> PresentationMarker {
        match token_type {
            // talk ライン: talk と sakura_script はいずれも発話テキストを宿主非依存に
            // 「Talk ライン」として表現する（さくらスクリプト変換は行わない）。
            "talk" | "sakura_script" => PresentationMarker::Talk {
                actor: fields.actor(),
                text: fields.text().unwrap_or_default(),
            },
            // アクター切替: act.lua のグループ化で生成される type="actor" 境界。
            "actor" => PresentationMarker::ActorSwitch {
                actor: fields.actor(),
            },
            // wait: UI 独立で ms のみ保持。
            "wait" => PresentationMarker::Wait {
                ms: fields.ms().unwrap_or(0),
            },
            // choice: 表示文字列とジャンプ先。
            "choice" => PresentationMarker::Choice {
                display: fields.display().unwrap_or_default(),
                target: fields.target().unwrap_or_default(),
            },
            // 最小集合外（surface/newline/clear/raw_script/spot/choice_timeout/未知）は
            // 拡張境界として既定パススルー。具体実装は本仕様の対象外（R2.7）。
            other => PresentationMarker::Extension {
                kind: format!("passthrough:{other}"),
                raw: other.to_string(),
            },
        }
    }
}

/// `classify` にトークンの付随フィールドを供給する薄いアクセサ契約。
///
/// 型名（`token.type`）だけでは `talk`/`wait`/`choice` 等の値が定まらないため、
/// 値を引く責務をこの trait に委譲する。VM 内 Lua テーブル / テスト用構造体の
/// いずれからも実装でき、コア↔アダプタ境界をホスト非依存に保つ。
///
/// 既定実装は全て `None`/`unwrap_or` 既定値へ落ちるため、未対応フィールドを
/// 持つ実装でも分類が失敗しない（拡張耐性・R2.6）。
pub trait TokenFields {
    /// 発言アクター名（無名/非該当なら `None`）。
    fn actor(&self) -> Option<String> {
        None
    }
    /// 発話テキスト（talk/sakura_script）。
    fn text(&self) -> Option<String> {
        None
    }
    /// 待機ミリ秒（wait）。
    fn ms(&self) -> Option<u64> {
        None
    }
    /// 選択肢の表示文字列（choice）。
    fn display(&self) -> Option<String> {
        None
    }
    /// 選択肢のジャンプ先（choice）。
    fn target(&self) -> Option<String> {
        None
    }
}
