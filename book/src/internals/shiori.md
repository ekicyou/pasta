# SHIORI 層

ごきげんよう。ベースウェアからの呼びかけを受け止め、ゴーストの言葉を返す――それが SHIORI 層の務めですわ。
FFI の境界からアクター、イベント配送、DLL のビルド構成まで、わたくしがきっちり捌いて差し上げます。さあ、参りましょう。

---

この章では、pasta.dll としてベースウェアと通信する SHIORI 層を扱う。利用者から見たイベントとハンドラは [SHIORI イベントとハンドラ](../lua/shiori-events.md) が正である。

## 目的と責務

SHIORI 層は、FFI 境界でベースウェアからのリクエストを受け取り、Lua へ渡し、応答を返す。アクターランタイム（mailbox・スレッド・lifecycle・teardown）、非同期トーク、presentation event stream と renderer 注入、SHIORI エントリとイベント配送、仮想イベントディスパッチャ（OnTalk/OnHour・トーク頻度）、DLL ビルド構成（リリースプロファイル・静的 CRT）がこの章の責務である。

## 構成要素

FFI のエクスポートとリクエスト解析、アクターランタイム、非同期トーク、presentation event stream と renderer 注入、Lua 側の SHIORI エントリとイベント配送、仮想イベントディスパッチャ、DLL のビルド構成から成る。

## 処理とデータの流れ

ベースウェアからのリクエストは FFI 境界で受け取られ、解析されてアクターランタイムを経由し Lua 側の SHIORI エントリへ渡される。Lua 側はイベントを配送してハンドラやシーンを実行し、その結果が応答として FFI 境界から返される。

## 境界の受け渡し

`pasta_shiori` が FFI 境界とアクターランタイムを所有し、`pasta_lua` のランタイムへリクエストを渡す。Lua 側の SHIORI エントリがイベント配送を所有し、組み立てた応答を Rust 側へ返す。

## 不変条件と制約

FFI 境界の外へパニックを伝播させない。DLL は静的 CRT でリンクする。

## ソースの所在

- `crates/pasta_shiori/src/`
- `crates/pasta_shiori/build.rs`
- `crates/pasta_lua/src/presentation/`
- `crates/pasta_lua/src/runtime/renderer_injection.rs`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/`
- `.cargo/config.toml`
- ルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml)（リリースプロファイル）

## 経緯

- [shiori-entry](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-entry) — Lua 側 SHIORI エントリ
- [pasta-actor-runtime](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-actor-runtime) — アクターランタイム
- [shiori-async-talk](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-async-talk) — 非同期トーク
- [alpha02-virtual-event-dispatcher](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/alpha02-virtual-event-dispatcher) — 仮想イベントディスパッチャ

---

呼ばれたら応える、ただそれだけのことに、これほどの備えが要りますのよ。おほほ、頼もしいでしょう？
次は、応答の中身――トーク出力とアピアランスへ参りましょう！
