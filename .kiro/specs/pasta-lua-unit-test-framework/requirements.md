# 要件定義：pasta_lua ユニットテストフレームワーク導入

## 背景

pasta_lua は Lua スクリプト層として、Rust の mlua によって管理される Lua VM統合を提供しています。現在、Rust側の統合テストのみ存在し、Lua スクリプト層そのもののユニットテストフレームワークが欠落しています。

## 目標

**pasta_lua に Lua デファクトスタンダード テストスイーツを導入し、スクリプト層のユニットテストを可能にする。**

## スコープ

### 導入対象フレームワーク：**Busted**

**理由：**
- Luaのデファクトスタンダード（1.6k★ on GitHub）
- RSpec/Mocha風の自然な読みやすいテスト記述
- Mock/Spy機能搭載
- CI/CD統合（GitHub Actions可能）
- Lua 5.1, LuaJIT, Lua 5.5対応
- JSON, TAP出力対応
- Docker化済み

### インストール戦略

**方式A：luarocks経由（推奨）**
```
luarocks install busted
```

**方式B：Docker コンテナ経由（CI/CD用）**
```
docker run ghcr.io/lunarmodules/busted:latest
```

## 要件

### R1: テストスイート導入
- [ ] Cargo.toml に luarocks ツール依存を追加
- [ ] tests/ ディレクトリに busted 設定ファイル `.busted` を配置
- [ ] luarocks に busted をインストール
- [ ] Busted CLI を通じてテスト実行可能にする

### R2: サンプルテスト作成
- [ ] transpiler_integration_test.rs の機能をLua側でも検証
- [ ] Code Generator の基本動作をテスト
- [ ] Context 管理の動作をテスト
- [ ] エラーハンドリングのテスト例

### R3: ローカル実行環境
- [ ] `luarocks install` でセットアップ可能
- [ ] `busted` コマンドでテスト実行
- [ ] `busted --verbose` で詳細出力

### R4: CI/CD統合（Optional）
- [ ] GitHub Actions で Busted 実行 (lunarmodules/busted@v0)
- [ ] テスト結果レポート出力

## 非要件

- Lua 側の本業務ロジック実装（別仕様）
- カバレッジ測定ツール導入（luacov 検討は後続）
- 独自テストフレームワーク開発

## 品質基準

| 項目 | 基準 |
|------|------|
| テスト実行可能性 | `busted` コマンドで全テスト実行可能 |
| サンプルテスト | 最小3個のテストを実装 |
| エラーハンドリング | テスト失敗時に明確なエラーメッセージ出力 |
| ドキュメント | tests/ に README.md で使用方法記載 |

## 定義言語

- テストコード：Lua（Busted準拠）
- ドキュメント：Japanese（この仕様と同じ）

## 依存する既知の外部要件

- **Lua 5.1** 以上
- **luarocks** インストール済み
- **mlua** クレート（既に導入済み）

## 成功基準

1. `cargo build` 時に Lua 環境が自動セットアップされる
2. `busted` コマンドで tests/*.lua のテストが実行される
3. GitHub Actions で Busted テストが自動実行される（Optional）
4. テスト失敗時に CI が失敗する

---

**仕様作成日：** 2025-12-31  
**対象フェーズ：** Phase 0（基盤構築）  
**優先度：** P1
