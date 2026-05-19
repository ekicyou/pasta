# Research & Design Decisions

---

## Gap Analysis (validate-gap)

### Summary
- **対象**: luajit-migration（Lua 5.5 → LuaJIT 2.1）
- **分析アプローチ**: Option A（既存コンポーネントの拡張）
- **Effort**: S（1-3日） — Cargo.toml設定変更 + StdLibマッピング更新 + Luaスクリプト1ファイル修正
- **Risk**: Low — コードベース調査で非互換箇所が限定的と確認済み

### Requirement-to-Asset Map

| Requirement            | 既存Asset              | Gap                                    | Status     |
| ---------------------- | ---------------------- | -------------------------------------- | ---------- |
| 1 (ランタイム切り替え) | Cargo.toml mlua依存    | feature flag変更のみ                   | Constraint |
| 2 (lua-src除去)        | Cargo.toml lua-src依存 | 削除のみ                               | Constraint |
| 3 (UTF-8識別子)        | ucid_test.rs           | LuaJIT 2.1ネイティブサポート、変更不要 | —          |
| 4 (テスト互換性)       | 950+ tests             | テスト全パス確認                       | —          |
| 5 (下流クレート)       | pasta_shiori等         | 直接mlua依存なし、pasta_lua経由のみ    | —          |
| 6 (mlua-stdlib)        | json/regex/yaml        | mlua抽象層経由、バックエンド非依存     | —          |
| 7 (ステアリング)       | tech.md                | 記載更新のみ                           | —          |

### Gap Details

#### GAP-1: `StdLib::UTF8` コンパイルエラー（既知）
- **箇所**: `crates/pasta_lua/src/runtime/runtime_config.rs:159`
- **内容**: `"std_utf8" => Ok(StdLib::UTF8)` — LuaJITでは `StdLib::UTF8` が未定義
- **影響**: コンパイル不可
- **対処**: マッピング削除 + LuaJIT固有マッピング追加（`std_jit`, `std_ffi`, `std_bit`）
- **Status**: design.mdで対処済み

#### GAP-2: `table.move` 非互換（★新規発見）
- **箇所**: `crates/pasta_lua/scriptlibs/lua_test/toDebugString.lua:89-90`
- **内容**: `table.move(arrayPart, 1, #arrayPart, #items + 1, items)` × 2箇所
- **影響**: lua_testフレームワークのテスト失敗（`toDebugString` はテスト出力整形で使用）
- **対処**: Lua 5.1互換のforループに置換:
  ```lua
  -- Before: table.move(arrayPart, 1, #arrayPart, #items + 1, items)
  -- After:
  for i = 1, #arrayPart do
      items[#items + 1] = arrayPart[i]
  end
  ```
- **Status**: ★ design.md / tasks.md に未反映 — 要追加

### False Alarm 検証

#### `Value::Integer` — 問題なし
- **調査箇所**: `log.rs:98`, `module_registry.rs:170`, `search/context.rs:221,242`
- **調査結果**: mlua 0.11 docs.rsで確認 — `Value::Integer` は全Luaバックエンドで常に利用可能（条件付きコンパイルではない）
- **メソッド**: `as_integer()` も全バックエンドで利用可能
- **結論**: false alarm、修正不要

#### luacheck `//` / ビット演算子 — 問題なし
- **調査箇所**: `scriptlibs/luacheck/parser.lua:565`, `lexer.lua:587`
- **調査結果**: `["//"] = "idiv"`, `["&"] = "band"` 等 — テーブルキー文字列であり、実際のLua演算子ではない
- **結論**: LuaJIT互換、修正不要

#### luacheck `table.move` — 問題なし
- `scriptlibs/luacheck/` 内には `table.move` の使用なし

### Implementation Options

#### Option A: 既存コンポーネント拡張（推奨）
- Cargo.toml feature変更
- `runtime_config.rs` StdLibマッピング更新
- `toDebugString.lua` の `table.move` → forループ置換
- **Pros**: 最小変更、影響範囲限定、ロールバック容易
- **Cons**: なし（変更が小さすぎてデメリットがない）

#### Option B: 新コンポーネント作成 — 不適用
- この移行には新コンポーネント作成は不要

#### Option C: ハイブリッドアプローチ — 不適用
- Feature flagsによる並行サポートは過剰（discoveryで除外済み）

### Recommendations for Design Phase
1. **GAP-2（table.move）を design.md と tasks.md に追加する必要あり**
   - `toDebugString.lua` はテストフレームワーク（lua_test）の一部であり、テスト実行に影響する
   - Task 1.1 のスコープに「`toDebugString.lua` の `table.move` 修正」を追加すべき
   - design.mdのFile Structure Planに `scriptlibs/lua_test/toDebugString.lua` を追加すべき
2. その他の既知ギャップ（StdLib::UTF8）は design.md で既に対処済み

---

## Summary
- **Feature**: `luajit-migration`
- **Discovery Scope**: Extension（既存システムのランタイム切り替え）
- **Key Findings**:
  - `StdLib::UTF8` が `runtime_config.rs` で使用されており、LuaJITでは存在しないため条件付きコンパイルが必要
  - LuaJIT固有の `StdLib::JIT`, `StdLib::FFI`, `StdLib::BIT` フラグの追加が必要
  - `default_libs()` は `"std_all"` を使用しており、`StdLib::ALL_SAFE` 経由のため影響なし

## Research Log

### mlua StdLib フラグのLuaJIT互換性
- **Context**: `runtime_config.rs` の `parse_std_lib()` が `StdLib::UTF8` を明示的にマッピングしている
- **Sources**: mlua 0.11 ソースコード、mlua documentation
- **Findings**:
  - mlua の `StdLib` は Lua バージョン feature に応じて条件付きで定義される
  - LuaJIT（`luajit`/`luajit52` feature）では `StdLib::UTF8` が定義されない
  - 代わりに `StdLib::JIT`, `StdLib::FFI`, `StdLib::BIT` が追加される
  - `StdLib::ALL_SAFE` はバージョンに応じて適切に定義されるため、`"std_all"` マッピングは影響なし
- **Implications**: `parse_std_lib()` の `"std_utf8"` マッピングを削除し、LuaJIT固有のマッピングを追加する必要あり

### LuaJIT 2.1 UTF-8 識別子サポート
- **Context**: 現在 `lua-src` の `ucid` feature でUTF-8識別子を有効化している
- **Sources**: LuaJIT 2.1 Extensions documentation
- **Findings**:
  - LuaJIT 2.1は UTF-8識別子をネイティブサポート（パッチ不要）
  - `ucid_test.rs` の3テスト（日本語変数名、関数名、テーブルフィールド）はそのまま動作するはず
  - `lua-src` 依存を除去しても `Lua::new()` で自動的にUTF-8識別子が利用可能
- **Implications**: `lua-src` 依存の完全除去が可能、テストコードの変更は不要

### mlua-stdlib LuaJIT互換性
- **Context**: `mlua-stdlib 0.1` が json, regex, yaml を提供
- **Findings**:
  - mlua-stdlib はRust側の mlua 抽象層を通じて動作するため、Luaバックエンドに非依存
  - LuaJITバックエンドでも問題なく動作する見込み
- **Implications**: mlua-stdlib の設定変更は不要

## Design Decisions

### Decision: StdLib::UTF8 の処理方針
- **Context**: `"std_utf8"` マッピングがコンパイルエラーを引き起こす
- **Alternatives Considered**:
  1. `#[cfg]` 属性で条件付きコンパイル — 両ランタイムサポート時に有効だが、現在は不要
  2. マッピングを完全削除し、LuaJIT固有マッピングに置換 — シンプルで明確
- **Selected Approach**: マッピングの完全置換（`std_utf8` → 削除、`std_jit`/`std_ffi`/`std_bit` → 追加）
- **Rationale**: ダイレクト切り替えアプローチのため、Lua 5.5サポートを残す理由がない
- **Trade-offs**: Lua 5.5に戻す場合は逆の変更が必要（git revertで対応可能）

### Decision: Lua::unsafe_new_with のオプション
- **Context**: ランタイム初期化が `Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default())` を使用
- **Selected Approach**: 変更不要 — `LuaOptions::default()` はLuaJITでも有効
- **Rationale**: mlua の抽象化により、バックエンド非依存のAPIが維持されている

## Risks & Mitigations
- `StdLib::UTF8` コンパイルエラー — マッピング置換で対応
- LuaJIT固有のメモリ制限（2GB）— 現在のユースケースでは問題にならない
- LuaJIT非サポートの `string.pack/unpack` — 現在のコードベースでは未使用

## References
- [mlua 0.11 documentation](https://docs.rs/mlua/0.11) — StdLib flags per Lua version
- [LuaJIT 2.1 Extensions](https://luajit.org/extensions.html) — UTF-8 identifier support
