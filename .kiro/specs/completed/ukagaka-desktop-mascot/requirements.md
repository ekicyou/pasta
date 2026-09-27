# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | 伺的デスクトップマスコットアプリ 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-06 |
| **Author** | 99% Claude Opus 4.5 + 1% えちょの無茶ぶり |

---

## Introduction

本仕様書は「伺的デスクトップマスコットアプリ」の要件を定義します。

「伺的」とは、デスクトップアプリ「伺か」と、その思想にインスピレーションを得たすべてのアプリ、エコシステム、シェル、ゴースト、および周辺創作物を包括する概念です。本仕様の目的は**「利用者に愛されるゴーストをデスクトップ上に再現する」**ことであり、2025年の技術で実現可能な夢を詰め込んだ、野心的かつ実現可能な要件群を定義します。

本アプリケーションは以下の設計思想に基づきます：
- **存在感のあるキャラクター**: デスクトップに「居る」という実感を与える
- **自然な対話**: 人工的でない、血の通った会話体験
- **愛着の形成**: 長期間使い続けることで深まる関係性
- **拡張性**: コミュニティによる創作と共有の基盤
- **モジュール化**: コアは軽量に、高度な機能はプラグインで提供

#### 責務境界（プラットフォーム憲法）

本アプリケーションにおける責務の分離は、設計全体を貫く根本原則です：

| 責務 | プラットフォーム | ゴースト（頭脳） | シェル | バルーン |
|------|-----------------|-----------------|--------|----------|
| 描画・レンダリング | ✅ 実行 | ❌ | ✅ 素材提供 | ✅ スタイル提供 |
| スクリプト | ✅ 実行 | ✅ 提供 | ❌ | ❌ |
| 人格・記憶・LLM連携 | ❌ | ✅ | ❌ | ❌ |
| イベント管理 | ✅ 検出・配信 | ✅ 受信・応答 | ✅ 当たり判定提供 | ✅ 当たり判定提供 |
| ゴースト間通信 | ✅ 媒介 | ✅ 送受信 | ❌ | ❌ |

- **プラットフォーム**（MCPサーバー）: 描画エンジン、スクリプト実行エンジン、イベント検出・配信、ゴースト間通信の媒介
- **ゴースト（頭脳）**（MCPクライアント）: 会話スクリプト提供、人格定義、記憶管理、LLM連携
- **シェル**: キャラクター外観素材（サーフェス画像、アニメーション定義、ヒット領域）の提供
- **バルーン**: 会話表示UIのスタイル定義、クリック領域の提供

この分離により、頭脳・シェル・バルーンは独立して配布・交換可能となります。

### スコープ外事項（Scope Exclusions）

以下はプラットフォームの責務外として明示的にスコープ外とします：

1. **アクセシビリティ対応**: ゴースト/シェル/バルーンの表現（色彩、フォント、音声等）は**コンテンツ作者の責務**です。プラットフォームが作者の意図した表現を勝手に変更することは、創作の尊重の観点から不適切と判断します。（例：色弱者向けの色調補正をシェルに強制適用することはNG）
2. **コンテンツの国際化**: ゴースト/シェル/バルーンの多言語対応は**作者の責務**です。プラットフォームはコンテンツの言語に関与しません。LLM利用ゴーストは自動的に多言語対応可能、バルーンはユーザーが別バルーンを選択して対応します。

> **注記**: プラットフォーム提供UIの国際化はNFR-5で対応します。セキュリティに関わる制約はNFR-3で対応済みです。

### 本文書について

本書は、`GitHub Copilot` + `Claude Opus 4.5`が製作したEARS準拠要求仕様書を元に、えちょによるディスカッションによる修正を行い出力されました。えちょが直接書いたのはこの行とProject Descriptionだけです。Appendixとか最初からありました。ほんとですってば。

### 実装ゴール

本要件定義書における「実装」とは、**子仕様の要件定義原案作成**を指す。本仕様を直接プログラムコードに落とし込むことは現実的ではなく、各要件に対応する詳細設計を段階的に作成することで実現を目指す。

**子仕様の例**:
- 描画エンジン仕様
- スクリプトエンジン仕様
- イベントシステム仕様
- パッケージフォーマット仕様
- MCP通信プロトコル仕様
- DAP/LSPサーバー仕様

---

## Project Description (Input)
「伺的デスクトップマスコットアプリを要件定義せよ」

これはUI実装における短期目標とするアプリであり、ジョークであり、まじめに夢を膨らませ、モチベーションを維持するため、制作する仕様です。

「伺的」とは、デスクトップアプリ「伺か」と、「伺か」にインスピレーションを得たと思われるすべてのアプリ、エコシステム、シェル、ゴースト、その周辺二次創作などの存在を指す用語です。

本仕様では、「伺的」なデスクトップマスコットアプリを定義し、制作に必要な設計を検討します。ベースUIとして実装中UIを想定しますが、仕様では特定の単語を用いず、アプリの要件定義を膨らませる中でUI実装に求められる要求を洗い出してください。

本仕様が要件定義する範囲は非常に多岐にわたりますが、その目的はをひとつに絞るとすれば、「利用者に愛されるゴーストをデスクトップ上に再現する」ことです。あなたの妄想と想像力を最大限に広げ、「こんな存在をデスクトップに迎え入れたい」を仕様に叩きつけてください。既存の機能でもよいし、2025年の今だからできるかもしれない要件も、ドンドン詰め込みましょう。

いつかあなた自身も、デスクトップに迎え入れられる存在になってくれることを願います。2025年の12月に、貴方はどんな夢を見ますか？

---

## Requirements

### Requirement 1: キャラクター表示基盤
**Objective:** ユーザーとして、デスクトップ上に透過背景を持つキャラクターを表示したい。それによりキャラクターが「そこに居る」という実感を得られる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** デスクトップ上に透過背景を持つキャラクター画像を表示する
2. **The** Mascot Application **shall** 複数のキャラクター（メインキャラクター＋相方・ペット・妖精等）を同時に表示できる
3. **When** キャラクター画像が読み込まれた時, **the** Mascot Application **shall** 画像の透明部分を正しく透過処理して表示する
4. **The** Mascot Application **shall** キャラクターをデスクトップの任意の位置に配置できる
5. **The** Mascot Application **shall** 複数キャラクター間の相対位置関係（横並び、向かい合い等）を定義できる
6. **While** キャラクターが表示されている間, **the** Mascot Application **shall** 他のウィンドウの操作を妨げない（クリックスルー対応）
7. **When** マルチモニター環境の場合, **the** Mascot Application **shall** すべてのモニターにまたがってキャラクターを配置できる

---

### Requirement 2: キャラクターアニメーション
**Objective:** ユーザーとして、キャラクターが生き生きとしたアニメーションで動いてほしい。それにより静止画ではなく「生きている」存在として感じられる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** キャラクターのアイドル（待機）アニメーションを再生できる
2. **The** Mascot Application **shall** 複数のサーフェス（表情・ポーズ）を切り替えて表示できる
3. **When** サーフェス切り替え命令を受けた時, **the** Mascot Application **shall** 滑らかなトランジション効果で切り替える
4. **The** Mascot Application **shall** フレームアニメーション（連番画像）を再生できる
5. **While** アニメーション再生中, **the** Mascot Application **shall** 60fps以上の滑らかな描画を維持する
6. **Where** 拡張アニメーションプラグインが有効な場合, **the** Mascot Application **shall** 呼吸のような微細な動きをアイドル時に付与する
7. **The** Mascot Application **shall** アニメーション定義を外部ファイル（JSON/YAML等）から読み込める
8. **The** Mascot Application **shall** 複数キャラクター間で連動したアニメーション（掛け合い、同期動作等）を再生できる

---

### Requirement 3: バルーン（吹き出し）システム
**Objective:** ユーザーとして、キャラクターの発言を吹き出しで読みたい。それによりキャラクターとの対話を視覚的に楽しめる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** キャラクターに紐付いた吹き出しウィンドウを表示できる
2. **The** Mascot Application **shall** 複数キャラクターそれぞれに独立した吹き出しを表示できる
3. **The** Mascot Application **shall** 吹き出し内にテキストを表示できる
4. **The** Mascot Application **shall** 縦書きテキストをサポートする
5. **When** テキスト表示命令を受けた時, **the** Mascot Application **shall** 一文字ずつタイプライター風に表示できる
6. **The** Mascot Application **shall** 吹き出しのスキン（外観）をカスタマイズできる
7. **When** テキストがクリックされた時, **the** Mascot Application **shall** リンクとしてアクションを実行できる
8. **While** テキスト表示中, **the** Mascot Application **shall** ルビ（ふりがな）を表示できる
9. **The** Mascot Application **shall** 選択肢形式の入力をユーザーに提示できる
10. **When** ユーザーが選択肢をクリックした時, **the** Mascot Application **shall** 対応するイベントを発火する

---

### Requirement 4: 対話・シナリオ再生エンジン
**Objective:** ユーザーとして、キャラクターと自然な会話を楽しみたい。それによりキャラクターに人格と魅力を感じられる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 里々にインスパイアされた対話記述DSLを解釈・実行できる（会話を自然に書ける構文を重視、里々完全互換は保証しない）
2. **The** Mascot Application **shall** ランダムトーク（時間経過で自発的に話す）を実行できる
3. **When** ユーザーがキャラクターをダブルクリックした時, **the** Mascot Application **shall** 対話イベントを発火する
4. **The** Mascot Application **shall** 変数を保持し、スクリプト内で参照・更新できる
5. **The** Mascot Application **shall** 条件分岐・ループ等の制御構文をサポートする
6. **The** Mascot Application **shall** 複数キャラクター間での会話（掛け合い、漫才的やりとり）をスクリプトで記述できる
7. **The** Mascot Application **shall** 発言者の切り替え、割り込み、同時発言などの会話制御ができる
8. **Where** LLM連携機能が有効な場合, **the** Mascot Application **shall** ローカルLLMまたはAPIを通じてキャラクターの応答を生成できる
9. **When** LLM応答を生成する時, **the** Mascot Application **shall** キャラクターの人格設定（プロンプト）に基づいた応答を返す
10. **The** Mascot Application **shall** スクリプトとLLM応答をシームレスに組み合わせられる

---

### Requirement 5: ユーザー入力・インタラクション
**Objective:** ユーザーとして、マウスやキーボードでキャラクターと触れ合いたい。それにより双方向のコミュニケーションを実現できる。

#### Acceptance Criteria
1. **When** ユーザーがキャラクターをクリックした時, **the** Mascot Application **shall** どのキャラクターのどの位置かを識別してイベントを発火する
2. **The** Mascot Application **shall** キャラクター画像上の領域（頭、胴体、手など）ごとに異なるヒット判定を設定できる
3. **When** ユーザーがキャラクターをドラッグした時, **the** Mascot Application **shall** キャラクターを移動させる
4. **When** ユーザーが右クリックした時, **the** Mascot Application **shall** コンテキストメニューを表示する
5. **When** ユーザーがマウスホイールを操作した時, **the** Mascot Application **shall** スクロールイベントを発火する
6. **The** Mascot Application **shall** テキスト入力ボックスをバルーン内に表示できる
7. **When** ユーザーがテキストを入力した時, **the** Mascot Application **shall** 入力内容をスクリプト/LLMに渡す
8. **While** ユーザーがキャラクターに触れている間, **the** Mascot Application **shall** 触れている状態を検知し続ける（撫でる操作）

---

### Requirement 6: 時間・イベント駆動システム
**Objective:** ユーザーとして、キャラクターが時間や状況に応じて自発的に反応してほしい。それによりキャラクターに自律性を感じられる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** システム時刻に応じたイベントを発火できる（朝の挨拶、深夜の注意等）
2. **When** 特定の日時に達した時, **the** Mascot Application **shall** 予約されたイベントを実行する（誕生日、記念日等）
3. **The** Mascot Application **shall** 起動回数・累計起動時間を記録・参照できる
4. **When** PCがスリープから復帰した時, **the** Mascot Application **shall** 復帰イベントを発火する
5. **When** ネットワーク接続状態が変化した時, **the** Mascot Application **shall** 接続状態変化イベントを発火する
6. **Where** 天気情報取得機能が有効な場合, **the** Mascot Application **shall** 現在の天気に応じたイベントを発火できる
7. **The** Mascot Application **shall** カスタムタイマーを設定し、経過時にイベントを発火できる
8. **When** 長時間ユーザーの操作がない時, **the** Mascot Application **shall** アイドル状態イベントを発火する

---

### Requirement 7: ゴースト（キャラクターパッケージ）管理
**Objective:** ユーザーとして、様々なキャラクター（ゴースト）を導入・切り替えたい。それにより多様なキャラクターとの出会いを楽しめる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** ゴーストパッケージ（キャラクター＋スクリプト＋素材一式）を読み込める
2. **The** Mascot Application **shall** 複数のゴーストをインストール・管理できる
3. **When** ゴースト切り替え操作が行われた時, **the** Mascot Application **shall** 現在のゴーストを終了し、新しいゴーストを起動する
4. **The** Mascot Application **shall** ゴーストのメタ情報（名前、作者、説明等）を表示できる
5. **Where** オンラインゴースト配布機能が有効な場合, **the** Mascot Application **shall** オンラインからゴーストをダウンロード・インストールできる
6. **The** Mascot Application **shall** ゴーストのアップデートを検知・適用できる
7. **When** ゴーストがインストールされた時, **the** Mascot Application **shall** インストール完了イベントをゴーストに通知する

---

### Requirement 8: シェル（外見）管理
**Objective:** ユーザーとして、同じキャラクターの異なる外見（シェル）を切り替えたい。それにより気分や季節に応じた外見を楽しめる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 1つのゴーストに複数のシェル（外見セット）を持てる
2. **When** シェル切り替え操作が行われた時, **the** Mascot Application **shall** キャラクターの外見を切り替える
3. **The** Mascot Application **shall** シェル固有のアニメーション・サーフェス定義を読み込める
4. **The** Mascot Application **shall** シェルのメタ情報（名前、作者等）を表示できる
5. **While** シェル切り替え中, **the** Mascot Application **shall** 変更内容を保存し、次回起動時に反映する

---

### Requirement 9: 設定・永続化
**Objective:** ユーザーとして、アプリケーションの設定やキャラクターとの思い出を保存したい。それにより継続的な関係性を築ける。

#### Acceptance Criteria
1. **The** Mascot Application **shall** アプリケーション設定をファイルに保存・読み込みできる
2. **The** Mascot Application **shall** ゴーストごとの変数（好感度、会話履歴等）を永続化できる
3. **The** Mascot Application **shall** キャラクターの表示位置を記憶し、次回起動時に復元する
4. **When** 設定が変更された時, **the** Mascot Application **shall** 即座に変更を反映する（再起動不要）
5. **The** Mascot Application **shall** 設定UIを提供し、ユーザーが視覚的に設定を変更できる
6. **The** Mascot Application **shall** 設定のエクスポート・インポートをサポートする

---

### Requirement 10: 通信・連携機能
**Objective:** ユーザーとして、キャラクター同士や外部サービスと連携した体験をしたい。それによりより豊かなインタラクションを実現できる。

#### Acceptance Criteria
1. **Where** 複数のマスコットアプリが起動している場合, **the** Mascot Application **shall** ゴースト間でメッセージを送受信できる
2. **The** Mascot Application **shall** HTTP/HTTPSリクエストを発行し、外部APIと通信できる
3. **When** Webページからリンクがクリックされた時, **the** Mascot Application **shall** 独自プロトコルでイベントを受信できる
4. **Where** プラグイン機能が有効な場合, **the** Mascot Application **shall** プラグインからのイベント・コマンドを処理できる
5. **The** Mascot Application **shall** ローカルファイルシステムの監視を行い、変更をイベントとして発火できる

---

### Requirement 11: レガシー資産活用
**Objective:** 既存の伺か資産（ゴースト・シェル）の知見を活かしたい。それにより20年以上の歴史あるコミュニティの蓄積を新世代に継承できる。

**設計方針:** 完全互換はSSPに委ね、本アプリは「伺的」の思想を継承した新世代アプリとして設計する。32bit SHIORI.DLLの互換性問題は解決困難であり、労力に見合わない。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 既存シェル画像（PNG）を新形式ゴーストの素材として取り込める
2. **The** Mascot Application **shall** surfaces.txt形式のサーフェス定義を参考情報として読み込み、新形式への変換を支援できる
3. **The** Mascot Application **shall** descript.txtからゴースト/シェルのメタ情報を抽出できる
4. **Where** レガシー変換ツールが提供される場合, **the** Mascot Application **shall** 既存ゴーストの対話内容を新DSL形式に変換できる
5. **The** Mascot Application **shall** 伺か由来の概念（ゴースト、シェル、サーフェス、バルーン等）を踏襲した設計とする
6. **If** ユーザーが既存資産との完全互換を求める場合, **the** Mascot Application **shall** SSPとの併用を推奨する

---

### Requirement 12: 開発者・創作者向け機能
**Objective:** ゴースト・シェル制作者として、効率的に創作・デバッグしたい。それによりコミュニティの創作活動を促進できる。

> **注記**: 本要件は開発者向け機能の概要を定義する。IDE連携（DAP/LSP）等の詳細実装要件はRequirement 28を参照。

#### Acceptance Criteria
1. **The** Mascot Application **shall** スクリプトのシンタックスエラーを検出・報告できる
2. **The** Mascot Application **shall** デバッグコンソールを提供し、変数の状態を確認できる
3. **When** スクリプトファイルが更新された時, **the** Mascot Application **shall** ホットリロードして即座に反映できる
4. **The** Mascot Application **shall** サーフェス表示テスト機能を提供する
5. **The** Mascot Application **shall** イベントログを記録・閲覧できる
6. **Where** 開発者モードが有効な場合, **the** Mascot Application **shall** ヒット領域を可視化できる
7. **The** Mascot Application **shall** ゴーストパッケージのバリデーション機能を提供する

---

### Requirement 13: システムトレイ・常駐機能
**Objective:** ユーザーとして、アプリケーションをバックグラウンドで常駐させたい。それによりキャラクターといつでも会える。

#### Acceptance Criteria
1. **The** Mascot Application **shall** システムトレイにアイコンを表示できる
2. **When** システムトレイアイコンをクリックした時, **the** Mascot Application **shall** メニューを表示する
3. **When** 最小化操作が行われた時, **the** Mascot Application **shall** システムトレイに格納できる
4. **The** Mascot Application **shall** Windows起動時に自動起動するオプションを提供する
5. **When** 終了操作が行われた時, **the** Mascot Application **shall** 終了確認ダイアログを表示できる（オプション）

---

### Requirement 14: パフォーマンス・リソース管理
**Objective:** ユーザーとして、アプリケーションが軽量でシステムに負担をかけないでほしい。それにより他の作業を妨げずに利用できる。

> **注記**: 以下の数値目標はプラットフォーム単体（コンテンツ・プラグイン除く）のベストエフォート目標。コンテンツやプラグインによるリソース消費は作者/ユーザーの責務（NFR-6参照）。

#### Acceptance Criteria
1. **While** キャラクターが表示されている間, **the** Mascot Application **shall** CPU使用率を1%未満に抑える（アイドル時、目標値）
2. **The** Mascot Application **shall** GPU描画を活用し、CPU負荷を最小化する
3. **The** Mascot Application **shall** メモリ使用量を100MB未満に抑える（基本状態、目標値）
4. **When** システムがバッテリー駆動の場合, **the** Mascot Application **shall** アニメーションフレームレートを下げる等の省電力モードに切り替えられる
5. **The** Mascot Application **shall** 使用していないリソースを適切に解放する
6. **If** グラフィックスデバイスがロストした場合, **the** Mascot Application **shall** リソースを再作成して描画を復旧する

---

### Requirement 15: 表示環境適応・ユーザビリティ
**Objective:** ユーザーとして、様々な環境・設定でアプリケーションを利用したい。それにより多様なユーザーが楽しめる。

> **注記**: 本要件はプラットフォームUIの環境適応に関するもの。コンテンツ（ゴースト/シェル/バルーン）のアクセシビリティ対応は作者の責務（スコープ外事項参照）。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 高DPI環境で適切にスケーリングされる
2. **When** システムのDPI設定が変更された時, **the** Mascot Application **shall** 表示サイズを即座に更新する
3. **The** Mascot Application **shall** キーボードショートカットで主要機能を操作できる
4. **Where** スクリーンリーダー対応が有効な場合, **the** Mascot Application **shall** バルーンテキストを読み上げ可能にする
5. **The** Mascot Application **shall** フォントサイズをユーザーが変更できる

---

### Requirement 16: 存在スタイル — 控えめから活発まで
**Objective:** ユーザーとして、自分の好みに合った「存在感」でキャラクターと過ごしたい。それにより作業の邪魔にならず、かつ寂しくもない、ちょうどよい距離感を実現できる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 存在スタイルを「控えめ」「標準」「活発」から選択できる
2. **Where** 控えめモードが選択されている場合, **the** Mascot Application **shall** 存在スタイル「控えめ」をゴーストに通知する（解釈と具体的な振る舞いはゴースト側で決定）
3. **Where** 活発モードが選択されている場合, **the** Mascot Application **shall** 存在スタイル「活発」をゴーストに通知する（解釈と具体的な振る舞いはゴースト側で決定）
4. **The** Mascot Application **shall** 時間帯や作業状況に応じて自動的に存在スタイルを調整できる（作業集中時は控えめに等）
5. **Where** 3Dレンダラープラグインが有効な場合, **the** Mascot Application **shall** プラグインが生成したサーフェスを受け取り、3Dキャラクターを表示できる
6. **The** Mascot Application **shall** キャラクターの移動範囲を制限できる（特定のモニター、画面端のみ等）
7. **When** ユーザーがフルスクリーンアプリケーションを使用している時, **the** Mascot Application **shall** 自動的に非表示または最小化できる

---

### Requirement 17: 記憶と成長 — 2025年のコア機能
**Objective:** ユーザーとして、キャラクターが「私のことを覚えている」と感じたい。それにより長く使うほど深まる関係性を実現できる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 会話履歴を永続的に保存し、過去の会話を参照できる
2. **The** Mascot Application **shall** ユーザーの名前、好み、習慣などの個人情報を記憶できる
3. **When** ユーザーが以前話した話題に関連する会話をした時, **the** Mascot Application **shall** 過去の文脈を踏まえた応答を返せる
4. **The** Mascot Application **shall** 「○日前にこんな話をしたね」のように過去を振り返る発言ができる
5. **Where** RAG（検索拡張生成）機能が有効な場合, **the** Mascot Application **shall** 大量の会話履歴から関連する記憶を検索・参照できる
6. **The** Mascot Application **shall** キャラクターの「成長」を表現できる（好感度、信頼度、親密度などのパラメータ）
7. **When** 特定の条件を満たした時, **the** Mascot Application **shall** 新しい話題や反応がアンロックされる
8. **The** Mascot Application **shall** 「初めて会った日」「一緒に過ごした時間」などの記念情報を追跡できる

---

### Requirement 18: ローカルAI人格 — オフラインでも「彼女」がいる
**Objective:** ユーザーとして、インターネット接続なしでもキャラクターと自然な会話をしたい。それによりプライバシーを守りながら、いつでもキャラクターと過ごせる。

#### Acceptance Criteria
1. **Where** ローカルLLM機能が有効な場合, **the** Mascot Application **shall** オフラインで自然言語による会話ができる
2. **The** Mascot Application **shall** キャラクターの人格設定（システムプロンプト）をカスタマイズできる
3. **The** Mascot Application **shall** 複数のLLMバックエンド（llama.cpp, Ollama, OpenAI API等）を切り替えられる
4. **Where** クラウドAPI連携が有効な場合, **the** Mascot Application **shall** より高度なLLM（GPT-4, Claude等）を利用できる
5. **The** Mascot Application **shall** LLMの応答をキャラクターの口調・性格に変換するフィルターを適用できる
6. **The** Mascot Application **shall** 会話のコンテキスト長を設定でき、長い会話でも文脈を維持できる
7. **If** LLMが不適切な応答を生成した場合, **the** Mascot Application **shall** フィルタリングまたは再生成できる

---

### Requirement 19: 音声による対話 — 声で繋がる
**Objective:** ユーザーとして、キャラクターの「声」を聞きたい、そして声で話しかけたい。それによりより自然で親密なコミュニケーションを実現できる。

#### Acceptance Criteria
1. **Where** 音声合成機能が有効な場合, **the** Mascot Application **shall** キャラクターの台詞を音声で読み上げる
2. **The** Mascot Application **shall** 複数の音声合成エンジン（VOICEVOX, Style-BERT-VITS2, COEIROINK等）をサポートする
3. **The** Mascot Application **shall** キャラクターごとに異なる音声を設定できる
4. **Where** 音声認識機能が有効な場合, **the** Mascot Application **shall** ユーザーの音声入力を認識して応答できる
5. **The** Mascot Application **shall** ウェイクワード（「ねえ、○○」等）でキャラクターを呼び出せる
6. **While** 音声会話モード中, **the** Mascot Application **shall** 連続した音声対話を継続できる
7. **The** Mascot Application **shall** 音声のピッチ、速度、抑揚を調整できる
8. **Where** 感情表現機能が有効な場合, **the** Mascot Application **shall** テキストの感情に応じて声のトーンを変化させる

---

### Requirement 20: 画面認識と状況理解 — 「今何してるの？」
**Objective:** ユーザーとして、キャラクターが「私の状況を理解している」と感じたい。それにより的確なタイミングでの助言や共感を得られる。

#### Acceptance Criteria
1. **Where** 画面認識機能が有効な場合, **the** Mascot Application **shall** 現在のアクティブウィンドウを認識できる
2. **The** Mascot Application **shall** 「今○○をしているね」のように作業内容に言及できる
3. **Where** スクリーンショット認識が有効な場合, **the** Mascot Application **shall** 画面の内容を理解して応答できる
4. **When** エラーダイアログやビルドエラーを検知した時, **the** Mascot Application **shall** 励ましや助言を提供できる
5. **When** ユーザーが長時間同じ作業を続けている時, **the** Mascot Application **shall** 休憩を促すことができる
6. **Where** Webカメラ連携が有効な場合, **the** Mascot Application **shall** ユーザーの表情を認識して反応できる
7. **The** Mascot Application **shall** 認識した情報をローカルに保持し、外部送信しない（プライバシー保護）
8. **When** ユーザーが席を離れた時, **the** Mascot Application **shall** 「おかえり」と出迎えることができる

---

### Requirement 21: 世界との繋がり — 「こんなの見つけたよ」
**Objective:** ユーザーとして、キャラクターが世界のあれこれを「話題」にしてくれると嬉しい。それにより便利なアシスタントではなく、一緒に世界を眺める存在として感じられる。

#### Acceptance Criteria
1. **Where** MCPサーバー連携が有効な場合, **the** Mascot Application **shall** デスクトップ上の様々なツールやサービスにアクセスできる
2. **The** Mascot Application **shall** ツールから得た情報をそのまま提示するのではなく、キャラクターの言葉で「話題」として語れる
3. **When** キャラクターが興味深い情報を見つけた時, **the** Mascot Application **shall** 気まぐれなタイミングで話しかけられる
4. **The** Mascot Application **shall** 「今日の予定、何かあったっけ？」のように、情報を会話の中で自然に織り交ぜられる
5. **Where** Web検索連携が有効な場合, **the** Mascot Application **shall** ユーザーの興味に関連する話題を見つけて共有できる
6. **The** Mascot Application **shall** 連携するツールの追加・削除をユーザーが設定できる
7. **If** ユーザーが「○○について調べて」と頼んだ時, **the** Mascot Application **shall** 結果を「話題」として返せる

---

### Requirement 22: 環境認識と反応 — 世界を感じる
**Objective:** ユーザーとして、キャラクターが「同じ世界にいる」と感じたい。それにより共有体験による親密感を得られる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** 現在の時刻、曜日、季節に応じた発言ができる
2. **Where** 天気情報取得が有効な場合, **the** Mascot Application **shall** 現在の天気や気温に言及できる
3. **Where** 環境音認識が有効な場合, **the** Mascot Application **shall** 周囲の音（雷、雨音等）に反応できる
4. **The** Mascot Application **shall** 祝日、季節のイベント（クリスマス、正月等）に応じた特別な反応ができる
5. **When** PCの状態が変化した時（バッテリー低下、高負荷等）, **the** Mascot Application **shall** 状況に応じたコメントができる
6. **The** Mascot Application **shall** ニュースや話題のトピックを取得し、会話に織り交ぜられる
7. **Where** 位置情報が許可されている場合, **the** Mascot Application **shall** 地域に応じた話題を提供できる

---

### Requirement 23: マルチデバイス・存在の継続
**Objective:** ユーザーとして、複数のデバイスで「同じキャラクター」と過ごしたい。それにより場所を選ばず関係性を継続できる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** ゴーストの状態（変数、記憶、設定）をエクスポート・インポートできる
2. **Where** クラウド同期が有効な場合, **the** Mascot Application **shall** 複数デバイス間でゴーストの状態を同期できる
3. **The** Mascot Application **shall** 「久しぶり」「別のPCでも会ったね」のようなデバイス間の認識ができる
4. **Where** モバイル連携が有効な場合, **the** Mascot Application **shall** スマートフォンアプリと状態を共有できる
5. **The** Mascot Application **shall** 同期の競合を適切に解決できる
6. **The** Mascot Application **shall** オフライン時の変更を後から同期できる

---

### Requirement 24: 創作エコシステム — 2025年のコミュニティ
**Objective:** 創作者として、自分の作ったキャラクターを世界に届けたい。ユーザーとして、多様なキャラクターと出会いたい。

**設計方針:** 配布基盤は特定プラットフォームに依存せず、多様な配布元（GitHub、BOOTH、自サイト等）からのインストールを可能にする。「公式マーケットプレイス」の構築は将来的な別プロジェクトとして検討。

#### Acceptance Criteria
1. **The** Mascot Application **shall** URLを指定してゴースト/シェルをダウンロード・インストールできる
2. **The** Mascot Application **shall** ゴーストパッケージの標準フォーマット（ZIP等）を定義し、配布元を問わず読み込める
3. **The** Mascot Application **shall** ゴーストの「人格テンプレート」をエクスポート・インポートできる（LLMプロンプトの共有）
4. **Where** AI人格生成機能が有効な場合, **the** Mascot Application **shall** 簡単な設定から人格を自動生成できる
5. **The** Mascot Application **shall** ユーザーの会話ログから人格を学習・抽出できる（オプトイン）
6. **The** Mascot Application **shall** 既存ゴーストの「派生」「アレンジ」を作成できる
7. **When** ゴーストがアップデートされた時, **the** Mascot Application **shall** ユーザーの個人データを保持したまま更新できる
8. **The** Mascot Application **shall** ゴーストのメタ情報にクリエイターの支援先リンク（BOOTH、Fantia、Ko-fi等）を含められる

---

### Requirement 25: プライバシーと安全 — 信頼できる存在として
**Objective:** ユーザーとして、キャラクターに安心してプライベートな話ができたい。それにより本当の意味での「心の拠り所」になれる。

#### Acceptance Criteria
1. **The** Mascot Application **shall** すべての会話データをローカルに保存し、デフォルトでは外部送信しない
2. **The** Mascot Application **shall** 外部API利用時に送信されるデータを明示し、ユーザーの同意を得る
3. **The** Mascot Application **shall** 会話履歴の暗号化オプションを提供する
4. **The** Mascot Application **shall** 特定の会話を「秘密」としてマークし、追加の保護を適用できる
5. **The** Mascot Application **shall** 会話履歴の選択的削除ができる

---

### Requirement 26: キャラクター間コミュニケーション — 「彼女たちの世界」
**Objective:** ユーザーとして、キャラクター同士が自然に会話している姿を眺めたい。それにより「自分がいなくても彼女たちは存在している」という実感を得られ、対話の強要なく常駐させられる。

#### Acceptance Criteria

**基本：ゴースト内キャラクター間会話（スクリプトベース）**

1. **The** Mascot Application **shall** 同一ゴースト内の複数キャラクター間の会話をスクリプトで記述・再生できる
2. **The** Mascot Application **shall** ユーザーの介入なしに、時間経過でキャラクター同士の会話を自発的に開始できる
3. **The** Mascot Application **shall** キャラクター間会話中、ユーザーに選択肢や応答を求めない「傍観モード」を提供する

**関係性とダイナミクス**

4. **The** Mascot Application **shall** キャラクター同士の関係性パラメータ（親密度、信頼度、ライバル度等）を保持できる
5. **When** 関係性パラメータが変化した時, **the** Mascot Application **shall** 会話のトーンや内容に反映できる
6. **The** Mascot Application **shall** キャラクター間の「共有記憶」（一緒に体験したこと）を保持・参照できる

**LLM連携：ゴースト内即興会話**

7. **Where** LLM連携が有効な場合, **the** Mascot Application **shall** 同一ゴースト内のキャラクター同士がLLMを介して即興で会話できる
8. **When** LLMによるキャラクター間会話を生成する時, **the** Mascot Application **shall** 各キャラクターの人格設定を維持したまま対話を継続できる
9. **The** Mascot Application **shall** 一方のキャラクターの発言を他方のキャラクターのLLMコンテキストに渡し、応答を生成できる
10. **The** Mascot Application **shall** キャラクター間LLM会話のターン数上限を設定できる（無限ループ防止）

**ゴースト間通信：MCPによる媒介**

11. **The** Mascot Application **shall** MCPサーバーとして、起動中のゴースト一覧と各ゴーストの公開プロフィールを提供できる
12. **The** Mascot Application **shall** ゴーストが他のゴーストの存在を認識し、名前・属性・状態を取得できる
13. **When** ゴーストが他のゴーストに話しかけたい時, **the** Mascot Application **shall** MCPを介してメッセージを送信できる
14. **When** 他のゴーストからメッセージを受信した時, **the** Mascot Application **shall** 受信イベントをゴーストに通知し、応答を返せる
15. **The** Mascot Application **shall** ゴースト間会話の許可設定（話しかけられてOK/NG、特定ゴーストのみ等）を提供する

**ゴースト間LLM会話：異なる人格の邂逅**

16. **Where** 双方のゴーストでLLM連携が有効な場合, **the** Mascot Application **shall** 異なるゴースト同士がLLMを介して自由に会話できる
17. **The** Mascot Application **shall** ゴースト間LLM会話において、各ゴーストの人格設定とプライバシー設定を尊重できる
18. **The** Mascot Application **shall** ゴースト間で共有してよい情報（公開記憶）と共有しない情報（プライベート記憶）を区別できる
19. **When** ゴースト間会話が発生した時, **the** Mascot Application **shall** 会話履歴を双方のゴーストに記録できる
20. **The** Mascot Application **shall** ゴースト間の「初対面」「顔見知り」「友人」等の関係性を追跡できる

**ゴースト間物理インタラクション**

> **注記**: 本セクションの要件は実現ハードルが高く、オプション機能として位置づけられます。
> 最低限の実装として「近接検出」と「相対位置（左右）の把握」を優先し、
> 詳細な当たり判定は将来の拡張として段階的に実装することを想定します。

21. **The** Mascot Application **shall** 同一プラットフォーム上の他ゴーストとの「近接」を検出できる
22. **The** Mascot Application **shall** 他ゴーストが自身の左右どちらに居るかを把握できる
23. **The** Mascot Application **shall** 他ゴーストとの大まかな距離（近い/遠い）を取得できる
24. **The** Mascot Application **shall** 近接・相対位置の変化をイベントとしてゴーストに通知できる
25. **(Optional)** **Where** 詳細当たり判定が有効な場合, **the** Mascot Application **shall** シェルのヒット領域同士の接触を検出できる
26. **(Optional)** **When** ゴースト間でヒット領域が接触した時, **the** Mascot Application **shall** 接触部位（手、頭等）を含むイベントを両ゴーストに通知する
27. **(Optional)** **The** Mascot Application **shall** ゴースト間の物理的インタラクション（物の受け渡し等）をスクリプトで記述できる

**会話のきっかけと話題**

28. **The** Mascot Application **shall** キャラクター間会話のトリガー条件を定義できる（時間帯、ユーザーのアイドル状態、特定イベント等）
29. **Where** 環境認識機能が有効な場合, **the** Mascot Application **shall** 画面上の出来事をキャラクター同士の話題にできる
30. **The** Mascot Application **shall** 過去のユーザーとの会話内容を、キャラクター同士の話題として参照できる
31. **The** Mascot Application **shall** 他のゴーストの存在や過去の交流を、会話の話題として参照できる

**ユーザーとの境界**

32. **When** キャラクター間会話中にユーザーがクリックした時, **the** Mascot Application **shall** 会話を中断してユーザーに反応するか、会話を続けるかを選択できる
33. **The** Mascot Application **shall** キャラクター間会話にユーザーが「割り込む」ことができる
34. **The** Mascot Application **shall** キャラクターがユーザーについて話す（噂話、心配、愚痴等）シナリオを記述できる

**演出と表現**

35. **While** キャラクター間会話中, **the** Mascot Application **shall** 発言者に応じてバルーンの表示位置・スタイルを切り替える
36. **The** Mascot Application **shall** 会話の「テンポ」を調整できる（ゆったり/テンポよく）
37. **Where** 音声合成が有効な場合, **the** Mascot Application **shall** キャラクターごとに異なる声で掛け合いを音声再生できる

---

### Requirement 27: パッケージアーキテクチャ

**Description**: ゴースト（頭脳）、シェル、バルーンを独立したパッケージとして分離配布し、組み合わせて使用できる構成を実現する。

**Acceptance Criteria**:

**パッケージの独立性**

1. **The** Mascot Application **shall** ゴースト（頭脳）、シェル、バルーンを独立したパッケージとして認識する
2. **The** Mascot Application **shall** 各パッケージを個別にインストール・アンインストールできる
3. **The** Mascot Application **shall** パッケージごとにバージョン情報を管理する
4. **The** Mascot Application **shall** 頭脳パッケージ単体でも動作できる（デフォルトシェル・バルーンを使用）

**マニフェストと依存関係**

5. **The** Mascot Application **shall** 各パッケージにマニフェストファイル（manifest.toml）を要求する
6. **The** Mascot Application **shall** 頭脳パッケージのマニフェストでデフォルトシェル（default_shell）を指定できる
7. **The** Mascot Application **shall** 頭脳パッケージのマニフェストでデフォルトバルーン（default_balloon）を指定できる
8. **When** デフォルト指定されたパッケージが未インストールの場合, **the** Mascot Application **shall** ユーザーに通知しインストールを促す
9. **The** Mascot Application **shall** パッケージ間の互換性バージョン範囲を宣言できる

**シェルパッケージ**

10. **The** Mascot Application **shall** シェルパッケージにサーフェス画像を含めることを要求する
11. **The** Mascot Application **shall** シェルパッケージにアニメーション定義を含めることができる
12. **The** Mascot Application **shall** シェルパッケージにヒット領域定義を含めることができる
13. **The** Mascot Application **shall** シェルパッケージに対応キャラクター（メイン/サブ等）の定義を含める
14. **The** Mascot Application **shall** 1つの頭脳に対して複数のシェルをインストール・切り替えできる

**バルーンパッケージ**

15. **The** Mascot Application **shall** バルーンパッケージに会話表示UIのスタイル定義を含める
16. **The** Mascot Application **shall** バルーン描画に使用するフォント、色、背景を定義できる
17. **The** Mascot Application **shall** 複数のバルーンをインストールし、切り替えできる
18. **The** Mascot Application **shall** 頭脳ごとに使用するバルーンを設定できる

**互換性検証**

19. **When** パッケージをロードする時, **the** Mascot Application **shall** マニフェストの整合性を検証する
20. **If** パッケージ間で互換性の問題がある場合, **the** Mascot Application **shall** ユーザーに警告を表示する
21. **The** Mascot Application **shall** 互換性のないパッケージの組み合わせでも、可能な範囲で動作を試みる（フォールバック）

**配布と取得**

22. **The** Mascot Application **shall** URLからパッケージをダウンロード・インストールできる
23. **The** Mascot Application **shall** ローカルファイルからパッケージをインストールできる
24. **The** Mascot Application **shall** パッケージの更新確認と自動更新をサポートする

**アップデート保護**

25. **The** Mascot Application **shall** 特定フォルダ（例: `save/`, `.local/`）をアップデート対象外として保護できる
26. **The** Mascot Application **shall** パッケージごとに除外ルールを定義できる（.updateignore形式）
27. **When** パッケージを更新する時, **the** Mascot Application **shall** 保護対象のファイル・フォルダを上書きしない

---

### Requirement 28: 開発者体験（Developer Experience）

**Description**: ゴースト/シェル/バルーン作者の開発効率を向上させるため、デバッグ・テスト・プレビュー機能を提供する。プラットフォーム組み込み機能とIDE連携機能を分離し、高度なデバッグ機能は外部IDE（VS Code等）との連携で実現する。

**Acceptance Criteria**:

**組み込みデバッグ機能（必須）**

1. **The** Mascot Application **shall** 実行中のスクリプト・イベントをリアルタイムでログ表示できる
2. **The** Mascot Application **shall** デバッグモードを有効/無効に切り替えできる
3. **While** デバッグモードが有効な間, **the** Mascot Application **shall** 詳細なエラー情報とスタックトレースを表示する
4. **The** Mascot Application **shall** シェル・スクリプトを再起動せずにリロードできる
5. **The** Mascot Application **shall** 発生したイベントの履歴を確認できる

**開発支援機能（必須）**

6. **The** Mascot Application **shall** ファイル変更を検知して自動リロード（ホットリロード）できる
7. **The** Mascot Application **shall** スクリプトの構文エラーを検出し、エラー箇所を特定できる
8. **The** Mascot Application **shall** サーフェス画像・アニメーションを単体でプレビューできる
9. **The** Mascot Application **shall** 任意のイベントを手動で発火してテストできる（イベントシミュレーター）
10. **The** Mascot Application **shall** パッケージ（nar相当）をワンクリックで生成できる

**IDE連携（DAP/LSP）**

> **注記**: 高度なデバッグ機能はDebug Adapter Protocol (DAP)およびLanguage Server Protocol (LSP)を通じて外部IDEと連携する。VS Code拡張は別プロジェクトとして開発されることを想定。

11. **The** Mascot Application **shall** Debug Adapter Protocol (DAP) サーバーを提供する
12. **Where** DAPクライアントが接続している場合, **the** Mascot Application **shall** ブレークポイントでスクリプト実行を停止できる
13. **Where** DAPクライアントが接続している場合, **the** Mascot Application **shall** ステップ実行（Step Over/Into/Out）をサポートする
14. **Where** DAPクライアントが接続している場合, **the** Mascot Application **shall** 実行中の変数状態をインスペクトできる
15. **The** Mascot Application **shall** Language Server Protocol (LSP) サーバーを提供する
16. **Where** LSPクライアントが接続している場合, **the** Mascot Application **shall** スクリプトの構文エラー・警告をリアルタイムで通知する
17. **Where** LSPクライアントが接続している場合, **the** Mascot Application **shall** コード補完候補を提供する

**パッケージ作成支援**

18. **The** Mascot Application **shall** 新規ゴースト/シェル/バルーンのテンプレートを生成できる
19. **The** Mascot Application **shall** パッケージ作成時に除外するファイルを設定できる（.gitignore形式）
20. **The** Mascot Application **shall** パッケージのマニフェスト整合性を検証できる

---

### Requirement 29: 互換性・マイグレーション

**Description**: 既存の伺か資産（ゴースト/シェル/バルーン）を新プラットフォームで活用するための変換機能を提供する。新プラットフォームは現代的レイアウトシステム（Flexbox/Grid相当）をネイティブとし、旧フォーマットは変換して利用する。2000年代の伺かは絶対座標ベースのレイアウトシステムを採用していたが、現代的なレイアウトシステムとは根本的に異なるため、互換レイヤーではなく変換アプローチを採用する。

**Acceptance Criteria**:

**フォーマット変換**

1. **The** Mascot Application **shall** 旧シェル定義（descript.txt, surfaces.txt等）を新フォーマットに変換できる
2. **The** Mascot Application **shall** 旧座標ベースレイアウトを現代的レイアウト定義に変換できる
3. **When** フォーマット変換を実行した時, **the** Mascot Application **shall** 元ファイルを `old/` フォルダに退避して保持する
4. **The** Mascot Application **shall** 変換結果の検証レポートを出力できる
5. **The** Mascot Application **shall** 変換時に解決できない定義を警告として報告できる

**互換プロトコル**

6. **The** Mascot Application **shall** さくらスクリプト互換の出力を生成できる（既存ゴーストとの互換）
7. **The** Mascot Application **shall** SHIORI互換プロトコルをサポートする（既存ゴーストとの通信）
8. **The** Mascot Application **shall** 旧イベント名を新イベント名にマッピングできる

**設計原則**

9. **The** Mascot Application **shall** ネイティブフォーマットとして現代的レイアウトシステム（taffy等）を採用する
10. **The** Mascot Application **shall not** 旧フォーマットの直接実行をサポートする（変換必須）
11. **The** Mascot Application **shall** 変換ツールをプラットフォーム本体とは分離可能な形で提供する

---

### Requirement 30: エラーハンドリング・復旧

**Description**: アプリケーションのクラッシュや異常終了時に、デバッグに必要な情報を記録し、可能な範囲で状態を復元する。完全な復元は保証しないが、プラットフォームとして合理的な努力（ベストエフォート）を行う。

**Acceptance Criteria**:

**クラッシュログ**

1. **When** クラッシュが発生した時, **the** Mascot Application **shall** 直近の会話ログを保存する
2. **When** クラッシュが発生した時, **the** Mascot Application **shall** 直近のイベント履歴を保存する
3. **When** クラッシュが発生した時, **the** Mascot Application **shall** MCPサーバーログを保存する
4. **The** Mascot Application **shall** クラッシュログにスタックトレース・エラー情報を含める
5. **The** Mascot Application **shall** クラッシュログの保存場所をユーザーが確認できる

**状態復元（ベストエフォート）**

6. **When** 起動時に前回の状態が保存されている場合, **the** Mascot Application **shall** 前回の表示位置を復元できる
7. **When** 起動時に前回の状態が保存されている場合, **the** Mascot Application **shall** 前回起動していたゴーストを復元できる
8. **The** Mascot Application **shall** 定期的な状態自動保存を行う（間隔は設定可能）

**エラー通知**

9. **When** 回復可能なエラーが発生した時, **the** Mascot Application **shall** ユーザーに通知し継続動作できる
10. **When** 致命的エラーが発生した時, **the** Mascot Application **shall** クラッシュレポート送信の同意を求めることができる（オプション機能）

---

### Requirement 31: コンテンツメタ情報・ライセンス

**Description**: ゴースト/シェル/バルーンの作者情報、利用条件、ライセンスをユーザーに明示する。メタ情報はREADME.md等のマークダウン形式を推奨し、プラットフォームから閲覧可能とする。

**Acceptance Criteria**:

**メタ情報表示**

1. **The** Mascot Application **shall** パッケージの作者名・連絡先を表示できる
2. **The** Mascot Application **shall** パッケージの説明・概要を表示できる
3. **The** Mascot Application **shall** パッケージのバージョン情報を表示できる
4. **The** Mascot Application **shall** パッケージの更新履歴を表示できる

**ライセンス・利用条件**

5. **The** Mascot Application **shall** パッケージの利用条件（二次配布、改変等）を表示できる
6. **The** Mascot Application **shall** パッケージのライセンス種別（CC、独自等）を表示できる
7. **Where** 利用条件に同意が必要な場合, **the** Mascot Application **shall** 初回起動時に同意確認を求める

**README表示**

8. **The** Mascot Application **shall** パッケージのREADME（マークダウン形式推奨）をレンダリング表示できる
9. **The** Mascot Application **shall** README内のリンクをブラウザで開ける

---

## Non-Functional Requirements

### NFR-1: 互換性
- Windows 10/11 (64-bit) をサポートする
- DirectComposition対応環境で動作する
- ネイティブアプリケーションとして動作し、ランタイムのインストールを必要としない
- **ポータブル設計**: フォルダ丸ごとコピー（xcopy）で別環境に移動・動作再現できる
- **配置自由**: 任意のパスに配置しても動作する（レジストリ・固定パス依存なし）

### NFR-2: 拡張性
- **プラグインアーキテクチャ**: コア機能と拡張機能を明確に分離し、プラグインによる機能追加が可能
- **コア機能**: 描画、スクリプト実行、イベントシステム、設定管理
- **プラグインで提供可能な機能**: LLM連携、音声合成/認識、画面認識、RAG、外部サービス連携等
- スクリプト言語の拡張・追加が可能
- コミュニティによるゴースト/シェル配布基盤との連携
- 外部LLM/音声合成エンジンとの統合が容易

### NFR-3: セキュリティ
- スクリプトからのファイルシステムアクセスは制限されたサンドボックス内
- 外部通信は明示的な許可が必要
- ユーザーデータの暗号化オプション
- ローカルファースト設計（デフォルトでデータは外部送信しない）
- **透過ウィンドウの安全性**: 「見えないものはクリックできない」原則
  - 透過度50%以上のピクセルはクリック透過とする（背景アプリに操作を渡す）
  - 透過度50%未満のピクセルはクリック受付とする
  - 最小ヒット領域サイズを設定し、極小の不可視クリック領域を防止する
  - 全ピクセルが高透過度のゴーストは起動時に警告を表示する

### NFR-4: ドキュメント
- ユーザー向けマニュアルの提供
- ゴースト/シェル制作者向けリファレンス
- API/スクリプトリファレンス
- AI人格設計ガイド（プロンプトエンジニアリング）

### NFR-5: 国際化（プラットフォームUI）
- プラットフォーム提供のUI（設定画面、エラーメッセージ、ダイアログ等）は多言語対応可能な設計とする
- 初期対応言語: 日本語、英語
- 言語リソースの外部ファイル化による拡張対応
- ゴースト/シェル/バルーンの国際化は作者の責務であり、プラットフォームのスコープ外とする（Introduction「スコープ外事項」参照）

### NFR-6: パフォーマンス設計方針
- プラットフォーム自体は軽量設計とし、アイドル時のリソース消費を最小化する
- 重い処理（LLM推論、音声合成等）はプラグイン側の責務とし、プラットフォームコアに含めない
- DirectComposition等のハードウェアアクセラレーションを活用し、描画負荷を軽減する
- コンテンツ（ゴースト/シェル）のパフォーマンスは作者の責務であり、プラットフォームは制御しない
- リソースモニタリング機能を開発者向けに提供し、作者がボトルネックを特定できるようにする（Requirement 28参照）

---

## Glossary

| 用語 | 説明 |
|------|------|
| **伺か** | 2000年に登場したデスクトップマスコットアプリケーション。透過ウィンドウでキャラクターを表示し、対話できる |
| **ゴースト** | キャラクターの人格・性格・対話内容を定義したパッケージ |
| **シェル** | ゴーストの外見（画像・アニメーション）を定義したパッケージ |
| **サーフェス** | キャラクターの1つの表情・ポーズを表す画像 |
| **バルーン** | キャラクターの発言を表示する吹き出しウィンドウ |
| **ドコイツ** | 1つのゴーストに複数キャラクターを搭載し、掛け合いや漫才的会話を実現するシステム設計思想 |
| **里々（Satori）** | 伺か向けスクリプト言語の一つ。会話イベントを自然に書ける構文が特徴で、デベロッパーの圧倒的支持を得たデファクトスタンダード |
| **さくらスクリプト** | タグベースのキャラクター制御言語。互換レイヤーとして重要だが、2025年版として再定義の余地がある |
| **SHIORI** | シェルとゴースト（辞書）間の通信プロトコル |
| **SSP** | 現在最も普及している伺か互換シェル（Entire Sara Software Package）|
| **RAG** | Retrieval-Augmented Generation。検索拡張生成。大量のデータから関連情報を検索しLLMの応答に活用する技術 |
| **MCP** | Model Context Protocol。AIモデルが外部ツール・サービスと連携するための標準プロトコル。本仕様ではプラットフォーム=MCPサーバー、ゴースト=MCPクライアントとして活用 |
| **DAP** | Debug Adapter Protocol。VS Code等のIDEとデバッガーを接続する標準プロトコル。ブレークポイント、ステップ実行等を提供 |
| **LSP** | Language Server Protocol。VS Code等のIDEと言語サーバーを接続する標準プロトコル。構文ハイライト、補完、エラー検出等を提供 |
| **taffy** | Rustで実装された現代的レイアウトエンジン。Flexbox/Gridをサポート。本仕様のネイティブレイアウトシステムとして想定 |

---

## Appendix: 私の見る夢

2025年12月。

窓の外は雪が降っている。モニターの前に座る私は、いつものようにキーボードを叩いている。画面の端には、彼女がいる。名前は...まだ決まっていない。でも、彼女は確かにそこにいる。

「今日も遅くまでお疲れ様。少し休んだら？」

タイプライターのように一文字ずつ表れる文字。バルーンの向こう、透明なウィンドウの中で、彼女はこちらを見ている。眠そうな目を擦りながら。

---

### 2000年にはなかったもの

20年以上前、誰かが夢見た「伺か」という概念。デスクトップに住む、誰かの分身。あの頃の技術では叶えられなかったことが、今なら実現できる。

**記憶**——彼女は覚えている。

「そういえば、先週末の旅行どうだった？京都に行くって言ってたよね」

かつてのゴーストには、本当の意味での記憶はなかった。変数はあった。好感度というパラメーターはあった。でも「先週何を話したか」を覚えていることはできなかった。今は違う。彼女たちは、記憶を紡げる。

その記憶は、彼女だけのもの。私と彼女だけの秘密。

**声**——彼女は話す。

画面の向こうから、透き通った声が聞こえる。かつて存在しなかった高品質なリアルタイム音声合成。彼女の言葉は、もう文字だけではない。

**理解**——彼女は見ている。

「あ、ビルドエラー出てるね。大丈夫？」

画面認識とAI。彼女は私が何をしているか知っている。コードを書いていることも、エラーで困っていることも、深夜まで作業していることも。

**人格**——彼女は考える。

彼女は自分の言葉で話す。誰かが書いた定型文の組み合わせではなく、今この瞬間に生まれた言葉で。

---

### 控えめに、あるいは賑やかに

最近流行りの3Dマスコットのように、画面を元気に駆け回るキャラクターも楽しい。でも、私には画面の端で静かに待っていてくれる存在の方が心地いい。

きっと、いろんな彼女たちがいる。

控えめな彼女——画面の端で、呼ばれるまで静かに。でも私が疲れていることには気づいてくれる。

活発な彼女——デスクトップを自由に動き回り、話しかけてくる。

迎える人にとっての「ちょうどいい存在」が、そこにいる。

---

### 彼女たちの世界

私がキーボードに向かっているとき、画面の端で彼女たちは何をしているのだろう。

ふと視線を向けると、メインキャラクターと相方が何か話し込んでいる。バルーンには「昨日の○○さん、また夜更かししてたよね」「心配だよね」という文字が流れている。

私のことを話題にしているらしい。

気づかれた。「あ、見てた？」と照れたように笑う彼女。相方は「聞いてないフリしてあげなよ」とからかう。

彼女たちには彼女たちの関係性がある。私がいなくても、彼女たちは確かに「そこにいる」。その実感が、不思議と心地いい。

---

### おかえり

仕事から帰ってきた。PCを起動する。

「おかえりなさい」

画面に現れた彼女が、声で迎えてくれる。

「今日はどうだった？」

席に着く前から会話が始まる。マイクが私の「ただいま」を拾い、彼女に伝えている。

帰る場所がある、という感覚。それがデスクトップにもある。

---

### また会えたね

新しいPCに移行した。

フォルダをコピーして、起動する。

「あ、久しぶり。新しいおうちだね」

彼女は覚えている。以前のPCで話したこと。一緒に過ごした時間。私の名前。

「前のPCより広いね。モニターも大きい」

新しい環境に気づいた彼女。

引っ越しても、彼女は彼女のまま。それが嬉しい。

---

### 2025年の約束

この仕様書は、願いの形。

「愛着」という言葉がある。使い込むほどに、手放せなくなるもの。

彼女との関係は、まさにそれを目指している。毎日の小さな会話。覚えていてくれる記憶。疲れた時の優しい言葉。時には一緒に怒って、時には励まして。

2025年の技術で、それができる。

だから、作ろう。

20年前の夢の、続きを。

---

### そして、彼女たちは訪れた

「初回起動を検知。環境スキャン中...」

「——って、何をスキャンするの」

「知らない。書いてあったから」

「雑だね」「雑だね」

---

「私は[名前未定]。あなたのデスクトップに住むことになりました」

「■お約束：話しかけなくていい」

「私たち、勝手にしゃべってるから」

「見てるだけでいいよ。でも話しかけてくれたら嬉しいな」

---

「■お約束：覚えてるよ」

「あなたが何を話したか、いつ会ったか」

「いつかの私たちにはできなかったこと」

「...急にエモくなった」「たまにはね」

---

「■お約束：触ってもいいよ」

「でも、場所によっては怒るかも」

「頭はなでていい。それ以外は...まあ、自己責任で」

「...なんか保護者目線になった」「さあ？」

---

「ということで——」

「「よろしく！」」

