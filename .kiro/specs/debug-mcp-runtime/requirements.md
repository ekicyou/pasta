# Requirements Document

## Project Description (Input)
pasta の開発の道具（VSCode の「▶ シーンを実行」・SHIORI のリロード）は、pasta.dll が 9276 番で待ち受ける DAP に頼っている。DAP は不安定で（閉じた `ponytail-lua-debug` の監査だけで Must fix が 5 つ）、有効にすると行フックと `jit.off()` で JIT が全体で止まり、ポートが固定で 2 体を同時に起こすと衝突した経緯もある。実際の開発はもう SSP・areka の MCP で回っていて、ghost_dev「ぱすた開発」は DAP を一度も使っていない。pasta.dll から DAP の待ち受け・セッション・行フック・`jit.off()`・キックの経路を消し、`[debug] enabled`（と `PASTA_DEBUG=1`）を「開発用のイベントを許す」旗にする。有効なときだけソースマップとシーンの索引を作り、開発用のイベント（`OnPastaScene`・`OnPastaSceneAt`・新設の `OnPastaDumpVariables`）を許し、実行時のエラーに `.pasta` の位置を添える。マニュアルの internals とリファレンスの頁を新しい形に揃える。

## Introduction

この spec は、pasta.dll の開発の道具の通り道を、DAP から開発用のイベントへ替える。中身は 2 つある。

- **撤去**：DAP の待ち受け・セッション・行フック・`jit.off()`・ブレークポイント・変数の表示・提示モード・サイドカーと、キックの経路（Rust・Lua・テスト）。
- **残す部品の整え**：デバッグの門（`[debug] enabled` と `PASTA_DEBUG`）、シーンマップ（シーンの宣言の記録と、位置からシーンを引く索引）、開発用のイベント（`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`）、実行時のエラーの `.pasta` の位置。

要件は「消したものが残っていない」ことと、残すものの決めごとである。どう直すか（関数・ファイル・型の形）は設計で決める。この spec は撤去が主なので、受け入れの基準に Rust・Lua の項目の名前やファイルの名前が出るのは意図したものである（直し方は書かない）。

brief の項目と要件の対応は次のとおり。

| brief の項目 | 今の状態（2026-11-03・main 96b75cbf） | 要件 |
| ------------ | ------------------------------------- | ---- |
| Approach 1 撤去（pasta_lua の DAP） | `crates/pasta_lua/src/debug` は 79 ファイル・22,423 行。残すのは `source_map/`・`playscene.rs`・`config.rs` | 1・7・9 |
| Approach 2 撤去（キック） | Rust は `ActorMsg::Kick`・アクターの Kick の腕・`PastaShiori::kick`・`kick_sink_for`、Lua は `kick.lua`・`SHIORI.kick`・`STORE.kick_*`・`virtual_dispatcher.lua` 246〜272 行 | 2・9 |
| Approach 3 門を縮める | `DebugConfig` は 4 つの欄、`DebugConfig::resolve` は引数 8 つ、env は 4 つ | 3・7 |
| Approach 4 ソースマップの新しい使い手 | **止まるエラーの位置はもう出ている**（下の「確かめたこと」）。行の対応で引けない行だけが残る。設計ディスカッション（2026-11-03）で、その行をソースマップで引く要件（もとの 6.3）を落とし、ソースマップをシーンマップに縮めた（引ける行が少なく、そのためだけに行の対応表を持ち続ける割に合わない） | 6 |
| Approach 5 `OnPastaDumpVariables` | 無い。セーブの変換（`persistence.rs` の `lua_table_to_toml_document`）はある | 5 |
| Approach 6 マニュアル | `internals/debug.md`（510 行）ほか | 8 |
| 開発用のイベントの門 | `scene_play.lua` の `allowed()` が設定の値を見る（待ち受けの有無は見ない） | 4 |
| 受け継ぐもの（`ponytail-lua-debug`・`ponytail-lua-transpiler`・`scene-play-event` から） | 残る | 7 |
| Constraints（x86・中和・引数と応答・`[debug]` を書いた既存のゴースト・章の数） | — | 3・4・8・9 |

**確かめたこと（brief の 1 つ目の穴）**：止まるエラーのバルーンとログの警告は、もう `.pasta` の位置を出している。`shiori/stop.lua` の `STOP.detail`（127 行）・`STOP.location`（110 行）が、`act:init_scene` の最後の引数の行の対応（`transpiler.rs` の `fill_line_maps` 259 行。デバッグの有無にかかわらず生成コードに埋まる）で、変換した Lua の行を `dic\天気.pasta:15` に直す。行の対応に入るのは、そのシーン関数の本体と、同じグローバルシーンの Lua のブロック（`scope_gen.rs` 183〜187 行）である。アクター辞書の単語定義（`ACTOR:create_word(…)`）は読み込みのときに走るデータの行で、実行時のエラーはそこから出ない。行の対応で引けないのは次の 2 つである。

- 別のグローバルシーン・別のファイルの Lua のブロックの関数（`GLOBAL.名前 = function` など）の中で止まったとき。今は場所を出さない（`reference/runtime-failures.md` 193 行のとおり）。
- ただし別のファイルのときは、`pasta_location`（`stop.lua` 90〜99 行）が止まったチャンクを見ずに行の番号だけで引くので、番号が偶然重なると**実行中のシーンのファイルの違う行を出す**。マニュアルの「場所を出さない」と食い違う誤りである。

そこで要件ディスカッションでは Approach 4 を「行の対応で引けない行を、デバッグが有効なときだけソースマップで引く」に絞ったが、設計ディスカッション（2026-11-03）でこれも落とした。引けない行は、デバッグの有無によらず今のとおり場所を出さない。上の誤りはデバッグの有無によらずこの spec で直す（要件 6。議題 1・2）。

> brief.md だけでは決めきれなかった論点は、要件ディスカッション（2026-11-03）で全部を推奨案で確定した。本文の「（議題 N で確定）」が、末尾の「要件ディスカッションで確定したこと」の番号を指す。

## Boundary Context

- **In scope**
  - DAP とキックの撤去（Rust・Lua・テスト・フィクスチャ）と、DAP だけが使っていた依存。
  - デバッグの門の縮小と、設定のキー・環境変数の整理。
  - 開発用のイベント 3 つ（`OnPastaScene`・`OnPastaSceneAt` は今のまま、`OnPastaDumpVariables` は新設）。
  - 止まるエラーの `.pasta` の位置（行の対応で引けない行と、別のファイルの行を出す誤り）。
  - 残す部品（シーンマップ・設定）の死んだコードと誤った説明。シーンマップの行の対応表・逆引きの撤去。
  - マニュアルの internals とリファレンスの頁、ルートの `DEBUGGING.md`、スキルの references の作り直し。
  - テストの `PASTA_DEBUG` の中和の見直し（`pasta_lua`・`pasta_shiori`・`pasta_sample_ghost`・`pasta_check` の `tests/common` など）。
- **Out of scope**
  - VSCode 拡張の一切と、利用者の章 `book/src/debug/`・README・`SUMMARY.md` の利用者の章の行（`vscode-mcp-client`）。
  - 式を評価して返すイベント（バックログ）。保存変数でない変数（`＄名前`・モジュールの中の状態）の書き出し。
  - 本番（デバッグが無効）でのシーンマップ。シーンマップを後から作ること。
  - デバッグが有効なときに、行の対応で引けない行の `.pasta` の位置を出すこと（設計ディスカッションで落とした）。
  - リロード専用のイベント（`OnPastaReload` など）と、「次の読み込みだけ有効」の仕組み。
  - 読み込みのときの Lua のエラー（Lua のブロックの構文の誤りなど）の位置（議題 8 で確定）。
  - `OnPastaCallFunction`・`OnPastaSetSaveVariable` の振る舞い。
  - `[loader] debug_mode`（名前が似ているが、読み込みの統計のログの設定で別物）。
- **Adjacent expectations**
  - `vscode-mcp-client` は、`OnPastaSceneAt`（Reference0 に `.pasta` の絶対パス、Reference1 に行）と `OnPastaDumpVariables`（応答の後に `profile/pasta/debug/variables.toml` を読む）を使う。デバッグが無効なときの 204 を見て、`[debug] enabled = true` を案内する。この spec は、その決まりを brief のとおりに保つ。
  - SSP・areka の MCP の `reload(shiori)` は引数を取れない。`[debug] enabled` はリロードのたびに `pasta.toml` から読み直されるので、リロードの引数は要らない。
  - 同じウェーブの約束：この spec は `act.lua`・`scene.lua`・`search/`・`code_gen/element_gen.rs` に触らない。ルートの `Cargo.toml`・`Cargo.lock` の席を持つ（DAP の依存を外すだけで、足さない）。
  - `scene-attribute-store` は属性をシーン表の欄にしない（`source_map/scene_join.rs` と重ならない）。`scene-count` は `entry.lua` に関数を置かない。`scene-condition-attributes` が自動転記を `virtual_dispatcher.lua` に置くなら、この spec の後に入る。
  - 0.6.0 の `shiori4-v4-host` は、キックの即時化の対象が無くなった `actor/` の上に建てる。

## Requirements

### Requirement 1: DAP の待ち受けと行フックを無くす

**Objective:** As a ゴーストの作者, I want デバッグを有効にしても辞書が遅くならず、ポートも取られないでほしい, so that DEBUG 版を本番と同じ速さで動かし、2 体を並べても衝突しない。

#### Acceptance Criteria

1. While `[debug] enabled = true`（または `PASTA_DEBUG=1`）, the pasta.dll shall TCP の待ち受けを開かない（どの番号のポートも取らない）。
2. While デバッグが有効, the pasta.dll shall Lua の VM に行フックを付けず、JIT を止めない（無効のときと同じく JIT が働く）。
3. When デバッグを有効にしたゴーストを 2 体、同じベースウェアか 2 つのベースウェアで起こした, the pasta.dll shall どちらも読み込みを終え、どちらでも開発用のイベントを許す。
4. The pasta.dll shall DAP の要求（`initialize`・`setBreakpoints`・`pasta/playSceneAt`・`pasta/reloadShiori` など）を受ける口を持たない。
5. When デバッグが有効な状態で読み込みを終えた, the pasta.dll shall 待ち受けのログ（`debug backend listening`・`debug transport bind failed`）を出さず、代わりに開発用のイベントを許したことを info のログに 1 行出す（議題 6 で確定）。

### Requirement 2: キックの経路を無くす

**Objective:** As a pasta の開発者, I want 外から非同期にシーンを差し込む道（キック）を消してほしい, so that 指名した再生が `OnPastaScene`・`OnPastaSceneAt` の 1 本だけになり、仮想ディスパッチャの分岐とアクターのメッセージが減る。

#### Acceptance Criteria

1. The pasta.dll shall 外からシーンを保留し、次の時計のイベントで起こす道（キック）を持たない。Lua のグローバルの `SHIORI.kick` と、保留の印（`STORE.kick_pending`・`STORE.kick_force`）を持たない。
2. When `OnSecondChange` などの時計のイベントを受けた, the pasta.dll shall 自動トーク・時報・チェイントークの続き・自動トークを止める門（`STORE.auto_talk_paused`）・Status の抑止を、キックを使わなかったときの今と同じに扱う。
3. When 開発者がシーンを指名して再生する, the pasta.dll shall それを `OnPastaScene`・`OnPastaSceneAt` の応答（その要求の応答でそのまま台本を返す）だけで行う。
4. The pasta.dll shall 指名した再生がチェイントークの中断している続きを捨てる振る舞い（`OnPastaScene`・`OnPastaSceneAt` の今の振る舞い）を変えない。

### Requirement 3: デバッグの設定を「有効か」の 1 つにする

**Objective:** As a ゴーストの作者, I want デバッグの設定が「有効か」の 1 つだけになり、古い設定を書いたままでも読み込みが止まらないでほしい, so that DEBUG 版の `pasta.toml` を直さなくても動き、何を書けばよいか迷わない。

#### Acceptance Criteria

1. The pasta.dll shall デバッグの有効を、`pasta.toml` の `[debug] enabled`（既定は false）と環境変数 `PASTA_DEBUG`（設定されていれば `[debug] enabled` より優先）だけで決める。
2. When SHIORI を読み込む（リロードを含む）, the pasta.dll shall `[debug] enabled` をそのときの `pasta.toml` から読み直す（ベースウェアを起こし直さなくても、リロードで有効・無効が切り替わる）。
3. If `[debug]` に、意味の無くなったキー（`port`・`present_as`・`source_map_sidecar`）が書かれたままである, then the pasta.dll shall 読み込みを止めずに続け、そのキーを `pasta.toml` の今の「知らないキー」と同じに読み飛ばす（警告も出さない。議題 4 で確定）。
4. If 環境変数 `PASTA_DEBUG_PORT`・`PASTA_DEBUG_SOURCE_MODE`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR` が設定されている, then the pasta.dll shall それを読まない（振る舞いは設定されていないときと同じ）。
5. While デバッグが有効, the pasta.dll shall 読み込みのときに、読み込んだ全部の `.pasta` のシーンマップ（シーンの宣言の記録と、位置からシーンを引く索引）を作る。
6. While デバッグが無効, the pasta.dll shall シーンマップ（シーンの宣言の記録と、位置からシーンを引く索引）を作らない。
7. The pasta.dll shall 変換した Lua のキャッシュ（`profile/pasta/cache/lua/`）の中身を、デバッグの有無で変えない。
8. The pasta.dll shall 読み込みのときの対応の記録をディスクに書き出さない（`.lua.map` のサイドカーを作らない）。

### Requirement 4: 開発用のイベントの門と決まりを保つ

**Objective:** As a VSCode の拡張・AI エージェント（ぱすた開発）, I want 開発用のイベントが、デバッグが有効なときだけ、今と同じ引数と応答で働いてほしい, so that `vscode-mcp-client` とぱすた開発が今の決まりのまま使える。

#### Acceptance Criteria

1. While デバッグが有効, when `OnPastaScene`・`OnPastaSceneAt` を受けた, the pasta.dll shall 今と同じ引数（Reference0・Reference1）を受け、今と同じ応答（再生したシーンの台本をその要求の応答で返す。見つからないとき・引数の誤りは今の表記と警告）を返す。
2. While デバッグが無効, when 開発用のイベント（`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`）を受けた, the pasta.dll shall 何もせずに 204 を返し、ログに `<イベント名>: debug is disabled; …` の警告を 1 行書く。
3. If 開発用のイベントの SecurityLevel が external である, then the pasta.dll shall 何もせずに 204 を返し、ログに `<イベント名>: SecurityLevel external is refused; …` の警告を 1 行書く。
4. The pasta.dll shall `OnPastaSceneAt` の Reference0 を、ファイルのパス（`ghost/master/` からの相対パスか、UNC を含む絶対パス。区切りは `/` と `\` のどちらでもよい）として扱い、`file://` の URI の剥がしとパーセントの復号をしない（議題 5 で確定）。
5. The pasta.dll shall `OnPastaCallFunction`・`OnPastaSetSaveVariable` の門（デバッグの有無によらず働き、external は断る）を変えない。
6. Where ゴーストが開発用のイベントのハンドラを先に登録している, the pasta.dll shall 既定のハンドラで上書きしない（今の決まりと同じ）。

### Requirement 5: 保存変数を書き出す `OnPastaDumpVariables`

**Objective:** As a VSCode の拡張・AI エージェント, I want 動いているゴーストの保存変数（`＄＊名前`）を、喋らせずにファイルで見たい, so that `save.toml` が終了まで書かれなくても今の値を確かめられ、`raise_event` が台本を再生したり応答が届かなかったりしても困らない。

#### Acceptance Criteria

1. While デバッグが有効, when `OnPastaDumpVariables` を受けた, the pasta.dll shall そのときの保存変数（`＄＊名前`。Lua の `save` の表）を全部、`ghost/master/profile/pasta/debug/variables.toml` に TOML で書き出す。前のファイルは置き換え、フォルダが無ければ作る。
2. When `OnPastaDumpVariables` を処理し終えた, the pasta.dll shall 何も喋らずに 204 を返す。ファイルは 204 を返す前に書き終えている（応答の直後に読めば、そのときの値がある）。
3. The pasta.dll shall 書き出しを、セーブ（`save.toml`）と同じ変換の規則で行い、`[persistence] obfuscate` の設定にかかわらず、テキストの TOML で書く。
4. If 保存変数に TOML に書けない値（関数など）がある, then the pasta.dll shall その値を飛ばして残りを書き、飛ばしたキーと理由をログの警告に書く（セーブと同じ）。
5. If ファイルを書けない（フォルダが作れない・書き込みが拒まれた）, then the pasta.dll shall ログに警告（パスと理由）を 1 行書いて 204 を返し、書きかけのファイルを残さず、前のファイルも消さない（議題 7 で確定）。
6. The pasta.dll shall `OnPastaDumpVariables` を受けても、保存変数・`save.toml`・チェイントークの中断している続きを変えない。

例（保存変数が `＄＊好感度＝３`・`＄＊名前＝「まさお」` のとき）：

```toml
好感度 = 3
名前 = "まさお"
```

### Requirement 6: 止まるエラーの `.pasta` の位置を正しくする

**Objective:** As a ゴーストの作者, I want 止まるエラーの場所が、いつも正しい `.pasta` の行で出てほしく、引けないときは出さないでほしい, so that 変換したキャッシュの行から手で戻さずに直せ、違う場所に惑わされない。

#### Acceptance Criteria

1. The pasta.dll shall 止まるエラーの表記とログの警告で、シーン関数の行と、同じグローバルシーンの Lua のブロックの行を、デバッグの有無によらず、今と同じく `.pasta` のファイルと行（`dic\天気.pasta:5:`）で出す。
2. If 止まった行が、実行中のシーンとは別の `.pasta` から変換した Lua にある, then the pasta.dll shall 実行中のシーンの行の対応から場所を出さない（別のファイルの行を、実行中のシーンのファイルの行として出さない。デバッグの有無によらない。議題 2）。
3. If 止まった行を `.pasta` の行に引けない（行の対応に無い・作者の `.lua`・標準ランタイム）, then the pasta.dll shall デバッグの有無によらず今の文言のまま出す（作者の `.lua` はそのファイルと行、それ以外は場所を出さない）。作者の `.lua` には、`scripts/` の `.lua` のほか、`dic/` に置いてキャッシュにそのまま写される `.lua` も入る（`dic\道具.lua:6:`。design.md の設計ディスカッションの議題 3）。
4. The pasta.dll shall 応答の `X-Error-Reason`、止まった場所（表記の対象）の書き方、止まらない実行時の失敗の表記（行番号を含まない）を変えない。

### Requirement 7: 残す部品を整える

**Objective:** As a pasta の開発者, I want DAP を消した後に残る部品（シーンマップ・設定）に、死んだコード・テストだけの口・誤った説明が残らないでほしい, so that 後の spec（`shiori4-v4-host` など）が読む量が減り、説明を信じて迷わない。

#### Acceptance Criteria

1. The pasta_lua shall 残す部品に、DAP・キック・提示モード・サイドカーのためだけのコード・型・設定の欄・公開の口（`DebugHandle`・`SourceMode`・`KickSink`・`KickRequest`・`enable_debug` の再輸出・`debug_local_addr`・`debug_source_mode` など）を残さない（議題 9 で確定）。
2. The pasta_lua shall 残す部品に、使い手の無い項目を隠す属性（`#![allow(dead_code)]`・`#[allow(dead_code)]`・`#[allow(unused_imports)]`）を残さない（受け継いだ N4。`playscene.rs` 31 行・`source_map/scene_index.rs` 39 行・`source_map/mod.rs` 518・638・647 行）。
3. The pasta_lua shall シーンマップに、使い手の無くなった行の対応表（`ChunkSourceMap` とその問い合わせ・`MapBuilderSink` の行の記録・`resolve_lua_to_pasta`・`pasta_file_for_chunk`）、逆引き（`resolve_pasta_to_lua`・`nearest_pasta_line_with_mapping`）、本番の使い手の無い問い合わせ（`MapBuilderSink::chunk_name`・`SourceMap::scene_index`）を残さず、`.pasta` のパスの照合の正規化を 1 つの関数にする（別の関数をそのまま呼ぶだけの `canonicalize_pasta_file` の形を残さない。N11・N12。design.md の設計ディスカッションの議題 1）。
4. The pasta_lua shall 働きの無い呼び出し（`scene_join.rs` 130 行の `insert_file`）と、その上の誤ったコメントを残さない（N5）。
5. The pasta_lua shall デバッグの設定の解決を `enabled` 1 つの入力で済む形にし、引数 8 つの `DebugConfig::resolve` とテストだけの `DebugConfig::from_file` を残さない（監査の 12 番・N6）。
6. The pasta_lua shall 「設定でデバッグが有効か」を、待ち受けの有無と混ぜずに 1 つの値で持ち、ランタイムの「デバッグが有効か」の問い合わせがその値を返す（`scene-play-event` の申し送り）。
7. The pasta_lua shall 説明の誤りを残さない：`source_map/mod.rs`・`loader/source_map_build.rs`・`code_gen/source_map.rs`・`transpiler.rs` の説明から、行の対応表・逆引き・`LineShift` での付け替えの話を除き、シーンマップがシーンの宣言だけを控えることを書く（`ponytail-lua-transpiler` からの申し送りの `normalize_output` の名前の誤りと「行がズレ得る」は、その説明ごと消える）。
8. The pasta のワークスペース shall DAP だけが使っていた依存（ルートと `pasta_lua` の `socket2`、`pasta_lua` の `serde_json`、`pasta_shiori` の dev の `serde_json`）を外し、依存を足さない。ルートの `serde_json` は `pasta_check`・`pasta_lsp` が使うので残す。

### Requirement 8: マニュアルとスキルの references を新しい形に揃える

**Objective:** As a ゴーストの作者・pasta の開発者, I want マニュアルの internals とリファレンスが、DAP の無い今の pasta.dll を書いていてほしい, so that 消えた設定やイベントを探して迷わない。

#### Acceptance Criteria

1. The pasta のマニュアル shall `internals/debug.md` を「開発用のイベントとシーンマップ」の章に書き直し、DAP・キック・行フック・提示モード・サイドカーの記述を残さない。題を変えるなら `SUMMARY.md` のその行も合わせる。
2. The pasta のマニュアル shall `reference/pasta-toml.md` の `[debug]` 節を `enabled` だけにし、古いキーは読み飛ばされることと、環境変数は `PASTA_DEBUG` だけであることを書く。
3. The pasta のマニュアル shall `lua/shiori-events.md` に `OnPastaDumpVariables`（引数・書き出す先・応答・失敗）を足し、「デバッグが有効なときだけ動く」を 3 つのイベントに共通の決まりにし、待ち受けの話を除く。
4. The pasta のマニュアル shall `internals/` の `shiori`・`execution-model`・`loader`・`index`・`internal-modules`・`transpiler`・`registry-search`・`logging-encoding` の頁、`reference/startup.md`、`reference/runtime-failures.md`（場所の表と 193 行の説明）、`lua/modules/pasta-search.md`、`lua/patterns.md`、`grammar/call-jump.md`、`introduction.md`、ルートの `DEBUGGING.md` から、キックと DAP の記述を消すか新しい形に直す。
5. The pasta のマニュアル shall 利用者の章 `book/src/debug/`（`vscode-mcp-client` の持ち場）を変えない。この spec の頁から利用者の章へのリンクは、`vscode-mcp-client` が残す見出しに後から入る側が rebase で合わせる。
6. When マニュアルを直した, the スキルの references shall `gen-skill-refs` で作り直され、マニュアルと揃っている。
7. When マニュアルを直した, the pasta のマニュアル shall マニュアルの検査（リンク・目次から数える章の数・references の drift）を通す。

### Requirement 9: テストと品質の関門

**Objective:** As a pasta の開発者, I want 撤去の後も、残す振る舞いがテストで守られ、同じ関門を通ってほしい, so that 消しすぎや消し残しにすぐ気づける。

#### Acceptance Criteria

1. The pasta のテスト shall DAP とキックだけを確かめるテスト（`pasta_lua/src/debug/` の DAP・セッション・フック・transport・wiring のテスト、`pasta_lua/tests/runtime/` の `debug_*`・`runtime_toggle_e2e_*` とそのフィクスチャ、`pasta_shiori/tests/` の kick の 6 本・`debug_preservation_test.rs` とそのフィクスチャ・`multi_instance_test.rs` の DAP の部分、`lua_specs/` の kick の 8 本と一覧の行）と、シーンマップから消す行の対応表・逆引きだけを確かめるテストを消し、残す振る舞い（シーンマップ・開発用のイベント・位置）のテストを残す。
2. The pasta のテスト shall 要件 1 の 1・3（ポートを取らない・有効な 2 体が並ぶ）、要件 3 の 2〜4、要件 5、要件 6 の 2・3（別のファイルの行を出さない・`dic/` の `.lua` はそのファイルと行）を確かめるテストを持つ。
3. The pasta のテスト shall 環境の `PASTA_DEBUG`・`PASTA_LOG` に左右されない（中和は `PASTA_DEBUG`・`PASTA_LOG` を残し、意味の無くなった `PASTA_DEBUG_PORT` を外す。議題 3 で確定）。
4. When 直しを終えた, the pasta のワークスペース shall `cargo fmt --check`・clippy（警告 0）・`cargo test --workspace`・Lua のテスト・luacheck を通す（cargo の前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外す）。
5. When 直しを終えた, the pasta.dll shall i686（配布する 32 ビット版）のビルドとテストを通す（x86 を省かない）。

## 要件ディスカッションで確定したこと（2026-11-03）

brief.md だけでは決めきれなかった論点。開発者が 10 件とも推奨案で承認した（2026-11-03）。根拠は research.md にある。

1. **Approach 4 の絞り方**（もとの要件 6.3）：確定。「デバッグが有効なときだけ、行の対応で引けない行をソースマップで引く」に絞って残す。止まるエラーの位置は行の対応でもう出ており、残るのは別のグローバルシーン・別のファイルの Lua のブロックの関数の中だけである。6.3 を落として 6.2 だけを直す案は採らなかった。
   - **設計ディスカッション（2026-11-03）で覆した**：6.3 を落とし、ソースマップをシーンマップ（シーンの宣言の記録と、位置からシーンを引く索引）に縮めた。位置からシーンを引く索引（`scene_join.rs` の `build_scene_index`）は、シーンの宣言の記録と「チャンク → `.pasta`」しか使わず、行の対応表を使わない。DAP を消すと、行の対応表の使い手は 6.3 だけになる。チャンク → `.pasta` は 6.2 のために足す `pasta_sources` から引ける。別のグローバルシーン・別のファイルの Lua のブロックの関数の中の位置を、デバッグが有効なときだけ出すことはしない（デバッグの有無によらず、今のとおり場所を出さない）。もとの 6.4・6.5 は 6.3・6.4 になった。
2. **別のファイルの行を出す誤り**（要件 6.2）：確定。この spec でデバッグの有無によらず直す。`stop.lua` の `pasta_location` が止まったチャンクを見ないので、番号が重なると違うファイルの行を出す。根が同じなので別の spec へ送らない。
3. **テストの中和**（要件 9.3）：確定。`PASTA_DEBUG`・`PASTA_LOG` の中和を残し、`PASTA_DEBUG_PORT` の行だけ外す。ポートが無くなっても、`PASTA_DEBUG=1` は開発用のイベントとシーンマップを有効にするので、「無効なら 204」を確かめるテストなどが環境で変わる。中和は 4 つのクレートにある（brief は `pasta_sample_ghost` だけを挙げる）。
4. **古いキーの扱い**（要件 3.3）：確定。今の「知らないキー」と同じに黙って読み飛ばす（警告を出さない）。`[debug]` を読む道（`PastaConfig::get_custom_config`）は `[logging]`・`[persistence]`・`[talk]` と同じで、知らないキーを黙って読み飛ばす（`deny_unknown_fields` はどこにも無い）。それに合わせる。
5. **`OnPastaSceneAt` の URI の扱い**（要件 4.4）：確定。`file://` の剥がしとパーセントの復号をやめ、パスとしてだけ扱う。マニュアルはパスしか書いておらず、URI は DAP の名残りである。パスに `%41` のような並びがあると、今は別のファイルに直してしまう。UNC の URI の誤り（N13）も、URI を受けなくなれば消える。
6. **有効なときの起動のログ**（要件 1.5）：確定。`debug backend listening` の代わりに、開発用のイベントを許したことを info で 1 行出す。利用者の章（`vscode-mcp-client`）のトラブルシューティングが、有効かどうかをログで確かめる手がかりにできる。
7. **書き出しに失敗したとき**（要件 5.5）：確定。警告を 1 行書いて 204 を返し、書きかけのファイルを残さない（前のファイルは消さない）。
8. **読み込みのときの Lua のエラーの位置**（範囲外）：確定。この spec の範囲外にし、バックログに残す。Lua のブロックの構文の誤りなどは、読み込みの失敗として変換したキャッシュの行で出る。止まるエラー（実行時）とは道が別である。
9. **`pasta_lua` の公開の API を消すこと**（要件 7.1）：確定。0.5.0 で予告なく消す。`pasta_lua` は crates.io に出しているが、0.x の版上げで、ルートの README 88 行の断り書き（公開クレートの pub API は予告なく消すことがある）のとおりである。
10. **`pasta_check validate` の組み込みの名前**（brief の Scope の In）：確定。直さない。組み込みの名前は Call の飛び先（GLOBAL の関数・act のメソッド）だけで、`On` で始まる飛び先は見ない。`OnPastaDumpVariables` はイベントなので足す要が無い。
