# Requirements Document

## Introduction

本仕様は、VSCode から pasta（組込 LuaJIT 2.1 / mlua 0.11 vendored 静的リンク）をデバッグする「Rust ホスト型 DAP・依存最小・トランスポートを Rust が提供する」方式の **go/no-go を実装着手前に確定する検証（PoC）仕様**である。

成功とは「方式が使える」ことではなく「方式が使えるか否かの**確定した結論と根拠**が得られる」ことを指す。したがって本仕様の成果物は、再現可能な検証ハーネスと、それが出力する **段階的な GO 判定（NO-GO ／ 条件付き GO ／ GO ／ GO+）の文書**である。検証は二値の合否ではなく**段階的成功**で評価し、チャレンジ項目（R1〜R4）は成否にかかわらず全て試行して結果を残す。検証コードは使い捨て（feature-gate またはテスト専用）とし、本体ビルドを汚さない。

唯一の本丸は「**`jit.off(true,true)` ＋ `mlua::Lua::set_global_hook` が、pasta の動的生成シーンコルーチン群でラインフックを確実に撃つか**」であり、これに「フック内ブロッキング停止・再開」「フック内変数 inspect」を加えた 3 点を最小 PoC で実証する。背景・アプローチ決定・却下理由は `brief.md` および `.kiro/steering/roadmap.md`（Phase 5）を参照。

## Boundary Context

- **In scope**: (1) LuaJIT デバッグフックの発火検証（jit.off + set_global_hook・コルーチン横断）、(2) フック内ブロッキングによる停止・再開検証、(3) フック内からの変数 inspect 検証、(4) トランスポート最小往復検証（GO+ tier・必須試行）、(5) 検証の隔離・再現性、(6) 段階的 go/no-go 判定成果物の文書化。
- **Out of scope**: DAP プロトコルの本実装、.pasta ソースマップ、VSCode 拡張の製品化、デバッグバックエンドの本体への恒久統合・正式設計（いずれも後続実装仕様 `pasta-vscode-lua-debug` の領分）。
- **Adjacent expectations**: pasta_lua ランタイム（`PastaLuaRuntime` が保持する `mlua::Lua`）と mlua 0.11 の挙動に依存して検証する。後続実装仕様 `pasta-vscode-lua-debug` は本仕様の GO 判定を着手前提とする（本仕様は判定を提供するが本実装は持たない）。

## Requirements

### Requirement 1: LuaJIT デバッグフックの発火検証（go/no-go 本丸）
**Objective:** As a pasta 開発者, I want `jit.off` ＋ `set_global_hook` が LuaJIT の動的生成コルーチンでラインフックを撃つことを実証したい, so that デバッグ方式全体の go/no-go を確定できる

#### Acceptance Criteria
1. When VM 全体の JIT エンジンを無効化（グローバル `jit.off()` 無引数）し `mlua::Lua::set_global_hook` で line トリガのフックを設定した状態で Lua コードを実行したとき, the 検証ハーネス shall フックコールバックが各実行行で呼ばれたことを記録し、期待行系列と一致することを assert する（注: `jit.off(true,true)` は関数単位制御でグローバルエンジン状態を変えないため不採用。詳細は design.md「jit.off セマンティクス注」／ research.md 参照）
2. When フック設定後に `coroutine.create` で動的生成した複数のコルーチンを駆動ループで順次 resume したとき, the 検証ハーネス shall すべてのコルーチン内の Lua 行でフックが発火したことを記録・assert する
3. The 検証ハーネス shall コルーチン生成・resume シナリオを pasta の実シーン実行モデル（シーン 1 つ＝コルーチン 1 つを動的生成し駆動ループで resume する形）に忠実に再現する
4. While デバッグ対象コードが JIT コンパイル対象となりうる状態, the 検証ハーネス shall `jit.off` 適用後はフックの取りこぼし（未発火行）が発生しないことを確認する
5. If 一部のコルーチンまたは行でフックが発火しない, then the 検証ハーネス shall 不発火の条件（JIT 状態・コルーチン継承・LuaJIT バージョン等）を切り分けて記録し NO-GO の根拠として残す

### Requirement 2: フック内ブロッキング停止・再開の検証
**Objective:** As a pasta 開発者, I want フック内でブロッキング待機して実行を停止し外部指示で再開できることを実証したい, so that ブレーク／ステップの基盤が成立すると確認できる

#### Acceptance Criteria
1. When フックコールバック内でブロッキング待機（チャネル受信またはソケット read）に入ったとき, the 検証ハーネス shall Lua の実行が当該行で停止し続けることを観測・確認する
2. When 外部から再開シグナル（チャネル送信またはソケット入力）を与えたとき, the 検証ハーネス shall Lua 実行が停止地点から再開することを確認する
3. The 検証ハーネス shall フック内で `coroutine.yield` / `lua_yield` を用いず、Rust コールバックのブロッキングのみで停止を実現する
4. If フック内ブロッキング中に VM がクラッシュ・パニック・デッドロックする, then the 検証ハーネス shall 当該事象を捕捉し NO-GO の根拠として記録する

### Requirement 3: フック内からの変数 inspect 検証
**Objective:** As a pasta 開発者, I want 停止中のフックからスタック情報・ローカル変数・upvalue を取得する手段（mlua 安全 API および ffi 経路）を見極めたい, so that 変数監視の実現可否と採用方式を確定できる

#### Acceptance Criteria
1. When フックで停止中にスタックフレーム情報を要求したとき, the 検証ハーネス shall mlua の安全 API（`Debug::source` / `current_line` / `names`）で現在のソース名・行番号・関数名を取得できることを確認する
2. When フックで停止中にローカル変数および upvalue の名前と値を要求したとき, the 検証ハーネス shall mlua の `ffi` 経路（`Lua::exec_raw` で生 `lua_State` を得て `lua_getstack` で現フレーム ar を再取得し `lua_getlocal` / `lua_getupvalue` を呼ぶ）で取得可能かを検証し、基本型（number / string / boolean / table）の判別可否を記録する
3. The 検証ハーネス shall 変数取得を `std_debug` 露出に依存せず（サンドボックス維持・Requirement 5.3 整合）成立させることを第一目標とし、採用方式・unsafe の範囲・コストを記録する
4. If FFI 方式が失敗または制限される, then the 検証ハーネス shall 取得可能な範囲と制約を記録し、回避策（デバッグモード限定の `std_debug` 露出など）を方式比較として残す

### Requirement 4: トランスポート最小往復の検証（GO+ tier・必須試行）
**Objective:** As a pasta 開発者, I want 別スレッドのソケットと VM スレッドのフックをチャネルで連携した「停止→取得→再開」の往復を実証したい, so that Rust 側でトランスポートを提供する方式の end-to-end 成立を確認できる

#### Acceptance Criteria
1. When 別スレッドの `std::net::TcpListener` が接続を受理し VM スレッドのフックへチャネル経由で停止指示を渡したとき, the 検証ハーネス shall フックで停止し、変数を取得し、再開指示で続行する一連の往復を完了する
2. The 検証ハーネス shall ソケットスレッドが I/O のみを担当し、VM 操作はフック内（VM 呼び出しスレッド上）に閉じる分離（`mlua::Lua` の `!Send` 制約遵守）で動作する
3. The 検証ハーネス shall 追加クレートを用いず `std::net` のみでトランスポートを成立させる
4. If トランスポート往復が成立しないとき, then the 検証ハーネス shall ブロッカー（スレッド分離・`!Send`・ブロッキング起因）を記録し、到達段階を GO（標準）以下に留める

### Requirement 5: 検証の隔離と再現性
**Objective:** As a pasta 開発者, I want 検証コードが本体ビルドを汚さず再現可能に実行できるようにしたい, so that 本番品質を損なわずに go/no-go を確認できる

#### Acceptance Criteria
1. The 検証ハーネス shall 既定（リリース）ビルドに影響を与えない形（feature-gate またはテスト専用）で実装される
2. When 該当 feature を有効にして `cargo test` で検証を実行したとき, the 検証ハーネス shall 全検証項目の結果を判定可能な形で出力する
3. The 検証ハーネス shall Lua 側へ `debug` ライブラリ（`std_debug`）を露出させず、Rust 側 `set_hook` / `set_global_hook` のみでフックを成立させる（既定サンドボックス維持の確認）
4. Where SSP ロード実機での確認が可能な場合, the 検証ハーネス shall 実機環境での発火・停止挙動の結果を補足記録する
5. The 検証ハーネス shall 検証完了後に本体への恒久統合を残さない（使い捨て前提）

### Requirement 6: 段階的 go/no-go 判定成果物
**Objective:** As a pasta 開発者, I want 検証結果を段階的（二値でない）な go/no-go 判定として文書化したい, so that 後続実装仕様の着手可否と到達水準を確定できる

#### Acceptance Criteria
1. The 検証ハーネス shall 判定を以下の段階で表現する: **NO-GO**（R1 フック発火がいかなる文書化された方式でも成立しない）／ **条件付き GO（最低ライン）**（R1 ＋ R2 ブレーク停止・再開が成立）／ **GO（標準）**（さらに R3 変数 inspect が実現方式付きで成立）／ **GO+（高信頼）**（さらに R4 トランスポート往復が成立）
2. When 各チャレンジ項目（R1〜R4）を試行したとき, the 検証ハーネス shall 項目ごとの成否・採用方式・制約を個別に記録する（成否にかかわらず全項目を試行し結果を残す）
3. While 検証を実施する間, the 検証ハーネス shall Requirement 5 の隔離条件（feature-gate／テスト専用・`cargo test` 実行・サンドボックス維持）が成立していることを判定の妥当性前提として確認する
4. If 最低ライン（R1 ＋ R2）が成立しないとき, then the 検証ハーネス shall NO-GO 判定とブロッカーおよび回避候補（例: lua51 動的リンク化、別デバッグ方式）を文書化する
5. When 条件付き GO 以上に達したとき, the 検証ハーネス shall 到達段階と、後続実装仕様 `pasta-vscode-lua-debug` が前提とする結論（採用フック方式・変数 inspect 方式・既知制約・SSP 応答ブロッキングの取り扱い方針）を明記する
