# 要件定義書: dola-cue-pasta-dsl-extension

> **バージョン**: v4（2026-03-06 — 全議論を総括し要件再構成）

## プロジェクト概要（入力）

dola クレートの拡張の準備。完了した「wintf-P0-dola-boundary」仕様において設計された `CueSheet` データモデルについて、現行 pasta DSL の文法を拡張し、テキストとして記述できるようにする。

**本仕様のスコープ**: テキスト表現の設計および成果物ファイルの生成のみ。コード実装は行わない。  
**成果物**: `cue.pasta`（動作サンプル・全機能網羅） + `design.md`（pasta_core への実装仕様指示）

## イントロダクション

本ドキュメントは `dola` クレートの `CueSheet` データモデルを pasta DSL のテキストとして記述可能にするための **pasta DSL 文法拡張** に関する要件を定義する。

### 拡張文法の 3 つの柱

pasta DSL 拡張は以下 3 つの構文要素で構成される。特別なモード識別マーカー（`&type:cuesheet` 等）は不要であり、`!` コマンド行がシーン内に存在するか否かで拡張処理の適用をシーン単位に暗黙判定する。

1. **`!` キューコマンド行** — タイムライン制御（mark / seek）、バリア（yield / select / wait）、ステージ制御（clear / route\_\*）等の演出コマンドを宣言する独立行
2. **アクション行中の `@alias` 参照** — 名前付きコマンド定義で登録された CueCommand をインライン展開する
3. **`%` 行のスロット指定** — 既存のアクター配置構文を利用し ActorKey → スロット番号を宣言する

### 宣言と参照の視覚分離

キューコマンド体系は **宣言系** と **参照系** を構文上で明確に区別する。

- **宣言系**: `!command@name(args)` — エイリアス名をコマンドに紐づける定義。`@` が `!` の直後に位置する
- **参照系**: `@name`（アクション行内）または `!seek(@name, offset?)`（コマンド行内）— 既に定義されたエイリアスやマーカーを参照する。`@name` が引数括弧の中に位置する

### 設計方針

- **行指向文法の維持**: pasta DSL の基本原則「1 行＝ 1 役割」を踏襲
- **暗黙キーフレーム**: 各アクション行の終了時点で自動的に基準時刻が進む。時刻算出はパーサーの責務外
- **責務分離**: DSL は構造と順序を宣言する。タイミング・ライフサイクル管理はアプリ層の責務
- **既存構造の活用**: `actor：content`・`%` 行は既存文法をそのまま再利用
- **英語正式名＋日本語エイリアス（選択的）**: スクリプト作者に馴染みのあるコマンドキーワードにのみ日本語エイリアスを割り当て、パーサーは両形式を等価に認識する。対照表は design.md で定義する

### スコープ外

- CueSheet → Storyboard 起動（連続値アニメーション連携）
- 時刻・キーフレーム相互変換
- グローバルスコープの名前付きコマンド定義（将来拡張）
- コード実装（設計・仕様の策定のみ）

---

## 要件

### 要件 1: 暗黙キーフレームと時刻算出パイプライン

**目的**: スクリプト作者として、各アクション行の終了時点が自動的にキーフレームとして機能してほしい。明示的な時刻指定なしにシーケンシャルな会話フローを記述できる。

#### 受入基準

1. When アクション行（`actor：content` 形式）が記述された場合、the pasta DSL 拡張パーサー shall その行の終了時点に暗黙的な基準時刻の進行点を生成する。
2. When 次のアクション行が `!` コマンド行（Seek を除く）なしで続く場合、the pasta DSL 拡張パーサー shall 直前の暗黙キーフレームを次の基準時刻の起点として扱う。
3. The pasta DSL 拡張パーサー shall シーン開始時の初期基準時刻を `0.0` とし、キーフレームスコープをシーン単位に限定する。
4. The pasta DSL 拡張パーサー shall 各コマンドの所要時間を算出せず、行の出現順序と構造情報のみを中間表現（CueIR）として出力する。
5. The CueSheet 構築層 shall 所要時間を外部注入インターフェース（Duration Resolver）から取得し、各 Cue の `start_time` を確定する。

> **設計注記**: パーサー → CueIR（順序 + 構造） → Duration Resolver 注入 → CueSheet（時刻確定）のパイプライン。

---

### 要件 2: キューコマンド行（`!` 行）

**目的**: スクリプト作者として、タイムライン制御・バリア・ステージ制御をパーサーが認識する `!` 接頭辞の独立行として記述したい。

#### 受入基準

1. The pasta DSL 拡張パーサー shall `!` または `！` で始まる行をキューコマンド行として認識する。
2. When シーン内に `!` コマンド行が 1 つ以上存在する場合、the pasta DSL 拡張パーサー shall そのシーン全体をキューシート拡張処理の対象とする。
3. The pasta DSL 拡張パーサー shall 以下のコマンドを提供する:
   - **タイムライン**: mark（マーカー登録）、seek（基準時刻移動）
   - **バリア**: yield（入力待ち）、select（選択待ち）、wait（時間待ち）
   - **ステージ制御**: clear（バルーンクリア）、route\_add / route\_switch / route\_remove（ルーティング）
4. The pasta DSL 拡張パーサー shall 日本語エイリアスが定義されたコマンドキーワードについて、英語正式名と日本語エイリアスの両方を等価に認識する。日本語エイリアスの割り当て対象は design.md の対照表で定義する。
5. The pasta DSL 拡張パーサー shall `!mark@name` で現在時刻にキーフレームマーカーを登録し、`!seek(@name)` / `!seek(@name, offset)` で基準時刻カーソルを指定マーカー時点（+ オフセット秒）に移動する。
6. The pasta DSL 拡張パーサー shall マーカー名のスコープをシーン単位とし、重複宣言および未宣言マーカーへの参照をエラーとする。
7. The pasta DSL 拡張パーサー shall mark をグローバル専用とし、`!mark@actor:name` のようなアクター修飾付き定義をエラーとする。
8. The CueSheet 構築層 shall mark エイリアス `@name` のアクション行内使用を 1 回限りとし、2 回以上の使用をエラーとする。
9. The pasta DSL 拡張パーサー shall yield / select に省略可能なタイムアウト秒引数を、wait に必須の秒数引数を認める。オフセット・秒数は 0.0 以上の浮動小数点数とする。
10. The pasta DSL 拡張パーサー shall `!clear` を明示記述時のみ `CueCommand::Clear` として生成する。
11. The CueSheet 構築層 shall 同一基準時刻を持つ複数の要素を並列演出として別々の Cue エントリに生成する。

> **設計注記**: コマンドキーワードの正式対照表（英語・日本語・舞台用語）は design.md で定義する。

---

### 要件 3: 名前付きコマンド定義

**目的**: スクリプト作者として、`@alias` に対して CueCommand の詳細を `!command@alias(args)` で宣言的に定義し、アクション行では `@alias` の参照だけで CueCommand を挿入したい。

#### 受入基準

1. The pasta DSL 拡張パーサー shall 以下の CueCommand に対応する名前付きコマンド定義を認識する:
   - emote — `Emote { key }`（表情変更）
   - choice — `Choice { id, text }`（選択肢データ）
   - custom — `Custom { command, params }`（カスタムコマンド）
2. The pasta DSL 拡張パーサー shall `!command@actor:alias(args)` をアクターローカル定義、`!command@alias(args)` をグローバル定義として区別する。
3. When `@alias` を解決する場合、the CueSheet 構築層 shall 発話アクターのローカル定義 → グローバル定義の優先順で探索する。
4. The pasta DSL 拡張パーサー shall 名前付きコマンド定義のスコープをシーン単位とする。
5. When `@alias` がいずれの定義にも見つからない場合、the CueSheet 構築層 shall `CueCommand::Emote { key: "alias" }` へフォールバックする。

> **設計注記**: Emote フォールバックは `@command` の最頻出用途が表情変更であることに基づく。actor 修飾はすべての定義で同一アルゴリズムだが、実務上もっとも意味が大きいのは `Emote` である。

---

### 要件 4: アクション行の CueCommand マッピング

**目的**: スクリプト作者として、既存の pasta アクション行で `CueCommand::Text` と CueCommand 挿入を自然に記述したい。

#### 受入基準

1. When アクション行 `actor：content` が記述された場合、the pasta DSL 拡張パーサー shall `content` 部分を `CueCommand::Text(content)` にマッピングする。
2. When アクション行に `@alias` が含まれる場合、the pasta DSL 拡張パーサー shall 要件 3 のエイリアス解決ルールに従い CueCommand に展開する。
3. When 継続行（`：content` 形式）が記述された場合、the pasta DSL 拡張パーサー shall `\n` 区切りで直前アクション行の `CueCommand::Text` に結合する。
4. If 継続行に `@command` が含まれる場合、the pasta DSL 拡張パーサー shall エラーを報告する。
5. When アクション行に複数の `@command` が含まれる場合、the pasta DSL 拡張パーサー shall 出現順に Text 断片と CueCommand を交互に生成する。

---

### 要件 5: Routing の自動生成と明示指定

**目的**: スクリプト作者として、ルーティングをアクター配置から自動生成させつつ、明示コマンドで任意の EntityKey を指定したい。

#### 受入基準

1. When 未割り当て actor（スロット未登録）が初出現した場合、the CueSheet 構築層 shall Shell・Balloon 両 CueTarget に `RoutingCommand::RouteAdd` を自動生成する。
2. The pasta DSL 拡張パーサー shall `!route_add(target, entity_key)` で CueTarget・EntityKey を明示指定した RouteAdd を、`!route_switch(target, entity_key)` で RouteSwitch を生成する。
3. The pasta DSL 拡張パーサー shall `!route_remove(target)` を明示記述時のみ生成する。自動 RouteRemove は行わない。
4. The pasta DSL 拡張パーサー shall `%actor、actor＝N` 記法（C# enum 式自動番号付け）で ActorKey → スロット番号マッピングを解析する。
5. The CueSheet 構築層 shall スロット割り当てをセッション永続とし、`%` 行指定を優先、未割り当てアクター初出現時は空き最小番号を割り当てる。
6. The pasta DSL 拡張パーサー shall `entity_key` 引数として `actor:<name>:<target>` / `spot:<name>` / `balloon:<name>` を受け入れる。

---

### 要件 6: 後方互換性

**目的**: 既存の pasta スクリプトが変更なく動作し続けることを保証する。

#### 受入基準

1. When シーン内に `!` コマンド行が存在しない場合、the pasta DSL 拡張パーサー shall そのシーンを現行 pasta DSL 仕様のみで処理する。
2. The pasta DSL 拡張パーサー shall 既存の構文要素（属性行・アクター配置・アクション行・継続行）の挙動をキューシート拡張対象外のシーンで変更しない。
3. The pasta DSL 拡張パーサー shall `@command` 記法の既存挙動（ランダムワード置換辞書）をキューシート拡張対象外のシーンで維持する。

---

### 要件 7: エラーハンドリング

**目的**: スクリプト作者として、文法エラーの箇所と原因が明確なエラーメッセージを受け取りたい。

#### 受入基準

1. The pasta DSL 拡張パーサー shall パースエラーに行番号・エラー種別・修正ヒントを含むメッセージを生成する。
2. The pasta DSL 拡張パーサー shall 重複マーカー名、未宣言マーカー参照、不正なリテラル値、不正な構文、不正なスロット番号、継続行内 `@command` を個別のエラー種別で報告する。
3. The CueSheet 構築層 shall アクター修飾付き mark 定義を `ActorScopedMarkUnsupported` エラーとして報告する。
4. The CueSheet 構築層 shall mark エイリアスの 2 回以上使用を `MarkUsedMultipleTimes` エラーとして報告する。

---

### 要件 8: 設計成果物

**目的**: 本仕様を元に pasta_dsl 実装者が文法拡張を実施できる設計書と動作サンプルを提供する。

#### 受入基準

1. The pasta DSL 拡張仕様 shall 全機能を網羅したサンプル `cue.pasta` を提供し、免責コメント（現行 pasta_core 非互換）を冒頭に記載する。
2. The pasta DSL 拡張仕様 shall `design.md` に PEG 文法定義、コマンドキーワード対照表（英語正式名・日本語エイリアス・舞台用語）、Duration Resolver / SlotRegistry インターフェース定義を記載する。
3. The pasta DSL 拡張仕様 shall `design.md` に段階的 MVP 実装フェーズ計画を記載する。

---

## 注記

### 確定事項サマリー

| # | 議題 | 決定 |
|---|------|------|
| Q1 | 暗黙キーフレームの所要時間算出 | Duration Resolver（外部注入）。パーサーは順序・構造のみ出力 |
| Q2 | `@command` 未定義時の挙動 | `CueCommand::Emote { key }` にフォールバック |
| Q3 | 継続行の扱い | `\n` 結合で同一 Cue に追記。継続行内 `@command` は不許可 |
| Q4 | `%` 行不在時のスロット割り当て | セッション永続。未割り当てアクターのみ空き最小番号を割り当て |
| Q5 | `CueCommand::Clear` 生成ポリシー | `!clear` 明示のみ。自動生成はアプリ層の責務 |
| Q6 | RouteRemove / RouteSwitch 発行条件 | 明示コマンドのみ。自動生成はアプリ層の責務 |
| Q7 | 1 行内複数 `@command` | 出現順に Text/CueCommand を交互に生成 |
| Q8 | キューシートモード識別 | `&type:cuesheet` は不要。`!` コマンド行の有無でシーン単位に暗黙判定 |
| Q9 | コマンド命名方針 | 英語正式名が基本。日本語エイリアスはスクリプト作者向けの一部コマンドのみ（emote/表情、choice/選択肢、custom/演出、select/選択待ち）。舞台用語はドキュメント注釈 |
| Q10 | actor ローカル定義 | `!command@actor:alias(args)` — 解決は actor ローカル → グローバル → Emote フォールバック |
| Q11 | mark のスコープ制約 | グローバル専用（actor 修飾不可）。アクション行内使用は 1 回限り |
| Q12 | 宣言/参照の視覚分離 | 宣言系: `!cmd@name(args)`、参照系: `@name`（行内）/ `!seek(@name, offset?)`（括弧内） |

### 設計フェーズへの引き継ぎ事項

- `!` コマンド行の PEG 文法（全角・半角両対応）
- コマンドキーワード対照表（英語正式名・日本語エイリアス・舞台用語）
- CueIR 型定義と配置モジュール
- Duration Resolver / SlotRegistry トレイト設計
- RouteAdd 自動生成判定ロジック
- 並列演出検出アルゴリズム
- 実装 MVP フェーズ計画

### 将来拡張候補

- グローバルスコープのエイリアス定義
- `CueCommand::Custom` の詳細パラメータ記法
- Storyboard 統合（CueSheet → Storyboard 起動、キーフレーム相互参照）
