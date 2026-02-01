# 実装タスク: alpha04-sample-ghost

## タスク概要

hello-pasta ゴーストの完全実装。専用クレート構成、画像自動生成、ukadoc準拠設定、pasta DSLスクリプトを含む自己完結型サンプルゴースト。

---

## 実装タスク

- [x] 1. クレート基盤構築
- [x] 1.1 (P) pasta_sample_ghost クレート作成
  - `crates/pasta_sample_ghost/` ディレクトリ作成
  - Cargo.toml 作成（workspace members 登録、依存: image 0.25, imageproc 0.25）
  - src/lib.rs 作成（公開API: `generate_ghost()` 関数シグネチャ定義）
  - README.md 作成（クレート概要、使用方法、ビルド手順）
  - _Requirements: 1.1, 1.2, 1.4, 1.5_

- [x] 1.2 (P) 配布物ディレクトリ構造生成機能
  - `ghosts/hello-pasta/` 構造作成機能（install.txt, ghost/, shell/）
  - `ghost/master/` サブディレクトリ作成（descript.txt, pasta.toml, dic/）
  - `shell/master/` サブディレクトリ作成（descript.txt, surfaces.txt, *.png）
  - _Requirements: 1.1, 1.3_

- [x] 2. シェル画像生成機能
- [x] 2.1 (P) ピクトグラム画像生成ロジック
  - ImageGenerator モジュール実装（128x256px 透過PNG生成、3頭身比率）
  - トイレマーク風ピクトグラム描画（頭部: ○、胴体: △/▽、手足なし）
    - 女の子（sakura）: ○ + △（正三角形、スカート風）
    - 男の子（kero）: ○ + ▽（逆三角形）
  - 表情バリエーション実装（9種: 笑顔 `^ ^`, 通常 `-  -`, 照れ `> <`, 驚き `o o`, 泣き `; ;`, 困惑 `@ @`, キラキラ `* *`, 眠い `= =`, 怒り `# #`）
  - 表情の線: 太さ3px、目の間隔36px（視認性重視）
  - キャラクター色分け（sakura: 赤 #DC3545, kero: 青 #007BFF）
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 2.2 (P) サーフェス画像出力機能
  - sakura サーフェス生成（surface0.png 〜 surface8.png）
  - kero サーフェス生成（surface10.png 〜 surface18.png）
  - PNG ファイル書き込み機能（CI再現可能、外部依存なし）
  - _Requirements: 6.1, 6.4_

- [x] 3. 設定ファイル生成機能
- [x] 3.1 (P) ukadoc 設定ファイルテンプレート
  - install.txt 生成（type=ghost, name=hello-pasta, directory=hello-pasta）
  - ghost/master/descript.txt 生成（charset=UTF-8, shiori=pasta.dll, キャラクター名設定）
  - shell/master/descript.txt 生成（balloon offset 設定、craftman 情報）
  - surfaces.txt 生成（サーフェス定義）
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 3.2 (P) pasta.toml 設定テンプレート
  - [package] セクション生成（教育的コメント付き: 伺かゴーストでは省略可能、汎用用途サンプル）
  - [loader], [logging], [persistence], [lua] セクション生成
  - [ghost] セクション生成（spot_switch_newlines, talk_interval, hour_margin）
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 4. pasta DSL スクリプト実装
- [x] 4.1 起動・終了トーク
  - OnFirstBoot 初回起動メッセージ（pasta.shiori.act 使用）
  - OnBoot 起動挨拶（時間帯別: 朝/昼/夕/夜）
  - OnClose 終了挨拶
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 4.2 (P) ダブルクリック反応
  - OnMouseDoubleClick イベントハンドラ実装
  - ランダム選択による反応バリエーション（5種以上）
  - シンプルな pasta DSL のみの実装（入門者向け）
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4.3 (P) ランダムトーク
  - OnTalk 仮想イベントハンドラ実装
  - 複数トークパターン（5〜10種）
  - sakura と kero の掛け合いトーク
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.4 (P) 時報機能
  - OnHour 仮想イベントハンドラ実装
  - 時刻変数参照（`＄時`, `＄時１２` - onhour-date-var-transfer 仕様準拠）
  - 時間帯別バリエーション（24時間制/12時間制）
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. 統合テスト実装
- [x] 5.1 テストヘルパー実装
  - tests/common/mod.rs 作成
  - `copy_pasta_shiori_dll()` 実装（target/i686-pc-windows-msvc/release/pasta.dll 検出・コピー）
    - **注**: Cargo.toml の `[lib] name = "pasta"` により `pasta.dll`（`pasta_shiori.dll` ではない）
  - DLL 不在時の明確なエラーメッセージ
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 5.2 ゴースト生成テスト
  - TempDir 環境構築
  - `generate_ghost()` 実行テスト
  - ディレクトリ構造検証（install.txt, descript.txt, pasta.toml, dic/*.pasta, shell/*.png 存在確認）
  - _Requirements: 8.4, 8.7_

- [x] 5.3 イベントハンドラ動作テスト
  - PastaLoader 使用による各イベント検証（OnFirstBoot, OnBoot, OnClose, OnMouseDoubleClick, OnTalk, OnHour）
  - さくらスクリプト出力正確性検証
  - pasta.toml 設定読み込み検証
  - _Requirements: 8.4, 8.5, 8.6_

- [x] 5.4 CI 統合確認
  - `cargo test --workspace` 成功確認
  - 実SSP不要のモック環境完結性確認
  - pasta.dll 配布物含有確認
  - _Requirements: 8.6, 8.7, 8.8_

- [x] 5.5 (P) 配布ビルド自動化
  - `scripts/build-ghost.ps1` PowerShell スクリプト作成
  - pasta_shiori.dll ビルド（32bit Windows ターゲット i686-pc-windows-msvc）
  - テンプレートコピー（crates/pasta_sample_ghost/ghosts/hello-pasta/ → dist/hello-pasta/）
  - DLLコピー: `target/i686-pc-windows-msvc/release/pasta.dll` → `dist/hello-pasta/ghost/master/pasta.dll`
    - **注**: Cargo.toml の `[lib] name = "pasta"` により出力は `pasta.dll`（`pasta_shiori.dll` ではない）
  - Lua ランタイム再帰コピー（crates/pasta_lua/scripts/ → dist/hello-pasta/ghost/master/scripts/）
  - pasta.toml に `lua_search_paths = ["scripts/pasta", "scripts"]` 設定済み
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [x] 6. Soul Gate: SOUL.md 整合性確認
  - alpha04-sample-ghost が SOUL.md のビジョン・コアバリュー・設計原則と整合していることを確認
  - pasta DSL の教育的サンプルとしての役割が SOUL.md の「学習可能性」と一致することを検証
  - ukadoc 準拠による「伺か」エコシステム統合が SOUL.md の「統合性」と整合することを確認
  - _Requirements: 全要件（SOUL.md との整合性）_

---

## 完了基準（DoD）

- [x] **Spec Gate**: requirements, design, tasks すべて承認済み
- [x] **Test Gate**: `cargo test --workspace` 成功
- [x] **Doc Gate**: README.md にクレート概要・使用方法記載
- [x] **Steering Gate**: steering/structure.md, steering/tech.md との整合性確認
- [x] **Soul Gate**: SOUL.md との整合性確認完了（タスク 6 で検証）

---

## 要件カバレッジ

| 要件ID | 要件名 | 対応タスク |
|--------|--------|-----------|
| 1.1-1.5 | ディレクトリ構成 | 1.1, 1.2 |
| 2.1-2.4 | 起動・終了トーク | 4.1 |
| 3.1-3.4 | ダブルクリック反応 | 4.2 |
| 4.1-4.3 | ランダムトーク | 4.3 |
| 5.1-5.4 | 時報 | 4.4 |
| 6.1-6.5 | シェル素材 | 2.1, 2.2 |
| 7.1-7.5 | 設定ファイル（pasta.toml） | 3.2 |
| 8.1-8.8 | テスト要件 | 5.1, 5.2, 5.3, 5.4 |
| 9.1-9.4 | ukadoc設定ファイル | 3.1 |
| 10.1-10.6 | 配布ビルド自動化 | 5.5 |

全10要件、36個のAcceptance Criteriaをカバー。
