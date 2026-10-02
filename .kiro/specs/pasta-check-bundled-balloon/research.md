# ギャップ分析: pasta-check-bundled-balloon

> 実施日: 2026-10-02 / 対象: `requirements.md`（requirements-generated・未承認）
> 方針は brief.md の案 A（`pasta_check` が `install.txt` を読み、同梱バルーンを汎用に扱う）で確定済み。本書は既存コードとの差分と実装の選択肢を整理する。最終判断は設計フェーズで行う。

## 1. 分析サマリ

- 変更の中心は `crates/pasta_check/src/update_files.rs` の 1 ファイル。収集（`collect_files`）と書き出し（`generate_updates_txt`）は「基準フォルダからの相対パス」で動くので、バルーン用 `updates.txt` にほぼそのまま使える。足りないのは (a) `install.txt`／`descript.txt` の読み取り、(b) 「特定のフォルダの配下だけ」を外す除外（今はフォルダ名だけで一致させている）、(c) 警告を呼び出し元へ返す仕組み。
- `nar.rs` は配布フォルダをそのまま固めるので変更不要の見込み（Req 6）。`release.rs` は進捗表示と警告の表示、エラー時に nar を作らない流れの確認だけ。
- 外部仕様の確認で、brief に無かった論点が 2 つ見つかった。番号付きのバルーン指定（`balloon0.*`・`balloon1.*`…）と、SSP 2.9.00 以降は `*.source.directory` に階層付きの相対パス（`extra\bal1`・区切りは `\` と `/` のどちらでも可）を書けること。要件では両方を対象に含める前提で起草し、OPEN QUESTION にした。
- スキル `pasta-check` の references は**マニュアル（`book/`）からの生成対象外の手書きファイル**である（`book/tools/gen-skill-refs.mjs` の生成対象は `pasta-ghost-authoring`・`pasta-lua-coding` だけ。`.kiro/steering/workflow.md` の Manual Sync Gate も同じ 2 スキルと `book/` に触れるときだけ発火）。したがって直接編集でよく、`gen-skill-refs.mjs --check` の対象にもならない。マニュアルには `pasta_check` の章が無い。
- 規模は S〜M、リスクは低〜中。リスクの多くは「同梱バルーンが無いゴーストの出力をバイト単位で変えない」ことと、Windows の大文字小文字を区別しないパス照合にある。

## 2. 現状調査

### 2.1 関連ファイル

| ファイル | 役割 | 本件との関係 |
|---|---|---|
| `crates/pasta_check/src/release.rs` | 5 段の流れの制御・進捗表示（stdout） | 段 4 の表示（バルーン用の件数）と警告の表示を足す |
| `crates/pasta_check/src/update_files.rs` | `generate_update_files(root)`：全ファイル収集 → ルート `updates.txt` 書き出し → `ghost/master` へコピー。戻り値は件数 `usize` | 主な変更箇所 |
| `crates/pasta_check/src/nar.rs` | 配布フォルダを ZIP 化。`profile` という名前のフォルダはどの階層でも外す。`..` を含むエントリ名を拒否 | 変更不要の見込み |
| `crates/pasta_check/src/copy.rs` | `fs::copy` によるバイト単位の複製。`ensure_within` でパストラバーサル防御 | 内容を変換しない（Req 5.2）ことは既に満たしている |
| `crates/pasta_check/src/main.rs` | lexopt による引数解析。エラーは stderr に `Error: …` で表示し exit 1 | 警告の表示形式の手本（`Warning: …` を stderr へ、が自然） |
| `crates/pasta_check/tests/cli_test.rs` | バイナリ経由の E2E（stdout・stderr・終了コード） | 警告・エラーの E2E を足す場所 |
| `.claude/skills/pasta-check/SKILL.md`・`references/updates-txt-spec.md`・`references/nar-spec.md` | 手書きのスキル文書 | Req 9 |
| `crates/pasta_check/README.md` | crates.io 向けの説明。「仕様メモ」に updates.txt と NAR の規則がある | Req 9.4 |

### 2.2 既存の決まりごと

- 依存は `lexopt`・`md5`・`zip`（dev: `tempfile`）だけ。文字コード変換のクレート（旧 `encoding_rs`）はすでに外れている。
- 除外は名前一致：`EXCLUDED_DIRS = ["profile", "var"]`（どの階層のフォルダにも効く）、`EXCLUDED_FILES = ["updates2.dau", "updates.txt", "developer_options.txt"]`（どの階層のファイルにも効く）。このため、同梱バルーンのフォルダ内に既にある `updates.txt` は、今でもゴースト用一覧には載らない。
- エントリはパスの辞書順にソート（決定的出力）。書式は `charset,UTF-8` ＋ CRLF ＋ SOH 区切り。md5 は 8 KiB ずつ読んで計算。
- `read_dir` の失敗は空集合に倒す（存在しないルートで `Ok(0)`）。シンボリックリンクは追わない。
- 0 件のときは `updates.txt` を作らない。
- テストは各モジュール内の `#[cfg(test)]`（tempfile を使う）と `tests/cli_test.rs`。パス区切りは `\` を `/` に置き換えて扱う。
- 過去の知見：CI では `%TEMP%` が 8.3 短縮名になり、パスの形に依存する比較は CI だけで壊れる。正規化は `fs::canonicalize` ではなく `std::path::absolute` を使う（メモリ「CI固有の8.3短縮名パスバグ」）。

### 2.3 実データ（ghost_dev、読み取りのみで確認）

- `ghost_dev/release/emo2/emo2/install.txt`：`Charset,UTF-8`（**先頭が大文字**）・`balloon.directory,emo2-kakukaku`・`balloon.source.directory,emo2-kakukaku`。CRLF。
- `emo2-kakukaku/` には 20 ファイル。バルーン自身の `install.txt`（`type,balloon`）と `online.pdn` を含む。`descript.txt` に `homeurl` は**無い** → 新しい版では警告が出る（Req 7.1 の想定どおり）。
- 配布フォルダ直下に `delete.txt`・`readme.txt` がある（ゴースト用一覧に残る。Req 3.3）。
- `project/sample-ghost/dot_sakura/install.txt` は `charset` 行が無い Shift_JIS で、`balloon.directory,bottle` のみ（`source.directory` 無し）。Req 1.2（`directory` で代用）と Req 1.9（Shift_JIS でも ASCII のキーと値は読める）の実例になる。
- `pasta-in-windows` の `install.txt` にバルーン指定は無い → Req 8 の回帰対象。

### 2.4 外部仕様（ukadoc、要約経由で取得）

- `install.txt`（`descript_install.html`）
  - `balloon.directory` はインストール先の名前、`balloon.source.directory` はアーカイブ内のフォルダ。後者を省略すると前者を使う。
  - 番号付き `balloon0.*`・`balloon1.*`… で複数同梱できる。探索順は 番号なし → 0 → 1 → …、欠番で打ち切り。
  - SSP 2.9.00 以降、`*.source.directory` に `extra\bal1` のような階層付き相対パス（`\`・`/` どちらでも可）を書ける。それより前の版では区切りが `_` に置き換わる。
  - `charset` 行は 1 行目に書く。省略時は Shift_JIS。キーの大文字小文字の扱いは記載なし。
  - SSP の生成機能は `desktop.ini`・`thumbs.db`・`.DS_Store`・`profile/`・`var/` を自動で外し、`developer_options.txt` の `noupdate`（更新ファイルから外す）・`nonar`（nar から外す）を解釈する。`pasta_check` はこれらの一部しか実装していない（既存の差。本件の対象外）。
- ネットワーク更新（`manual_update.html`・`descript_balloon.html`）
  - バルーンの `updates.txt`／`updates2.dau`／`delete.txt` は、バルーンのルート（`descript.txt` と同じ階層）に置く。
  - バルーンの `descript.txt` に `homeurl` を書くとネットワーク更新できる。
  - バルーン用一覧のパスがバルーンフォルダ基準であることは明記されていない（配置から推測。brief の判断と一致）。
  - ゴーストの更新時に同梱バルーンも更新されるかは記載なし。
- ゴーストの `updates.txt` はゴーストのルートに置く（`ghost/master` ではない）。`pasta_check` はルートと `ghost/master` の両方に置いており、そのまま。

### 2.5 関連セッションからの事実（2026-10-02 照会）

**areka（alpha release セッション、areka HEAD 25c5f22c と実機記録 run-A3r.log）**
- `install.txt`: `source.directory` が無いか空なら `directory` を取り出し元にする。同梱を読むのは `type` が ghost/shell のときだけ。
- 番号付き: `balloon` の直後が数字だけの接頭辞を全キー走査で拾う（欠番で打ち切らない）。ukadoc（欠番で打ち切り）と異なる。
- 階層付きの値: `directory`・`source.directory` とも 1 階層の名前だけを受け、`/`・`\` を含むとインストール全体を拒否する（SSP 2.9.00 以降の仕様とは異なる）。
- キーは大文字小文字を無視し、キーと値の前後空白を trim。値の大小はそのまま比較。
- `charset` 行が無ければ CP932。UTF-8 の BOM は吸収する。
- `..`・絶対パス・空などの値や、取り出し元フォルダが nar に無い（0 件）場合はインストール全体を拒否する。
- ネットワーク更新: ゴーストの更新で同梱バルーンの実体は更新しない。対象は今のゴースト・シェル・使用中のバルーンで、それぞれ自分の `homeurl` を使う。バルーンの一覧のパスはバルーンのフォルダ基準。
- バルーンに `homeurl` が無ければ更新対象から外し `no_homeurl` の WARN を出す。
- `updates.txt` の `file,`・`charset,` は小文字の接頭辞だけを認識する（pasta_check の出力は小文字で適合）。
- 要望: 同梱バルーンのファイルがゴーストの一覧に載らないこと（本件で満たす）、`file,`・`charset,` が小文字であること（現行で満たす）。

**ghost_dev（emo2 開発セッション、ghost_dev 31cffa0）**
- emo2 の `install.txt`: UTF-8・BOM なし・CRLF、1 行目が `Charset,UTF-8`。番号付き・階層付きの予定は無い。
- `emo2-kakukaku/descript.txt` のソースには `homeurl` を追加済み。`release/emo2` は areka alpha の署名まで凍結中で旧版のまま。
- バルーン内の `install.txt`・`online.pdn` は特別扱い不要。除外規則はゴースト側と揃えてほしい。
- 実際に pasta_check でリリースしているのは emo2 と pasta-in-windows だけ。

## 3. 要件と既存資産の対応表

| 要件 | 既存資産 | ギャップ |
|---|---|---|
| 1.1–1.5 バルーン指定の読み取り（番号付き含む） | なし | **Missing**：`install.txt` の行解析（`key,value`）、番号付き指定の探索、重複の除去 |
| 1.6–1.7 直下の `install.txt`・`--copy` 後 | 段 4 の時点で配布フォルダは完成している | 段 4 の中で読めば満たせる（**Constraint**：段の順序を守る） |
| 1.8 キーの大文字小文字・前後空白 | なし | **Missing**（`Charset` の実例あり） |
| 1.9–1.11 BOM・charset・非 ASCII | 文字コード変換の依存なし | **Missing**：バイト列のまま ASCII のキーを照合する方式なら新しい依存は不要。非 ASCII の値を Shift_JIS で解釈するなら変換手段が要る（**Unknown**、OPEN QUESTION 1） |
| 2.1–2.2 不正な値・階層付きの値 | `copy.rs` の `ensure_within`、`nar.rs` の `ensure_no_parent_component` | **Missing**：値の検証（空・絶対・ドライブ・UNC・`..`）と、`\`・`/` の正規化。既存の防御関数は流用の候補 |
| 2.3 `ghost/master`・他バルーンとの重なり | なし | **Missing**：相対パス同士の上位・配下の判定 |
| 2.4 配布フォルダの外を触らない | 上記 2 関数 | **Constraint**：検証してから結合する順序を守る |
| 2.5 フォルダ不在の警告 | なし | **Missing**：警告の仕組みそのものが無い |
| 2.6 大文字小文字だけ違う場合の一貫性 | なし | **Constraint/Unknown**：`root.join(値).is_dir()` は Windows では大文字小文字を無視して真になるが、収集時の除外を文字列比較で行うと一致せず、**除外漏れと生成先のずれが起きる**。実在のフォルダ名を取り出して使うか、比較を大文字小文字無視にするかを設計で決める（Research Needed） |
| 3.1–3.4 ゴースト用からの除外 | `collect_files_recursive` の名前一致除外 | **Missing**：「基準フォルダからの相対パスが一致するフォルダだけ」を外す除外。今の `EXCLUDED_DIRS` はどの階層でも名前一致なので、そのままでは 3.4（同名の別フォルダは外さない）を満たせない |
| 3.2 `ghost/master` への複製 | `fs::copy` による複製 | 変更不要（除外済みの内容が複製される） |
| 4.1–4.4 バルーン用の生成 | `collect_files(root)`・`generate_updates_txt(root, entries)` は基準フォルダを引数に取る | ほぼ流用可。基準をバルーンのフォルダにすれば相対パスも自分自身の除外も既存の挙動で満たせる |
| 4.5 同じ除外規則 | `EXCLUDED_DIRS`・`EXCLUDED_FILES` | 流用で満たせる（OPEN QUESTION 2 の結論次第） |
| 4.6 既存 `updates.txt` の置き換え | `File::create` は上書き | 満たしている |
| 4.7 0 件で生成しない | ゴースト用の早期 return | 同じ分岐を流用 |
| 4.9 進捗表示 | `release.rs` の `println!` | **Missing**：戻り値を件数 1 つから「ゴースト件数＋バルーンごとの件数＋警告」へ広げる必要 |
| 5.1–5.3 実ファイルとの一致 | 実バイトから md5・size、`fs::copy`・nar はバイトそのまま | 満たしている。回帰テストで固定する |
| 6.1–6.2 nar への封入 | `create_nar` は配布フォルダ全体を固める | 変更不要。テストを足すだけ |
| 7.1–7.5 homeurl 警告 | なし | **Missing**：`descript.txt` の読み取り（`install.txt` と同じ行解析を流用できる）と警告 |
| 7.3–7.4 警告で止めない・接頭辞 | エラーは `Error:` で stderr | **Missing**：警告の表示経路。stderr に `Warning:` が既存の流儀に合う |
| 8.1–8.3 後方互換 | 既存テスト（ソート・SOH・`ghost/master` 複製・除外） | **Constraint**：除外の仕組みを差し替えても、バルーン無しの出力をバイト単位で変えない。差し替え前に「現行出力を固定する特性化テスト」を置くのが安全（メモリ「リファクタリングは安全かつ可逆に」） |
| 8.4–8.5 テスト | `update_files.rs` の単体テスト群・`release.rs`・`cli_test.rs` | 追加のみ |
| 9.1–9.5 文書 | 手書きのスキル 3 ファイル・README | 直接編集でよい。既存の食い違い（下記 4.3）も同じ文書内 |

## 4. 実装上の注意点

### 4.1 エラー時の後始末

Req 1.11・2.1・2.3 は「ゼロ以外で終了し nar を作らない」。現在の流れでは段 4 の途中で失敗すると、配布フォルダに書きかけの `updates.txt` が残り、`--nar` の位置に**前回の nar が残る**（既存の IO エラーでも同じ）。`install.txt` の検証を書き出しより前に済ませれば書きかけは防げる。前回の nar を消すかどうかは既存の挙動に合わせるのが自然（設計で確認）。

### 4.2 除外の仕組みの差し替え

いまの除外は「フォルダ名・ファイル名がどの階層でも一致したら外す」の 1 種類。本件では「基準フォルダからの相対パスが一致するフォルダを外す」が新たに要る。名前一致の規則（`profile` など）は残したまま、相対パス一致の除外集合を追加で渡す形にすれば、バルーン無しのときは除外集合が空になり、出力は変わらない。

### 4.3 既存文書の食い違い（Req 9.5 の対象）

- `updates-txt-spec.md` の除外ファイル表に `updates2.dau` が無い（コードは除外している）。
- `SKILL.md` のトラブルシューティングに「updates.txt が Shift_JIS でない → pasta_check のバグ」とあるが、現行は UTF-8 で出力している。
- `nar-spec.md` の内部構造の例に `ghost/master/pasta_scripts/` がある（自己展開方式への移行で同梱は廃止済み。tech.md 参照）。
- `nar.rs` は `profile` だけを外し `var` は外さない（`updates.txt` は両方外す）。文書は実装どおりなので食い違いではないが、規則の非対称として記録しておく。

### 4.4 その他

- ghost_dev 側にも同じスキル `pasta-check` の写しがある。同期は本件の対象外（emo2 開発セッションに委ねる）。
- 生成されたバルーン用 `updates.txt` が `date=` を含むため、emo2 の再生成では毎回差分が出る。これは既存のゴースト用と同じ性質で、問題ではない。

## 5. 実装方針の選択肢

### Option A: `update_files.rs` を拡張する

- `update_files.rs` に `install.txt`／`descript.txt` の行解析、バルーン指定の解決と検証、相対パス一致の除外を足し、`generate_update_files` の戻り値を「ゴースト件数・バルーンごとの件数・警告の一覧」を持つ型に変える。`release.rs` は表示だけを担う。
- ✅ 変更が 2 ファイルに収まる。収集・書き出しの部品をそのまま共有できる。
- ✅ 既存テストがそのまま回帰の網になる。
- ❌ `update_files.rs`（現在 534 行、うちテスト約 330 行）に解析と検証が入り、責務が「更新ファイル生成」から少し広がる。

### Option B: `install.txt` の解析を新しいモジュールに分ける

- 新しく `install_txt.rs`（仮）を作り、`key,value` の行解析・キー照合・文字コードの扱い・バルーン指定の解決と値の検証を持たせる。`update_files.rs` は「除外する相対パスの集合」と「バルーンのフォルダの一覧」を受け取って生成するだけにする。`descript.txt` の `homeurl` 読み取りも同じ行解析を使う。
- ✅ brief の Boundary Candidates（`install.txt` の解析／更新ファイル生成の分割／警告）にそのまま沿う。解析と検証を単体でテストしやすい。
- ✅ 将来、同梱シェルなど他の `install.txt` のキーを扱うときの置き場になる（ただし本件の対象外なので、それを理由に作り込まない）。
- ❌ ファイルが 1 つ増え、受け渡しの型を決める必要がある。

### Option C: 折衷

- 行解析と値の検証だけを小さな新モジュールに出し、バルーン指定の解決・警告の組み立ては `update_files.rs` に置く。
- ✅ 新しいファイルを最小限にしつつ、テストしにくい部分（検証）だけを切り出せる。
- ❌ 責務の線引きがやや曖昧になる。

## 6. 規模とリスク

- **Effort: S（1〜3 日）** — 既存の収集・書き出しを流用でき、新しい依存も要らない（非 ASCII の値を Shift_JIS で解釈しない前提）。文書更新を含めても小さい。番号付き指定と階層付きの値を含めても解析と検証が増えるだけ。
- **Risk: Low〜Medium** — 既存のパターンの延長で、外部連携も無い。ただし (1) バルーン無しの出力をバイト単位で変えない制約、(2) Windows の大文字小文字を区別しない照合と 8.3 短縮名による CI だけの不一致、(3) 階層付きの値と重なり判定、の 3 点で取りこぼすと実配布物を壊すため Medium 寄り。

## 7. 設計フェーズへの申し送り

### 推奨の進め方

- Option B または C を軸に、解析・検証を単体でテストできる形にする。いずれも `update_files.rs` の収集・書き出しの部品は共有する（brief の方針）。
- 除外の差し替えの前に、バルーン無しの現行出力（`date=` を除く）を固定する特性化テストを置く。
- 警告は stderr に `Warning:` 接頭辞、エラーは既存どおり `Error:` で stderr・exit 1、が既存の流儀に合う。

### 設計で決める点

1. 戻り値の形（ゴースト件数・バルーンごとの件数・警告）と、警告の表示を誰が担うか。
2. 相対パス一致の除外の渡し方と、大文字小文字の違いをどう揃えるか（実在のフォルダ名を採るか、比較を大文字小文字無視にするか）。
3. 値の正規化（`\` → `/`、末尾の区切り、`.` の要素）と検証の順序。
4. 検証エラーを書き出しより前に確定させる流れ。

### Research Needed

- SSP が `install.txt`／`descript.txt` のキーの大文字小文字を区別するか（実例 `Charset` は動いているので、少なくとも `charset` は区別しないと見られる）。
- SSP 2.9.00 未満で階層付きの `source.directory` を書いた配布物の扱い（区切りが `_` に置き換わる）を、`pasta_check` が警告すべきか。
- `balloon.directory` だけで `source.directory` が無く、アーカイブにそのフォルダが無い場合のベースウェアの挙動（ukadoc に記載なし。Req 2.5 の「警告して続行」の妥当性の裏付け）。

## 8. OPEN QUESTIONS（要件ディスカッションで決める）

1. **`install.txt` の文字コード**（Req 1.9–1.11, 7.5）— **決定: pasta_check は UTF-8 に限って動く。`install.txt` は同梱バルーンの有無にかかわらず 1 行目の `charset,UTF-8` 宣言を必須とし、無い・UTF-8 以外・不正なバイト列はエラー。同梱バルーンの `descript.txt` も同じ。文字コード変換はしない。hello-pasta の `install.txt` に宣言を足す。** 当初の前提: バイト列のまま ASCII のキーを照合し、BOM は無視。値が非 ASCII のときは `charset,UTF-8` の場合だけ UTF-8 で解釈し、それ以外はエラーで止める（Shift_JIS 変換はしない・依存を増やさない）。
2. **既存の除外規則をバルーン側にも効かせるか**（Req 4.5）— **決定: 同じ規則を適用する（ukadoc より自明）。** 当初の前提: 同じ規則（`profile`・`var`・`updates2.dau`・`updates.txt`・`developer_options.txt`）をそのまま適用する。
3. **不正な値・不在フォルダ・重なりの扱い**（Req 2.1, 2.2, 2.3, 2.5）— **一部決定: 空・絶対パス・`..` はエラー。階層付きの値は `source.directory` でだけ受け、`directory` で代用する値の区切りはエラー（ukadoc より自明）。フォルダ不在（2.5）もエラーで停止（ディスカッションで決定）。重なり（2.3）もエラーで停止（作成ツールは問題があれば止める）。** 当初の前提: 空・絶対パス・`..` はエラーで停止。階層付きの相対パス（`\`・`/`）は SSP 2.9.00 以降の仕様として受け入れ、`directory` で代用する場合も同じ扱い。`ghost/master` や他のバルーンとの入れ子はエラー。フォルダ不在は警告して続行。
4. **キーの照合規則**（Req 1.8）— **決定: 大文字小文字を無視する（実データより自明）。** 当初の前提: キー名は大文字小文字を無視し、値の前後の空白を無視する。
5. **`descript.txt` が無い同梱バルーン**（Req 7.2）— **決定: エラーで停止する（作成ツールは問題があれば止める）。** 当初の前提: 警告のみでビルドは止めない。
6. **文書の範囲**（Req 9.4, 9.5）— **決定: README も更新し、触る文書の既存の食い違いも同時に直す。** 当初の前提: `crates/pasta_check/README.md` も更新し、触るスキル文書の既存の食い違い（`updates2.dau`・Shift_JIS 記述・`pasta_scripts` の例）も同時に直す。
7. **番号付きのバルーン指定**（Req 1.4, 1.5, 3.1, 4.1）— **決定（#1）: 対象に含める。** 当初の前提: brief は番号なしの `balloon.*` だけを挙げているが、ukadoc の `balloon0.*`… も同じ問題を起こすため対象に含め、ベースウェアと同じ探索順（欠番で打ち切り）で扱う。

---

# 設計フェーズの調査と決定（2026-10-02）

## Summary
- **Feature**: `pasta-check-bundled-balloon`
- **Discovery Scope**: Extension（既存 CLI の段 4 の拡張。light discovery。新しい外部依存なし、外部仕様は §2.4・§2.5 で調査済み）
- **Key Findings**:
  - 既存テストのフィクスチャ（`release.rs` の `test_execute_release_full_pipeline`、`tests/cli_test.rs` の `test_release_end_to_end_via_binary`）は `install.txt` の中身が `"install"` で、Req 8.6（UTF-8 宣言必須）により新しい実装では失敗する。フィクスチャの更新が必要。
  - リポジトリの `release/hello-pasta/`（`updates.txt`・`ghost/master/updates.txt`・`install.txt`）と `release/hello-pasta.nar` は git 管理下にある。`install.txt` だけを直すと写しの `updates.txt` の `install.txt` 行の md5・size が実ファイルと食い違うため、Req 8.7 の同期は再生成で行うのが一貫する（OPEN QUESTION 3）。hello-pasta の開発フォルダは `pasta.dll` まで git 管理下なので、段 4 だけ（`release.ps1 -SkipSetup` 相当）で再生成できる。
  - CI（`.github/workflows/build.yml`）のテストは `windows-latest`。大文字小文字を区別しないファイルシステムに依存するテスト（Req 2.6）は `#[cfg(windows)]` で CI 上でも実行される。
  - `nar.rs` は変更不要（段 4 → 段 5 の順序で、配布フォルダ内の `updates.txt` は通常ファイルとして封入される）。

## Research Log

### 既存コードとの接点（コード確認）
- **Context**: §7 の申し送り（戻り値・大文字小文字・正規化順・書く前の検証）を決めるため。
- **Sources Consulted**: `crates/pasta_check/src/{update_files,release,main,nar,copy}.rs`、`tests/cli_test.rs`、`crates/pasta_sample_ghost/{release.ps1,tests/*.rs,ghosts/hello-pasta/install.txt}`、`.github/workflows/build.yml`、steering `structure.md`・`tech.md`
- **Findings**:
  - `collect_files_recursive` の相対パスは `strip_prefix(root)` ＋ `\`→`/` で作られ、各要素は `read_dir` が返す実在名になる。
  - `generate_update_files` の呼び出し元は `release.rs` だけ（テストを除く）。戻り値の形を変えても影響は閉じている。
  - `main.rs` はエラーを `Error: {e}` で stderr に出し exit 1。警告の経路は無い。
  - hello-pasta の `install.txt` は `type,ghost`・`name`・`directory`・`accept` の 4 行で `charset` 行が無い（CRLF）。`pasta_sample_ghost` のテストは `contains` で確認しており、1 行目の追加では壊れない。
  - steering `structure.md` と README の「ソース構成」が `pasta_check/src/` のファイル一覧を持つ。新しいモジュールを足すと両方の更新が要る。
- **Implications**: 戻り値の変更・除外の追加・新モジュールはいずれも `pasta_check` の中で閉じる。既存フィクスチャの更新はタスクに明示する。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `update_files.rs` に集約 | 解析・検証・生成を 1 ファイルに | ファイルが増えない・文書の「ソース構成」更新が不要 | 1 ファイルが 1000 行級になり、「更新ファイル生成」と「install.txt の判定」が混ざる | §5 Option A |
| B: `balloon.rs` を新設（採用） | 判定（読み取り専用）を新モジュール、生成は既存 | 判定のエラー種別ごとの単体テストを分けて置ける。読み取り専用と書き込みの境界がファイル境界と一致し、「書く前に検証」が構造で保証される | ファイル 1 つと steering・README の 1 行追記 | §5 Option B。OPEN QUESTION 1 |
| C: 行解析と検証だけ新設 | 折衷 | 新ファイル最小 | 判定の責務が 2 ファイルに割れる | §5 Option C |

## Design Decisions

### Decision: 戻り値の形と警告の表示（§7-1）
- **Context**: Req 4.9（バルーンごとの件数表示）、7.3・7.4（警告は止めず接頭辞付き）。
- **Alternatives Considered**:
  1. `update_files.rs` が直接 `eprintln!` する
  2. 判定は `BalloonPlan { dirs, warnings }`、生成は `UpdateSummary { ghost_entries, balloon_entries }` を返し、表示は `release.rs` に集約
- **Selected Approach**: 2。警告文は `balloon.rs` が組み立て（接頭辞なし）、`release.rs` が段 4 の進捗の後に `eprintln!("Warning: {w}")`。
- **Rationale**: 既存の「表示は `release.rs`／`main.rs`、処理は戻り値で返す」の形に合い、警告の有無を単体テストで確かめられる。
- **Trade-offs**: 構造体が 2 つ増える。
- **Follow-up**: E2E で stderr の `Warning:` と exit 0 を確認。

### Decision: 大文字小文字の扱い（§7-2、Req 2.6）
- **Context**: Windows では `root.join("EMO2-KAKUKAKU").is_dir()` が真になるが、収集の相対パスは実在名（`emo2-kakukaku`）になるため、文字列比較の除外がずれる。
- **Alternatives Considered**:
  1. 除外・重複除去・重なりの比較をすべて大文字小文字無視にする
  2. 判定時に `read_dir` で各要素を実在名へ解決し、以降は完全一致で扱う
  3. `fs::canonicalize` で実在名を得る
- **Selected Approach**: 2。完全一致の実在名を優先し、無ければ `親.join(要素).is_dir()` が真のときに限り小文字化一致の実在名を採る。`ghost/master` との重なり判定だけは ASCII 大文字小文字無視。
- **Rationale**: 比較の箇所（除外・重複除去・重なり・表示）がすべて同じ文字列で済み、「ファイルシステムが同じとみなすときだけ同じ」（Req 2.6 の条件）をそのまま実装できる。結合に `read_dir` の実在名だけを使うので、配布フォルダの外を指さないことの保証にもなる（Req 2.4）。案 3 は 8.3 短縮名・`\?\` 接頭辞で CI だけ壊れる既知の問題がある（メモリ「CI固有の8.3短縮名パスバグ」）。案 1 は大文字小文字を区別するファイルシステムで別フォルダを誤って外しうる。
- **Trade-offs**: 要素ごとに `read_dir` を 1 回。同梱バルーンは数個なので無視できる。
- **Follow-up**: `#[cfg(windows)]` のテスト。

### Decision: 正規化と検証の順序（§7-3、Req 2.1・2.2）
- **Selected Approach**: trim → 空 → （`directory` 代用時）区切りの有無 → 先頭区切り（ルート・UNC） → `:`（ドライブ・ドライブ相対・代替ストリーム） → `/`・`\` で分割し空要素と `.` を捨てる → `..` → 残りが空。
- **Rationale**: 区切りの種類に依存しない判定を分割前に済ませ、分割後は要素単位で判定する。`:` を一律に拒むことで `C:x` のようなドライブ相対も漏れない。末尾の区切り（`extra/bal1/`）は `source.directory` では許し、`directory` では区切りとして拒む（ukadoc の「1 階層の名前」）。

### Decision: 書く前に検証を確定させる（§7-4、§4.1）
- **Selected Approach**: `plan_bundled_balloons` は読み取り専用で、判定に関わるエラー（Req 1.10・2.1・2.3・2.5・7.2・7.5）をすべてここで返す。書き出し（`generate_update_files`）はその後。
- **Rationale**: エラー時に書きかけの `updates.txt` が配布フォルダに残らない。モジュール境界（読むだけ／書く）と一致する。
- **Trade-offs**: 前回の nar は既存どおり残る（OPEN QUESTION 2）。

### Decision: 番号付き指定の探索と同じキーの扱い
- **Selected Approach**: 番号なしを調べた後、`balloon0` から順に `.source.directory` か `.directory` の行がある番号を拾い、どちらも無い最初の番号で打ち切る。番号なしの有無は番号付きの探索に影響しない。同じキーが複数あるときは最初の行を採る。
- **Rationale**: ukadoc の探索順に従う（Req 1.4）。§2.5 のとおり areka は欠番で打ち切らないが、要件はベースウェア（ukadoc）に合わせると決めている。同じキーの扱いは SSP の挙動が未確認のため前提とし、OPEN QUESTION 4・5 に回す。

### Synthesis
- **Generalization**: ゴースト用とバルーン用の `updates.txt` は「基準フォルダ＋除外する相対フォルダ」の 1 つの操作の 2 つの呼び方として扱う（`collect_files(root, excluded)` → `generate_updates_txt`）。`install.txt` と `descript.txt` の読み取りは同じ `read_utf8_kv` を共有する。
- **Build vs. Adopt**: `install.txt` の `key,value` 形式は数行の標準ライブラリで足り、INI/CSV クレートは採らない（依存を増やさない方針。キーの大文字小文字・BOM・charset 行の扱いが独自）。
- **Simplification**: エラー型は新設せず `io::Error(InvalidData)` と既存の `Error:` 経路を使う。警告は `Vec<String>`。nar.rs・copy.rs は触らない。バルーン用には `ghost/master` 複製をしないので、既存の `generate_update_files` を再帰的に呼ぶのではなく内部の部品を直接呼ぶ。

## Risks & Mitigations
- 除外の追加でバルーン無しの出力が変わる — 除外の変更より前に、`date=` を伏せた全バイト比較の特性化テストを置く。
- UTF-8 宣言の必須化で既存の配布フォルダが止まる（pasta-in-windows など） — README・スキルに明記。下流の確認は OPEN QUESTION 6。
- 大文字小文字の違いで除外と生成先がずれる — 実在名への解決で一本化し、Windows でテスト。
- `profile/` 配下を指す同梱バルーンは nar に入らない — 要件の範囲外として受け入れる前提（OPEN QUESTION 7）。

## §7 の申し送りの解決状況
1. 戻り値の形と警告の表示 → 上記「戻り値の形と警告の表示」で決定
2. 相対パス一致の除外の渡し方と大文字小文字 → `collect_files(root, excluded_dirs)`、実在名への解決で決定
3. 値の正規化と検証の順序 → 上記で決定
4. 検証エラーを書き出しより前に確定させる流れ → 上記で決定
