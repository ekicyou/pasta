---
name: update
description: 'pasta の依存ライブラリ（Rust のクレートと、editors/vscode・book の npm パッケージ）を調べ、互換の更新・下限の引き上げ・互換の無い更新（メジャー）の 3 つに分けて、推奨つきで一括で提案する。承認された分を、区分ごと・依存ごとのコミットにして検証し、PR を作って squash で統合する。Use when: /update, 依存の更新, cargo update, 依存ライブラリの最新化, 古い依存の洗い出し。DO NOT USE FOR: リリース（/release を使用。リリースの PR は版の行しか変えない）, 新しい依存を足す（その spec で行う）。'
disable-model-invocation: true
argument-hint: '[--check]（調べて提案するところで終える）'
---

# update — 依存ライブラリの更新と提案

依存を、次の 3 つの区分に分けて扱う。

| 区分 | 変えるもの | 推奨の既定 |
|------|-----------|-----------|
| A. 互換の更新 | `Cargo.lock` と `package-lock.json` だけ（`Cargo.toml`・`package.json` の要求は変えない） | 入れる |
| B. 下限の引き上げ | `Cargo.toml` の要求の版を、互換の範囲の最新へ上げる | 見送る（下の「推奨の決め方」） |
| C. 互換の無い更新 | メジャー（`0.x` は MINOR、`0.0.x` は PATCH）を上げる。`Cargo.toml`・`package.json` と、必要ならコードを直す | 依存ごとに見積もって決める |

## 行う時期

- いちばんよいのは、リリースの後、次の版の作業を始めるときである。上げた依存が、その版の開発のあいだずっと試される。
- リリースの直前に行うなら、このスキルの PR を統合し終えてから `/release` を始める。`/release` の実行中は行わない。リリースの PR は `Cargo.lock` の版の行しか変えない（`release` の段 4 の手順 6 の (2)）ので、依存の更新を混ぜない。
- `Cargo.lock` とルートの `Cargo.toml` は、ウェーブの中では 1 本の spec だけが触る（`kiro-next` の Single-seat resources）。`.kiro/steering/roadmap.md` に、走っている spec がこの 2 つを持つと書いてあれば、そのことを提案の頭に書き、開発者の判断を待つ。

## 実行の約束

1. 作業ブランチで行う。main で実行されたら止まり、ハーネスのワークツリーで実行し直すよう求める。main へ直接 push しない。
2. 段 2 と段 3 では何も変えない。変えるのは、段 3 で承認された分だけである。
3. コミットは、A で 1 つ、B で 1 つ、C は依存ごとに 1 つずつにする。どのコミットも、それだけを戻せば元に戻る形にする。`git add` にはファイルのパスを並べる。
4. 各コミットの前に、段 4 の検証をすべて通す。通らないものをコミットしない。
5. `cargo build`・`cargo test`・`cargo clippy` の前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す（残っていると LuaJIT のビルドが exit 101 で落ちる）。
6. テストが `sample.generated.lua` に改行だけの差分を残したら、`git checkout --` で戻し、コミットに混ぜない。
7. `cargo publish`・`npm publish` を実行しない。版の表記（`[workspace.package]` の `version` など）を変えない。

## 段 1. 始める前の確認

1. 現在のブランチが main でないこと、`git status --porcelain` が空であることを確かめる。どちらかが違えば止まる。
2. `git fetch origin` の後、`git merge origin/main` で main を取り込む。衝突したら `git merge --abort` して止まる。
3. npm の 2 つのフォルダは、`node_modules` が無いことがある。無いと `npm outdated` は何も出さずに終わる（古い依存が無いように見える）ので、調べる前に入れる。

   ```bash
   npm --prefix editors/vscode ci
   npm --prefix book ci
   ```

## 段 2. 調べる（何も変えない）

次のコマンドを実行し、出力を区分に振り分ける。

| コマンド | 分かること |
|----------|-----------|
| `cargo update --dry-run --verbose` | A（`Updating`・`Adding` の行）。最後の `behind latest` の行は、直接の依存の要求に止められている間接の依存（C の候補を上げれば動くことがある） |
| `cargo upgrade --dry-run --incompatible` | 直接の依存ごとの `old req`・`compatible`・`latest`。`latest` が `compatible` と同じ行は B の候補、違う行は C の候補 |
| `cargo deny check` | 勧告・ライセンス・取得元。勧告に当たった依存は、どの区分でも「入れる」を推奨にする |
| `npm --prefix editors/vscode outdated` と `npm --prefix book outdated` | `Wanted` が `Current` と違えば A、`Latest` が `Wanted` と違えば C |
| `npm --prefix editors/vscode audit` と `npm --prefix book audit` | npm 側の勧告 |

`cargo upgrade` と `cargo deny` が無ければ、`cargo install cargo-edit cargo-deny` を案内して止まる。

`Adding` の行（新しく入る間接の依存）は、どの直接の依存が引き込んだかを `cargo tree -i <名前>` で調べて提案に書く（`cargo update` の後でないと出ないときは、段 4 の A の後に調べる）。

### C の候補の見積もり

C の候補ごとに、次の 3 つを調べる。

1. **変更履歴**: 今の版から最新までの破壊的変更。crates.io の `repository` の GitHub の Releases か `CHANGELOG.md` を読む（crates.io の API は `curl -A` で User-Agent をつける）。npm は `npm view <名前> repository.url` から同じように読む。
2. **pasta での使い方**: その依存を使っているファイルの数と場所（`Grep` で `use <名前>::` や `<名前>::`、npm は `import`・`require`）。破壊的変更が、その使い方に当たるかどうか。
3. **据え置きの理由**: `.kiro/steering/tech.md` の「主要依存関係」、`Cargo.toml` のコメント、記憶に、今の版に留めている理由が書いてあれば、それを示す（たとえば mlua は LuaJIT のビルドと `coroutine.close` の扱いに関わる）。

## 段 3. 一括で提案する

すべての候補を、1 回で、推奨つきで示す。1 つずつ尋ねない。`AskUserQuestion` を使わない。

```
依存の提案（base: <origin/main の短い SHA>）

A. 互換の更新 — 推奨: 入れる
  Rust: <件数> 件（<目立つものを 5 つまで: 名前 今→新>）
  新しく入る依存: <名前（引き込んだ依存）> / なし
  npm: editors/vscode <件数> 件・book <件数> 件

B. 下限の引き上げ — 推奨: <入れる / 見送る>
  <名前>  <old req> → <new req>  <推奨の理由>

C. 互換の無い更新
  1. <名前>  <今> → <最新>  推奨: <入れる / 起票 / 見送る>
     破壊的変更: <pasta に当たるものを 1〜3 行。当たらなければ「当たらない」>
     影響: <触るファイルの数と場所>
     例: <直す箇所が 1 つでもあれば、変更前と変更後のコードを 1 か所>
  2. ...

勧告: <名前と勧告の ID。無ければ「なし」>
席: <Cargo.lock を持つ spec が走っていれば、その名前。無ければ「なし」>

承認: 「推奨どおり」か、番号で指定（例: A と C1 だけ）
```

`--check` で実行されたときは、ここで終える。

### 推奨の決め方

- **A**: 入れる。新しく入る依存が `cargo deny check` に通らないときは、その依存を引き込んだ更新だけを `cargo update -p <名前> --precise <今の版>` で据え置く案にする。
- **B**: 見送る。`tech.md` の方針は「依存ライブラリ: メジャーバージョン指定」で、下限を上げると、pasta のクレートを使う側の選べる版が狭まる。上げるのは次のときだけである: 勧告の修正版を必須にしたい／pasta が新しい版で入った API を使っている（`Cargo.lock` の版でしか試していないため、使っていれば下限が嘘になる）。
- **C**: 次の順に当て、最初に当てはまったものにする。
  1. 勧告の修正が新しいメジャーにしか無い → 入れる。
  2. 破壊的変更が pasta の使い方に当たらないか、直しが数ファイルで済む → 入れる（この PR で、依存ごとのコミット）。
  3. 据え置きの理由が書いてあり、その理由がまだ当てはまる → 見送る。理由を書く。
  4. 直しが公開 API・DSL の挙動・Lua ランタイムの土台に及ぶ → `/kiro-discovery` での起票を推奨にする。何を調べればよいかを 1 行添える。

## 段 4. 入れる（承認の後）

A → B → C の順に行う。区分（C は依存）ごとに、変える → 検証 → コミットする。

### 変える

| 区分 | コマンド |
|------|---------|
| A | `cargo update`、`npm --prefix editors/vscode update`、`npm --prefix book update`。続けて、勧告の残ったフォルダだけ `npm --prefix <フォルダ> audit fix`（`--force` はつけない。互換の無い更新になるので、要るなら C で扱う） |
| B | `cargo upgrade -p <名前>` |
| C（Rust） | `cargo upgrade -p <名前> --incompatible` の後、コードを直す |
| C（npm） | `npm --prefix <フォルダ> install <名前>@latest --save-dev` の後、コードを直す |

C で版を上げたら、`.kiro/steering/tech.md` の「主要依存関係」の版の表記も合わせる。

### 検証

変えたものに応じて、次をすべて通す。

| 変えたもの | 検証 |
|-----------|------|
| `Cargo.lock`・`Cargo.toml`・Rust のコード | `cargo test --all --locked --target x86_64-pc-windows-msvc`、`cargo test --all --locked --target i686-pc-windows-msvc`（どちらも時間がかかるので、バックグラウンドで走らせる）、`cargo clippy --all-targets --workspace --target x86_64-pc-windows-msvc -- -D warnings`、`cargo deny check` |
| `Cargo.lock` の `wasm-bindgen` 系の版 | 上に加えて `pwsh -NoProfile -File editors/vscode/scripts/build-wasm.ps1 -Release` |
| `editors/vscode` | `npm --prefix editors/vscode test`、`npm --prefix editors/vscode run lint`、`npm --prefix editors/vscode run compile` |
| `book` | `for t in $(find book/tools -name '*-test.mjs' -not -path '*/node_modules/*' \| sort); do node "$t" \|\| break; done`（manual.yml の「Book tools self-tests」と同じ） |

### 失敗したとき

- **A**: どの依存の更新で壊れたかを、`cargo update -p <名前> --precise <今の版>` で 1 つずつ戻して突き止める。その依存だけを据え置いて A を入れ、据え置いた依存と原因を報告に書く。原因が pasta 側の不具合なら、ここで直す。
- **C**: 段 3 の見積もりより直しが大きくなったら、その依存の変更を `git restore` で捨て（コミット前なので消えるのはその依存の分だけ）、起票を推奨にして報告に回す。ほかの依存は続ける。

### コミット

件名は次の形にする。

- A: `chore(deps): 互換の範囲で依存を更新する`
- B: `chore(deps): <名前> の下限を <版> に上げる`
- C: `chore(deps): <名前> を <版> に上げる`

本文に、上げた依存の一覧（A は件数と目立つもの）を書く。

## 段 5. PR を作って統合する

1. 作業ブランチを push し、`gh pr create` で PR を作る。題名は `chore(deps): 依存の更新（<YYYY-MM-DD>）`、本文は段 3 の提案のうち入れた分・見送った分・起票を勧める分。
2. CI を待たずに `gh pr merge --squash --delete-branch` で統合する。成否はこのコマンドの結果だけで判定する（`main already used by worktree` の警告は失敗ではない）。

## 段 6. 報告する

```
依存の更新: <統合した / 提案だけ（--check）>（PR: <URL>）
- 入れた: A <件数> 件・B <名前…>・C <名前 今→新…>
- 据え置いた: <名前と理由。無ければ「なし」>
- 起票を勧める: <名前と、調べること。無ければ「なし」>
```
