//! レンダラ注入シーム（さくらスクリプト登録のアダプタ起点デカップリング）。
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R2.4, R3.1, R3.2, R3.3, R3.4, R3.5, R3.6
//! - 設計: design.md の **SakuraRenderer（アダプタ注入）** コンポーネント、
//!   Service Interface（`register_sakura_script_module(lua, config, renderer)` —
//!   既定 = SHIORI さくらレンダラ = 既存挙動）、RN3（注入 IF 形）。
//!
//! ## 目的（R3 論理デカップリング）
//! これまで `factory.rs` は `register_sakura_script_module` を **コア無条件起点**で
//! 呼び出していた（コアがさくら描画を必須責務として hard-code 保持）。本シームは、
//! `@pasta_sakura_script` の登録を **アダプタが注入するレンダラ**へ置き換える。
//! SHIORI 宿主（既定）時はアダプタがさくらスクリプトレンダラを注入し、コアの
//! トーク構築フローがそれを呼ぶ（R3.1/R3.2）。**注入が既定（SHIORI さくらレンダラ）
//! の場合、出力は既存どおりバイト不変**（R3.5/R3.6）。
//!
//! ## マーカー契約との結線（task 2.1 との整合）
//! 注入レンダラは概念的に [`crate::presentation::RenderBoundary`] の具体実装であり、
//! 宿主非依存の [`crate::presentation::PresentationMarker`] 列を宿主固有出力（さくら
//! スクリプト文字列）へ畳み込む責務を担う。本シームの注入境界はその **契約（マーカー
//! 列 → 宿主固有文字列）** として表現する。
//!
//! ただし実レンダリングは **VM 内 Lua**（`sakura_builder.lua` + `@pasta_sakura_script`）
//! に物理維持される（Lua 集約死守・design.md）。マーカー enum を経由して Rust 側で
//! 文字列を再構築するとバイト不変の逆算が崩れる懸念があるため（marker.rs の警告参照）、
//! ここでは **マーカー層を境界契約の型として用い**、実際のバイト生成は VM 内で
//! 既存どおり行う（バイト不変が硬性ゲート）。注入の差し替え可能性そのものが
//! 宿主非依存性を達成し、クレート配置を責務境界としない（design.md Invariants）。
//!
//! ## 注入 IF の選択（RN3）
//! 単一レンダラに対する過度な抽象化（trait object の動的ディスパッチや登録フラグの
//! 状態分岐）を避け、**Rust 関数ポインタ**を最小の差し替え点として採用する。
//! `RendererInjection` は「`(lua, talk_config)` を受けて `@pasta_sakura_script` を
//! VM へ登録する関数」を保持する薄いラッパで、既定は SHIORI さくらレンダラ
//! （[`default_sakura_renderer`]）= 既存 `crate::sakura_script::register` 経路。

use crate::loader::TalkConfig;
use crate::presentation::RenderBoundary;
use mlua::{Lua, Result as LuaResult, Table};

/// レンダラ注入が VM へ `@pasta_sakura_script` モジュールを登録する関数の型。
///
/// `lua` VM と解決済み [`TalkConfig`]（無ければ `None`）を受け取り、宿主固有の
/// レンダラモジュールテーブルを構築して返す。返したテーブルはコア側で
/// `package.loaded["@pasta_sakura_script"]` に登録される。
///
/// 既定（SHIORI 宿主）は [`default_sakura_renderer`]。
type RendererRegisterFn = fn(&Lua, Option<&TalkConfig>) -> LuaResult<Table>;

/// レンダラ注入シーム（アダプタ起点の `@pasta_sakura_script` 登録点）。
///
/// 設計の `RendererInjection`（design.md Service Interface）。コアはこの注入を
/// 通じてのみレンダラを登録し、さくら描画を **無条件 hard-code 起点としては保持
/// しない**（R3.3）。既定は SHIORI さくらレンダラで既存挙動・バイト不変（R3.5/R3.6）。
///
/// 概念上はマーカー列（[`crate::presentation::PresentationMarker`]）を宿主固有出力へ
/// 畳み込む [`RenderBoundary`] の具体注入であるが、実バイト生成は VM 内 Lua に維持する
/// （モジュールドキュメント参照）。
#[derive(Clone, Copy)]
pub struct RendererInjection {
    register: RendererRegisterFn,
}

impl RendererInjection {
    /// 任意のレンダラ登録関数から注入を構築する（アダプタ／テスト用）。
    ///
    /// これによりコアのさくら登録経路を差し替え可能にし、宿主非依存性を
    /// 「注入の差し替え可能性」で達成する（design.md Invariants・R3.4）。
    pub fn new(register: RendererRegisterFn) -> Self {
        Self { register }
    }

    /// 注入されたレンダラで `@pasta_sakura_script` モジュールテーブルを構築する。
    pub fn register_module(&self, lua: &Lua, config: Option<&TalkConfig>) -> LuaResult<Table> {
        (self.register)(lua, config)
    }
}

impl Default for RendererInjection {
    /// 既定 = SHIORI さくらレンダラ（既存挙動・バイト不変）。
    ///
    /// 注入が明示されない（既定）経路では従来どおり `crate::sakura_script::register`
    /// を呼ぶため、`talk_to_script` 出力はゴールデンとバイト不変（R3.5/R3.6）。
    fn default() -> Self {
        Self::new(default_sakura_renderer)
    }
}

impl std::fmt::Debug for RendererInjection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererInjection")
            .field("register", &"fn(&Lua, Option<&TalkConfig>) -> LuaResult<Table>")
            .finish()
    }
}

/// 既定の SHIORI さくらスクリプトレンダラ登録関数（= 既存挙動）。
///
/// `crate::sakura_script::register` に委譲し、`talk_to_script`/`break_lines` を備えた
/// `@pasta_sakura_script` モジュールテーブルを構築する。実レンダリングは VM 内 Lua に
/// 維持され、出力はデカップリング前とバイト不変（R3.5）。
pub fn default_sakura_renderer(lua: &Lua, config: Option<&TalkConfig>) -> LuaResult<Table> {
    crate::sakura_script::register(lua, config)
}

/// 既定 SHIORI さくらレンダラのマーカー境界契約上の位置づけを表す薄いマーカー型。
///
/// 実レンダリングは VM 内 Lua に維持するため、本型は [`RenderBoundary`] 契約と注入
/// シームの結線を **型レベルで明示**する目的のゼロサイズ証憑である（task 2.1 の
/// マーカー層を dead code にしないための結線・byte 不変は VM 側で担保）。
///
/// 最小集合のマーカーをデバッグ／観測用途で宿主非依存文字列へ畳み込めるが、
/// 出荷経路のバイト生成には用いない（バイト不変ゲート）。
#[derive(Debug, Default, Clone, Copy)]
pub struct SakuraRenderBoundary;

impl RenderBoundary for SakuraRenderBoundary {
    fn render_known(
        &mut self,
        marker: &crate::presentation::PresentationMarker,
    ) -> Option<String> {
        use crate::presentation::PresentationMarker as M;
        match marker {
            // Talk ラインの本文をそのまま（さくらスクリプトタグ変換は VM 内 Lua の責務）。
            M::Talk { text, .. } => Some(text.clone()),
            // 最小集合外（Wait/Choice/ActorSwitch/Extension）は注入レンダラ本体（VM 内）が
            // 扱うため、本観測用境界では既定（on_unknown）に委ねる。
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::TalkConfig;
    use mlua::Lua;

    /// (a) 既定注入経路が既存 `sakura_script::register` とバイト不変な
    ///     `talk_to_script` 出力を生むこと（R3.5/R3.6）。
    #[test]
    fn default_injection_renders_byte_identically_to_direct_register() {
        let config = TalkConfig::default();

        // 既存（直接 register）経路
        let lua_direct = Lua::new();
        let direct = crate::sakura_script::register(&lua_direct, Some(&config)).unwrap();
        let direct_fn: mlua::Function = direct.get("talk_to_script").unwrap();
        let direct_out: String = direct_fn
            .call((mlua::Value::Nil, "こんにちは。".to_string()))
            .unwrap();

        // 既定注入シーム経路
        let lua_inj = Lua::new();
        let injection = RendererInjection::default();
        let injected = injection.register_module(&lua_inj, Some(&config)).unwrap();
        let inj_fn: mlua::Function = injected.get("talk_to_script").unwrap();
        let inj_out: String = inj_fn
            .call((mlua::Value::Nil, "こんにちは。".to_string()))
            .unwrap();

        assert_eq!(
            direct_out.as_bytes(),
            inj_out.as_bytes(),
            "default injection must be byte-invariant vs direct register"
        );
    }

    /// (b) 代替レンダラを注入できること（デカップリングの実証・R3.4）。
    #[test]
    fn alternative_renderer_can_be_injected() {
        fn alt_renderer(lua: &Lua, _config: Option<&TalkConfig>) -> LuaResult<Table> {
            let t = lua.create_table()?;
            t.set("_VERSION", "alt-test")?;
            let f = lua.create_function(|_, talk: Option<String>| {
                Ok(format!("ALT:{}", talk.unwrap_or_default()))
            })?;
            t.set("talk_to_script", f)?;
            Ok(t)
        }

        let lua = Lua::new();
        let injection = RendererInjection::new(alt_renderer);
        let module = injection.register_module(&lua, None).unwrap();

        let version: String = module.get("_VERSION").unwrap();
        assert_eq!(version, "alt-test");

        let f: mlua::Function = module.get("talk_to_script").unwrap();
        let out: String = f.call("hi".to_string()).unwrap();
        assert_eq!(out, "ALT:hi");
    }

    /// マーカー境界結線（task 2.1）が dead code でないことの実証:
    /// `SakuraRenderBoundary` が `RenderBoundary` 契約を満たし、最小集合マーカーを
    /// 宿主非依存に畳み込める（出荷バイト生成には用いない観測用）。
    #[test]
    fn sakura_render_boundary_implements_marker_contract() {
        use crate::presentation::PresentationMarker as M;
        let mut boundary = SakuraRenderBoundary;
        let stream = [
            M::Talk {
                actor: Some("sakura".into()),
                text: "やあ".into(),
            },
            // 未対応マーカーは既定（無視）で破壊せず吸収（R2.6 契約）。
            M::Wait { ms: 100 },
        ];
        let out = boundary.render_stream(stream.iter());
        assert_eq!(out, "やあ");
    }
}
