# Research Log: audit-pasta-check

## Discovery Scope
pasta_check CLIツール（~500行、5ファイル）の脆弱性監査・コード簡素化。ファイルI/O操作、ZIPアーカイブ作成、MD5ハッシュ計算、CLI入力処理を調査対象とする。

## 調査結果

### 1. パストラバーサル安全性

**現状分析**:
- `strip_prefix`によるパス正規化は各モジュールで実施済み（nar.rs, update_files.rs）
- `strip_prefix`失敗時はエラーを返して処理中断（良好）
- しかし、シンボリックリンクの追跡に対する防御がない
- `fs::read_dir` → `entry.path()` はシンボリックリンクを透過的に追跡する
- `copy_dir_inner`も同様にシンボリックリンクを追跡する

**リスク評価**: 低〜中。CLIツールはローカルで実行され、入力はユーザーが明示的に指定するパス。ただし、悪意あるゴーストデータをリリースビルドする場合にシンボリックリンクを通じた情報漏洩の可能性がある。

**対策方針**: `entry.file_type()` の `is_symlink()` チェックを追加し、シンボリックリンクをスキップする。

### 2. ZIPアーカイブ安全性

**現状分析**:
- `add_dir_to_zip`は`strip_prefix`で相対パスを生成し、`\` → `/`変換を実施
- `strip_prefix`が成功する限り、`..`コンポーネントは生成されない（`root`はfs::read_dirの起点であるため）
- ZIPエントリ名に対する明示的な`..`チェックは存在しない

**リスク評価**: 低。`strip_prefix`が正しく機能していれば安全だが、防御的プログラミングとして`..`チェックの追加は妥当。

### 3. MD5ハッシュ使用の適切性

**現状分析**:
- `update_files.rs`の`calculate_md5`でファイル変更検出に使用
- `updates.txt`はSSPネットワーク更新仕様のフォーマット（SSP側がMD5を要求）
- 暗号学的用途ではない（ファイル整合性チェック）

**評価**: 適切。SSP仕様がMD5を要求しているため、別のハッシュアルゴリズムに変更する必要はない。コメントでの用途明記のみ。

### 4. デッドコード

**発見**:
- `update_files.rs`: `generate_updates2_dau` に `#[allow(dead_code)]` が付与されている
- この関数は`updates2.dau`形式を生成するが、現在は`generate_update_files`から呼ばれていない
- `updates2.dau`はSSPの古い更新フォーマットで、`updates.txt`が後継
- briefに「`updates2.dau` は将来用に保持」とあるが、実際にこの関数を使う計画は不明

**対策方針**: `#[allow(dead_code)]`を除去し、関数自体も除去する。将来必要になればgit履歴から復元可能。

### 5. 冗長表現

**発見**:
- 各モジュールで繰り返される `map_err(|e| io::Error::new(io::ErrorKind::Other, e))` パターン
- `nar.rs`の`zip.finish()`後の`fs::metadata`呼び出し: `zip.finish()`は`Write`トレイトの`Result<W>`を返すため、`into_inner()`でファイルを取得し直接メタデータを取れる可能性あり → ただし`zip::ZipWriter::finish()`は`Result<W>`を返すが、実際にはWriteが消費されるためfsからのmetadataが必要。現行で問題なし。
- `update_files.rs`の日時変換ロジック: 手動実装だがSSP互換性のため依存追加は避けるべき（現行で適切）

## Design Decisions

- **シンボリックリンク対策**: `is_symlink()`チェックによるスキップ方式を採用。リンク先解決方式は複雑になるため不採用。
- **`..`チェック**: 防御的プログラミングとしてZIPエントリ名とコピー先パスに対するチェックを追加。
- **MD5**: 変更なし。コメント追記のみ。
- **デッドコード**: `generate_updates2_dau`と関連する`updates2.dau`ロジックを除去。
- **冗長パターン**: `map_err`パターンは出現箇所が限定的（2-3箇所）のため、ヘルパー関数化せず個別にイディオマティックに改善。
