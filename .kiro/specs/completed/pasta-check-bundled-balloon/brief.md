# Brief: pasta-check-bundled-balloon

> **ステータス**: 未着手（discovery 完了・2026-10-02）。`/kiro-start pasta-check-bundled-balloon` で開始する。
> 起票元: emo2 開発セッション（ghost_dev）からのブリーフィング。方針（案 A＝pasta_check 側で汎用に直す）はユーザー決定済み。

## Problem

`pasta_check release` は、配布フォルダの全ファイルをゴーストの `updates.txt`（ルートと `ghost/master` の両方）に載せる。そのため、ゴーストと一緒に配るバルーン（同梱バルーン）のファイルまで、ゴースト相対のパスで載ってしまう。

- 発覚: 2026-10-02、areka の alpha の実機ラップ A3r 項目 8。areka 側の欠陥ではなく、配布物の形の問題。
- emo2 の nar（ghost_dev 75e560e、pasta_check 0.3.6 で生成）は、根元に同梱バルーン `emo2-kakukaku/` を持つ。`install.txt` には `balloon.directory,emo2-kakukaku` と `balloon.source.directory,emo2-kakukaku` がある。
- インストール時は、ベースウェアが `emo2-kakukaku/` を `<root>/balloon/emo2-kakukaku/` へ振り分ける。
- ところがゴーストの `updates.txt` には `file,emo2-kakukaku/arrow0.png…` など 20 件が載っている。ネットワーク更新でゴーストを更新すると、`<root>/ghost/emo2/emo2-kakukaku/` に 20 ファイルが誤って置かれる。本来のバルーン（`balloon/emo2-kakukaku`）は更新されない。
- areka の実機記録: `OnUpdate.OnDownloadBegin ["emo2-kakukaku/arrow0.png","0","20","ghost","manual"]` …、`OnUpdateResult ["ghost\u{1}OK\u{1}21"]`。

困るのは、バルーンを同梱して配るゴーストの作者と、その利用者（更新のたびにゴミのフォルダが増え、バルーンは古いまま）。

## Current State

- `crates/pasta_check/src/release.rs`: 準備 → target コピー → `--copy` 上書き → 更新ファイル生成 → nar 作成、の 5 段。
- `crates/pasta_check/src/update_files.rs` の `generate_update_files(root_dir)`: ルートから全ファイルを再帰収集し、ルートの `updates.txt` を書いて `ghost/master/updates.txt` へコピーする。除外は `EXCLUDED_DIRS`（`profile`・`var`）と `EXCLUDED_FILES`（`updates2.dau`・`updates.txt`・`developer_options.txt`、ファイル名一致なのでどの階層でも除外）だけ。
- `install.txt` を読む処理はどこにも無い。同梱バルーンという概念も無い。
- `crates/pasta_check/src/nar.rs`: release フォルダの中身をそのまま nar にする（サブフォルダの `updates.txt` も入る）。
- 形式の仕様書: スキル `.claude/skills/pasta-check/references/updates-txt-spec.md`・`nar-spec.md`（ghost_dev 側にも同じスキルがある）。
- 既存の spec `pasta-check`・`audit-pasta-check` は完了済み。

## Desired Outcome

- `install.txt` が同梱バルーンのフォルダを指定しているとき、そのフォルダ配下はゴーストの `updates.txt`（ルートと `ghost/master` の両方）に 1 行も載らない。
- 同梱バルーンのフォルダ直下（`descript.txt` と同じ階層。ukadoc「ネットワーク更新」の配置どおり）に、バルーン用の `updates.txt` が生成される。
  - パスはバルーンのフォルダからの相対（例 `file,arrow0.png…`）。
  - 形式（文字コード、区切りのバイト値 1、md5、size、date、自分自身を載せないこと）は既存のゴースト用 `updates.txt` と同じ。
- nar には生成したバルーン用 `updates.txt` も入る。nar の構造はこれまでどおり。
- 同梱バルーンの `descript.txt` に `homeurl` が無いときは警告を出す（バルーンのネットワーク更新の前提が欠けているため。ビルドは止めない）。
- 同梱バルーンが無いゴースト（例 pasta-in-windows）の出力は、date を除いて今と完全に同じ。
- 改行変換をしない（md5 が実ファイルと一致する）。過去に CRLF 変換で md5 不一致を起こしている（ghost_dev fb2bbe5）。

### 受け入れ基準の例（ブリーフィングより）

- emo2 で release を実行すると:
  - ゴーストの `updates.txt` に `emo2-kakukaku/` の行が 1 件もない。
  - `release/emo2/emo2/emo2-kakukaku/updates.txt` が生成され、その 20 ファイルの md5 とサイズが実ファイルと一致する。
- pasta-in-windows の `updates.txt` は、date を除いて変化がない。

## Approach

**案 A: pasta_check が `install.txt` を読み、同梱バルーンを汎用に扱う**（ユーザー決定）。

- 同梱バルーンのフォルダは、release フォルダ直下の `install.txt` の `balloon.source.directory` で決める。無ければ `balloon.directory` を使う（ukadoc では `balloon.source.directory` がアーカイブ内のフォルダ名、`balloon.directory` がインストール先の名前。省略時は同じになる）。
- 更新ファイル生成の段で、ゴースト用の収集からそのフォルダを除外し、同じ収集・書き出しの部品でバルーン用の `updates.txt` をそのフォルダ直下に書く。
- nar 作成は release フォルダをそのまま固めるので、生成順（更新ファイル生成 → nar）を守れば変更は要らない見込み。

却下:
- フォルダ内の `descript.txt` の `type,balloon` を走査して同梱バルーンを見つける方式。`install.txt` がベースウェアの振り分けの根拠そのものなので、同じ根拠で判定するほうが食い違わない。
- ghost_dev 側の運用（除外設定や後処理スクリプト）で回避する方式。同梱バルーンを持つすべてのゴーストで同じ問題が起きるため、ツール側で直す（ユーザー決定）。

## Scope

- **In**:
  - release フォルダ直下の `install.txt` の読み取り（`balloon.source.directory`／`balloon.directory`）
  - ゴースト用 `updates.txt`（ルート・`ghost/master`）からの同梱バルーン配下の除外
  - 同梱バルーン直下へのバルーン用 `updates.txt` の生成（nar への封入を含む）
  - 同梱バルーンの `descript.txt` に `homeurl` が無いときの警告
  - 指定されたフォルダが存在しない・不正な値（`..` を含む、絶対パスなど）のときの扱い
  - 同梱バルーンが無いゴーストの出力が変わらないことの回帰テスト
  - スキル `pasta-check` の `references/updates-txt-spec.md`・`nar-spec.md`（と `SKILL.md` の該当箇所）の更新
- **Out**:
  - 同梱シェル・プラグインなど、バルーン以外の同梱物（`install.txt` の他のキー）
  - `delete.txt` の生成・編集（ghost_dev 側で対応済み）
  - nar の構造の変更
  - crates.io へのリリース作業そのもの（`release-workflow` の再実行で行う）

## Boundary Candidates

- `install.txt` の解析（どのキーをどう読むか・文字コード・不正値）
- 更新ファイル生成の分割（ゴースト用とバルーン用で、収集・書き出しの部品を共有する）
- 警告・ログ出力（`homeurl` 欠落、指定フォルダ不在）
- スキルの形式仕様書の更新

## Out of Boundary

- updates.txt の形式そのものの変更（Version 3・CRLF・SOH 区切りは不変）
- ベースウェアのネットワーク更新の挙動
- ghost_dev の release/emo2 の再生成（emo2 開発セッションが areka alpha の署名後に行う）

## Upstream / Downstream

- **Upstream**: 完了済み spec `pasta-check`（release サブコマンド・updates.txt 生成）、`audit-pasta-check`
- **Downstream**:
  - `release-workflow`（新しい pasta_check を crates.io へ出す）
  - emo2 開発セッション（ghost_dev）: 新しい pasta_check のバージョン、またはローカルインストール手順を受け取り、署名後に release/emo2 を再生成する。完了時に連絡する約束がある

## Existing Spec Touchpoints

- **Extends**: なし（`pasta-check` は完了済みのため、新規 spec として起票）
- **Adjacent**: `pasta-check`、`audit-pasta-check`、`release-workflow`

## Constraints

- 同梱バルーンが無いゴーストの出力は、date を除いてバイト単位で不変。
- 改行変換をしない。md5・size は実ファイルのバイト列から計算する。
- 既存の除外規則（`profile`・`var`・`updates.txt` ほか）はバルーン側の収集にも同じく効かせるかを要件で決める（自分自身の `updates.txt` を載せないことは必須）。
- `install.txt` は配布物の一部で、作者が書く入力。値にパス区切りや `..` が入っても release フォルダの外を触らない。
- `install.txt` の文字コード（`charset` 行の有無・Shift_JIS の可能性）の扱いを要件で決める。キーとフォルダ名は ASCII が想定。
- 参考: ukadoc「ネットワーク更新への対応」「ネットワーク」（ファイル構成）、「install.txt」。
