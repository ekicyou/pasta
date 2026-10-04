# Brief: pasta-toml-logging-consistency

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 1（バグ修正）。着手するときは `/kiro-start pasta-toml-logging-consistency` で開始する。

## Problem

`pasta.toml` には、読み込まれるのに使われない設定キーがあり、ログの出力先とフィルタにも、マニュアルと食い違う挙動がある。作者が設定を書いても効かず、障害の調査に要るログが残らない。

| 項目 | 現象 |
| ---- | ---- |
| U26 `[lua] libs` | ロード時に読まれず、標準ライブラリ・mlua-stdlib の構成が変わらない |
| U32 `[logging] rotation_days` | 読まれず、ログファイルはローテーションされない |
| FFI 入口スレッドのログの破棄 | `request`・`unload`・`DllMain` の detach で出したログが、本番でファイルに残らない |
| 不正な `file_path` の扱い | `[logging] file_path` が不正だと、既定のログファイル `profile/pasta/logs/pasta.log` に書き続ける。マニュアル `reference/pasta-toml.md` は「ログファイルを作らない」と書く |

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（U26・U32）と `pasta-runtime-internals-doc` の吸収台帳付録 B（「FFI 入口スレッドのログの破棄」「ログフィルタの再読み込み不整合」）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **U26**:
  - `crates/pasta_lua/src/loader/mod.rs` 78 行付近と `crates/pasta_shiori/src/shiori.rs` 121 行付近は、常に `RuntimeConfig::new()` を作る。
  - `PastaConfig::lua()`（`loader/config/mod.rs` 173 行付近）と `From<LuaConfig>`（`runtime/runtime_config.rs` 321 行付近）に呼び出し元は無い。
  - マニュアル `book/src/reference/pasta-toml.md` 430–435 行付近は `[lua]` を載せ、既定値を示すだけで、`@env` は有効にできないと書く。
  - `lua/modules/mlua-stdlib.md` 116 行付近は「ゴーストから有効にする方法は無い」と書く（吸収台帳 X18 の決定）。
- **棚卸の即時修正で見つかった観察（未調査）**: `RuntimeConfig::from_libs(["std_string"])` だけで VM を作ると、`with_config` が「nil→table の変換エラー」で失敗した（randomseed のテストを書く途中で発見）。U26 を撤去しても組み込み向けに残す `from_libs` の不具合の可能性があるため、要件フェーズで原因を確かめ、範囲に入れるかを決める。
- **U32**: `loader/config/sections.rs` 24–25 行付近がフィールドを宣言し、`logging/logger.rs` 65 行付近は `Rotation::NEVER` 固定。
  - サンプルゴーストの `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` 31 行付近が書いている。
  - 内部設計 `book/src/internals/logging-encoding.md` 39・131 行付近が触れる。利用者章は載せていない。
- **ログの破棄**:
  - `crates/pasta_shiori/src/windows.rs` の `unload`（216–247 行付近）と `request`（251–293 行付近）、`actor/marshaling.rs` の観測ログは、`LoadDirGuard` を張らない。
  - `logging/registry.rs` 121–125 行付近は、ガードの無いスレッドのログを黙って捨てる。`load` だけがガードを張り直す（`windows.rs` 192 行付近）。
  - `unload` の「teardown の異常」の warn は、`PastaShiori::drop` がロガーを登録解除した後（`shiori.rs` 60 行付近）に出るため、ガードを張るだけでは残らない。
- **不正な `file_path`**: SHIORI 経由では、Stage 1 の既定ロガー（`shiori.rs` 100–103 行付近）が登録されたまま残り、既定ファイルへ書き続ける。マニュアル `reference/pasta-toml.md` 423 行付近の記述と食い違う。
- **棚卸の即時修正で修正済み（関連）**:
  - `[logging]` の無い再読み込みで前のフィルタが残る問題。
  - 不正な `file_path` で `level`・`filter` まで効かなくなる問題。
  - `unload` を経ないプロセス終了で 5 秒止まる問題（`DllMain` の detach）。

## Desired Outcome

- `pasta.toml` のすべてのキーが、書けば効くか、マニュアルにもサンプルにも無いかのどちらかになっている（読まれて捨てられるキーが無い）。
- `request`・`unload`・プロセス終了時の detach で出したログが、ゴーストのログファイルに残る（ロガーが生きている範囲で）。
- 不正な `file_path` のときの挙動がマニュアルと一致している。
- マニュアル（`reference/pasta-toml.md`・`internals/logging-encoding.md`・`internals/shiori.md`）が新しい挙動を書き、スキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **U26**: 実装するか撤去するか。推奨は撤去。
  - 実装すると、ゴーストが `env`・`std_all_unsafe`・`std_ffi`・`std_debug` を有効にできるようになる。セキュリティ上の変更であり、マニュアル 2 ページの決定（X18）と矛盾する。
  - 撤去の範囲: `PastaConfig::lua()`・`LuaConfig`・`From<LuaConfig>`・再エクスポート（`lib.rs` 58 行付近・`loader/mod.rs` 36 行付近）。Rust の組み込み向けの `RuntimeConfig::from_libs`・`default_libs` は残す。
  - crates.io に出している公開 API の削除になる（0.x のため可。リリース時にマイナーを上げる）。
- **U32**: 実装するか撤去するか。推奨は撤去。
  - `tracing_appender` の `DAILY`＋`max_log_files` で実装すると、ファイル名が `pasta.log.YYYY-MM-DD` に変わり、`file_path` の意味が崩れる。
  - serde は未知のキーを無視するため、撤去しても既存のゴーストは読み込める。
- **入口スレッドのログ**: 次の 2 案から選ぶ。
  - (a) ロード時のディレクトリを `windows.rs` の static に持ち、`request`・`unload` でガードを張る。
  - (b) `RoutingWriter` が、ガードの無いスレッドのログを登録済みの唯一のロガーへ流す。
  - `unload` の後の warn をどう残すか（登録解除の前に出す、など）。
- **不正な `file_path`**: 推奨は「既定ファイルへのフォールバックを残し、warn を出し、マニュアル 423 行付近とスキルの写しを直す」。

## Scope

- **In**:
  - U26・U32 の決定と実装（撤去ならコード・サンプル・マニュアル・テストからの削除）
  - FFI 入口スレッドのログの保存
  - 不正な `file_path` の挙動の確定とマニュアルの一致
  - 修正を固定するテスト
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - ログの書式・レベルの既定値の変更
  - デバッグバックエンドのログ

## Boundary Candidates

- 設定（`pasta_lua` の `loader/config/`・`runtime/runtime_config.rs`・`lib.rs` の再エクスポート）
- ロガー（`pasta_lua` の `logging/`・`loader/mod.rs` の `create_and_register_logger`）
- FFI 入口（`pasta_shiori` の `windows.rs`・`shiori.rs`）
- サンプルゴーストの `pasta.toml`・マニュアル・生成スキル

## Out of Boundary

- `pasta_lua` の Lua ランタイム（`pasta_scripts/`）
- ローダのファイル探索・キャッシュ（即時修正で修正済み）

## Upstream / Downstream

- **Upstream**: `logger-configuration`・`lua-logging`・`lua-stdlib-config`（完了。元の設計）
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `load-error-logging`・`lua-require-robustness`（完了。起動失敗の記録経路）、`release-workflow`（公開 API の削除はリリース時にマイナーを上げる）

## Constraints

- pasta.dll は静的 CRT リンク・単一 DLL。依存を増やさない。
- マニュアルが利用者向け設定の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する（`reference/pasta-toml.md` はスキルへ写される）。
- 並走条件（Wave 1）: 編集するソースは上の境界候補に限る。`pasta_lua` の `pasta_scripts/`・`code_gen/`・`search/`・`debug/` は触らない。
