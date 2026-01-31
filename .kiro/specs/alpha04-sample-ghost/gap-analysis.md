# Gap Analysis Report

## Feature: alpha04-sample-ghost

**Analysis Date**: 2026-01-31  
**Requirements Version**: v2.0（ghosts/ 配布構成版）

---

## Executive Summary

本ギャップ分析では、9要件すべてについて既存コードベースとの差分を検証しました。

**結論**: 全要件は実装可能。既存基盤（PastaLoader, イベントシステム, pasta.shiori.act）を活用し、新規コードは画像生成とサンプルスクリプトに集中できます。

| カテゴリ | 既存コード流用 | 新規実装 |
|---------|---------------|---------|
| Rustクレート構造 | ワークスペースパターン | 新規追加 |
| PastaLoader統合 | ✅ 完全流用 | テストコード |
| イベントシステム | ✅ 完全流用 | ハンドラスクリプト |
| 画像生成 | ❌ なし | 新規実装 |
| 設定ファイル | ✅ パターン流用 | 新規ファイル |

---

## 1. Current State Investigation

### 1.1 PastaLoader パターン

**調査箇所**: `crates/pasta_lua/tests/loader_integration_test.rs`

```rust
// 既存パターン（完全流用可能）
fn copy_fixture_to_temp(name: &str) -> TempDir {
    let src = fixtures_path(name);
    let temp = TempDir::new().unwrap();
    copy_dir_recursive(&src, temp.path()).unwrap();
    // scripts/ と scriptlibs/ を自動コピー
    ...
}

let runtime = PastaLoader::load(temp.path()).unwrap();
```

**利点**: 
- `copy_fixture_to_temp` パターンで独立したテスト環境を構築
- `scripts/`, `scriptlibs/` を自動コピー（pastaランタイムモジュール利用可能）
- `PastaLoader::load()` 一発で環境構築完了

### 1.2 イベントハンドラ実装状況

| イベント | 状態 | テスト | 参照 |
|----------|------|-------|------|
| OnFirstBoot | ✅ 実装済 | shiori_event_test.rs L600 | Reference0（バニッシュ復帰） |
| OnBoot | ✅ 実装済 | shiori_event_test.rs | Reference0-7（起動情報） |
| OnClose | ✅ 実装済 | shiori_event_test.rs | Reference0（理由コード） |
| OnMouseDoubleClick | ✅ 実装済 | shiori_event_test.rs L857 | Reference0/4（スコープ/当たり判定） |
| OnTalk | ✅ 仮想イベント済 | virtual_event_dispatcher_test.rs | virtual_dispatcher.lua |
| OnHour | ✅ 仮想イベント済 | virtual_event_dispatcher_test.rs | virtual_dispatcher.lua |

### 1.3 pasta.shiori.act

**調査箇所**: `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua`

```lua
-- 既存パターン（完全流用可能）
local SHIORI_ACT = require("pasta.shiori.act")
local actors = {
    sakura = { name = "さくら", spot = "sakura" },
    kero = { name = "うにゅう", spot = "kero" },
}
local act = SHIORI_ACT.new(actors)
act:talk(actors.sakura, "Hello")
act:build() -- さくらスクリプト生成
```

**API一覧**（テストより抽出）:
- `SHIORI_ACT.new(actors)` - ビルダー初期化
- `act:talk(actor, text)` - 台詞追加
- `act:face(actor, surface)` - 表情変更
- `act:wait(seconds)` - 待機挿入
- `act:build()` - さくらスクリプト生成

### 1.4 pasta.toml 設定

**既存fixture**: `tests/fixtures/loader/with_ghost_config/pasta.toml`

```toml
ghost_name = "TestGhostWithSpotConfig"
version = "1.0.0"

[loader]
debug_mode = true

[ghost]
spot_switch_newlines = 2.0
```

**要件との差分**:
- `[package]` セクション: 新規追加が必要（要件7で定義）
- `talk_interval_min/max`: 既存 `virtual_dispatcher.lua` でサポート済み
- `hour_margin`: 既存でサポート済み

### 1.5 画像生成クレート

**調査結果**: `Cargo.toml` に `image` / `imageproc` 依存なし

```toml
# 現状の依存関係（画像関連なし）
[workspace.dependencies]
pest = "2.8"
mlua = { version = "0.11", ... }
toml = "0.9.8"
# image クレートなし！
```

**ギャップ**: 画像生成は新規実装が必要

---

## 2. Gap Analysis by Requirement

### Requirement 1: ディレクトリ構成

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| クレート作成 | ❌ | `crates/pasta_sample_ghost/` | **新規** |
| Cargo.toml | ❌ | クレート設定 | **新規** |
| ghosts/ 配布物 | ❌ | ゴーストファイル一式 | **新規** |
| パターン参照 | ✅ | pasta_lua/pasta_shiori | 流用可 |

**実装アプローチ**:
```
crates/pasta_sample_ghost/
├── Cargo.toml           # image依存、dev-dependencies: pasta_lua, tempfile
├── src/lib.rs           # 画像生成API
├── tests/integration_test.rs  # PastaLoader統合テスト
└── ghosts/hello-pasta/  # 配布物
```

**工数**: S（構造作成のみ）

---

### Requirement 2: 起動・終了トーク

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| OnFirstBoot ハンドラ | ✅ | スクリプト | スクリプト作成 |
| OnBoot ハンドラ | ✅ | スクリプト | スクリプト作成 |
| OnClose ハンドラ | ✅ | スクリプト | スクリプト作成 |
| pasta.shiori.act | ✅ | 利用 | **既存流用** |

**実装アプローチ**: `dic/boot.pasta` に Pasta DSL でハンドラ定義

```pasta
// 概念例（Pasta DSL）
「OnFirstBoot」
  sakura「はじめまして！pasta のサンプルゴーストです」
  kero「よろしくな！」
```

**工数**: S（スクリプト作成のみ）

---

### Requirement 3: ダブルクリック反応

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| OnMouseDoubleClick | ✅ | スクリプト | スクリプト作成 |
| Reference0解析 | ✅ | req.reference[0] | **既存流用** |
| キャラ判定 | ✅ | "0"=sakura, "1"=kero | ロジック記述 |

**実装アプローチ**: `dic/click.pasta` にハンドラ定義

**工数**: S

---

### Requirement 4: ランダムトーク

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| OnTalk 仮想イベント | ✅ | virtual_dispatcher.lua | **既存流用** |
| SCENE.search("OnTalk") | ✅ | シーン登録 | シーン関数作成 |
| 複数パターン | ❌ | 5〜10種 | スクリプト作成 |

**実装アプローチ**: `dic/talk.pasta` にランダムトーク定義

**工数**: S

---

### Requirement 5: 時報

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| OnHour 仮想イベント | ✅ | virtual_dispatcher.lua | **既存流用** |
| req.date.hour 参照 | ✅ | 時刻取得 | **既存流用** |
| 時間帯バリエーション | ❌ | 朝/昼/夕/夜 | スクリプト作成 |

**実装アプローチ**: `dic/talk.pasta` に時報セクション追加

**工数**: S

---

### Requirement 6: シェル素材

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| `image` クレート | ❌ | 透過PNG生成 | **新規依存追加** |
| プログラマティック生成 | ❌ | ピクトグラム描画 | **新規実装** |
| surface0-8, surface10-18 | ❌ | 18種PNG | **新規生成** |
| descript.txt | ❌ | シェル設定 | **新規作成** |

**実装アプローチ**:

```rust
// src/lib.rs 概念
use image::{Rgba, RgbaImage};

pub fn generate_surfaces(output_dir: &Path) -> Result<()> {
    for (id, expression) in EXPRESSIONS.iter() {
        let img = draw_character(*id, expression)?;
        img.save(output_dir.join(format!("surface{}.png", id)))?;
    }
    Ok(())
}
```

**依存追加**:
```toml
[dependencies]
image = "0.25"
imageproc = "0.25"
```

**工数**: M（描画ロジック実装）

---

### Requirement 7: 設定ファイル

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| pasta.toml 構造 | ✅ | 参照可 | **パターン流用** |
| [package] セクション | ❌ | 新規定義 | ファイル作成 |
| [ghost] 設定 | ✅ | 設定値 | ファイル作成 |

**実装アプローチ**: `ghosts/hello-pasta/ghost/master/pasta.toml` に記述

**工数**: S（コピー&調整）

---

### Requirement 8: テスト要件

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| PastaLoader | ✅ | テスト基盤 | **既存流用** |
| copy_fixture_to_temp | ✅ | テストヘルパー | **パターン流用** |
| 独立クレートテスト | ❌ | pasta_sample_ghost/tests/ | **新規作成** |

**実装アプローチ**:

```rust
// tests/integration_test.rs
use pasta_lua::loader::PastaLoader;

fn ghost_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ghosts/hello-pasta/ghost/master")
}

#[test]
fn test_onboot_response() {
    // ghosts/ から一時ディレクトリにコピー
    // scripts/, scriptlibs/ を pasta_lua から借用
    let runtime = PastaLoader::load(temp.path()).unwrap();
    // イベント発行テスト
}
```

**注意**: `pasta_lua` の scripts/, scriptlibs/ へのアクセス方法要検討

**工数**: M（ヘルパー整備 + テストケース）

---

### Requirement 9: ukadoc設定ファイル

| 項目 | 既存 | 必要 | ギャップ |
|------|-----|------|---------|
| install.txt | ❌ | ゴースト情報 | **新規作成** |
| ghost/master/descript.txt | ❌ | ゴースト設定 | **新規作成** |
| shell/master/descript.txt | ❌ | シェル設定 | **新規作成** |
| ukadoc仕様参照 | ✅ | research/ | 参照済み |

**実装アプローチ**: 仕様書に基づきファイル作成

**工数**: S

---

## 3. Implementation Approach Options

### Option A: Pictogram Style (採用) ✅

**戦略**: トイレマーク風のピクトグラムで視認性と実装コストのバランスを取る

| 項目 | 内容 |
|------|------|
| 画像 | トイレマーク風人型アイコン |
| 頭部 | 塗りつぶし円 |
| 胴体 | 男の子=四角形、女の子=スカート（台形） |
| 手足 | 線描画 |
| 表情 | 線描画で顔に重ねる（"^ ^", "- -" 等） |
| 色 | sakura=ピンク、kero=水色 |
| 依存 | `image` + `imageproc` |
| 工数 | 2-3日 |

**メリット**:
- 視認性が高い（標識として認識しやすい）
- フォント不要（CI再現性確保）
- `imageproc` で塗りつぶし円・ポリゴン・線描画が可能
- シンプルな実装（複雑な曲線なし）

**デメリット**:
- 装飾的な要素が少ない

---

### Option B: External Asset (代替案・不採用)

**戦略**: 外部フリー素材を同梱

| 項目 | 内容 |
|------|------|
| 画像 | CC0/パブリックドメインの素材 |
| 工数 | 1-2日 |

**メリット**:
- 実装コストゼロ
- 見た目が良い可能性

**デメリット**:
- ライセンス確認が必要
- 「自動生成」要件を満たさない
- CI再現性に課題

---

## 4. Recommendations

### 4.1 推奨アプローチ: Option A (Pictogram Style) ✅

**理由**:
1. トイレマーク風のピクトグラムで視認性と実装コストのバランスが良い
2. 「著作権問題なし」「CI再現可能」要件を確実に満たす
3. `imageproc` で必要な描画機能が揃っている（塗りつぶし円・ポリゴン・線）
4. フォント依存なし（CI環境差異を回避）

### 4.2 スクリプト配置先の推奨変更

**現状の懸念**:
- テストで `pasta_lua/scripts/` と `pasta_lua/scriptlibs/` が必要
- `pasta_sample_ghost` から参照する方法を検討要

**解決策**:
```rust
// テストで pasta_lua のパスを取得
let pasta_lua_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent().unwrap()  // crates/
    .join("pasta_lua");

// scripts/, scriptlibs/ をテンポラリにコピー
copy_dir_recursive(&pasta_lua_root.join("scripts"), &temp.path().join("scripts"));
copy_dir_recursive(&pasta_lua_root.join("scriptlibs"), &temp.path().join("scriptlibs"));
```

### 4.3 工数見積

| 要件 | 工数 | 備考 |
|------|-----|------|
| Req 1: ディレクトリ構成 | 0.5d | 構造作成 |
| Req 2-5: スクリプト | 1.5d | Pasta DSL記述 |
| Req 6: シェル素材 | 2.0d | 画像生成実装 |
| Req 7: 設定ファイル | 0.5d | pasta.toml作成 |
| Req 8: テスト | 1.5d | 統合テスト |
| Req 9: ukadoc設定 | 0.5d | descript.txt等 |
| **合計** | **6.5d** | Option A基準 |

---

## 5. Risk Assessment

| リスク | 影響度 | 発生確率 | 対策 |
|-------|-------|---------|------|
| 画像生成の複雑化 | 中 | 中 | Option Aで単純化 |
| pasta_lua依存問題 | 低 | 低 | ビルド時パスコピー |
| Pasta DSL構文問題 | 低 | 低 | 既存テスト参照 |
| CI環境差異 | 低 | 低 | 外部依存排除済み |

---

## 6. Conclusion

**実装可能性**: ✅ 全要件実装可能

**主要ギャップ**:
1. **画像生成**: `image` クレート新規依存 + 描画ロジック実装
2. **テスト環境**: pasta_lua への参照パス解決

**既存流用**:
- PastaLoader統合テストパターン: 完全流用
- イベントハンドラ: 全種実装済み
- pasta.shiori.act: 完全流用
- pasta.toml: パターン流用

**次ステップ**: 設計フェーズ（`/kiro-spec-design alpha04-sample-ghost`）

---

*Analysis completed by GitHub Copilot*
