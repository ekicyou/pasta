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
