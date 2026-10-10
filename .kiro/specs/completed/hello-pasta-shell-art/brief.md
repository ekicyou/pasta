# Brief: hello-pasta-shell-art

## Problem
入門ガイドの題材であるサンプルゴースト hello-pasta の見た目は、`image_generator.rs` が描くピクトグラム（赤・青の丸い頭と三角の胴、128x256）である。表情の違いは目の記号だけで、初心者が「自分のゴーストが表情を変えてしゃべった」と実感しにくい。物語のある入門ガイド（`getting-started-story-guide`）の主役としても、スクリーンショットとしても弱い。

## Current State
- シェル画像 `surface0-8.png`（女の子）・`surface10-18.png`（男の子）と `surfaces.txt` は、`crates/pasta_sample_ghost` の `generate_ghost()`（`cargo run`）が生成する。
- README は「自己完結型: シェル画像を Rust で自動生成（外部素材不要）」を方針として掲げ、画像を生成物として扱っている。
- 表情 9 種（笑顔・通常・照れ・驚き・泣き・困惑・キラキラ・眠い・怒り）とサーフェス番号の対応は `actors.pasta` が持つ。

## Desired Outcome
- 女の子・男の子の立ち絵を、fal.ai の画像生成で作ったイラストに置き換える。表情 9 種ずつ、同じポーズ・構図・寸法で、透過 PNG とする。
- 表情を切り替えても、髪・服・輪郭がずれない（サーフェスがきれいに入れ替わる）。
- 画像は生成物ではなく、リポジトリにコミットした素材として扱う。`generate_ghost()` が素材を上書きしない。
- 生成元（モデル名・手順）を記録し、ライセンス表記と食い違わない。
- `surfaces.txt` に当たり判定（collision）を足し、部位名（例: `Head`・`Face`・`Bust`）を確定する。`hello-pasta-tutorial-stages` の 8 段目が、ダブルクリックの `Reference4`（`＄ｒ４`）で触られた部位を台詞に使うため（2026-10-08 申し送り。現行シェルには当たり判定が無く、`＄ｒ４` が空になる）。部位名を確定・変更したら同 spec に知らせる。

## Approach
2026-10-06 の実現性調査の結論（有料ジョブは未実行）。
- 本線は `fal-ai/qwen-image-edit-2511` とする。重みは Apache-2.0 で、透かしの記述は見つかっていない（未確認）。seed とサイズを固定できる。品質の比較用に `fal-ai/nano-banana-pro/edit` を使う（全出力に SynthID が入る）。
- 基準画像を 1 枚作り、「表情だけ変える」編集を 9 回かける。
- 単色の背景で生成し、`fal-ai/birefnet/v2`（Matting）で切り抜いてから縮小する。
- 表情を変えると髪・服・輪郭が数ピクセル動く（ドリフト）見込みがある。対策は 2 案のどちらかを設計で選ぶ。
  - 顔以外を基準画像から合成し直す。
  - `surfaces.txt` の element で胴体と顔を重ねる構成にする。
- 本番の前に、1 キャラ × 3 表情の試作（$1 未満）で、ドリフトと縮小後の縁の品質を実測する。
- 費用は全体で数ドル〜十数ドルの見込み。

## Scope
- **In**: 立ち絵 18 枚（または胴体＋顔パーツの構成）の生成・後処理・コミット、`surfaces.txt` の更新、`image_generator.rs`・`generate_ghost()`・README・`release.ps1` の整理（素材の扱いへの切り替え）、生成元の記録、関連テストの更新
- **Out**: 表情の種類やサーフェス番号の変更（`actors.pasta` の対応は維持する）、バルーンの作り直し、辞書の書き換え（`hello-pasta-tutorial-stages` が持つ）、マニュアル本文（`getting-started-story-guide` が持つ）

## Boundary Candidates
- 画像の生成・後処理の手順（再現できる形で残すか、成果物だけ残すか）
- `pasta_sample_ghost` クレートの、生成物から素材への切り替え

## Out of Boundary
- 入門ガイドのスクリーンショットの撮影と掲載（`getting-started-story-guide`）
- 辞書の内容と段階辞書（`hello-pasta-tutorial-stages`）

## Upstream / Downstream
- **Upstream**: fal.ai（fal MCP）、`crates/pasta_sample_ghost`、`release.ps1`（ゴーストのリリース手順。`release-workflow` の常駐 spec）
- **Downstream**: `getting-started-story-guide`（スクリーンショットに新しいシェルを使う）

## Existing Spec Touchpoints
- **Extends**: なし（hello-pasta のシェル生成は初期開発の spec の成果物）
- **Adjacent**: `release-ci`（先行する。同じ `crates/pasta_sample_ghost/release.ps1` を触り、生成物の git 追跡をやめる。本 spec は画像を「生成物」から「追跡する素材」に移すので、`release-ci` が決めた追跡の境界と `.gitignore` に合わせる）、`release-workflow`（ゴーストの `.nar` に画像を詰める手順）、`hello-pasta-tutorial-stages`（同じ `pasta_sample_ghost` クレートの辞書とテストを触る。画像と辞書でファイルは分かれる）

## Constraints
- 要件定義の初めに、次の 3 点を決める。
  - ライセンス表記の食い違い: ルートの `LICENSE` は MIT の全文だけで、`Cargo.toml` と README は「MIT OR Apache-2.0」と書いている。`LICENSE-APACHE` は存在しない。
  - 「Rust で生成・外部素材不要」の方針をやめて、外部素材を持つ方針に変える。
  - 生成元とモデル名を同梱物（README など）に記録するかどうか。
- 生成した画像には、著作権が認められない可能性が高い（USCO の AI 報告書 Part 2）。再配布の妨げにはならない。日本法での扱いは未確認。
- FLUX.1 Kontext [dev] 系は、重みのライセンスが非商用で、出力の扱いも曖昧なので使わない。
- 配布物の `.nar` のサイズが過大にならないこと（立ち絵は 150〜300px 幅程度を想定する）。

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: 無い。画像は `image_generator.rs`（572 行）が描き、`generate_ghost()`（`lib.rs`）が `ghosts/hello-pasta/shell/master/` に 18 枚と `surfaces.txt` を書く。`release.ps1` の Step 2 が `cargo run -p pasta_sample_ghost` で毎回上書きする。
- **触るファイル**: `crates/pasta_sample_ghost/` の `src/image_generator.rs`（削るか縮める）・`src/lib.rs`・`src/main.rs`・`src/config_templates.rs`（`surfaces.txt`）・`build.rs`・`README.md`・`release.ps1`（Step 2）・`ghosts/hello-pasta/shell/master/`（png 18 枚・`surfaces.txt`・`descript.txt` のバルーン位置）・`tests/integration_test.rs`（画像のテスト。14〜65 行・367〜385 行）、生成手順の記録（新規）、場合によりライセンスのファイル（新規）。1,000 行に近いファイルは無い。
- **規模**: 10〜12 タスク（試作・基準画像・表情 9 種 × 2 人・切り抜き・ずれの対策・`surfaces.txt`・クレートの整理・`release.ps1`・README と出典・テスト）。
- **先に要るもの**: `release-ci`（`release.ps1`・`.gitignore`・追跡の境界）。
- **ファイルの重なり**: `hello-pasta-tutorial-stages` と `tests/integration_test.rs` を共有する（本 spec は画像のテスト、相手は辞書のテスト 269〜365 行）。どちらもウェーブ 1 なので、運用ルールでは並走できない。関数が分かれているので rebase で済む見込みだが、どちらを先にするか、画像のテストを別のファイルへ移すかを決める。
- **種別**: 機能（見た目の差し替えと、生成物から素材への切り替え）。
- **要件定義のモデル**: Fable（要件の初めに開発者が決める 3 点がある。ライセンス表記・外部素材の方針・生成元の記録）。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - `shell/master/descript.txt` のバルーン位置（`sakura.balloon.offsetx,64` など）は、幅 128px の今の絵が前提。寸法が変われば直す。
  - `book/src/getting-started/first-ghost.md`（60 行目あたり）がこの `descript.txt` を ```text で転記している。`tutorial-check.mjs` は ```pasta しか照合しないので、食い違っても CI は気づかない（`getting-started-story-guide` の書き直しで拾う）。
  - `release-ci` の起票文の「追跡を外す一覧」にシェル画像は入っていない。`release-ci` の要件で「シェル画像は追跡したまま」と明記してもらうと、本 spec が戻す手間が要らない。
  - 胴体と顔を重ねる構成にするなら、`lib.rs` のテスト（生成ファイル数 19）と `config_templates.rs` のテスト（18 ブロック）も直す。

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: 先に要る 2 本（`release-ci`・`hello-pasta-tutorial-stages`）は、どちらも main に入った。
  - `release-ci` は、絵を作る段に手を付けていない。配布物を作るスクリプト（`crates/pasta_sample_ghost/release.ps1`）の 2 番目の段は、今も `cargo run -p pasta_sample_ghost` で絵 18 枚と `surfaces.txt` を毎回上書きする。リリースの自動実行（`.github/workflows/release.yml`）もこのスクリプトを呼ぶ。この段を外さないと、コミットした絵が配布物を作るたびに丸と三角の絵へ戻る。
  - `.gitignore` に「シェルの画像は追跡を続ける」と書かれた。`shell/master/` の 20 ファイル（絵 18 枚・`surfaces.txt`・`descript.txt`）は今も git に入っている。追跡を戻す作業は要らない。
  - 表情とサーフェス番号の対応は、`actors.pasta` から 3 段目の辞書（`ghosts/hello-pasta/ghost/master/dic/03-face.pasta`）へ移った。中身は同じ（1 人 9 表情、0〜8 と 10〜18）。emo2 との切り替えの作例（`07-greeting.pasta`）は、表情もサーフェス番号も増やしていない。描くのは 18 枚のまま。
  - 辞書のテストは別のファイル（`tests/tutorial_stages_test.rs`）へ出た。`tests/integration_test.rs`（306 行）を触る未完了の spec は、本 spec だけになった。
- **触るファイル**: `crates/pasta_sample_ghost/` の次のもの。1,000 行に近いファイルは無い。
  - 絵を描くプログラム `src/image_generator.rs`（572 行。消す）、`src/lib.rs`・`src/main.rs`・`src/config_templates.rs`・`build.rs`・`Cargo.toml`。絵を描く部品 `image`・`imageproc` はこのクレートしか使っていないので、ルートの `Cargo.toml`（62〜63 行）と `Cargo.lock` からも外れる。
  - `README.md`、`release.ps1`（2 番目の段と段の番号）、ルートの `release.bat` の説明、段階表 `STAGES.md`（部位名が「仮」という注記）。
  - `ghosts/hello-pasta/shell/master/`（絵 18 枚・`surfaces.txt`・`descript.txt`）。
  - `tests/integration_test.rs`（絵のテストは 14〜64 行と 269〜286 行。吹き出しの位置を決め打ちで確かめる 251〜266 行も）。前回の再測定の行番号は古い。
  - 新しく作るもの: 生成の手順と出どころの記録、場合によりライセンスのファイル。
- **規模**: 12〜14 タスク（前回の 10〜12 に、当たり判定と部位名、絵を描く部品の取り外し、配布物の中身と大きさの確認を足した）。
- **先に要るもの**: 無い。ファイルが重なりうる未完了の spec は `getting-started-story-guide` だけで、条件つき。8 段目の辞書（`dic/08-touch.pasta`）を変えると、入門ガイド（`book/src/getting-started/first-ghost.md`）の同じ作例も同時に直さないと照合が落ちる。頭の部位名を今の `Head` のまま確定し、辞書と `book/` を触らなければ、ガイドの本文と同じ時期に進められる。`release-workflow` の手順の書き換えとはファイルが重ならないが、リリースの実行中は `release.ps1` を変えない。
- **種別**: 機能（見た目の差し替え、生成物から素材への切り替え、当たり判定の追加）。
- **要件定義のモデル**: Fable（開発者が決めることが、ライセンス表記・外部素材の方針・生成元の記録の 3 点に加えて、部位名がある）。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - Current State の「対応は `actors.pasta` が持つ」は古い。今は `03-face.pasta`。
  - 「部位名を確定・変更したら同 spec に知らせる」は、相手の spec が完了したので成り立たない。名前を変えるなら、本 spec が辞書・`STAGES.md` の検証の表（`4=Head`）・入門ガイドの作例を直す。
  - 8 段目の台詞は、部位名をそのまま声に出す（「えへへ……Head なでなで？」）。頭以外に付けた名前もそのまま読まれる。台詞として読んでおかしくない名前にする。
  - 今の絵は 1 枚 2〜3KB（18 枚で約 50KB）。新しい絵は桁が上がるので、配布物の大きさの上限を要件で数字にする。
  - `README.md` 72 行目は `scripts/` を「Lua ランタイム」と書くが、置くのは利用者向けの説明 1 枚だけ（ランタイムは `pasta.dll` の中にある）。README を書き直すときに直す。

## 申し送り（getting-started-story-guide より・2026-10-10）

本文の spec が完了した。参照先は `.kiro/specs/completed/getting-started-story-guide/` の `design.md` と、`tasks.md` の `## Implementation Notes`。

- **`first-ghost.md` は消えた**: 上の節の `book/src/getting-started/first-ghost.md` は、次のとおり読み替える。`descript.txt` の転記は `setup.md` にある（配布版から `homeurl` の行を落としている。`tutorial-check.mjs` が照合するのは今も ```pasta だけ）。8 段目の作例は `08-touch.md` にあり、`dic/08-touch.pasta` の全体と逐語で照合される。辞書を変えるなら同じ章も直す。
- **入門ガイドはシェルを「hello-pasta からフォルダごと写す」と教える**: `setup.md` は、読者に `shell/master/` を丸ごと写させ、中のファイルには手を入れさせない。シェルのファイルの名前や数が変わっても章は変わらないが、`pasta.dll`・`THIRD_PARTY_LICENSES.txt` の置き場所（`ghost/master/`）を動かすと章が外れる。
- **`STAGES.md` を直した**: 「確かめるための道具」の節（6 段目は道具を使わず実際の正時を待つ。開発者用機能を有効にするのは 7 段目が初出）と、13 段目の「7 段目で有効にした」。部位名の注記と表の内容は変えていない。
- **`STAGES.md` の 7 段目の「使う文法要素」に `＞ゴースト終了` が無い**: 辞書（`dic/07-greeting.pasta`）は使っていて、章は辞書のコメントの範囲で説明している。`STAGES.md` を触るついでに、表に足すかを決める。
- **8 段目の実機の見え方**: 今のシェルでは、部位名が空のときの台詞だけが出る。当たり判定を足すと、初めて部位名つきの台詞（「わっ！……え、いまのHead？」）が実機で出るようになる。
- **辞書が無い間は立ち絵が出ない**: 立ち絵は、最初の台詞と一緒に出る（シェルの問題ではない。別に起票した）。
