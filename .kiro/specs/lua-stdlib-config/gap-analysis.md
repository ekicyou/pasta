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
    pub enable_std_libs: bool,      // 標準ライブラリ一括有効化
    pub enable_assertions: bool,    // mlua-stdlib @assertions
    pub enable_testing: bool,       // mlua-stdlib @testing
    pub enable_env: bool,           // mlua-stdlib @env（unsafe）
    pub enable_regex: bool,         // mlua-stdlib @regex
    pub enable_json: bool,          // mlua-stdlib @json
    pub enable_yaml: bool,          // mlua-stdlib @yaml
}
```

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
| Req1: Cargo風配列記法 | `LuaStdLibConfig`構造体 + TOML配列解析 | **Missing** |
| Req2: 減算記法 | 加算/減算処理ロジック | **Missing** |
| Req3: debug警告 | 警告ログ出力 | **Missing** |
| Req4: バリデーション | serde + カスタムバリデータ | **Missing** |
| Req5: 後方互換性 | デフォルト値、既存API維持 | **OK（設計配慮必要）** |

### 2.2 mlua StdLibフラグ対応（Lua 5.5）

| 要件ライブラリ | mlua定数 | Lua 5.5対応 | セーフティ |
|--------------|----------|------------|-----------|
| coroutine | `StdLib::COROUTINE` | ✅ | Safe |
| table | `StdLib::TABLE` | ✅ | Safe |
| io | `StdLib::IO` | ✅ | Safe |
| os | `StdLib::OS` | ✅ | Safe |
| string | `StdLib::STRING` | ✅ | Safe |
| utf8 | `StdLib::UTF8` | ✅ | Safe |
| math | `StdLib::MATH` | ✅ | Safe |
| package | `StdLib::PACKAGE` | ✅ | Safe |
| debug | `StdLib::DEBUG` | ✅ | **Unsafe** |

**確定事項**: `ALL_SAFE = StdLib((1 << 30) - 1)` = 全ライブラリ - DEBUG - FFI  
io, os, packageは`ALL_SAFE`に含まれる（safeライブラリ扱い）

### 2.3 制約と課題

| 制約/課題 | 詳細 | 対応方針 |
|---------|------|---------|
| 既存`enable_std_libs`フラグ | bool一括制御のみ | 拡張または置換 |
| `Lua::new()` vs `unsafe_new_with` | 現状の分岐ロジック | `StdLib`フラグ動的構築に変更 |
| テスト互換性 | 多数のテストが`Lua::new()`使用 | テスト影響軽微 |

---

## 3. 実装アプローチオプション

### Option A: RuntimeConfig拡張

**概要**: 既存の`RuntimeConfig`に`LuaStdLibConfig`をネストして追加

**変更対象:**
- `runtime/mod.rs`: `RuntimeConfig`にフィールド追加
- `runtime/mod.rs`: `with_config`でStdLib構築ロジック追加
- `loader/config.rs`: `[lua.stdlib]`セクション解析追加

**コード例:**
```rust
pub struct RuntimeConfig {
    pub stdlib: LuaStdLibConfig,  // NEW
    pub enable_assertions: bool,
    pub enable_testing: bool,
    // ... existing fields
}

pub struct LuaStdLibConfig {
    pub preset: StdLibPreset,
    pub allow_unsafe: bool,
    pub coroutine: Option<bool>,
    pub table: Option<bool>,
    // ... individual options
}
```

**トレードオフ:**
- ✅ 最小限のファイル変更
- ✅ 既存テストへの影響最小
- ✅ `enable_std_libs`を`stdlib.preset`で代替可能
- ❌ `RuntimeConfig`が肥大化
- ❌ ファイル設定とコード設定の統合が複雑

---

### Option B: 設定階層の分離

**概要**: `LuaStdLibConfig`を独立構造体として、`PastaConfig`に追加

**変更対象:**
- `loader/config.rs`: `LuaStdLibConfig`追加、`PastaConfig::lua_stdlib()`メソッド
- `runtime/mod.rs`: `RuntimeConfig`から`enable_std_libs`削除、`LuaStdLibConfig`受け取り
- `loader/mod.rs`: ローダーで設定統合

**コード例:**
```rust
// loader/config.rs
pub struct LuaStdLibConfig { ... }

impl PastaConfig {
    pub fn lua_stdlib(&self) -> Option<LuaStdLibConfig> {
        self.custom_fields.get("lua")
            .and_then(|v| v.get("stdlib"))
            .and_then(|v| v.clone().try_into().ok())
    }
}

// runtime/mod.rs
impl PastaLuaRuntime {
    pub fn with_stdlib_config(context, stdlib: LuaStdLibConfig) -> LuaResult<Self> { ... }
}
```

**トレードオフ:**
- ✅ 設定とランタイムの責務分離
- ✅ TOML設定に自然にマップ
- ✅ `PastaConfig`の既存パターンに準拠
- ❌ 既存の`RuntimeConfig::enable_std_libs`との整合性
- ❌ API変更が広範

---

### Option C: ハイブリッドアプローチ（推奨）

**概要**: `LuaStdLibConfig`を`loader/config.rs`に追加し、`RuntimeConfig`はそれを参照

**変更対象:**
- `loader/config.rs`: `LuaStdLibConfig`構造体追加
- `runtime/mod.rs`: `RuntimeConfig`に`pub stdlib: LuaStdLibConfig`追加
- `runtime/mod.rs`: Lua VM初期化ロジック更新
- `lib.rs`: `LuaStdLibConfig`をre-export

**段階的実装:**
1. Phase 1: `LuaStdLibConfig`追加、デフォルト値で後方互換維持
2. Phase 2: TOML解析統合
3. Phase 3: `enable_std_libs`を`stdlib.preset`で代替（非推奨化）

**コード例:**
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
| 全体 | **M（3-7日）** | 新規構造体追加 + 既存統合 |
| `LuaStdLibConfig`構造体 | S（1日） | serde derive活用 |
| StdLib変換ロジック | S（1日） | ビットフラグ操作 |
| TOML解析統合 | S（1日） | 既存パターン踏襲 |
| RuntimeConfig統合 | M（2日） | 後方互換維持が複雑 |
| テスト作成 | M（2日） | 網羅的テスト必要 |

### リスク評価

| リスク | レベル | 緩和策 |
|-------|-------|-------|
| 後方互換性破壊 | **中** | デフォルト値で既存動作維持 |
| unsafeフラグの誤設定 | **低** | バリデーション + 警告ログ |
| テストリグレッション | **低** | 既存テストは`RuntimeConfig::new()`使用 |
| mlua API変更 | **低** | mlua 0.11はstable |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option C（ハイブリッド）

**理由:**
1. 既存の`PastaConfig`パターン（`logging()`, `persistence()`）に準拠
2. 段階的移行で後方互換性維持
3. ランタイム設定とファイル設定の責務分離

### 設計フェーズで調査が必要な項目

| 調査項目 | 目的 |
|---------|------|
| `StdLib::ALL_SAFE`の正確な内容 | io/os/packageが含まれるか確認 |
| `Lua::new()`の初期化ライブラリ | デフォルト挙動の確認 |
| serde flatten vs nested | TOML構造の最適化 |

### 次のステップ

1. **要件承認**: `/kiro-spec-requirements lua-stdlib-config`で承認
2. **設計フェーズ**: `/kiro-spec-design lua-stdlib-config -y`で設計ドキュメント生成
