# Requirements Document

## Project Description (Input)
VSCode でゴーストを書く作者が困っている。「▶ シーンを実行」と「SHIORI をリロード」は DAP のセッションに attach していないと動かず、デバッガーの制御は不安定である。送り先は固定の `127.0.0.1:9276` で、拡張は編集中のファイルがどのゴーストのものかを見ていない。一方で、SSP と areka は MCP の窓口を持ち、ghost_dev「ぱすた開発」は SSP の MCP だけで開発を回している（DAP は一度も使っていない）。

拡張から DAP の部品（`debuggers`・`breakpoints`・`onDebug:pasta`・提示モードの切り替え・attach）を消し、編集中の `.pasta` のゴーストを見つけて、そのゴーストが動いているベースウェア（SSP か areka）の MCP へ直接送るようにする。コマンドは 3 つ（毎回リロードしてから `OnPastaSceneAt` で再生する「▶ シーンを実行」・リロードの要約を出す「SHIORI をリロード」・`OnPastaDumpVariables` で保存変数を見る「保存変数を表示」）。利用者のマニュアルの章（`book/src/debug/`）を MCP を前提に書き直し、AI エージェントから MCP で使う手順を足す。詳しくは `brief.md`。

## Introduction

pasta の VSCode 拡張（以下「拡張」）の開発の道具を、DAP から SSP・areka の MCP へ移す。

- 拡張は、編集中の `.pasta` からゴーストのルートフォルダを見つけ、そのゴーストが動いているベースウェアを 127.0.0.1 の MCP の窓口から選んで、直接送る。
- コマンドは 3 つ。「▶ シーンを実行」は毎回 SHIORI を読み込み直してから、カーソルの位置のシーンを `OnPastaSceneAt` で再生する。「SHIORI をリロード」は読み込み直して、増えた誤りを要約する。「保存変数を表示」は `OnPastaDumpVariables` で書き出させた保存変数を見せる。
- DAP の部品（デバッガーの種類・ブレークポイント・提示モード・attach・`host`）は、拡張とマニュアルから消す。
- 利用者のマニュアルの章（`book/src/debug/`）を MCP を前提に書き直し、AI エージェントから MCP で使う手順を足す。

例: `…\pasta-in-windows-debug\ghost\master\dic\pose.pasta` の 42 行目で「▶ シーンを実行」を選ぶと、拡張はルート `…\pasta-in-windows-debug` を見つけ、表示名「窓際のぱすたさんDEBUG」で出力に出し、このゴーストが動いている SSP（か areka）へ、保存 → 読み込み直し → 待つ → `OnPastaSceneAt`（`["…\dic\pose.pasta","42"]`）を送り、出た台本を出力のパネルに出す。

## Boundary Context

- **In scope**
  - 拡張からの DAP の部品の撤去（`ponytail-editor-tooling` が外した `host` の廃止を含む）
  - ゴーストの特定と、ゴーストが動いているベースウェアの見つけ方・選び方
  - 3 つのコマンドと、その結果の出し方
  - 拡張の単体の試験と、MCP を真似るサーバーを使う e2e
  - `book/src/debug/` の書き直し、AI エージェントの手順、`SUMMARY.md` のデバッグの行、`editors/vscode/README.md`、ルートの `README.md`
- **Out of scope**
  - pasta.dll の側（DAP の待ち受けの撤去・門の縮小・`OnPastaDumpVariables` の実装。`debug-mcp-runtime`）
  - ブレークポイント・ステップ・変数のツリー・式の評価（DAP と一緒に捨てる）
  - SSP・areka の MCP そのものの直し
  - 「リロードしない」設定（要望が無い）
  - LSP・診断・着色（`ponytail-editor-tooling`）、`vsce` の版上げ（`vsce-4-upgrade`）
  - マニュアルの internals とリファレンスの頁（`debug-mcp-runtime`）
- **Adjacent expectations**
  - `debug-mcp-runtime`（pasta.dll）が次を約束する。拡張はこれに頼る。
    - `OnPastaSceneAt`: Reference0 にファイル（絶対パスか `ghost/master/` からの相対パス。`file://` の URI は受けない）、Reference1 に行（1 以上の半角数字）。再生したら台本を返し、見つからないときと引数の誤りでは失敗の表記を返す。
    - `OnPastaDumpVariables`: 今の保存変数（`＄＊名前`）を `ghost/master/profile/pasta/debug/variables.toml` にふつうの TOML で書き出し終えてから、何も喋らずに 204 を返す。書けなかったときは警告を出して 204 を返し、前のファイルを消さない（読む側が更新時刻で古さを見分ける）。
    - デバッグが有効なことは、`debug backend listening` に代わる info の 1 行で `pasta.log` に出る（文言は `debug-mcp-runtime` の設計で決まる）。古い `[debug]` のキー（`port`・`present_as`・`source_map_sidecar`）は黙って読み飛ばされる。
    - どちらも `pasta.toml` の `[debug] enabled = true`（か `PASTA_DEBUG=1`）のときだけ働き、無効なときと SecurityLevel が external のときは 204 を返す。門は待ち受けを持たない。
  - SSP・areka が次を持つ。拡張はこれに頼り、直さない。
    - 127.0.0.1 だけで待ち受ける MCP の窓口。番号は 9801・9821 の早い者勝ちで、塞がっていれば 9802・9822…9810・9830 の 20 個のどれか。
    - `get_active_ghost_list`・`get_property`・`get_status`・`get_log`・`reload`・`raise_event` の道具。`ghost_name` にゴーストのルートフォルダのフルパスを渡せる。
  - 依存を増やさない。拡張の配る物に新しいライブラリを足さず、`package-lock.json` を変えない。

## Requirements

### Requirement 1: DAP の部品の撤去

**Objective:** As a ゴーストの作者, I want 拡張から使わない DAP の部品が消えること, so that 動かないデバッガーの入口に迷わない

#### Acceptance Criteria

1. The 拡張 shall 「実行とデバッグ」に pasta のデバッガーの種類を出さず、`launch.json` の pasta の構成の雛形と、その属性（`host`・`port`・`sourcePresentation`）を持たない。
2. The 拡張 shall `.pasta` と `.lua` をブレークポイントを置ける言語として登録しない。
3. The 拡張 shall 提示モード（`.pasta` ⇔ `.lua`）を切り替えるコマンド・デバッグのツールバーのボタン・ステータスバーの表示を持たない。
4. The 拡張 shall デバッグのセッションの開始を起動の条件にせず、どのメニューの表示の条件にもデバッグのセッションの種類を使わない。
5. The 拡張 shall 撤去の後も、構文の着色・構文の診断・ゴースト全体の検査・単語参照の装飾を、撤去の前と同じに働かせる。

### Requirement 2: ゴーストの特定

**Objective:** As a ゴーストの作者, I want 編集中の `.pasta` がどのゴーストのものかを拡張が自分で見つけること, so that 同じ名前のゴーストや、ワークツリーの写しを取り違えずに送れる

#### Acceptance Criteria

1. When 作者が `.pasta` のエディタで 3 つのコマンドのどれかを選んだ, the 拡張 shall その `.pasta` のフォルダから上へ 3 フォルダまでの間で、`descript.txt` を持つ `ghost/master` のフォルダを探し、その 2 つ上のフォルダをゴーストのルートとする（`ghost`・`master` の大文字・小文字は区別しない）。
2. The 拡張 shall ベースウェアへゴーストを指名するときは、ゴーストのルートフォルダの絶対パスを使い、`descript.txt` の `name` を使わない。
3. The 拡張 shall 出力と知らせに出すゴーストの表示名に `descript.txt` の `name` を使い、`name` を読めないときはルートフォルダの名前を使う。
4. If ゴーストのルートが見つからない, then the 拡張 shall 「ゴーストのフォルダの中の `.pasta` ではない」ことと、開いているファイルのパスを伝え、ベースウェアへは何も送らずに止まる。
5. If 今のエディタが、ディスクに保存された `.pasta` ではない（新規の名前の無い文書・別の言語のファイル）, then the 拡張 shall ゴーストを特定できないことを伝えて止まる。

### Requirement 3: ベースウェアの見つけ方と選び方

**Objective:** As a ゴーストの作者, I want ゴーストが動いている SSP か areka を拡張が見つけて送ること, so that 番号を合わせたり、違うベースウェアへ送ったりしない

#### Acceptance Criteria

1. When 送る相手を決める, the 拡張 shall 127.0.0.1 の MCP の窓口の候補を 9801・9821・9802・9822…9810・9830 の順に聞き、特定したゴースト（ルートの絶対パス）が動いている相手だけを候補にする。
2. The 拡張 shall 番号だけで相手を決めず、相手のベースウェアの名前（`baseware.name`）を聞き、送るたびに出力のパネルへ、相手の名前・番号・ゴーストの表示名・ルートのパスを出す。
3. When 2 つ以上の相手でそのゴーストが動いている, the 拡張 shall 相手の名前と番号を並べて作者に 1 つを選ばせる。
4. If 作者が選ばずに選択を閉じた, then the 拡張 shall 何も送らずに止まる。
5. When 相手が決まった, the 拡張 shall その相手をゴーストごとに覚え、この VSCode の窓が開いている間は、次のコマンドでも聞き直さずに同じ相手へ送る。
6. If 覚えた相手が答えない、またはその相手でそのゴーストがもう動いていない, then the 拡張 shall 覚えを捨てて、候補を聞き直す。
7. If どの候補にもそのゴーストが動いている相手が無い, then the 拡張 shall 「このゴーストが動いている SSP・areka が見つからない」ことを、ゴーストの表示名・ルートのパス・確かめること（ベースウェアでゴーストを起動しているか、MCP の窓口が有効か）とともに伝えて止まる。
8. While 候補を聞いている, the 拡張 shall 受け付けても答えない相手で止まり続けず、一定の時間で打ち切って次の候補へ進む。
9. The 拡張 shall 127.0.0.1 の他へは送らず、送り先のホストや番号を変える設定を持たない。

### Requirement 4: ▶ シーンを実行

**Objective:** As a ゴーストの作者, I want カーソルの位置のシーンを、いま保存した辞書でその場で再生すること, so that 書いてすぐ動きを確かめられる

#### Acceptance Criteria

1. The 拡張 shall `.pasta` のエディタの右クリックのメニューの先頭に「▶ シーンを実行」を、いつでも出す。
2. When 作者が「▶ シーンを実行」を選んだ and 文書に未保存の変更がある, the 拡張 shall 作者に確かめずに文書を保存してから進む。
3. When 保存が済み、相手が決まった, the 拡張 shall 相手へそのゴーストの SHIORI の読み込み直しを頼み、読み込みが終わるまで待つ。
4. If 読み込みが一定の時間で終わらない, then the 拡張 shall 再生を頼まずに、読み込みの終わりを確かめられなかったことを伝える。
5. If 読み込み直しの間に、そのゴーストの読み込みの誤りが増えた（Requirement 5 の 4 と同じ 2 つの記録で数える。警告だけなら再生する）, then the 拡張 shall 再生を頼まずに、増えた誤りの数と内容の要約を出力のパネルに出し、再生しなかったことを知らせる。
6. When 読み込みが誤り無く終わった, the 拡張 shall `OnPastaSceneAt` を、Reference0 に `.pasta` の絶対パス、Reference1 にカーソルの行（1 から数えた半角数字）を付けて送る。
7. When ゴーストが台本を返した, the 拡張 shall その台本を出力のパネルに出す。台本が失敗の表記（`【シーンが見つからない：…】` など）のときも、そのまま出す。
8. If 相手の答えが「No response from server」だった, then the 拡張 shall それを失敗とせず、相手の再生の記録からこの再生の台本を引いて出力のパネルに出す。
9. If 「No response from server」の後で、相手の再生の記録にもこの再生の台本が見つからない, then the 拡張 shall 台本を確かめられなかったことを伝える。
10. If ゴーストが台本を返さなかった（204。デバッグが無効か、SecurityLevel が external で断られた）, then the 拡張 shall 「`pasta.toml` に `[debug] enabled = true` を書いてリロードしてください」と案内する。
11. The 拡張 shall 読み込み直しを省く設定を持たず、「▶ シーンを実行」のたびに読み込み直す。

### Requirement 5: SHIORI をリロード

**Objective:** As a ゴーストの作者, I want 読み込み直したときに誤りの要約が出ること, so that ログを自分で読みに行かずに、読み込みの成否が分かる

#### Acceptance Criteria

1. The 拡張 shall `.pasta` のエディタの右クリックのメニューに「SHIORI をリロード」を、いつでも出す。
2. When 作者が「SHIORI をリロード」を選んだ and 文書に未保存の変更がある, the 拡張 shall 作者に確かめずに文書を保存してから進む。
3. When 保存が済み、相手が決まった, the 拡張 shall 相手へそのゴーストの SHIORI の読み込み直しを頼み、読み込みが終わるまで待つ。
4. When 読み込みが終わった, the 拡張 shall 読み込み直しの間に増えた、ベースウェアの誤りの記録（そのゴーストの分）と、ゴーストの `pasta.log` の行を要約して出力のパネルに出す（誤りの数・警告の数と、その行）。
5. When 増えた誤りと警告が無い, the 拡張 shall 読み込みが誤り無く終わったことを短く知らせる。
6. If ゴーストの `pasta.log` が見つからない、または読めない, then the 拡張 shall ベースウェアの誤りの記録だけで要約し、`pasta.log` を読めなかったことを添える。
7. If 読み込みが一定の時間で終わらない, then the 拡張 shall 読み込みの終わりを確かめられなかったことを伝え、そこまでに増えた分を要約する。
8. The 拡張 shall 読み込み直しの後で、シーンを自分から再生しない。

### Requirement 6: 保存変数を表示

**Objective:** As a ゴーストの作者, I want 動いているゴーストの保存変数の今の値を見ること, so that `save.toml` に書かれるまで待たずに、値の変わり方を確かめられる

#### Acceptance Criteria

1. The 拡張 shall 「保存変数を表示」を、コマンドパレットと `.pasta` のエディタの右クリックのメニューから選べるようにする。
2. When 作者が「保存変数を表示」を選び、相手が決まった, the 拡張 shall 相手へ `OnPastaDumpVariables` を送り、ゴーストが書き出した `ghost/master/profile/pasta/debug/variables.toml` を読んで、その中身を出力のパネルに出す。
3. If 送った後に `variables.toml` が無い、または送る前から書き換わっていない, then the 拡張 shall 古い中身を今の値として出さずに、Requirement 4 の 10 と同じデバッグを有効にする案内を出す。
4. The 拡張 shall 「保存変数を表示」の前に SHIORI を読み込み直さず、未保存の文書も保存しない（動いている間の値を、そのまま見せる）。

### Requirement 7: 結果の出し方

**Objective:** As a ゴーストの作者, I want 3 つのコマンドの結果と失敗の理由が 1 か所にまとまって出ること, so that 何が送られ、何が返ったかを後から追える

#### Acceptance Criteria

1. The 拡張 shall 3 つのコマンドの結果（相手・ゴースト・台本・要約・案内）を、拡張の 1 つの出力のパネルに時刻の順に積む。
2. When コマンドが失敗や案内で止まった, the 拡張 shall 通知で短く知らせ、詳しい内容を出力のパネルに出す。
3. If 相手が誤りを返した（道具の誤り・つながらない・答えの形が違う）, then the 拡張 shall その文言を出力のパネルに出し、そのコマンドを止める。
4. While コマンドが相手とやり取りしている, the 拡張 shall 進んでいることを作者に見せる。
5. While 同じゴーストへのコマンドが進んでいる, the 拡張 shall そのゴーストへの次のコマンドを始めずに、進んでいるものがあることを伝える。

### Requirement 8: 試験

**Objective:** As a pasta の開発者, I want ベースウェア無しで新しい流れを試せること, so that SSP・areka の実機を起こさずに、壊れたことに気づける

#### Acceptance Criteria

1. The 拡張の試験 shall ベースウェアの代わりに、手元で MCP の窓口を真似る小さなサーバーを立て、3 つのコマンドの流れを確かめる。
2. The 拡張の試験 shall 少なくとも、相手が見つからない・2 つの相手で動いている・同じ番号に別のベースウェアが答える・読み込みの誤りが増えた・読み込みが終わらない・「No response from server」・204 の場合を確かめる。
3. The 拡張の試験 shall 拡張の宣言に DAP の部品（デバッガーの種類・ブレークポイント・`onDebug:pasta`・デバッグのツールバー・デバッグのセッションの種類の条件）が無いことを確かめる。
4. The 拡張の試験 shall DAP の部品に向けた試験を残さず、拡張の試験の全部が通る。

### Requirement 9: 利用者のマニュアルの章（`book/src/debug/`）

**Objective:** As a ゴーストの作者, I want マニュアルが今の拡張の使い方を書いていること, so that 消えた attach やブレークポイントの手順に迷わない

#### Acceptance Criteria

1. The 利用者のマニュアル shall `source-level.md` と `constraints.md` の頁を消し、目次（`SUMMARY.md`）のデバッグの行から外す。
2. The 利用者のマニュアル shall `index.md`・`vscode-setup.md`・`dev-actions.md`・`troubleshooting.md` を、拡張が SSP・areka の MCP へ送る形で書き直し、attach・`launch.json`・`host`・番号 9276・ブレークポイント・ステップ・変数の inspect・提示モードの手順を残さない。
3. The 利用者のマニュアル shall シーンの再生と保存変数の表示には、`pasta.toml` の `[debug] enabled = true`（か `PASTA_DEBUG=1`）が要ること、書いた後に読み込み直すことを書く。
4. The 利用者のマニュアル shall 3 つのコマンドの動き（ゴーストの見つけ方・相手の選び方と覚え方・毎回の読み込み直し・誤りが増えたとき・「No response from server」のとき・204 のときの案内）と、相手が見つからないときの確かめ方を書く。
5. The 利用者のマニュアル shall 前の版の `launch.json` の pasta の構成と、`pasta.toml` の `[debug]` の古いキー（`port`・`present_as`・`source_map_sidecar`）はもう効かず、消してよいことを書く。
10. The 利用者のマニュアル shall デバッグが有効になったかの確かめ方を、`debug backend listening` の行ではなく、`debug-mcp-runtime` が出す新しい info の行で書く。
6. The 利用者のマニュアル shall `dev-actions.md` の動画を、新しい流れ（シーンの実行・編集・もう一度の実行）を撮り直したものに替える。
7. The 利用者のマニュアル shall `validate.md` を変えない。
8. The 利用者のマニュアル shall 書き直した頁の導入と締めの台詞の決まりを保ち、マニュアルのリンクの検証と中身の検証が通る。
9. The 利用者のマニュアル shall 他の章から引かれているデバッグの章のアンカー（「シーン再生（▶ シーンを実行）」・「デバッグの有効化」・「pasta.toml による有効化」・「優先順位」）を、同じ見出しで残す。

### Requirement 10: AI エージェントから MCP で使う手順

**Objective:** As a AI エージェントでゴーストを開発する作者, I want エージェントが MCP で安全に試す手順がマニュアルとスキルに載ること, so that ぱすた開発のように違う相手へ送ったり、「No response」で迷ったりしない

#### Acceptance Criteria

1. The 利用者のマニュアル shall 「AI エージェントから MCP で使う」手順を載せる。中身は、相手の確かめ（`get_property(baseware.name)`）、`ghost_name` にルートフォルダの絶対パスを渡すこと、`reload(target="shiori")` と終わるまでの待ち、`OnPastaScene`（完全一致の名前で 1 本を指名）と `OnPastaSceneAt`、`OnPastaSetSaveVariable` と `OnPastaDumpVariables`、「No response from server」のときの `get_log(script)` での読み直しである。
2. The 手順 shall 配るスキル（`pasta-lua-coding` か `pasta-ghost-authoring`）の references に、マニュアルと同じ中身で届く。
3. The 手順 shall `OnPastaSceneAt`・`OnPastaDumpVariables` の引数・書き出す場所・204 の決まりを、`debug-mcp-runtime` の決まりと食い違わずに書く。

### Requirement 11: README

**Objective:** As a 拡張やリポジトリを初めて見る人, I want README が今の拡張の働きを書いていること, so that 無くなったデバッガー接続を期待しない

#### Acceptance Criteria

1. The `editors/vscode/README.md` shall 機能の一覧から「デバッガ接続（DAP）・提示モードの切り替え」を消し、3 つのコマンドと、動いている SSP・areka へ MCP で送ることを書く。
2. The ルートの `README.md` shall デバッグの章と VSCode 拡張の説明から「`.pasta` の行で止めて調べる」「デバッガ接続」を消し、新しい働きに合わせる。
