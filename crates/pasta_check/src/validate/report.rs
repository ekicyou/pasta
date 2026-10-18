//! 報告の種類・分類・文言・並び順・出力の形（7.1〜7.8・2.9・6.7・6.8）。
//!
//! 分類（誤り／注意）は種類ごとに固定で、オプションや状況で変えない（7.7）。

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    /// 1 行の形に出す日本語の分類（7.8）
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "誤り",
            Severity::Warning => "注意",
        }
    }
}

/// 報告の種類。並び順（7.3）で code を比べるときは宣言の順を使う
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    ConfigMissing,
    ConfigSyntax,
    ConfigLoaderType,
    ConfigSceneAlias,
    ConfigActorFont,
    ConfigPatternInvalid,
    PatternIgnored,
    PatternNotPasta,
    NoDictionaryFiles,
    FolderMatchesPattern,
    InitFileName,
    FileNotUtf8,
    FileUnreadable,
    Syntax,
    TranspileArgAssign,
    TranspilePropertyInExpr,
    TranspileContinuationWithoutActor,
    ModulePastaCollision,
    ModuleLuaCollision,
    ModuleLuaShadowed,
    GlobalSceneCollision,
    LocalSceneCollision,
    SceneShadowedByBuiltin,
    CallTargetMissing,
    ChoiceTargetMissing,
}

impl FindingKind {
    pub const ALL: [FindingKind; 25] = {
        use FindingKind::*;
        [
            ConfigMissing,
            ConfigSyntax,
            ConfigLoaderType,
            ConfigSceneAlias,
            ConfigActorFont,
            ConfigPatternInvalid,
            PatternIgnored,
            PatternNotPasta,
            NoDictionaryFiles,
            FolderMatchesPattern,
            InitFileName,
            FileNotUtf8,
            FileUnreadable,
            Syntax,
            TranspileArgAssign,
            TranspilePropertyInExpr,
            TranspileContinuationWithoutActor,
            ModulePastaCollision,
            ModuleLuaCollision,
            ModuleLuaShadowed,
            GlobalSceneCollision,
            LocalSceneCollision,
            SceneShadowedByBuiltin,
            CallTargetMissing,
            ChoiceTargetMissing,
        ]
    };

    /// 固定の対応。オプションや状況で変えない（7.7）
    pub fn severity(self) -> Severity {
        use FindingKind::*;
        match self {
            PatternIgnored
            | PatternNotPasta
            | NoDictionaryFiles
            | ModuleLuaShadowed
            | GlobalSceneCollision
            | LocalSceneCollision
            | SceneShadowedByBuiltin
            | CallTargetMissing
            | ChoiceTargetMissing => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// 診断の code（例 "syntax"）。エディタの重複の判定に使う。serde の形と同じ
    pub fn code(self) -> &'static str {
        use FindingKind::*;
        match self {
            ConfigMissing => "config-missing",
            ConfigSyntax => "config-syntax",
            ConfigLoaderType => "config-loader-type",
            ConfigSceneAlias => "config-scene-alias",
            ConfigActorFont => "config-actor-font",
            ConfigPatternInvalid => "config-pattern-invalid",
            PatternIgnored => "pattern-ignored",
            PatternNotPasta => "pattern-not-pasta",
            NoDictionaryFiles => "no-dictionary-files",
            FolderMatchesPattern => "folder-matches-pattern",
            InitFileName => "init-file-name",
            FileNotUtf8 => "file-not-utf8",
            FileUnreadable => "file-unreadable",
            Syntax => "syntax",
            TranspileArgAssign => "transpile-arg-assign",
            TranspilePropertyInExpr => "transpile-property-in-expr",
            TranspileContinuationWithoutActor => "transpile-continuation-without-actor",
            ModulePastaCollision => "module-pasta-collision",
            ModuleLuaCollision => "module-lua-collision",
            ModuleLuaShadowed => "module-lua-shadowed",
            GlobalSceneCollision => "global-scene-collision",
            LocalSceneCollision => "local-scene-collision",
            SceneShadowedByBuiltin => "scene-shadowed-by-builtin",
            CallTargetMissing => "call-target-missing",
            ChoiceTargetMissing => "choice-target-missing",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub code: FindingKind,
    pub severity: Severity,
    /// ghost/master からの相対パス（例 "dic/02-talk.pasta"、"pasta.toml"）
    pub path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    /// 日本語の理由（CLI とエディタで同じ文字列）
    pub message: String,
}

impl Finding {
    /// 分類は種類から決める（7.7）。理由に改行があれば各行を空白 1 つでつなぎ、1 件 1 行にする（7.1）
    pub fn new(
        code: FindingKind,
        path: impl Into<String>,
        line: Option<u32>,
        column: Option<u32>,
        message: impl Into<String>,
    ) -> Self {
        Finding {
            code,
            severity: code.severity(),
            path: path.into(),
            line,
            column,
            message: one_line(message.into()),
        }
    }

    /// `ghost/master/{path}:{line}:{column}: {誤り|注意}: {message}`。無い位置は省く（7.1・7.2）
    pub fn line_text(&self) -> String {
        let mut s = ghost_path(&self.path);
        if let Some(line) = self.line {
            s += &format!(":{line}");
            if let Some(col) = self.column {
                s += &format!(":{col}");
            }
        }
        format!("{s}: {}: {}", self.severity.label(), self.message)
    }
}

/// 飛び先の検査を省いた理由。宣言の順が案内の行の順
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkipReason {
    Syntax,
    UnreadableFile,
    Config,
}

impl SkipReason {
    /// 案内の行の言い方
    pub fn label(self) -> &'static str {
        match self {
            SkipReason::Syntax => "構文エラー",
            SkipReason::UnreadableFile => "読めない辞書ファイル",
            SkipReason::Config => "pasta.toml の誤り",
        }
    }
}

/// findings から省く理由を決める（昇順・重複なし）。空なら飛び先を検査する（1.1〜1.3・1.6）。
/// 辞書の中身・集め方・別名表が分からないとき、検査した辞書のシーンの集まりが pasta.dll と同じとは言えない
pub(crate) fn skip_reasons(findings: &[Finding]) -> Vec<SkipReason> {
    use FindingKind::*;
    let mut reasons: Vec<SkipReason> = findings
        .iter()
        .filter_map(|f| match f.code {
            Syntax => Some(SkipReason::Syntax),
            // pasta.toml を読めないのは設定の誤り。ほかのパス（渡し忘れを含む）は読めない辞書ファイル
            FileUnreadable if f.path == super::input::PASTA_TOML => Some(SkipReason::Config),
            FileNotUtf8 | FileUnreadable => Some(SkipReason::UnreadableFile),
            ConfigSyntax | ConfigLoaderType | ConfigPatternInvalid | ConfigSceneAlias => {
                Some(SkipReason::Config)
            }
            _ => None,
        })
        .collect();
    reasons.sort();
    reasons.dedup();
    reasons
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub dictionary_files: usize,
    pub errors: usize,
    pub warnings: usize,
    pub unchecked_dynamic_calls: usize,
    /// 空なら飛び先を検査した
    pub skip_reasons: Vec<SkipReason>,
}

impl Stats {
    pub fn references_skipped(&self) -> bool {
        !self.skip_reasons.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub stats: Stats,
}

impl Report {
    /// findings を 7.3 の順に並べ、誤りと注意の数を findings から数え直す
    pub fn new(mut findings: Vec<Finding>, mut stats: Stats) -> Self {
        // Option の順で、行の無いものが先になる
        findings.sort_by(|a, b| {
            (&a.path, a.line, a.column, a.code, &a.message)
                .cmp(&(&b.path, b.line, b.column, b.code, &b.message))
        });
        stats.errors = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        stats.warnings = findings.len() - stats.errors;
        Report { findings, stats }
    }

    pub fn has_errors(&self) -> bool {
        self.stats.errors > 0
    }

    /// 問題の行（7.3 の順）、案内の行、まとめの行。CLI はこれを順に標準出力へ出す
    pub fn lines(&self) -> Vec<String> {
        let s = &self.stats;
        let mut lines: Vec<String> = self.findings.iter().map(Finding::line_text).collect();
        let mut summary = format!(
            "検査した辞書ファイル {} 個: 誤り {} 件、注意 {} 件",
            s.dictionary_files, s.errors, s.warnings
        );
        if s.references_skipped() {
            lines.push(msg::references_skipped(&s.skip_reasons));
        } else {
            summary += &format!(
                "、飛び先を検査しなかった Call {} 件",
                s.unchecked_dynamic_calls
            );
        }
        lines.push(summary);
        lines
    }
}

/// toml・pest などの説明は改行を含むことがある
fn one_line(message: String) -> String {
    if !message.contains(['\n', '\r']) {
        return message;
    }
    message
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn ghost_path(path: &str) -> String {
    format!("ghost/master/{path}")
}

fn ghost_paths(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|p| ghost_path(p))
        .collect::<Vec<_>>()
        .join("、")
}

/// 理由の文言（design.md の Report の文言の表）。パスは ghost/master からの相対パスで渡す
pub mod msg {
    use super::{SkipReason, ghost_path, ghost_paths};

    /// 案内の行。理由を表の順に「と」でつなぐ（reasons は空でないこと）。
    /// 半角で始まる言い方（pasta.toml）の前には空白を置く
    pub fn references_skipped(reasons: &[SkipReason]) -> String {
        let mut s = String::new();
        for (i, label) in reasons.iter().map(|r| r.label()).enumerate() {
            if i > 0 {
                s += "と";
                if label.starts_with(|c: char| c.is_ascii()) {
                    s += " ";
                }
            }
            s += label;
        }
        format!("{s}があるため、飛び先の検査をしていません。")
    }

    pub fn config_missing() -> String {
        "pasta.toml がありません。pasta.dll はゴーストを読み込めません（辞書は既定の dic/**/*.pasta で集めました）".into()
    }
    pub fn config_syntax(toml_desc: &str) -> String {
        format!("pasta.toml を TOML として読めません: {toml_desc}")
    }
    pub fn config_loader_type(key: &str, toml_desc: &str) -> String {
        format!("[loader] の {key} の型が違います: {toml_desc}")
    }
    /// `loader = 1` のように [loader] を表でなく書いたとき（config-loader-type）
    pub fn config_loader_not_table(toml_desc: &str) -> String {
        format!("[loader] が表ではありません: {toml_desc}")
    }
    /// 置き換え先は pasta.dll と同じく名前の昇順で並べる
    pub fn config_scene_alias_duplicate(alias: &str, targets: &[String]) -> String {
        format!(
            "[scene.alias] の別名「{alias}」が {} に重複しています",
            targets.join("・")
        )
    }
    /// 空の名前（置き換え先が空なら `target` は ""）
    pub fn config_scene_alias_empty(target: &str) -> String {
        if target.is_empty() {
            "[scene.alias] に置き換え先の名前が空のものがあります".into()
        } else {
            format!("[scene.alias] の {target} に空の別名があります")
        }
    }
    pub fn config_scene_alias_chain(name: &str) -> String {
        format!(
            "[scene.alias] の「{name}」は置き換え先と別名の両方に書かれています（別名を別名で置き換えることはできません）"
        )
    }
    /// `[scene]`・`[scene.alias]` が表でない、値が文字列の配列でない
    pub fn config_scene_alias_type(toml_desc: &str) -> String {
        format!("[scene.alias] の型が違います: {toml_desc}")
    }
    pub fn config_actor_font_unknown(actor: &str, attribute: &str) -> String {
        format!(
            "[actor.\"{actor}\"] の font に知らない属性「{attribute}」があります（書ける属性: {}）",
            pasta_core::FONT_ATTRIBUTES.join("・")
        )
    }
    /// 型違い・空。`written` は書かれていたもの（「配列」「空の文字列」など）
    pub fn config_actor_font_value(actor: &str, attribute: &str, written: &str) -> String {
        format!(
            "[actor.\"{actor}\"] の font の {attribute} は文字列・整数・真偽値のどれかで、空にできません: {written}が書かれています"
        )
    }
    pub fn config_actor_font_breaks_tag(actor: &str, attribute: &str) -> String {
        format!(
            "[actor.\"{actor}\"] の font の {attribute} に「]」か「\\」が含まれています（さくらスクリプトのタグが壊れます）"
        )
    }
    pub fn config_actor_font_not_table(actor: &str) -> String {
        format!(
            "[actor.\"{actor}\"] の font は表で書きます（例: font = {{ color = \"90,110,140\" }}）"
        )
    }
    pub fn config_pattern_invalid(pattern: &str, desc: &str) -> String {
        format!("pasta_patterns の「{pattern}」はパターンとして読めません: {desc}")
    }
    pub fn pattern_ignored(pattern: &str) -> String {
        format!("pasta_patterns の「{pattern}」は .. などを含むため使われません")
    }
    pub fn pattern_not_pasta(pattern: &str) -> String {
        format!(
            "pasta_patterns の「{pattern}」は .pasta で終わらないため、.lua の辞書モジュールを探すのに使われません"
        )
    }
    pub fn no_dictionary_files() -> String {
        "pasta_patterns に一致する辞書ファイルが 1 つもありません".into()
    }
    pub fn folder_matches_pattern() -> String {
        "辞書のパターンに一致するフォルダです。pasta.dll はこのフォルダを辞書として読めず、ゴーストを読み込めません".into()
    }
    pub fn init_file_name(file_name: &str) -> String {
        format!("ファイル名 {file_name} は使えません。pasta.dll はゴーストを読み込めません")
    }
    pub fn file_not_utf8() -> String {
        "UTF-8 として読めません".into()
    }
    pub fn file_unreadable(reason: &str) -> String {
        format!("ファイルを読めません: {reason}")
    }
    pub fn syntax(pest_desc: &str) -> String {
        format!("構文エラー: {pest_desc}")
    }
    pub fn transpile_arg_assign() -> String {
        "シーンの引数（＄０ など）には代入できません".into()
    }
    pub fn transpile_property_in_expr() -> String {
        "プロパティ参照（＄％名前）は式の中で使えません。先に変数へ代入してください".into()
    }
    pub fn transpile_continuation_without_actor() -> String {
        "アクターの無い継続行です。前の行にアクター行が必要です".into()
    }
    /// 重なったほかのファイルが 2 つ以上なら「、」で並べる（モジュール名の 3 つの文言で同じ）
    pub fn module_pasta_collision(module: &str, others: &[&str]) -> String {
        format!(
            "モジュール名 {module} が {} と同じです。どちらかの辞書の内容が失われます",
            ghost_paths(others)
        )
    }
    pub fn module_lua_collision(module: &str, others: &[&str]) -> String {
        format!(
            "モジュール名 {module} が {} と同じです。どちらかの .lua が読み込まれません",
            ghost_paths(others)
        )
    }
    pub fn module_lua_shadowed(module: &str, pasta_paths: &[&str]) -> String {
        format!(
            "モジュール名 {module} が {} と同じため、この .lua は読み込まれません",
            ghost_paths(pasta_paths)
        )
    }
    /// global-scene-collision・local-scene-collision の両方で使う
    pub fn scene_collision(
        written: &str,
        match_name: &str,
        other_name: &str,
        other_path: &str,
        other_line: u32,
    ) -> String {
        format!(
            "シーン「{written}」は照合用の名前「{match_name}」が「{other_name}」（{}:{other_line}）と同じになり、同じ名前のシーンとして選ばれます",
            ghost_path(other_path)
        )
    }
    pub fn scene_shadowed_by_builtin(name: &str) -> String {
        format!("シーン「{name}」は組み込みの飛び先と同じ名前のため、＞{name} では呼ばれません")
    }
    pub fn call_target_missing(key: &str) -> String {
        format!(
            "Call の飛び先「{key}」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
        )
    }
    pub fn choice_target_missing(target: &str) -> String {
        format!(
            "選択肢のジャンプ先「{target}」が辞書のシーンにありません（Lua で定義・振り分けていれば問題ありません）"
        )
    }
}
