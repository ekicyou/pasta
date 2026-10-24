# Brief: pest-2-9-4-upgrade

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。`/update`（PR #187）で据え置いた 1 本）。版は未定。要件定義は Opus。

## Problem

pest 2.9.4 に上げると、`pasta_lsp` の WASM のビルドが手元で落ちる。そのため PR #187 では、pest 一式（`pest`・`pest_derive`・`pest_generator`・`pest_meta`）を 2.9.2 に据え置いた。

- pest 2.9.4 は `std` feature で、`stacker` 0.1.25 と `psm` =0.1.24 を必ず引き込む（ターゲットを問わない）。`pest_derive` は pest の既定の features（`std` を含む）を使うので、外せない。
- `psm` のビルドスクリプトは、`wasm32-unknown-unknown` 向けに C とアセンブリを clang でコンパイルする。手元には clang が無く、`ToolNotFound: failed to find tool "clang"` で止まる。
- 据え置いたままだと、pest の直し（再帰の深さの守りなど。stacker を入れた理由と思われる）が入らない。`cargo update` のたびに pest を戻す手間も要る。

## Current State

2026-10-24（main `f472fc27`）に確かめた。

- **依存**: ルートの `Cargo.toml` 20 行 `pest = "2.8.6"`、21 行 `pest_derive = { version = "2.8.6", features = ["grammar-extras"] }`。使うのは `pasta_dsl` と `pasta_shiori`。`Cargo.lock` は 2.9.2。
- **WASM をビルドする所は 3 つ**:
  - 手元の `editors/vscode/scripts/build-wasm.ps1`（wasm-pack。VSIX と拡張のテストが使う）
  - `build.yml` の job `wasm`（`ubuntu-latest`。122 行 `cargo build -p pasta_lsp --target wasm32-unknown-unknown --release`、125 行 `cargo check -p pasta_check --lib --no-default-features --target wasm32-unknown-unknown`、132 行に大きさの検査）
  - `release.yml` の build の job（`windows-latest`。wasm-pack で VSIX の WASM を作る）
- **CI の clang**: GitHub の ubuntu と windows の runner には clang が入っている見込みで、CI だけなら通るかもしれない。確かめていない。手元（開発機）には無い。
- **上流**: 2.9.4 の `Cargo.toml` の `[features]` は `std = ["ucd-trie/std", "dep:stacker", "dep:psm"]`。stacker を外す feature は無い。

## Desired Outcome

- pest 2.9.4 以降に上げても、手元と CI の 3 か所で WASM のビルドが通る。
- `/update` の A（互換の更新）で、pest を毎回据え置かなくて済む。

## Approach

次の案を要件で比べる。

1. **wasm32 向けの C コンパイラを用意する**: 手元に LLVM（clang）を入れ、`CC_wasm32_unknown_unknown` を設定する。CI の 3 か所で clang が使えるかを確かめ、無ければ入れる。開発の前提（`tech.md`・`build-wasm.ps1` の事前の確かめ）を足す。
2. **上流に直してもらう**: pest に、stacker を wasm32 では引き込まない（`cfg(not(target_family = "wasm"))` の依存にする）か、外す feature を足すように頼む。直った版が出るまで据え置く。
3. **WASM では stacker が要らない形にする**: 例えば `pasta_lsp` だけ pest を別の features で使う。`pest_derive` が既定の features を固定しているので、できるかを先に確かめる。

推奨は、2 を出したうえで 1 で進める。1 は pest の外の C の依存が増えても効く。

## Scope

- **In**:
  - pest 一式の版と `Cargo.lock`（下限は `/update` の方針どおり上げない）
  - WASM のビルドの前提（手元の clang・`CC_wasm32_unknown_unknown`・`build-wasm.ps1` の事前の確かめ・CI の 3 か所）
  - `build.yml` の WASM の大きさの検査（psm・stacker で増えるなら）
  - `tech.md` の開発環境の節
- **Out**:
  - 文法（`.pest`）や解析器のコードの変更（`ponytail-dsl-parser` が持つ）
  - pest から別の解析器への移行

## Boundary Candidates

- 依存の版
- WASM のビルドの環境（手元と CI）

## Out of Boundary

- `pasta_lsp` の作り（`ponytail-editor-tooling` が WASM の解析器だけにする）

## Upstream / Downstream

- **Upstream**: 案 2 なら上流の pest の対応
- **Downstream**: 以後の `/update`

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-editor-tooling`（`Cargo.lock` とルートの `Cargo.toml` の席を持つ。`pasta_lsp` と `build-wasm.ps1` まわり。同じウェーブに置かない）
  - `ponytail-dsl-parser`（pest の行の索引を使う。同じウェーブに置かない）
  - `ponytail-manual-ci`・`shiori4-v3-portable`（`build.yml`）

## Constraints

- 手元で `cargo build` などを走らせる前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す。
- 非対話では `npm run package` の `build:wasm` が AllSigned で落ちる。`build-wasm.ps1` を pwsh から走らせる。
- `Cargo.lock` とルートの `Cargo.toml` は、ウェーブの中では 1 本の spec だけが触る。
