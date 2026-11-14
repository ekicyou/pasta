# 設計レビュー: date-var-calendar

- 対象: `.kiro/specs/completed/date-var-calendar/design.md`（requirements.md・research.md・steering と照合）
- 実施日: 2026-11-15
- 方式: kiro-validate-design（非対話。Analysis → Critical Issues → Strengths → GO/NO-GO）

## 設計レビューの要約

規則を `pasta_core::calendar` に置いて pasta.dll（ローダの段 1.6）と `pasta_check` の核の両方から呼ぶ形は、今の `SceneAliasTable`・`ActorFont` の層の分け方にそのまま乗っており、要件 1〜10 の全部の受け入れ基準が部品と検査に結び付いている。コードと照らした主な主張（段 1.5／1.7 の位置、`LoaderError` の形、`RuntimeConfig` の 4 つの構築子、`transfer_date_to_var` の `＄節句` の行、`select_dictionary_files` を読むだけの 3 つの殻、`skip_reasons` の対象、Lua のテストの実行器の登録の並び、`tutorial_stages_test.rs` の段の数）はどれも事実どおりだった。実装へ進めるが、CI の発火条件の取り残しなど 3 点を設計ディスカッションで直したい。

## コードと照らして確かめたこと

| 設計の主張 | 確かめた場所 | 結果 |
|------------|--------------|------|
| 段 1.5（ロガー）の後・段 1.7（`LoaderContext`）の前に段 1.6 を置ける。段 6 で `with_scene_aliases` と同じく組み立てる | `crates/pasta_lua/src/loader/mod.rs` の `load_with_config` | 一致 |
| `LoaderError::Io(PathBuf, io::Error)` があり、`Config` は `toml::de::Error` を持つので新しい種類が要る | `crates/pasta_lua/src/loader/error.rs` | 一致 |
| `RuntimeConfig` の構築子は 4 つ（`new`・`full`・`minimal`・`from_libs`）、`with_scene_aliases` の型がある | `crates/pasta_lua/src/runtime/runtime_config.rs` | 一致 |
| `@pasta_log`・`@toml` は `with_config_and_source_map` で常に登録する | `crates/pasta_lua/src/runtime/mod.rs`・`module_registry.rs` | 一致（登録の実体は `module_registry.rs` の `set_loaded_module`） |
| `transfer_date_to_var` は `CLOCK.date(self.req)` を使い、`＄節句` は `SEKKU_JA[...] or "なし"` | `pasta_scripts/pasta/shiori/act.lua` 155〜195 行 | 一致 |
| 呼び出しごとの `pcall(require, "@pasta_search")` の前例 | `pasta_scripts/pasta/act.lua` 399 行・`actor.lua` 114 行 | 一致（前例は `shiori/act.lua` でなく `pasta/act.lua`。文言の小さな違い） |
| ランダムトーク・時報の前にも自動転記がある | `shiori/event/virtual_dispatcher.lua` 182 行・`event/init.lua` 106 行 | 一致 |
| 3 つの殻は `select_dictionary_files` の返すパスだけを読む | `pasta_check/src/ghost_files.rs`（CLI・recipe の `check.rs` が共用）、`pasta_lsp/src/transport.rs`（WASM）、`editors/vscode/src/ghostValidator.ts` | 一致。VSCode は `ignoreBOM: true` で BOM を残して渡すので `parse` が外す設計と合う |
| `CalendarInvalid` を `skip_reasons` に入れなければ辞書の検査は省かれない | `pasta_check/src/validate/report.rs` の `skip_reasons` | 一致 |
| Lua のテストの実行器は公開の `register` で登録する | `crates/pasta_lua/tests/lua_unittest_runner.rs` | 一致 |
| 段階の検査は 1〜13 段・ファイルの無い段 13・検証イベントは 1〜12、`ghost/master` を丸ごと写す | `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs`・`tests/common/mod.rs` | 一致 |
| 春分・秋分の式が 2022〜2027 年の内閣府の日と合う | 手計算（例 2022 年: 春分 21 日・秋分 23 日、2024 年: 20 日・22 日） | 一致 |
| 振替休日の「前の日から遡る」規則が 2023-01-02・2025-05-06・2026-05-06 などを出す | 規則の手計算 | 一致 |

## 重大な課題（3 件）

🔴 **重大な課題 1**: マニュアルの CI の発火条件に `calendar.csv` が無い
**Concern**: 設計は `tutorial-check.mjs` に「入門の章の ```` ```csv ```` ブロック ↔ hello-pasta の `calendar.csv`」の照合を足すが、それを走らせる `.github/workflows/manual.yml` の `push`・`pull_request` の `paths` は `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/**` だけを見ている。`calendar.csv` だけを直した PR・push ではマニュアルの CI が起動せず、照合が働かない。同じファイルの注記「入門の段の章 12 枚（`01-boot.md`〜`12-lua.md`）」も 13 枚（`13-calendar.md` まで）に変わる。Modified Files に `manual.yml` が無い。
**Impact**: Req 10.6 の「すべての段階の検査を通す」が、表だけを変えたときに CI で守られない。章の数の取り残しは過去の棚卸（2026-10-17）でも直した種類の不具合。
**Suggestion**: Modified Files に `.github/workflows/manual.yml` を足し、両方の `paths` に `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/calendar.csv`（または `ghost/master/*.csv`）を加え、注記の章の数と範囲を直す。
**Traceability**: Req 10.6・9.10
**Evidence**: design.md「File Structure Plan > Modified Files」（`book/tools/tutorial-check.mjs` の行）、「SampleGhostStage13」の `tutorial-check.mjs` の説明

🔴 **重大な課題 2**: ローダと `pasta_check`・配布物とで、シンボリックリンクの `calendar.csv` の見え方が食い違う
**Concern**: CalendarLoader は `read_dir` で「ファイルかフォルダかで絞らない」で集めて `fs::read` で読むので、シンボリックリンクの `calendar.csv` もたどって読む。一方 `pasta_check` の殻の一覧（`ghost_files::walk`。CLI と pasta-recipe が共用）はシンボリックリンクを一覧から外し、`pasta_check release` のコピー（`copy.rs`）もシンボリックリンクを写さない。その表は pasta.dll には効くのに `pasta_check validate` は何も報告せず（Req 6.3 の扱い）、`.nar` にも入らない。
**Impact**: Req 6.4「検査とランタイムで判定が食い違わない」の例外になり、手元では行事のトークが出るのに配布したゴーストでは黙って `なし` になる（Req 5 の目的「黙って出ないまま気づかない」に反する）。起こるのは稀だが、設計の「判定は同じコード」の約束の穴。
**Suggestion**: ローダの段 1.6 でも `file_type().is_symlink()` の項目を数えない（殻の一覧と同じ規則）と決め、CalendarLoader の Responsibilities に書く。Unix の CI で `validate_parity_test.rs` の表の事例にシンボリックリンクを 1 つ足す（Windows では作れないので `cfg(unix)`）。
**Traceability**: Req 6.4・5.8・4.1
**Evidence**: design.md「CalendarLoader > Responsibilities & Constraints」1 つ目、「CalendarCheck > Responsibilities & Constraints」2 つ目

🔴 **重大な課題 3**: 春分・秋分の式は、遠い年では月の日の範囲を外れて祝日が出なくなる
**Concern**: 式 `floor(23.2488 + 0.242194·t − floor(t/4))` は 1 年に約 0.0078 日ずれるため、およそ 990 年より前では秋分の日が 9 月 31 日以上に、4520 年ごろより後では春分の日が 0 日以下になり、`base` の比較に当たる日が無くなる（春分はおよそ 550〜4520 年、秋分はおよそ 990〜4830 年の範囲でだけ出る）。要件の確定した論点 3 と Req 2.3 は「範囲で区切らずに当てはめる。出さなくすることはしない」で、設計の単体テストの「1970 年より前と 2100 年より後でも返す」はどの年で確かめるかを決めていない。負の値を `u32` に直す書き方次第では panic の余地もある。
**Impact**: SSP の時計で届く年（1601 年以降）と実用の年では起こらないが、`X-Pasta-Time` やテストの年の選び方次第で、要件の文言と実装が食い違ったまま気づかれない。
**Suggestion**: 次のどちらかに決める。(a) 式の日を `i64` で計算し、月の日数の範囲（3 月は 1〜31、9 月は 1〜30）に収める（範囲の外の年でも必ず 1 日を春分・秋分にする）。(b) 範囲を外れる年では出ないことを許し、マニュアルと要件の注記に書く。どちらでも、テストの年を明記する（例 1600 年・2500 年）。推奨は (a)（Req 2.3 の文言のまま守れ、コードは 1 行の `clamp`）。
**Traceability**: Req 2.3・9.4、確定した論点 3
**Evidence**: design.md「HolidayRules > Responsibilities & Constraints」4 つ目、「Testing Strategy > Unit Tests」の `holiday.rs`

## 軽微な指摘（議題にしなくてよい）

- `pcall(require)` の前例は `pasta/shiori/act.lua` でなく `pasta/act.lua`（399 行）。設計の「Allowed Dependencies」の文言だけの違い。
- `@pasta_calendar` の登録の実体は `runtime/module_registry.rs` の `set_loaded_module` の並び（`register_log_module`・`register_toml_module`）に置くのが今の形。Modified Files に `module_registry.rs` を足すか、`runtime/mod.rs` から呼ぶだけと明記するとよい。

## 設計の強み

1. **判定を 1 か所に置き、一致テストで閉じている**: `CalendarTable::parse` と `is_calendar_file_name` を `pasta_core` に置き、ローダと `pasta_check` の核が同じコードを呼ぶ。殻 3 つが `select_dictionary_files` の返すパスを読むだけであること、`CalendarInvalid` を `skip_reasons` に入れなければ辞書の検査が続くことは、コードで確かめたとおりで、殻のコードを変えずに Req 6 を満たせる。`load_failure_matches_errors` に行番号の集合の比較まで足す計画も、Req 6.4 を機械で守る。
2. **祝日の規則が自己完結し、ネットワークなしで全日を答え合わせする**: 規則を Rust の純粋な関数（通日の整数算術・依存なし）にしたので、内閣府の CSV との全日の照合が `pasta_core` の結合テストだけで閉じ、データとテストを crates.io のパッケージから外す手当て（テストのファイルも外す理由まで）が書かれている。振替休日を「遡って基本の祝日の連なりに日曜日があるか」で数える形は、2022〜2027 年の `休日` を手計算で全部再現した。

## 最終判断

**判断: GO**

**理由**: 層の分け方・エラーの扱い・依存の向きは今のアーキテクチャと揃っており、要件の全部の受け入れ基準に実装と検査の道筋がある。3 つの課題はどれも局所的（CI の設定 1 か所・ローダの 1 条件・式の 1 行と注記）で、設計の骨組みを変えない。

**次の手順**:
- 設計ディスカッション（`/kiro-design-discussion date-var-calendar`）で、上の重大な課題 1〜3 と、design.md の Open Questions 1〜3（出典の文書のライセンスの版の書き方・13 段目で Call する場面・名前が大文字・小文字だけ違う表の扱い）を一緒に決める。
- 決まった分を design.md に入れてから `/kiro-spec-tasks date-var-calendar` でタスクを作る。
