# Gap Analysis: ai-friendly-file-split

## 1. Current State Investigation

### インラインテスト実態調査

全クレートのソースファイル（`src/`配下）に存在する `#[cfg(test)] mod tests` ブロックを網羅的に調査した。

#### 調査結果サマリー

| ファイル | テスト行数 | ファイル比率 | テスト関数数 | privateアクセス | 判定 |
|----------|-----------|-------------|-------------|---------------|------|
| `shiori.rs` | 793行 | 66% | 20 | 6フィールド+1メソッド | **Must stay inline** |
| `scene_table.rs` | 604行 | 57% | 23 | 5フィールド+1メソッド | **Must stay inline** |
| `config.rs` | 423行 | 50% | 28 | 3関数 | Needs pub(crate) |
| `word_table.rs` | 402行 | 62% | 20 | 1フィールド(getter有) | Minor refactor |
| `runtime/mod.rs` | 343行 | 29% | 29 | **なし** | **Can externalize** |
| `cache.rs` | 314行 | 45% | 18 | 1定数 | Needs pub(crate) |
| `parser/mod.rs` | 267行 | 19% | 18 | 1関数 | Needs pub(crate) |
| `code_generator.rs` | 224行 | 22% | 11 | 2メソッド | Needs pub(crate) |
| `analysis.rs` | 138行 | 11% | 12 | 2関数 | Needs pub(crate) |
| `ast.rs` | 126行 | 14% | 14 | **なし** | **Can externalize** |
| **合計** | **3,634行** | — | **193** | — | — |

#### 判定カテゴリ別集計

| 判定 | ファイル数 | テスト行数計 | 外部化可能性 |
|------|-----------|------------|-------------|
| Can externalize（変更不要） | 2 | 469行 | ✅ そのまま `tests/` へ |
| Minor refactor（1行修正） | 1 | 402行 | ✅ getter使用に変更のみ |
| Needs pub(crate)（可視性変更） | 5 | 1,366行 | ✅ 少数のpub(crate)昇格 |
| Must stay inline（構造的制約） | 2 | 1,397行 | ❌ カプセル化が崩壊 |

### privateアクセスの詳細

#### Must stay inline: shiori.rs
テストが `PastaShiori` の6つのprivateフィールド（`hinst`, `load_dir`, `runtime`, `load_fn`, `request_fn`, `unload_fn`）に直接アクセスし、SHIORI内部状態（Lua関数キャッシュ、ランタイム有無）を検証している。フィールドをpubにすると、DLLインターフェースのカプセル化が崩壊する。

#### Must stay inline: scene_table.rs
テストが `SceneTable` を全privateフィールド（`labels`, `prefix_index`, `cache`, `random_selector`, `shuffle_enabled`）指定で直接構築。内部のRadixMapやキャッシュ構造が外部に露出するリスク。

#### 簡単に外部化できるもの
- **runtime/mod.rs**: 29テスト全てがpub APIのみ使用。変更なしで外部化可能
- **ast.rs**: 14テスト全てがpub型のコンストラクタのみ使用。変更なしで外部化可能
- **word_table.rs**: `table.entries.len()` → `table.entries().len()` の1行修正で外部化可能
- **cache.rs**: `CURRENT_VERSION` 定数を `pub(crate)` にするだけ
- **parser/mod.rs**: `normalize_number_str()` を `pub(crate)` にするだけ（15/18テストは変更不要）
- **analysis.rs**: `get_line_text()`、`line_byte_offset()` を `pub(crate)` にするだけ（9/12テストは変更不要）
- **config.rs**: `from_str()`、`default_log_file_path()`、`default_lua_search_paths()` を `pub(crate)` に
- **code_generator.rs**: `generate_action()`、`generate_var_set()` を `pub(crate)` に

### 既存テスト配置パターン

現在のプロジェクトには2種類のテスト配置が混在：

| パターン | 場所 | 現在の使用状況 |
|----------|------|---------------|
| インラインテスト | `src/*.rs` 内 `#[cfg(test)]` | 10ファイル、3,634行 |
| 外部統合テスト | `crates/*/tests/*.rs` | 多数（pasta_lua/tests/ に16ファイル等） |

外部統合テスト（`tests/`）は既に広く使われており、プロジェクトの慣例として確立されている。

## 2. Requirements Feasibility Analysis

### 要件とのギャップ

#### Requirement 1（ファイルサイズ基準）へのギャップ

**Gap: テスト配置ポリシーが未定義**

現要件はファイルサイズの上限のみ定めているが、「インラインテストを原則外部化する」というポリシーが欠けている。テストがソースファイル肥大化の最大要因（30〜66%）であり、サイズ基準だけでは根本対処にならない。

- **Missing**: インラインテストの原則禁止／制限ポリシー
- **Constraint**: 2ファイル（shiori.rs, scene_table.rs）はprivateフィールドアクセスのためインラインが必須

#### Requirement 2（ソースファイル分割）へのギャップ

**Gap: テスト外部化後の本体サイズの再評価が必要**

設計ではテスト分離後も300行を超えるファイルに対して責務分割を計画しているが、テスト外部化後の実際の本体サイズは再計測が必要。外部化により多くのファイルが300行以下になる可能性がある。

| ファイル | 現在 | テスト外部化後 | 300行超? |
|----------|------|--------------|---------|
| `parser/mod.rs` | 1,301行 | ~1,034行 | ✅ 要分割 |
| `analysis.rs` | 1,186行 | ~1,048行 | ✅ 要分割 |
| `runtime/mod.rs` | 1,026行 | ~683行 | ✅ 要分割 |
| `shiori.rs` | 1,000行 | 1,000行(inline維持) | ✅ 要分割 |
| `scene_table.rs` | 923行 | 923行(inline維持) | ✅ 要分割 |
| `code_generator.rs` | 892行 | ~668行 | ✅ 要分割 |
| `ast.rs` | 800行 | ~674行 | ✅ 要分割 |
| `config.rs` | 763行 | ~340行 | ✅ 微超 |
| `cache.rs` | 582行 | ~268行 | ❌ 基準以下！ |
| `word_table.rs` | 557行 | ~155行 | ❌ 基準以下！ |

**Finding**: テスト外部化により `cache.rs` と `word_table.rs` は本体分割が不要になる。

#### Requirement 3（テストファイル分割）へのギャップ

**Gap: 外部化されたインラインテストのサイズ管理**

インラインテストを `tests/` に外部化した場合、新たに500行超のテストファイルが生まれうる（shiori_tests: 793行、scene_table_tests: 604行）。これらもRequirement 3の分割基準に該当する。

- **Missing**: 外部化テストが500行超の場合の扱い（さらに分割するか、例外とするか）

#### Requirement 4（API互換性）— ギャップなし

テスト外部化は公開APIに影響しない。`pub(crate)` 昇格も外部クレートからのアクセスに変化なし。

#### Requirement 5（分割優先順位）へのギャップ

**Gap: テスト外部化フェーズが未定義**

現要件は「ソースの責務分割」と「テストファイル分割」の2種類のみ。「インラインテストの外部化」はこれらとは異なるカテゴリの作業であり、実行レイヤーの順序内にテスト外部化フェーズを追加すべき。

**提案する実行順序**:
1. **Phase A**: インラインテスト外部化（全クレート。最もリスクが低く効果が大きい）
2. **Phase B**: ソースファイルの責務分割（テスト外部化後に300行超のもののみ）
3. **Phase C**: テストファイル分割（`tests/`配下で500行超のもの）

## 3. Implementation Approach Options

### Option A: テスト外部化優先アプローチ（推奨）

**要件変更**: Requirement 1にテスト配置ポリシーを追加

1. **Phase A**: 8ファイルのインラインテストを `tests/` へ外部化
   - 2ファイルは変更不要で即座に外部化
   - 5ファイルは少数の `pub(crate)` 昇格で外部化
   - 1ファイルは1行のgetter使用変更で外部化
2. **Phase B**: テスト外部化後に300行超のソースファイルを責務分割
3. **Phase C**: テストファイル分割（500行超のもの）
4. 2ファイル（shiori.rs, scene_table.rs）のインラインテストは例外として記録

**Trade-offs**:
- ✅ ソースファイルの肥大化根本原因を解消
- ✅ 既存の `tests/` パターンに統一（プロジェクト慣例と一致）
- ✅ テスト外部化後にcache.rsとword_table.rsは本体分割不要に
- ✅ pub(crate)は外部クレートへの影響ゼロ
- ❌ 少数のprivate関数/定数の可視性が `pub(crate)` に変わる
- ❌ shiori.rsとscene_table.rsは依然として大きいまま

### Option B: 現設計維持 + #[path]パターン

**要件変更なし**: 現在の設計（design.md）のまま `#[path]` でテストを外部ファイル化

1. テストを `#[cfg(test)] #[path = "xxx_tests.rs"] mod tests;` で分離
2. ソースファイルの行数は減るが、テストは同一モジュールとして残る
3. 責務分割は設計通り実行

**Trade-offs**:
- ✅ privateアクセスの問題が完全に回避される
- ✅ 実装が単純（ファイル移動のみ）
- ❌ 対症療法：テストは依然としてソースモジュールの一部
- ❌ `src/` ディレクトリにテストファイルが混在（`xxx_tests.rs`）
- ❌ Rustの慣例から外れる（`#[path]` は非標準パターン）
- ❌ IDE/ツールがテストファイルをソースファイルとして扱う場合あり

### Option C: ハイブリッド（privateアクセスの有無で分岐）

1. **privateアクセスなし（8ファイル）**: テストを `tests/` に外部化（Option A）
2. **privateアクセスあり（2ファイル）**: `#[path]` パターン（Option B）
   - shiori.rs → `#[path = "shiori_tests.rs"] mod tests;`
   - scene_table.rs → `#[path = "scene_table_tests.rs"] mod tests;`

**Trade-offs**:
- ✅ 最善策の組み合わせ
- ✅ カプセル化を維持しつつ最大限の外部化
- ❌ 2つのパターンが混在（一貫性の低下）

## 4. Must stay inlineの2ファイルに関する追加調査

### shiori.rs のprivateアクセスパターン

テストが検証しているもの：
- `PastaShiori::load()` 後の内部状態（`runtime.is_some()`, `load_fn.is_some()`）
- `PastaShiori::request()` 後のキャッシュ状態
- `default_400_response()` のフォーマット

**代替案**: テスト用トレイトや検査メソッド（`#[cfg(test)]`付き）を追加すれば外部化可能だが、工数対効果が低い。shiori.rs本体は350行程度なので、テストをインラインに残しても`#[path]`で分離すれば十分。

### scene_table.rs のprivateアクセスパターン

テストが検証しているもの：
- `SceneTable` を全フィールド指定で直接構築（テスト用のセットアップ）
- `fn_name_to_search_key()` 内部変換ロジック

**代替案**: テスト用ビルダーパターン（`#[cfg(test)] impl SceneTable { fn test_new(...) }`）導入で外部化可能だが、既存のテスト設計を大幅に変更する必要がある。

## 5. Implementation Complexity & Risk

- **Effort**: **M (3–7 days)** — テスト外部化自体は機械的だが、ファイル数が多く`pub(crate)`昇格の影響確認が必要
- **Risk**: **Low** — カプセル化崩壊のリスクは `pub(crate)` 使用で回避。`cargo test --workspace` で即座にリグレッション検出可能

## Output Checklist

### Requirement-to-Asset Map

| Requirement | 既存Asset | Gap |
|-------------|----------|-----|
| 1. ファイルサイズ基準 | — | **Missing**: テスト配置ポリシー未定義 |
| 2. ソース分割 | 10ファイル対象（テスト外部化後8ファイルに削減可能） | **Constraint**: 2ファイルはテスト外部化後も再検討要 |
| 3. テスト分割 | 6テストファイル対象, 外部化により新たな500行超テスト発生 | **Missing**: 外部化テストのサイズ管理 |
| 4. API互換性 | pub(crate)のみ。外部API不変 | なし |
| 5. 分割優先順位 | レイヤー順は妥当 | **Missing**: テスト外部化フェーズの追加 |
| 6. ステアリング更新 | structure.md更新計画あり | なし |

### Recommendations for Design Phase

#### 推奨アプローチ: Option A（テスト外部化優先）

**Key Decisions**:
1. **要件にテスト配置ポリシーを追加**: 「src/配下のインラインテストは原則禁止。privateアクセスが構造的に必要な場合のみ#[path]パターンで許容」
2. **3フェーズ実行順序を採用**: テスト外部化 → ソース責務分割 → テストファイル分割
3. **Must stay inlineの2ファイルは `#[path]` パターン**: shiori.rsとscene_table.rsのみ例外

**Research Items**:
- `pub(crate)` 昇格対象の関数/定数が他クレートから意図せずアクセスされるリスクの確認（Rustでは `pub(crate)` は同一クレート内のみなので低リスク）
- shiori.rsの350行本体のさらなる分割可能性（`Shiori` trait定義を別ファイルに分離等）
