# Requirements Document

## Introduction

本仕様は、pasta エンジンを SHIORI スレッド束縛から解放し「自前スレッドのアクター化」する本番実装（`pasta-actor-runtime` / `pasta-scene-kick`）へ踏み込む前に、構造的未知（FFI 境界・`!Send` Lua VM・SSP reload teardown・既存コルーチン生存・実 SSP キック配信・GET レイテンシ）の **可否（GO/no-go）を実装着手前に確定する検証（PoC）仕様**である。

Phase 5 デバッガ実装後の結論として、ゴースト作者にとって本当に必要なのは「デバッグ位置でのブレーク」ではなく「**任意シーンの再生を今すぐキック**」することであった。それを実現するにはエンジンを宿主非依存のアクターとして独立スレッド化し、`wintf_winmsg_executor`（公開済みの `winmsg-executor` フォーク）上で `!Send` Lua VM をホストする必要がある。本仕様の唯一の目的は、その方式の「一番怖い」未知点を最小の使い捨て PoC で潰し、後続実装仕様の着手可否を確定することである。

成功とは「方式が使える」ことではなく「方式が使えるか否かの**確定した結論と根拠**が得られる」ことを指す。本仕様の成果物は、再現可能な検証ハーネスと、それが出力する **段階的な GO 判定（NO-GO ／ 条件付き GO ／ GO ／ GO+）の文書**である。検証は二値の合否ではなく**段階的成功**で評価し、6 つのチャレンジ項目は成否にかかわらず全て試行して結果と根拠を残す。検証コードは使い捨て（feature-gate）とし、本体のリリースビルドをバイト不変に保つ。背景・アプローチ決定・却下理由は `brief.md` および `.kiro/steering/roadmap.md`（Phase 7）を参照。

## Boundary Context

- **In scope**: 以下 6 項目の GO/no-go 実証と、それを支える検証ハーネス・隔離・段階判定の文書化。
  1. executor 上での `!Send` Lua VM ホスティングと SSP reload 時の clean teardown
  2. SSP スレッド → executor スレッドへの block-on-reply marshaling（GET 同期契約／NOTIFY 即 204 fire-and-forget）
  3. drop→204 ガードによる「応答未送信のまま drop」デッドロック経路の原理的消滅
  4. ホスト tick 駆動 → executor 駆動への移行後の既存 coroutine/callback（`STORE.co_scene` / `CALLBACK`）生存
  5. talk FIFO ＋ OnSecondChange drain ＋ `Status: talking` gate ＋即時 preempt による実 SSP での ≤1 秒キック配信
  6. GET block-on-reply のレイテンシ実測と、GET タイムアウト→204 フォールバックの要否判断
- **Out of scope**: 本番化（`pasta-actor-runtime`）、キック機能の作り込み（`pasta-scene-kick`）、presentation event stream 契約の確定（PoC では最小限の仮契約で可）、UI、挙動保存（バイト不変）の網羅検証、`pasta_novel` アダプタ・`*.pasta` ウィンドウ・SSTP 出力。
- **Adjacent expectations**: 検証は `pasta_shiori`（Windows DLL）／`pasta_lua` ランタイム（`PastaLuaRuntime` が保持する `mlua` の `!Send` VM）／既存 yield/resume 基盤（`STORE.co_scene`・`resume_until_valid`・`CALLBACK`）／`wintf_winmsg_executor` の挙動に依存する。後続実装仕様 `pasta-actor-runtime` は本仕様の GO 判定を着手前提とする（本仕様は判定を提供するが本実装は持たない）。隣接仕様 `pasta-vscode-lua-debug` / `debug-transport-hardening` とは `pasta_lua/src/debug/` のスレッド／チャネル前例とソケット/reload 再バインドの知見を共有する。

## Requirements

### Requirement 1: executor 上での `!Send` Lua VM ホスティングと reload teardown（go/no-go 本丸）
**Objective:** As a pasta 開発者, I want `wintf_winmsg_executor` が `!Send` Lua VM を素直にホストし SSP reload で clean teardown できることを実証したい, so that エンジンの SHIORI スレッド束縛解放（アクター化）方式全体の go/no-go を確定できる

#### Acceptance Criteria
1. When 検証ハーネスが executor スレッド上で `!Send` の `mlua` VM を生成し当該スレッドに pin（保持）したまま Lua コードを実行したとき, the 検証ハーネス shall VM がスレッド境界を越えず executor スレッド内で実行を完了したことを記録・assert する
2. When SSP reload に相当する unload→再ロードのライフサイクルを実行したとき, the 検証ハーネス shall executor とそれが保持する VM・関連リソース（メッセージ専用ウィンドウ・スレッド・チャネル）が漏れなく解放（clean teardown）されたことを確認する
3. While reload を反復実行する間, the 検証ハーネス shall ソケット/ポート枯渇・リソースリーク・ハンドル枯渇など過去に観測された teardown 系不具合が再発しないことを確認する（待受を用いる場合はエフェメラルまたは再利用可能ポートで成立させる）
4. If executor 上での VM ホスティングまたは teardown が成立しない（VM の `!Send` 制約違反・リーク・reload 後のクラッシュ等）, then the 検証ハーネス shall ブロッカー条件を切り分けて記録し NO-GO の根拠として残す

### Requirement 2: block-on-reply marshaling（GET 同期契約／NOTIFY fire-and-forget）の検証
**Objective:** As a pasta 開発者, I want SSP スレッドから executor スレッドへの block-on-reply marshaling が SHIORI/3.0 の同期契約を守ることを実証したい, so that 宿主リクエストとアクター実行の橋渡しが end-to-end で成立すると確認できる

#### Acceptance Criteria
1. When SSP スレッドが GET リクエストを発行し executor スレッドへ marshaling したとき, the 検証ハーネス shall SSP スレッドが executor からの応答値を受け取るまでブロックし、その応答値を GET の戻り値として返すことを確認する
2. When SSP スレッドが NOTIFY リクエストを発行したとき, the 検証ハーネス shall executor の処理完了を待たず即座に 204（成功・返り値なし）を返す fire-and-forget 動作を確認する
3. The 検証ハーネス shall I/O（SSP リクエスト受理）を担うスレッドと VM 操作を担う executor スレッドを分離し、VM 操作が executor スレッド上に閉じる（`mlua` の `!Send` 制約遵守）形で marshaling を成立させる
4. If GET の block-on-reply 往復が成立しない（応答が返らない・スレッド分離違反・デッドロック）, then the 検証ハーネス shall ブロッカーを記録し NO-GO の根拠として残す

### Requirement 3: drop→204 ガードによるデッドロック経路の消滅検証
**Objective:** As a pasta 開発者, I want 応答未送信のまま応答チャネルが drop された場合に自動的に 204 が返る保護を実証したい, so that panic や応答忘れに起因するデッドロック経路が原理的に消えると確認できる

#### Acceptance Criteria
1. When GET 処理中に executor 側が応答を送信しないまま応答経路（応答チャネル／responder）を drop したとき, the 検証ハーネス shall SSP スレッドが無限待機に陥らず 204 フォールバック応答を受け取ることを確認する
2. When executor 側のタスクが panic して応答経路が巻き戻り（drop）されたとき, the 検証ハーネス shall 当該 GET が 204 として終結し SSP スレッドが解放されることを確認する
3. The 検証ハーネス shall drop→204 ガードにより「応答未送信のまま drop（panic／忘れ）」のデッドロック経路が原理的に発生しないことを、再現シナリオの試行を通じて実証する
4. If drop→204 ガードが機能せず SSP スレッドがブロックし続ける経路が残る, then the 検証ハーネス shall 当該経路と条件を記録し NO-GO の根拠として残す

### Requirement 4: ホスト tick 駆動から executor 駆動への移行後の coroutine/callback 生存検証
**Objective:** As a pasta 開発者, I want 駆動主体をホスト tick から executor へ移しても既存の継続実行基盤が生存することを実証したい, so that アクター化が既存トーク継続・非同期 callback を壊さないと確認できる

#### Acceptance Criteria
1. When 駆動主体を従来のホスト tick（SHIORI リクエスト周期）から executor 駆動へ置き換えてシーンコルーチン（`STORE.co_scene` 相当）を resume したとき, the 検証ハーネス shall コルーチンが中断地点から正しく継続実行されることを確認する
2. When executor 駆動下で非同期 callback（`CALLBACK.pending` / `get_property` 相当）を発行し後続の駆動契機で resume したとき, the 検証ハーネス shall callback が解決されコルーチンが継続することを確認する
3. The 検証ハーネス shall コルーチン生成・yield・resume シナリオを pasta の実シーン実行モデル（シーン継続と callback 待機を含む形）に忠実に再現する
4. If executor 駆動への移行後にコルーチンまたは callback の継続が失われる（resume 不能・状態喪失・継続契機の消失）, then the 検証ハーネス shall 喪失条件を切り分けて記録し NO-GO の根拠として残す

### Requirement 5: 実 SSP での ≤1 秒キック配信検証（talk FIFO ＋ gate ＋ preempt）
**Objective:** As a pasta 開発者, I want talk FIFO・OnSecondChange drain・`Status: talking` gate・即時 preempt によって任意シーンのキックが実 SSP に ≤1 秒で配信されることを実証したい, so that 「任意シーンの再生を今すぐキック」という本来要件が方式上成立すると確認できる

#### Acceptance Criteria
1. When 検証ハーネスが executor 経由で 1 シーンのキックを talk FIFO に投入したとき, the 検証ハーネス shall OnSecondChange の drain 契機で FIFO が排出されシーンが SSP で再生されることを確認する
2. While SSP が `Status: talking`（再生中）である間, the 検証ハーネス shall talk FIFO の drain を gate（抑止）し、再生中トークへの割り込み配信を防ぐことを確認する
3. When 即時 preempt（先行トークを閉じて新規キックを優先）を指示したとき, the 検証ハーネス shall キックされたシーンが先行トークより優先して配信されることを確認する
4. When 実 SSP 相当の呼び出しパターンでキックを発行したとき, the 検証ハーネス shall キック指示から配信までの所要時間を実測し ≤1 秒で配信されることを確認・記録する
5. If ≤1 秒キック配信が成立しない（drain 不発・gate 誤動作・preempt 不能・配信遅延 >1 秒）, then the 検証ハーネス shall 未達条件と実測値を記録し NO-GO もしくは制約付き判定の根拠として残す
6. The 検証ハーネス shall 本仕様における「実 SSP 相当」を、OnSecondChange の周期と `Status: talking` 遷移を忠実に再現する自前ドライバ（忠実シミュレータ）と定義する（実機 SSP への attach 計測は任意とし、実機での絶対性能保証は後続実装仕様 `pasta-actor-runtime` へ申し送る）

### Requirement 6: GET block-on-reply レイテンシ実測とフォールバック要否判断
**Objective:** As a pasta 開発者, I want GET block-on-reply の実レイテンシを実機で実測したい, so that GET タイムアウト→204 フォールバックの要否を根拠付きで判断できる

#### Acceptance Criteria
1. When 実 SSP 相当の呼び出しパターン（R5.6 で定義する忠実シミュレータ。実機 attach は任意）で GET block-on-reply を反復実行したとき, the 検証ハーネス shall 各 GET の応答レイテンシを実測し代表値（最大・分布等）を記録する
2. When レイテンシ実測値が得られたとき, the 検証ハーネス shall GET タイムアウト→204 フォールバックが必要か否かの判断と、必要な場合の閾値候補を文書化する
3. If GET レイテンシが宿主の許容応答時間を超過しうる経路が観測される, then the 検証ハーネス shall 当該経路・条件・推奨フォールバック方針を記録し後続実装仕様への申し送りとして残す

### Requirement 7: 検証の隔離・再現性とリリースビルドのバイト不変
**Objective:** As a pasta 開発者, I want 検証コードが本体のリリースビルドを汚さず再現可能に実行できるようにしたい, so that 本番品質（バイト不変）を損なわずに go/no-go を確認できる

#### Acceptance Criteria
1. The 検証ハーネス shall 既定（リリース）ビルドに影響を与えない使い捨ての形（default 無効の feature-gate）で実装される
2. While 検証 feature が無効である間, the 検証ハーネス shall 本体のリリースビルド成果物が検証コード導入前とバイト不変であることを担保する
3. When 該当 feature を有効にして検証を実行したとき, the 検証ハーネス shall 全検証項目（Requirement 1〜6）の結果を判定可能な形で出力する
4. The 検証ハーネス shall 既存テストスイートを汚染しないこと（固定ポート枯渇等を避けるためエフェメラル／再利用可能ポートを用い、`PASTA_DEBUG` 系の環境依存でテストが破壊されないガードを踏襲する）を満たす
5. The 検証ハーネス shall 検証完了後に本体への恒久統合を残さない（使い捨て前提・後続本番移行完了時に除去）

### Requirement 8: 段階的 go/no-go 判定成果物
**Objective:** As a pasta 開発者, I want 検証結果を段階的（二値でない）な go/no-go 判定として文書化したい, so that 後続実装仕様 `pasta-actor-runtime` の着手可否と到達水準を確定できる

#### Acceptance Criteria
1. The 検証ハーネス shall 判定を段階で表現する: **NO-GO**（Requirement 1 の executor 上 VM ホスティング・reload teardown がいかなる文書化された方式でも成立しない）／ **条件付き GO（最低ライン）**（Requirement 1 ＋ Requirement 2 ＋ Requirement 3 が成立し、安全な marshaling 基盤が成り立つ）／ **GO（標準）**（さらに Requirement 4 の coroutine/callback 生存が成立）／ **GO+（高信頼）**（さらに Requirement 5 の ≤1 秒キック配信と Requirement 6 のレイテンシ実測が成立）
2. When 各チャレンジ項目（Requirement 1〜6）を試行したとき, the 検証ハーネス shall 項目ごとの成否・採用方式・制約を個別に記録する（成否にかかわらず全項目を試行し結果を残す）
3. While 検証を実施する間, the 検証ハーネス shall Requirement 7 の隔離条件（default 無効 feature-gate・バイト不変・テスト非汚染・使い捨て）が成立していることを判定の妥当性前提として確認する
4. If 最低ライン（Requirement 1 ＋ 2 ＋ 3）が成立しないとき, then the 検証ハーネス shall NO-GO 判定とブロッカーおよび回避候補を文書化する
5. When 条件付き GO 以上に達したとき, the 検証ハーネス shall 到達段階と、後続実装仕様 `pasta-actor-runtime` が前提とする結論（採用 executor 統合方式・VM pin／teardown 方針・marshaling 契約・drop→204 ガード方針・coroutine 生存条件・GET レイテンシとフォールバック要否）を明記する
