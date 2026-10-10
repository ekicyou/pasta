# Requirements Document

## Introduction

入門ガイドの題材であるサンプルゴースト hello-pasta の立ち絵は、`crates/pasta_sample_ghost` の `image_generator.rs` が描くピクトグラム（赤・青の丸い頭と三角の胴、128×256）で、表情の違いは目の記号だけである。初心者が「自分のゴーストが表情を変えてしゃべった」と実感しにくく、物語のある入門ガイドの主役としても、スクリーンショットの被写体としても弱い。

本仕様は次の 3 つを行う。

1. 女の子・男の子の立ち絵を、画像生成（fal.ai）で作ったイラストに置き換える。表情 9 種ずつ、同じポーズ・構図・寸法の透過 PNG とし、表情を切り替えても髪・服・輪郭がずれないようにする。
2. 画像を「Rust で毎回生成する生成物」から「リポジトリにコミットした素材」へ切り替える。絵を描くプログラムと、配布スクリプト・リリース CI の「絵を生成する段」を外し、コミットした絵が上書きされないようにする。生成元（モデル名・手順）とライセンス表記を、食い違いなく記録する。
3. `surfaces.txt` に当たり判定を足し、部位名を確定する。入門ガイド 8 段目（`dic/08-touch.pasta`）がダブルクリックの `Reference4`（`＄ｒ４`）で触られた部位名を台詞に読み上げるため。

表情の種類とサーフェス番号（女の子 0〜8・男の子 10〜18）、辞書、マニュアル本文、バルーンは変えない。

## Boundary Context

- **In scope**:
  - 立ち絵 18 枚（または胴体＋顔パーツの構成）の生成・後処理・コミット
  - `shell/master/surfaces.txt`（当たり判定を含む）と `shell/master/descript.txt`（吹き出しの位置）の更新
  - `crates/pasta_sample_ghost` の「生成物」から「素材」への切り替え（絵を描くプログラム・生成 API・CLI・依存部品・ビルドスクリプト・README・テスト）
  - 配布スクリプト `release.ps1`（絵を生成する段の除去と段の番号）、ルートの `release.bat` の説明、`STAGES.md` の部位名の注記と 8 段目の検証の表
  - 生成元・手順・ライセンスの記録（新規の記録と、ライセンス表記の食い違いの解消）
  - 配布物 `.nar` の中身と大きさの確認
  - 入門ガイドの準備の章（`book/src/getting-started/setup.md`）の節「シェルの中身」に、新しい立ち絵の見本を載せる（絵の行と、本に置く絵のファイルだけ。2026-10-10 開発者の指示）
- **Out of scope**:
  - 表情の種類・サーフェス番号の変更（`dic/03-face.pasta` の対応は維持する）
  - バルーンの作り直し、辞書の内容の変更（`hello-pasta-tutorial-stages` の成果物）、マニュアル本文（`getting-started-story-guide` の成果物。ただし `setup.md` の節「シェルの中身」に見本の絵の行を足すことは In scope）
  - 入門ガイドのスクリーンショットの撮影と掲載（後続の `getting-started-screenshots`）
  - シェルの説明の章（後続の `manual-shell-guide`）
  - 辞書が無い間に立ち絵を出すかどうか（`boot-surface-without-dic`）
- **Adjacent expectations**:
  - `release-ci`（完了）: `.gitignore` は「シェルの画像は追跡を続ける」と明記済み（R10.5）。本仕様はその境界をそのまま使い、追跡の作業を増やさない。
  - `release-workflow`（常駐）: リリース CI（`.github/workflows/release.yml`）は `release.ps1` をそのまま呼ぶ。本仕様が `release.ps1` を変えている間はリリースを実行しない（ウェーブの約束。リリースの実行中は `release.ps1` の変更を main へ入れない）。
  - `hello-pasta-tutorial-stages`（完了）・`getting-started-story-guide`（完了）: 8 段目の辞書（`dic/08-touch.pasta`）と章（`book/src/getting-started/08-touch.md`）は逐語で照合される。頭の部位名を `Head` のまま確定したので、辞書・8 段目の章・`book/tools/` は触らない。本で触るのは `setup.md` の節「シェルの中身」への絵の行と、本に置く絵のファイルだけである。これはウェーブの約束（「`book/` は部位名の都合で 8 段目だけ」）を開発者の指示（2026-10-10）で広げたもので、ロードマップの記述は完了処理で合わせる。同じウェーブのほかの spec は `setup.md` を触らない。
  - `getting-started-screenshots`（後続）: 新しい立ち絵が入った後にスクリーンショットを撮る。本仕様はスクリーンショットを撮らない。本仕様が `setup.md` に足す絵の行は、今の本文検査に通る書き方（表のセルまたは箇条書き）にする。章に絵の行を足す書き方の決まりは同 spec が要件で決めるので、本仕様の書き方を完了時に申し送る。
  - `manual-shell-guide`（後続）: 「見本の絵は Unlicense で、借りて・変えて・配ってよい」と読者に伝える文は、シェルの説明の章が持つ。本仕様は完了時に申し送る（本仕様が本に足すのは、見本の絵の行だけである）。

## 未確定事項（要件ディスカッションで開発者が決める。確定したものは「【確定】」と記す）

brief の Constraints が「要件定義の初めに決める」と定めた 4 点と、起草中に見つかった事項。各項目は下の要件に【仮定】として織り込んである。決定が変われば、該当する受け入れ基準を書き換える。

| # | 事項 | 本書の仮定 |
| --- | --- | --- |
| Q1 | ライセンス表記の食い違い（ルートの `LICENSE` は MIT の全文のみ。`Cargo.toml`・4 つのクレート README・`tech.md` は「MIT OR Apache-2.0」。一方 `about.hbs` の「pasta 自体は MIT License」・`editors/vscode/package.json` の `"license": "MIT"` は MIT 単独） | **【確定 2026-10-10】** このリポジトリで開発したものは MIT 単独。表記を MIT に揃える（ルートの `Cargo.toml` L12・`pasta_check`/`pasta_dsl`/`pasta_lsp`/`pasta_sample_ghost` の README・`tech.md` L165 の「MIT OR Apache-2.0」を直す）。`LICENSE`（MIT）はそのまま。`LICENSE-APACHE` は作らない。`about.hbs`・`package.json` は既に MIT なので触らない。公開済みの版が持つ「MIT OR Apache-2.0」は取り消せない（過去の版の許諾はそのまま） |
| Q2 | 「Rust で生成・外部素材不要」の方針をやめ、外部素材（コミットした絵）を持つ方針に変える | 変える。絵を描くプログラム・生成 API・CLI・`image`/`imageproc` は取り外す |
| Q3 | 生成元（モデル名・手順）を同梱物に記録するか | **【確定 2026-10-10】** リポジトリ内の記録（クレート README と生成の記録）に置き、配布物 `.nar` には入れない。絵を Unlicense にしたので（Q14）、配布物に表示を入れる必要が無い |
| Q4 | 当たり判定の部位名 | **【確定 2026-10-10】** `Head`・`Face`・`Body` の 3 つ。2 人とも同じ名前の組にする。足元には当たり判定を置かない（足を触ると部位名が空になり、入門ガイドが教える「当たり判定の外では空になる」を実機で確かめられる）。子どもの絵なので、手本の `Bust` は使わない。8 段目の辞書は名前を差し込むだけで分岐しないので、辞書・入門ガイドの章は変えない |
| Q5 | 配布物の大きさの上限の数字 | **【確定 2026-10-10】** 絵 1 枚 ≤ 250 KB・シェルの絵の合計 ≤ 4.5 MB・`.nar` ≤ 7 MB（現行 `.nar` は約 2.0 MB、うち絵は約 50 KB。手本の紹介ページ用の絵は 256×477 px で 168 KB）。切り詰めた予算ではなく、置き間違いや無圧縮の絵を捕まえる歯止めとして置く |
| Q6 | ドリフト（表情で髪・服・輪郭がずれる）対策の方式 | **【確定 2026-10-10】** 1 枚絵 18 枚を保つ（サーフェス番号 1 つに PNG 1 枚）。ずれは作る段階で消す: 基準の絵に、表情ごとの顔の部分だけを貼り合わせてから書き出す。顔以外は、同じ人物の 9 枚で画素まで一致させる。手本のように基準の絵へ顔の部品を重ねる構成は採らない（入門ガイドの準備の章が「`surface0.png`〜`surface8.png` は女の子の立ち絵（表情 9 種）」「同じ番号の画像を差し替えるところから始められる」と教えているため） |
| Q7 | 生成の手順を再現できる形で残すか、成果物だけ残すか | **【確定 2026-10-10】** 手順（モデル・指示文の全文・seed・参照画像・切り抜きと縮小のコマンド）を文書として残す。再実行スクリプトは作らず、コミットした絵を正本とする。加えて、生成の元になる設定画像（2 人それぞれの基準の絵。縮小前）をリポジトリにコミットする（表情を足す・作り直すときの起点になるため）。表情ごとの縮小前の原画はコミットしない |
| Q8 | `image`・`imageproc` の依存を外すか | 外す（このクレートしか使っておらず、ルートの `Cargo.toml`・`Cargo.lock` からも消える） |
| Q9 | 絵のテストの置き場所 | `tests/integration_test.rs` の既存の絵のテストを、コミットした素材を検証するテストに置き換える（別ファイルへ移さない） |
| Q10 | `cargo run -p pasta_sample_ghost`（CLI）と `generate_ghost()` の扱い | 両方とも削除する。クレートは辞書・配布物の検証テストを持つライブラリとして残す |
| Q11 | 立ち絵の寸法 | **【確定 2026-10-10】** キャンバスは手本と同じ 333×500 px（2 人とも同じ）。描く細かさ（頭・瞳の大きさ、線の太さ）は手本に合わせる。人物の背丈は手本の 8〜9 割にして、頭の大きさは保ったまま胴と脚を少し短くする（令嬢の隣で子どもに見え、2 人組が占める面積も抑えられる） |
| Q12 | 生成の費用の上限 | **【確定 2026-10-10】** 試作 3 ドル以内・全体 40 ドル以内。この範囲なら、実装のときに都度の確認なしで生成を進めてよい。上限に近づいたら止めて開発者に相談する。生成サービスの残高は前払いで、開発者が全体の上限と同じ 40 ドルを先に入れる（残高が尽きることも歯止めとして働く）。使った額は生成の記録に残す |
| Q13 | `STAGES.md` 7 段目の「使う文法要素」に `＞ゴースト終了` を足すか（`getting-started-story-guide` からの申し送り） | 本仕様では足さない（ウェーブの約束で `STAGES.md` は部位名の注記と 8 段目の行だけ触る） |
| Q14 | 生成した絵のライセンス表示 | **【確定 2026-10-10】** 絵（シェルの立ち絵と設定画像）だけ Unlicense にする。手本と同じ扱い。入門ガイドは読者に「見本の絵をそのまま借りて進めばよい」と教えるので、借りた読者に表示の義務を残さない（MIT は複製に著作権表示と許諾文を求める）。絵以外（`surfaces.txt`・`descript.txt`・辞書・コード）は MIT のまま。記録とクレート README に「絵は Unlicense・AI 生成・画風の手本はクローディア」と書く。`THIRD_PARTY_LICENSES.txt`（`pasta.dll` の依存の表示）には載せない。ライセンスの説明を本に足すことはしない |
| Q15 | 画風と構図（2026-10-10 の議論で追加。どんな絵にするかは要件で決める） | **【確定 2026-10-10】** ponapalt さんのゴースト「悪役令嬢クローディア」（`https://github.com/ponapalt/claudia`、Unlicense。マニュアルの顔アイコンの出典と同じ）のシェルに画風と構図を揃える。全身・正面・約 3 頭身のちびキャラ、アニメ調（大きな瞳・柔らかい塗り・細い茶色の主線）。「同じ作者が、同じ画風で別の題材のシェルを頼まれて描いた」という想定で方向づける。手本のキャンバスは 333×500 px（2 人とも同じ）。題材（誰を・どんな服で描くか）は Q16 で決める |
| Q16 | 題材とキャラクターの見た目（2026-10-10 の議論で追加） | **【確定 2026-10-10】** パスタ屋さんの見習い 2 人。女の子（本体側）は赤いエプロンドレスの給仕見習いで、栗色のおさげにトマト色のリボン。男の子（相方側）は青いネッカチーフに白いコック服の見習い料理人で、短い黒髪に小さなコック帽。2 人とも人間の子ども（手本の相方はマスコットだが、台詞に合わせて人間の男の子にする）。赤と青の色分けは現行のまま引き継ぐ。手には何も持たせず、両腕を下ろした立ち姿にする（表情の差し替えでずれにくく、当たり判定が単純になる） |

## Requirements

### Requirement 1: 立ち絵の差し替え

**Objective:** 入門者として、hello-pasta の女の子・男の子がイラストの立ち絵で表情を変えてしゃべるのを見たい。自分のゴーストが生きていると実感でき、入門ガイドのスクリーンショットの被写体にもなるため。

#### Acceptance Criteria

1. The サンプルゴースト hello-pasta shall 女の子の立ち絵 9 枚（`surface0.png`〜`surface8.png`）と男の子の立ち絵 9 枚（`surface10.png`〜`surface18.png`）を、画像生成で作ったイラストとして持つ。
2. The サンプルゴースト hello-pasta shall 各キャラクターの 9 枚を、同じキャラクター・同じポーズ・同じ構図・同じ寸法で描き、表情（笑顔・通常・照れ・驚き・泣き・困惑・キラキラ・眠い・怒り）だけが異なるようにする。
3. The サンプルゴースト hello-pasta shall 各立ち絵を、背景が透明な PNG（アルファチャンネル付き）とし、背景の塗り残し・単色の縁取りが無いようにする。
4. The サンプルゴースト hello-pasta shall 女の子と男の子を、一目で区別できる見た目（髪型・服・配色）で描き、2 人の立ち絵を同じ幅・同じ高さにする。女の子は赤いエプロンドレスの給仕見習い（栗色のおさげ・トマト色のリボン）、男の子は青いネッカチーフと白いコック服の見習い料理人（短い黒髪・小さなコック帽）とし、どちらも人間の子どもとして描く（Q16 確定）。
5. The サンプルゴースト hello-pasta shall 立ち絵のキャンバスを 2 人とも 333×500 px とし、頭と瞳の大きさ・線の太さを手本のシェルに合わせ、人物の背丈を手本の令嬢の 8〜9 割にする（Q11 確定）。
6. The サンプルゴースト hello-pasta shall 配布する立ち絵に、目に見える透かし・署名・生成サービスのロゴを含めない。
7. When 表情の切り替えを SSP で確かめたとき, the サンプルゴースト hello-pasta shall 各表情が名前どおりに読み取れる（笑顔と通常、驚きと困惑などが見分けられる）絵を表示する。
8. The サンプルゴースト hello-pasta shall 立ち絵を、「悪役令嬢クローディア」のシェルと同じ画風と構図（全身・正面・約 3 頭身のちびキャラ、アニメ調、大きな瞳、柔らかい塗り、細い主線）で描き、マニュアルの語り手の顔アイコンと並べて同じ作者の絵に見えるようにする（Q15 確定）。
9. The サンプルゴースト hello-pasta shall 2 人とも、手に何も持たず両腕を下ろした立ち姿で描く（Q16 確定）。

### Requirement 2: 表情の切り替えでずれない

**Objective:** 入門者として、台詞の途中で表情が変わっても立ち絵がガタつかないでほしい。表情だけが変わったと分かり、安っぽく見えないため。

#### Acceptance Criteria

1. When 同じキャラクターの表情を別の表情に切り替えたとき, the サンプルゴースト hello-pasta shall 髪・服・体の輪郭が動かないようにする。同じキャラクターの 9 枚は、顔の領域の外の画素（色とアルファ）がすべて一致する（Q6 確定）。
2. The サンプルゴースト hello-pasta shall 表情で変わる領域を顔（目・口・眉・頬）に限り、顔以外の領域を全表情で共通にする。
3. The サンプルゴースト hello-pasta shall サーフェス番号 1 つにつき PNG 1 枚（`surfaceN.png`）の構成を保ち、基準の絵に顔の部品を重ねる構成（複数の element）を採らない。顔の部分を貼り合わせた継ぎ目が見えないようにする（Q6 確定）。
4. The 開発者 shall 本番の生成の前に 1 キャラクター × 3 表情の試作を行い、ずれの大きさと縮小後の縁の品質を実測し、その結果を生成の記録に残す（Q12 確定。試作の費用は 3 ドル以内）。
5. The 開発者 shall 立ち絵の生成にかける費用を全体で 40 ドル以内とし、上限に近づいたら生成を止めて判断を仰ぎ、使った額を生成の記録に残す（Q12 確定）。

### Requirement 3: サーフェス番号と表情の対応の維持

**Objective:** 入門ガイドの読者として、辞書（`dic/03-face.pasta`）に書いたとおりの表情が出てほしい。辞書とマニュアルを書き換えずに済むため。

#### Acceptance Criteria

1. The サンプルゴースト hello-pasta shall 女の子のサーフェス 0〜8 と男の子のサーフェス 10〜18 の番号と表情の対応（0/10 笑顔・1/11 通常・2/12 照れ・3/13 驚き・4/14 泣き・5/15 困惑・6/16 キラキラ・7/17 眠い・8/18 怒り）を変えない。
2. The サンプルゴースト hello-pasta shall `surfaces.txt` に 18 のサーフェス定義（0〜8・10〜18）を持ち、欠番 9 と 19 以上を定義しない。
3. The サンプルゴースト hello-pasta shall `dic/03-face.pasta`・ほかの辞書・`ghost/master/descript.txt` を変えずに、新しい立ち絵で動作する。
4. If 辞書が無い（読み込みに失敗した）状態である, then the サンプルゴースト hello-pasta shall 本仕様で立ち絵の見え方を変えない（この状態の扱いは `boot-surface-without-dic` が決める）。

### Requirement 4: 当たり判定と部位名

**Objective:** 入門ガイドの読者として、8 段目でキャラクターをダブルクリックしたとき、触った部位の名前が台詞に入ってほしい。`＄ｒ４` が空のままだと「触られた部位で台詞を変える」段が成り立たないため。

#### Acceptance Criteria

1. The サンプルゴースト hello-pasta shall `surfaces.txt` の各サーフェス（18 すべて）に、頭の当たり判定を名前 `Head` で定義する（ウェーブの約束により `Head` は変えない）。
2. The サンプルゴースト hello-pasta shall 各サーフェス（18 すべて）に、顔の当たり判定を名前 `Face` で、胴体の当たり判定を名前 `Body` で定義し、部位名を `Head`・`Face`・`Body` の 3 つに限る（Q4 確定）。
3. The サンプルゴースト hello-pasta shall 部位名を、半角英字のみで、8 段目の台詞（「えへへ……＄ｒ４　なでなで？」「いま触られたのは……＄ｒ４　？」）にそのまま差し込んで読んでも不自然でない語にする。
4. The サンプルゴースト hello-pasta shall 同じキャラクターの 9 表情すべてで、同じ部位名の当たり判定を同じ位置・同じ大きさに置く（表情を切り替えても触れる場所が変わらない）。
5. The サンプルゴースト hello-pasta shall 女の子と男の子で同じ部位名の集合を使う。
6. When 利用者が当たり判定の中をダブルクリックしたとき, the サンプルゴースト hello-pasta shall `OnMouseDoubleClick` の `Reference4` にその部位名を受け取り、8 段目の台詞に部位名を表示する（実機の SSP で確認する）。
7. When 利用者が当たり判定の外をダブルクリックしたとき, the サンプルゴースト hello-pasta shall `Reference4` を空のまま受け取り、部位名の無い台詞を表示する（現行の挙動を維持する）。この場合を実機で確かめられるよう、足元には当たり判定を置かない（Q4 確定）。
8. The サンプルゴースト hello-pasta shall 当たり判定の領域を、立ち絵のその部位の描画範囲とおおむね一致させ、立ち絵の透明な領域や他の部位に大きくはみ出さないようにする。
9. When 部位名を確定したとき, the 開発者 shall `STAGES.md` の「8 段目の `Head` は仮の部位名である」の注記を確定した部位名の記述に改め、検証の表（`4=Head`）と整合させる。
10. The 開発者 shall `dic/08-touch.pasta` と `book/src/getting-started/08-touch.md` を変えない（8 段目の辞書は部位名を差し込むだけで、名前ごとに分岐しない。Q4 確定）。

### Requirement 5: 生成物から素材への切り替え

**Objective:** 開発者として、コミットした立ち絵が、テスト・配布スクリプト・リリース CI のどれを実行しても上書きされないでほしい。絵を素材として管理し、丸と三角の絵に戻る事故を無くすため。

#### Acceptance Criteria

1. The リポジトリ shall `crates/pasta_sample_ghost/ghosts/hello-pasta/shell/master/` の立ち絵 18 枚・`surfaces.txt`・`descript.txt` を git で追跡する素材として持つ（`release-ci` が定めた境界のまま。【仮定 Q2】）。
2. The pasta_sample_ghost クレート shall 立ち絵と `surfaces.txt` を生成するプログラム（`image_generator.rs`・`generate_ghost()`・`config_templates.rs` の生成）を持たない（【仮定 Q10】CLI `cargo run -p pasta_sample_ghost` も削除する）。
3. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall `shell/master/` のファイルを書き換えない。
4. When 開発者またはリリース CI が `release.ps1` を実行したとき, the 配布スクリプト shall `shell/master/` のファイルを書き換えず、コミットした絵と `surfaces.txt` をそのまま配布物に詰める。
5. The pasta_sample_ghost クレート shall 画像を描く部品（`image`・`imageproc`）に依存せず、ワークスペースのルートの `Cargo.toml`・`Cargo.lock` からもそれらが消える（【仮定 Q8】）。
6. The pasta_sample_ghost クレート shall `build.rs` の「`cargo run` で生成する」案内と README の「シェル画像を Rust で自動生成（外部素材不要）」「`[gen] cargo run`」の記述を、素材として管理する記述に改める。
7. The pasta_sample_ghost クレート shall README の `scripts/` の説明「Lua ランタイム」を、「利用者向けの説明 1 枚（README.md）だけ。ランタイムは `pasta.dll` の中にある」に直す（brief の指摘）。
8. The リポジトリ shall `.kiro/steering/structure.md`・`tech.md` の「画像生成」「image/imageproc」の記述を、本仕様の完了時に素材の管理に合わせる（完了処理の範囲）。

### Requirement 6: 配布スクリプトとリリース CI の整合

**Objective:** 開発者として、`release.ps1` とリリース CI が絵を生成する段なしで最後まで通り、配布物に新しい絵が入っていてほしい。公開物が手元の見え方と一致するため。

#### Acceptance Criteria

1. The 配布スクリプト（`release.ps1`）shall 「絵を生成する段（現行の Step 2 `cargo run -p pasta_sample_ghost`）」を持たず、段の番号・`-SkipSetup`／`-SkipDllBuild` の説明・進捗表示（`[n/N]`）を新しい段の数に合わせる。
2. The リポジトリ shall `release.bat`・クレート README・`RELEASE.md` の段の説明（「7 ステップ」「1-3. ... generate images」など）を、新しい段の構成に合わせる。
3. When `release.ps1` が完了したとき, the 配布物 `hello-pasta.nar` shall `shell/master/` の立ち絵 18 枚・`surfaces.txt`・`descript.txt` を含む。
4. When リリース CI の「配布物の検査」が実行されたとき, the リリース CI shall `hello-pasta.nar` に `shell/master/surfaces.txt` と立ち絵 18 枚が揃っていることを確かめ、欠けていれば失敗する（【仮定】検査の対象に絵を足す。現行は `install.txt`・`updates.txt`・`descript.txt`・`pasta.dll`・ライセンス表示だけを見る）。
5. The 配布スクリプト shall 既存の段（DLL のビルド・`THIRD_PARTY_LICENSES.txt` の生成・`scripts/` の同期・`pasta_check release`・`pasta.dll.zip`・版の確認）の動作を変えない。
6. While リリースが実行中である, the 開発者 shall `release.ps1` の変更を main に入れない（ウェーブの約束）。

### Requirement 7: 配布物の大きさ

**Objective:** 利用者として、`hello-pasta.nar` のダウンロードとインストールが今までどおり軽くあってほしい。入門者が最初に触る配布物のため。

#### Acceptance Criteria

1. The サンプルゴースト hello-pasta shall シェルの絵（PNG）1 枚を 250 KB 以下、シェルの絵の合計を 4.5 MB 以下にする（Q5 確定）。
2. The 配布物 `hello-pasta.nar` shall 7 MB 以下にする（Q5 確定。現行は約 2.0 MB で、`pasta.dll` が大半を占める）。
3. The 開発者 shall `release.ps1` の完了表示に出る `.nar` の大きさ（MB 表示）と、立ち絵の合計の大きさを、実装の検証で記録する。
4. If 立ち絵の合計が上限を超える, then the 開発者 shall 寸法の縮小または圧縮の見直しで上限内に収め、絵の枚数・表情の種類を減らさない。

### Requirement 8: 生成元・手順・ライセンスの記録

**Objective:** 開発者として、立ち絵をどのモデルでどう作ったか、どのライセンスで配るかを後から迷わず参照したい。再配布の問い合わせや作り直しに答えられるため。

#### Acceptance Criteria

1. The リポジトリ shall 立ち絵の生成の記録（使ったサービスとモデル名、基準画像の作り方、表情ごとの編集の指示、seed と寸法、切り抜きと縮小の後処理、試作の実測の結果、生成した日付、費用）を、`crates/pasta_sample_ghost/` 配下の文書として持つ（Q7 確定。文書として残し、再実行スクリプトは作らない）。
9. The リポジトリ shall 2 人それぞれの設定画像（表情の編集の元にした基準の絵。縮小する前のもの）を、`crates/pasta_sample_ghost/` 配下の素材として git で追跡し、生成の記録からその場所を示す（Q7 確定）。
10. The 配布物 `hello-pasta.nar` shall 設定画像を含めない。
11. The リポジトリ shall 表情ごとの縮小前の原画をコミットしない（正本は縮小後のシェルの絵。作り直すときは設定画像と生成の記録から生成し直す）。
2. The リポジトリ shall 生成の記録に、絵が AI 生成物であることと、著作権の扱いに関する注記（USCO の報告書 Part 2 の見解・日本法での扱いは未確認）を記す。
3. The リポジトリ shall 絵（シェルの立ち絵と設定画像）のライセンスを Unlicense とし、その旨と Unlicense の原文を生成の記録に、その旨をクレート README に記す（Q14 確定。絵以外は MIT のまま）。
4. The リポジトリ shall ライセンスの表記を MIT 単独に揃える。具体的には、ルートの `Cargo.toml` の `license`・`pasta_check`/`pasta_dsl`/`pasta_lsp`/`pasta_sample_ghost` の README・`tech.md` の「MIT OR Apache-2.0」を「MIT」に改め、ルートの `LICENSE`（MIT の全文）と食い違わないようにする（Q1 確定。`LICENSE-APACHE` は作らない）。
5. The リポジトリ shall 生成に使ったモデルの重みのライセンスと、出力の利用条件（透かしの有無を含む）を、生成の記録に出典付きで記す。
6. The 配布物 `hello-pasta.nar` shall 生成の記録もライセンスの表示のファイルも新たに含めない（Q3 確定。記録はリポジトリに置く。絵は Unlicense なので、絵を借りた人に表示の義務が無い）。
7. The 開発者 shall 重みのライセンスが非商用、または出力の扱いが曖昧なモデル（FLUX.1 Kontext [dev] 系など）を使わない。
8. The リポジトリ shall 生成の記録に、画風の手本（「悪役令嬢クローディア」のリポジトリ・参照したコミット・Unlicense）を出典として記し、立ち絵を生成したのが本リポジトリの開発者であることを事実のとおりに書く（「同じ作者に依頼した」は方向づけの想定であり、記録やシェルの作者欄には書かない）。

### Requirement 9: 吹き出しの位置

**Objective:** 利用者として、吹き出しが新しい立ち絵に重ならず、キャラクターの横に自然に出てほしい。立ち絵の寸法が変わると今の位置の指定が合わなくなるため。

#### Acceptance Criteria

1. When 立ち絵の寸法が現行（128×256）から変わったとき, the サンプルゴースト hello-pasta shall `shell/master/descript.txt` の吹き出しの位置（`sakura.balloon.offsetx/offsety`・`kero.balloon.offsetx/offsety`）を新しい寸法に合わせる。
2. When SSP で hello-pasta を起動して台詞を表示したとき, the サンプルゴースト hello-pasta shall 吹き出しが立ち絵の顔に重ならない位置に出る（実機で確認する）。
3. The pasta_sample_ghost クレート shall `descript.txt` の吹き出しの位置を決め打ちで確かめるテスト（`sakura.balloon.offsetx,64` など）を、新しい値に合わせる。
4. The サンプルゴースト hello-pasta shall `shell/master/descript.txt` の吹き出し以外の項目（`charset`・`type`・`name`・`craftman`・`craftmanw`・`seriko.use_self_alpha,1`）を変えない。

### Requirement 10: テストによる素材の検証

**Objective:** 開発者として、コミットした素材が壊れていないこと（枚数・寸法・透過・`surfaces.txt` との対応・当たり判定）を `cargo test` で確かめたい。素材は手で置くので、置き間違いを機械で捕まえるため。

#### Acceptance Criteria

1. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall `shell/master/` に `surface0.png`〜`surface8.png`・`surface10.png`〜`surface18.png` の 18 枚が存在し、`surface9.png` が存在しないことを確かめる。
2. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall シェルの絵が PNG として読め、各サーフェスの基準の絵が 333×500 px で、アルファチャンネルを持ち、四隅が透明であることを確かめる。
3. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall `surfaces.txt` が `charset,UTF-8` で始まり、18 のサーフェス定義を持ち、各定義が同じ番号の PNG を参照し、各定義に `Head` を含む当たり判定があることを確かめる。
4. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall 立ち絵 1 枚と 18 枚の合計の大きさが Requirement 7 の上限内であることを確かめる。
5. The pasta_sample_ghost クレート shall これらの検証を、ネットワークや画像生成サービスに接続せずに行う。
6. The pasta_sample_ghost クレート shall 現行の絵の生成を前提にしたテスト（`test_generated_images_structure`・`test_shell_images`・`test_image_dimensions`・`test_expression_variations`・`lib.rs`・`config_templates.rs`・`image_generator.rs` の単体テスト）を、素材の検証のテストに置き換える（【仮定 Q9】置き場所は `tests/integration_test.rs`）。
7. The リポジトリ shall 本仕様の変更後も、ワークスペースの `cargo test --all` と clippy がクリーンなチェックアウトで成功する。
8. When 開発者が `cargo test -p pasta_sample_ghost` を実行したとき, the pasta_sample_ghost クレート shall 同じキャラクターの 9 枚について、顔の領域の外の画素がすべて一致することを確かめる（Requirement 2 の 1）。

### Requirement 11: 入門ガイドに見本の絵を載せる

**Objective:** 入門ガイドの読者として、写してきたシェルにどんな絵が入っているかを、準備の章で目で見たい。辞書が 1 枚も無い間は SSP に立ち絵が出ないので、本の上で見本を見られると安心できるため（2026-10-10 開発者の指示）。

#### Acceptance Criteria

1. The 入門ガイド shall 準備の章（`book/src/getting-started/setup.md`）の節「シェルの中身」に、女の子と男の子の新しい立ち絵を 1 枚ずつ（`surface0.png` と `surface10.png`）載せる。
2. The 入門ガイド shall 載せる絵を、配布するシェルの絵と同じ内容のファイルとして `book/src/img/` 配下に置き、外部の画像を読み込まない。
3. If 本に置いた絵が、配布するシェルの同じ番号の絵と食い違う, then the リポジトリ shall 自動の検査でそれを失敗として報告する。
4. The 入門ガイド shall 絵の行を、今の本文検査（`book/tools/tutorial-check.mjs` の地の文の検査）に通る書き方（表のセルまたは箇条書き）で書き、絵の内容を伝える代わりの文を付ける。
5. The 開発者 shall `book/tools/` を変えず、節「シェルの中身」の今ある本文と表の行、およびほかの章を変えない。
6. When マニュアルの検査一式（CI と同じ順）を実行したとき, the マニュアル shall すべての検査に通る。
