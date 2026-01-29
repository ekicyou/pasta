# Gap Analysis: lua-stdlib-config

## 1. 現状調査（Current State Investigation）

### 1.1 関連アセットの構造

| ファイル/モジュール | 役割 | 変更影響度 |
|------------------|------|-----------|
| [runtime/mod.rs](../../../crates/pasta_lua/src/runtime/mod.rs) | `RuntimeConfig`構造体、`PastaLuaRuntime`定義 | **高** |
| [loader/config.rs](../../../crates/pasta_lua/src/loader/config.rs) | `PastaConfig`/`LoaderConfig`TOML解析 | **高** |
| [lib.rs](../../../crates/pasta_lua/src/lib.rs) | 公開API定義、re-export | 中 |
| [Cargo.toml](../../../crates/pasta_lua/Cargo.toml) | 依存関係（mlua lua55） | 低 |

### 1.2 既存パターンと規約

**Lua VM初期化パターン:**
```rust
// 現在のパターン (runtime/mod.rs:160)
let lua = if config.enable_std_libs {
    unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }
} else {
    Lua::new()
};
```

**TOML設定パターン:**
- `PastaConfig` - メイン設定（`pasta.toml`）
- `LoaderConfig` - `[loader]`セクション（serde derive）
- `custom_fields: toml::Table` - 未知のセクションを保持

**既存RuntimeConfig構造:**
```rust
pub struct RuntimeConfig {
    pub enable_std_libs: bool,      // 標準ライブラリ一括有効化 → libs配列で置き換え
    pub enable_assertions: bool,    // mlua-stdlib @assertions → libs配列で置き換え
    pub enable_testing: bool,       // mlua-stdlib @testing → libs配列で置き換え
    pub enable_env: bool,           // mlua-stdlib @env（unsafe） → libs配列で置き換え
    pub enable_regex: bool,         // mlua-stdlib @regex → libs配列で置き換え
    pub enable_json: bool,          // mlua-stdlib @json → libs配列で置き換え
    pub enable_yaml: bool,          // mlua-stdlib @yaml → libs配列で置き換え
}
```
**→ すべてのフラグを`libs`配列に統合予定**

### 1.3 統合サーフェス

| コンポーネント | 統合ポイント | 備考 |
|--------------|-------------|------|
| `PastaLoader` | `load_with_config(base_dir, RuntimeConfig)` | RuntimeConfigを受け取る |
| `PastaLuaRuntime::with_config` | `RuntimeConfig`からLua VM作成 | StdLib設定の適用箇所 |
| `PastaConfig::parse` | TOML解析、セクション抽出 | `[lua.stdlib]`追加ポイント |

---

## 2. 要件実現可能性分析

### 2.1 技術的要件マッピング

| 要件 | 必要な実装 | ギャップ状態 |
|-----|-----------|------------|
| Req1: Cargo風配列記法 | `LuaLibConfig`構造体 + TOML配列解析 | **Missing** |
| Req2: 減算記法 | 加算/減算処理ロジック | **Missing** |
| Req3: セキュリティ警告 | std_debug/env警告ログ | **Missing** |
| Req4: バリデーション | serde + カスタムバリデータ | **Missing** |
| Req5: 後方互換性 | デフォルト値、既存フラグ非推奨化 | **OK（設計配慮必要）** |

### 2.2 mlua StdLibフラグ対応（Lua 5.5）

| ライブラリ（TOML要素） | mlua定数 | Lua 5.5対応 | セーフティ |
|--------------|----------|------------|-----------|
| std_coroutine | `StdLib::COROUTINE` | ✅ | Safe |
| std_table | `StdLib::TABLE` | ✅ | Safe |
| std_io | `StdLib::IO` | ✅ | Safe |
| std_os | `StdLib::OS` | ✅ | Safe |
| std_string | `StdLib::STRING` | ✅ | Safe |
| std_utf8 | `StdLib::UTF8` | ✅ | Safe |
| std_math | `StdLib::MATH` | ✅ | Safe |
| std_package | `StdLib::PACKAGE` | ✅ | Safe |
| std_debug | `StdLib::DEBUG` | ✅ | **Unsafe** |

### 2.3 mlua-stdlibモジュール対応

| モジュール（TOML要素） | mlua-stdlib | 備考 |
|--------------|-----------|------|
| assertions | `@assertions` | アサーション・検証 |
| testing | `@testing` | テストフレームワーク |
| env | `@env` | 環境変数・FS（セキュリティ注意） |
| regex | `@regex` | 正規表現 |
| json | `@json` | JSON |
| yaml | `@yaml` | YAML |

**確定事項**: `ALL_SAFE = StdLib((1 << 30) - 1)` = 全ライブラリ - DEBUG - FFI  
std_io, std_os, std_packageは`ALL_SAFE`に含まれる（safeライブラリ扱い）

### 2.4 制約と課題

| 制約/課題 | 詳細 | 対応方針 |
|---------|------|---------|
| 既存個別フラグ | `enable_std_libs`, `enable_testing`等 | `libs`配列で置換・非推奨化 |
| `Lua::new()` vs `unsafe_new_with` | 現状の分岐ロジック | `StdLib`フラグ動的構築に変更 |
| mlua-stdlibモジュール登録 | 個別フラグで制御中 | 動的登録に変更 |

---

## 3. 実装アプローチオプション

### Option A: RuntimeConfig完全置換

**概要**: 既存の`RuntimeConfig`の個別フラグを廃止し、`LuaLibConfig`に統合

**変更対象:**
- `runtime/mod.rs`: `RuntimeConfig`のすべてのフラグを`libs: Vec<String>`に置換
- `runtime/mod.rs`: `with_config`でStdLib構築+mlua-stdlib登録ロジック追加
- `loader/config.rs`: `[lua]`セクション解析追加

**コード例:**
```rust
pub struct RuntimeConfig {
    pub libs: Vec<String>,  // NEW: ["std_all", "testing", "regex", ...}
}

impl RuntimeConfig {
    /// libs配列からStdLib（Lua標準ライブラリ）を構築
    pub fn to_stdlib(&self) -> Result<StdLib, ConfigError> {
        let mut result = StdLib::NONE;
        let mut subtract = StdLib::NONE;
        
        for lib in &self.libs {
            if let Some(name) = lib.strip_prefix('-') {
                // 減算記法
                subtract |= Self::parse_std_lib(name)?;
            } else {
                // 加算記法
                result |= Self::parse_std_lib(lib)?;
            }
        }
        
        Ok(result & !subtract)
    }
    
    fn parse_std_lib(name: &str) -> Result<StdLib, ConfigError> {
        match name {
            "std_all" => Ok(StdLib::ALL_SAFE),
            "std_all_unsafe" => Ok(StdLib::ALL),
            "std_coroutine" => Ok(StdLib::COROUTINE),
            "std_table" => Ok(StdLib::TABLE),
            "std_io" => Ok(StdLib::IO),
            "std_os" => Ok(StdLib::OS),
            "std_string" => Ok(StdLib::STRING),
            "std_utf8" => Ok(StdLib::UTF8),
            "std_math" => Ok(StdLib::MATH),
            "std_package" => Ok(StdLib::PACKAGE),
            "std_debug" => Ok(StdLib::DEBUG),
            _ => Err(ConfigError::UnknownLibrary(name.to_string())),
        }
    }
    
    /// mlua-stdlibモジュール登録が必要か確認
    pub fn should_enable_module(&self, module: &str) -> bool {
        self.libs.iter().any(|lib| {
            lib == module || (!lib.starts_with('-') && lib == module)
        }) && !self.libs.iter().any(|lib| lib == &format!("-{}", module))
    }
}
```

**メリット:**
- すべてのライブラリ設定を1つの配列に統一
- 既存の個別フラグを段階的に廃止可能
- 将来的な拡張性が高い

**デメリット:**
- 既存コードの影響範囲が広い
- 移行期間の互換性維持が必要

**トレードオフ:**
- ✅ 設計の統一性（1つの配列ですべて制御）
- ✅ 将来の拡張性（新しいライブラリ追加が容易）
- ✅ Cargo.toml featuresとの類似性（学習コスト低）
- ❌ 既存テストの修正範囲が広い
- ❌ 移行期間の後方互換処理が必要

---

### Option B: 段階的移行アプローチ

**概要**: 既存フラグを残したまま、`libs`配列を優先する設計

**変更対象:**
- `runtime/mod.rs`: `RuntimeConfig`に`libs: Option<Vec<String>>`追加
- `runtime/mod.rs`: `libs`が指定されていれば優先、なければ既存フラグ使用
- `loader/config.rs`: `[lua]`セクション解析追加

**コード例:**
```rust
pub struct RuntimeConfig {
    pub libs: Option<Vec<String>>,  // NEW（優先）
    #[deprecated = "Use libs array instead"]
    pub enable_std_libs: bool,
    #[deprecated = "Use libs array instead"]
    pub enable_testing: bool,
    // ... other flags
}

impl RuntimeConfig {
    pub fn effective_libs(&self) -> Vec<String> {
        if let Some(libs) = &self.libs {
            libs.clone()
        } else {
            // 既存フラグからlibsを生成（後方互換）
            let mut result = vec![];
            if self.enable_std_libs {
                result.push("std_all".to_string());
            }
            if self.enable_testing {
                result.push("testing".to_string());
            }
            // ... 他のフラグ
            result
        }
    }
}
```

**メリット:**
- 完全な後方互換性維持
- 段階的な移行が可能

**デメリット:**
- 過渡期のコード複雑化
- 二重メンテナンス期間が発生

**トレードオフ:**
- ✅ 既存コードへの影響ゼロ
- ✅ ユーザー側の移行コスト最小
- ❌ コードベースの二重管理
- ❌ 最終的には個別フラグ削除が必要

---

### 推奨アプローチ: **Option A (完全置換)**

統合設計の観点から、すべてのライブラリ設定を`libs`配列に統一する**Option A**を推奨します。

**理由:**
1. **設計の一貫性**: 1つの配列ですべてのライブラリ（Lua標準+mlua-stdlib）を制御
2. **Cargo.toml featuresとの親和性**: ユーザーにとって直感的
3. **将来の拡張性**: 新しいライブラリ追加が容易
4. **技術的負債の削減**: 過渡期の二重管理を避ける

**移行戦略:**
- デフォルト値で既存動作を再現（`["std_all", "assertions", "testing", "regex", "json", "yaml"]`）
- 既存の個別フラグは `#[deprecated]` マークし、ドキュメントで移行方法を説明
- テストは新しい`libs`配列ベースに段階的に更新

---
```rust
// loader/config.rs (または新規 stdlib_config.rs)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LuaStdLibConfig {
    #[serde(default = "default_stdlib")]
    pub libs: Vec<String>,  // ["all"], ["coroutine", "table"], etc.
}

fn default_stdlib() -> Vec<String> {
    vec!["all".to_string()]
}

impl LuaStdLibConfig {
    pub fn to_stdlib(&self) -> mlua::StdLib {
        let mut result = StdLib::NONE;
        let mut subtractions = StdLib::NONE;
        
        for lib in &self.libs {
            if let Some(name) = lib.strip_prefix('-') {
                // 減算記法
                subtractions |= parse_lib_name(name);
            } else {
                // 加算記法
                result |= parse_lib_name(lib);
            }
        }
        
        result & !subtractions
    }
}

// runtime/mod.rs
pub struct RuntimeConfig {
    pub stdlib: LuaStdLibConfig,
    // ... other fields
}
```

impl LuaStdLibConfig {
    pub fn to_stdlib(&self) -> mlua::StdLib {
        // プリセット + 個別オプション + unsafe検証 → StdLibフラグ
    }
}

// runtime/mod.rs
pub struct RuntimeConfig {
    pub stdlib: LuaStdLibConfig,
    // ... other fields (enable_std_libs deprecated)
}
```

**トレードオフ:**
- ✅ 段階的移行が可能
- ✅ 後方互換性維持
- ✅ 既存パターンに準拠
- ✅ テスト影響最小
- ❌ 一時的にAPI重複（`enable_std_libs` + `stdlib`）

---

## 4. 実装複雑度とリスク

### 工数見積もり

| 項目 | 見積もり | 根拠 |
|-----|---------|------|
| 全体 | **M（3-5日）** | RuntimeConfig統合 + mlua-stdlib登録 |
| `libs`配列パース処理 | S（0.5日） | serde Vec<String> |
| StdLib変換ロジック | S（1日） | ビットフラグ操作 + 減算処理 |
| mlua-stdlib動的登録 | M（1日） | 条件分岐追加 |
| TOML解析統合 | S（0.5日） | 既存パターン踏襲 |
| RuntimeConfig統合 | M（1-2日） | 既存フラグ非推奨化 |
| テスト作成 | M（1日） | 配列パース、減算記法、バリデーション |

### リスク評価

| リスク | レベル | 緩和策 |
|-------|-------|-------|
| 後方互換性破壊 | **低** | デフォルト値で既存動作維持 |
| std_debug/env誤設定 | **中** | バリデーション + 警告ログ |
| 既存テスト影響 | **中** | 既存フラグからlibsへ移行 |
| mlua API変更 | **低** | mlua 0.11はstable |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（完全置換）

**理由:**
1. すべてのライブラリ設定を`libs`配列に統一（Lua標準+mlua-stdlib）
2. Cargo.toml featuresとの高い親和性
3. 将来の拡張性（新しいライブラリ追加が容易）
4. 技術的負債の削減（過渡期の二重管理を避ける）

### 設計フェーズで決定が必要な項目

| 検討項目 | 選択肢 |
|---------|------|
| デフォルト値 | `["std_all", "assertions", "testing", "regex", "json", "yaml"]` vs 最小構成 |
| 既存フラグ扱い | 即座に非推奨 vs 段階的移行 |
| バリデーション戦略 | 厳格（unknown error） vs 寛容（unknown warning） |
| 警告ログレベル | `tracing::warn` vs `tracing::info` |

### 次のステップ

1. **要件承認**: 議題クローズ後、`/kiro-spec-design lua-stdlib-config -y`
2. **設計フェーズ**: 構造体定義、変換ロジック、TOML schema確定
