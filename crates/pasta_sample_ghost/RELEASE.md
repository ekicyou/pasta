# hello-pasta リリース手順書

> このドキュメントは AI と相談しながらリリースを進めるためのガイドです。

## 前提条件

- [x] `release.bat` を実行してゴースト配布物を生成し `hello-pasta.nar` を作成済み
- [x] GitHub CLI (`gh`) がインストール・認証済み (`gh auth status` で確認)

## リリース手順

### Step 1: release.bat 実行

```bat
cd crates\pasta_sample_ghost
release.bat
```

- ビルド、ゴースト生成、バリデーション、`.nar` ファイル作成まで一括実行されます
- 表示されたバージョン番号を確認してください

### Step 2: リリースノート作成

下記テンプレートを元に `release-notes.md` を作成してください。
AI と相談しながら、変更点を記入するのがおすすめです。

### Step 3: GitHub Release 公開

```bash
gh release create v{VERSION} "crates/pasta_sample_ghost/hello-pasta.nar" \
  --title "hello-pasta v{VERSION}" \
  --notes-file release-notes.md
```

## リリースノートテンプレート

```markdown
# hello-pasta v{VERSION}

## リリース概要

hello-pasta ゴーストのアルファリリースです。

### 変更点・新機能

- （ここに変更点を記入）

### 含まれるコンポーネント

| コンポーネント | バージョン | 説明 |
|---------------|-----------|------|
| pasta.dll | v{VERSION} | SHIORI DLL (x86) |
| hello-pasta ゴースト | v{VERSION} | サンプルゴースト |

## 必要環境

- **OS**: Windows (x86/x64)
- **SSP**: 2.x 以上
- **アーキテクチャ**: x86 (32bit DLL)

## インストール方法

### 方法 1: ドラッグ＆ドロップ（推奨）

1. `hello-pasta.nar` をダウンロード
2. SSP のウィンドウに `.nar` ファイルをドラッグ＆ドロップ
3. インストール確認ダイアログで「はい」を選択

### 方法 2: 手動展開

1. `hello-pasta.nar` をダウンロード
2. `.nar` の拡張子を `.zip` に変更
3. ZIP を展開し、中身を SSP の `ghost` フォルダにコピー

## 動作確認方法

1. SSP を起動（または再起動）
2. タスクトレイの SSP アイコンを右クリック → 「ゴーストの切り替え」
3. 「hello-pasta」を選択
4. ゴーストが表示され、会話が開始されることを確認
5. 右クリックメニューが正常に表示されることを確認

## 既知の問題

- （既知の問題があれば記入）

## 問題報告

バグ報告・機能リクエストは GitHub Issues へお願いします:
https://github.com/ekicyou/pasta/issues
```

## トラブルシューティング

### `gh` コマンドが見つからない

```bash
# GitHub CLI のインストール
winget install GitHub.cli
# または https://cli.github.com/ からダウンロード

# 認証
gh auth login
```

### タグが既に存在する

```bash
# 既存タグを削除してから再作成
gh release delete v{VERSION} --yes
git tag -d v{VERSION}
git push origin :refs/tags/v{VERSION}
```

### .nar を SSP が認識しない

- `Compress-Archive` の ZIP 形式が SSP と非互換の可能性があります
- その場合は 7-Zip などの外部ツールで ZIP を再作成してください：
  ```bash
  7z a -tzip hello-pasta.zip .\ghosts\hello-pasta\*
  ren hello-pasta.zip hello-pasta.nar
  ```
