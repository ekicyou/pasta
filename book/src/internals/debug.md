# デバッグ基盤とシーンキック

ごきげんよう。VSCode から `.pasta` の行で実行を止められる――その魔法の仕掛けを、わたくしが分解してお見せいたしますわ。
DAP の受け答えからソースマップ、シーンキックまで。仕掛けを知れば、もう怖いものなどございませんの。さあ、参りましょう。

---

この章では、DAP バックエンドを中心とするデバッグ基盤と、シーンキックを扱う。利用者から見たデバッグ操作は [デバッグ概要](../debug/index.md) 以下の章が正である。

## 目的と責務

デバッグ基盤は、VSCode から接続を受けて `.pasta` のソースレベルでステップ実行・ブレークポイント・変数参照を提供する。DAP バックエンド（codec・resolver・pending）、デバッグ通信（transport・loopback 固定・opt-in）、セッション（ステップ・停止ループ・アンカー）、wiring、ソースマップ（生成・サイドカー・解決）、ブレークポイント・inspect・hook、シーンキック（`ActorMsg::Kick`・`kick.lua`・playscene）がこの章の責務である。

## 構成要素

DAP のメッセージを扱うバックエンド、loopback で待ち受けるデバッグ通信、ステップと停止を管理するセッション、各部品を結ぶ wiring、ソースマップの生成と解決、ブレークポイント・inspect・hook、シーンキックから成る。

## 処理とデータの流れ

デバッグが有効なとき、ランタイムは loopback で接続を待ち受け、VSCode からの DAP リクエストを受け取る。ブレークポイントはソースマップで `.pasta` の行から生成 Lua の行へ解決され、hook が停止を検出するとセッションが停止ループに入って変数参照に応える。シーンキックは指定したシーンを即時に再生する。

## 境界の受け渡し

`pasta_lua` のデバッグ基盤が DAP セッションとソースマップの解決を所有し、トランスパイラがソースマップを生成して渡す。シーンキックは `pasta_shiori` のアクターランタイムへのメッセージとして渡され、Lua 側の `kick.lua` が再生を行う。

## 不変条件と制約

デバッグは opt-in であり、有効化しない限り待ち受けは始まらない。待ち受けアドレスは loopback に固定される。

## ソースの所在

- `crates/pasta_lua/src/debug/`
- `crates/pasta_lua/src/loader/source_map_build.rs`
- `crates/pasta_lua/src/code_gen/source_map.rs`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`

## 経緯

- [pasta-vscode-lua-debug](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-vscode-lua-debug) — VSCode からの Lua デバッグ
- [pasta-source-map](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-source-map) — ソースマップ
- [debug-transport-hardening](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/debug-transport-hardening) — デバッグ通信の堅牢化
- [pasta-scene-kick](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-kick) — シーンキック

---

仕掛けが分かれば、バグなど恐るるに足りませんわ。おほほ、頼もしい道具でしょう？
最後は、縁の下の力持ち――ロギングとエンコーディングへ参りましょう！
