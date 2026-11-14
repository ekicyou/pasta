# テスト専用「国民の祝日」CSV の出典とライセンス

本ディレクトリの `syukujitsu.csv` は、祝日の規則（`pasta_core::calendar` の `holiday_name`）を
内閣府の公表データと全日で突き合わせる答え合わせテスト（`tests/holiday_cao_test.rs`）のための、
**テスト専用に同梱**したデータです。

- **ファイル**: `syukujitsu.csv`（1955 年〜2027 年、見出し 1 行＋`YYYY/M/D,名称` の行）
- **データセット ID**: `cao_20190522_0002`
- **データセットのページ**: https://www.data.go.jp/data/dataset/cao_20190522_0002
- **リソースの URL**: https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv
- **取得日**: 2026-11-15（元ファイル 21,538 バイト）
- **ライセンス**: CC BY（データカタログの `license_id` は `cc-by`。版の記載なし）
- **出典の表示**: 内閣府「「国民の祝日」について」

## 加工

次の 2 点だけを直しました。中身（行・年の範囲・名称）は変えていません。

- 文字コード: Shift_JIS → UTF-8（BOM なし）
- 改行: CRLF → LF

## 取得し直す方法

data.go.jp のページは証明書の期限切れで取れないことがあります。データカタログの CKAN API で
リソースの URL を確かめてから取得します。

- https://data.e-gov.go.jp/data/api/action/package_show?id=cao_20190522_0002

取得したら上の加工をして差し替えます（毎年の更新は手作業。テストは CSV から年の範囲を決めるのでコードは直しません）。

## 配布物に入らないこと

このデータは**テストの答え合わせにだけ**使います。ゴーストの配布物（NAR）にも、crates.io の
パッケージにも入りません（`crates/pasta_core/Cargo.toml` の `exclude` で外しています）。
