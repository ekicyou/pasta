# Brief: ponytail-check-release

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域に `/ponytail:ponytail-audit` をやり直す。
  - 領域：`crates/pasta_check` の `release`・`copy`・`nar`・`update_files`・`main`、`crates/pasta_sample_ghost`。
  - 対象外：`validate/` の `syntax.rs`・`report.rs`（`ponytail-editor-tooling`）と `scenes.rs`（`ponytail-core-registry`）。
- 控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

`pasta_check release` を使うゴーストの作者が困る。`--target`・`--release`・`--copy`・`--nar` の関係を確かめていないので、組み合わせによって次のことが起きる。

- 作者のゴーストのフォルダを、丸ごと消す。
- 自分自身を、無限に入れ子に写す。
- 書きかけの .nar を、その .nar の中に入れたまま「成功」と報告する。これは「壊れた配布物はエラーで止める」という方針に反する。

## Current State

2026-10-19 の監査で確かめた。`cargo test -p pasta_check` の 232 本は通り、clippy も 0 件。1〜3 の壊れるケースは実際には走らせておらず、コードの道筋から推定した。

1. **`release` が作者のフォルダを消す**（Must fix）
   - 場所：`src/copy.rs` L6-11 の `prepare_release_dir`（`release.rs` L12-16 から呼ばれる）。
   - `--release` を `--target` と比べずに `remove_dir_all` する。
   - `--target C:\dev\myghost --release C:\dev` や `--target . --release .` で、開発のフォルダごと消える。これを確かめるテストは無い。
2. **`--release` が `--target`（または `--copy`）の中にあると、無限の入れ子になる**
   - 場所：`copy.rs` L35-60、`release.rs` L19-21。
   - `G\out\out\out…` と続き、パスが長すぎて失敗する。作者のフォルダに、深いごみの木が残る。
3. **`--nar` が `--release` の中にあると、.nar が自分を含む**
   - 場所：`nar.rs` L9-27。
   - 例：`--release out --nar out\g.nar`。
4. **サンプルゴーストのテストの道具が、作られない名前の DLL を探す**
   - 場所：`crates/pasta_sample_ghost/tests/common/mod.rs` L16-46、`tests/integration_test.rs` L216-234。
   - `pasta_shiori.dll` を探すが、lib の名前は `pasta` なので、作られるのは `pasta.dll`。そのため、そのテストは決して落ちない。
5. **`updates.txt` を集めるときに、読み取りの誤りを握りつぶす**
   - 場所：`update_files.rs` L175-180。
6. **決して働かない守り**
   - 場所：`copy.rs` L15-27・L50-51 の `ensure_within`、`nar.rs` L32-41・L80-81 の `ensure_no_parent_component`。
   - `ReadDir` は、`..` も区切り文字も返さない。
   - この 2 つは、`audit-pasta-check` の Req 1.3/1.4 から来ている。消すことは開発者が了承した（2026-10-19）。
7. **`non_unicode_name` の写しが 2 つ**
   - 場所：`nar.rs` L44-49、`update_files.rs` L147-153。
8. **サンプルゴーストのテストの写し**
   - `copy_master` と `#[ctor] neutralize_debug_env` が、3 つのファイルにある。
   - `dist_src_validation_test.rs` は、`integration_test.rs` と重複している。

## Desired Outcome

- 危ない組み合わせは、ファイルに触れる前に英語の誤りで止まる。
  - `--release` が `--target` または `--copy` と同じか、それを中に含む。
  - `--release` が `--target` または `--copy` の中にある。
  - `--nar` が `--release` の中にある。
- 読み取りの誤りは、起きた段で報告される。
- 働かない守りと写しが無くなり、本当に起きる場合を守るテストがある。

## Approach

監査の最小の直しを採る。

- 段 1 の前に、`--target` と `--copy` を `canonicalize` する。`--release` も、在れば `canonicalize` する。
  - `canonicalize` を使うのは、8.3 の短い名前・大文字と小文字・junction を解くため。ここではどのパスも存在するので、`std::path::absolute` でなくてよい。
- 包含の関係を比べ、`--nar` は `absolute` で比べる。

## Scope

- **In**: 上の 1〜8 と、ギャップ分析で増えた同じ領域の指摘。選択肢の表（`.claude/skills/pasta-check/SKILL.md`。下の Constraints を参照）。
- **Out**:
  - `validate/syntax.rs`・`validate/report.rs` と `FindingKind::code()` の削除（`ponytail-editor-tooling`）
  - `validate/scenes.rs`（`ponytail-core-registry`）

## Boundary Candidates

- パスの関係の守り（release）
- 収集の誤りの扱い（update_files）
- 守りと写しの整理（copy・nar）
- サンプルゴーストのテストの整理

## Out of Boundary

- `validate` の規則（pasta.dll と同じ規則を保つ）
- リリースの CI（`ponytail-manual-ci`）

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**:
  - ゴーストの作者の `release.bat`・`release.ps1`
  - VSCode 拡張（validate だけを使う）

## Existing Spec Touchpoints

- **Extends**: `audit-pasta-check`（完了。Req 1.3/1.4 の守りを、本当に効く守りに置き換える）
- **Adjacent**:
  - `ponytail-editor-tooling`
  - `ponytail-core-registry`（どちらも `pasta_check` の別のファイルを触る）

## Constraints

- `.claude/skills/pasta-check/SKILL.md` の編集は、分類器が Self-Modification として止める。その場面になったら、回避策を探さずに、入った分と残りを示して、開発者の許可を求める。
- 挙動を変える直しは、同じ変更でマニュアル（`book/` の pasta_check の章）を直し、`gen-skill-refs` を走らせる。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。消す・壊すケースは、一時フォルダで再現してから直す。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_check -p pasta_sample_ghost`。cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。
- パスの形に依存するバグは、CI でしか再現しないことがある（GitHub のランナーの `%TEMP%` は `RUNNER~1` という 8.3 の短い名前）。テストは、短い名前が混じっても通るように書く。

## 申し送り（pasta-check-release-asset より）

2026-10-22 に main に入った変更。

- `release.ps1` は 7 段になった。段 3 が x64 の `pasta_check.exe` をビルドし、`THIRD_PARTY_LICENSES.txt`（`about-pasta_check.hbs` から生成）と合わせて `release\pasta_check.zip` を作って検査する。段 4 の `pasta_check release` は、`cargo run -p pasta_check` ではなく、その exe（`target\x86_64-pc-windows-msvc\release\pasta_check.exe`）で走る。引数と失敗の扱いは変えていない。
- `pasta_check` の `--version` の出力の形を変えると、リリース CI の版の検査（`pasta_check <タグの版>` と完全一致）が落ちる。`pasta_check` の依存に許可に無いライセンスが入ると、段 3 のライセンス表示の生成が止まる（`about.toml` の `accepted`）。
- 参照: `.kiro/specs/completed/pasta-check-release-asset/design.md` の CheckZipStep と Revalidation Triggers。
