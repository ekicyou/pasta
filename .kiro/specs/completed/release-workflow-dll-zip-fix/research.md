# Research & Design Decisions: release-workflow-dll-zip-fix

## Summary
- **Feature**: `release-workflow-dll-zip-fix`
- **Discovery Scope**: Extension / Simple Addition
- **Key Findings**:
  - `Compress-Archive` は PowerShell 5.1+ 標準搭載。追加依存なし
  - VSIX（`.vsix`）と NAR（`.nar`）はすでに ZIP 形式であり、追加圧縮は不要。DLL のみが対象
  - `gh release delete-asset` / `gh release upload` は冪等ではないため、手順の順序が重要

## Research Log

### PowerShell Compress-Archive の挙動
- **Context**: zip 圧縮に使用するコマンドレットの仕様確認
- **Sources Consulted**: PowerShell 公式ドキュメント、実行環境での動作確認
- **Findings**:
  - `Compress-Archive -Path <src> -DestinationPath <dst>` で単一ファイル圧縮可能
  - `-Force` フラグで既存ファイルを上書き（冪等性確保）
  - 出力先にディレクトリが存在する必要はない（自動作成）
  - zip 内のパスはファイル名のみ（ディレクトリ構造は含まれない）
- **Implications**: ワンライナーで完結。エラーハンドリングは PowerShell の `$?` / `$LASTEXITCODE` で判定

### VSIX / NAR ファイル形式の確認
- **Context**: 圧縮対象を DLL のみに限定してよいかの確認
- **Sources Consulted**: ローカル VSIX ファイルのヘッダ確認（マジックナンバー `50-4B-03-04` = ZIP）
- **Findings**:
  - `.vsix` は ZIP アーカイブ（`vsce package` が生成）
  - `.nar` も ZIP ベースのアーカイブ（伺か仕様）
  - DLL（`pasta.dll`）のみが非圧縮バイナリとして公開されている
- **Implications**: 圧縮対象は `pasta.dll` のみで正しい。VSIX と NAR は追加圧縮不要

### gh release コマンドの操作順序
- **Context**: v0.1.5 アセット差し替えの安全な手順確認
- **Sources Consulted**: `gh release delete-asset --help`, `gh release upload --help`
- **Findings**:
  - `gh release delete-asset <tag> <asset-name> -y` で確認スキップ可能
  - `gh release upload <tag> <file>` で同名アセットが存在するとエラー（`--clobber` フラグで上書き可能だが、明示的な delete → upload が安全）
  - `gh release download <tag> -p <pattern> -D <dir>` でアセット取得可能
- **Implications**: download → compress → delete-asset → upload の順序が最も安全

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存仕様の拡張 | release-workflow の3ファイルにピンポイント修正 | 最小変更、既存構造維持 | なし | **採用** |
| B: 新規スクリプト | zip 圧縮用の専用スクリプトを作成 | 再利用可能 | 過剰（ワンライナーで済む） | 不採用 |

## Design Decisions

### Decision: zip 圧縮の挿入位置
- **Context**: Phase 4（ゴーストビルド）のどの時点で zip 圧縮を実行するか
- **Alternatives Considered**:
  1. DLL 存在確認の直後（Phase 4 内）
  2. Phase 5（タグ作成）の前に独立ステップとして
- **Selected Approach**: Option 1 — Phase 4 内、DLL 存在確認直後
- **Rationale**: zip 圧縮は DLL の派生成果物であり、ゴーストビルドの文脈に属する。コミット前に圧縮を完了することで、Phase 6 のアセットリストがシンプルになる
- **Trade-offs**: Phase 4 のステップ数が増えるが、論理的に一貫性がある
- **Follow-up**: なし

### Decision: v0.1.5 差し替え時の DLL 取得方法
- **Context**: ローカルの DLL を使うか、リリースからダウンロードするか
- **Alternatives Considered**:
  1. ローカルの `target/i686-pc-windows-msvc/release/pasta.dll` を使用
  2. `gh release download` でリリース済み DLL を取得
  3. `git checkout v0.1.5` でリビルド
- **Selected Approach**: Option 2 — リリースからダウンロード
- **Rationale**: 開発者との議論により、DLL 同一性の正確な担保が重要と判断。ローカルの DLL がリリース時と同一である保証がないため
- **Trade-offs**: ネットワーク依存が発生するが、DLL 同一性が確実に担保される
- **Follow-up**: なし
