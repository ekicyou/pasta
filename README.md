# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

## アーキテクチャ

Pastaは、areka デスクトップマスコットアプリケーション向けのスクリプトエンジンです。

### レイヤー構成

```
Engine (上位API) → Cache/Loader
    ↓
Transpiler (2pass) ← Parser (Pest)
    ↓
Runtime (Rune VM) → IR Output (ScriptEvent)
```

### パーサー/トランスパイラーアーキテクチャ

Pastaは `parser2` + `transpiler2` スタックを使用しています：

| モジュール | 文法ファイル | 状態 | 用途 |
|------------|--------------|------|------|
| `pasta::parser2` | `grammar.pest` | **現行** | engine.rsで使用 |
| `pasta::transpiler2` | - | **現行** | 2-pass トランスパイル |
| `pasta::parser` | `pasta.pest` | レガシー（非推奨） | 後方互換性のため維持 |
| `pasta::transpiler` | - | レガシー（非推奨） | 後方互換性のため維持 |

#### 使用方法

```rust
// 現行スタック（推奨）
use pasta::parser2::{parse_str, parse_file};
use pasta::transpiler2::Transpiler2;

// レガシースタック（非推奨）
use pasta::parser::{parse_str, parse_file};
use pasta::transpiler::Transpiler;
```

#### 移行履歴

1. **Phase 1**: ✅ parser2モジュールを作成、parser と並存
2. **Phase 2**: ✅ transpiler2を作成、parser2と連携
3. **Phase 3**: ✅ engine.rs を parser2/transpiler2 に完全移行
4. **Phase 4**: （保留）レガシースタックの削除は後方互換性確認後に実施

### parser2について

`parser2`モジュールは、検証済みの`pasta2.pest`文法（`grammar.pest`として配置）に基づいています。

主な特徴：
- 3層スコープ構造：`FileScope` ⊃ `GlobalSceneScope` ⊃ `LocalSceneScope`
- 未名グローバルシーンの自動名前継承
- 全角/半角数字の自動正規化
- 継続行アクション（`：`で始まる行）のサポート
