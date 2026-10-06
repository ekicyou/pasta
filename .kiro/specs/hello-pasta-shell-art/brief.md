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
