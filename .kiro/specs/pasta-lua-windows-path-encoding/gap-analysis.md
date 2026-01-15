# Gap Analysis Document

## 分析サマリー

**重要な発見**: カスタムサーチャー実装は不要。`package.path`にANSIバイト列を設定するだけで解決可能。

本ギャップ分析は、Windows環境でのLuaモジュールロード時のパスエンコーディング問題を解決するための実装戦略を検討した結果、当初想定していたカスタムサーチャー実装が不要であることを確認した。要件は以下の2領域に簡略化される:

1. **package.path設定の修正** (Req 1-4): ANSIバイト列を`Lua::create_string`で設定
2. **@encモジュール実装** (Req 5-7): Lua側へのエンコーディング変換API公開
3. **後方互換性維持** (Req 8): 既存API変更なし

**工数見積**: **S (1-2日)** → **Risk: Low**

既存コードベースには`@pasta_search`モジュールの実装例があり、`@enc`実装に流用可能。主な作業は`encoding/mod.rs`と`loader/context.rs`、`runtime/mod.rs`の軽微な修正のみ。

---

## 1. Current State Investigation

### 既存アーキテクチャ

**主要コンポーネント**:

```
crates/pasta_lua/src/
├── runtime/mod.rs          # PastaLuaRuntime - VM初期化、モジュール登録
├── loader/
│   ├── mod.rs              # PastaLoader - 起動シーケンス統合
│   └── context.rs          # LoaderContext - 検索パス管理
├── encoding/
│   ├── mod.rs              # Encoder trait, path_to_lua/path_from_lua
│   ├── windows.rs          # Windows API実装（MultiByteToWideChar等）
│   └── unix.rs             # UTF-8パススルー実装
└── search/
    ├── mod.rs              # @pasta_search登録パターン（参考実装）
    └── context.rs          # SearchContext - UserData実装例
```

**既存モジュール登録パターン** (`search/mod.rs`):
```rust
pub fn register(lua: &Lua, ...) -> LuaResult<AnyUserData> {
    let module = loader(lua, ...)?;
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set("@pasta_search", module.clone())?;
    Ok(module)
}
```

**RuntimeConfig設定** (`runtime/mod.rs`):
- `enable_std_libs`, `enable_assertions`, `enable_testing`等のフラグ
- `RuntimeConfig::new()`, `::full()`, `::minimal()`の3プリセット
- 各モジュール登録は`mlua_stdlib::*::register(&lua, None)?`形式

**LoaderContext統合** (`loader/context.rs`):
- `lua_search_paths: Vec<String>` - 相対パス
- `generate_package_path()` - `?.lua` / `?/init.lua`形式の文字列生成
- `absolute_search_paths()` - base_dir基準の絶対パス変換

**エンコーディングモジュール** (`encoding/mod.rs`):
- `Encoding::ANSI`, `Encoding::OEM`
- `Encoder` trait: `to_string(bytes) -> String`, `to_bytes(str) -> Vec<u8>`
- **重要なバグ発見**: `path_to_lua`が`String::from_utf8_lossy`を使用（ANSIバイト列を破壊）
- Windows: `windows-sys` 0.59依存
- Unix: UTF-8パススルー

**Lua文字列の性質**:
- Lua文字列は任意のバイト列を格納可能（UTF-8である必要なし）
- mluaの`Lua::create_string(&[u8])`はバイト列を直接受け入れる
- Luaの標準サーチャー（`package.searchers[2]`）は`package.path`のバイト列をそのまま`fopen`に渡す
- **結論**: `package.path`にANSIバイト列を設定すれば、標準サーチャーで問題解決

### 技術的制約

1. **mlua API**:
   - `Lua::create_string(&[u8])`で任意のバイト列をLua文字列化
   - `Table::set("path", lua_string)`で`package.path`設定
   - **カスタムサーチャー不要** - 標準サーチャーがANSIバイト列を正しく処理

2. **クロスプラットフォーム対応**:
   - Windows: `std::fs`がUTF-16 Wide API使用（`CreateFileW`相当）
   - Unix: UTF-8ネイティブなので特別な処理不要
   - `#[cfg(windows)]` / `#[cfg(not(windows))]`で条件分岐

3. **既存依存関係**:
   - `mlua` 0.11（既存）
   - `windows-sys` 0.59（既存、Windows専用）
   - `glob` 0.3（ファイル探索、既存）

### 命名・レイヤリング規約

- **モジュールディレクトリ**: `src/<module>/mod.rs`形式
- **エラー型**: `<Module>Error` (thiserror派生)
- **設定型**: `<Module>Config` (serde派生可能)
- **Luaモジュール名**: `@<name>` (例: `@pasta_search`, `@pasta_config`, `@enc`)
- **ログ**: `tracing::{debug, info, warn, error}`使用

---

## 2. Requirements Feasibility Analysis

### 技術的要求と既存機能のマッピング

| Requirement | 既存機能 | ギャップ | 複雑度 |
|-------------|---------|---------|--------|
| **Req 1: package.pathのANSIバイト列設定** | `Lua::create_string` | **Missing**: バイト列生成 | Low |
| **Req 2: encoding/mod.rs修正** | `Encoding::ANSI.to_bytes` | **Bug Fix**: `from_utf8_lossy`削除 | Low |
| **Req 3: LoaderContext対応** | `generate_package_path()` | **Missing**: バイト列版メソッド | Low |
| **Req 4: runtime設定修正** | `setup_package_path()` | **Modify**: バイト列使用 | Low |
| **Req 5: @encモジュール実装** | `@pasta_search`パターン | **Missing**: 新モジュール | Low |
| **Req 6: encoding再利用** | `encoding/mod.rs` | ✅ 既存実装を直接使用可能 | Low |
| **Req 7: @encドキュメント** | - | **Missing**: 新規モジュールテーブル | Low |
| **Req 8: 後方互換性** | 既存API | ✅ API変更なし（内部修正のみ） | Low |

### 主要なギャップ

#### 1. encoding/mod.rsのto_ansi_bytes追加（Req 2）

**現在のバグ（path_to_lua）**:
```rust
pub fn path_to_lua(path: &str) -> Result<String> {
    let bytes = Encoding::ANSI.to_bytes(path)?;  // UTF-8 → ANSIバイト列 (OK)
    Ok(String::from_utf8_lossy(&bytes).into_owned())  // ← バグ！ANSIバイト列が破壊される
}
```

**新規追加方法**:
```rust
pub fn to_ansi_bytes(s: &str) -> Result<Vec<u8>> {
    #[cfg(windows)]
    {
        Encoding::ANSI.to_bytes(s)
    }
    #[cfg(not(windows))]
    {
        Ok(s.as_bytes().to_vec())
    }
}
```

**複雑度**: Low（単純な関数追加、既存path_to_luaはテスト専用として維持）

#### 2. LoaderContextのバイト列生成（Req 3）

**状況**:
- `@pasta_search`の実装パターンが完全に再利用可能
- `encoding::Encoding::ANSI.to_bytes()` / `to_string()`が既存
- UserDataではなく、Luaテーブルでの公開（関数のみ）

**実装要素**:
```rust
// 新規ファイル: src/runtime/enc.rs (または src/enc/mod.rs)
pub fn register(lua: &Lua) -> LuaResult<Table> {
    let module = lua.create_table()?;
    module.set("_VERSION", "0.1.0")?;
    module.set("_DESCRIPTION", "Encoding conversion (UTF-8 <-> ANSI)")?;
    
    let to_ansi = lua.create_function(|lua, s: String| {
        use crate::encoding::{Encoder, Encoding};
        match Encoding::ANSI.to_bytes(&s) {
            Ok(bytes) => {
                // バイト列を直接Lua文字列化（ANSIエンコードされたバイト列を保持）
                let lua_str = lua.create_string(&bytes)?;
                Ok((Some(lua_str), None::<String>))
            }
            Err(e) => Ok((None::<LuaString>, Some(e.to_string()))),
        }
    })?;
    module.set("to_ansi", to_ansi)?;
    
    let to_utf8 = lua.create_function(|lua, s: LuaString| {
        use crate::encoding::{Encoder, Encoding};
        let bytes = s.as_bytes();
        match Encoding::ANSI.to_string(bytes) {
            Ok(utf8_string) => {
                let lua_str = lua.create_string(utf8_string.as_bytes())?;
                Ok((Some(lua_str), None::<String>))
            }
            Err(e) => Ok((None::<LuaString>, Some(e.to_string()))),
        }
    })?;
    module.set("to_utf8", to_utf8)?;
    
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set("@enc", module.clone())?;
    Ok(module)
}
```

**複雑度**: Low（`@pasta_search`パターン確立済み、80行程度）

### 非機能要件
package.path設定のみの修正、セキュリティ影響なし
- **パフォーマンス**: エンコーディング変換は起動時1回のみ、影響軽微
- **スケーラビリティ**: 変更範囲が限定的、スケーラビリティ問題なし

---

## 3. Implementation Approach Options

### Option A: 最小限修正（推奨）

**概要**: 
- `encoding/mod.rs`に`to_ansi_bytes`追加
- `loader/context.rs`に`generate_package_path_bytes`追加
- `runtime/mod.rs`の`setup_package_path`を修正
- `runtime/enc.rs`を新規作成（@encモジュール）

**ファイル変更**:
- 修正: `src/encoding/mod.rs` (+15行)
- 修正: `src/loader/context.rs` (+20行)
- 修正: `src/runtime/mod.rs` (+5行修正、+2行呼び出し追加)
- 新規: `src/runtime/enc.rs` (+80行)
- 修正: `src/runtime/mod.rs` (+1行: `mod enc;`, +1行: `enc::register`呼び出し)

**互換性**:
- ✅ 既存API変更なし（内部実装のみ修正）
- ✅ テスト影響なし（既存テストは変更不要）
- ✅ 既存の`path_to_lua`も維持（後方互換性）

**複雑度・保守性**:
- ✅ 変更範囲最小（120行追加）
- ✅ 既存コード構造維持
- ✅ 責務が明確（エンコーディング、ローダー、ランタイム、@enc独立）

**Trade-offs**:
- ✅ 最速実装（1-2日）
- ✅ リスク最小
- ✅ テスト容易

---

### Option B: リファクタリング込み（非推奨）

**概要**:
- Option Aに加えて、encoding/mod.rsを全面的にリファクタリング
- `path_to_lua`を削除し、`path_to_lua_bytes`に一本化

**Trade-offs**:
- ❌ 追加工数（+1日）
- ❌ 既存コード影響（`path_to_lua`使用箇所の確認必要）
- ✅ コード整理

**結論**: 現時点で不要（`path_to_lua`は未使用なら後で削除可能）

---

## 4. 実装複雑度とリスク評価

### 全体評価

| 評価軸 | 判定 | 根拠 |
|--------|------|------|
| **Effort** | **S (1-2日)** | 変更範囲明確、既存パターン流用、新規コード120行程度 |
| **Risk** | **Low** | カスタムサーチャー不要、mluaの既知API使用、クロスプラットフォーム対応済み |

### リスク詳細

**Low Risk要因**:
- ✅ カスタムサーチャー不要（最大リスク要因が消滅）
- ✅ エンコーディング変換は実装済み（`encoding/`再利用）
- ✅ バイト列→Lua文字列は標準API（`Lua::create_string`）
- ✅ `@pasta_search`の実装パターンが完全に適用可能
- ✅ 既存API変更なし（後方互換性確保）

**注意点**:
- Windows環境でのテスト必須（日本語パス）
- CI設定でWindows/Linux両方のテストを実施

### 段階的実装戦略

**フェーズ1**: encoding修正（0.5日）
- `path_to_lua_bytes`実装
- ユニットテスト追加

**フェーズ2**: LoaderContext対応（0.5日）
- `generate_package_path_bytes`実装
- 統合テスト

**フェーズ3**: Runtime修正（0.5日）
- `setup_package_path`修正
- Windows環境での日本語パステスト

**フェーズ4**: @encモジュール（0.5-1日）
- `runtime/enc.rs`実装
- @encモジュールテスト

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A: 最小限修正**

理由:
1. **リスク最小化**: カスタムサーチャー不要により技術的不確実性が消滅
2. **工数削減**: M(4-5日) → S(1-2日)に短縮
3. **既存パターン準拠**: `@pasta_search`と同じモジュール構成で@enc実装

実装スケジュール: 1-2日（S）

---

### 設計フェーズで決定すべき項目

#### 1. encoding/mod.rsの関数名

**選択肢**:
- `path_to_lua_bytes` - 新規関数、既存`path_to_lua`維持
- `path_to_lua`を修正してバイト列返却に変更（破壊的変更）

**推奨**: `path_to_lua_bytes`新規追加（後方互換性維持）

#### 2. @encモジュールの関数セット

**必須**:
- `to_ansi(utf8_string) -> (ansi_string | nil, error)`
- `to_utf8(ansi_bytes) -> (utf8_string | nil, error)`

**オプション**:
- `to_oem(utf8_string)` - コンソール出力用
- `from_oem(oem_bytes)` - コンソール入力用

**推奨**: 必須のみ実装（OEMは将来追加可能）

#### 3. テスト戦略

**必須テスト**:
1. ASCII パスでのモジュールロード（基本動作、Linux/Windows）
2. 日本語パスでのモジュールロード（Windows専用、CI注意）
3. @encモジュールの`to_ansi` / `to_utf8`基本動作
4. @encモジュールのエラーハンドリング（型エラー、変換失敗）
5. 既存テストの回帰確認

**CI考慮**:
- Windows環境でのみ日本語パステスト実行
- Unix環境ではパススルー動作確認

---

### 外部依存関係

**追加依存なし**:
- 既存の`mlua`, `windows-sys`, `glob`, `toml`で実装可能

---

## 6. 結論

### 実装可能性: **非常に高**

- カスタムサーチャー不要（当初懸念していた技術リスクが消滅）
- エンコーディング変換は実装済み
- 変更範囲が明確（3ファイル修正 + 1ファイル新規）
- 既存パターン完全流用可能

### 推奨実装戦略

**Option A: 最小限修正**
- `encoding/mod.rs` - `path_to_lua_bytes`追加
- `loader/context.rs` - `generate_package_path_bytes`追加
- `runtime/mod.rs` - `setup_package_path`修正
- `runtime/enc.rs` - @encモジュール新規作成
- Effort: **S (1-2日)**、Risk: **Low**

### 次のステップ

1. `/kiro-spec-design pasta-lua-windows-path-encoding`で技術設計フェーズへ進む
2. 設計フェーズで以下を実施:
   - 詳細なモジュールAPI設計
   - テストケース詳細化（Windows日本語パステスト含む）
   - 実装タスク分解

---

**分析完了日**: 2026-01-15  
**分析更新日**: 2026-01-15（カスタムサーチャー不要を確認）
**分析完了日**: 2026-01-15
