# ギャップ分析レポート: suppress-ontalk-on-choosing

## サマリー

- **スコープ**: virtual_dispatcher.lua の `check_talk()` / `check_hour()` に choosing 状態の抑制ガードを追加。付随して既存の talking 判定をカンマ区切り対応に修正。
- **既存資産**: 抑制ロジックの構造（ガード節 → `return nil`）は talking 抑制で確立済み。同パターンを踏襲可能。
- **主要ギャップ**: Status ヘッダーの判定方式が完全一致（`==`）であり、SSP が実際に送出するカンマ区切り複合値（`talking,choosing,balloon(0=2)`）に対応していない。
- **実装量**: S（1–3日）。変更対象は Lua ファイル 1 本 + テスト 2 本（Lua spec / Rust 統合テスト）。
- **リスク**: Low。既存パターンの拡張であり、アーキテクチャ変更なし。

---

## 1. 現状調査

### 1.1 対象ファイルと責務

| ファイル | 責務 |
|---------|------|
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | OnTalk/OnHour の条件判定・発行。**変更対象**。 |
| `crates/pasta_shiori/src/lua_request.rs` | SHIORI リクエスト → Lua テーブル変換。`Status` ヘッダーを `req.status` に文字列としてそのまま格納。 |
| `crates/pasta_shiori/src/util/parsers/req.rs` | SHIORI リクエストの PEG パーサー。`status: Option<&str>` としてヘッダー値を生保存。 |
| `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua` | Lua BDD テスト。talking スキップのテストケースあり。 |
| `crates/pasta_lua/tests/shiori/virtual_event_config_test.rs` | Rust 統合テスト。talking スキップのテストケースあり。 |

### 1.2 Status ヘッダーのデータフロー

```
SSP → SHIORI/3.0 リクエスト
  Status: talking,choosing,balloon(0=2)
    ↓
pasta_shiori (PEG パーサー)
  ShioriRequest.status = Some("talking,choosing,balloon(0=2)")
    ↓
lua_request.rs
  table.set("status", "talking,choosing,balloon(0=2)")
    ↓
Lua 側: act.req.status == "talking,choosing,balloon(0=2)"
```

**重要**: Status ヘッダーの値は**パース・分割されずに文字列のまま** Lua に渡される。現在の `== "talking"` は `Status: talking` 単独の場合のみマッチし、`Status: talking,balloon(0=0)` にはマッチしない。

### 1.3 SSP ログの実態（shiori-sample.log より）

| Status 値 | 出現パターン |
|-----------|-------------|
| `talking` | 単独出現あり（初期のみ） |
| `talking,balloon(0=0)` | 最も多い出現パターン |
| `talking,choosing,balloon(0=2)` | 選択肢表示中 |
| `talking,choosing,balloon(0=2,1=0)` | 選択肢表示中（複数スコープ） |
| `balloon(0=0)` | トーク終了後、バルーン残留中 |

### 1.4 既存 talking 抑制の実装

```lua
-- check_talk() L130-132
if act.req.status == "talking" then
    return nil
end

-- check_hour() L98-100
if act.req.status == "talking" then
    return nil
end
```

**ギャップ**: 完全一致（`==`）のため、`talking,balloon(0=0)` では talking 状態を検出**できない**。

### 1.5 既存テストの状況

| テスト | 手法 | talking テスト | choosing テスト |
|--------|------|---------------|----------------|
| `virtual_dispatcher_spec.lua` | Lua BDD (lua_test) | ✅ `status = "talking"` 単独値 | ❌ なし |
| `virtual_event_config_test.rs` | Rust 統合テスト | ✅ `status = "talking"` 単独値 | ❌ なし |

**ギャップ**: カンマ区切り値でのテストケースが存在しない。

---

## 2. 要件→アセット マッピング

| 要件 | 既存アセット | ギャップ |
|------|------------|---------|
| Req 1: choosing で OnTalk 抑制 | `check_talk()` のガード節パターン | **Missing**: choosing 判定ガード未実装 |
| Req 2: choosing で OnHour 抑制 | `check_hour()` のガード節パターン | **Missing**: choosing 判定ガード未実装 |
| Req 3: カンマ区切り Status 対応 | `req.status` は文字列のまま渡される | **Missing**: 部分一致検出ロジック |
| Req 4: talking のカンマ区切り整合 | `== "talking"` 完全一致 | **Constraint**: 既存判定ロジックの修正が必要 |
| Req 5: テストカバレッジ | talking テスト既存 | **Missing**: choosing テスト + カンマ区切りテスト |

---

## 3. 実装アプローチ評価

### Option A: 文字列検索（`string.find`）方式 — Lua 側のみ修正

**概要**: `act.req.status` に対して `string.find()` で部分一致検索を行い、`talking` / `choosing` を検出する。

**変更箇所**:
- `virtual_dispatcher.lua`: ヘルパー関数 1 つ追加 + ガード節 4 箇所修正
- テスト: Lua spec + Rust 統合テスト

**実装イメージ**:
```lua
local function has_status(status, keyword)
    if not status then return false end
    return status:find(keyword, 1, true) ~= nil
end

-- check_talk() / check_hour() 内:
if has_status(act.req.status, "talking") then return nil end
if has_status(act.req.status, "choosing") then return nil end
```

**トレードオフ**:
- ✅ 最小変更量。Lua ファイル 1 本のみ
- ✅ Rust 側の変更不要
- ✅ 既存のアーキテクチャパターンに完全準拠
- ❌ `balloon` 等の部分文字列誤検出リスク（ただし `talking` / `choosing` は他トークンの部分文字列にはならない）

### Option B: Rust 側で Status を分割して配列化

**概要**: `lua_request.rs` で Status ヘッダーをカンマ分割し、Lua テーブル（配列）として渡す。

**変更箇所**:
- `lua_request.rs`: Status 値のカンマ分割処理追加
- `virtual_dispatcher.lua`: ガード節を配列検索に変更
- テスト: Rust パーサーテスト + Lua spec + Rust 統合テスト

**トレードオフ**:
- ✅ 型安全。構造化データとして扱える
- ✅ 将来的に他の Status 値（`balloon` パラメータ等）にもアクセスしやすい
- ❌ Rust 側の変更が必要（影響範囲拡大）
- ❌ `act.req.status` の型が `string` → `table` に変わり、**既存コードの後方互換性が破壊される**
- ❌ Status を直接参照している他のコード（もしあれば）にも影響

### Option C: Rust 側でフラグ化

**概要**: `lua_request.rs` で `req.is_talking` / `req.is_choosing` のようなブーリアンフィールドを追加。

**変更箇所**:
- `lua_request.rs`: Status パース + フラグ設定追加
- `virtual_dispatcher.lua`: ガード節をフラグ参照に変更
- テスト: 全レイヤー

**トレードオフ**:
- ✅ 判定ロジックが最もシンプル
- ✅ Lua 側の判定は `if act.req.is_choosing then`
- ❌ Rust 側の変更が最も多い
- ❌ 新しい Status 値が追加されるたびに Rust 側の対応が必要
- ❌ 過剰設計の可能性が高い

---

## 4. 複雑性・リスク評価

| 項目 | 評価 | 理由 |
|------|------|------|
| **実装量** | **S**（1–3日） | 既存パターン踏襲、Lua ガード節追加 + テスト |
| **リスク** | **Low** | 馴染みのあるパターン、明確なスコープ、テスト既存 |
| **後方互換性** | Option A: 互換性維持 / Option B,C: 破壊的変更あり | |

---

## 5. 推奨事項（設計フェーズ向け）

### 推奨アプローチ: Option A（`string.find` 方式）

**理由**:
1. **最小変更原則**: Lua ファイル 1 本の修正で全要件を充足
2. **後方互換**: `act.req.status` の型・インターフェースを維持
3. **既存パターン踏襲**: talking ガード節と同構造で、コードの一貫性を保持
4. **誤検出リスクなし**: `talking` / `choosing` は SSP Status トークンとして一意であり、他トークンの部分文字列にならない

### 設計フェーズでの決定事項
1. ヘルパー関数（`has_status`）のスコープ: モジュールローカル関数 vs 共有ユーティリティ
2. choosing ガードの挿入位置: talking ガードの直後 vs 統合判定
3. `check_hour()` における choosing スキップ時の正時タイムスタンプ更新有無の確認

### リサーチ不要
- SSP の Status ヘッダー仕様は shiori-sample.log で実態確認済み
- 外部依存なし
