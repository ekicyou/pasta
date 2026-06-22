//! 宿主非依存 presentation マーカー契約（コア出力の薄い層）。
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R2.1, R2.2, R2.3, R2.5, R2.6, R2.7, R2.8
//! - 設計: design.md の **PresentationMarker** コンポーネント（pasta_lua / Core）、
//!   File Structure Plan（`crates/pasta_lua/src/presentation/` mod.rs + marker.rs）。
//!
//! ## 概要
//! 本モジュールは、エンジンコア（`pasta_lua`）が出力するシーン実行結果を **宿主非依存の
//! presentation マーカー列**として表現する薄い型層を確立する（R2.1/R2.2）。さくらスクリプト
//! のような宿主固有の出力形式や executor 選択に依存しない（R2.3）。
//!
//! 本タスク（2.1）は **内部リファクタかつバイト不変**であり、レンダリング経路
//! （`sakura_builder.lua`・`@pasta_sakura_script` 登録・`factory.rs`）は一切変更しない。
//! 本層はあくまで「現状の Lua トークンを宿主非依存名で表現する契約」を導入するに留め、
//! 最終さくらスクリプト出力をゴールデンとバイト不変に保つ（レンダラ・デカップリングは
//! task 2.2）。
//!
//! ## 拡張可能境界（R2.6）
//! [`RenderBoundary`] は、レンダラが「対応マーカー」を処理し「未対応/未知マーカー」を
//! **エラーでなく既定動作（無視/パススルー）**で扱う契約を表す。これにより将来宿主が
//! 新マーカー（`PresentationMarker::Extension` 経由）を追加しても、**既存コア・既存
//! アダプタ・既存テストを破壊しない**（単なる `#[non_exhaustive]` を超えた境界 API）。

pub mod marker;

pub use marker::{PresentationMarker, TokenFields};

/// マーカー列を宿主固有出力へ畳み込むレンダラの拡張可能境界（R2.6）。
///
/// アダプタ（将来の SHIORI さくらレンダラ等）はこの trait を実装し、対応マーカーを
/// [`RenderBoundary::render_known`] で処理する。**未対応/未知マーカーは
/// [`RenderBoundary::on_unknown`] の既定動作（無視 = 空出力）で吸収される**ため、
/// 新マーカー追加時にレンダラ側を破壊的に変更する必要がない。
///
/// 本タスクではこの境界 API の **契約のみ**を確立する（実レンダリング経路は不変）。
pub trait RenderBoundary {
    /// 対応マーカーをこのレンダラの宿主固有表現へ変換する。
    ///
    /// 対応できないマーカー（最小集合外・未知）は `None` を返し、
    /// 既定の [`on_unknown`](RenderBoundary::on_unknown) に委ねる。
    fn render_known(&mut self, marker: &PresentationMarker) -> Option<String>;

    /// 未対応/未知マーカーの既定動作。
    ///
    /// 既定実装は **空文字列（無視/パススルー）**。エラーにしないことで拡張耐性を保つ
    /// （R2.6: 未対応マーカーを既定動作で扱う）。レンダラは必要に応じて上書きできる。
    fn on_unknown(&mut self, _marker: &PresentationMarker) -> String {
        String::new()
    }

    /// マーカー 1 件をレンダリングする（既定: 対応なら `render_known`、未対応なら
    /// `on_unknown`）。未知マーカーでも **panic せず必ず文字列を返す**。
    fn render_one(&mut self, marker: &PresentationMarker) -> String {
        match self.render_known(marker) {
            Some(s) => s,
            None => self.on_unknown(marker),
        }
    }

    /// マーカー列を順に畳み込み、宿主固有出力へ連結する。
    ///
    /// 未知マーカーを含む列でも破壊せず（panic/早期エラーなし）に処理できる（R2.6）。
    fn render_stream<'a, I>(&mut self, markers: I) -> String
    where
        I: IntoIterator<Item = &'a PresentationMarker>,
    {
        let mut out = String::new();
        for m in markers {
            out.push_str(&self.render_one(m));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    //! 宿主非依存 presentation マーカー層の単体テスト。
    //!
    //! - 最小集合（talk / actor-switch / wait / choice）の分類・表現を実証（R2.1/R2.7）。
    //! - 拡張境界が未知/新マーカーを **破壊的変更なし**に既定動作で受容することを実証
    //!   （R2.6）= 「仮想的な新マーカー variant を含む列を既存レンダラが panic/error なしに
    //!   処理できる」テスト。
    //! - UI 独立（Wait はマーカーのみ・さくらスクリプトタグを持たない）を実証（R2.5）。

    use super::*;

    /// テスト用のトークンフィールド供給体（VM Lua テーブルの代理）。
    #[derive(Default)]
    struct Fields {
        actor: Option<String>,
        text: Option<String>,
        ms: Option<u64>,
        display: Option<String>,
        target: Option<String>,
    }

    impl TokenFields for Fields {
        fn actor(&self) -> Option<String> {
            self.actor.clone()
        }
        fn text(&self) -> Option<String> {
            self.text.clone()
        }
        fn ms(&self) -> Option<u64> {
            self.ms
        }
        fn display(&self) -> Option<String> {
            self.display.clone()
        }
        fn target(&self) -> Option<String> {
            self.target.clone()
        }
    }

    // --- 最小集合の分類（R2.1/R2.2/R2.7） ---------------------------------

    #[test]
    fn classifies_talk_line() {
        let f = Fields {
            actor: Some("sakura".into()),
            text: Some("こんにちは".into()),
            ..Default::default()
        };
        let m = PresentationMarker::classify("talk", &f);
        assert_eq!(
            m,
            PresentationMarker::Talk {
                actor: Some("sakura".into()),
                text: "こんにちは".into()
            }
        );
        assert!(m.is_minimal());
    }

    #[test]
    fn classifies_sakura_script_as_talk_line() {
        // sakura_script トークンも宿主非依存「Talk ライン」として表現する。
        let f = Fields {
            actor: Some("kero".into()),
            text: Some("\\s[10]".into()),
            ..Default::default()
        };
        let m = PresentationMarker::classify("sakura_script", &f);
        assert_eq!(
            m,
            PresentationMarker::Talk {
                actor: Some("kero".into()),
                text: "\\s[10]".into()
            }
        );
    }

    #[test]
    fn classifies_actor_switch() {
        let f = Fields {
            actor: Some("kero".into()),
            ..Default::default()
        };
        let m = PresentationMarker::classify("actor", &f);
        assert_eq!(
            m,
            PresentationMarker::ActorSwitch {
                actor: Some("kero".into())
            }
        );
        assert!(m.is_minimal());
    }

    #[test]
    fn classifies_wait_as_marker_only() {
        // R2.5: Wait はマーカーのみ（ms 値）であり、さくらスクリプトタグ（\w[..]）を持たない。
        let f = Fields {
            ms: Some(950),
            ..Default::default()
        };
        let m = PresentationMarker::classify("wait", &f);
        assert_eq!(m, PresentationMarker::Wait { ms: 950 });
        // 宿主固有のタグ形式が型に焼き込まれていないこと（UI 独立）の確認。
        if let PresentationMarker::Wait { ms } = m {
            assert_eq!(ms, 950);
        } else {
            panic!("expected Wait marker");
        }
    }

    #[test]
    fn classifies_choice() {
        let f = Fields {
            display: Some("はい".into()),
            target: Some("OnYes".into()),
            ..Default::default()
        };
        let m = PresentationMarker::classify("choice", &f);
        assert_eq!(
            m,
            PresentationMarker::Choice {
                display: "はい".into(),
                target: "OnYes".into()
            }
        );
        assert!(m.is_minimal());
    }

    #[test]
    fn minimal_set_excludes_out_of_scope_tokens() {
        // R2.7: 最小集合外（surface/newline/clear 等）は具体実装せず拡張境界（既定パススルー）。
        for ty in ["surface", "newline", "clear", "raw_script", "spot", "choice_timeout"] {
            let m = PresentationMarker::classify(ty, &Fields::default());
            assert!(
                !m.is_minimal(),
                "{ty} should not be a minimal-set marker (kept as Extension passthrough)"
            );
            assert!(matches!(m, PresentationMarker::Extension { .. }));
        }
    }

    #[test]
    fn unknown_token_classifies_without_data_loss_or_panic() {
        // 未知トークン型でも分類が失敗（panic/欠落）しないこと。
        let m = PresentationMarker::classify("future_novel_marker", &Fields::default());
        match m {
            PresentationMarker::Extension { raw, .. } => assert_eq!(raw, "future_novel_marker"),
            other => panic!("expected Extension, got {other:?}"),
        }
    }

    // --- 拡張可能境界（R2.6） ----------------------------------------------

    /// 最小集合のみ対応する代表レンダラ（既存アダプタの代理）。
    /// 未対応/未知マーカーは既定の `on_unknown`（無視）に委ねる。
    struct MinimalRenderer;

    impl RenderBoundary for MinimalRenderer {
        fn render_known(&mut self, marker: &PresentationMarker) -> Option<String> {
            match marker {
                PresentationMarker::Talk { text, .. } => Some(text.clone()),
                PresentationMarker::ActorSwitch { actor } => {
                    Some(format!("[switch:{}]", actor.clone().unwrap_or_default()))
                }
                PresentationMarker::Wait { ms } => Some(format!("[wait:{ms}]")),
                PresentationMarker::Choice { display, target } => {
                    Some(format!("[choice:{display}->{target}]"))
                }
                // 最小集合外（Extension 含む）は未対応として None。
                _ => None,
            }
        }
    }

    #[test]
    fn extension_boundary_tolerates_unknown_marker_without_breaking() {
        // R2.6 の核心: 既存レンダラ（最小集合のみ対応）が、**仮想的な新マーカー**
        // （Extension variant で表現）を含む列を panic/error なしに処理できる。
        // これは「破壊的変更なしの拡張」の構造的実証である。
        let stream = [
            PresentationMarker::Talk {
                actor: Some("sakura".into()),
                text: "やあ".into(),
            },
            // 将来宿主（ノベルゲーム push/常駐系）の新マーカーを模した拡張 variant。
            PresentationMarker::Extension {
                kind: "novel:portrait".into(),
                raw: "portrait".into(),
            },
            PresentationMarker::Wait { ms: 100 },
        ];

        let mut renderer = MinimalRenderer;
        // 未知マーカーがあっても全体のレンダリングは成功し、既知分は欠落しない。
        let out = renderer.render_stream(stream.iter());
        assert_eq!(out, "やあ[wait:100]"); // Extension は既定（無視）でスキップ
    }

    #[test]
    fn unknown_marker_default_is_passthrough_ignore_not_error() {
        // 既定 on_unknown は空文字列（無視）であり、エラーにしない（R2.6）。
        let mut renderer = MinimalRenderer;
        let unknown = PresentationMarker::Extension {
            kind: "novel:bgm".into(),
            raw: "bgm".into(),
        };
        assert_eq!(renderer.render_one(&unknown), "");
    }
}
