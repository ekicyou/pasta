# Requirements Document

## Project Description (Input)

ゴーストの開発者は、AI エージェントに SSP の MCP サーバーを通してゴーストを動かさせ、デバッグさせたい。そのためには、MCP から「このシーンを今すぐ再生して」と頼めなければならない。

MCP の `raise_event` は、SSTP NOTIFY と同じく SHIORI にイベントを送り、返ってきた台本を再生して、ツールの結果にも台本を載せる。ところが、pasta には指名したシーンを再生するイベントが無い。イベント名と同じ名前のシーンは前方一致で再生されるだけで、任意のシーン（とくにローカルシーン）は指名できない。シーンを指名して再生する仕組み（シーンキック）はあるが、入口は VSCode のデバッグの通り道だけで、SHIORI のイベントからは届かない。

そこで、名前でシーンを指名する `OnPastaScene` と、位置で指名する `OnPastaSceneAt` の 2 つの組み込みのイベントを足す。デバッグを有効にしたときだけ動き、配るゴーストの挙動は何も変えない。詳しい経緯と決定は `brief.md` を参照。

## Introduction

この spec は、SHIORI のイベントで指名したシーンをその場で再生する 2 つの組み込みのイベントを定める。

- `OnPastaScene`: 名前で指名する。Reference0 が大域シーンの名前、Reference1 がローカルシーンの名前（省略できる）。
- `OnPastaSceneAt`: 位置で指名する。Reference0 がファイル、Reference1 が行。

```
raise_event("OnPastaScene",   ["会話"])                     → ＊会話 を再生
raise_event("OnPastaScene",   ["会話", "挨拶"])             → ＊会話 の中の ・挨拶 を再生
raise_event("OnPastaSceneAt", ["dic/boot.pasta", "42"])     → その行のシーンを再生
```

1 ビート目はイベントの応答として返すので、MCP の `raise_event` の結果に台本が載る。続きのビートと、進行中の会話の中断は、今のシーンの続きの仕組みに任せる。見つからないときと引数の誤りは、ほかの実行時の失敗と同じ形でログとバルーンに出す。

主な利用者は 2 人である。

- **ゴーストの開発者**: pasta.toml か環境変数でデバッグを有効にし、AI エージェントに MCP を使わせる人。
- **AI エージェント**: SSP の MCP サーバーの `raise_event` でイベントを送り、結果の台本とログを読んで辞書を直す。

## Boundary Context

- **In scope**
  - `OnPastaScene`・`OnPastaSceneAt` の 2 つの組み込みのイベントと、その引数の確かめ
  - 書いた名前どおりの完全一致でシーン（大域とローカル）を引くこと
  - ファイルと行からシーンを引くこと（ゴーストの `ghost/master/` からの相対パスを含む）
  - デバッグが有効かどうかでイベントの働きを切り替えること
  - 見つからないときと引数の誤りを、失敗を出す仕組みに載せること
  - 自動テスト
  - マニュアルとスキルの references
- **Out of scope**
  - VSCode の拡張と、SSP の MCP サーバーの側の変更
  - シーンに引数やアクターを渡すこと
  - 登録名（`会話_1` のように番号の付いた名前）での指名
  - 前方一致での指名
  - SSTP 以外の経路（SSTP の押し出し、`\![raise]` の作例）
  - デバッグを無効にしたゴーストでの再生
  - `pasta_check validate` にイベント名の一覧を持たせること
- **Adjacent expectations**
  - **シーンキック**（完了 spec `pasta-scene-kick`・`pasta-scene-kick-from-cursor`・`scene-identity-format`）: 進行中の会話の差し替えと続きの流し方、位置からシーンを引く規則を、そのまま受け継ぐ。作り直さない。同じ名前のシーンが別のファイルにあるときの、位置の索引と通し番号の制約も受け継ぎ、直さない。
  - **書いた名前どおりの検索と別名**（完了 spec `scene-search-key-normalization`・`scene-name-alias`）: 名前の直し方と `[scene.alias]` の当て方は、ふだんの検索と同じにする。
  - **失敗を出す仕組み**（完了 spec `failure-output-unification`・`runtime-error-balloon`）: 失敗の表記の形と、出す・出さないの設定に従う。
  - **ランダムトークを止めるスイッチ**（`ontalk-suppression`、同じウェーブ）: 止めている間に、このイベントのシーンの続きを流すかどうかは、あちらが決める。あちらは場面の続きを「止める」案を推奨にしている。この spec の 1 ビート目は、イベントの応答としてその場で返すので、あちらの影響を受けない。
  - **外から呼ぶイベント**（`external-control-events`、この spec の後に入る）: 既定のハンドラーの置き方と、マニュアルの既定ハンドラーの表を共有する。あちらがこの spec の形に揃える。
  - **SSP の MCP サーバー**: `raise_event` が SHIORI に GET で届くことを前提にする（ツールの説明に「返った台本を再生する」とある）。設計のときに SSP の実機で確かめる。

## Requirements

### Requirement 1: 名前でシーンを再生する（OnPastaScene）

**Objective:** As a AI エージェント, I want 書いた名前どおりの名前でシーンを指名して再生させること, so that 辞書に書いた任意のシーンを、MCP からすぐに試せる。

#### Acceptance Criteria

1. When `OnPastaScene` を受け取り、Reference0 に大域シーンの名前があり、Reference1 が無いとき, the pasta shall その名前の大域シーンを再生し、1 ビート目のさくらスクリプトをこのイベントの応答として返す。
2. When `OnPastaScene` を受け取り、Reference0 に大域シーンの名前が、Reference1 にローカルシーンの名前があるとき, the pasta shall Reference0 の大域シーンの中にある Reference1 のローカルシーンを再生し、1 ビート目のさくらスクリプトをこのイベントの応答として返す。
3. The pasta shall 名前を、番号の付かない書いたとおりの名前（`＊会話` なら `会話`、`・挨拶` なら `挨拶`）として受け付ける。
4. The pasta shall 名前を完全一致で探し、前方一致はしない（`会話` で `＊会話2` を再生しない）。
5. The pasta shall 名前の直し方と、大域シーンの名前への `[scene.alias]` の別名の当て方を、ふだんのシーンの検索と同じ規則にする。
6. When 同じ名前の候補が複数あるとき, the pasta shall ふだんのシーンの検索と同じ選び方（重ならない順送り）で 1 つを選んで再生する。
7. When Reference1 が空の文字列のとき, the pasta shall Reference1 を省略したときと同じに扱う。
8. When ローカルシーンを指名し、同じ名前の大域シーンが複数あるとき, the pasta shall その名前のローカルシーンを持つ大域シーンだけを候補にし、その中からふだんと同じ選び方（重ならない順送り）で 1 つを選んで、そのローカルシーンを再生する（brief の「まず大域を選ぶ」を改めた。持たない方を選んで「見つからない」が時々混じるのを避けるため）。

### Requirement 2: 位置でシーンを再生する（OnPastaSceneAt）

**Objective:** As a AI エージェント, I want ファイルと行でシーンを指名して再生させること, so that 同じ名前のシーンが複数あっても、狙った 1 つを確実に試せる。

#### Acceptance Criteria

1. When `OnPastaSceneAt` を受け取り、Reference0 にファイルが、Reference1 に行があるとき, the pasta shall その位置のシーンを再生し、1 ビート目のさくらスクリプトをこのイベントの応答として返す。
2. The pasta shall 位置からシーンを、VSCode の「カーソル位置からシーンを再生」と同じ規則で引く（その行を含むシーンのうち最も内側のシーン。無ければ、その行と同じか下にある最も近いシーンの宣言）。
3. When Reference0 が相対パスのとき, the pasta shall ゴーストの `ghost/master/` からの相対パスとして扱う（SSP の今のフォルダには左右されない）。
4. When Reference0 が絶対パスのとき, the pasta shall そのパスのファイルとして扱う。
5. The pasta shall パスの区切りに `/` と `\` のどちらも受け付け、ファイル名の大文字・小文字は VSCode からの再生と同じ規則で突き合わせる。
6. The pasta shall 行をエディタと同じく 1 から数え、半角の 1 以上の整数だけを行として受け付ける（全角の数字や前後の空白は誤りにする）。
7. When 同じ名前のシーンが複数あり、そのうちの 1 つの位置を指したとき, the pasta shall 指した位置のシーンそのものを再生する（同じ名前のシーンが別のファイルにあるときの、受け継いだ制約の範囲で）。

### Requirement 3: 再生と続きの動き

**Objective:** As a ゴーストの開発者, I want 指名したシーンが、ふだんのシーンと同じ動きで再生されること, so that MCP で確かめた結果が、実際のゴーストの動きと食い違わない。

#### Acceptance Criteria

1. When 指名したシーンを再生するとき, the pasta shall 進行中の会話（止まっている場面の続き）を中断し、元には戻さない（VSCode からのシーンキックと同じ）。
2. The pasta shall このイベントの応答に、再生したシーンの 1 ビート目だけを載せる。
3. When 再生したシーンに 2 ビート目以降があるとき, the pasta shall 続きを、ふだんの場面の続きと同じ時機（ランダムトークの次の時刻に達したとき）に出す。
4. The pasta shall このイベントを受け取ったその場でシーンを再生し、ランダムトークを抑える Status（選択肢の表示中など）では止めない（ほかのイベントのハンドラーと同じ）。
5. If 再生したシーンの実行がエラーで止まったとき, the pasta shall ふだんのイベントと同じく、止まるエラーの表示とログで知らせる。

### Requirement 4: デバッグを有効にしたときだけ動く

**Objective:** As a ゴーストの開発者, I want 2 つのイベントがデバッグを有効にしたときだけ働くこと, so that 配るゴーストの動きが何も変わらず、外から勝手にシーンを再生されない。

#### Acceptance Criteria

1. While pasta.toml の `[debug]` か環境変数 `PASTA_DEBUG` でデバッグを有効にしているとき, the pasta shall 2 つのイベントでシーンを再生する。
2. The pasta shall デバッグが有効かどうかを設定（pasta.toml と環境変数）で決め、VSCode のデバッグの待ち受けが動いているか（ポートが取れたか）には左右されない。
3. While デバッグが有効なとき, the pasta shall 位置からシーンを引くためのデバッグ情報（ソースマップと位置の索引）を読み込みのときに作り、待ち受けが立たなかったときも持ち続ける。
4. If デバッグが無効なときに 2 つのイベントのどちらかを受け取ったとき, the pasta shall シーンを再生せずに 204 No Content を返し、「デバッグが無効なので再生しない」ことをログに書く。
5. Where 作者が Lua で `OnPastaScene` か `OnPastaSceneAt` のハンドラーを先に定義しているとき, the pasta shall 作者のハンドラーを優先する。
6. If リクエストの `SecurityLevel` が `external` のとき, the pasta shall シーンを再生せずに 204 No Content を返し、その旨をログに書く（`OnPastaClock` が `external` を断る先例に揃える）。

### Requirement 5: 見つからないときと引数の誤り

**Objective:** As a AI エージェント, I want 再生できなかった理由がツールの結果とログに出ること, so that 名前や位置の誤りを自分で直せる。

#### Acceptance Criteria

1. If `OnPastaScene` で、指名した大域シーンが見つからないとき, the pasta shall 再生せず、イベント名と探した名前と「見つからない」ことを、失敗の表記とログの警告に出す。
2. If `OnPastaScene` で、大域シーンは見つかったが、その中に指名したローカルシーンが無いとき, the pasta shall 再生せず、イベント名と大域・ローカルの両方の名前（例: `＊会話 ・挨拶`）と「見つからない」ことを、失敗の表記とログの警告に出す。
3. If `OnPastaSceneAt` で、指したファイルが辞書に無いか、指した行と同じか下にシーンが無いとき, the pasta shall 再生せず、イベント名とファイルと行と「見つからない」ことを、失敗の表記とログの警告に出す。
4. If Reference0 が無いか空のとき、または `OnPastaSceneAt` の Reference1 が無いか 1 以上の整数でないとき, the pasta shall 再生せず、イベント名と、どの引数が足りないか・どの値が誤りかを、失敗の表記とログの警告に出す。
5. The pasta shall 失敗の表記を、ほかの実行時の失敗と同じ形（`【種類：対象】`。対象にイベント名と、探した名前か位置を含める）で、同じ設定（`[logging] balloon_warning`）に従ってバルーンに出し、出すときはこのイベントの応答に載せる。
6. If 再生できなかったとき, the pasta shall 進行中の会話（止まっている場面の続き）を中断せず、そのまま残す。

### Requirement 6: マニュアルとスキルの references

**Objective:** As a ゴーストの開発者, I want 2 つのイベントの使い方がマニュアルとスキルに載っていること, so that 自分も AI エージェントも、呼び方と結果の読み方が分かる。

#### Acceptance Criteria

1. The マニュアル shall SHIORI イベントの章（`book/src/lua/shiori-events.md`）の既定ハンドラーの表に 2 つのイベントを載せ、それぞれの節で、引数、名前と位置の探し方、デバッグが有効なときだけ動くこと、見つからないときの出方、2 ビート目からは後で出ることを説明する。
2. The マニュアル shall MCP の `raise_event` からの呼び方の例を載せる。
3. The マニュアル shall MCP から 2 つのイベントを呼ぶ前に SHIORI のリロード（MCP の `reload`）を呼ぶ作法を、呼び出し側の務めとして書き、その理由（辞書を書き換えたあとは、読み込んだ中身と辞書の位置がずれる）を説明する。
4. The マニュアル shall 内部設計のデバッグの章（`book/src/internals/debug.md`）のシーンキックの節と、デバッグの章（`book/src/debug/`）のシーンの再生の節から、2 つのイベントの節を参照する。
5. When マニュアルを直したとき, the スキルの references shall マニュアルから作り直され、マニュアルと食い違わない。

### Requirement 7: 今の動きを壊さない

**Objective:** As a ゴーストの開発者, I want この spec が今の動きを変えないこと, so that 配るゴーストと VSCode からのデバッグが、今までどおりに動く。

#### Acceptance Criteria

1. While デバッグが無効なとき, the pasta shall 2 つのイベント以外のすべてのイベントの挙動を変えない。
2. The pasta shall VSCode からのシーンの再生（名前・位置のキック）と SHIORI のリロードの挙動を変えない。
3. The pasta shall 今ある自動テストを、書き換えずに通す。
4. The pasta shall 名前での再生（大域・ローカル）、位置での再生（相対パス・絶対パス）、見つからないとき、引数の誤り、デバッグが無効なときを、自動テストで確かめる。

## 仮定と未決事項

未決事項は無い。要件の話し合い（2026-10-22）で、次のとおり閉じた。

### 精査で閉じた項目（2026-10-22）

- **デバッグが有効かの決め方とデバッグ情報**（要件 4.2・4.3）: 設定で決める。デバッグ情報（ソースマップと位置の索引）は読み込みのときに作り、ポートが取れず待ち受けが立たなかったときも持ち続ける（今は捨てている。`research.md` 3.1）。必要になったときに後から作る方式は採らない。後から作ると、辞書を書き換えたあとに、読み込んだ中身と食い違う表ができ、違うシーンを黙って再生しうるため。辞書と読み込みを揃えるのは、呼ぶ前に SHIORI をリロードする呼び出し側の務めとし、マニュアルに書く（要件 6.3。2026-10-22 開発者の判断）。
- **デバッグが無効なときの応答**（要件 4.4）: brief のとおり 204 とログ。`OnPasta` で始まる名前は pasta の組み込みのイベントの名前なので、同じ名前で始まるシーンを書いたゴーストへのフォールバックは守らない（2026-10-22 開発者の承認）。
- **同じ名前の大域シーンが複数あるときのローカルの指名**（要件 1.8）: そのローカルシーンを持つ大域シーンの中から、順送りで選ぶ。brief の「まず大域を選ぶ」は、呼ぶたびに成功と失敗が入れ替わるので改めた（2026-10-22 開発者の承認）。
- **`SecurityLevel` が `external` のとき**（要件 4.6）: `OnPastaClock` に揃えて断る（2026-10-22 開発者の承認）。SSP の MCP サーバーが送る値は、設計のときに実機で確かめる。
- **失敗の文言の形**（要件 5.5）: 失敗の表記を揃えた先の spec（`failure-output-unification`）の形 `【種類：対象】` に従う。brief の例の文言は採らない。引数の誤りの種類の名前を新しく足すかは、設計で決める。
- **`raise_event` が GET で届くか**: 設計のときに SSP の実機で確かめる（`research.md` 6 の Research Needed）。今の pasta は NOTIFY でもハンドラを動かし、応答だけを捨てる（`research.md` 3.4）。
- **引数の細かい形**（要件 1.7・2.6）: 空の Reference1 は省略と同じ。行は半角の 1 以上の整数だけ。想定外の書き方を推して救わない。
- **Status による抑止**（要件 3.4）: 抑止は OnSecondChange の中だけで、ほかのイベントは止まらない。このイベントも同じにする。
