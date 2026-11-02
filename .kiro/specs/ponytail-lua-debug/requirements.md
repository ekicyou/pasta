# Requirements Document

## Project Description (Input)
VSCode でゴーストをデバッグする作者が困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。提示モードを `.lua` に切り替えると `.pasta` のブレークポイントが黙って効かなくなり、コルーチンの最後でのステップや停止中の切断でステップの状態が抜けず、後の無関係なトークで偽の停止が起き、大きい数の変数が誤って表示され、一時停止・ウォッチ・デバッグコンソールが応答なしで固まる。加えて、デバッグが有効なときの行ごとの費用、止まったクライアントでの片付けの固まり、テストだけが使う型・道・写しが、実行と読み手の手間を増やしている。`crates/pasta_lua/src/debug` の監査の指摘を残らず直し、`.pasta` のブレークポイントが提示モードによらず効き、ステップはコルーチンを抜けるか切断すると止まらずに進み、数は正しく表示され、知らない要求には `success: false` と「unsupported」で答え、行ごとの費用が下がり、死んだコードと写しが無くなるようにする。デバッグを無効にしているときの費用ゼロ・単一の DLL・opt-in・127.0.0.1 固定の待ち受けは変えない。

## Introduction

この spec は、2026-10-19 の監査が `crates/pasta_lua/src/debug`（VSCode 用のデバッグのバックエンド）で見つけた 12 の指摘と、ギャップ分析（`research.md`）で同じ領域に増えた指摘を、残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：作者の手元で起きる困りごと（Lua 表示で `.pasta` のブレークポイントが死ぬ・ステップが抜けない・大きい数の表示・応答の無い要求・止まったクライアントでの片付けの固まり）。
- **挙動を変えない撤去と軽量化**：行ごとの費用、テストだけ・誰も使わないコード、写し、往復しても何もしない道。

要件は「その領域の監査の指摘が残らず直っている」ことと、挙動を変える直しの決めごとである。どう直すか（関数・ファイル・型の形）は設計で決める。brief.md だけで決めきれない論点は、【前提】と書いて推奨の形で要件にし、`research.md` の「要件ディスカッションへの持ち越し」に並べた。

用語：
- **提示モード**：コールスタックや停止位置を `.pasta` の座標で見せるか（`.pasta` 提示）、生成 `.lua` の座標で見せるか（`.lua` 提示）。
- **対応表**：トランスパイラが作る `.pasta` 行と生成 `.lua` 行の対応（ソースマップ）。デバッグが有効なときだけ作られる。
- **受け口**：位置でのシーンの再生・SHIORI リロードを受けるホストの送り先（キックの送り先）。`pasta_shiori` は常に渡す。

## Boundary Context

- **In scope**
  - 監査の 1〜12 番（ブレークポイントの訳・ステップの状態・数の表示・応答の無い要求・行ごとの費用・ステップ中の全スタック計算・止まったクライアントでの片付け・死んだコード・scopes の往復・提示モードの写し・transport の「無効」の道・`DebugConfig::resolve` の引数）。
  - ギャップ分析で増えた、同じ領域の指摘（`research.md` の N1〜N13。要件 1.4・4.3・4.4・4.6・7.6〜7.10・8.4）。
  - `crates/pasta_lua/src/runtime/exec.rs` の `debug_source_mode()` の説明の誤り（ギャップ分析で拾うと決めた。要件 8.4）。
  - 直しと撤去に合わせた、マニュアル `book/src/internals/debug.md` と `book/src/debug/source-level.md` の該当の行（ほかの `book/src/debug/*.md` は、書き足す必要が出た行だけ）。
- **Out of scope**
  - VSCode 拡張（`editors/vscode`）。`ponytail-editor-tooling` が持つ（`host` の廃止もそちら）。この spec は拡張が送る要求の形を変えない。
  - 単一の DLL・opt-in・127.0.0.1 固定の待ち受けの方針（確定済み）。待ち受けは 1 回の起動につき 1 接続という今の決まり（`troubleshooting.md` に書いてある）も変えない。
  - トランスパイラが作る行の対応（`ponytail-lua-transpiler` との約束で変えない）。サイドカー（`.map`）の書式（`toml-serialization` が決めた）。
  - DAP の機能の追加（一時停止・式の評価・変数の書き換え・表の中身の展開など）。対応していない要求に「unsupported」で答えるだけで、機能は足さない。
- **Adjacent expectations**
  - `pasta_shiori` は、今と同じく受け口を渡してデバッグを有効にする。この spec は `pasta_shiori` に触らない。
  - デバッグの「設定の有効」（`@pasta_debug.enabled()`・ソースマップの保持）と「待ち受けの有無」（`debug_enabled()`）の 2 つを混ぜない（`scene-play-event` の申し送り）。
  - `ponytail-editor-tooling` は `book/src/debug/*.md` の別の節を触る。後から入る側が rebase で合わせる。
  - `pasta_lua/src/lib.rs` の再輸出を消すなら、`ponytail-lua-transpiler` と同じファイルの別の行になる。後から入る側が合わせる。
  - 公開クレートの pub API は予告なく消すことがある（README・`tech.md` に書いてある）。この spec が消す pub の型・値に、非推奨の期間は設けない。

## Requirements

### Requirement 1: `.pasta` のブレークポイントを提示モードによらず効かせる

**Objective:** As a ゴーストの作者, I want 提示モードを `.lua` に切り替えても、`.pasta` 行に張ったブレークポイントで止まってほしい, so that マニュアルの約束（`source-level.md`「切替をまたいでそのまま有効」）のとおりに、見え方だけを切り替えて使える。

#### Acceptance Criteria

1. When 対応表を持っている間に、拡張子が `.pasta`（大文字と小文字を区別しない）のソースへ `setBreakpoints` 要求が届いた, the Pasta Debug Backend shall 提示モード（`.pasta` 提示・`.lua` 提示）によらず、各行を対応する生成 `.lua` の実行座標へ訳して登録し、`.pasta` 提示のときと同じ `verified` と行（対応の無い行は後ろの最も近い対応行）を応答で返す。
2. When 提示モードを `.lua` に切り替えた後に、VSCode が `.pasta` のソースのブレークポイントを送り直した, the Pasta Debug Backend shall その `.pasta` 行に対応する `.lua` の行が実行されたときに、`reason: breakpoint` で止まる。
3. When 提示モードを `.pasta` → `.lua` → `.pasta` と切り替え、そのたびにブレークポイントが送り直された, the Pasta Debug Backend shall 同じ `.pasta` 行のブレークポイントで、切り替えの前と同じ実行の位置に止まる。
4. If 対応表を持っていない間に、拡張子が `.pasta` のソースへ `setBreakpoints` 要求が届いた, then the Pasta Debug Backend shall その各行を `verified: false` で返し、止まることのない座標を「確かめ済み」と応答しない。【前提】今は `verified: true` を返して当たらない（1 番と根が同じ・`research.md` N9）。brief は対応表が無い場合に触れていないので、推奨の形で書いた（持ち越し 1）。
5. When 拡張子が `.pasta` でないソースへ `setBreakpoints` 要求が届いた, the Pasta Debug Backend shall 今と同じく、渡されたパスと行をそのまま実行座標として登録し、全行を `verified: true` で返す。
6. While 提示モードが `.lua` である, the Pasta Debug Backend shall `.pasta` のブレークポイントで止まった後の続行とステップを、今の `.lua` 提示と同じく `.lua` の粒度で行う（同じ `.pasta` 行での止まり直しの合体と `.pasta` 粒度のステップは、`.pasta` 提示のときだけ働く。今と同じ）。

### Requirement 2: ステップの状態を必ず抜ける

**Objective:** As a ゴーストの作者, I want ステップ中にシーンが終わったり、デバッグを切断したりしたら、ステップがそこで終わってほしい, so that 後の無関係なトークで偽の停止が起きず、デバッグを終えた後のゴーストが余計な判定を続けない。

#### Acceptance Criteria

1. When ステップ（`next`・`stepIn`・`stepOut`）を始めたコルーチンが、ステップが止まる前に終わった（最後まで実行した・エラーで終わった）, the Pasta Debug Backend shall そのステップを `continue` と同じ扱いに変え、以後はブレークポイントでだけ止まる。
2. When シーンのコルーチンの最後の行で `next`（F10）を押した後に、別のイベントで別のシーンが実行された, the Pasta Debug Backend shall その無関係なシーンで `reason: step` の停止を起こさない。
3. While ステップが進行中である, the Pasta Debug Backend shall ステップを始めたコルーチンとは別のコルーチンの行では、そのコルーチンが終わったコルーチンのメモリを使い回していても、ステップとして止まらない。
4. When ステップを始めたコルーチンが `yield` し、後で `resume` された, the Pasta Debug Backend shall 今と同じくそのステップを続け、`yield` をコルーチンの終わりとして扱わない。
5. When ステップで止まっている間、または停止中に、クライアントが切断した（`disconnect` 要求・接続の切断のどちらでも）, the Pasta Debug Backend shall 実行の状態を「実行中（ステップなし）」に戻して VM を進め、以後の行でステップの判定を行わない。
6. While ステップが進行中である, the Pasta Debug Backend shall 現在の行がステップを始めたコルーチンのものかを、コールスタック全体を数える前に判定し、別のコルーチンの行ではコールスタックを数えない（監査の 6 番）。
7. If ステップを始めたコルーチンが中断（`yield` したまま）のまま参照を外されて捨てられた（`co_scene` の差し替えなど）, then the Pasta Debug Backend shall そのステップを、次にブレークポイントで止まるか切断されるまで保ち（その間も別のコルーチンの行では止まらない）、この残る限界をコードの該当の所に `shortcut:`（または `ponytail:`）の形の注釈で書く（brief の Approach）。

### Requirement 3: 数を正しく表示する

**Objective:** As a ゴーストの作者, I want 変数に入った大きい数を、本当の値で見たい, so that 誤った値を見て原因を取り違えない。

#### Acceptance Criteria

1. When 変数の値が整数の値の数で、64 ビットの整数の範囲を超える（例：`1e20`・`-1e20`）, the Pasta Debug Backend shall その数を正しい十進の表記（`1e20` なら `100000000000000000000`）で表示し、`9223372036854775807` のような飽和した値を表示しない。
2. When 変数の値が 64 ビットの整数の範囲に入る整数の値の数である, the Pasta Debug Backend shall 今と同じく小数点の無い表記（`42`・`-3`・`0`）で表示する。
3. When 変数の値が整数でない数である, the Pasta Debug Backend shall 今と同じ表記（`3.5`・`0.1`）で表示する。
4. When 変数の値が無限大か非数である, the Pasta Debug Backend shall 今と同じ表記（`inf`・`-inf`・`NaN`）で表示する。

### Requirement 4: すべての DAP 要求に正しく応答する

**Objective:** As a ゴーストの作者, I want 対応していない操作をしたときに、VSCode が固まらずに「使えない」と分かってほしい, so that 一時停止・ウォッチ・デバッグコンソールを触っても、デバッグの画面が応答待ちのまま残らない。

#### Acceptance Criteria

1. When 対応していない要求（例：`pause`・`evaluate`・`setVariable`・`launch`）が届いた, the Pasta Debug Backend shall 待たせずに、その要求の `seq` と `command` に対応づけた `success: false`・`message: "unsupported"` の応答を返す（2026-10-19 開発者の決定）。
2. When 作者が VSCode で一時停止のボタンを押した、ウォッチ式を足した、またはデバッグコンソールに式を打った, the Pasta Debug Backend shall 1 の応答を返し、VM の実行と停止の状態を変えない。
3. Where 受け口が渡されていない, when `pasta/playSceneAt` か `pasta/reloadShiori` の要求が届いた, the Pasta Debug Backend shall 1 と同じ形の `success: false` の応答を返し、何も再生・リロードしない。【前提】今は応答を返さない（`research.md` N2）。本番の `pasta_shiori` は常に受け口を渡すので、作者の手元では起きにくい（持ち越し 3）。
4. When `threads` 要求が、VM が止まっていないときに届いた, the Pasta Debug Backend shall 次に止まるまで待たせずに、今と同じ固定のスレッド（ID `1`・名前 `main`）で応答する。【前提】今は次に止まるまで応答しない（`research.md` N1）。VSCode は接続の直後に `threads` を送る（持ち越し 4）。
5. The Pasta Debug Backend shall 失敗の応答（対応していない要求・受け口の無い要求・位置でのシーンの再生の失敗）を、1 つの形（`seq`・`type: "response"`・`request_seq`・`success: false`・`command`・`message`）で返す。位置でのシーンの再生の失敗の理由の文（「カーソル下にシーンがありません」など）は今と同じにする。
6. When `pasta/playSceneAt` の `uri` が、ホスト名を持つ `file://` の URI（Windows の UNC のパス。例：`file://server/share/ghost/master/dic/会話.pasta`）である, the Pasta Debug Backend shall それを `\\server\share\ghost\master\dic\会話.pasta` の `.pasta` として引き当て、ホスト名を捨てた別のパスとして扱わない。【前提】今はホスト名を捨てて `\share\…` として引き、シーンが見つからない（`research.md` N13・持ち越し 8）。
7. The Pasta Debug Backend shall 今応答している要求（`initialize`・`attach`・`configurationDone`・`setBreakpoints`・`threads`・`stackTrace`・`scopes`・`variables`・`continue`・`next`・`stepIn`・`stepOut`・`disconnect`・`pasta/sourcePresentation`・受け口のある `pasta/playSceneAt` と `pasta/reloadShiori`）の応答とイベントの中身と順序を、4 の応答の時機を除いて変えない。

### Requirement 5: デバッグが有効なときの行ごとの費用を下げる

**Objective:** As a ゴーストの作者, I want デバッグを有効にしたままでも、トークがむだに重くならないでほしい, so that ブレークポイントを張っていない間も、ふだんに近い速さで試せる。

#### Acceptance Criteria

1. While 登録されたブレークポイントが 0 件である, the Pasta Debug Backend shall 行ごとの停止の判定で、チャンク名の正規化（文字列の作り直し）をしない。
2. While ブレークポイントが登録されている, the Pasta Debug Backend shall 各ブレークポイントのチャンク名の正規化を登録のときに 1 回だけ行い、行ごとには行わない。
3. While 提示モードが `.pasta` で、対応表を持っている, the Pasta Debug Backend shall 行ごとの `.lua` から `.pasta` への引き当てで、同じチャンクのチャンク名を行ごとに作り直さない（監査の 5 番の `resolve_lua_to_pasta`）。【前提】作り直しを避ける方法（前の行の結果を覚えるなど）は設計で決める。
4. The Pasta Debug Backend shall 行ごとの判定を軽くしても、止まる位置・止まる回数・停止の理由（ブレークポイントの止まり直しの合体と `.pasta` 粒度のステップを含む）を今と同じにする。
5. While デバッグが無効である, the Pasta Debug Backend shall 今と同じく、行フック・待ち受け・スレッドを一切作らない（費用ゼロ）。

### Requirement 6: 止まったクライアントがいても片付けを終える

**Objective:** As a ゴーストの作者, I want 接続中のデバッガが固まっていても、ゴーストの終了や SHIORI の読み込み直しが固まらないでほしい, so that ベースウェアごと強制終了しなくて済む。

#### Acceptance Criteria

1. If クライアントがフレームの途中（ヘッダの途中・ヘッダの後・本体の途中）で送るのを止めたまま、接続を保っている, then the Pasta Debug Backend shall デバッグの片付け（SHIORI の unload・読み込み直し）を、クライアントが接続していないときと同じく有限の時間で終える。【前提】上限は 1 秒とする（今の接続の無いときの片付けは 5ms の待ちの数回分）。値は持ち越し 6。
2. If 1 行のヘッダが改行の無いまま上限の長さを超えた, then the Pasta Debug Backend shall その接続を閉じ、メモリを際限なく使わない。上限の値は設計で決める（本体の上限 16 MiB より小さい）。
3. When 正常なクライアントが、1 つのフレームを TCP のいくつかの断片に分けて送った, the Pasta Debug Backend shall 今と同じく 1 つのフレームとして読む。
4. When デバッグの片付けが終わった, the Pasta Debug Backend shall 今と同じく待ち受けのポートを解放しており、直後の読み込み直しで同じポートの待ち受けが立つ。

### Requirement 7: 死んだコードと写しを無くす

**Objective:** As a pasta の開発者, I want デバッグのバックエンドから、本番で使われない型・道・写しが無くなってほしい, so that 読む・直す・試す手間が減り、使われない道の誤りに惑わされない。

#### Acceptance Criteria

1. The Pasta Debug Backend shall 本番で作られない・使われない次のものを持たない：`LineEvent`、`StopReason::Entry`・`StopReason::Pause`、`DebugError::Protocol`、`SessionEvent::Error` と `report_error`、`HookHandle`・`PanicCause`・`record_panic_cause`、停止ループの `SetBreakpoints` の分岐（監査の 8 番）。
2. When `scopes` 要求が届いた, the Pasta Debug Backend shall 今と同じ応答（`Locals` のスコープ 1 つ・`variablesReference = frameId + 1`）をその場で返し、セッションへ何も送らない（往復して捨てる道を持たない。監査の 9 番）。
3. The Pasta Debug Backend shall 実効の提示モードを 1 つの所だけに持ち、セッションの中に「`enable` のときの値」と「共有の値」の 2 つの写しを持たない（監査の 10 番）。`attach` の `sourcePresentation`・実行時の切り替えが、ステップの粒度と提示に効くことは今と同じにする。
4. The Pasta Debug Backend shall 待ち受けの「無効」の道（`Transport::start(None)`・`Transport::shutdown`・`Transport::join` と、`enable` の `unreachable!`）を持たない（監査の 11 番）。
5. The `DebugConfig::resolve` shall テストでしか値の入らない引数（`attach_source_mode`）と、`file` から導ける引数（`file_source_mode`・`file_sidecar`）を取らない。解決の結果（有効・待ち受けのアドレス・提示モード・サイドカーの要否と、その優先順位 環境変数 > `pasta.toml` の `[debug]` > 既定）は今と同じにする（監査の 12 番）。
6. The Pasta Debug Backend shall `crates/pasta_lua/src/debug` の本番のコードに、`#[allow(dead_code)]`・`#[allow(unused_imports)]`（`cfg_attr` の形を含む）を残さない（監査の 8 番の `playscene.rs`・`scene_index.rs` と、`research.md` N4 の残り）。
7. The 位置からシーンを引き当てる索引 shall 働きの無い登録（`insert_file`）を持たず、シーンの無いファイルと索引に無いファイルの位置で、今と同じく「シーンが無い」を返す（`research.md` N5）。
8. The Pasta Debug Backend shall 本番で読まれない `Decoded::reload_shiori` の値を持たない（`research.md` N3）。
9. The Pasta Debug Backend shall `research.md` N6〜N8 の小物（テストだけが使う `DebugConfig::from_file`、捨てるためだけに持つ `DebugHandle` の encoder の join handle、引数が 6 つの関数に付いた要らない `too_many_arguments` の許可、`capture_stack`・`capture_variables` の使わない `lua` の引数）を持たない。【前提】`from_file` を消すとテスト（`tests/loader/config_test.rs` など）を `resolve` か `from_env` へ移す（持ち越し 5）。
10. The Pasta Debug Backend shall テストだけが使う対応表の公開の問い合わせ（`ChunkSourceMap::lua_lines_for_pasta`・`new`・`len`・`is_empty`、`MapBuilderSink::chunk_name`）と、`canonicalize_chunk_name` をそのまま呼ぶだけの `canonicalize_pasta_file` を持たない（`research.md` N11・N12）。【前提】監査の 8 番と同じ種類なので推奨の形で範囲に入れた。テストは本番の問い合わせ（`pasta_for_lua`・`resolve_pasta_to_lua`）へ移す（持ち越し 7）。
11. The Pasta Debug Backend shall 撤去の前後で、DAP の応答とイベント（要件 4 で変えるものを除く）・停止の判定・`@pasta_debug`・位置でのシーンの再生（`pasta/playSceneAt`・`OnPastaSceneAt`）の振る舞いを変えない。

### Requirement 8: マニュアルと説明を今のコードにそろえる

**Objective:** As a ゴーストの作者と pasta の開発者, I want マニュアルとコードの説明が、直した後の振る舞いのとおりであってほしい, so that 書いてあることを信じて使える。

#### Acceptance Criteria

1. When 要件 4 の直しが入った, the Pasta Manual shall `book/src/internals/debug.md` の要求ごとの扱いの表で、「その他: なし（無視する）」を `success: false`・`unsupported` の応答に直し、`threads`・`scopes`・受け口の無い `pasta/playSceneAt`・`pasta/reloadShiori` の行を新しい扱いにそろえる。
2. When 要件 2・5・6・7 の直しが入った, the Pasta Manual shall `book/src/internals/debug.md` の、消した型・道・写し（`HookHandle`・`DebugError::Protocol`・`SessionEvent::Error`・`Transport::start` の `None`・`DebugConfig::from_file`・`ThreadId` によるスレッドの比べ方・停止ループの `Scopes`・`SetBreakpoints` の行など）と、行ごとの判定・片付けの記述を、今のコードにそろえる。
3. The Pasta Manual shall `book/src/debug/source-level.md` の「`.pasta` 行に張ったブレークポイントは、切替をまたいでそのまま有効」という約束を保つ（要件 1 で本当にする）。対応表の無いときに `.pasta` のブレークポイントが `verified: false` になること（要件 1.4）は、利用者向けの頁に 1 行で書く。
4. The `PastaLuaRuntime::debug_source_mode` shall 返す値を今と同じにし、その説明を「`enable` のときに解決した初期の提示モード（環境変数 > `pasta.toml` > 既定）。`attach` の `sourcePresentation` と実行時の切り替えは映らない」に直す（今の説明は `attach` の値も映ると書いており、誤り。`research.md` N10）。
5. When この spec がマニュアルを直した, the Pasta Manual shall マニュアルの自己テスト（mdBook のビルドと、マニュアルの CI が見る検査）を通る状態に保つ。
6. The Pasta Manual shall `ponytail-editor-tooling` が持つ `book/src/debug/*.md` の節（VSCode 拡張の側の記述）を、この spec の直しのために書き換えない。

### Requirement 9: 振る舞いを固めるテストと品質の関門

**Objective:** As a pasta の開発者, I want 直しのたびに、直したことと壊していないことを機械で確かめたい, so that 安全に戻せる小さな一歩で直せる。

#### Acceptance Criteria

1. When 要件 1〜4・6 の挙動を変える直しを入れた, the Test Suite shall その直しの前の挙動で失敗し、直した後に通るテストを、直しごとに 1 つ以上持つ。
2. When 要件 5・7 の挙動を変えない直しを入れた, the Test Suite shall 直しの前に今の振る舞いを固めるテスト（特性化テスト）を持ち、直しの前後で同じく通る。
3. Where 新しいテストがデバッグの有効化（`enable`・ローダーの `load`）を通る, the Test Suite shall 環境変数 `PASTA_DEBUG=1` のもとでも固定のポート 9276 に頼らずに通る（今の `#[ctor]` の守りと同じ）。
4. The Test Suite shall x64 と x86 のどちらのビルドでも同じテストを走らせ、ポインタの幅で期待が変わる所（表のアドレスの表記など）は期待を分ける。
5. The change shall `cargo fmt --check`・`cargo clippy`（警告 0）・`cargo test -p pasta_lua` を通る。
