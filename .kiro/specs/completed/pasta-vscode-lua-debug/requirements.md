# Requirements Document

## Introduction

ゴースト作者・pasta 開発者は、VSCode 上でスクリプトをステップ実行・ブレークポイント設定・変数監視できない。現状の手段は print デバッグと `@pasta_log`/tracing ログのみで、シーン実行（yield/resume コルーチン）の挙動追跡が困難である。VSCode 拡張は `languages`/`grammars`/`semanticTokens` のみを提供し、デバッグ機能（DAP）は皆無。Lua VM 側もデバッグフック未使用・`debug` ライブラリ既定 OFF である。

本仕様は、上流の実現可能性検証 `pasta-lua-debug-feasibility`（**GO+ 達成済み・2026-06-07**）が実証した「Rust ホスト型デバッグバックエンド」方式を本番化し、**VSCode から生成 Lua（`.lua`）レベルでブレークポイント設定・ステップ実行（over/into/out）・コールスタック表示・変数 inspect ができる**デバッグ基盤を提供する。これが本仕様の出荷コアである。デバッグ基盤はホスト非依存（SHIORI 以外の pasta ホストでも再利用可能）とし、デバッグ無効時は本番動作にゼロコストを保つ。あわせて、配布物に未配線で残存する旧 luasocket デバッグ資産を撤去し、本番移行完了後に使い捨て前提の検証ハーネスを除去する。

### スコープ分割（`.pasta` ソースマップ）

`.pasta`↔生成 `.lua` の**ソースマップ本番化**、および**`.pasta` 座標でのブレークポイント設定・コールスタック提示**は、それ自体が独立した最大級の塊（code_gen 改修・全 `generate_*` 網羅・双方向変換の正確性確保）であるため、**ダウンストリームの別仕様（仮称 `pasta-source-map`）へ切り出す**。

ただし本仕様は、その別仕様が安全に着手できるよう、**`.pasta` ソースマップの実現可能性を「調査確定」する**責務を負う。具体的には、(1) 実現可能性の調査、(2) **薄い実証スライス**（少なくとも 1 つの代表経路を実コードで end-to-end に実証する experimental／フィーチャーgate 付き実装）、(3) **将来別仕様が差し込める設計シーム**（生成行↔`.pasta` span 記録の接合点、マップ用インターフェース、DAP の source 取り扱い口）の整備までを本仕様スコープとする。`.pasta` ソースマップの**本番品質の実装**（全 `generate_*` 網羅・本番マップ出力・`.pasta` 座標の常時提示）は本仕様の対象外であり、別仕様へ委ねる。

要件は WHAT（利用者・運用者から観測可能な振る舞い）を定義する。トランスポート手段・フック API・JIT 制御方式・プロトコル実装などの HOW は design フェーズで確定する。

## Boundary Context

- **In scope**:
  - **Lua レベルのデバッグ（出荷コア）**: VSCode から生成 `.lua` 上で **ブレークポイント／ステップ実行（over/into/out）／続行／コールスタック表示／変数 inspect**
  - **コルーチン本体フレームの変数 inspect**（pasta のシーンはコルーチンで走るため必須。上流 PoC 制約 R3.4 の本番対応）
  - 外部 DAP クライアントが接続（attach）して上記操作を行うための **DAP 最小サブセット**の提供と、**VSCode 拡張のデバッグ構成**（薄い Descriptor Factory）
  - **`.pasta` ソースマップの実現可能性確定**: 調査 ＋ **薄い実証スライス**（代表 1 経路の end-to-end 実証・experimental／gate）＋ **将来別仕様向けの設計シーム**（生成行↔`.pasta` span 接合点・マップ用 IF・DAP source 取り扱い口）
  - **デバッグ有効化フラグ**（設定／環境変数）と、**無効時の本番ゼロコスト・サンドボックス維持**（実証スライスも同様に gate 配下）
  - デバッグ基盤の **ホスト非依存化**（SHIORI 非依存・pasta_lua 内蔵で再利用可能）
  - **ブレーク中のホスト応答停止**に関する運用注意と緩和策
  - **旧 luasocket デバッグ資産の撤去**（配布物肥大の解消）
  - 本番移行完了後の **検証ハーネス（PoC）の除去**（完了条件）
- **Out of scope**:
  - **`.pasta`↔`.lua` ソースマップの本番実装**（全 `generate_*` 網羅・本番マップ出力）と **`.pasta` 座標でのブレークポイント／コールスタックの常時提示** → **ダウンストリーム別仕様（仮称 `pasta-source-map`）**
  - 条件付きブレークポイント・ウォッチ式・ホットリロード（将来）
  - areka／非 SHIORI ホストへの実配線（基盤は再利用可能にするが、実ホスト統合は将来）
  - ブレーク中のホスト応答停止（SSP タイムアウト）の根本解決（構造的制約として明示し、緩和策に留める）
  - LSP 機能（既存 pasta_lsp の領分）
- **Adjacent expectations**:
  - **要件出典**: `ukagaka-desktop-mascot` Requirement 28 AC11–14（DAP サーバ提供／ブレークポイント停止／ステップ実行／変数 inspect）。本仕様はその pasta（Lua バックエンド）向け具体化である。
  - **上流**: `pasta-lua-debug-feasibility`（採用方式と既知制約の出典・GO+）、pasta_lua ランタイム（VM 初期化）、code_gen（ソースマップ素材）、editors/vscode（拡張統合先）。
  - **ダウンストリーム（新規・将来）**: `.pasta` ソースマップ本番化仕様（仮称 `pasta-source-map`）。本仕様が確定する**実現可能性ノート・薄い実証スライス・設計シーム**を入力として消費し、`.pasta` 座標の本番デバッグ体験を完成させる。
  - **依存方針**: 追加の外部依存（クレート／npm パッケージ）はサプライチェーンと配布サイズの観点から最小に留めることを期待する。

## Requirements

### Requirement 1: Lua レベルのブレークポイントとステップ実行（出荷コア）

**Objective:** pasta 開発者として、生成 `.lua` の任意行でスクリプト実行を止め、行単位で進めたい。print デバッグに頼らずシーン実行の流れを追跡できるようにするため。

#### Acceptance Criteria

1. When 利用者が生成 `.lua` の行にブレークポイントを設定したとき, the Pasta Debug Backend shall その実行行を停止対象として登録する。
2. When デバッグ対象の実行がブレークポイント設定行に到達したとき, the Pasta Debug Backend shall その行で実行を停止し、停止を DAP クライアントへ通知する。
3. While 実行がブレークポイントで停止している間, when 利用者がステップオーバーを指示したとき, the Pasta Debug Backend shall 現在行の呼び出しに入らず次の実行行で停止する。
4. While 実行がブレークポイントで停止している間, when 利用者がステップインを指示したとき, the Pasta Debug Backend shall 呼び出し先の実行位置で停止する。
5. While 実行がブレークポイントで停止している間, when 利用者がステップアウトを指示したとき, the Pasta Debug Backend shall 呼び出し元へ戻った実行位置で停止する。
6. While 実行が停止している間, when 利用者が続行（continue）を指示したとき, the Pasta Debug Backend shall 次のブレークポイントまたは終了まで実行を再開する。
7. The Pasta Debug Backend shall シーン実行を担うコルーチン群を横断して停止・ステップ操作を発火する。

### Requirement 2: コールスタック表示と変数 inspect

**Objective:** pasta 開発者として、停止地点でのコールスタックと変数の値を確認したい。どのフレームのどの局所変数が何になっているか把握できるようにするため。

#### Acceptance Criteria

1. While 実行が停止している間, when 利用者がコールスタックを要求したとき, the Pasta Debug Backend shall 各フレームを実行位置（生成 `.lua` のソース・行）で示したスタックを返す。
2. While 実行が停止している間, when 利用者が選択フレームの変数を要求したとき, the Pasta Debug Backend shall そのフレームで可視な局所変数・上位値を名前と値の組で返す。
3. The Pasta Debug Backend shall 文字列・数値・真偽値・テーブルを利用者が判別できる形で変数値として提示する。
4. While シーンを実行中のコルーチン本体フレームで停止している間, when 利用者が当該フレームの変数を要求したとき, the Pasta Debug Backend shall そのコルーチン本体フレームの局所変数を inspect できる。
5. If 変数値が安全に取得できない種別または到達不能なフレームであるとき, then the Pasta Debug Backend shall エラーで停止せず、取得不能である旨を提示して処理を継続する。

### Requirement 3: DAP 最小サブセットの提供と接続

**Objective:** pasta 開発者として、VSCode（DAP クライアント）からデバッグセッションを開始（attach）したい。標準 IDE 経由でデバッグ操作を行えるようにするため。

#### Acceptance Criteria

1. The Pasta Debug Backend shall DAP クライアントがローカル接続でアタッチするための接続口を、デバッグ有効時に提供する。
2. When DAP クライアントが初期化（initialize）を要求したとき, the Pasta Debug Backend shall 対応するケイパビリティを応答する。
3. The Pasta Debug Backend shall DAP の最小サブセットとして、ブレークポイント設定（setBreakpoints）・設定完了（configurationDone）・スレッド列挙（threads）・コールスタック取得（stackTrace）・スコープ取得（scopes）・変数取得（variables）・続行（continue）・ステップオーバー（next）・ステップイン（stepIn）・ステップアウト（stepOut）の各要求を処理する。
4. When 実行が停止したとき, the Pasta Debug Backend shall 停止イベント（stopped）を DAP クライアントへ送出する。
5. When デバッグ対象の実行が終了したとき, the Pasta Debug Backend shall 終了イベント（terminated）を DAP クライアントへ送出する。
6. The VSCode Pasta Debug 拡張 shall デバッグ構成を提供し、利用者が VSCode から Pasta Debug Backend へアタッチできるようにする。

### Requirement 4: `.pasta` ソースマップの実現可能性確定（調査＋薄い実証スライス＋設計シーム）

**Objective:** pasta 開発者として、`.pasta` 座標でのデバッグ（ソースマップ・`.pasta` ブレークポイント）を将来別仕様で安全に実装できるよう、本仕様で実現可能性を確定し、差し込み口を用意したい。別仕様の着手リスクを最小化するため。

> **注**: 本要件は実現可能性の確定（調査・薄い実証スライス・設計シーム）までを対象とする。`.pasta` ソースマップの**本番品質の実装**（全 `generate_*` 網羅・本番マップ出力・`.pasta` 座標の常時提示）は本仕様の対象外であり、ダウンストリーム別仕様（仮称 `pasta-source-map`）へ委ねる。

#### Acceptance Criteria

1. The 開発チーム shall `.pasta`↔生成 `.lua` の行対応が解決可能であることを実現可能性検証として確認し、結論・前提・残課題を `research.md` に記録する。
2. The Pasta コードジェネレータ shall 将来の本番ソースマップ仕様が差し込めるよう、出力行↔`.pasta` span を記録できる接合点（設計シーム）を用意する（本番マップ出力は将来仕様）。
3. The Pasta Debug Backend shall DAP の source 取り扱い口を、将来 `.pasta` パスを提示できる構造で用意する（本仕様の既定提示は生成 `.lua`）。
4. Where `.pasta` ソースマップ実証スライスが有効化されている場合, the Pasta Debug Backend shall 少なくとも 1 つの代表経路について、生成 `.lua` の停止位置を対応する `.pasta` 行へ変換して提示できる。
5. Where 同実証スライスが有効化されている場合, the Pasta Debug Backend shall 当該代表経路で `.pasta` 行に設定したブレークポイントをヒットできる。
6. While デバッグおよび実証スライスが無効の間, the Pasta ランタイム shall 実証スライスのコード経路を本番動作へ露出せず、追加コストを与えない。

### Requirement 5: デバッグ有効化フラグと本番ゼロコスト

**Objective:** 運用者として、デバッグ機能を明示的に有効化したときのみ動作させ、無効時は本番実行に一切の影響が出ないようにしたい。本番のサンドボックス安全性と性能を損なわないため。

#### Acceptance Criteria

1. The Pasta ランタイム shall デバッグ機能の有効・無効を設定または環境変数で切り替えられるようにする。
2. While デバッグが無効の間, the Pasta ランタイム shall デバッグ用フックを設置せず、本番実行の挙動・性能に追加コストを与えない。
3. While デバッグが無効の間, the Pasta ランタイム shall Lua の `debug`／`std_debug` 機能をスクリプトへ露出せず、サンドボックスを維持する。
4. Where デバッグが有効化されている場合, the Pasta ランタイム shall ブレーク・ステップ・変数 inspect に必要なフックと診断機能を有効化する。
5. While デバッグが無効の間, if 接続待ち受け口が設定されていないとき, then the Pasta ランタイム shall デバッグ用ネットワーク接続口を開かない。

### Requirement 6: ホスト非依存のデバッグ基盤

**Objective:** pasta 開発者として、デバッグ基盤を SHIORI 専用にせず pasta_lua に内蔵したい。SHIORI 以外の pasta ホストでも将来再利用できるようにするため。

#### Acceptance Criteria

1. The Pasta Debug Backend shall デバッグ機能（停止制御・ステップ・変数 inspect・接続口）を SHIORI 固有の処理に依存せず提供する。
2. The Pasta Debug Backend shall 任意の pasta ホストが組み込めるよう、ホスト種別に依存しないインターフェースでデバッグ基盤を公開する。
3. Where ホストが SHIORI 以外であっても, the Pasta Debug Backend shall 同一のデバッグ機能を再利用可能な形で提供する（実ホストへの配線は本仕様の対象外）。

### Requirement 7: ブレーク中のホスト応答停止と緩和

**Objective:** 運用者・ゴースト作者として、ブレーク中にホスト（SHIORI/SSP）応答が止まる構造的制約を理解し、想定外の挙動として混乱しないようにしたい。

#### Acceptance Criteria

1. While 実行がブレークポイントで停止している間, the Pasta Debug Backend shall 停止中はホストへのリクエスト応答が保留される構造的制約を持つことを、デバッグ利用ガイダンスとして明示する。
2. The Pasta Debug Backend shall ブレーク中のホスト応答停止に起因するタイムアウト（SSP 等）を避けるための運用上の注意・緩和策を提供する。
3. The Pasta Debug Backend shall ブレーク中のホスト応答停止（SSP タイムアウト）の根本解決を本仕様の対象外とし、緩和策に留める旨を明示する。

### Requirement 8: 旧 luasocket デバッグ資産の撤去

**Objective:** pasta 開発者として、配布物に未配線で残存する旧 luasocket デバッグ資産を取り除きたい。配布物（DLL 内同梱物）の肥大を解消し、未使用コードによる混乱を避けるため。

#### Acceptance Criteria

1. The Pasta 配布パッケージ shall 旧 luasocket デバッグ資産（`vscode-debuggee.lua`・`socket/core.dll`・`mime/core.dll`・`dkjson.lua`）を同梱しない。
2. When 旧 luasocket デバッグ資産を撤去した後, the Pasta ランタイム shall 既存の起動・スクリプト実行を従来どおり行える（撤去による回帰がない）。
3. The Pasta デバッグ基盤 shall 本仕様で提供する新デバッグ経路のみを用い、旧 Lua 側 luasocket デバッグ経路を維持しない。

### Requirement 9: 検証ハーネス（PoC）の除去（完了条件）

**Objective:** pasta 開発者として、役目を終えた使い捨て前提の検証ハーネスを本番移行完了後に撤去したい。検証足場を残置せずコードベースを整理するため。

#### Acceptance Criteria

1. While 本番デバッグ基盤（停止制御・DAP・ホスト非依存化・`.pasta` 実現可能性確定）が実装・検証済みで、かつ本番側に同等以上の自動テストが存在する間, the 開発チーム shall 上流 PoC が用いた検証ハーネス（`lua-debug-poc` フィーチャー一式）を除去できる状態にする。
2. When 除去の前提（本番実装の検証完了・PoC 知見の本番移行完了・PoC ハーネスへの依存解消）がすべて満たされたとき, the 開発チーム shall 検証ハーネス（`lua-debug-poc` フィーチャー、関連テストモジュール一式）を撤去する。
3. After 検証ハーネスを撤去した後, the 開発チーム shall 採用方式の妥当性（GO+）の担保を、本番自動テストおよび上流 `pasta-lua-debug-feasibility` の検証記録（research.md）に置き換える。
4. If 上記の除去前提のいずれかが未充足であるとき, then the 開発チーム shall 検証ハーネスを再検証エビデンスとして残置する。
