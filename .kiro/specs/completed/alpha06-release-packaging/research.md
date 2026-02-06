# Research & Design Decisions

## Summary
- **Feature**: `alpha06-release-packaging`
- **Discovery Scope**: Simple Addition（既存基盤を活用した手動ワークフロー）
- **Key Findings**:
  - `setup.bat` + `gh release create` で全リリースフローが完結する
  - `profile/` ディレクトリ（ログ・キャッシュ・セーブデータ）は `.nar` から除外すべき実行時生成物
  - `.nar` は ZIP 形式であり、PowerShell の `Compress-Archive` で生成可能

## Research Log

### .nar 形式の仕様
- **Context**: `.nar` ファイルの正確な仕様を確認
- **Sources Consulted**: SSP/伺か配布形式の慣例
- **Findings**:
  - `.nar` = ZIP 形式（拡張子のみ変更）
  - SSP はドラッグ＆ドロップで `.nar` をインストール可能
  - `install.txt` が必須（`type,ghost` / `name,hello-pasta` / `directory,hello-pasta`）
  - ZIP 内のルートは `ghost/`, `shell/`, `install.txt` 等がフラットに並ぶ構成
- **Implications**: `ghosts/hello-pasta/` 配下をそのまま ZIP 圧縮すれば正しい `.nar` になる

### profile/ ディレクトリの除外
- **Context**: ゴースト配布物に含めるべきでないファイルの特定
- **Sources Consulted**: 実際の `ghosts/hello-pasta/` ディレクトリ構成
- **Findings**:
  - `ghost/master/profile/` — pasta ランタイムの実行時生成物（キャッシュ、ログ、セーブデータ）
  - `shell/master/profile/` — SSP シェルの実行時生成物
  - `.gitignore` で `profile` は既に除外済み
  - ZIP 圧縮時に `**/profile/**` を除外する必要がある
- **Implications**: ZIP 圧縮スクリプトに除外パターンが必要

### ZIP 生成手法
- **Context**: Windows 環境での ZIP 圧縮方法
- **Sources Consulted**: PowerShell ドキュメント
- **Findings**:
  - `Compress-Archive` はディレクトリの ZIP 圧縮が可能
  - ただし `Compress-Archive` には除外パターン機能がない
  - 代替案: 一時ディレクトリにコピー（profile 除外）→ ZIP 圧縮 → .nar にリネーム
  - 別案: `setup.bat` に ZIP 生成ステップを追加（`tar -a -cf` でも ZIP 作成可能）
- **Implications**: スクリプトで profile 除外 → ZIP → .nar の手順を自動化

## Design Decisions

### Decision: パッケージングスクリプトの形式
- **Context**: `.nar` 生成を自動化するスクリプトの形式
- **Alternatives Considered**:
  1. `setup.bat` に ZIP ステップを追加
  2. 独立した `release.ps1` スクリプトを新規作成
- **Selected Approach**: 独立した `release.ps1` を `crates/pasta_sample_ghost/` に配置
- **Rationale**: `setup.bat` はゴースト生成に特化。リリース固有の処理（バージョン確認・ZIP・.nar 変換）は責務が異なる
- **Trade-offs**: ファイルが増えるが、責務の分離が明確になる

### Decision: profile/ の除外方法
- **Context**: ZIP 圧縮時に実行時生成物を除外する方法
- **Alternatives Considered**:
  1. `Compress-Archive` + 一時ディレクトリ方式
  2. `robocopy` で除外コピー → ZIP
- **Selected Approach**: `robocopy /MIR /XD profile /XF *.bak *.tmp` で一時ディレクトリにコピー → `Compress-Archive` → リネーム
- **Rationale**: `robocopy` の `/XD` オプションでディレクトリ除外、`/XF` でファイルパターン除外が可能。PowerShell ネイティブで完結。

## Risks & Mitigations
- `Compress-Archive` の ZIP 形式が SSP と互換性がない可能性 → 初回リリース時に SSP でインストール検証を実施
- `profile/` 以外にも除外すべきファイルが存在する可能性 → `*.bak` 等のパターンも確認

## References
- [SSP 公式サイト](http://ssp.shillest.net/) — ベースウェア仕様
- [GitHub CLI リリース作成](https://cli.github.com/manual/gh_release_create) — `gh release create` のドキュメント
