# ギャップ分析: scene-actors-ast-support

## 分析サマリー

本分析は、`scene_actors_line`文法に対応するAST実装のギャップを調査したものである。

### 主要な発見

1. **既存パターンの確立**: `GlobalSceneScope`への新規フィールド追加と`parse_global_scene_scope`関数拡張は、既存の`attrs`/`words`フィールドと同様のパターンで実装可能
2. **影響範囲の限定**: pasta_coreの`ast.rs`と`mod.rs`のみが主要な変更対象で、pasta_luaは最小限の対応（フィールド無視）で済む
3. **テスト戦略の明確化**: 既存の`mod.rs`内テストパターンに従い、パーステストを追加可能
4. **低リスク実装**: 既存アーキテクチャに完全に適合し、破壊的変更なし

---

## 1. 現状調査

### 1.0 前提条件（憲法）

> **grammar.pestは検証済みの憲法として扱う**
> - 文法定義は完成・検証済みであり、本仕様では変更対象外
> - 「　％さくら、うにゅう＝２」のパース動作は確認済み
> - `scene_actors_line`は`global_scene_init`の文脈でのみ登場可能

### 1.1 関連ファイル構成

| ファイル | 役割 | 変更要否 |
|---------|------|---------|
| `crates/pasta_core/src/parser/grammar.pest` | Pest文法定義 | ❌ 変更不要（憲法） |
| `crates/pasta_core/src/parser/ast.rs` | AST型定義 | **要変更** |
| `crates/pasta_core/src/parser/mod.rs` | パース変換ロジック | **要変更** |
| `crates/pasta_lua/src/transpiler.rs` | Luaトランスパイラ | 最小変更 |
| `crates/pasta_lua/src/code_generator.rs` | Luaコード生成 | 最小変更 |
| `crates/pasta_lua/src/context.rs` | コンテキスト管理 | 最小変更 |

### 1.2 既存パターン分析

#### grammar.pestの構造
```pest
scene_actors_line    = { pad ~ actor_marker ~ actors ~ or_comment_eol }
actors       = _{ actors_item ~ ( comma_sep ~ actors_item )* ~ comma_sep? }
actors_item  =  { id ~ ( s ~ set_marker ~ s ~ digit_id )? }
```

`global_scene_init`に`scene_actors_line`が組み込まれている：
```pest
global_scene_init  =_{ global_scene_attr_line | global_scene_word_line | scene_actors_line | blank_line }
```

#### ast.rsの既存パターン
`GlobalSceneScope`は現在以下のフィールドを持つ：
- `name: String`
- `is_continuation: bool`
- `attrs: Vec<Attr>` ← 類似パターン
- `words: Vec<KeyWords>` ← 類似パターン
- `code_blocks: Vec<CodeBlock>`
- `local_scenes: Vec<LocalSceneScope>`
- `span: Span`

**パターン**: `attrs`と`words`はそれぞれ`global_scene_attr_line`と`global_scene_word_line`から収集される。`actors`も同様のパターンで追加可能。

#### mod.rsのパース変換パターン
`parse_global_scene_scope`関数内のmatch分岐：
```rust
Rule::global_scene_attr_line => {
    for attr_pair in inner.into_inner() {
        if attr_pair.as_rule() == Rule::attr {
            attrs.push(parse_attr(attr_pair)?);
        }
    }
}
Rule::global_scene_word_line => {
    for kw_pair in inner.into_inner() {
        if kw_pair.as_rule() == Rule::key_words {
            words.push(parse_key_words(kw_pair)?);
        }
    }
}
```

**パターン**: `scene_actors_line`も同様のmatch分岐を追加すればよい。

### 1.3 既存の関連型

| 型 | 用途 | 再利用可否 |
|---|------|----------|
| `ActorScope` | アクター定義スコープ（`actor_scope`） | 名前のみ類似、構造は異なる |
| `Span` | ソース位置情報 | ✅ 再利用可 |
| `KeyWords` | 単語定義 | 参考パターン |

**注意**: `ActorScope`は`actor_scope`文法用であり、`scene_actors_line`とは別概念。混同しないこと。

---

## 2. 要件実現可能性分析

### 2.1 要件1: SceneActorItem AST型定義

| 技術要件 | 既存資産 | ギャップ |
|---------|---------|---------|
| `SceneActorItem`型 | なし | **Missing** - 新規作成必要 |
| アクター名保持 | `String`型使用パターン確立 | なし |
| 番号保持 | `u32`型 | なし |
| Span保持 | `Span`型再利用可 | なし |

**設計決定**:
- **議題1**: `SceneActorLine`型は定義しない（中間型不要）
- **議題2**: 番号はC#のenum採番ルールで計算し、`u32`として保持
  - 複数行にまたがる場合でも、最後の番号を引き継いで+1
  - 実装: `parse_global_scene_scope`で`next_number: u32`を管理し、`parse_scene_actors_line`に渡す

### 2.2 要件2: GlobalSceneScopeへの統合

| 技術要件 | 既存資産 | ギャップ |
|---------|---------|---------|
| `actors`フィールド追加 | `Vec<T>`パターン確立 | **Missing** - フィールド追加必要 |
| コンストラクタ更新 | `new()`/`continuation()`存在 | 要更新 |
| 蓄積ロジック | `attrs`/`words`と同様 | 実装必要 |

### 2.3 要件3: パーサー変換実装

| 技術要件 | 既存資産 | ギャップ |
|---------|---------|---------|
| `parse_scene_actors_line`関数 | 類似関数多数 | **Missing** - 新規作成必要 |
| `parse_actors_item`関数 | 類似関数多数 | **Missing** - 新規作成必要 |
| `Rule::scene_actors_line`マッチ | Pestが自動生成 | 要確認（grammar.pestから生成） |
| Span設定 | `Span::from(&pair.as_span())`パターン確立 | なし |

### 2.4 要件4: pasta_lua最低限対応

| 技術要件 | 既存資産 | ギャップ |
|---------|---------|---------|
| コンパイルエラー回避 | - | フィールド追加時に対応必要 |
| 未使用警告回避 | `#[allow(unused)]`パターン確立 | 適用必要 |
| `GlobalSceneScope`使用箇所 | `transpiler.rs`, `code_generator.rs`, `context.rs` | 変更不要（フィールド無視） |

### 2.5 要件5: テスト

| 技術要件 | 既存資産 | ギャップ |
|---------|---------|---------|
| パーステスト | `mod.rs`内テストパターン確立 | **Missing** - 新規テスト必要 |
| フィクスチャ | `tests/fixtures/`使用可 | オプション |

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（推奨）

**対象変更ファイル**:
1. `ast.rs`: `SceneActorItem`型追加、`GlobalSceneScope`に`actors`フィールド追加
2. `mod.rs`: `parse_scene_actors_line`関数追加、`parse_global_scene_scope`にmatch分岐追加

**トレードオフ**:
- ✅ 既存パターンに完全準拠
- ✅ 最小限のファイル変更
- ✅ レビュー・テストが容易
- ❌ 該当なし

**互換性評価**:
- `GlobalSceneScope`への`actors`フィールド追加は破壊的変更だが、pasta_luaはフィールドを無視するため実質的影響なし
- 新規コンストラクタの引数変更は不要（`new()`/`continuation()`内で`Vec::new()`初期化）

### Option B: 新規モジュール作成

**構成**:
- `ast.rs`に型追加
- 新規`scene_actors.rs`モジュールでパース処理

**トレードオフ**:
- ✅ 関心の分離
- ❌ ファイル増加
- ❌ 既存パターンと不一致
- ❌ 過剰設計

### Option C: ハイブリッドアプローチ

本ケースでは不要。Option Aが最適。

---

## 4. 実装複雑度・リスク評価

### 工数見積

| 領域 | 工数 | 根拠 |
|-----|------|------|
| AST型定義 | S (1日未満) | 既存パターンに準拠、2つの型追加のみ |
| パーサー変換 | S (1日未満) | 既存パターンに準拠、2つの関数追加 |
| pasta_lua対応 | S (1日未満) | コンパイルエラー確認のみ |
| テスト作成 | S (1日未満) | 既存テストパターンに準拠 |
| **合計** | **S (1-2日)** | 全体として軽量な拡張タスク |

### リスク評価

| リスク | レベル | 理由 |
|-------|-------|------|
| 技術的不確実性 | **Low** | 既存パターンに完全準拠 |
| 統合リスク | **Low** | pasta_luaはフィールド無視で対応可 |
| リグレッションリスク | **Low** | 既存機能への影響なし |
| パフォーマンスリスク | **Low** | `Vec`追加のみ、軽微なメモリ増 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option A: 既存コンポーネント拡張**

### キー設計決定
1. `SceneActorItem`を`ast.rs`に追加（`Span`付き）
2. `GlobalSceneScope`に`actors: Vec<SceneActorItem>`追加
3. `mod.rs`に`parse_scene_actors_line`と`parse_actors_item`追加
4. `parse_global_scene_scope`に`Rule::scene_actors_line`分岐追加

### 研究不要事項
- 外部依存なし
- 新技術導入なし
- アーキテクチャ変更なし

### 設計時の注意点
1. **`SceneActorLine` vs `SceneActorItem`**: 要件では`SceneActorLine`型を定義するとあるが、実際にはASTに保持するのは`Vec<SceneActorItem>`のみで十分。`SceneActorLine`は中間表現として不要の可能性あり（設計フェーズで確定）
2. **番号の型**: `digit_id`は全角数字を含む可能性があるため、パース時に正規化が必要（既存`normalize_number_str`関数を参照）

---

## 6. 結論

本仕様は既存アーキテクチャに完全に適合し、確立されたパターンに従って実装可能である。工数はS（1-2日）、リスクはLowと評価する。Option A（既存コンポーネント拡張）を推奨し、設計フェーズへの移行を推奨する。
