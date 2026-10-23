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

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**
  - pasta-check-release-asset が、`crates/pasta_sample_ghost/release.ps1`（7 段になった）・ルートの `release.bat`・`crates/pasta_sample_ghost/README.md`・`RELEASE.md`・スキル `pasta-check/SKILL.md` を変えた。この spec の持ち場（`crates/pasta_check/src/` の `release.rs`・`copy.rs`・`nar.rs`・`update_files.rs`・`main.rs` と `crates/pasta_sample_ghost/tests/`）には触れていない。
  - 指摘の場所は 2026-10-19 のまま。作者のフォルダを消す `copy.rs` の `prepare_release_dir`（6〜11 行）、包含を確かめない `release.rs` の段 1〜3（12〜30 行）、`nar.rs` の働かない守り（33 行、呼び出しは 84 行）、`update_files.rs` の `read_dir.flatten()`（170 行）、DLL の名前を取り違える `tests/common/mod.rs`（16〜46 行）。
  - 手元の `release.ps1` は、段 4 で `pasta_check release --target ghosts\hello-pasta --release <ルート>\release\hello-pasta --nar <ルート>\release\hello-pasta.nar` を、段 3 で作った x64 の `pasta_check.exe` で走らせる（234 行）。この組み合わせは、新しく断る 3 つの関係のどれにも当たらないので、リリースは止まらない。設計で、この呼び出しの形を守るテストを置くとよい。
- **触るファイル**: 上の `pasta_check` の 5 ファイル（`release.rs` 720 行・`update_files.rs` 789 行・`main.rs` 393 行・`nar.rs` 385 行・`copy.rs` 232 行）、`crates/pasta_sample_ghost/tests/` の 6 ファイル（`common/mod.rs`・`integration_test.rs`・`dist_src_validation_test.rs`・`touch_shortcuts_test.rs`・`tutorial_stages_test.rs`・`self_deploy_integration_test.rs`）、選択肢の説明 3 か所（`crates/pasta_check/README.md` の release の節 57〜107 行、スキル `.claude/skills/pasta-check/SKILL.md` の 63〜86 行、マニュアル `book/src/debug/validate.md` の「release との関係」202〜221 行）。1,000 行に近いものは無い。
- **規模**: 12〜15 タスク。
- **先に要るもの**
  - 機能の上では無い。0.4.2 の残りと 0.5.0 の spec は、`pasta_check` のこの 5 ファイルを触らない。
  - `tutorial_stages_test.rs` は、失敗の種類を足す 0.5.0 の spec（`call-attribute-filter`・`shell-element-click`）が種類の一覧の確かめを合わせることがある。この spec が触るのはファイルの頭の写し（`copy_master`・`neutralize_debug_env`）で、別の所。
  - 隙間の中では、`ponytail-editor-tooling`（`validate/syntax.rs`・`report.rs`）・`ponytail-core-registry`（`validate/scenes.rs`）と同じクレートの別のファイル。
- **種別**: バグ（`release` の引数の組み合わせで、作者のゴーストのフォルダを丸ごと消す。データが失われる）。隙間の決まりで、最初の空いた所に入れる。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - スキルの編集の計画は今も当たる。pasta-check-release-asset が直したのは、入手の段落（12〜28 行）・段の図（111〜124 行）・フォルダの図（140〜145 行）・hello-pasta の注意（161〜165 行）。この spec が直す選択肢の表と段の説明（63〜86 行）とは別の所。分類器が止める点も変わらない。
  - Scope の「マニュアル（`book/` の pasta_check の章）」は不正確。選択肢の権威は `crates/pasta_check/README.md` の release の節で、マニュアルは `debug/validate.md` の「release との関係」から README へ案内しているだけ。直すのは上の 3 か所。
  - Current State 8 の「`copy_master` と `neutralize_debug_env` が 3 つのファイルにある」は、`neutralize_debug_env` が 3 ファイル（`self_deploy_integration_test.rs`・`touch_shortcuts_test.rs`・`tutorial_stages_test.rs`）、`copy_master` が 2 ファイル（後の 2 つ）。
  - `pasta_check release` を呼ぶのは、今は `release.ps1` の段 4（以前は段 3）。`pasta_check --version` の出力の形は、リリースの版の検査が完全一致で見るので変えない（申し送りのとおり）。

## 申し送り（pasta-skills-release-asset より）

- `release.ps1` は 8 段になった。段 6 が `pasta-skills.zip` を作って検査し、版の確認が段 7、大きさの検査とまとめが段 8。`pasta_check release` を呼ぶのは段 4 のまま。`pasta-check/SKILL.md` の段の図・`release/` の木・配布物の並びは、本 spec が開発者の許可を得て 8 段と `pasta-skills.zip` に合わせた。
- `pasta-check/SKILL.md` は `pasta-skills.zip` に入って作者へ配られるようになった。リポジトリの中の話（`crates/…` のパス・hello-pasta の段）を文字として含んだまま配られるので、中身を直すときはこの点も論点になる。参照先: `.kiro/specs/completed/pasta-skills-release-asset/design.md` の「Decisions / Risks」。
