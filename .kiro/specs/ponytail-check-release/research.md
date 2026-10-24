# ギャップ分析: ponytail-check-release

## Summary

- **Feature**: `ponytail-check-release`
- **Discovery Scope**: Extension（既存の `pasta_check release` の段 1 の前に引数の確かめを足し、同じ領域の監査の指摘を片づける）
- **基準**: worktree の HEAD `55da9722`（main の c20ad104 の後、棚卸 #196 まで）。`cargo test -p pasta_check` は全件緑（lib 126・cli 10 ほか）。
- **Key Findings**:
  - brief の 1〜8 は、どれも今も残っている。行番号は 2026-10-24 の再測定のまま（`ensure_within` などは 1 行ずれ）。
  - 1〜3 の壊れるケースを、スクラッチの一時フォルダで実際に再現した。2 は「パスが長すぎて失敗」ではなく、**スタックあふれでプロセスが落ちる**（`Error:` も終了コード 1 も出ない）。
  - 監査のやり直しで、同じ領域の新しい指摘が 11 件（N1〜N11）。いちばん重いのは N1（`--nar` が開発フォルダの中の作者のファイルを指すと、段 1 で消して nar に置き換え、成功と報告する。再現済み）。
  - brief の Approach の「`--release` は在れば `canonicalize`、`--nar` は `absolute` で比べる」は、そのままでは働かない。Windows の `canonicalize` は `\\?\C:\…` の形を返し、`absolute` は `C:\…` を返すので、`starts_with` が決して真にならない（5 の 1 を参照）。まだ無いパスは「在る一番近い親を `canonicalize` して、残りを足す」形で比べる必要がある。
  - 説明の文書は brief の 3 か所のほかに、README の仕様メモ（外す守りの記述）と、スキルの `references/nar-spec.md`（段 1 の説明）にも及ぶ（3 の表の 8.x の行を参照）。`gen-skill-refs` は `debug/validate.md` を元にしないので、走らせても変わらない（`--check` で確かめるだけ）。

---

## 1. 監査のやり直し（2026-10-24）

### 1.1 やり方

- 見方は `/ponytail:ponytail-audit` と同じ「最小で完全な変更」: 死んだコード・写し・握りつぶした誤り・欠けた守り・要らない抽象。
- 領域: `crates/pasta_check/src/` の `release.rs`（720 行）・`copy.rs`（232）・`nar.rs`（385）・`update_files.rs`（789）・`main.rs`（393）と、`crates/pasta_sample_ghost`（`src/` 2 ファイル・`tests/` 7 ファイル）。
- 対象外: `validate/`（`syntax.rs`・`report.rs`・`scenes.rs` を含む）。`balloon.rs`・`validate_cmd.rs` は brief の領域に無いので、`release` から呼ばれる口だけを見た（中身は監査していない）。
- 壊れるケースは、スクラッチ（セッションの scratchpad の下の `repro/`）にゴーストの最小の一式を作り、debug ビルドの `pasta_check.exe` を実際の引数で走らせて確かめた。本物のフォルダには触れていない。確かめた後に `repro/` は消した。

### 1.2 再現の結果

| 記号 | 引数 | 結果 |
| ---- | ---- | ---- |
| A | `--target P\ghost --release P`（P に他のファイルもある） | 段 1 で P を丸ごと消し（ghost も other.txt も）、段 2 で `Error: 指定されたパスが見つかりません。 (os error 3)`。終了コード 1。どのパスの話か出ない |
| B | `--target G --release G` | 段 1 で G を消して空で作り直し、段 2 で 0 件を写し、段 4 で `Error: …\ghost\master を読めません` |
| C | `--target G --release G\out` | 段 2 で `G\out\out\out…` を写し続け、深さ 123 で `thread 'main' has overflowed its stack` で落ちる。G に 247 ファイルのごみの木が残る |
| D | `--release out --nar out\d.nar` | 6 段を終えて `Release complete!`（終了コード 0）。nar に 35 バイトの書きかけの `d.nar` が入っている。`updates.txt` には `d.nar` の行が無い |
| E | 作業フォルダ G で `--target . --release .` | 段 1 で G の中身を全部消し、G 自体は使用中で消せず `Error: … (os error 32)`。G は空になる |
| F | `--release out --nar G\notes.toml`（作者のファイル） | 段 1 で `Removed previous …\notes.toml` と出して消し、段 6 で同じ場所に nar を書く。成功と報告し、notes.toml は ZIP（`PK\x03\x04`）に置き換わる |
| G | `release.ps1` の段 4 の形（`--target S\ghosts\hello --release S\release\hello --nar S\release\hello.nar`） | 今どおり 6 段を終えて nar を作る |

### 1.3 brief の Current State（2026-10-19 の控え）との突き合わせ

| # | 控えの指摘 | 今 | 違い |
| - | ---------- | -- | ---- |
| 1 | `copy.rs` の `prepare_release_dir` が `--release` を `--target` と比べずに消す | **残っている**。`copy.rs` 6〜11 行（`remove_dir_all` は 8 行）、呼び出しは `release.rs` 15 行 | 再現 A・B・E で確認。E（`.` 同士）は、フォルダは残るが中身が消える。比べる相手は `--copy` もある（`--release build --copy build\extra` は上書きフォルダを消す） |
| 2 | `--release` が `--target`（`--copy`）の中にあると無限の入れ子 | **残っている**。`copy.rs` 36〜62 行、呼び出しは `release.rs` 20・27 行 | 再現 C。終わり方は「パスが長すぎる」誤りではなく、スタックあふれでプロセスが落ちる（Rust の std は長いパスを `\\?\` で扱うので、先に再帰が尽きる）。`Error:` の行も終了コード 1 も出ないので、バッチファイルが失敗を読み違える恐れもある |
| 3 | `--nar` が `--release` の中にあると .nar が自分を含む | **残っている**。`nar.rs` 9〜28 行（`File::create` 17 行の後に `read_dir` が書きかけの nar を拾う） | 再現 D。成功と報告する |
| 4 | 見本のゴーストの試験の道具が `pasta_shiori.dll` を探す（lib の名前は `pasta`） | **残っている**。`tests/common/mod.rs` 16〜46 行、`tests/integration_test.rs` 216〜234 行 | 道具を使うのは、その試験 1 本だけ。DLL が無ければ自分で作った文言に「pasta_shiori.dll」が入っているかを見るだけなので、決して落ちない。この道具のために `shell_assets_test.rs` 13〜15 行が `#[allow(dead_code)] mod common;` を置いている（N9） |
| 5 | `updates.txt` を集めるとき、読み取りの誤りを握りつぶす | **残っている**。`update_files.rs` 170 行の `read_dir.flatten()` | 控えより広い。165〜168 行で `read_dir` 自体の失敗も `Ok(())` にしている（N3）。その振る舞いを `test_generate_update_files_nonexistent_root_returns_zero`（359〜366 行）が固定している。205 行の更新日時も握りつぶす（N4） |
| 6 | 決して働かない守り（`ensure_within`・`ensure_no_parent_component`） | **残っている**。`copy.rs` 13〜27 行と呼び出し 51〜52 行、試験 135〜151 行。`nar.rs` 30〜41 行と呼び出し 83〜84 行、試験 311〜341 行 | 行が 1 つずれただけ。README の仕様メモ 137 行がこの守りを説明している（3 の表の 8.x の行を参照） |
| 7 | `non_unicode_name` の写しが 2 つ | **残っている**。`nar.rs` 43〜49 行、`update_files.rs` 150〜156 行 | 同じ |
| 8 | 見本のゴーストの試験の写し | **残っている** | `neutralize_debug_env` は 3 ファイル（`self_deploy_integration_test.rs` 35〜41・`touch_shortcuts_test.rs` 15〜21・`tutorial_stages_test.rs` 25〜31 行。どれも `#[ctor::ctor(unsafe)]`）。写す処理は名前違いを含めて 3 つ（`self_deploy` の `copy_tree_skip_profile` 59〜76 行、`touch` の `copy_master` 36〜49 行、`tutorial` の `copy_master` 184〜198 行。違いは `pasta.dll` を外すかだけ。3 つとも `.flatten()` で読み取りの誤りを握りつぶす＝N10）。`dist_src_validation_test.rs` の 4 ファイルの存在の確かめは、`integration_test.rs` の `test_pasta_toml_content`・`test_ukadoc_files` が同じ 4 ファイルを `unwrap` で読むので、丸ごと重複 |

### 1.4 新しい指摘（同じ領域）

| ID | 種類 | 場所 | 内容 | 要件 |
| -- | ---- | ---- | ---- | ---- |
| N1 | 欠けた守り（データが失われる） | `release.rs` 16 行の `remove_stale_nar`、`nar.rs` 17 行 | `--nar` が開発フォルダ（上書きフォルダ）の中の既存のファイルを指すと、段 1 で「前回の nar」として消し、段 6 で nar に置き換える。再現 F。brief の 3 つの関係に入っていない | 1.6（OPEN QUESTION 1） |
| N2 | 誤りの報告 | `copy.rs`・`nar.rs`・`update_files.rs` の `?` | 段 2・3・5・6 の I/O の誤りが、OS の文言だけでパスを含まない。再現 A は `Error: 指定されたパスが見つかりません。 (os error 3)` で、どのフォルダの話か分からない（日本語の Windows では OS の文言が日本語になる） | 5.3（OPEN QUESTION 2） |
| N3 | 握りつぶした誤り | `update_files.rs` 165〜168 行 | `read_dir` の失敗を `Ok(())` にする。在るはずのフォルダが読めないと、その下が黙って `updates.txt` から抜ける | 5.1 |
| N4 | 握りつぶした誤り | `update_files.rs` 205 行 | `metadata.modified()` の失敗を 1970-01-01 にする | 5.2 |
| N5 | 要らない確かめ | `nar.rs` 11〜15 行 | `parent.exists()` を見てから `create_dir_all` する。`create_dir_all` は在っても成功し、空のパス（`--nar g.nar`）でも成功する | 6.3 |
| N6 | 要らない作り | `nar.rs` 21〜22・55・89〜91 行、`update_files.rs` 216〜231 行 | nar は再帰の引数で `Vec<u8>` の入れ物を持ち回り、ファイルを丸ごと読んでから書く（`io::copy` 1 行で済み、大きなファイルでメモリも食わない）。md5 も手で 8 KB ずつ読む（md5 0.8 の `Context` が `io::Write` を持つなら `io::copy` で済む。要確認） | 6.3（OPEN QUESTION 4） |
| N7 | 古い注記・名前 | `update_files.rs` 89 行（「段 4 の生成結果」。今は段 5）、91 行（「従来の戻り値と同じ意味」）、`release.rs` 536 行の試験の名前 `…_on_stage5_failure`（中身は段 6 の失敗） | 読む人を惑わす | 6.3 |
| N8 | 要らない抽象 | `main.rs` 12〜14 行の `struct Args { command }` | 欄が 1 つだけの包み。`parse_args_from` が `Command` を返せば済む | 6.3（OPEN QUESTION 4） |
| N9 | 死んだコードの余波 | `tests/shell_assets_test.rs` 13〜15 行 | 指摘 4 の道具のために `#[allow(dead_code)]` を置いている。4 を直せば外せる | 6.4 |
| N10 | 握りつぶした誤り（試験） | 指摘 8 の写す処理 3 つ | `.flatten()` で `read_dir` の項目の誤りを捨てる。写し漏れたまま試験が進む | 6.7 |
| N11 | 写し（試験） | `src/scripts.rs` の `test_face_pasta_contains_all_characters` | 同じファイルの `test_face_pasta_expression_surface_mapping` が、両アクターの宣言と 9 表情すべての行を確かめているので、包まれている。roadmap の持ち場（`tests/**`）の外 | 6.9（OPEN QUESTION 5） |

### 1.5 指摘にしなかったもの（閉じた論点）

- `nar.rs` 76〜78 行・`update_files.rs` 182〜184 行の `strip_prefix(…).map_err(…)`: 起きない誤りの枝だが、`strip_prefix` が `Result` を返すための 1 行で、外すと `expect` に替わるだけで短くならない（audit-pasta-check Req 1.2 の形）。残す。
- `update_files.rs` 31〜34 行の `duration_since(UNIX_EPOCH).unwrap_or_default()`: 1970 年より前の日時を丸める値の扱いで、読み取りの誤りではない。残す。
- `days_to_ymd` の手書きの暦: `pasta_check` に日時の依存は無く、依存を足すほどではない。残す。
- 段 4 の `validate_cmd.rs` の誤りが日本語（「…を読めません」）なのは、`validate` の出力を日本語にした既存の決定（pasta-check-dic-validate）で、この spec の領域の外。
- 配布フォルダが「前回の配布フォルダでない、中身の入ったフォルダ」かを見て断る守り（例: `--release C:\Users\me\Documents`）: brief の Desired Outcome に無く、README の「毎回クリーンされる」が今の約束。入れない（requirements の Out of scope に書いた）。
- `main.rs` で `--target` などを 2 回書くと後勝ちになる: 壊す振る舞いではない。変えない。

---

## 2. 今の作り（Current State）

- **段の流れ**: `main.rs` の `parse_args_from`（lexopt。I/O なし）→ `release::execute_release`（段 1〜6）。段 1 は `copy::prepare_release_dir`（消して作る）と `remove_stale_nar`、段 2・3 は `copy::copy_dir_recursive`、段 4 は `validate_cmd::run_on_master`、段 5 は `balloon::plan_bundled_balloons` と `update_files::generate_update_files`、段 6 は `nar::create_nar`（失敗したら作りかけを消す）。
- **誤りの型**: どの段も `io::Result<()>`。`main.rs` が `Error: {e}` を出して終了コード 1。新しい誤りも `io::Error::new(InvalidInput, …)` で同じ口から出せる。
- **使える既存の型**: `balloon.rs` の `check_overlaps`（同梱バルーンのフォルダ同士の「同じ・上位・配下」を断る）が、関係の判定と英語の文言（`… overlaps with …`）の先例。ただし相手は配布フォルダからの相対の文字列で、ファイルシステムのパスの正規化はしていない。
- **パスの正規化の先例**: このリポジトリで `fs::canonicalize`・`std::path::absolute` を使う Rust のコードは無い（`pasta_lua` の debug の 8.3 の扱いは別の作り）。8.3 の短い名前を作る試験の道具は `pasta_lua/tests/ffi/dll_fixture.rs` の `short_path`（`windows-sys` の `GetShortPathNameW`）。
- **試験の置き場**: `release.rs`・`copy.rs`・`nar.rs`・`update_files.rs` の `#[cfg(test)] mod tests`（一時フォルダ）と、バイナリを走らせる `crates/pasta_check/tests/cli_test.rs`（`--version` の形もここで固定）。Rust の試験は CI では Windows（`windows-latest`）だけで走る。`#[cfg(unix)]` の試験は CI では走らない。
- **見本のゴーストの試験**: `tests/` の各ファイルが別々のバイナリ。`mod common;` を読むファイルは、使わない関数に dead_code の警告が出る（今は `shell_assets_test.rs` だけが `#[allow(dead_code)]` で抑えている）。

## 3. 要件と資産の対応（Requirement-to-Asset Map）

| 要件 | 今の資産 | ギャップ |
| ---- | -------- | -------- |
| 1.1〜1.6 関係の確かめ | 無い。`execute_release` は段 1 でいきなり消す | **Missing**: 段 1 の前の確かめ（`release.rs` に足すのが自然） |
| 1.7 入力のフォルダが無い | 段 2・3 の `read_dir` の失敗（段 1 の後） | **Constraint**: 先に確かめると、`test_execute_release_removes_stale_nar_on_failure`（無い `--copy` で段 1 の後の失敗を作っている）の意味が変わる。段 1 の後で失敗する別の作り方（段 4 の検査の誤りなど）に替える必要がある（OPEN QUESTION 3） |
| 1.8・1.9 何にも触れない | — | **Missing**。確かめの中で `create_dir_all` などを呼ばないこと |
| 2.1・2.2 同じ場所として比べる | 無い | **Missing／Research Needed**: 下の 5.1 |
| 2.4 渡した書き方のまま表示 | 今は渡した `PathBuf` をそのまま表示 | **Constraint**: 比べるための正規化した形（`\\?\…`）を、段の処理や表示に流さない。確かめの中だけで使う |
| 3.x 断る理由 | `balloon.rs` の `overlaps with` の文言 | **Missing**: 文言の決め（設計） |
| 4.1 `release.ps1` の段 4 | 再現 G で今どおり動く | **Missing**: それを固める試験 |
| 4.3 `--version` | `main.rs` 49 行、`cli_test.rs` 29〜38 行 | 変えない。試験は既にある |
| 5.1・5.2 段 5 の読み取りの誤り | `update_files.rs` 165〜170・205 行が握りつぶす | **Missing**: `?` で返す。固定している試験を直す |
| 5.3 パスを含む誤り | `validate_cmd.rs` 122 行の `format!("{} を読めません: {e}", …)` が形の先例（ただし日本語） | **Missing**（OPEN QUESTION 2） |
| 6.1 働かない守り | `ensure_within`・`ensure_no_parent_component` と試験 | 消すだけ。README 137 行も |
| 6.2 写し | `non_unicode_name` ×2 | どちらかへ寄せる（置き場は設計。`update_files.rs` から `nar.rs` が使う、など） |
| 6.3 N5〜N8 | 上の表 | 消す・縮める |
| 6.4〜6.7 見本のゴーストの試験 | `common/mod.rs`・3 ファイルの写し | **Constraint**: `common` に写す処理と `#[ctor]` を寄せると、使わないバイナリ（`integration_test`・`shell_assets_test`）で dead_code の警告が出る。`common` を小分けにするか、使う関数だけに `#[allow(dead_code)]` を付けるか（設計）。`#[ctor]` を `common` に置くと、`mod common;` を読む全バイナリで環境変数が外れる（害は無い） |
| 7.x 試験 | 一時フォルダの試験の型は揃っている | **Missing**: 関係ごとの試験・書き方の違いの試験・段 5 の読み取りの誤りの試験（読めないフォルダを作る方法は Research Needed） |
| 8.x 文書 | README 57〜107・129〜137 行、SKILL.md 63〜86 行と 170〜190 行の表、nar-spec.md 71 行、validate.md 202〜221 行 | **Constraint**: SKILL.md と nar-spec.md の編集は分類器が Self-Modification として止める。開発者の許可が要る |

## 4. 実装の選択肢

### Option A: `release.rs` に確かめを足す（推奨）

- `execute_release` の頭（段 1 の表示の前）で、引数の関係を確かめる小さな関数を呼ぶ。正規化の手助け（在る一番近い親を `canonicalize` して残りを足す）も同じファイルに置く。
- `copy.rs`・`nar.rs` は守りを消して縮むだけ。`update_files.rs` は握りつぶしを `?` に替える。
- ✅ 変更が `release.rs` の頭に集まり、段の流れも呼び出す側も変わらない。`release.rs` は 720 行だが、本体は 86 行で、残りは試験。
- ❌ `release.rs` の試験がさらに増える（600 行の目安は既に超えている。試験を `release_tests.rs` に出すかは設計）。

### Option B: パスの関係を新しいモジュール（例: `paths.rs`）に出す

- 正規化と関係の判定を別ファイルにし、`release.rs` から呼ぶ。
- ✅ 単体の試験を分けて置ける。
- ❌ 呼ぶのは 1 か所だけ。ファイルが 1 つ増えるだけの抽象になりやすい。roadmap の持ち場（`release.rs`・`copy.rs`・`nar.rs`・`update_files.rs`）の外の新しいファイルになる（同じクレートで他の spec が触らないので衝突はしない）。

### Option C: `main.rs` の引数の読み取りで確かめる

- `parse_args_from` の後で確かめる。
- ✅ 段の処理より前なのが目に見える。
- ❌ `parse_args_from` は I/O をしない純粋な関数で、試験もそう書いてある。I/O を混ぜると、`execute_release` を直接呼ぶ試験（`release.rs` の試験の全部）が確かめを通らなくなる。推奨しない。

## 5. Research Needed（設計へ持ち越す）

1. **正規化の形**: `fs::canonicalize` は Windows で `\\?\C:\…`（UNC なら `\\?\UNC\…`）を返す。`std::path::absolute` は `C:\…` を返す。両者を `Path::starts_with` で比べても一致しない（5 の 1 を参照）。どのパスも「`absolute` で絶対にし、在る一番近い親を `canonicalize` して、無い残りの部品を足す」形にそろえる案が、8.3・大文字と小文字・junction を在る部分で解き、まだ無いパス（`--release`・`--nar`）にも効く。まだ無い部分は在るフォルダと同じになりえないので、残りの部品を書かれたまま足しても判定は崩れない。`..` が無い残りの部品に入る場合（Windows の `absolute` は字面で `..` を畳むので問題にならない見込み）を確かめる。
2. **同じ・中にあるの比べ方**: `Path::starts_with` は部品ごとの比べで、`C:\dev\g` と `C:\dev\ghost` を取り違えない。正規化の後は大文字と小文字もそろうので、そのまま使える見込み。ドライブ文字の大小（`c:` と `C:`）は `canonicalize` がそろえる。
3. **8.3 の短い名前の試験**: `pasta_check` に `windows-sys` の dev-dependency を足すと `Cargo.lock` が変わり、同じウェーブで `Cargo.lock` の席を持つ `mlua-stdlib-removal` と重なる。要件 7.3 は書き方の違いの試験に 8.3 を入れていない。CI の `%TEMP%` が `RUNNER~1` なので、`TempDir` のパスと正規化した形が混ざる試験は CI で自然に 8.3 を通る。明示の試験を置くかは設計で決める（置くなら、依存を足さずに作る方法を探す）。
4. **段 5 の読み取りの誤りの試験**: 読めないフォルダを Windows の一時フォルダで作る方法（ACL で読み取りを拒む・`generate_update_files` を在らないフォルダに直接呼ぶ など）。いちばん小さいのは、関数の単位で在らないフォルダを渡す試験（今の `…_nonexistent_root_returns_zero` を誤りの期待に替える）。
5. **md5 0.8 の `Context` の `io::Write`**: N6 の md5 側を `io::copy` にできるか（手元の cargo のレジストリの場所が分からず、確かめていない）。
6. **誤りの文言**: 関係ごとの英語の文言と、調べる順（要件 3.4）。`balloon.rs` の `overlaps with` の言い回しに寄せるか。

## 6. 規模とリスク

- **Effort: M（3〜7 日）**。確かめの関数は小さいが、関係ごとの試験・見本のゴーストの試験の寄せ・文書 5 か所（うち 2 か所は開発者の許可待ち）があり、1 つ直すごとに確かめて 1 コミットにする進め方（brief の Constraints）を取るため。brief の見込みの 12〜15 タスクと合う。
- **Risk: Medium**。消す・作る前の確かめで、間違えると正しい使い方（`release.ps1` の段 4）を断るか、危ない組み合わせを通す。パスの形の差（`\\?\`・8.3）は CI でしか出ないことがある（既知の落とし穴）。どちらも試験で押さえられ、技術は標準ライブラリの範囲。

## 7. 設計への申し送り

- **推奨**: Option A。段 1 の表示の前に確かめ、正規化は「在る一番近い親の `canonicalize`＋残りの部品」。表示と段の処理は渡したパスのまま。
- **先に固める試験**: 再現 A〜G を `release.rs` の試験（一時フォルダ）に起こしてから直す（brief の「消す・壊すケースは一時フォルダで再現してから直す」）。C は今のコードではスタックあふれで試験のプロセスごと落ちるので、直す前の赤の確かめは、確かめの関数の単体の試験か、深さを抑えた形で行う。
- **振る舞いを変える直し**: 1.1〜1.7（断る）、5.1〜5.3（段 5 で止まる・パスを含む）。同じ変更で README・validate.md を直し、SKILL.md・nar-spec.md は許可を求める。
- **変えないもの**: 6 段の表示、`--version`・`--help`、段 1 の前回の nar の削除と段 6 の作りかけの削除、nar のエントリ、`updates.txt` のバイト（日時を除く）。既存の特性化の試験（`test_updates_txt_characterization_without_balloon` など）がそのまま通ること。
- **順番**: 後の `mlua-0-12-upgrade` が `self_deploy_integration_test.rs` を触るので、見本のゴーストの試験の寄せはこの spec で済ませておく。
