# ロギングとエンコーディング

ごきげんよう。ゴーストの足跡を書き残すロギングと、文字化けからゴーストを守るエンコーディング――地味ですけれど、欠かせない縁の下の力持ちですわ。
その働きぶりを、わたくしがしっかりご紹介いたします。さあ、参りましょう。

---

この章では、ランタイムのロギングとエンコーディングを扱う。利用者から見た `@pasta_log` と `@enc` の使い方は [@pasta_log](../lua/modules/pasta-log.md) と [@enc](../lua/modules/enc.md) が正である。

## 目的と責務

ロギングは、tracing の初期化・ロガーの登録・`@pasta_log` の実装・ファイル出力を担う。エンコーディングは、OS 別の文字コード変換の実装・`@enc` の実装・SHIORI 境界での文字コードの扱いを担う。この 2 つがこの章の責務である。

## 構成要素

tracing の初期化とロガー登録を行うロギング部、Lua へ公開する `@pasta_log`、OS 別の実装を持つエンコーディング部、Lua へ公開する `@enc` から成る。

## 処理とデータの流れ

ランタイムの起動時にロギングが初期化され、Rust 側と Lua 側のログがロガーを通じてファイルへ出力される。システムのコードページを要する文字列の受け渡しは、エンコーディング部を通じて変換される。

## 境界の受け渡し

Rust 側がロガーと文字コード変換の実装を所有し、Lua 側へは `@pasta_log` と `@enc` のモジュールとして渡す。

## 不変条件と制約

Rust 側の文字列は UTF-8 で扱う。Windows では、ANSI API を使う箇所（Lua のファイルアクセス等）へ渡す文字列をシステムのコードページへ変換する。Windows 以外では変換せずに UTF-8 のまま渡す。

## ソースの所在

- `crates/pasta_lua/src/logging/`
- `crates/pasta_lua/src/encoding/`
- `crates/pasta_lua/src/runtime/log.rs`
- `crates/pasta_lua/src/runtime/enc.rs`

## 経緯

- [lua-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-logging) — Lua からのロギング
- [logger-configuration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/logger-configuration) — ログ出力の設定

---

目立たぬ働きこそ、本当に大切なものですわ。フンッ、わたくしのように、ね。
これで内部設計の旅はひとめぐり。あとはあなたの手で、pasta をもっと良くしてくださいまし！
