# Requirements Document

## Project Description (Input)

## token→さくらスクリプトbuildの前処理

`pasta.shiori.act`:build()にて、token→さくらスクリプトへのビルドを行っているが、tokenをactor切り替え単位でグループ化する前処理を入れる。

アクター単位で連続したtalkを1つにまとめることで、会話速度調整（文字単位でウェイト）などの後処理フィルター（別仕様）などを投入予定のため。

### フェーズ1: actor切り替え単位でグループ化

```json（概念コード）
[
  { type: "spot", actor: <うにゅうactor>, spot: 1 },
  {
    type: "actor",
    actor: <さくらactor>,
    tokens: [
        {type: "talk", text: "今日は"},
        {type: "talk", text: "晴れ"},
        {type: "talk", text:"でした。"}
    ]
  },
  {
    type: "actor",
    actor: <うにゅうactor>,
    tokens: [
        {type: "talk", text: "明日は"},
        {type: "talk", text: "雨"},
        {type: "talk", text: "らしいで。"}
    ]
  }
]
```

### フェーズ2: 連続したtalkを1つにまとめる

```json（概念コード）
[
  { type: "spot", actor: <うにゅうactor>, spot: 1 },
  {
    type: "actor",
    actor: <さくらactor>,
    tokens: [
        {type: "talk", text: "今日は晴れでした。"},
    ]
  },
  {
    type: "actor",
    actor: <うにゅうactor>,
    tokens: [
        {type: "talk", text: "明日は雨らしいで。"},
    ]
  }
]
```

### 重要な設計決定（2026-02-03追記）

1. **グループ化の実装箇所**: `pasta.act`モジュールの`ACT_IMPL.build()`
2. **責務分離**: 
   - `ACT_IMPL.build()` → グループ化されたトークン配列を返す
   - `SHIORI_ACT_IMPL.build()` → グループ化されたトークンを処理してさくらスクリプト生成
3. **後方互換性**: 最終的な`SHIORI_ACT_IMPL.build()`の出力（さくらスクリプト）は変化なし
4. **トークン分類（2026-02-03追記）**:
   - **アクター属性設定**: `spot`, `clear_spot` - グループ化対象外、独立トークンとして維持
   - **アクター行動**: `talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script` - グループ化対象

---

## Introduction

本仕様は、`pasta.act`モジュールにおいて、トークン配列をアクター切り替え単位でグループ化する前処理機能を定義する。

現在の`ACT_IMPL.build()`はフラットなトークン配列を返しているが、将来の拡張（会話速度調整、文字単位ウェイト挿入等のフィルター機能）に対応するため、アクター切り替え単位でトークンをグループ化する中間表現を導入する。

### トークン分類

| 分類 | トークン | グループ化 | 説明 |
|------|---------|-----------|------|
| アクター属性設定 | `spot`, `clear_spot` | 対象外 | 位置設定、遅延適用される属性 |
| アクター行動 | `talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script` | 対象 | 直前のアクターに紐づく行動 |

**処理フロー（変更後）**:

```
self.token[] 
  → ACT_IMPL.build() [グループ化・talk統合]
  → grouped_token[] (spot | clear_spot | type="actor")
  → SHIORI_ACT_IMPL.build() [さくらスクリプト生成]
  → さくらスクリプト文字列
```

本機能は2段階のフェーズで構成される:

1. **グループ化フェーズ**: アクター切り替え境界でトークンを分割（`pasta.act`）
2. **Talk統合フェーズ**: グループ内の連続talkトークンを単一talkに結合（`pasta.act`）

---

## Requirements

### Requirement 1: グループ化後のトークン構造定義

**Objective:** 開発者として、グループ化後のトークン構造を明確に定義したい。これにより、アクター単位での後処理フィルター（会話速度調整等）の実装が可能になる。

#### 設計根拠

グループ化後の出力は3種類のトークンで構成される：

1. `spot` - アクター属性設定（位置）、独立トークン
2. `clear_spot` - 全スポット情報クリア、独立トークン
3. `type="actor"` - アクター行動グループ、内部にtokens配列を持つ

#### Acceptance Criteria

1. The `ACT_IMPL.build()` function shall return an array containing three token types:
   - `{ type = "spot", actor = <Actor>, spot = <number> }`
   - `{ type = "clear_spot" }`
   - `{ type = "actor", actor = <Actor|nil>, tokens = <table[]> }`

2. When トークン配列が空の場合, the `ACT_IMPL.build()` function shall 空の配列を返す。

3. When トークン配列にtalkトークンのみが含まれる場合, the `ACT_IMPL.build()` function shall 単一の`type="actor"`トークンを生成する。

---

### Requirement 2: アクター切り替え境界でのグループ化

**Objective:** 開発者として、トークン配列をアクター切り替え境界で分割したい。これにより、アクター単位での処理が可能になる。

#### 設計根拠

- **アクター属性設定**（`spot`, `clear_spot`）: グループ化対象外。独立トークンとして維持され、行動を伴わない。
- **アクター行動**（`talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script`）: グループ化対象。直前のアクターに紐づく行動として処理される。
- **グループ境界**: `talk.actor`の変化のみでグループを分割する。

#### Acceptance Criteria

1. When `spot`または`clear_spot`トークンが現れた場合, the `ACT_IMPL.build()` function shall それらを独立トークンとして出力する（グループに含めない）。

2. When `talk`トークンのactorが前の`talk`トークンと異なる場合, the `ACT_IMPL.build()` function shall 新しい`type="actor"`トークンを開始する。

3. When `talk`トークンのactorが前の`talk`トークンと同一の場合, the `ACT_IMPL.build()` function shall 同一`type="actor"`トークン内にトークンを追加する。

4. When アクター行動トークン（`surface`, `wait`, `newline`, `clear`, `sakura_script`）が現れた場合, the `ACT_IMPL.build()` function shall 現在の`type="actor"`トークン内に追加する。

5. While グループ化処理中, the `ACT_IMPL.build()` function shall トークンの順序を保持する。

---

### Requirement 3: 連続talkトークンの統合

**Objective:** 開発者として、同一アクターグループ内の連続したtalkトークンを単一のtalkトークンに統合したい。これにより、後処理フィルターが完全な発言テキストにアクセスできる。

#### Acceptance Criteria

1. When 同一`type="actor"`トークン内にtalkトークンが連続する場合, the merge function shall それらのtextを結合して単一のtalkトークンを生成する。

2. When talkトークンの間にアクター行動トークン（`surface`, `wait`, `newline`等）が挟まる場合, the merge function shall talkトークンを分離したまま維持する。

3. When 結合する場合, the merge function shall 最初のtalkトークンのactor情報を保持する。

4. The merge function shall アクター行動トークンをそのまま保持する。

---

### Requirement 4: SHIORI_ACT_IMPL.build()のグループ対応

**Objective:** 開発者として、`SHIORI_ACT_IMPL.build()`がグループ化されたトークンを処理できるようにしたい。最終出力（さくらスクリプト）は変化なし。

#### Acceptance Criteria

1. The `SHIORI_ACT_IMPL.build()` function shall `ACT_IMPL.build()`からグループ化されたトークン配列を受け取る。

2. The `SHIORI_ACT_IMPL.build()` function shall `type="actor"`トークンをフラット化してsakura_builderに渡す、またはsakura_builderが直接処理する。

3. The `SHIORI_ACT_IMPL.build()` function shall 既存の出力結果（さくらスクリプト）と完全互換を維持する。

4. While 処理中, the SHIORI_ACT module shall 外部APIに変更を加えない。

---

### Requirement 5: エッジケース処理

**Objective:** 開発者として、様々なエッジケースでも安定して動作することを保証したい。

#### Acceptance Criteria

1. When actorがnilのtalkトークンが現れた場合, the grouping function shall 「nilアクター」として独立したグループを形成する。

2. When 同一actorが断続的に現れる場合（A→B→A）, the grouping function shall 別々の`type="actor"`トークンとして扱う。

3. When talkトークンのtextが空文字列の場合, the merge function shall 空文字列もそのまま結合に含める。

4. When `type="actor"`トークン内にtalkトークンが存在しない場合, the merge function shall そのトークンをそのまま出力する。

5. When 最初のトークンがアクター行動（非talk）の場合, the grouping function shall actor=nilの`type="actor"`トークンを作成して追加する。

---

### Requirement 6: 後方互換性の保証

**Objective:** 開発者として、本変更によって既存のさくらスクリプト出力が変化しないことを保証したい。

#### Acceptance Criteria

1. The `SHIORI_ACT_IMPL.build()` function shall 既存のsakura_builder_test.luaの全テストをパスする。

2. While グループ化・統合処理後, the final sakura script output shall 既存出力と完全一致する。

3. The implementation shall 既存の外部API（`ACT.new()`, `SHIORI_ACT_IMPL.build()`のシグネチャ）を変更しない。ただし`ACT_IMPL.build()`の戻り値型は`token[]`から`grouped_token[]`に変更される。

---

### Requirement 7: 将来拡張への準備

**Objective:** 開発者として、本仕様の実装が将来の拡張（会話速度フィルター等）を容易にする設計であることを保証したい。

#### Acceptance Criteria

1. The `type="actor"` token structure shall グループ単位でのフィルター関数適用を可能にする設計とする。

2. The merge function shall オプションで無効化可能とする（将来の細粒度制御のため）。

3. The grouping function shall 純粋関数として実装し、副作用を持たない。
