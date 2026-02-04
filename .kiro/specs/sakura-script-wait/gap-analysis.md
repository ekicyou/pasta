# Gap Analysis Report: sakura-script-wait

## エグゼクティブサマリー

本レポートは、`sakura-script-wait`機能の要件と既存コードベースとのギャップを分析する。

### 主要所見

- **既存基盤は整備済み**: `@pasta_*`モジュールパターン（search, persistence, config, enc）が確立されており、新モジュール追加は低リスク
- **設定拡張は容易**: `PastaConfig`は`custom_fields`で任意のTOMLセクションを扱える設計。`[talk]`セクション追加は既存パターンに従う
- **トークン分解は新規実装**: さくらスクリプトタグと日本語文字種別の分類ロジックは新規開発が必要
- **正規表現は利用可能**: `mlua-stdlib`経由でLua側regex利用可能。Rust側では`regex`クレート追加が必要

---

## 1. 現状調査

### 1.1 既存モジュール構造

| モジュール           | 配置                         | 登録パターン                         |
| -------------------- | ---------------------------- | ------------------------------------ |
| `@pasta_search`      | `src/search/mod.rs`          | `loader()` + `register()` → UserData |
| `@pasta_persistence` | `src/runtime/persistence.rs` | `register()` → Table                 |
| `@pasta_config`      | `src/runtime/mod.rs`         | `toml_to_lua()` → Table              |
| `@enc`               | `src/runtime/enc.rs`         | `register()` → Table                 |
| `@pasta_sakura_script` (新規) | `src/sakura_script/mod.rs` (予定) | `register()` → Table |

**パターン**: 全モジュールが`package.loaded["@module_name"]`に登録される。

### 1.2 設定管理 (PastaConfig)

```rust
// crates/pasta_lua/src/loader/config.rs
pub struct PastaConfig {
    pub loader: LoaderConfig,        // [loader]セクション専用
    pub custom_fields: toml::Table,  // その他全セクション
}
```

**既存アクセサパターン**:
```rust
pub fn logging(&self) -> Option<LoggingConfig>
pub fn persistence(&self) -> Option<PersistenceConfig>
pub fn lua(&self) -> Option<LuaConfig>
```

**`[talk]`セクション追加**: 同様のアクセサ`talk() -> Option<TalkConfig>`を追加すれば整合性を保てる。

### 1.3 sakura_builder.lua の現状

```lua
-- crates/pasta_sample_ghost/.../sakura_builder.lua
function BUILDER.build(grouped_tokens, config)
    for _, token in ipairs(grouped_tokens) do
        if inner_type == "talk" then
            table.insert(buffer, escape_sakura(inner.text))
        end
    end
end
```

**統合ポイント**: `talk`トークン処理時に`WAIT.talk_to_script(actor, text)`を呼び出す箇所。

### 1.4 依存関係

| 依存                 | 現状     | 必要性                              |
| -------------------- | -------- | ----------------------------------- |
| `regex` (Rust)       | 未使用   | さくらスクリプトタグ検出に必要      |
| `mlua-stdlib::regex` | 登録済み | Lua側では使用可能だが、Rust実装推奨 |

---

## 2. 要件実現可能性分析

### Requirement 1: Luaモジュール公開

| 技術要素              | ギャップ | 対応策                        |
| --------------------- | -------- | ----------------------------- |
| モジュール登録        | なし     | `@pasta_search`パターン踏襲   |
| 関数エクスポート      | なし     | `lua.create_function()`       |
| actorテーブル読み取り | なし     | `Table::get::<String, i64>()` |

**ギャップ**: なし（既存パターンで実現可能）

### Requirement 2: actorパラメーター参照

| 技術要素                   | ギャップ | 対応策                            |
| -------------------------- | -------- | --------------------------------- |
| Luaテーブルからi64取得     | なし     | `table.get("script_wait_normal")` |
| デフォルト値フォールバック | なし     | `Option::unwrap_or(default)`      |
| 型検証                     | なし     | `Value::as_integer()`             |

**ギャップ**: なし

### Requirement 3: pasta.toml設定

| 技術要素               | ギャップ | 対応策                 |
| ---------------------- | -------- | ---------------------- |
| `[talk]`セクション定義 | 新規     | `TalkConfig`構造体定義 |
| デフォルト値定義       | 新規     | `Default` trait実装    |
| アクセサ追加           | 新規     | `PastaConfig::talk()`  |

**ギャップ**: 新規実装必要だが、既存パターン(`LoggingConfig`等)に従うため低リスク。

### Requirement 4: トークン分解

| 技術要素                     | ギャップ | 対応策                   |
| ---------------------------- | -------- | ------------------------ |
| さくらスクリプトタグ正規表現 | 新規     | `regex`クレート使用      |
| Unicode文字分類              | 新規     | 文字セット照合ロジック   |
| トークナイザー実装           | 新規     | 状態機械またはイテレータ |

**ギャップ**: コア機能。新規実装必要。

**正規表現パターン**:
```regex
\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?
```
- `\h`, `\s[0]`, `\_w[100]`, `\![set,...]` 等の一般的なさくらスクリプトタグにマッチ

### Requirement 5: ウェイト挿入ルール

| 技術要素             | ギャップ | 対応策                   |
| -------------------- | -------- | ------------------------ |
| ウェイト計算ロジック | 新規     | `(wait - 50).max(0)`     |
| 連続トークン処理     | 新規     | 状態変数で最大値追跡     |
| 出力文字列生成       | なし     | `format!("\_w[{}]", ms)` |

**ギャップ**: アルゴリズム実装必要。

### Requirement 6: エラーハンドリング

| 技術要素         | ギャップ | 対応策                    |
| ---------------- | -------- | ------------------------- |
| nil/空文字列処理 | なし     | 早期return                |
| フォールバック   | なし     | `Result`/`Option`パターン |
| ログ出力         | なし     | `tracing::warn!`          |

**ギャップ**: なし

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張

**対象**: `src/runtime/` に `sakura_wait.rs` 追加

```
src/runtime/
├── mod.rs          # register_sakura_wait_module() 追加
├── enc.rs
├── persistence.rs
├── finalize.rs
└── sakura_wait.rs  # 新規
```

**トレードオフ**:
- ✅ 既存パターンに沿った配置
- ✅ `mod.rs` の初期化フローに統合しやすい
- ❌ `runtime/` の責務が広がる

### Option B: 新規モジュールディレクトリ

**対象**: `src/sakura_wait/` として独立

```
src/
├── runtime/
├── search/
├── sakura_wait/   # 新規ディレクトリ
│   ├── mod.rs     # 公開API
│   ├── config.rs  # TalkConfig
│   ├── tokenizer.rs  # トークン分解
│   └── wait_inserter.rs  # ウェイト挿入
```

**トレードオフ**:
- ✅ 明確な責務分離
- ✅ テスト独立性
- ✅ 将来的な拡張性
- ❌ ファイル数増加

### Option C: ハイブリッドアプローチ（推奨）

**戦略**:
1. コアロジック: `src/sakura_wait/` として独立モジュール
2. 設定: `src/loader/config.rs` に `TalkConfig` 追加
3. 登録: `src/runtime/mod.rs` から呼び出し

```
src/
├── loader/
│   └── config.rs      # TalkConfig追加
├── runtime/
│   └── mod.rs         # register_sakura_wait_module()
└── sakura_wait/       # 新規
    ├── mod.rs         # loader() + register()
    ├── tokenizer.rs   # トークン分解ロジック
    └── wait_inserter.rs  # ウェイト挿入ロジック
```

**トレードオフ**:
- ✅ 設定管理は既存パターン踏襲
- ✅ コアロジックは独立でテスト可能
- ✅ 登録は既存フローに統合

---

## 4. 実装複雑度・リスク評価

### 工数見積もり: **M (3-7日)**

| タスク                         | 工数  |
| ------------------------------ | ----- |
| TalkConfig定義・pasta.toml対応 | 0.5日 |
| トークナイザー実装             | 1-2日 |
| ウェイト挿入ロジック           | 1日   |
| Luaモジュール公開              | 0.5日 |
| テスト実装                     | 1-2日 |

**根拠**: 新規アルゴリズム実装（トークナイザー）があるが、既存パターンに従う部分が多い。

### リスク評価: **Medium**

| リスク                   | 影響                       | 緩和策                       |
| ------------------------ | -------------------------- | ---------------------------- |
| 正規表現パターンの網羅性 | さくらスクリプトタグ見逃し | 実際のスクリプトでテスト     |
| Unicode文字分類の正確性  | 誤分類によるウェイト不正   | 文字セット定義をpasta.toml化 |
| パフォーマンス           | 長文処理での遅延           | ベンチマーク実施             |

---

## 5. 設計フェーズへの引き継ぎ事項

### 確定事項

1. **モジュール名**: `@pasta_sakura_script_wait`
2. **設定セクション**: `[talk]`
3. **実装方式**: Option C（ハイブリッドアプローチ）

### 設計フェーズで決定すべき事項

1. **さくらスクリプト正規表現の最終形**
   - `\![...]` のネスト対応要否
   - エスケープシーケンス（`\\`）の扱い

2. **トークナイザーのAPI設計**
   - `Vec<Token>` vs イテレータ
   - エラー表現方法

3. **TalkConfig構造体の詳細設計**
   - フィールド名の最終決定
   - バリデーションルール

4. **テスト戦略**
   - ユニットテスト範囲
   - 統合テストシナリオ

### 調査継続事項

- **Research Needed**: さくらスクリプトタグの完全な仕様確認（UKADOC参照）

---

## 6. 次のステップ

ギャップ分析完了。以下のコマンドで設計フェーズへ進行:

```bash
/kiro-spec-design sakura-script-wait -y
```
