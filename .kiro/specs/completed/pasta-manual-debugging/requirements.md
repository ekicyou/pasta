# Requirements Document

## Introduction

Phase 5 で VSCode Lua デバッグ連携（`pasta-vscode-lua-debug` + `pasta-source-map`、いずれも 2026-06-08 完了）が `.pasta` ソースレベルまで本番化されたが、その使い方が利用者マニュアル（mdBook サイト・GitHub Pages 公開）に存在しない。既存のルート `DEBUGGING.md` は運用ガイドとして有用だが「`.pasta` ソースレベルは実験的・将来」と記述しており陳腐化している。

本仕様は、pasta ゴースト作者が VSCode から `.pasta` ソースレベルでデバッグできるよう、**有効化 → VSCode 接続 → `.pasta` ソースレベル操作 → 構造的制約と緩和策**までを完全網羅するデバッグ章を mdBook 利用者マニュアルに追加する。あわせて `DEBUGGING.md` をマニュアルへ統合・最新化し、デバッグ内容の情報源を mdBook に一本化する。本仕様は **文書のみ** を対象とし、デバッグ機能そのものの実装は変更しない。

## Boundary Context

- **In scope**:
  - mdBook 利用者マニュアルへのデバッグ章追加（公開・検索・静的閲覧）
  - デバッグ有効化方法（`pasta.toml [debug]` / 環境変数）の説明
  - VSCode 接続（attach）手順の説明
  - `.pasta` ソースレベルのデバッグ操作（BP・停止・コールスタック・ステップ・変数 inspect・提示モード切替）の説明
  - 構造的制約「ブレーク中はホスト応答が止まる」と SSP タイムアウト緩和策の説明
  - `DEBUGGING.md` のマニュアルへの統合・最新化とルートファイルのリダイレクト化
  - 既存マニュアル執筆規約・検証ゲートへの準拠
- **Out of scope**:
  - デバッグ機能（Rust/DAP バックエンド）の実装変更
  - 文法・Lua を含むマニュアル全体の SSOT/権威化の再編（別仕様 `manual-ssot-authority` へ申し送り済み）
  - ランタイム内部設計の解説（別将来仕様 `pasta-runtime-internals-doc` の領域）
  - 構造的制約の根本解決（ホスト非同期化アーキテクチャ）
- **Adjacent expectations**:
  - 記述するデバッグ挙動は、完了済み仕様 `pasta-vscode-lua-debug` / `pasta-source-map` の実装に整合させる（マニュアルは消費側・読み取り専用）。
  - デバッグ章は `doc/spec/` 由来を持たないため、`manual-sources.toml` のドリフト追跡（文法章用）の対象外とする。
  - 既存の `book/tools` 検証（drift-check / static / search / verify-content）を壊さない。

## Requirements

### Requirement 1: デバッグ章のマニュアル統合と公開導線

**Objective:** pasta ゴースト作者として、デバッグの使い方を公開マニュアルサイトから検索・閲覧したい。これにより、実装済みのデバッグ機能をドキュメントだけで使い始められる。

#### Acceptance Criteria

1. The pasta 利用者マニュアル shall デバッグ章を `SUMMARY.md` に独立したセクションとして登録し、公開サイトのナビゲーションから到達可能にする。
2. The デバッグ章 shall 空でない本文（有効化・接続・操作・制約を含む）を持つ。
3. When `mdbook build` を実行したとき, the マニュアルビルド shall デバッグ章を含む静的サイトをエラーなく生成する。
4. When 利用者が公開サイトの日本語全文検索でデバッグ関連語（例: 「デバッグ」「ブレークポイント」）を検索したとき, the マニュアル検索 shall デバッグ章を検索結果に含める。
5. The デバッグ章 shall サーバー不要の静的 HTML+JS としてオフラインで閲覧可能である。

### Requirement 2: デバッグの有効化方法の説明

**Objective:** pasta ゴースト作者として、デバッグを安全に有効化／無効化する方法を知りたい。これにより、本番運用に影響を与えずデバッグを始められる。

#### Acceptance Criteria

1. The デバッグ章 shall `pasta.toml` の `[debug]` セクション（`enabled`・`port`）による有効化方法と各既定値を説明する。
2. The デバッグ章 shall 環境変数 `PASTA_DEBUG`（有効化）と `PASTA_DEBUG_PORT`（待ち受けポート上書き）による有効化方法を説明する。
3. Where 設定ファイルと環境変数が併存する場合, the デバッグ章 shall 環境変数が設定ファイルより優先されることを明示する。
4. The デバッグ章 shall デバッグが既定で無効であることを明示する。
5. While デバッグが無効のとき, the デバッグ章 shall 追加コストが発生せず（フック未設置・待ち受けポート未開放）、Lua の `debug` 機能がスクリプトへ露出しない（サンドボックス維持）ことを説明する。

### Requirement 3: VSCode 接続（attach）手順の説明

**Objective:** pasta ゴースト作者として、VSCode から実行中の pasta へ接続する手順を知りたい。これにより、デバッグセッションを開始できる。

#### Acceptance Criteria

1. The デバッグ章 shall VSCode のデバッグ拡張から Rust 側デバッグバックエンドへ **アタッチ（attach）** して操作する方式であることを説明する。
2. The デバッグ章 shall VSCode の `launch.json` によるアタッチ構成の設定手順を、利用者が再現できる具体例とともに示す。
3. The デバッグ章 shall 既定の接続先が `127.0.0.1:9276`（TCP・ローカル接続）であることを明示する。
4. When 利用者がポートを既定値から変更した場合, the デバッグ章 shall 有効化設定と接続構成のポートを一致させる必要があることを説明する。
5. The デバッグ章 shall 接続手順を VSCode を主軸に具体記述しつつ、バックエンドが DAP-over-TCP のホスト非依存実装であり VSCode 以外の DAP 互換クライアントからも接続しうることを簡潔に補足する。
6. The デバッグ章 shall デバッグに pasta VSCode 拡張が必須であることを明示し、拡張が未導入の読者を前提に拡張の導入手順を記述する。
7. Where VSCode 本体が未導入の場合, the デバッグ章 shall VSCode 本体のインストールは外部リンクの案内に留める（手順の詳細を二重記述しない）。

### Requirement 4: `.pasta` ソースレベルのデバッグ操作の説明

**Objective:** pasta ゴースト作者として、生成 `.lua` ではなく自分が書いた `.pasta` の座標でデバッグしたい。これにより、Pasta DSL の記述単位で挙動を確認できる。

#### Acceptance Criteria

1. The デバッグ章 shall `.pasta` ファイルの行に対するブレークポイント設定方法を説明する。
2. When ブレークポイントで実行が停止したとき, the デバッグ章 shall `.pasta` 座標での停止位置とコールスタックが提示されることを説明する。
3. The デバッグ章 shall `.pasta` 粒度のステップ実行（over / into / out）が、コルーチンを跨ぐ実行でも行えることを説明する。
4. While 実行が停止しているとき, the デバッグ章 shall 変数の inspect 方法を説明する。
5. The デバッグ章 shall 提示モードの切替（`.pasta` 既定／`.lua` への切替）が可能であることを説明する。
6. Where 利用者が任意のディスクサイドカー出力を有効化した場合, the デバッグ章 shall ソースマップのサイドカー出力が任意で得られることを説明する。
7. The デバッグ章 shall 体系的なガイド説明に加え、hello-pasta 等の実ゴーストを対象に「ブレークポイント設定 → attach → `.pasta` 座標での停止位置・変数確認」までを辿る短いウォークスルーを 1 つ含める。

### Requirement 5: 構造的制約と運用緩和策の説明

**Objective:** pasta ゴースト作者として、デバッグ中にゴーストが無反応になる理由と、その回避運用を知りたい。これにより、不具合と誤解せず、SSP タイムアウトを避けて作業できる。

#### Acceptance Criteria

1. The デバッグ章 shall ブレーク中はホスト（SHIORI/SSP）応答が停止することを、既知かつ意図された構造的挙動として説明する。
2. While 実行がブレークで停止しているとき, the デバッグ章 shall 後続の SHIORI リクエストも実行再開（continue）まで待機することを説明する。
3. The デバッグ章 shall SSP タイムアウトを避けるための運用緩和策（ブレークを短く保つ・デバッグ専用プロファイルの利用・時間に敏感なイベント処理中のブレーク回避）を説明する。
4. The デバッグ章 shall 構造的制約の根本解決（ホスト非同期化）は本マニュアルの対象外であり、提供するのは緩和策のみであることを明示する。

### Requirement 6: デバッグ情報源の一本化（DEBUGGING.md 統合・権威）

**Objective:** プロジェクト保守者として、デバッグの説明を一箇所で管理したい。これにより、陳腐化と二重管理を防ぎ、読者を最新情報へ導ける。

#### Acceptance Criteria

1. The デバッグ章 shall 既存ルート `DEBUGGING.md` の内容を取り込み、完了済み実装に整合した最新の記述にする。
2. The デバッグ章 shall `.pasta` ソースレベルのデバッグを「実験的・将来仕様」ではなく本番提供の機能として記述する。
3. When 統合後にルート `DEBUGGING.md` を参照したとき, the ルート `DEBUGGING.md` shall マニュアル該当章への薄いリダイレクト（誘導）として機能し、陳腐化した重複本文を残さない。
4. The 本仕様 shall 同一のデバッグ事実をマニュアルとルート文書で並行管理しない（mdBook をデバッグ内容の権威とする）。
5. The デバッグ章 shall `doc/spec/` 由来を持たないため `manual-sources.toml` のドリフト追跡対象に登録しない。

### Requirement 7: 執筆ボイス・規約準拠

**Objective:** マニュアル読者として、他章と一貫したトーンと正確さでデバッグ章を読みたい。これにより、学習体験が損なわれず正確に理解できる。

#### Acceptance Criteria

1. The デバッグ章 shall 章の導入と締めを Claudia 令嬢のキャラ口調で記述する（`book/AUTHORING.md` 準拠）。
2. The デバッグ章 shall 仕様・手順・設定・操作の説明本体を普通の文体（淡々・正確）で記述する。
3. The デバッグ章 shall コードブロック・表のセル・コマンド例・設定例の内部にキャラ口調を持ち込まない。
4. If キャラ口調によって技術的内容が不正確または読みにくくなる場合, the デバッグ章 shall 普通の文体を優先する。

### Requirement 8: 実装整合と既存検証の非回帰

**Objective:** プロジェクト保守者として、デバッグ章が実装と一致し、既存のマニュアル検証を壊さないことを保証したい。これにより、ドキュメント品質ゲートを維持できる。

#### Acceptance Criteria

1. The デバッグ章 shall 記述するデバッグ挙動を、完了済み仕様 `pasta-vscode-lua-debug` および `pasta-source-map` の実装と整合させる。
2. When 既存のマニュアル検証（drift-check / static / search / コンテンツ検証）を実行したとき, the マニュアル検証 shall デバッグ章追加後もエラーなく完了する。
3. The 本仕様 shall デバッグ機能そのもの（Rust/DAP バックエンド）の実装を変更しない。
4. When デバッグ章の追加・整合を機械的に確認したとき, the コンテンツ検証 shall デバッグ章の存在・本文・ボイス準拠を検査可能な形で確認できる。

### Requirement 9: 接続確認とトラブルシューティング

**Objective:** pasta ゴースト作者として、アタッチできたか分からないときや繋がらないときに、自力で状態を確認し原因を切り分けたい。これにより、成功を失敗と誤解せず、問題がアプリ側か VSCode 側かを判断できる。

#### Acceptance Criteria

1. The デバッグ章 shall アタッチが成立したかを利用者が確認する VSCode 側の接続サイン（実行とデバッグビューのコールスタックにセッションが表示される・デバッグツールバーが出る・ステータスバーの色が変わる）を説明する。
2. The デバッグ章 shall デバッグバックエンドが待ち受けているかを OS 側で確認する手段（待ち受けポートの確認・`Get-NetTCPConnection -LocalPort 9276`、確立済み接続の確認）を説明する。
3. While アタッチ済みでブレークしていないとき, the デバッグ章 shall ゴーストが停止せず通常どおり動作するのが正常である（不具合ではない）ことを明示する。
4. If アタッチに失敗する, then the デバッグ章 shall 代表的な失敗症状と原因の切り分け（pasta VSCode 拡張の未導入／`launch.json` の記述不足・必須フィールド欠落／ポート不一致／拡張更新後の VSCode 未リロード）を提供する。
5. The デバッグ章 shall 「アプリ側（待ち受け）か VSCode 側（接続）か」を二分する診断手順を提供する。
