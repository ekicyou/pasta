# Implementation Gap Analysis

## 分析概要

**仕様**: pasta-lua-cache-transpiler  
**分析日**: 2026-01-22  
**フェーズ**: Requirements Generated

### エグゼクティブサマリー

- **スコープ**: Pasta DSL → Lua トランスパイル結果のキャッシュファイル生成機能
- **既存実装**: トランスパイル機能は完備、キャッシュ保存も部分実装済み（debug_mode時のみ）
- **主要ギャップ**: ファイルタイムスタンプ比較による増分トランスパイル機能が未実装
- **推奨アプローチ**: Option B（新規コンポーネント作成）- CacheManager による明確な責任分離

---

## 1. 現状調査

### 1.1 既存資産の特定

#### 主要コンポーネント

| コンポーネント      | ファイル                 | 現在の責務                                   |
| ------------------- | ------------------------ | -------------------------------------------- |
| **PastaLoader**     | `loader/mod.rs`          | 起動シーケンス統合、全体オーケストレーション |
| **LuaTranspiler**   | `transpiler.rs`          | AST → Luaコード変換                          |
| **LoaderConfig**    | `loader/config.rs`       | pasta.toml設定管理                           |
| **FileDiscovery**   | `loader/discovery.rs`    | globパターンによるファイル検出               |
| **TranspileResult** | `loader/mod.rs` (struct) | トランスパイル結果保持                       |

#### 関連データ構造

```rust
// 既存: loader/mod.rs
pub struct TranspileResult {
    pub module_name: String,      // 例: "dic_baseware_system"
    pub lua_code: String,
    pub source_path: PathBuf,
}

// 既存: loader/config.rs
pub struct LoaderConfig {
    pub pasta_patterns: Vec<String>,           // デフォルト: ["dic/*/*.pasta"]
    pub transpiled_output_dir: String,         // デフォルト: "profile/pasta/cache/lua"
    pub debug_mode: bool,                      // デフォルト: true
    // ...
}
```

### 1.2 既存の処理フロー

**現在のPastaLoader起動シーケンス** (`loader/mod.rs::load_with_config`):

```
1. Phase 1: 設定ロード (PastaConfig::load)
2. Phase 2: ディレクトリ準備
   - profile/pasta/save, cache 等を作成
   - cache/lua を「完全削除して再作成」← ⚠️ 既存キャッシュ破棄
3. Phase 3: ファイル検出 (discovery::discover_files)
   - globパターンで .pasta ファイルを検出
4. Phase 4: 全ファイルトランスパイル (transpile_all)
   - 全ファイルを無条件にトランスパイル ← ⚠️ 増分処理なし
5. Phase 5: キャッシュ保存 (save_cache_files) - debug_mode 時のみ
   - 単純なファイル書き込み、タイムスタンプ比較なし
6. Phase 6: ランタイム初期化
```

**キャッシュ保存の現在実装** (`loader/mod.rs::save_cache_files`):

```rust
fn save_cache_files(
    base_dir: &Path,
    output_dir: &str,
    results: &[TranspileResult],
) -> Result<(), LoaderError> {
    let cache_dir = base_dir.join(output_dir);
    for result in results {
        let cache_file = cache_dir.join(format!("{}.lua", result.module_name));
        fs::write(&cache_file, &result.lua_code)?;
        debug!(file = %cache_file.display(), "Saved cache");
    }
    Ok(())
}
```

### 1.3 既存のアーキテクチャパターン

- **レイヤー構造**: loader → transpiler → runtime という明確な階層
- **エラーハンドリング**: Result型による伝播、LoaderError で統一
- **設定管理**: pasta.toml による外部設定、デフォルト値提供
- **モジュール命名**: `path_to_module_name` による一貫した変換（`/` → `_`）
- **ログ戦略**: tracing クレート利用、構造化ログ
- **テスト配置**: 各モジュール内に `#[cfg(test)]` セクション

---

## 2. 要件実現性分析

### 2.1 技術的要求事項の抽出

| 要件領域                     | 技術要素                                   | 既存実装状況                    |
| ---------------------------- | ------------------------------------------ | ------------------------------- |
| **Req1: ファイル変更検出**   | ファイルメタデータ取得、タイムスタンプ比較 | ❌ 未実装                        |
| **Req2: キャッシュ出力**     | ファイルI/O、ディレクトリ作成              | ✅ 部分実装（save_cache_files）  |
| **Req3: scene_dic.lua生成**  | テキスト生成、require文列挙                | ❌ 未実装                        |
| **Req4: モジュール命名**     | パス→モジュール名変換                      | ✅ 実装済（path_to_module_name） |
| **Req5: ローダー統合**       | scene_dic.lua の require                   | ❌ 未実装                        |
| **Req6: エラーハンドリング** | 部分失敗許容、サマリー報告                 | △ LoaderError 基盤あり          |
| **Req7: パス解決**           | pasta.toml 連携                            | ✅ 実装済（LoaderConfig）        |

### 2.2 ギャップと制約の特定

#### Missing（完全欠落）

1. **ファイルタイムスタンプ比較ロジック**
   - `std::fs::metadata()` によるmtime取得が必要
   - Pastaファイルとキャッシュファイルの比較判定
   
2. **scene_dic.lua 生成機能**
   - 全キャッシュモジュールのrequire文列挙
   - `require("pasta").finalize_scene()` 呼び出し追加
   - モジュール名の `pasta.scene.<path>` 形式への変換

3. **増分トランスパイル制御**
   - トランスパイル対象の選別（変更あり/スキップ）
   - スキップ時の既存キャッシュ保持

4. **scene_dic.lua の自動ロード**
   - ランタイム初期化時の `require "pasta.scene_dic"` 呼び出し

#### Constraint（既存制約）

1. **cache/lua ディレクトリの完全削除**（`prepare_directories`）
   - 現在は起動時に `remove_dir_all` → `create_dir_all` を実行
   - ⚠️ 増分トランスパイルと矛盾：既存キャッシュが破棄される
   - **対応必須**: 削除処理を条件付きに変更、またはディレクトリスキャン方式に変更

2. **debug_mode依存のキャッシュ保存**
   - 現在はdebug_mode=true時のみキャッシュを保存
   - **対応必須**: キャッシュ機能は常時有効化（本仕様の前提）

3. **モジュール命名とディレクトリ構造**
   - 既存: `dic_baseware_system`（フラット構造、アンダースコア区切り）
   - 要件: `pasta.scene.baseware.system`（階層構造、ドット区切り）
   - **Design Decision**: Option A採用 - ディレクトリ階層を再現
     - 出力: `{transpiled_output_dir}/pasta/scene/{relative_path}.lua`
     - 例: `dic/subdir/scene.pasta` → `pasta/scene/subdir/scene.lua`
     - Lua標準のモジュール解決に準拠

#### Unknown（調査要）

1. **scene_dic.lua のロードタイミング**
   - Rust側でrequireするか、main.luaに追記するか
   - 設計フェーズで判断必要

2. **finalize_scene() の実体**
   - 本仕様ではスコープ外だが、呼び出しシグネチャの確認必要

### 2.3 複雑性シグナル

- **アルゴリズム**: ファイルタイムスタンプ比較（単純）
- **ワークフロー**: 増分処理の制御フロー（中程度）
- **外部統合**: ファイルシステムメタデータ、既存トランスパイラーとの統合（低）
- **全体複雑度**: 中（既存基盤を活用、新規アルゴリズムは少ない）

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張

#### 拡張対象ファイル

- `loader/mod.rs::PastaLoader`
  - `prepare_directories`: cache削除ロジックを条件付きに変更
  - `transpile_all`: タイムスタンプ比較による選別追加
  - `save_cache_files`: scene_dic.lua 生成追加
- `loader/mod.rs::TranspileResult`
  - `needs_transpile: bool` フィールド追加？

#### 互換性評価

- ✅ 既存API（`PastaLoader::load`）は変更不要
- ⚠️ `transpile_all` の責務増加（ファイル検出 → 比較 → トランスパイル → 辞書生成）
- ⚠️ `prepare_directories` の条件分岐複雑化

#### トレードオフ

- ✅ 新規ファイル不要、実装速度が速い
- ✅ 既存テストの再利用が容易
- ❌ PastaLoader の Single Responsibility 違反リスク
- ❌ `transpile_all` が肥大化（80行 → 150行程度に増加予想）
- ❌ キャッシュ管理ロジックがloaderに混在

### Option B: 新規コンポーネント作成（推奨）

#### 新規ファイル

**`loader/cache_manager.rs`** - キャッシュ管理専用モジュール

```rust
pub struct CacheManager {
    base_dir: PathBuf,
    cache_dir: PathBuf,
}

impl CacheManager {
    /// ファイルタイムスタンプ比較
    pub fn needs_transpile(&self, source: &Path) -> Result<bool, LoaderError>;
    
    /// キャッシュファイル保存
    pub fn save_cache(&self, result: &TranspileResult) -> Result<(), LoaderError>;
    
    /// scene_dic.lua 生成
    pub fn generate_scene_dic(&self, modules: &[String]) -> Result<(), LoaderError>;
    
    /// キャッシュディレクトリの準備（既存ファイル保持）
    pub fn prepare_cache_dir(&self) -> Result<(), LoaderError>;
}
```

#### 統合ポイント

- `PastaLoader::load_with_config` から `CacheManager` を生成
- Phase 2: `CacheManager::prepare_cache_dir()` 呼び出し
- Phase 4: トランスパイル前に `needs_transpile()` でフィルタリング
- Phase 5: `save_cache()` および `generate_scene_dic()` 呼び出し

#### 責任境界

| コンポーネント    | 責務                                             |
| ----------------- | ------------------------------------------------ |
| **PastaLoader**   | 起動シーケンス統合、Phase制御                    |
| **CacheManager**  | キャッシュライフサイクル管理、タイムスタンプ判定 |
| **LuaTranspiler** | AST → Lua変換（既存のまま）                      |
| **FileDiscovery** | ファイル検出（既存のまま）                       |

#### トレードオフ

- ✅ 単一責任原則に準拠、保守性向上
- ✅ キャッシュロジックのテスト独立性
- ✅ 将来のキャッシュ戦略変更が容易
- ❌ ファイル数増加（+1ファイル）
- ❌ インターフェース設計の追加工数

### Option C: ハイブリッドアプローチ

#### 組み合わせ戦略

1. **Phase 1**: Option A でMVP実装（`PastaLoader` 直接拡張）
2. **Phase 2**: 動作確認後、Option B にリファクタリング（`CacheManager` 抽出）

#### 段階的実装

- 初期フェーズ: 最小限の変更でタイムスタンプ比較を実装
- 後続フェーズ: コード品質改善、CacheManager への分離

#### リスク軽減

- ✅ 早期の動作確認が可能
- ✅ リファクタリング時にテストで品質保証
- ⚠️ 計画なき段階的実装は技術的負債化リスク

#### トレードオフ

- ✅ 段階的リスク低減
- ✅ MVP → Production への明確な道筋
- ❌ 2度の実装コスト（初期 + リファクタリング）
- ❌ 中途半端な状態でのマージリスク

---

## 4. 実装複雑度とリスク

### 工数見積もり

**M (Medium: 3-7日)**

- タイムスタンプ比較ロジック: 1日
- scene_dic.lua 生成機能: 1日
- ローダー統合（scene_dic requireの実装場所判断含む）: 1日
- 既存ロジック修正（cache削除の条件化等）: 1日
- テスト作成（統合テスト、ユニットテスト）: 2日
- ドキュメント更新（README.md、コメント）: 1日

**根拠**:
- 既存トランスパイル基盤が完備
- ファイルI/O、設定管理パターンが確立済み
- 新規アルゴリズムは単純（mtime比較のみ）
- 統合ポイントが明確

### リスク評価

**Medium（中リスク）**

**技術リスク（低）**:
- ✅ 使用技術は既知（std::fs、既存パターン）
- ✅ 外部依存なし、Rust標準ライブラリのみ
- ✅ 性能要件は緩い（起動時の一度きり処理）

**統合リスク（中）**:
- ⚠️ cache削除処理の変更による既存挙動への影響
- ⚠️ モジュール命名規則の不一致（要確認）
- ⚠️ scene_dic.lua ロードタイミングの判断

**アーキテクチャリスク（低）**:
- ✅ 既存レイヤー構造を維持
- ✅ 公開APIへの破壊的変更なし

**根拠**:
- 既存パターンに沿った実装で対応可能
- 破壊的変更は限定的（cache削除ロジックのみ）
- 設計判断項目が存在（scene_dic.lua のrequire方法）

---

## 5. 設計フェーズへの推奨事項

### 推奨実装アプローチ

**Option B: 新規コンポーネント作成（CacheManager）**

**理由**:
1. **保守性**: キャッシュ管理の責任を明確に分離
2. **テスタビリティ**: CacheManager 単独でのユニットテスト可能
3. **拡張性**: 将来のキャッシュ戦略変更（TTL、依存関係追跡等）に対応しやすい
4. **品質**: 既存のPastaLoader複雑化を回避

### 設計時の重要判断事項

1. **scene_dic.lua のrequireタイミング**
   - [ ] Option 1: Rust側（PastaLuaRuntime::from_loader内）で `lua.load(...).exec()` により明示的にrequire
   - [ ] Option 2: main.lua テンプレートに `require "pasta.scene_dic"` を追記
   - **推奨**: Option 1（Rust側制御）- ユーザーファイル変更不要、確実性高い

2. **モジュール命名規則の統一**
   - 現在: `dic_baseware_system`（既存のpath_to_module_name）
   - 要件: `pasta.scene.baseware.system`
   - **Research Needed**: 
     - Luaのrequire仕様確認（`.` はパス区切りと解釈される）
     - 既存コードへの影響範囲調査
     - **推奨**: `pasta/scene/baseware/system.lua` というファイル構造を採用し、require文では `"pasta.scene.baseware.system"` とする

3. **cache削除処理の扱い**
   - [ ] Option 1: 条件付き削除（初回のみ、またはコマンドフラグ）
   - [ ] Option 2: 削除処理を完全廃止、ファイル単位で上書き
   - **推奨**: Option 2（削除廃止）- 増分処理と整合

### 技術調査項目

1. **Luaモジュールパス解決の詳細**
   - `require "pasta.scene.system"` と `pasta/scene/system.lua` の対応確認
   - package.path 設定への影響

2. **finalize_scene() 呼び出しシグネチャ**
   - 引数の有無、戻り値の扱い
   - エラーハンドリング方法

3. **ファイル削除時のキャッシュクリーンアップ戦略**
   - 孤立したキャッシュファイルの扱い（Requirements 3.7に該当）
   - scene_dic.lua 生成時に既存キャッシュをスキャンして同期

---

## 6. 要件-資産マッピング

| 要件                                    | 既存資産                     | ギャップタグ   | 対応方針                                     |
| --------------------------------------- | ---------------------------- | -------------- | -------------------------------------------- |
| Req1.1: キャッシュファイル存在確認      | -                            | **Missing**    | CacheManager::needs_transpile 新規実装       |
| Req1.2: タイムスタンプ比較（新しい）    | -                            | **Missing**    | std::fs::metadata 利用                       |
| Req1.3: キャッシュ不在時マーク          | -                            | **Missing**    | 同上                                         |
| Req1.4: タイムスタンプ比較（古い/同じ） | -                            | **Missing**    | 同上                                         |
| Req1.5: ミリ秒精度判定                  | -                            | **Missing**    | SystemTime 比較                              |
| Req2.1-2.5: キャッシュ出力              | save_cache_files             | **Constraint** | パス構造変更（pasta/scene/サブディレクトリ） |
| Req3.1: scene_dic.lua 常時再生成        | -                            | **Missing**    | CacheManager::generate_scene_dic 新規実装    |
| Req3.2-3.4: require文列挙               | -                            | **Missing**    | 同上                                         |
| Req3.5: finalize_scene() 呼び出し       | -                            | **Missing**    | 固定テンプレート追加                         |
| Req3.6-3.7: ファイル追加/削除対応       | -                            | **Missing**    | キャッシュディレクトリスキャン               |
| Req4.1-4.5: モジュール命名              | path_to_module_name          | **Constraint** | 新関数 path_to_scene_module_name 追加        |
| Req5.1: scene_dic require               | -                            | **Missing**    | PastaLuaRuntime 拡張                         |
| Req5.2-5.4: ローダー統合                | PastaLuaRuntime::from_loader | **Extend**     | require実行追加                              |
| Req6.1-6.5: エラーハンドリング          | LoaderError                  | **Extend**     | 部分失敗サマリー追加                         |
| Req7.1-7.5: パス解決                    | LoaderConfig                 | **OK**         | 既存実装で対応可能                           |

---

## まとめ

### 実装難易度
**Medium（3-7日）** - 既存基盤が整備されており、新規アルゴリズムは単純

### 推奨アプローチ
**Option B（新規CacheManager作成）** - 責任分離による保守性・拡張性の確保

### クリティカルパス
1. モジュール命名規則の統一（`pasta.scene.<path>` への移行）
2. cache削除処理の廃止または条件化
3. scene_dic.lua のrequire実装場所の決定

### 次のステップ
設計フェーズにて以下を詳細化：
- CacheManager API設計
- scene_dic.lua のrequire統合方法確定
- モジュールパス構造の最終決定
- エラーハンドリング戦略の詳細化
