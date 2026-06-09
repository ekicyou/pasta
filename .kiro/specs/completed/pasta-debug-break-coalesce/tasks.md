# Implementation Plan: pasta-debug-break-coalesce

- [x] 1. `.pasta` 行アンカーの状態機械（状態フィールド＋遷移ヘルパー＋単体テスト）
  - 停止状態機械に「直前停止の `.pasta` 行」を保持する内部状態（VM スレッド単一・`&self` 内部可変・既存 run mode と同型のスレッド規律）を追加し、初期値は「アンカーなし」。ソースマップ／提示モードの注入ヘルパーはこの状態に触れない。
  - アンカーを1ステップ進める遷移ヘルパーを実装：現在行の `.pasta` 位置に対し、(a) アンカーと同一 `.pasta` 行なら「抑制適格」を返しアンカー不変、(b) 別の対応 `.pasta` 行ならアンカーを解除して非適格、(c) 未対応行（解決不能）はアンカー不変で非適格、(d) アンカーなしは非適格。副作用は「別 `.pasta` 行へ移動時の解除」のみ。
  - 単体テストで上記4遷移を非空に検証（同一→適格・不変／別行→非適格・解除／未対応→非適格・不変／なし→非適格）。
  - 同一 `.pasta` 行へマップする異なる2つの `.lua` 行が等価な `.pasta` 位置（同一 file・同一 line）へ解決されることを単体テストで固定（アンカー等価判定の前提・既存ステップ集約と同一不変条件）。
  - 観測可能な完了：本ヘルパーの新規単体テスト群（4遷移＋等価不変条件）が緑で、アンカー状態の遷移が試験で確認できる。
  - _Requirements: 1.1, 2.1, 2.2, 2.3_
  - _Boundary: DebugSession_

- [x] 2. 行フック判定への統合（再ブレーク消化の本体）
  - 行フック判定のブレークポイント優先分岐に、Pasta 提示モードかつソースマップ在のときのみ作用するアンカー処理を組み込む：毎行アンカーを1ステップ進め（離脱解除を保証）、ブレークポイント命中かつ抑制適格なら停止せず消化し追加の停止イベントを出さない、非適格なら停止直前に現在の `.pasta` 位置をアンカーへ確立してブレークポイント停止する。
  - `.lua` 提示モード／ソースマップ非在／デバッグ無効では一切作用させず、従来経路と同一（既存挙動の不変）。未対応行でのブレークポイント停止はアンカー未設定＝lua 粒度（多対1 `.pasta` 行シナリオ外）。
  - ブレークポイント優先→ステップ判定の評価順序は不変。停止位置（`.pasta` ソース・行）と停止理由（ブレークポイント由来）は既存提示のまま。
  - 観測可能な完了：`.pasta` 行ブレークで停止後の Continue が同一 `.pasta` 行で再停止せず次の対応行／停止点へ進み、`.lua` モード・ソースマップ非在の既存挙動が不変であることを session レベルの集約テストで確認できる。
  - _Requirements: 1.1, 1.2, 1.3, 2.4, 3.1, 3.2, 4.1, 4.2, 5.2_
  - _Boundary: DebugSession_
  - _Depends: 1_

- [ ] 3. 検証（実 DAP-over-TCP E2E ＋ 無回帰）
- [x] 3.1 多対1・ループ再訪を含む `.pasta` E2E フィクスチャ整備
  - 1つの `.pasta` 行が複数の `.lua` 行へ展開される構成、および同一 `.pasta` 行を複数回通るループ構成を含む最小 `.pasta` 辞書（およびロード経路）を用意する。
  - 観測可能な完了：当該フィクスチャがトランスパイル・ロードでき、対象 `.pasta` 行がソースマップ上で2つ以上の `.lua` 行に対応することを試験で確認できる。
  - _Requirements: 6.2_
  - _Boundary: debug_integration_test fixtures_
  - _Depends: 2_

- [x] 3.2 多対1 Continue の実 DAP-over-TCP E2E
  - 既存の実ソケットデバッグハーネス様式（接続→setBreakpoints→configurationDone→停止→continue）を用い、多対1フィクスチャで `.pasta` 行ブレークに停止後、continue を1回送ると次の `.pasta` 行（または停止点）まで進み、同一 `.pasta` 行で停止イベントが再送されないことを検証する。
  - 観測可能な完了：continue 1回で同一 `.pasta` 行の再停止イベントが発生せず次行で停止する E2E が緑。
  - _Requirements: 1.1, 1.2, 3.2, 6.2_
  - _Depends: 2, 3.1_

- [x] 3.3 ループ再訪の実 DAP-over-TCP E2E
  - ループフィクスチャで、同一 `.pasta` 行をループで通過した回数ぶんブレークポイント停止イベントが発生することを検証する。
  - 観測可能な完了：N 回訪問で N 回の停止が観測できる E2E が緑。
  - _Requirements: 2.2, 6.2_
  - _Depends: 2, 3.1_

- [x] 3.4 無回帰・モード直交・OFF 不変の確認
  - 既存の Lua レベルデバッグ E2E・ソースマップ受け渡し試験を含む `pasta_lua` 全テストが緑であること、`.lua` 提示モード・ソースマップ非在で挙動不変、デバッグ無効（OFF）経路がバイト不変（既存のゼロコスト試験で担保）であることを確認する。
  - 注意：ビルド／テスト前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外さないと mlua-sys/LuaJIT ビルドが失敗（exit 101）するため、実行環境で解除しておく。
  - 観測可能な完了：`cargo test -p pasta_lua` 全緑で、既存のデバッグ／ソースマップ試験に赤がない。
  - _Requirements: 4.3, 5.1, 6.1_
  - _Depends: 2_

## Implementation Notes
- **Pasta+SourceMap の実 DAP-over-TCP E2E は `crates/pasta_lua/src/debug/wiring.rs` の `#[cfg(test)]` モジュールに置く**（`tests/runtime/debug_integration_test.rs` ではない）。理由：外部結合テストから到達可能な `PastaLuaRuntime::with_config` は source map に `None` を渡し `.lua` 行 BP のみ対応するため、`SourceMode::Pasta` + `Some(map)` の coalescing 経路に到達できない。当該経路を駆動できる実ソケットハーネスはクレート内部の `enable(&lua,&cfg,Some(map))` + `read_frame`/`write_frame` を使う wiring.rs の既存 `pasta_bp_e2e`/`pasta_step_e2e` のみ。Task 3.3 のループ再訪 E2E も同様にこのハーネス様式で書く。
- `crates/pasta_lua/tests/fixtures/sample.generated.lua` は `cargo test` 実行で CRLF のみの churn（テキスト差分ゼロ）が出る既存ハーネス副作用。コミットにはステージしない。
