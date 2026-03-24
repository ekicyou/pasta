# Gap Analysis: budoux-line-breaker

## 1. 現状調査

### 1.1 関連アセット一覧

| アセット | パス | 役割 |
|---------|------|------|
| sakura_script/mod.rs | `crates/pasta_lua/src/sakura_script/mod.rs` | Lua モジュール登録 (`@pasta_sakura_script`) |
| tokenizer.rs | `crates/pasta_lua/src/sakura_script/tokenizer.rs` | さくらスクリプトタグ検出・文字分類 |
| wait_inserter.rs | `crates/pasta_lua/src/sakura_script/wait_inserter.rs` | ウェイトタグ挿入 |
| module_registry.rs | `crates/pasta_lua/src/runtime/module_registry.rs` | Lua モジュール一括登録 |
| config.rs | `crates/pasta_lua/src/loader/config.rs` | `TalkConfig` パース（pasta.toml [talk]） |
| pasta.toml | `dist-src/ghost/master/pasta.toml` | サンプルゴースト設定 |

### 1.2 検出された設計パターン

| パターン | 内容 |
|---------|------|
| Arc 状態パターン | `Arc<SakuraScriptState>` をクロージャでキャプチャし Lua 関数に注入 |
| モジュール登録 | `register(lua, config) → LuaResult<Table>` で独立テーブル作成 → `package.loaded` に注入 |
| アクターオーバーライド | `resolve_wait_values()` でアクターテーブル → デフォルト → ハードコードの3段階フォールバック |
| TOML パススルー | `[actor.*]` のフィールドはスキーマ検証なしで Lua テーブルに自動伝搬 |
| トークナイザー | `Tokenizer::SAKURA_TAG_PATTERN` 正規表現でさくらスクリプトタグを最優先検出 |

### 1.3 統合サーフェス

- **Tokenizer**: `SAKURA_TAG_PATTERN` 正規表現と `tokenize()` はbudoux前処理で再利用可能
- **CONFIG テーブル**: `CONFIG.actor["名前"].budoux` として TOML → Lua 自動伝搬（追加コード不要）
- **Lua パイプライン**: `talk_to_script()` 呼出後に Lua 側で budoux 関数を呼ぶ（mod.rs に新関数登録）

---

## 2. 要件実現性分析

### 要件→アセットマップ

| 要件 | 既存アセット | ギャップ |
|------|-------------|---------|
| Req 1: クレート依存追加 | Cargo.toml | **Missing**: `budoux`, `unicode-width` 未登録 |
| Req 2: さくらスクリプト透過 | `Tokenizer::SAKURA_TAG_PATTERN` | **部分対応**: 正規表現は再利用可能。タグ除去→budoux→タグ再挿入ロジックは新規 |
| Req 3: budoux 分割 | なし | **Missing**: budoux API 呼出ラッパー新規 |
| Req 4: 行幅閾値 | なし | **Missing**: 幅計算+閾値比較+改行挿入アルゴリズム新規 |
| Req 5: pasta.toml 設定 | TOML パススルー | **対応済み**: `[actor."名前"].budoux = [10,12]` は自動で `CONFIG.actor` に伝搬 |
| Req 6: Lua API 公開 | `register()` パターン | **部分対応**: 既存パターン踏襲で新関数追加可能 |
| Req 7: パイプライン統合 | `talk_to_script()` | **Missing**: Lua 側の呼出判定ロジック（actor.budoux 存在チェック）新規 |
| Req 8: テスト | `sakura_script/basic_test.rs` | **部分対応**: テストパターン確立済み、テストケース新規 |

### 技術的課題

#### 課題1: さくらスクリプトタグの位置保存アルゴリズム

budoux は平文で分割位置を判定するが、実際の入力にはさくらスクリプトタグが散在する。タグを除去して budoux に渡し、結果を元の文字列にマッピングし直す処理が核心。

**アプローチ候補**:
- (a) バイトオフセットマッピング: タグ位置を記録 → 平文を budoux で分割 → オフセットで再挿入
- (b) トークン列ベース: 既存 `Tokenizer` でトークン化 → SakuraScript 以外を連結して budoux に渡す → トークン列に改行を挿入

**観察**: 方式 (b) は既存 `Tokenizer` を直接再利用できるが、budoux の分割境界がトークン粒度（文字単位）と合致しないケースに注意が必要。

#### 課題2: budoux 分割と幅計算の統合

budoux は「分割可能な位置」を返すが、「どの位置で実際に分割すべきか」は幅閾値に依存する。budoux の出力ワード列を幅閾値と照合して改行を挿入するロジックが必要。

**複雑度**: 低〜中。ワード列を順に幅加算し、閾値超過時に改行を挿入する貪欲法で十分。

#### 課題3: `\n` の扱い

要件では改行文字として `\n` を使用するが、これはさくらスクリプトの改行タグでもある。`Tokenizer::SAKURA_TAG_PATTERN` は `\n` をさくらスクリプトタグとしてマッチする。budoux が挿入した `\n` と元々あった `\n` の区別は不要（両方とも改行として機能する）。

---

## 3. 実装アプローチ選択肢

### Option A: 既存モジュール拡張

`sakura_script/mod.rs` の `register()` に新関数 `break_lines` を追加。ロジックは `wait_inserter.rs` に隣接して `line_breaker.rs` として新規ファイル作成。

**変更対象**:
- `crates/pasta_lua/Cargo.toml` — 依存追加
- `crates/pasta_lua/src/sakura_script/mod.rs` — `break_lines` 関数登録
- `crates/pasta_lua/src/sakura_script/line_breaker.rs` — 新規（改行挿入ロジック）

**Lua 側呼出**:
```lua
local SAKURA = require "@pasta_sakura_script"
local result = SAKURA.talk_to_script(actor, text)
if actor.budoux then
    result = SAKURA.break_lines(result, actor.budoux)
end
```

**トレードオフ**:
- ✅ `@pasta_sakura_script` モジュールに統合 — Lua 側の `require` が増えない
- ✅ `Tokenizer` を同モジュール内で直接再利用
- ✅ 既存パターン（Arc 状態パターン、config 伝搬）をそのまま踏襲
- ✅ Lua 側の呼出判定がシンプル（actor.budoux の有無チェック）
- ❌ `@pasta_sakura_script` モジュールの責務が増える

### Option B: 独立モジュール作成

`@pasta_budoux` として新規 Lua モジュールを作成。`crates/pasta_lua/src/budoux/` ディレクトリ新設。

**変更対象**:
- `crates/pasta_lua/Cargo.toml` — 依存追加
- `crates/pasta_lua/src/budoux/mod.rs` — 新規（モジュール登録）
- `crates/pasta_lua/src/budoux/line_breaker.rs` — 新規（改行挿入ロジック）
- `crates/pasta_lua/src/runtime/module_registry.rs` — 新モジュール登録

**Lua 側呼出**:
```lua
local BUDOUX = require "@pasta_budoux"
local result = SAKURA.talk_to_script(actor, text)
if actor.budoux then
    result = BUDOUX.break_lines(result, actor.budoux)
end
```

**トレードオフ**:
- ✅ 単一責任原則の遵守
- ✅ budoux 固有の状態管理が独立
- ❌ 新規 Lua モジュール追加（`package.loaded` に登録）
- ❌ `Tokenizer` の再利用に `pub(crate)` 可視性調整が必要
- ❌ `module_registry.rs` への登録コード追加

### Option C: ハイブリッドアプローチ（推奨）

ロジックは `sakura_script/line_breaker.rs` に新規ファイルとして実装（Option A と同じ場所）。Lua 公開は `@pasta_sakura_script` モジュールへの関数追加（Option A）。ただし、`line_breaker.rs` 内部は独立した純粋関数として実装し、将来の切り出しを容易にする。

**変更対象**:
- `crates/pasta_lua/Cargo.toml` — 依存追加
- `crates/pasta_lua/src/sakura_script/line_breaker.rs` — 新規（独立ロジック）
- `crates/pasta_lua/src/sakura_script/mod.rs` — `break_lines` 関数を `@pasta_sakura_script` テーブルに追加

**トレードオフ**:
- ✅ `wait_inserter.rs` と同じディレクトリ・同じ公開パターン（要件 Req 6 と完全一致）
- ✅ ロジックは独立ファイルで単体テスト容易
- ✅ `Tokenizer` + `SAKURA_TAG_PATTERN` を同一モジュール内で直接利用
- ✅ Lua 公開が最小差分（`register()` に1関数追加のみ）
- ✅ 将来の独立モジュール化が容易（ファイル移動のみ）
- ❌ `@pasta_sakura_script` モジュールの関数が増える（軽微）

---

## 4. 実装複雑度・リスク

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **S〜M**（2〜5日） | 新規アルゴリズムは中程度だが、既存パターンへの適合は容易。外部クレート導入も単純 |
| **リスク** | **Low** | 既存パターン踏襲、独立機能、破壊的変更なし。budoux / unicode-width は安定クレート |

### リスク詳細

| リスク | 評価 | 対策 |
|--------|------|------|
| budoux クレートの API 互換性 | Low | v0.1.1 安定。`budoux::parse(model, text)` のみ使用 |
| unicode-width の CJK 幅精度 | Low | v0.2.2 安定。`width_cjk()` は Unicode Annex #11 準拠 |
| さくらスクリプトタグ再挿入の正確性 | Medium | トークン列ベースの方式で位置ずれリスクを最小化 |
| パフォーマンス | Low | budoux は O(n) の線形処理。トーク文字列は通常数百文字程度 |

---

## 5. 設計フェーズへの持ち越し事項

### 確定事項
- **pasta.toml 設定**: 追加コード不要（TOML パススルーで自動伝搬）
- **Lua API**: `@pasta_sakura_script` モジュールに `break_lines` 関数追加
- **ファイル配置**: `sakura_script/line_breaker.rs` 新規

### Research Needed（設計フェーズで検討）
1. **タグ位置マッピングの具体的アルゴリズム**: トークン列ベース vs バイトオフセットの最終決定
2. **budoux 分割結果とウェイトタグの相互作用**: `\_w[50]` が2文字以上つながった場合の分割境界の扱い
3. **Lua 側の呼出統合**: `talk_to_script()` 内部に組み込むか、外部呼出にするかの最終判断
4. **テストの配置**: `tests/sakura_script/` 配下に追加か、新サブディレクトリ作成か
