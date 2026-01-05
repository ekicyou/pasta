# 実装ギャップ分析: pasta-lua-output-normalization

**作成日**: 2026-01-05  
**対象仕様**: pasta-lua-output-normalization  
**分析者**: GitHub Copilot

---

## エグゼクティブサマリー

- **スコープ**: pasta_lua トランスパイラーの出力フォーマッティング正規化（末尾空行の除去、シーン定義フォーマット統一）
- **現状**: コード生成ロジックが各要素の末尾に無条件で空行を追加、ファイル末尾で余分な空行が発生
- **主要課題**:
  1. `generate_local_scene()` が各ローカルシーン後に `write_blank_line()` を呼び出し
  2. `generate_global_scene()` がシーン終了時（`end` 後）に `write_blank_line()` を呼び出し
  3. 最後のシーン定義後、ファイル末尾に余分な改行が蓄積
- **推奨アプローチ**: **Option A: 既存コンポーネント拡張**（出力正規化パスを追加）
- **見積もり**: **S (1-3日)** - 既存パターンに沿った単純な修正
- **リスク**: **Low** - 局所的な変更、明確な期待値、既存テスト基盤完備

---

## 1. 現状分析

### 1.1 問題の特定

**差分分析結果**:
```diff
@@ -111,4 +111,6 @@ do
         act.さくら:talk("グローバルの分岐に飛んできた。")
         act.うにゅう:talk("世界取れるで。")
     end
+
 end
+
```

**具体的な差異**:
| 項目                | 期待値    | 生成値    | 差異  |
| ------------------- | --------- | --------- | ----- |
| 総行数              | 114 行    | 116 行    | +2 行 |
| シーン閉じ `end` 前 | 空行なし  | 空行 1 行 | +1 行 |
| ファイル末尾        | 空行 1 行 | 空行 2 行 | +1 行 |

### 1.2 既存実装の確認

#### コード生成フロー

```
LuaTranspiler::transpile()
  └── LuaCodeGenerator::generate_actor()
        └── write_blank_line()  ← アクター定義後に空行追加
  └── LuaCodeGenerator::generate_global_scene()
        ├── generate_local_scene()
        │     └── write_blank_line()  ← 各ローカルシーン後に空行追加（問題箇所1）
        ├── generate_code_block()
        └── write_blank_line()  ← シーン全体後に空行追加（問題箇所2）
```

#### 問題箇所 1: `generate_local_scene()` (L276-277)

```rust
// code_generator.rs:276-277
self.dedent();
self.writeln("end")?;
self.write_blank_line()?;  // ← 毎回空行追加
```

**影響**: 最後のローカルシーン関数定義後に空行が追加され、グローバルシーンの `end` の前に空行が挿入される。

#### 問題箇所 2: `generate_global_scene()` (L214-217)

```rust
// code_generator.rs:214-217
self.dedent();
self.writeln("end")?;
self.write_blank_line()?;  // ← シーン終了後に空行追加
```

**影響**: 最後のグローバルシーン定義後、ファイル末尾に余分な空行が追加される。

### 1.3 期待される出力フォーマット

**sample.expected.lua の構造**:
```lua
local PASTA = require "pasta"
                                    -- 空行
do                                  -- アクター開始
    ...
end                                 -- アクター終了
                                    -- 空行
do                                  -- アクター開始
    ...
end                                 -- アクター終了
                                    -- 空行
do                                  -- シーン開始
    local SCENE = PASTA.create_scene("...")
                                    -- 空行
    function SCENE.__start__(...)
        ...
    end                             -- ローカルシーン終了
                                    -- 空行（次のローカルシーンがある場合）
    function SCENE.__XXX_1__(...)
        ...
    end                             -- 最後のローカルシーン終了
                                    -- ※ここに空行を入れてはいけない
end                                 -- シーン終了（空行なし）
                                    -- 空行（次のシーンがある場合）
do                                  -- 次のシーン
    ...
end                                 -- 最後のシーン終了
                                    -- 1つの改行でファイル終了（EOF）
```

---

## 2. 要件との対応分析

### 要件マッピング

| 要件                      | 現状                       | ギャップ       | 対応策                                     |
| ------------------------- | -------------------------- | -------------- | ------------------------------------------ |
| 1. 末尾空行の除去         | 最後のシーン後に余分な空行 | **Missing**    | 最後の要素判定を追加、または後処理で正規化 |
| 2. シーン定義の正規化     | `end` 前に空行が入る       | **Missing**    | 最後のローカルシーン後の空行を抑制         |
| 3. 出力バッファ正規化     | 正規化パスなし             | **Missing**    | `transpile()` 終了時に正規化処理を追加     |
| 4. テストフィクスチャ一致 | 114 vs 116 行              | **Constraint** | 上記修正で解決                             |
| 5. リグレッション防止     | 既存テストあり             | **Constraint** | 変更後にテストスイート実行                 |

---

## 3. 実装アプローチの評価

### Option A: 既存コンポーネント拡張（**推奨**）

**方針**: `transpile()` メソッドの終了時に出力バッファの正規化パスを追加

**変更ファイル**:
- `crates/pasta_lua/src/transpiler.rs` - 正規化処理の追加

**実装イメージ**:
```rust
pub fn transpile<W: Write>(
    &self,
    file: &PastaFile,
    writer: &mut W,
) -> Result<TranspileContext, TranspileError> {
    // 内部バッファに書き込み
    let mut buffer = Vec::new();
    // ... 既存処理 ...

    // 正規化パス
    let normalized = Self::normalize_output(&buffer);
    writer.write_all(&normalized)?;

    Ok(context)
}

fn normalize_output(buffer: &[u8]) -> Vec<u8> {
    let s = String::from_utf8_lossy(buffer);
    let normalized = s
        .trim_end()
        .to_string() + "\n";  // 末尾に1つの改行のみ
    normalized.into_bytes()
}
```

**Trade-offs**:
- ✅ 最小限の変更で対応可能
- ✅ 既存のコード生成ロジックに影響なし
- ✅ 将来の類似問題にも対応可能
- ❌ 内部バッファ経由になるため、若干のメモリオーバーヘッド

### Option B: コード生成ロジックの修正

**方針**: 各 `write_blank_line()` 呼び出しを条件付きに変更

**変更ファイル**:
- `crates/pasta_lua/src/code_generator.rs` - 複数箇所の修正

**実装イメージ**:
```rust
// generate_local_scene に「最後かどうか」のフラグを追加
pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,
    counter: usize,
    actors: &[SceneActorItem],
    is_last: bool,  // 新規追加
) -> Result<(), TranspileError> {
    // ...
    self.dedent();
    self.writeln("end")?;
    if !is_last {
        self.write_blank_line()?;
    }
    Ok(())
}
```

**Trade-offs**:
- ✅ 問題の根本原因を直接修正
- ✅ 出力が最初から正しいフォーマット
- ❌ 複数箇所の変更が必要
- ❌ 既存の関数シグネチャ変更が必要
- ❌ テスト修正も必要になる可能性

### Option C: ハイブリッドアプローチ

**方針**: Option B の部分修正 + Option A の正規化パス

**変更ファイル**:
- `crates/pasta_lua/src/code_generator.rs` - シーン終了時の空行制御
- `crates/pasta_lua/src/transpiler.rs` - 末尾正規化パス

**Trade-offs**:
- ✅ バランスの取れたアプローチ
- ✅ 各レイヤーで適切な責務分担
- ❌ 2つのファイルを変更する必要あり

---

## 4. 推奨事項

### 推奨アプローチ: **Option A（既存コンポーネント拡張）**

**理由**:
1. **最小リスク**: 既存のコード生成ロジックに変更を加えない
2. **シンプル**: 1ファイル1箇所の変更で済む
3. **汎用性**: 将来の類似問題にも対応可能
4. **テスト容易性**: 正規化ロジック単体でテスト可能

### 実装見積もり

| 項目       | 見積もり      |
| ---------- | ------------- |
| **工数**   | **S (1-3日)** |
| **リスク** | **Low**       |

**工数根拠**:
- 変更箇所が限定的（1-2ファイル）
- 明確な期待値が存在（sample.expected.lua）
- 既存テストで検証可能

**リスク根拠**:
- 既存パターンに沿った変更
- ローカライズされた影響範囲
- 既存テスト基盤で品質保証可能

### 設計フェーズへの持ち越し事項

1. **正規化ロジックの詳細設計**
   - 連続空行の処理ルール（2連続→1に縮小？）
   - 末尾処理（trim_end + 1 改行？）

2. **バッファリング戦略**
   - 内部バッファ vs ストリーミング
   - メモリ効率の考慮

3. **テスト戦略**
   - 正規化ロジックのユニットテスト追加
   - 既存統合テストの活用

---

## 5. 検証項目チェックリスト

### 実装後の確認事項

- [ ] `test_transpile_sample_pasta_line_comparison` が合格
- [ ] 生成行数が期待値と一致（114 行）
- [ ] 既存テスト全合格（`cargo test --all`）
- [ ] `sample.generated.lua` == `sample.expected.lua`（完全一致）

### リグレッションチェック対象

- [ ] `test_transpile_sample_pasta_header`
- [ ] `test_transpile_sample_pasta_actors`
- [ ] その他の transpiler_integration_test.rs 内テスト

---

## 6. 結論

pasta_lua トランスパイラーの出力フォーマッティング問題は、**コード生成ロジックの末尾空行処理**に起因しています。

**Option A（出力正規化パスの追加）** が最もリスクが低く、実装も容易なアプローチです。transpile() メソッドの終了時に正規化処理を追加することで、既存のコード生成ロジックに変更を加えることなく、期待される出力フォーマットを実現できます。

設計フェーズでは、正規化ロジックの詳細設計とバッファリング戦略を確定し、タスクフェーズで実装を進めることを推奨します。
