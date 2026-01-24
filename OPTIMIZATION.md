# Transpiler Optimization Reference

このドキュメントは、pasta_luaトランスパイラが適用する最適化について説明します。

**最終更新**: 2026-01-25  
**ステータス**: Phase 0完了

---

## 1. 最適化の概要

pastaトランスパイラは、Lua出力コードに対して以下の最適化を適用します：

| 最適化 | レベル | 説明 | 実装状態 |
|--------|--------|------|----------|
| 末尾呼び出し最適化 (TCO) | コード生成 | 末尾Call文に`return`を付加 | ✅ 実装済み |
| アクター最適化 | コード生成 | 連続発言のアクター切替最小化 | ✅ 実装済み |
| 文字列リテラル最適化 | コード生成 | ロングブラケット記法 | ✅ 実装済み |

---

## 2. 末尾呼び出し最適化 (Tail Call Optimization)

### 2.1 概要

Luaは**末尾呼び出し最適化 (TCO)** をサポートしています。これにより、関数の最後の文が別の関数呼び出しである場合、スタックフレームを再利用して無限再帰を可能にします。

### 2.2 適用条件

トランスパイラは以下の条件で自動的にTCOを適用します：

1. **末尾位置判定**: LocalSceneItemsの最後の要素が`CallScene`である
2. **return付加**: 末尾Call文に`return`を前置

### 2.3 コード例

**Pasta入力**:
```pasta
＊メイン
  さくら：「こんにちは」
  ＞サブ

  ・サブ
    うにゅう：「やあ」
```

**Lua出力**:
```lua
function SCENE.__start__(act, ...)
    local args = { ... }
    local save, var = act:init_scene(SCENE)

    act.さくら:talk("「こんにちは」")
    return act:call(SCENE.__global_name__, "サブ", {}, table.unpack(args))  -- TCO適用
end
```

### 2.4 TCO非適用ケース

Call文の後にアクション行がある場合、TCOは適用されません：

```pasta
＊メイン
  ＞サブ
  さくら：「戻ってきました」
```

出力:
```lua
act:call(SCENE.__global_name__, "サブ", {}, table.unpack(args))  -- return なし
act.さくら:talk("「戻ってきました」")
```

---

## 3. アクター最適化

### 3.1 概要

連続するアクション行で同じアクターが発言する場合、コンテキストを保持して効率化します。

### 3.2 実装

`last_actor` 変数でアクター状態を追跡し、必要な場合のみアクターを切り替えます。

---

## 4. 文字列リテラル最適化

### 4.1 概要

特殊文字を含む文字列に対して、Luaのロングブラケット記法 `[=[ ... ]=]` を使用します。

### 4.2 適用条件

- 文字列に `"`, `\`, 改行が含まれる場合
- Unicode文字を含む場合

### 4.3 例

```lua
ACTOR:create_word("通常"):entry([=[\s[0]]=])  -- さくらスクリプトのエスケープ保持
```

---

## 5. 今後の最適化候補

以下の最適化は将来的な実装候補です：

| 最適化 | 優先度 | 説明 |
|--------|--------|------|
| 定数畳み込み | Medium | コンパイル時に定数式を評価 |
| デッドコード削除 | Low | 到達不能コードの除去 |
| インライン展開 | Low | 小さなローカルシーンのインライン化 |
| 単語プリフェッチ | Medium | 単語辞書の先読み最適化 |

---

## 6. ビルドプロファイル最適化

`Cargo.toml` で設定されているリリースビルド最適化：

```toml
[profile.release]
opt-level = "z"       # サイズ最適化
lto = true            # リンク時最適化
codegen-units = 1     # 単一コード生成ユニット
panic = "abort"       # アンワインド無効化
strip = true          # シンボル削除
```

これにより、SHIORI.DLLのサイズを最小化しています。

---

## 7. 関連ドキュメント

- [SPECIFICATION.md](SPECIFICATION.md) - 言語仕様
- [crates/pasta_lua/src/code_generator.rs](crates/pasta_lua/src/code_generator.rs) - コード生成実装
- [tests/fixtures/tail_call_optimization.pasta](tests/fixtures/tail_call_optimization.pasta) - TCOテストケース

---

**参照実装**: [code_generator.rs#L320-L460](crates/pasta_lua/src/code_generator.rs)
