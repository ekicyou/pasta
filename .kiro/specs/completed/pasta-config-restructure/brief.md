# Brief: pasta-config-restructure

## Problem
ゴースト作者（pasta DSL でゴーストを書く利用者）が、最初に書く `pasta.toml` の記述量が多すぎて負担になっている。
- `[package]`（name/version/edition）が必須のように見えるが、Rust では一切消費されておらず、伺かゴーストでは install.txt 等で代替できる
- SHIORI（pasta.dll）として動作に必要な設定と、将来のノベルゲームエンジン用途で必要になる設定が混在している
- 多くのフィールドはデフォルト値が事実上一意に決まるのに、必須に見えてしまい「本当に要るのか？」という不安を与える

## Current State
`pasta.toml` の解釈は [crates/pasta_lua/src/loader/config.rs](../../../crates/pasta_lua/src/loader/config.rs) に集約されている。
- Rust が明示的にパースするのは `[loader]` のみ。`[logging]`/`[persistence]`/`[lua]`/`[talk]`/`[debug]` は要求時に `custom_fields` から遅延デシリアライズされ、**すべて `Default` 実装あり＝省略可**
- `[ghost]`（talk_interval_min/max, hour_margin, spot_newlines）/`[actor]`（spot）は Rust ではパースされず、Lua 側が `pasta.config.get(...)` / `STORE.actors = CONFIG.actor` で動的参照
- `[package]` はどのコードパスからも消費されていない（実質デコラティブ／将来用）
- `PastaConfig::load()` は **pasta.toml が存在しないと `ConfigNotFound` エラー**になる
- 各セクションの「SHIORI用か将来用か」「必須か任意か」がドキュメント上で分類されておらず、サンプル（hello-pasta）がフル記述のため作者がコピーして冗長化する

サンプル: [crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml](../../../crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml)

## Desired Outcome
- ゴースト作者は「最小限の必須セクションだけ書いた簡易な `pasta.toml`」でゴーストを起動できる
- 各セクション・各フィールドが「**SHIORIコア必須 / SHIORI任意（デフォルト有） / 将来エンジン予約**」のどれに属するか、仕様・ドキュメント上で明確に分類されている
- `[package]` は将来エンジン用途の予約概念として位置づけられ、SHIORI 用途では不要であることが明示される（デフォルトテンプレート・サンプルから除去）
- 既存のフル記述 `pasta.toml`（hello-pasta 等）は **完全後方互換** で従来どおり動作する
- 「最小テンプレート」と「フルリファレンステンプレート」の2層が提供され、デフォルト値は単一の真実の源（SSOT）から導かれる

## Approach
**アプローチ A（ドキュメント/スキーマ再整理）を核に、アプローチ C（テンプレートのティア化／SSOT）の思想を取り込む。**
- 既存セクション名は一切変更しない（完全後方互換のため、物理的再グルーピング案 B は互換シムのコスト・リスクが見合わず却下）
- 全セクション・全フィールドを「SHIORIコア必須 / SHIORI任意 / 将来エンジン予約」の3分類でカテゴリ化し、仕様として確立する
- 「最小限の必須セクション」を明示的に定義する（＝空ファイル/ファイル不在は許さず、明示的な必須を残す方針）
  - **最有力候補: 少なくとも1つの `[actor]` 定義を必須とする。** `spot`（サーフェス位置）はゴースト固有でデフォルト化が困難であり、「喋る主体」の定義が無いと SHIORI として意味をなさないため
  - 対照的に `[ghost]`（talk_interval_min/max, hour_margin, spot_newlines）は無難なデフォルトが効くため任意化する
  - 最小必須セクションの最終確定は要件フェーズで行う（actor 必須 / ghost デフォルト化の妥当性を検証）
- 最小テンプレートとフルリファレンステンプレートの2層を提供し、デフォルト値を SSOT 化する

## Scope
- **In**:
  - `pasta.toml` 各セクション・各フィールドの「SHIORI必須/任意/将来予約」3分類モデルの確立
  - 「最小限の必須セクション」の定義（actor 必須が最有力）と、それ以外の任意化・デフォルト化の明確化
  - `[package]` を将来エンジン用予約として位置づけ、デフォルトテンプレ・サンプルから除去
  - 最小テンプレート／フルリファレンステンプレートの2層提供（デフォルト値 SSOT 化）
  - 既存フル記述の完全後方互換の保証（回帰テスト含む）
  - 利用者向けドキュメント（マニュアル／README）への分類の反映
- **Out**:
  - セクション名のリネームや名前空間による物理的再グルーピング（アプローチ B）
  - 将来ノベルゲームエンジンの設定スキーマの**具体実装**（予約・方針設計に留める）
  - SSP プロパティ／永続化フォーマット等、設定解釈以外のランタイム挙動の変更

## Boundary Candidates
- 設定スキーマ分類モデル（SHIORI必須/任意/将来予約のメタ定義）
- 最小必須セクションの判定・バリデーション（actor 必須チェック等）
- ファイル不在/最小構成時のデフォルト適用ロジック（`PastaConfig::load` / `parse` 周辺）
- テンプレート生成・SSOT（最小／フルの2層、デフォルト値の単一源）
- 利用者ドキュメント／サンプルゴーストへの分類反映

## Out of Boundary
- 将来エンジン（areka 等）の設定スキーマ実装そのもの
- 設定解釈以外のランタイム挙動（トーク合成、永続化、デバッグ基盤など）の変更
- 設定値のバリデーション強化（型安全ラッパー等）— 本仕様は分類・最小化・後方互換に集中

## Upstream / Downstream
- **Upstream**: [crates/pasta_lua/src/loader/config.rs](../../../crates/pasta_lua/src/loader/config.rs)（既存の設定パース基盤）、`pasta.config` Lua モジュール、`STORE.actors` 初期化フロー
- **Downstream**: 利用者マニュアル（pasta-user-manual）の設定章、サンプルゴースト（hello-pasta）、将来のノベルゲームエンジン設定仕様（`[package]` 予約を消費する側）

## Existing Spec Touchpoints
- **Extends**: なし（pasta.toml スキーマの責務を持つ既存スペックは存在しない）
- **Adjacent**:
  - ukagaka-desktop-mascot の子仕様 `areka-P0-package-manager`（将来エンジンのパッケージ概念。`[package]` 予約の将来消費者になり得る。重複しないよう注意）
  - pasta-user-manual（設定ドキュメントの権威。分類反映時に連携）

## Constraints
- 完全後方互換: 既存のフル記述 `pasta.toml` はそのまま動作すること
- 設定解釈は Rust 側（`pasta_lua` ローダ）と Lua 側（`pasta.config` / `STORE.actors`）の両経路で一貫すること
- 既存テスト（loader/config_test.rs, startup_test.rs, virtual_event_config_test.rs 等）の意図を壊さないこと
- ドキュメント言語は日本語（spec.json.language = ja）
