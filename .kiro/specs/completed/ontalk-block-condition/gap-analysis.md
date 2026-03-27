# ギャップ分析レポート: ontalk-block-condition

## 1. 現状調査

### 対象モジュール・ファイル一覧

| ファイル | 役割 |
|--------|------|
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | 仮想イベント(OnTalk/OnHour)の条件判定・発行 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua` | OnSecondChangeデフォルトハンドラ（dispatcher呼び出し元） |
| `crates/pasta_shiori/src/lua_request.rs` | SHIORIリクエスト解析 → `req.status` へのマッピング |
| `crates/pasta_shiori/src/util/parsers/req_parser.pest` | `key_status` PEGルール定義 |
| `.agents/skills/pasta-lua-coding/references/shiori-handlers.md` | スキルリファレンス（仮想ディスパッチャ記述） |

### テスト資産

| テストファイル | 内容 |
|-------------|------|
| `crates/pasta_lua/tests/shiori/virtual_event_dispatch_test.rs` | Rust統合テスト（モジュールロード、OnHour/OnTalk発火、優先順位） |
| `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua` | Lua BDDテスト（基本動作、talking/choosingブロック、タイマー） |
| `crates/pasta_lua/tests/lua_specs/second_change_thread_test.lua` | second_change経由のthread受け渡しテスト |
| `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_thread_test.lua` | thread返却形式テスト |

### 既存アーキテクチャパターン

#### Status文字列のフロー
```
SSP → SHIORI GET リクエスト (Status ヘッダ)
  → Rust: lua_request.rs → key_status → table.set("status", value)
    → Lua: act.req.status (生の文字列: "talking,balloon(0=0)" 等)
      → virtual_dispatcher: has_status() でキーワードマッチ
```

- `act.req.status` はSSPからの生文字列がそのまま格納される（カンマ区切り複合値）
- `has_status(status, keyword)` は `string:find(keyword, 1, true)` で部分文字列検索（プレーンマッチ）

#### 現行ブロック条件の配置
- `check_hour()`: talking, choosing を**個別チェック**（正時超過後、next_hour更新前）
- `check_talk()`: talking, choosing を**個別チェック**（interval計算前）
- `dispatch()`: ブロック判定**なし**（req.dateチェックのみ）

#### scripts/ カスタマイズパターン
- `scripts/` は `pasta_scripts/` より優先される Luaファイルロード（同名ファイル上書き方式）
- モジュールテーブル `M` をrequireで取得し、フィールドを上書きするのが標準パターン
- 例: `REG.OnSecondChange` の上書きで完全カスタムハンドラ設定可能

---

## 2. 要件実現可能性分析

### 要件→既存資産マッピング

| 要件 | 既存資産 | ギャップ |
|-----|---------|---------|
| Req 1: dispatch集約ブロック | `dispatch()` にはブロック判定なし。`has_status()` ユーティリティは既存 | **Missing**: dispatch入口のブロック判定ロジック |
| Req 2: 重複チェック廃止 | `check_hour`/`check_talk` に talking/choosing チェックあり | **要リファクタ**: 4箇所の個別判定を削除 |
| Req 3: カスタマイズ可能 | `M` テーブル公開パターン、scripts/上書きパターン既存 | **Missing**: ブロックリストテーブルの公開APIなし |
| Req 4: minimizing対応 | ブロック対象に未含 | **Missing**: minimizingのブロック判定 |
| Req 5: テスト | talking/choosingブロックテストは既存 | **要拡張**: 新Status全9種 + カスタマイズテスト |
| Req 6: ドキュメント | shiori-handlers.md にdispatch記述あり | **要更新**: ブロック条件の記述が不完全 |

### has_status() の安全性

`has_status("opening(communicate/input)", "opening")` → `true` ✓  
`has_status("choosing,balloon(0=0)", "choosing")` → `true` ✓  
`has_status("idle", "online")` → `false` ✓

⚠️ 潜在リスク: `has_status("nouserbreak", "user")` → `true`（部分一致）  
→ ただし、SSP Status値にはこのような衝突パターンは現在存在しない。  
→ 将来の安全性のため、ワード境界マッチ（カンマ分割 or 先頭/末尾チェック）を検討可能だが、現時点ではプレーンfindで十分。

### 制約

- **後方互換**: `check_hour()`/`check_talk()` は公開APIとして既存テストから直接呼ばれている
  - これらの関数からブロック判定を削除しても、`dispatch()` 経由で使う限り安全
  - 直接呼び出すユーザーにとっては、dispatch入口のガードをバイパスできることになる → **設計判断必要**

---

## 3. 実装アプローチ評価

### Option A: dispatch入口に集約（最小変更）

**変更対象**: `virtual_dispatcher.lua` 1ファイル + テスト + ドキュメント

1. `M.blocked_statuses` テーブルを宣言（デフォルト: 全9キーワード）
2. `dispatch()` の `req.date` チェック直後に、`M.blocked_statuses` をループして `has_status()` で判定
3. `check_hour()`/`check_talk()` 内の talking/choosing チェックを削除
4. テスト追加・ドキュメント更新

**トレードオフ**:
- ✅ 1ファイルのみ変更、既存パターンに沿う
- ✅ `M.blocked_statuses` テーブル公開で scripts/ から上書き可能
- ✅ テスト資産がそのまま活用可能
- ❌ `check_hour()`/`check_talk()` を直接呼ぶとガードがバイパスされる

### Option B: 共通ガード関数を check_hour/check_talk にも適用

**変更対象**: `virtual_dispatcher.lua` 1ファイル + テスト + ドキュメント

1. `M.blocked_statuses` テーブルを宣言
2. ローカル関数 `is_blocked(status)` を追加（テーブルをループ）
3. `dispatch()` 入口で `is_blocked()` → `return nil`
4. `check_hour()`/`check_talk()` 冒頭にも `is_blocked()` を配置（既存の talking/choosing チェックを置換）

**トレードオフ**:
- ✅ 直接呼び出しでも安全
- ✅ 重複は排除（共通関数1つ）
- ❌ dispatch直後にcheck_*を呼ぶ通常パスで二重判定（パフォーマンス影響は無視可能）
- ❌ Req 2「個別チェック廃止」の文面と微妙に矛盾（ただし実質は共通化で達成）

### Option C: dispatch入口のみ + check_hour/check_talk 個別チェック完全削除

**Option Aと同一**（Req 2の狭義解釈）

**推奨: Option A**

理由:
- 変更量最小、Req 2の意図（一元管理）を達成
- 直接呼び出しのバイパスリスクは、ドキュメントで「dispatch()経由で使うこと」を明記すれば対応可能
- check_hour/check_talk は内部関数的に使われており、外部から直接呼ぶユースケースは限定的

---

## 4. 複雑性・リスク評価

| 項目 | 評価 | 根拠 |
|-----|------|------|
| **工数** | **S (1-3日)** | 単一Luaモジュールの修正、既存パターンの延長、テスト追加 |
| **リスク** | **Low** | 既存の `has_status()` + テーブル公開パターンの組み合わせ。アーキテクチャ変更なし。全テスト既存フレームワーク内で記述可能 |

---

## 5. 設計フェーズへの引き継ぎ事項

### 決定事項候補
1. **Option A/B選択**: dispatch入口のみ vs check_*にも共通ガード適用
2. **has_status() の安全性**: プレーンfind維持 vs ワード境界チェック強化

### Research Needed なし
- 外部依存なし、SSP Status仕様は既に取得済み
- 全実装がLuaレイヤーで完結

### テスト戦略
- Lua BDDテスト (`virtual_dispatcher_spec.lua`) に新ブロック条件テストを追加
- Rust統合テスト (`virtual_event_dispatch_test.rs`) にStatus複合値テストを追加
- カスタマイズテスト: `M.blocked_statuses` の変更が反映されることを検証
