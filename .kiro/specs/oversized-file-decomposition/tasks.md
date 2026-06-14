# Implementation Plan

> 純粋リファクタリング（振る舞い不変）。全ステップで `cargo build --workspace` / `cargo test --workspace`（env `NoDefaultCurrentDirectoryInExePath` 無効化前提）を green に保ち、1 ファイル/単位 = 1 検証 = 1 コミットの revert 可能な小ステップで進める。各兄弟テストファイルは `use super::*;`、本番側は `#[cfg(test)] #[path] mod ...;`。正確なファイル割当は design.md「File Structure Plan」を参照。

- [ ] 1. Foundation: 検証ベースラインと是正対象集合の確定
- [x] 1.1 テスト棚卸しベースライン捕捉と対象集合確定
  - `cargo` 実行前に `NoDefaultCurrentDirectoryInExePath` 環境変数を無効化する手順を確立する
  - 是正着手前に全テスト関数名の集合（`cargo test --workspace -- --list` 相当）をベースラインとして記録する
  - 確定インベントリ（design.md / research.md §2.1）と現状再スキャン（`src/` 本番にインライン `#[cfg(test)]` を持つ全ファイル＋純粋肥大本番）の和集合で是正対象ファイル集合を確定する
  - 各分割ステップで失敗時は次へ進まず同ステップ内で green を回復する段階的検証規律を全カテゴリ（C1–C4）共通で適用することを確認
  - 観測: `cargo build --workspace` / `cargo test --workspace` が green、ベースラインのテスト名集合と対象ファイル一覧が記録済み
  - _Requirements: 1.7, 5.1, 5.2, 5.3, 5.4_

- [ ] 2. C1: インラインテストの兄弟ファイルへの外出し
- [x] 2.1 (P) debug session モジュールのインラインテスト外出し
  - 単一インライン `mod tests` を論理クラスタ別の複数兄弟テストファイルへ分離（単一移動後も肥大のため複数クラスタへ）
  - テストが参照する private / `pub(crate)` 項目への到達性を可視性変更なしで保持し、テスト集合（名前・件数・アサーション）を移動のみに留める
  - 観測: session 本番にインライン `mod tests` が残存せず、全WS test green、ベースライン差分ゼロ、各兄弟ファイル < 600 行
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug session)_

- [x] 2.2 (P) debug dap モジュールのインラインテスト外出し
  - 単一インライン `mod tests` を論理クラスタ別の複数兄弟テストファイルへ分離
  - private / `pub(crate)` 到達性を `use super::*;` 経由で保持、移動のみ
  - 観測: dap 本番にインライン `mod tests` 残存せず、全WS test green、差分ゼロ、各兄弟 < 600 行
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug dap)_

- [x] 2.3 (P) debug source_map モジュールのインラインテスト外出し
  - 単一インライン `mod tests` を論理クラスタ別（resolve / builder / sidecar 等）の複数兄弟へ分離
  - 観測: source_map 本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ、各兄弟 < 600 行
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug source_map)_

- [x] 2.4 (P) debug transport / inspect / hook の単一クラスタテスト外出し
  - 各ファイルの単一インライン `mod tests` を 1 兄弟ファイルへ移動（規約パターン）
  - 観測: 3 ファイルそれぞれ本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug transport, inspect, hook)_

- [x] 2.5 (P) debug backend エントリ (debug/mod) のインラインテスト外出し
  - ディレクトリモジュールエントリの単一インライン `mod tests` を 1 兄弟ファイルへ移動（名前衝突回避の命名）
  - 観測: debug エントリ本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug mod)_

- [x] 2.6 (P) code_gen (element_gen / scope_gen) のインラインテスト外出し
  - 各ファイルの単一インライン `mod tests` を 1 兄弟ファイルへ移動
  - 観測: 2 ファイル本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (code_gen)_

- [x] 2.7 (P) loader (config / discovery / extract) のインラインテスト外出し
  - 各ファイルの単一インライン `mod tests` を 1 兄弟ファイルへ移動
  - 観測: 3 ファイル本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (loader)_

- [x] 2.8 (P) transpiler のインラインテスト外出し
  - 単一インライン `mod tests` を 1 兄弟ファイルへ移動
  - 観測: transpiler 本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (transpiler)_

- [x] 2.9 (P) pasta_shiori windows のインラインテスト外出し
  - 単一インライン `mod tests` を 1 兄弟ファイルへ移動
  - 観測: windows 本番にインライン `mod tests` 残存せず、全WS green、差分ゼロ
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (pasta_shiori windows)_

- [x] 2.10 (P) debug wiring モジュールのインラインテスト外出し
  - 11 個の独立テスト `mod` を 11 兄弟ファイルへ 1:1 対応で分離（既存モジュール = 1 兄弟）
  - private / `pub(crate)` 到達性を保持、移動のみ・新規追加なし
  - 観測: wiring 本番にインライン `mod tests` が残存せず、全WS test green、差分ゼロ、各兄弟 < 600 行
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.5, 6.1, 6.2, 6.3_
  - _Boundary: C1 Inline Test Externalization (debug wiring)_

- [ ] 3. C2: 巨大テストファイルのクラスタ分割
- [x] 3.1 (P) runtime 統合テスト群のクラスタ分割
  - 巨大な runtime トグル E2E テストを task banner 境界で各 < 600 行の最小分割へ、共有ヘルパーは共有モジュール化
  - debug 統合テストを top-level と内側 mod の 2 ファイルへ分割し、各カテゴリ `main.rs` に `mod` 登録
  - 固定ポート中和 `#[ctor]`（テストハーネス側）がテストバイナリにリンクされ続けることを確認
  - 観測: 分割後も全WS test green、ベースライン差分ゼロ、各ファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_lua tests/runtime)_

- [x] 3.2 (P) loader 統合テストのクラスタ分割
  - 巨大な loader config テストを凝集境界で各 < 600 行の最小分割へ（既存同名ファイルと区別する命名）、`main.rs` に `mod` 登録
  - 観測: 全WS green、差分ゼロ、各ファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_lua tests/loader)_

- [x] 3.3 (P) transpiler 統合テストのクラスタ分割
  - record-wiring テストを被テスト本番モジュール境界（element / scope）で 2 分割、`main.rs` に `mod` 登録
  - 観測: 全WS green、差分ゼロ、各ファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_lua tests/transpiler)_

- [x] 3.4 (P) shiori 仮想イベント設定テストの分割判定
  - 僅少超過（~605 行）を凝集優先で据え置くか最小 2 分割するかを判定し、据え置く場合は理由を記録
  - 観測: 判定結果を反映し全WS green、差分ゼロ、超過据え置き時は理由が記録済み
  - _Requirements: 2.1, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_lua tests/shiori)_

- [ ] 3.5 (P) pasta_dsl キューコマンドテストのクラスタ分割
  - 巨大な cue コマンドテストを凝集境界で各 < 600 行の最小分割へ（flat tests・各々が独立テストバイナリ）
  - 観測: 全WS green、差分ゼロ、各ファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_dsl tests)_

- [ ] 3.6 (P) pasta_shiori 統合テスト群のクラスタ分割
  - 非同期コールバック統合テストと Lua リクエストテストを各 < 600 行の最小分割へ（内側 mod を凝集グルーピング・flat tests）
  - 観測: 全WS green、差分ゼロ、各ファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_shiori tests)_

- [ ] 3.7 (P) pasta_shiori 兄弟テスト (shiori_tests) の再分割
  - 既 `#[path]` 外出し済みの巨大テストを 2 サブファイルへ分割し、親サイトを多重 `#[cfg(test)] #[path] mod` 宣言へ置換（各サブに `use super::*;`）
  - 観測: 全WS green、差分ゼロ、各サブファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_shiori shiori_tests)_

- [ ] 3.8 (P) pasta_core 兄弟テスト (scene_table_tests) の再分割
  - 既 `#[path]` 外出し済みの巨大テストを被テスト挙動別に 2 サブファイルへ分割し、親サイトを多重 `#[path] mod` 宣言へ置換
  - 観測: 全WS green、差分ゼロ、各サブファイル < 600 行
  - _Requirements: 2.1, 2.2, 2.3, 5.5, 6.1_
  - _Boundary: C2 Test File Clustering (pasta_core scene_table_tests)_

- [ ] 4. C3: 純粋肥大本番ファイルの責務単位サブモジュール分割
- [ ] 4.1 (P) LSP 解析ビジターの責務分割
  - テストを含まない AST ビジター群を責務単位（scope / expr / action 系）の複数サブモジュールへ split-`impl` 分割
  - 親モジュールの公開 API・re-export を不変に保ち、実行時振る舞いを不変に保つ
  - 観測: `cargo check -p pasta_lsp` がコンパイル成功、全WS test green、各ファイル < 600 行、公開 API 不変
  - _Requirements: 3.1, 3.2, 3.3, 5.5, 6.1_
  - _Boundary: C3 Production Responsibility Split (pasta_lsp analysis)_

- [ ] 4.2 (P) ローダーエントリの責務分割
  - 肥大したローダーエントリを起動オーケストレーション / 処理パイプライン / ソースマップ構築の責務へ split-`impl` 分割
  - 公開 API・可視性を不変に保ち、内部で必要な可視性調整は never re-export の範囲に限定（戻り値構造体の `pub(super)` 化のみ）
  - 観測: `cargo check -p pasta_lua` 成功、全WS green、各ファイル < 600 行、公開 API 不変
  - _Requirements: 3.1, 3.2, 3.3, 5.5, 6.1_
  - _Boundary: C3 Production Responsibility Split (loader mod)_

- [ ] 4.3 (P) ランタイムエントリの責務分割
  - 肥大したランタイムエントリを VM 構築コア / ファクトリ / 実行・アクセサ / ライフサイクル(Drop) の責務へ split-`impl` 分割
  - private フィールドの子モジュール参照を可視性変更なしで成立させ、Drop 移設後も振る舞い不変
  - 観測: `cargo check -p pasta_lua` 成功、全WS green、各ファイル < 600 行、公開 API・Drop 挙動不変
  - _Requirements: 3.1, 3.2, 3.3, 5.5, 6.1_
  - _Boundary: C3 Production Responsibility Split (runtime mod)_

- [ ] 4.4 (P) DAP 本番残余のディレクトリモジュール化
  - テスト外出し後の DAP 本番を resolver / pending / decode / encode / codec ＋ hub のサブモジュールへ分割
  - 共有 private 自由関数の兄弟参照は `pub(super)` の最小シームで成立させ、`DapAdapter` の公開到達性（fully-qualified path）を不変に保つ
  - 観測: `cargo check -p pasta_lua` 成功、全WS green、各ファイル < 600 行、外部到達名不変
  - _Requirements: 3.1, 3.2, 3.3, 5.5, 6.1_
  - _Depends: 2.2_
  - _Boundary: C3 Production Responsibility Split (debug dap)_

- [ ] 4.5 (P) デバッグバックエンドエントリの責務分割
  - 本番を hub 残置＋プレゼンテーションモード値型 / 設定解決(zero-cost gate) / エラー型 / ハンドル(Drop) / 起動エントリの兄弟へ分割
  - 兄弟からの構築用に `pub(crate) fn new()` をハンドルへ追加（カプセル化強化・公開 API 不変）、`enable()` の無効時 early-return(zero-cost) を verbatim 保持
  - 観測: `cargo check -p pasta_lua` 成功、全WS green、各ファイル < 600 行、`lib.rs` 公開再公開が byte 不変
  - _Requirements: 3.1, 3.2, 3.3, 5.5, 6.1_
  - _Depends: 2.5_
  - _Boundary: C3 Production Responsibility Split (debug mod)_

- [ ] 5. C4: handle_inbound ループの順序保証付き解体
- [ ] 5.1 順序・原子性を固定する特性化テストの先行追加
  - 解体着手前に、`setBreakpoints` 要求が単一の breakpoints 応答を返し、かつ session へ何も転送しないこと（非転送・原子性）を直接 pin する特性化テストを追加
  - 対照として stop-context コマンドが session へ 1 件転送することを検証し、`apply→response→event→command` 順序の既存カバレッジを確認
  - 観測: 新規特性化テストが green、ベースラインに対する唯一の許容追加として記録、全WS green
  - _Requirements: 4.1, 4.2, 4.5, 5.3_
  - _Depends: 2.10_
  - _Boundary: C4 handle_inbound Decomposition (wiring tests)_

- [ ] 5.2 トグル/attach 適用分岐のヘルパー抽出
  - source-presentation トグル交換と明示 attach モード適用の分岐を、同一モジュール内 private free fn へ抽出（A・B）
  - apply が response/event より前に走る順序と poison→停止(非 panic) 不変条件を保持
  - 観測: 抽出後も特性化テスト・全WS test green、差分ゼロ、`handle_inbound` 呼び出し列が固定順序
  - _Requirements: 4.2, 4.3, 4.6, 5.2_
  - _Depends: 5.1_
  - _Boundary: C4 handle_inbound Decomposition (wiring)_

- [ ] 5.3 応答/イベント送出分岐のヘルパー抽出
  - 即時応答＋handshake イベント送出と attach 完了時の初期プレゼンテーションイベント送出を private free fn へ抽出（C・D）
  - ack が event より前に出る順序とピア切断→`false` 伝播を保持
  - 観測: 抽出後も特性化テスト・全WS green、差分ゼロ
  - _Requirements: 4.2, 4.3, 4.6, 5.2_
  - _Depends: 5.2_
  - _Boundary: C4 handle_inbound Decomposition (wiring)_

- [ ] 5.4 コマンド routing 抽出と順序文書化の確定
  - コマンド routing を private free fn へ抽出し、`setBreakpoints` 分岐を単一ユニットとして原子保持・session 非転送を維持（E）
  - `handle_inbound` に `apply→response→event→command` 順序を列挙する doc comment を確定し、`run_socket_bridge` のシグネチャ・I/O 多重化コアを不変に保つ
  - 観測: 抽出後も特性化テスト・全WS green、差分ゼロ、`run_socket_bridge` 無変更、順序 doc 記載済み
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.6, 5.2_
  - _Depends: 5.3_
  - _Boundary: C4 handle_inbound Decomposition (wiring)_

- [ ] 6. Validation: 全体回帰と完了基準の検証
- [ ] 6.1 完了基準の最終検証
  - 全是正の最終状態で `cargo build --workspace` / `cargo test --workspace` が green、ベースラインのテスト名集合が不変（C4 特性化テスト 1 本の追加のみ許容）であることを確認
  - 主基準（バイナリ）: `src/` 本番ファイルにインライン `#[cfg(test)] mod` テストが残存しないことを確認
  - 副基準: 是正対象の各 Rust ファイルが 600 行未満（凝集優先の超過は理由記録済み）、公開 API シグネチャ・可視性が不変であることを確認
  - 観測: 上記すべてを満たす検証結果が記録済み、回帰ゼロ
  - _Requirements: 1.6, 3.2, 5.1, 5.3, 6.1, 6.2, 6.3_
  - _Depends: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 4.1, 4.2, 4.3, 4.4, 4.5, 5.4_

## Implementation Notes
- ベースライン不変条件は `cargo test --workspace -- --list` の総数 **2021**（leaf 名 multiset 一致）。実行 passed 数(2010)ではなく `--list` 総数で差分判定する。
- マルチクラスタ分割でクラスタ跨ぎの共有テストヘルパーが必要な場合: 専用の `<name>_test_support.rs` 兄弟に `pub(super)` でまとめ、各クラスタが `use super::<name>_test_support::*;` で参照する（本番可視性は不変。test-only ヘルパーのみ pub(super) 化可）。各クラスタ <600行。
- `cargo test` 実行は `crates/pasta_lua/tests/fixtures/sample.generated.lua` を LF→CRLF で touch する（内容差分ゼロ）。dirty 化したら `git checkout --` で復元し boundary を保つ。
- `cargo` 実行前に必ず `unset NoDefaultCurrentDirectoryInExePath`（さもないと LuaJIT/mlua ビルドが exit 101）。
- C1 外出しは BYTE-IDENTICAL であること（task 2.5 でコメント1語の改変が reject された）。許容変換は `mod tests {` ラッパ除去＋本体の1段(4スペース)デデントのみ。コメント・空行・空白を一切変えない。報告前に `diff <(git show HEAD:<file> | sed -n '<start>,<end>p' | sed 's/^    //') <sibling>` が空であることを確認する。
- `--list` 不変条件の正準メトリクスは `cargo test --workspace -- --list | grep -cE ': test$'` == 2021（バイナリ別サマリ行の素朴な合計 2014 とは別物。後者を使わない）。
