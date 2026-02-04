# Gap Analysis: shiori-integration-test

## 1. 現状調査

### 1.1 関連アセット分析

| カテゴリ | ファイル/ディレクトリ | 役割 |
|----------|----------------------|------|
| テスト対象 | `crates/pasta_shiori/src/shiori.rs` | PastaShiori::load/request 実装 |
| 既存テスト | `crates/pasta_shiori/tests/shiori_lifecycle_test.rs` | ライフサイクルテスト（fixture使用） |
| テストユーティリティ | `crates/pasta_shiori/tests/common/mod.rs` | `copy_fixture_to_temp()` |
| サポートファイル | `crates/pasta_shiori/tests/support/` | scripts/, scriptlibs/ |
| ゴースト定義 | `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/` | 実際のゴースト定義 |
| 設定 | `crates/pasta_lua/src/loader/config.rs` | TalkConfig（[talk]セクション） |

### 1.2 既存パターン分析

#### テストパターン（shiori_lifecycle_test.rs）
```rust
// 1. fixture をテンポラリディレクトリにコピー
let temp = copy_fixture_to_temp("shiori_lifecycle");

// 2. PastaShiori を作成・初期化
let mut shiori = PastaShiori::default();
shiori.load(hinst, temp.path().as_os_str());

// 3. request を実行して検証
let response = shiori.request("GET SHIORI/3.0\r\nID: OnBoot\r\n\r\n").unwrap();
```

#### common::copy_fixture_to_temp() の動作
1. `tests/support/` の `scripts/` と `scriptlibs/` を先にコピー
2. `tests/fixtures/{fixture_name}/` を上書きコピー
3. `profile/` ディレクトリはスキップ

### 1.3 統合ポイント

| 統合ポイント | 現状 | 課題 |
|-------------|------|------|
| hello-pasta ゴースト | 直接アクセス可能 | コピー処理が必要 |
| pasta.toml [talk] | TalkConfig 実装済み | hello-pasta に未設定 |
| scripts/ ランタイム | hello-pasta に含まれる | support/ と重複管理不要 |
| OnBoot シーン | 2つ存在（ランダム選択） | 1つに削減必要 |

## 2. 要件実現性分析

### 2.1 要件-アセット対応表

| 要件 | 技術的ニーズ | 既存アセット | ギャップ |
|------|-------------|-------------|----------|
| R1: OnBoot 修正 | boot.pasta 編集 | ✅ 存在 | **修正必要**: 1シーンに削減 |
| R2: [talk] 設定 | pasta.toml 編集 | ✅ TalkConfig 実装済み | **追加必要**: セクション未設定 |
| R3: テスト環境 | ディレクトリコピー | ✅ copy_fixture_to_temp() | **拡張必要**: hello-pasta 対応 |
| R4: load 検証 | PastaShiori::load | ✅ 実装済み | なし |
| R5: request 検証 | レスポンス解析 | ✅ パターン存在 | **新規**: さくらスクリプト詳細検証 |
| R6: ファイル配置 | テストファイル作成 | ✅ 規約存在 | **新規**: shiori_sample_ghost_test.rs |

### 2.2 ギャップ詳細

#### Missing: hello-pasta 用コピー関数
- 現在の `copy_fixture_to_temp()` は `tests/fixtures/` と `tests/support/` を前提
- hello-pasta は `crates/pasta_sample_ghost/ghosts/` に配置
- **Research Needed**: 既存関数を拡張するか新規関数を作成するか

#### Missing: さくらスクリプト検証ヘルパー
- ウェイトタグ `\_w[ms]` のパース/検証
- スポット切り替え `\0`, `\1` の検出
- 表情タグ `\s[...]` の検出
- **Research Needed**: 既存のさくらスクリプト解析ユーティリティの有無

#### Constraint: setup.bat 依存
- boot.pasta 修正後は setup.bat の実行が必要
- CI/テスト自動化への影響
- **Constraint**: ゴースト再生成はテスト前提条件として手動実行

## 3. 実装アプローチオプション

### Option A: 既存構造を拡張

**概要**: 現在の common モジュールを拡張し、hello-pasta 対応を追加

**対象ファイル**:
- `crates/pasta_shiori/tests/common/mod.rs` - コピー関数追加
- `crates/pasta_shiori/tests/shiori_sample_ghost_test.rs` - 新規テスト

**変更内容**:
```rust
// common/mod.rs に追加
pub fn copy_sample_ghost_to_temp() -> TempDir {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sample_ghost = manifest_dir
        .parent().unwrap()
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
    // ... コピー処理
}
```

**Trade-offs**:
- ✅ 既存パターンとの一貫性
- ✅ common モジュールの再利用
- ❌ pasta_sample_ghost への依存がテストコードに追加
- ❌ クレート間参照パスがハードコード

### Option B: 新規 fixture として hello-pasta をコピー

**概要**: hello-pasta のコピーを tests/fixtures/ に配置し、既存パターンを完全に踏襲

**対象ファイル**:
- `crates/pasta_shiori/tests/fixtures/hello_pasta/` - 新規 fixture
- `crates/pasta_shiori/tests/shiori_sample_ghost_test.rs` - 新規テスト

**変更内容**:
- hello-pasta の必要ファイルを fixtures にコピー（または symlink）
- 既存の `copy_fixture_to_temp("hello_pasta")` で使用

**Trade-offs**:
- ✅ 既存テストパターンと完全に一致
- ✅ 依存関係が明確
- ❌ ファイル重複による保守コスト
- ❌ hello-pasta 更新時の同期が必要

### Option C: ハイブリッドアプローチ（推奨）

**概要**: common モジュールに pasta_sample_ghost 専用関数を追加しつつ、テスト実行時に直接コピー

**対象ファイル**:
- `crates/pasta_shiori/tests/common/mod.rs` - 新規関数追加
- `crates/pasta_shiori/tests/shiori_sample_ghost_test.rs` - 新規テスト
- `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/boot.pasta` - 修正
- `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` - [talk] 追加

**戦略**:
1. boot.pasta を修正（OnBoot を1つに）
2. pasta.toml に [talk] セクション追加
3. setup.bat でゴースト再生成
4. common に `copy_sample_ghost_to_temp()` 関数追加
5. テストファイル作成・検証

**Trade-offs**:
- ✅ 実際のゴースト定義を直接使用（リアルなテスト）
- ✅ 保守コストが低い（重複なし）
- ✅ 既存パターンを部分的に活用
- ❌ クレート間依存の管理が必要
- ❌ テスト実行前に setup.bat が必要

## 4. 複雑性とリスク評価

### 工数見積もり

| タスク | 工数 | 根拠 |
|--------|------|------|
| boot.pasta 修正 | S (1時間) | 単純な削除 |
| pasta.toml [talk] 追加 | S (1時間) | TalkConfig 定義済み |
| common 拡張 | S (2時間) | 既存パターン踏襲 |
| テストファイル作成 | M (4時間) | 新規テスト・検証ロジック |
| setup.bat 実行・確認 | S (30分) | 手動作業 |

**総工数**: **M (3-7日相当、実質1-2日)**

### リスク評価

| リスク | レベル | 軽減策 |
|--------|--------|--------|
| さくらスクリプト期待値の精度 | Medium | ウェイト設定を明示的に指定 |
| クレート間パス解決 | Low | CARGO_MANIFEST_DIR 使用 |
| テスト安定性 | Low | ランダム要素完全排除 |

**総合リスク**: **Low** - 既存パターンと実装済みコンポーネントを活用

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option C（ハイブリッド）** を推奨

### 主要決定事項
1. `copy_sample_ghost_to_temp()` 関数の実装方法
2. さくらスクリプト検証の詳細（部分一致 vs 完全一致）
3. [talk] セクションのウェイト値（テスト期待値との整合性）

### 調査継続項目
- さくらスクリプト出力のウェイトタグ生成パターン確認（refine-talk-conversion 仕様との整合）
- pasta_sample_ghost ビルド後の scripts/ 内容確認

### 前提条件
- setup.bat によるゴースト再生成はテスト実行前に完了していること
- i686-pc-windows-msvc ターゲットがインストールされていること（CI環境考慮）
