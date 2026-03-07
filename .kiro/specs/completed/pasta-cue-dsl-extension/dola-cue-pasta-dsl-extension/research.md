# リサーチ記録: dola-cue-pasta-dsl-extension

---

## サマリー

- **フィーチャー**: `dola-cue-pasta-dsl-extension`
- **ディスカバリースコープ**: Extension（既存システムへの統合）
- **主要知見**:
  - dola の `CueSheet` データモデルは実装済み。`command.rs` / `sheet.rs` が型定義を提供し、7 バリアント (`CueCommand` + `BarrierKind` + `RoutingCommand`) が確定している。
  - 既存 `design.md`（旧版、`[timestamp]` + `\cue_*` トークン方式）は要件 v3（暗黙キーフレーム + `!` コマンド行 + エイリアス定義）と根本的に乖離しており、全面再設計が必要。
  - Q1〜Q7 ディスカッションで確定した「Duration Resolver 外部注入パターン」がアーキテクチャの核心。パーサーは順序・構造のみ出力し、時刻計算は委譲される。

---

## リサーチログ

### トピック 1: 既存 dola CueSheet 実装の確認

- **契機**: 設計の前提となるデータモデルと実装済み型を正確に把握するため
- **調査対象**: `crates/dola/src/cue/command.rs`, `crates/dola/src/cue/mod.rs`
- **主要知見**:
  - `CueCommand`: `Text(String)`, `Clear`, `Emote { key }`, `Choice { id, text }`, `EntityRef(u64)`, `Custom { command, params: DynamicValue }` の 6 バリアント
  - `BarrierKind`: `WaitForInput { timeout: Option<f64> }`, `WaitForChoice { timeout: Option<f64> }`, `Timeout { duration: f64 }` の 3 バリアント
  - `RoutingCommand`: `RouteAdd { target, to }`, `RouteSwitch { target, to }`, `RouteRemove { target }` の 3 バリアント
  - `CuePayload`: 上記 3 種を統合する enum（CueSheet 記述で使用）
  - `CueSheet` / `Cue` / `ActorKey` / `CueTarget` / `EntityKey` — ECS 非依存のドメイン型
- **設計への示唆**: pasta DSL 側では上記型に対応するテキスト表現を設計するのみ。型定義は変更不要。

### トピック 2: 旧 design.md（v1）との差分分析

- **契機**: 既存 `design.md` がいつ、どのアプローチで作成されたかを確認するため
- **調査内容**: 旧 `design.md` 全文と `CONTINUATION.md` を照合
- **主要知見**:
  - 旧設計は「タイムスタンプ記法 `[0.5]`」と「`\cue_*` sakura スクリプトトークン」を採用していた
  - 要件 v3 では「暗黙キーフレーム（時刻なし）」と「`!` コマンド行」に転換済み
  - `cue.pasta` サンプルも旧設計のままで、要件 v3 に合わせて更新が必要
- **設計への示唆**: 旧 `design.md` は参考程度に留め、新アーキテクチャで全面再設計を行う

### トピック 3: pasta DSL の行指向モデルと既存文法

- **契機**: 新文法要素（`!` 行・エイリアス定義行）が既存 pasta 文法とどう共存するかを設計するため
- **調査内容**: `CONTINUATION.md` のディスカッション確定事項、`product.md` の pasta 記述
- **主要知見**:
  - pasta DSL は「1 行 1 役割」の行指向文法を基本とする
  - `&key:value` 属性行、`%actor=slot` 配置行、`actor:content` アクション行、`:content` 継続行が既存構文
  - `@command` は pasta の「ランダムワード置換辞書」として既存実装。cuesheet モード外では現行動作を維持
  - `!` は pasta の既存行種別に存在しないため、新規行種別として追加可能
  - `@alias = Command(args)` 記法の `=` セパレータは既存文法に干渉しない
- **設計への示唆**: 行種別を `&type:cuesheet` というスコープで条件分岐させることで後方互換性を担保できる

### トピック 6: 宣言系と参照系の視覚分離

- **契機**: `!mark@name` と `!seek@name` が見た目上同型になり、宣言と参照の区別が弱いという議論を受けたため
- **調査内容**: 既存 cue 設計案とアクション行内 `@name` 参照モデルの整合性を比較
- **主要知見**:
  - 名前付き定義は `!command@name(args)` に統一すると、宣言の入口が一本化される
  - 参照系コマンドは `!command(@name, ...)` にすると、`@name` が参照であることを視覚的に保てる
  - `!seek@name` は `!mark@name` と同形に見え、宣言/参照の役割分離を弱める
  - アクション行は局所挿入に留め、タイムライン制御は独立した `!` 行に残す方が構造性を維持しやすい
- **設計への示唆**: 宣言系は `!mark@name`, `!emote@name(args)`, `!choice@name(args)`, `!custom@name(args)`、参照系は `!seek(@name, offset?)` に統一する

### トピック 4: Duration Resolver パターンの調査

- **契機**: Q1 決定（外部注入方式）の具体的なトレイト設計を確定するため
- **調査内容**: Q1 ディスカッション確定事項（`CONTINUATION.md`）、Rust トレイトオブジェクトのパターン
- **主要知見**:
  - パーサーは「行の出現順序と構造情報のみ」を出力する（IR: Intermediate Representation）
  - CueSheet への変換時（ビルダー層）に Duration Resolver が start_time を計算する
  - トレイトオブジェクト (`Box<dyn DurationResolver>`) か型パラメータジェネリクス (`T: DurationResolver`) のどちらでもよい
  - デフォルト実装（固定時間）を提供することで実用性を確保できる
- **設計への示唆**: `CueSheetBuilder::new(resolver: impl DurationResolver)` パターンが Rust 的に自然

### トピック 5: ルーティング自動生成アルゴリズムの分析

- **契機**: 要件 6（Routing 自動生成）の判定ロジックを具体化するため
- **調査内容**: Q4・Q6 ディスカッション確定事項、`RoutingCommand` 型定義
- **主要知見**:
  - スロット割り当て状態はセッション永続（Req 6.6）
  - `%` 行が存在する場合は明示優先、未割り当てアクター出現時のみ空き最小番号を自動割り当て
  - `RouteAdd`: 未割り当てアクターが初出現 → 自動生成
  - `RouteSwitch`: 既割り当てアクターが異なる CueTarget に切り替わる場合 → 自動生成
  - `RouteRemove`: 明示 `!route_remove` コマンドのみ（Req 6.7）
  - 判定には `get_slot_assignment(actor) -> Option<SlotId>` API が必要（Req 6.8）
- **設計への示唆**: `SlotRegistry` トレイトまたは `SlotTable` 構造体をビルダーが保持し、進行に伴い更新する

---

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク / 制限 | 評価 |
|---------|------|------|-------------|------|
| **A: pasta_dsl 完結** | pasta_dsl クレート内で CueSheet まで生成 | 単一クレート対応 | pasta_dsl に dola 依存が生まれる。循環依存リスク | 不採用 |
| **B: ブリッジクレート** | `pasta_cue` 等の独立クレートで変換全体を担う | 依存関係が明確 | 新クレート作成コスト。dola/pasta 両方の型を扱う | 有力候補 |
| **C: ハイブリッド（採用 → C' へ発展）** | pasta_dsl が CueIR（中間表現）を出力、dola または bridge 層が CueSheet に変換 | 責務分離が明確。pasta_dsl は構造/順序のみ担当 | IR 型定義の配置を決める必要あり → 決定 5 で解決済み | **採用（C' へ発展）** |

---

## 設計決定

### 決定 1: 実装アプローチ C（ハイブリッド）の採用

- **文脈**: pasta_dsl と dola の依存関係を整理しながら変換パイプラインを設計する必要がある
- **検討した選択肢**:
  1. Option A — pasta_dsl 完結。dola 依存関係がパーサー内に生まれる
  2. Option B — ブリッジクレート。新クレート追加コストが高い
  3. Option C — pasta_dsl が CueIR 出力、dola が CueSheetBuilder で消費
- **採用アプローチ**: Option C。pasta grammar が CueIR（Rust 中間 struct/enum）を出力し、`dola` クレートの `CueSheetBuilder` が DurationResolver・SlotRegistry と組み合わせて CueSheet を構築
- **根拠**: 責務分離が明確。pasta_dsl は文法解析のみ、時刻計算・ルーティング判定は dola 側に集約して単体テストしやすい
- **トレードオフ**: CueIR 型の配置先を決める必要がある（pasta_dsl クレートに定義 or 共有クレート） → 決定 5 で解決済み
- **フォローアップ**: 決定 5 により、CueIR は dola クレート内（`dola::cue::ir`）に配置。pasta_core は汎用 AST のみを出力し、dola 側の CueSheet コンパイラが AST → CueIR 変換を担う

### 決定 2: `!` コマンド行の記法設計（既存 `\cue_*` を廃止）

- **文脈**: 旧設計では sakura スクリプトトークン `\cue_*` をアクション行本文に埋め込んでいたが、要件 v3 では `!` 行として独立させる設計に変更
- **検討した選択肢**:
  1. `\cue_*` トークン方式（旧設計）— 行本文に埋め込む
  2. `!command` 独立行（新設計）— 演出制御を独立行として宣言
- **採用アプローチ**: `!command` 独立行。Barrier / Clear / Routing 制御は `!` 行として宣言し、アクション行本文には含めない
- **根拠**: 「1 行 1 役割」原則に合致。デバッグ時の可読性が向上。アクション行（Text/Emote生成）と制御行（Barrier/Routing）の責務が明確に分離される

### 決定 3: Duration Resolver の注入タイミング

- **文脈**: Q1 確定事項「パーサーは時刻を計算しない」の実現方法
- **採用アプローチ**: `CueSheetBuilder::build(scene, resolver: &impl DurationResolver, slot_registry: &mut impl SlotRegistry)` のメソッド引数で注入。ビルダーが IR を処理しながら逐次呼び出す
- **根拠**: 構造体の型パラメータを避け、`&impl T` でゼロコスト抽象化を維持しつつ API をシンプルに保つ（決定 6 参照）

---

### 決定 4: コマンドキーワードの日本語エイリアス割り当て方針

- **文脈**: 要件 2.4 は「日本語エイリアスが定義されたコマンドキーワードについて等価に認識する」と規定。全 12 コマンドへの割り当て有無を決定する必要があった
- **検討した選択肢**:
  1. 全コマンドに日本語エイリアスを付与 — 当て字感が強くなり不自然
  2. 一切付与しない — スクリプト作者の利便性が低下
  3. 利用頻度・自然さに基づき選択的に付与
- **採用アプローチ**: 選択肢 3。4 コマンドのみに日本語エイリアスを割り当て
  - `emote`⇔`表情`、`choice`⇔`選択肢`、`custom`⇔`演出`、`select`⇔`選択待ち`
- **根拠**:
  - 名前付きコマンド定義（emote/choice/custom）はスクリプト作者が頻繁に記述するため日本語が馴染む
  - `select` は `choice` と対概念であり「選択待ち」が自然
  - `yield` に「待機」を充てると `wait` と紛らわしい（両者とも"待つ"の意味合い）
  - `mark`/`seek`/`clear` はプログラマーにとって英語の方が通じやすい
  - `route_*` 系は定型コマンドであり日本語化の利点が薄い
- **フォローアップ**: PEG 文法ルールで日本語キーワードを定義するのは `emote`/`choice`/`custom`/`select` の 4 ルールのみ

---

### 決定 5: pasta_core と dola の責務境界と CueIR 配置

- **文脈**: 決定 1 のフォローアップとして、CueIR の配置先クレートを確定させる必要があった。設計レビュー (v3) で CueIR 型が dola ドメイン型（`ActorKey`, `CueCommand`, `CueTarget`, `EntityKey`）を直接参照しており、pasta_dsl に CueIR を配置すると循環依存が発生する問題が指摘された
- **開発者からの入力**:
  - pasta_core は誰からも依存されない独立クレートである
  - pasta_core の AST はキューコマンドそのものを理解せず、`!` + コマンド名 + 引数トークン群という構造のみを解釈して保持する（意味解釈なし）
  - CueIR は pasta_core に存在せず、dola 側でコンパイル時に生成される
- **採用アプローチ**: Option C'（ハイブリッド改）。元の Option C から以下の変更を加える:
  1. pasta_core は汎用構造的 AST のみを出力（CueIR を保持しない）
  2. CueIR 型は `dola::cue::ir` モジュールに配置（dola ドメイン型を自由に使用可能）
  3. dola に CueSheet コンパイラ（`dola::cue::compiler`）を新設し、pasta_core AST → CueIR 変換を担う
  4. 依存方向: dola → pasta_core（一方向）。pasta_core は誰からも依存されない
- **根拠**: 循環依存の根本解決。CueIR が dola 型を直接使うため変換コストゼロ。pasta_core の独立性を完全に維持
- **トレードオフ**: dola に CueSheet コンパイラという新コンポーネントが増えるが、責務分離としては自然
---

### 決定 6: SlotRegistry の所有権・注入方式・永続スコープ

- **文脈**: 設計レビューで SlotRegistry の寿命・所有者が未定義と指摘された。要件 5.5「セッションをまたいで永続」の具体的な実現モデルを確定させる必要があった
- **開発者からの入力**:
  - 永続層の最終的な所有者は areka ランタイム（確定）
  - 永続層の実装コード（トレイト定義・デフォルト実装）は dola に置いても問題ない
  - トレイトオブジェクト方式ではなく `&mut impl SlotRegistry` ジェネリクス方式が Rust 流
  - 永続スコープはアプリケーション起動〜終了（メモリ上）
  - 将来ディスク永続化する場合は serde 対応を具象型に追加（トレイトには影響しない）
- **検討した選択肢**:
  1. ジェネリクス（構造体型パラメータ）— `CueSheetBuilder<R, S>` が呼び出し元まで型伝播する
  2. トレイトオブジェクト — `&mut dyn SlotRegistry` で型消去
  3. メソッド引数での `&mut impl` 注入 — 構造体から型パラメータを除去、呼び出し元への型伝播を避けつつ静的ディスパッチを維持
- **採用アプローチ**: 選択肢 3。`CueSheetBuilder` は型パラメータなしの構造体。`build()` メソッドが `&impl DurationResolver` と `&mut impl SlotRegistry` を引数で受け取る
- **根拠**:
  - `&mut impl T` が成立する場合はトレイトオブジェクトを避けるのが Rust の慣例
  - areka が具象型の所有権を保持し、Builder は借用するだけなので `&mut` が自然
  - DurationResolver は冪等・副作用なしなので `&`（不変借用）で十分
- **トレードオフ**: 無し。全面的に合理的
---

## リスクと軽減策

- **R1: pasta_dsl が外部リポジトリのため直接修正できない** — 軽減策: `vendors/pasta` として管理。本設計書で pasta_core 実装者向けの仕様指示として機能させる。実装は別フェーズ
- **R2: Duration Resolver のデフォルト実装が「テスト用固定値」のみになるリスク** — 軽減策: `FixedDurationResolver` をデフォルト実装として提供し、テスト・プロトタイプで即利用可能にする。本番ではキャラクター発話速度などを参照する実装を想定
- **R3: PEG 文法の全角/半角両対応が複雑化するリスク** — 軽減策: キーワードの全角化は「記法対応表」として別途管理し、PEG ルールには英語キーワードのみ定義して変換レイヤーで正規化する方針も検討
- **R4: CueIR 型の配置先クレートが確定しないと双方向依存になるリスク** — **解決済み**（決定 5）: CueIR は `dola::cue::ir` に配置。pasta_core は汎用 AST のみを出力し、CueIR を保持しない。dola が pasta_core に一方向依存
- **R5: エイリアステーブルのスコープ（シーン単位 vs グローバル）の曖昧さ** — 軽減策: 要件 4.3 でシーン単位と明示済み。グローバルは将来拡張として留保
- **R6: SlotRegistry の寿命・所有者が未定義** — **解決済み**（決定 6）: areka ランタイムが具象型を所有、アプリ存続期間メモリ保持、build() に `&mut impl` で注入

---

## 参照

- [dola CueCommand 定義](../../../../crates/dola/src/cue/command.rs) — 全コマンド型定義の最新版
- [dola CueSheet 定義](../../../../crates/dola/src/cue/sheet.rs) — CueSheet 構造体
- [要件定義書 v4](requirements.md) — 命名方針反映済み最終版
- [CONTINUATION.md](CONTINUATION.md) — ディスカッション全記録
