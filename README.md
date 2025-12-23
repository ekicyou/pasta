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

### 並行パーサーアーキテクチャ

Pastaは2つのパーサーモジュールを持ちます：

| モジュール | 文法ファイル | 状態 | 用途 |
|------------|--------------|------|------|
| `pasta::parser` | `pasta.pest` | 現行（レガシー） | 既存のtranspiler/runtimeで使用 |
| `pasta::parser2` | `grammar.pest` | 新規開発中 | pasta2.pestに基づく新実装 |

#### 使い分け

```rust
// レガシーパーサー（現在のメイン）
use pasta::parser::{parse_str, parse_file};

// 新パーサー（開発中）
use pasta::parser2::{parse_str, parse_file};
```

#### 移行計画

1. **Phase 1（現在）**: parser2モジュールを作成、parser と並存
2. **Phase 2**: transpiler2を作成、parser2と連携
3. **Phase 3**: 検証完了後、parser → parser2への完全移行
4. **Phase 4**: レガシーparserを削除

### parser2について

`parser2`モジュールは、検証済みの`pasta2.pest`文法（`grammar.pest`として配置）に基づいています。

主な特徴：
- 3層スコープ構造：`FileScope` ⊃ `GlobalSceneScope` ⊃ `LocalSceneScope`
- 未名グローバルシーンの自動名前継承
- 全角/半角数字の自動正規化
- 継続行アクション（`：`で始まる行）のサポート
