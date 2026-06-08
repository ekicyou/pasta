# Implementation Plan

## 1. Foundation: デバッグ章の足場とナビゲーション

- [x] 1.1 デバッグセクションの足場作成と SUMMARY 登録
  - `book/src/debug/` に 4 ページ（概要 / 接続・拡張導入 / `.pasta` 操作 / 構造的制約）の見出し付きスケルトンを作成する
  - `SUMMARY.md` に「デバッグ」セクション見出しと 4 ページへのリンクを追加する
  - `mdbook build book` がエラーなく完了し、4 ページが公開ナビゲーションに出現する（観測可能な完了条件）
  - _Requirements: 1.1, 1.3_
  - _Boundary: SUMMARY.md, debug/index.md, debug/vscode-setup.md, debug/source-level.md, debug/constraints.md_

## 2. Core: デバッグ章コンテンツ執筆

- [x] 2.1 (P) 概要・有効化・ウォークスルー章の執筆
  - デバッグの全体像（attach 方式・Rust ホスト型バックエンド・既定無効）を概説する
  - 有効化を記述: `pasta.toml [debug]` の `enabled`(既定 false)/`port`(既定 9276)、環境変数 `PASTA_DEBUG`/`PASTA_DEBUG_PORT`、環境変数が設定ファイルより優先、既定で無効、無効時はゼロコスト（フック未設置・ポート未開放）かつ Lua `debug` 非露出（サンドボックス維持）
  - hello-pasta を題材に「ブレークポイント設定 → attach → `.pasta` 座標での停止・変数確認」までを辿る短いウォークスルーを 1 つ置き、接続詳細は接続章へ前方リンクする
  - `.pasta` ソースレベルデバッグを本番提供機能として位置づける
  - 導入/締めを Claudia 令嬢ボイス、説明本体は普通文体、コードブロック/表/コマンドに口調を入れない、本文 800 字以上
  - 完成時: 概要章が有効化 4 値・env 優先・既定無効・ゼロコスト・ウォークスルーを含む本文を持つ（観測可能な完了条件）
  - _Requirements: 1.2, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.4, 4.7, 6.2, 7.1, 7.2, 7.3, 7.4, 8.1_
  - _Boundary: debug/index.md_
  - _Depends: 1.1_

- [x] 2.2 (P) 接続・拡張導入章の執筆
  - pasta VSCode 拡張（displayName「Pasta DSL」/ publisher ekicyou）が必須である旨を明示し、未導入前提で導入手順（Marketplace 検索＝主経路 / GitHub Releases の `pasta-vscode-<version>.vsix` ＝代替経路）を記述、VSCode 本体は外部リンクのみで案内する
  - attach 方式と `launch.json` の具体例（`type:"pasta"` / `request:"attach"` / `host` / `port` / 任意 `sourcePresentation`）を再現可能な形で示す
  - 既定接続先 `127.0.0.1:9276`（TCP・ローカル）を明示し、ポート変更時は有効化設定と接続構成のポート一致が必要と説明する
  - VSCode を主軸にしつつ、バックエンドが DAP-over-TCP ホスト非依存で他 DAP 互換クライアントからも接続しうる旨を一言補足する
  - 導入/締めをボイス、本体は普通文体、コード/表に口調なし、本文 800 字以上
  - 完成時: 接続章が拡張導入手順・`launch.json` 例・`127.0.0.1:9276`・attach 必須を含む（観測可能な完了条件）
  - _Requirements: 1.2, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 6.1, 7.1, 7.2, 7.3, 7.4, 8.1_
  - _Boundary: debug/vscode-setup.md_
  - _Depends: 1.1_

- [x] 2.3 (P) `.pasta` ソースレベル操作章の執筆
  - `.pasta` 行へのブレークポイント設定方法を記述する
  - 停止時に `.pasta` 座標での停止位置・コールスタックが提示されることを記述する
  - `.pasta` 粒度のステップ（over/into/out）がコルーチンを跨いでも行えることを記述する
  - 停止中の変数 inspect 方法を記述する
  - 提示モード切替（`.pasta` 既定 ⇄ `.lua`）と 3 経路の優先順位（`sourcePresentation` > `PASTA_DEBUG_SOURCE_MODE` > `present_as` > 既定 `.pasta`）を簡潔に記述する
  - 任意のディスクサイドカー出力（生成 `.lua` の隣 `<lua>.map`・`source_map_sidecar`/`PASTA_DEBUG_SOURCE_MAP_SIDECAR`）を記述する
  - `.pasta` ソースレベルを本番機能として記述、ボイス準拠、本文 800 字以上
  - 完成時: 操作章が 行BP/座標停止/ステップ/変数/提示モード/サイドカー の 6 観点を含む（観測可能な完了条件）
  - _Requirements: 1.2, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 6.1, 6.2, 7.1, 7.2, 7.3, 7.4, 8.1_
  - _Boundary: debug/source-level.md_
  - _Depends: 1.1_

- [x] 2.4 (P) 構造的制約と緩和策章の執筆
  - ブレーク中はホスト（SHIORI/SSP）応答が停止することを、`Arc<Mutex>` 直列化に由来する既知かつ意図された挙動として記述する
  - 停止中は現リクエストだけでなく後続 SHIORI リクエストも continue まで待機することを記述する
  - SSP タイムアウトを避ける運用緩和策（ブレークを短く・デバッグ専用プロファイル・時間に敏感なイベント中のブレーク回避）を記述する
  - 根本解決（ホスト非同期化）は本マニュアルの対象外であり提供するのは緩和策のみと明示する
  - ボイス準拠、本文 800 字以上
  - 完成時: 制約章が 構造的制約/後続待機/緩和策/スコープ外明示 を含む（観測可能な完了条件）
  - _Requirements: 1.2, 5.1, 5.2, 5.3, 5.4, 6.1, 7.1, 7.2, 7.3, 7.4, 8.1_
  - _Boundary: debug/constraints.md_
  - _Depends: 1.1_

## 3. Verification Tooling: コンテンツ機械検証の拡張

- [x] 3.1 verify-content にデバッグ章カテゴリ G を追加
  - 既存の `ok/fail/assert` と章ループのイディオムに倣い、デバッグ 4 ページを対象に検証カテゴリ G を追加する
  - G で検査: 各ページの存在、本文実体（最低 800 字）、散文部のボイスマーカー、コードフェンス内のナレーションマーカー非混入、主要事実（`9276`・`PASTA_DEBUG`・`attach`・`.pasta`・`sourcePresentation`）の登場
  - 既存カテゴリ A〜F が非回帰で維持される
  - 完成時: `node book/tools/verify-content.mjs` が G 全項目 PASS かつ A〜F 維持で exit 0（観測可能な完了条件）
  - _Requirements: 7.1, 7.3, 8.4_
  - _Boundary: verify-content.mjs_
  - _Depends: 2.1, 2.2, 2.3, 2.4_

## 4. Integration: デバッグ情報源の一本化

- [x] 4.1 DEBUGGING.md のリダイレクトスタブ化
  - ルート `DEBUGGING.md` の陳腐化本文（`.pasta` ソースレベル=実験的/将来 を含む）を全撤去する
  - 公開サイト URL（`https://ekicyou.github.io/pasta/`）＋ リポジトリ内相対パス（`book/src/debug/`）＋ デバッグ章が権威である旨の数行スタブへ置換し、同一のデバッグ事実を再掲しない
  - 完成時: `DEBUGGING.md` がリダイレクトスタブのみで、実験的/将来表現や重複本文が残っていない（観測可能な完了条件）
  - _Requirements: 6.3, 6.4_
  - _Boundary: DEBUGGING.md_
  - _Depends: 2.1, 2.2, 2.3, 2.4_

- [x] 4.2 (P) README ドキュメント表の誘導先更新
  - `README.md` のドキュメント表の `DEBUGGING.md` エントリ説明を、デバッグの権威がマニュアルのデバッグ章（公開 URL / 相対パス）へ移った旨へ更新する
  - 完成時: README 表がデバッグ説明の権威＝マニュアルを指し、読者をスタブへ誤誘導しない（観測可能な完了条件）
  - _Requirements: 6.4_
  - _Boundary: README.md_
  - _Depends: 2.1_

## 5. Validation: 検証と整合確認

- [x] 5.1 マニュアル検証スイートの実行（非回帰）
  - `mdbook build book` / `verify-content.mjs`（G+A〜F）/ `verify-static.mjs` / `verify-search.mjs` / `drift-check.mjs` を実行し全て成功させる
  - デバッグ章が日本語全文検索でヒットし、静的出力で file:// オフライン解決でき、未マップ警告ゼロ・リンク切れゼロを確認する
  - 完成時: 全検証スクリプトが exit 0、デバッグ章が検索ヒット（観測可能な完了条件）
  - _Requirements: 1.3, 1.4, 1.5, 6.5, 8.2_
  - _Depends: 3.1, 4.1, 4.2_

- [x] 5.2 コンテンツ整合・人手確認の記録
  - `CONTENT-REVIEW.md` にデバッグ章の確認項目を追補し、記載値が research.md §7 の確定事実と一致（8.1）、説明本体が普通文体で誤読の余地なし（7.2/7.4）、`.pasta` ソースレベルが本番機能として記述（6.2）、`DEBUGGING.md` に二重管理がない（6.3/6.4）、デバッグ実装（Rust/DAP・VSCode 拡張）を変更していない（8.3）を確認・記録する
  - 完成時: CONTENT-REVIEW.md にデバッグ章の確認記録が追記され全項目が確認済み（観測可能な完了条件）
  - _Requirements: 6.2, 6.3, 6.4, 7.2, 7.4, 8.1, 8.3_
  - _Depends: 5.1_

## 6. Follow-up: トラブルシューティング章（接続確認・二分診断）

- [x] 6.1 トラブルシューティング章の執筆と SUMMARY 登録
  - `book/src/debug/troubleshooting.md` を新設し SUMMARY の「デバッグ」セクションへ登録する
  - 内容: アタッチ成立の VSCode 側確認サイン（コールスタックのセッション表示・デバッグツールバー・ステータスバー色）／待ち受け・接続の OS 側確認（`Get-NetTCPConnection -LocalPort 9276`・確立済み接続）／アタッチ中もゴーストは固まらないのが正常である旨／代表的な失敗症状→原因（拡張未導入・`launch.json` 記述不足・ポート不一致・VSCode 未リロード）／「アプリ側か VSCode 側か」を二分する診断手順
  - 導入/締めは Claudia ボイス、本体は普通文体、コード/表/コマンド例に口調を入れない、本文 800 字以上
  - 完成時: troubleshooting.md が本文を持ち、`mdbook build book` が成功し SUMMARY ナビに出現する（観測可能な完了条件）
  - _Requirements: 1.2, 1.3, 9.1, 9.2, 9.3, 9.4, 9.5, 7.1, 7.2, 7.3, 7.4_
  - _Boundary: debug/troubleshooting.md, SUMMARY.md_
  - _Depends: 2.2, 2.4_

- [x] 6.2 verify-content カテゴリ G の対象へ troubleshooting を追加し検証スイートを再実行
  - `verify-content.mjs` の `DEBUG_CHAPTERS` に `troubleshooting` を追加し、新章も G の存在・本文・ボイス検査対象に含める
  - `mdbook build book` / `verify-content`(G+A〜F) / `verify-static` / `verify-search` / `drift-check` を全て緑にする
  - 完成時: 全検証が exit 0、troubleshooting 章が G 検査対象かつ日本語検索でヒット（観測可能な完了条件）
  - _Requirements: 1.4, 8.2, 8.4_
  - _Boundary: verify-content.mjs_
  - _Depends: 6.1_
