# Brief: novel-backlog-rollback

2026-11-20 の棚卸（`/kiro-next`・main `58aba27c`）で、`novel-checkpoint-rollback` から「バックログの行へ戻る」の分を切り出して起票した（経緯は `novel-checkpoint-rollback/brief.md` の「2026-11-20 棚卸の分割」）。ノベルゲーム（Phase 13）の道筋の 1 本で、フェーズ 2（SSP の対応を待つ。areka でも動く）に入る。

## Problem

市販品の体裁のノベルゲームでは、遊ぶ人がバックログ（読んだ行の一覧）から前の行を選ぶと、その行まで物語が巻き戻る。バックログの一覧は、ベースウェアの機能である。pasta の側には、行ごとの目印を出す手段も、選ばれた目印を受けて戻る口も無い。

戻る仕組みそのもの（1 本ごとの控え・早回し・2 本返す）は、`novel-checkpoint-rollback` が作る。ロードに要るのは、いちばん新しい控え 1 つである。バックログの行へ戻るには、控えの列と、行ごとの目印と、巻き戻しのイベントの受け口が要る。この分は、目印の命令が `play` の命令（正典に無い）なので、今の SSP では読み飛ばされ、実機で確かめられない（`novel-checkpoint-rollback` の brief の「2026-11-20 棚卸の再測定」）。

## Current State

- 目印も、控えの列も、巻き戻しのイベントの受け口も無い。
- `novel-checkpoint-rollback` が、1 本ごとの控え・今のシーンを頭から早回しして位置を合わせる仕組み・2 本返す形（1 本目は出し直しの印 `\![play,again]` 付きで画面の状態を全部打つ・2 本目は選ばれた行をト書きごとふつうに再生する）を作る（未完了）。
- `novel-talk-flow` が、1 本の枠（頭・末尾・待っている物・ID）と、今のシーン・区切りの通し番号・待っている 1 本の ID をゴーストに 1 つ持つ（未完了）。目印を置く所は、その枠の頭の 1 か所と決まっている。
- イベントの受け口の読み込みは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`（352 行。読み込みの並びは 85〜92 行）。
- 開発用のシーン再生（`OnPastaScene`。名前で指名したシーンを、ローカル変数を入れてから頭から再生する。`crates/pasta_lua/pasta_scripts/pasta/shiori/event/scene_play.lua`）が、戻る形に近い。受け口を使い回せるかは、確かめていない。

## Desired Outcome

- 演劇の、待つ 1 本には、どれも目印が付く（字が無くても）。目印は、イベントの名前と ID を運ぶ（仮 `\![play,mark,OnPlayRollback,12]`）。

```
\![play,mark,OnPlayRollback,41]\p[0]おはよう。今日は早いのね。\x\![raise,OnPlayNext]
```

（記法とイベント名は仮。`novel-areka-contract` の要件定義で決まる。）

- 遊ぶ人がバックログの行を選ぶと、ベースウェアが目印のイベントを ID 付きで送り、pasta は、その控えへ戻る。それより後の控えは捨てる。
- 控えは、新しい方から決まった数だけ持つ。数は `pasta.toml` で変えられる。
- 控えの無い古い行を選ばれた時と、控えを全部捨てた後（ロードなど）の扱いが決まっている。

## Approach

決定（2026-11-19 の discovery「ノベルゲームの深掘り」と、同じ日の暫定設計の議論）。`novel-checkpoint-rollback` の brief の「決定（2026-11-19 の discovery『ノベルゲームの深掘り』。最終形）」の「バックログと目印」「控えと、戻り方」「設定のモード」「ベースウェアへ頼むこと」と、「決定（2026-11-19 の暫定設計の議論）」から、この spec の分を写した。どれも開発者の決定なので、要件定義で変えない。骨子は `novel-areka-contract/final-model-2026-11-19.md` の 2.5。名前は、どれも仮である。

- **バックログの一覧は、ベースウェアの機能である**（開発者の決定）。pasta.dll に依らない。どの SHIORI のゴーストでも、さくらスクリプトを再生すれば行が貯まる。目印が無くても動く。取り決めの名前に `pasta` を入れない。
- **目印は、イベントの名前と ID を運ぶ**（`\![play,mark,OnPlayRollback,12]`）。正典の選択肢 `\q[表示,OnXxx,引数]` と同じ読み方で、取り決めに固定のイベント名を作らない。行が選ばれると、ベースウェアは、そのイベントを GET で送り、Reference0 に ID を載せる。
- **演劇のどの 1 本にも、目印を 1 つ付ける**（開発者: 「アクション行単位にしましょう。」）。イベントで選ばれたシーンの行も、同じである。目印は、待つ 1 本には、どれも付ける（字が無くても）。待たない残りには、付けない（骨子 8 章の議題 14 の 7）。
- ID は、付けるたびに増える番号で、中身は pasta だけが読む。同じ行を 2 度通ったら、別の ID になる。英数字だけにする（areka の頼み）。
- **戻り方**: 選ばれた目印より後の控えを捨てる。そこから先は、`novel-checkpoint-rollback` の戻り方（早回し・控えを置く・2 本返す）を使う。2 本目の目印の ID は、選ばれた行と同じにする。バックログの行は、この 2 本目で貯め直る。
- **控えの数**: 新しい方から決まった数だけ持つ（仮 100。`pasta.toml` で変えられる。数え方は目印の数）。それより古い行を選んでも、何も起きない。
- **控えを全部捨てる時**: `スロット読込`・`スロット消去`・前の物語が終わって新しく演劇を始める時（トークから演劇のモードに入った時、が候補。要件定義で決める）。ロードの直後は、ロード前の行へ戻れない。
- **設定のモード**（骨子 4 章の C）: 目印を付けて控えを作るのは、柱 `＆再生モード` が演劇の行だけである。設定のモードの行には、目印を付けず、控えを作らず、バックログに貯めない。
- 目印を置く所は、`novel-talk-flow` の 1 本の枠の頭の 1 か所（2026-11-19 の暫定設計の照らし合わせ）。再生位置と ID は、自分で数えない。`novel-talk-flow` が持つ物を読む。先に入る spec は、後に入る spec の関数を呼ばない。先に入る spec が口を置き、後の spec が登録する。
- **ベースウェアへ頼むこと**（`novel-areka-contract` の取り決めに書く。areka のセッションの見立て）: 目印の無い行は、選べない（選んでも何も起きない）。行を選ばれたら、ベースウェアは、その行を含めて後ろを捨てる。選ばれた行は、2 本目で貯め直る（開発者の承認。今のまま）。字の無い 1 本は、バックログに行を作らない。
- 2026-11-20 の areka の決定は、この作りを変えない。出し直しの印を 1 本の先頭に書く・バックログを空にする命令は、取り決めで決める（`novel-checkpoint-rollback` の brief の「2026-11-20 棚卸の再測定」）。

## Scope

- **In**: 目印の出力（`novel-talk-flow` の枠の頭の口に登録する。演劇の待つ 1 本だけ）。控えの列（新しい方から決まった数）と、数の設定（`pasta.toml` の読み込みと `pasta_check validate` の検査）。巻き戻しのイベントの受け口。選ばれた目印より後の控えを捨てること。控えの無い行を選ばれた時の扱い。控えを全部捨てた後に、バックログを空にする合図を出すか（取り決めで決まれば）。テスト、マニュアル、スキル references の再生成。
- **Out**: 1 本ごとの控え・早回し・2 本返す・スロット変数の口・控えを全部捨てる時の判定・着せ替えの 0（`novel-checkpoint-rollback`）。1 本の枠・再生位置・ID の系列（`novel-talk-flow`）。セーブとロード（`novel-save-slots`）。バックログの一覧と画面（ベースウェア）。目印と巻き戻しのイベントの名前（`novel-areka-contract`）。

## Boundary Candidates

- 目印（`novel-talk-flow` の枠の頭の口へ登録）。
- 控えの列（`novel-checkpoint-rollback` の控えの倉に、列と数を足す）。
- 巻き戻しの受け口（新しいファイルと `init.lua` の読み込みの 1 行）。
- 設定（`loader/config/` と `pasta_check` の `validate/config.rs`）。
- マニュアル。

## Out of Boundary

- 画面と音の控えと再現（ベースウェア）。
- 場面を走り直さずに、さくらスクリプトを切り出す方式（2026-11-19 に取り下げた）。

## Upstream / Downstream

- **Upstream**: `novel-checkpoint-rollback`（控えと、戻る仕組み）、`novel-areka-contract`（目印・巻き戻しのイベント・出し直しの印・バックログを空にする合図の名前）。`novel-talk-flow`（枠の頭の口・再生位置・ID。`novel-checkpoint-rollback` を通して要る）。
- **Downstream**: `hello-novel-sample`、`manual-novel-guide`（バックログの行へ戻る節は、ベースウェアの対応の後）。

## Existing Spec Touchpoints

- **Extends**: `novel-checkpoint-rollback`（切り出し元。控えの倉に列を足す）。
- **Adjacent**: `novel-talk-flow`（`init.lua`・枠）、`novel-save-slots`（控えを全部捨てる時を共有する）、`shiori4-v4-host`・`shiori4-v4-lua-api`（`init.lua`）、`.kiro/specs/completed/scene-play-event/`（開発用のシーン再生の受け口）。

## Constraints

- 確かめるには、ベースウェア（SSP か areka）が、目印の命令とバックログの行の選択に対応している必要がある。対応の前は、机の上の試験（Lua の単体テスト）でしか確かめられない。
- 名前は、取り決めの要件定義で決まるまで仮である。
- 演劇のモードでない時、出力と保存の挙動を変えない。
- `init.lua` は `novel-talk-flow` と共有する。`novel-checkpoint-rollback` の後に置く。

## 測定（2026-11-20）

- **触るファイル**: 目印（`novel-talk-flow` の枠の頭の口へ登録する所。暫定設計の `crates/pasta_lua/pasta_scripts/pasta/play.lua` の見込み）、控えの列（`novel-checkpoint-rollback` が作る控えの Lua のファイル）、巻き戻しの受け口（新しいファイルと `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` の読み込みの 1 行）、控えの数の設定（`crates/pasta_lua/src/loader/config/`・`crates/pasta_check/src/validate/config.rs`（455 行）・`report.rs`・`crates/pasta_check/tests/validate_parity_test.rs`）、マニュアル（`book/src/reference/pasta-toml.md`・`book/src/lua/shiori-events.md`）、スキル references。テストは新しいファイルに置く。
- **規模**: 4〜6（目印 1、控えの列と数の設定 1〜2、巻き戻しの受け口と後を捨てる 1、戻れない行とバックログを空にする合図 1、マニュアルとテスト 1）。
- **先に要るもの**: `novel-checkpoint-rollback`（働きの上で要る）、`novel-areka-contract`（名前）。`novel-talk-flow` は、`novel-checkpoint-rollback` を通して要る。ファイルの重なり（未完了）: `novel-talk-flow`・`shiori4-v4-host`・`shiori4-v4-lua-api`（`init.lua`）、`novel-save-slots`（`loader/config/`・`validate/config.rs` を触るなら）。
- **フェーズ**: 2（SSP の対応を待つ。areka でも動く。今の SSP では、目印の `play` の命令が読み飛ばされ、実機で確かめられない）。
- **種別**: 機能（バックログの行へ戻る仕組みが無い）。方式は開発者の案（2026-10-10・2026-11-19）。
- **要件定義のモデル**: Opus（方式は決まった。残るのは名前と端の扱いで、取り決めの要件定義に合わせる）。
- **要件定義の議題**（6 つ）:
  1. 目印と巻き戻しのイベントの名前と引数（取り決めの要件定義の結果を写す。決まっていなければ、待つか仮のまま進めるか）。
  2. 控えの数の設定の置き場所と名前（`pasta.toml` のどの表か）と、誤りの扱い（`pasta_check validate` で誤りか注意か）。
  3. 戻れない行（古くて控えの無い行）を選ばれた時。ベースウェアは後ろを捨てた後で、pasta は何も返さないので、バックログだけが欠ける。areka の勧めは、捨てる時を「選ばれた時」から「同じ目印（イベント名と ID）の行を貯め直す時」へ移すこと（骨子の決定にはしていない。取り決めの要件定義で決める。開発者: 議題 E は「取り下げる」）。
  4. 控えを全部捨てた後（`スロット読込` など）、ベースウェアのバックログに古い ID の行が残る。areka の勧めは、ロードの 1 本目に「バックログを空にする」合図を足すこと（正典に無い）。足さないなら、ID をロードをまたいで重ねないこと。
  5. 目印ごとに変数とアクター状態を覚える量（メモリ）。差分で持つか。
  6. 巻き戻しの受け口を、開発用のシーン再生（`scene_play.lua`）と使い回せるか。遅れて届いた・知らない ID のイベントの扱い。
