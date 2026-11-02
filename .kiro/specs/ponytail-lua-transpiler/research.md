# ギャップ分析: ponytail-lua-transpiler

- 実施日: 2026-11-02（ワークツリーの base は main `ae04a28b`。この spec の領域は 2026-10-19 の監査の時点から変わっていない）
- 方法: brief.md の「進め方」のとおり、この spec の領域（`crates/pasta_lua/src` の `code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs`・`transpiler_tests.rs` と、`lib.rs` のそれらの再輸出）に `/ponytail:ponytail-audit` の手順をやり直した。読み込み中心で、cargo は走らせていない。未使用と書いたものは、リポジトリ全体（ほかのクレート・`tests/`・見本・`book/`・スキル）を grep して確かめた。
- 想定した負荷: 作者 1 人の辞書（数十〜数千行の `.pasta`）を、ゴーストの読み込みのたびに 1 本のスレッドで変換する。規模と並行性は問題にならない。

## 1. 監査のやり直しの結果と、控えとの突き合わせ

### 1.1 控え（brief の Current State）との違い

| 控えの番号 | 中身 | やり直しの結果 | 違い |
| ---------- | ---- | -------------- | ---- |
| 1 | Lua ブロックの字下げ（`element_gen.rs` 474〜492 行） | そのまま残る。`generate_code_block`（475 行から）が各行を `writeln` で出し、`do … end` の中の字下げ 1 段（空白 4 つ）が足される | 場所は 485〜490 行の繰り返し。関数の説明（457 行）の「without transformation」も事実と違う |
| 1 | 空行の削除（`normalize.rs` 108〜140 行） | そのまま残る（106〜146 行） | 根は 3 か所（下の 1.2 の 2） |
| 1 | ブロックの中の長い文字列のテストが無い | そのまま。ブロックのテストは `element_gen_tests.rs` 434〜457 行の 1 本だけで、字下げ 0 で長い文字列も空行も無い | なし |
| 2 | 死んだ `LineEnding`・`comment_mode` | そのまま。Windows の既定は CrLf だが、`write_line_terminator`（`mod.rs` 225 行）は常に `\n` を書き、`normalize` が CRLF を LF にするので、中間のバッファは混ざってから書き直される | 増えた: `TranspilerConfig::new` も `Default` の重複（テスト 1 か所だけが使う） |
| 3 | `TranspileContext` の死んだ状態 | そのまま。`current_module`（17・29〜36 行）、`register_global_scene`（41 行。本番は `_named` を使う）、`generate_global_scene` の `_context`（`scope_gen.rs` 112・120 行） | 増えた: `merge_from`（115〜126 行）が呼び手 0 になった（申し送り） |
| 4 | 素通し | そのまま。`with_defaults`（42 行）、`transpile_with_sink`（94 行。本番の呼び手は `transpile` だけ） | 増えた: `LuaTranspiler::config()`（テストだけ）、`normalize_output`（`normalize_output_with_shift(..).0` の包み。テストと doctest だけ） |

### 1.2 やり直しの指摘（番号は監査の報告の番号）

**直さなければならないもの**

1. **Lua ブロックの各行に空白 4 つが足され、長い文字列の値が変わる**（`code_gen/element_gen.rs` 485〜490 行）
   - 例: 次の辞書の `MSG` は、実行すると `"    こんにちは\n    end\n    "` になる（書いた中身は `"こんにちは\n\nend\n"`）。マニュアル（`book/src/grammar/block-structure.md` 156 行「そのまま生成される Lua コードに出力される」）とも違う。
     ```
     ＊会話
     ```lua
     local MSG = [[
     こんにちは

     end
     ]]
     ```
     　さくら：やあ
     ```
   - 最小の直し: ブロックの行は `write_raw(line)` と行末で出す（字下げを足さない）。1 行が 1 行のままなので、`record_block_line` と行の対応は変わらない。
2. **`normalize` が、次の空白でない行が `end` の空行を、Lua ブロックの中でも消す**（`normalize.rs` 106〜146 行）
   - 上の例の空行は、次の行が `end` なので消える。行の対応は `LineShift` が消した行を落とすので正しいが、文字列の中身は変わる。
   - 根: 生成器自身が `end` の前に空行を作る所が 3 つある。
     - `end_block`（`code_gen/mod.rs` 245〜250 行）が、どの `end` の後にも空行を書く。入れ子の `end` の後の空行の次に外側の `end` が来る。
     - `scope_gen.rs` 314 行が `init_scene` の後に空行を書く。項目の無いシーン（文法は `local_scene_item*` を許す）では、次の行が `end` になる。
     - 作者がブロックの末尾に書いた空行（シーンの閉じの `end` の直前）。
   - 最小の直し（brief の Approach）: 区切りの空行を「`end` の後」でなく「次の兄弟の前」に書く（項目が無いときは書かない）。そうすれば空行を消す規則は要らず、`normalize` は末尾を整えるだけになる。ブロックの外のスナップショットはバイト単位で同じに保てる見込み（要件 2.1）。

**直したほうがよいもの**

3. **Lua ブロックの中身がそのまま通ることを確かめるテストが無い**（`element_gen_tests.rs` 434〜457 行）。1 と 2 のどちらも、今のテストを赤にできない。シーンとアクター辞書の中のブロックに長い文字列と `end` の前の空行を置き、出力と行の対応を確かめるテストを 1 本足す（要件 7.1〜7.3）。
4. **アクター辞書の単語定義の行がソースマップに載らない。生成は `generate_word_definition` の写し**（`scope_gen.rs` 50〜72 行 と `element_gen.rs` 507〜541 行）【控えに無い・新規】
   - `generate_actor` は自前の繰り返しで `ACTOR:create_word("k"):entry(...)` を書き、`record_span` を呼ばない。グローバルとシーンの単語は `generate_word_definition` を通り、行ごとに記録される。`％さくら` の `　＠通常：\s[0]` の行には `.lua` の行が無く、ブレークポイントが結べない。
   - 2 つの写しは違う: アクターの方はキーを生の `"{}"` で書き、`literalize_with_span` を使う。共有の方は `literalize`（位置なし）を使う。
   - 最小の直し: 繰り返しを `generate_word_definition(word_def, "ACTOR", ":")` に替え、共有の方を `literalize_with_span` にする。文法の `id`（XID の文字）は `"` も `\` も含まないので、バイト列は同じ。約 20 行減る（要件 5・6.2）。
5. **改行とコメントの設定が効かない**（`config.rs` 7〜42・47〜57・65〜66・78〜94 行、`code_gen/mod.rs` 37〜38・69〜90・193・202 行）。控えの 2 番。`LuaCodeGenerator::new` と `with_line_ending` はほぼ同じ写しで、本番は `with_line_ending`（`transpiler.rs` 150 行）だけを使い、`new` は `tests/property_scope_codegen_test.rs` だけが使う（要件 3）。
6. **`TranspileContext` の使われない状態と関数**（`context.rs` 16〜17・28〜36・38〜43・116〜126 行、`scope_gen.rs` 112・114・120 行）。控えの 3 番と申し送りの `merge_from`。ファイルの属性の 3 つと `_file_attrs` は残す（brief の Out。今は `merge_attrs` の結果が、使われない `_file_attrs` に届くだけ）（要件 4.1・4.2・4.6）。
7. **テストだけが使う素通し**（`transpiler.rs` 41〜44・94〜106・270〜273 行、`normalize.rs` 30〜34 行）。控えの 4 番に `config()` と `normalize_output` が増えた（要件 4.3・4.4）。

**あるとよいもの**

8. **本番でシンクを付けない道があるかのような説明**（`code_gen/mod.rs` 23〜31・46〜50・94〜97・117・142 行、`code_gen/source_map.rs` 17〜20・49〜51 行、`transpiler.rs` 62〜67・78〜88・119〜127 行）【新規】。`transpiler.rs` 145〜151 行は常に `LineRecorder` を付けるので、どの変換でも行を記録する。「記録は働かない・費用 0」という説明は、行の対応を調べる人を迷わせる。約 60 行の説明が 1 文になる（要件 6.4）。
9. **「1 行書いて、行が進んだら位置を記録する」が 11 か所にある**（`element_gen.rs` 74・99・160・250・448・535 行、`scope_gen.rs` 43・142・269・405・436 行）【新規】。1 つの小さな関数にまとめれば約 30 行減り、直し漏れが無くなる（要件 6.3）。
10. **`generate_var_set` が変数の経路の規則を写している**（`element_gen.rs` 69〜71 行 と `resolve_var_path` 36〜43 行）【新規】。`var.{}`・`save.{}` を手で組み直している（要件 6.1）。

**確かめて指摘にしなかったもの**

- 生の `"{}"` の出力（`scope_gen.rs` 40・68・139・308 行、`element_gen.rs` 233 行）: 名前は文法の `id` か置き換え済みの名前なので安全。
- ローカルシーンの関数名: `sanitize_name` の結果に `_番号` が付く（予約語にならない）。LuaJIT は 0x80 以上のバイトを識別子に受け付ける。
- 文字列のリテラル化（範囲外、流し読み）: CR・LF のエスケープとロングブラケットの `=` の数え方は正しい。
- 行の対応: `fill_line_maps` と `LineShift` は、行を消した後のブロックの行を正しく写し直す。
- `choice_timeout` の 9.2e18 以上の浮動小数（`as i64` で飽和）、ブロックの行の中の孤立した CR（行の数がずれる）は、現実の入力ではない。
- 各 `init_scene` の行の対応は、同じグローバルシーンのすべてのブロックの行を繰り返す（ローカルシーンの数 × ブロックの行の数）。この負荷では最大でも数十 KB。
- 本番のローダとランタイムは `TranspileContext` を捨てる（`loader/process.rs` 197〜201 行、`loader/source_map_build.rs` 132 行、`runtime/factory.rs` 83 行）。シーンの通し番号は突合のキーに要る。外すと範囲外のテスト（約 39 か所）を組み直すことになるので指摘にしない。

判定: 1 と 2 を、テスト 3 といっしょに先に直す（作者の Lua の意味を黙って変える）。ほかは整理。本番のコードと説明で約 250 行、テストの呼び出しで約 60 か所が減らせる。依存は減らない。

## 2. 要件と資産の対応

| 要件 | 資産（ファイル・行） | ギャップ | 種別 |
| ---- | -------------------- | -------- | ---- |
| 1.1〜1.4 字下げしない・空行を消さない | `element_gen.rs` 475〜492 行（`generate_code_block`）、`normalize.rs` 106〜146 行 | 字下げを足す・空行を消す | Missing |
| 1.5〜1.6 出す位置と言語識別子 | `scope_gen.rs` 74〜83 行（アクター）・グローバルシーンの末尾 | 今のままでよい | なし |
| 2.1 ブロックを含まない辞書はバイト単位で同じ | `code_gen/mod.rs` 245〜250 行（`end_block`）・`scope_gen.rs` 78・170・178・314 行（`write_blank_line`）、見本 `sample.generated.lua`（Lua ブロックを含まない） | 空行の書き方を変えても同じ出力になるかは、設計で全スナップショットに当てて確かめる | Constraint |
| 2.2 ブロックの外の行の並び | スナップショット `transpiler__final_regression_test__r7_1_off_path__kind_code_block.snap` ほか、見本 `zero_cost_all_syntax.pasta`・`debug_break_coalesce.pasta`（Lua ブロックを含む） | 作り直しの差分を 2 種類に限って確かめる手順が要る | Constraint |
| 2.3〜2.4 行の対応 | `record_block_line`、`fill_line_maps`（`transpiler.rs` 308 行あたり）、`LineShift`（`normalize.rs`） | 中の空行を消さなくなると、`LineShift` の削除は末尾だけになる | Constraint |
| 2.5 LF・末尾の改行 1 つ | `normalize.rs` の末尾の整え | 今のままでよい | なし |
| 3.1〜3.4 設定の撤去 | `config.rs` 全体、`code_gen/mod.rs` 37〜90・193・202 行、`lib.rs` 50 行 | 撤去。テストの呼び出し約 25 か所（`with_line_ending(.., LineEnding::Lf)`）を 1 つの作り方へ | Missing |
| 4.1〜4.2 死んだ状態 | `context.rs` 16〜43・115〜126 行、`scope_gen.rs` 112〜121 行と呼び手 7 か所 | 撤去。`context.rs` のテスト 5 本ほどが消える | Missing |
| 4.3〜4.5 素通し | `transpiler.rs` 41〜44・94〜106・270〜273 行、`normalize.rs` 30〜34 行、`config.rs` 74 行 | 撤去。`transpile_with_sink` のテストの呼び出し約 30 か所を残る入口へ | Missing |
| 4.6 ファイルの属性は残す | `context.rs` 19・91〜114 行、`transpiler.rs` 159・238 行、`scope_gen.rs` 113・121 行 | 触らない | なし |
| 5.1〜5.3 アクターの単語定義 | `scope_gen.rs` 50〜72 行、`element_gen.rs` 507〜541 行 | 記録が無い・規則の写し | Missing |
| 6.1〜6.4 重なりと説明 | 上の 8〜10 | 整理 | Missing |
| 7.1〜7.7 テスト | `element_gen_tests.rs`・`tests/transpiler/*`・`final_regression_test.rs` の insta スナップショット | 新しいテスト 2〜3 本と、特性化（直す前に固める） | Missing |
| 8.1〜8.2 マニュアル | `book/src/internals/transpiler.md` 97・99〜102・326〜329・352〜356・357・376 行 | 直す。`block-structure.md` 156 行は保つ | Missing |
| 8.4 `pasta_core` の `merge_from` | `scene_registry.rs` 275 行・`word_registry.rs` 108 行（`ponytail-core-registry` の持ち場） | 入る順で決める（ロードマップの約束） | Constraint |
| 8.5 ほかの持ち場に触らない | `debug/source_map/mod.rs` 25 行の説明が `normalize_output` を名指ししている | 名前を消すと、他の持ち場の説明の 1 語が古くなる | Unknown |

## 3. 直し方の案

### 案 A: 今の部品を直す（推奨の候補）

- `generate_code_block` の 1 行を `write_raw`＋行末に替える。`end_block`・`write_blank_line` の呼び所を「次の兄弟の前に区切りの空行」に変え、`normalize` の空行を消す規則を消す。設定・状態・素通しは消すだけ。アクターの単語定義は共有の関数へ寄せる。
- 新しいファイルは無い。どれも削除が中心で、1,000 行に近いファイルは無い。
- ✅ 差分が最小で、brief の Approach と同じ。✅ 撤去は 1 つずつ独立なので、1 直し 1 コミットで戻せる。
- ❌ 空行の書き方を変える所は、スナップショット全部で「同じ出力」を確かめる必要がある（要件 2.1）。

### 案 B: `normalize` の規則を「Lua ブロックの範囲を除く」に狭める

- 生成器が Lua ブロックの行の範囲を `normalize` に渡し、その範囲の空行だけ消さない。
- ✅ 生成器の空行の書き方を変えないので、ブロックの外の出力は確実に今と同じ。
- ❌ 根（生成器が `end` の前に空行を作る）が残り、`normalize` と生成器の間に新しい受け渡しが増える。監査の「消す方を選ぶ」に反する。

### 案 C: 字下げだけ直し、空行は `normalize` の後で戻す

- ❌ 消してから戻すのは二重の手間で、行の対応の写し直しも 2 回になる。採る理由が無い。

案 A が brief の決定（Approach）と合う。案 B は、設計で案 A の「バイト単位で同じ」が保てないと分かったときの退き先になる。

## 4. 規模と危うさ

- 規模: **S〜M**（brief の見積もり 8〜10 タスク）。直す本体は数十行で、手間の多くは撤去に伴うテストの呼び出しの付け替え（約 60 か所）とスナップショットの作り直し。
- 危うさ: **低〜中**。直し方は既存の形の延長で新しい技術は無い。中にしたのは、行の対応（デバッガが読む）と、空行の書き方を変えたときのスナップショットの全数の一致を確かめる手間があるため。

## 5. 設計へ持ち越す調べもの（Research Needed）

1. 空行を「次の兄弟の前」に書く形で、Lua ブロックを含まない全スナップショットと `sample.generated.lua` がバイト単位で同じになるか（要件 2.1）。とくにアクター辞書だけのファイル・項目の無いシーン・ファイルの最後のシーン。
2. 中の空行を消さなくなった後、`LineShift` が記録するのは末尾の整えで消える行だけになる。`LineShift` と `normalize_output_with_shift` の形を保つか（持ち越し 3）。
3. `init_scene` の行の対応（シーン関数の頭の最後の引数）と、デバッグのソースマップの両方で、ブロックの行が同じ `.pasta` の行を指し続けることを、`debug_break_coalesce.pasta` などデバッグの見本の期待値で確かめる。
4. アクターの単語定義の行を記録したとき、デバッグのブレークポイントの解決（`ponytail-lua-debug` の持ち場）で、今のテストの期待値が変わらないか（増えるだけか）。
5. `pasta_lua` の版が上がるとキャッシュが作り直されること（`.cache_version`）を、直した版で古い字下げの変換結果が使われない根拠として確かめる。

## 6. 要件ディスカッションへの持ち越し（OPEN QUESTIONS）

1. **Lua ブロックを含まない辞書の出力は、バイト単位で今と同じにするか**（要件 2.1・2.2）
   - 仮定: 同じにする。ブロックを含む辞書は、ブロックの内容の行の字下げと消さなくなった空行の 2 種類だけが変わる。
   - 推奨: このまま。監査は、空行を「次の兄弟の前」に書けば同じにできると見ている。これを守れば、スナップショットの作り直しの差分が目で確かめられる量に収まり、行の対応のずれも起きない。保てないと設計で分かったら、案 B へ退く。
2. **アクター辞書の単語定義の行をソースマップに載せるか**（要件 5。ギャップ分析で増えた）
   - 仮定: 載せる。生成コードは変えない。
   - 推奨: 載せる。根は「アクターの単語定義だけが共有の規則を通らない」写しで、直しは写しを消すこと（約 20 行減る）。行の対応は増えるだけで、今の対応は変わらない。brief の「ギャップ分析で増えた同じ領域の指摘」に当たり、別の spec へ送る理由が無い。
3. **`LineShift` と `normalize_output_with_shift` の形を保つか**
   - 仮定: 保つ（要件の Out of scope）。`debug/` のソースマップ（`ponytail-lua-debug` の持ち場）と `loader/source_map_build.rs` が読むため。`debug/source_map/mod.rs` 25 行の説明の `normalize_output` の 1 語は、この spec では直さない。
   - 推奨: 保つ。直した後は、中の行を消さないので写しはほぼ恒等になるが、外すとこのウェーブの別の spec の持ち場（`debug/`）に入る。同じウェーブなので、`ponytail-lua-debug` の側で説明の 1 語を合わせてもらうか、後から入る側が合わせる。`LineShift` 自体を外すかは、このウェーブが入った後の棚卸で測る。
4. **小さな整理（監査の 8〜10）を要件に入れるか**（要件 6）
   - 仮定: 入れる。
   - 推奨: 入れる。brief の決めごと「その領域の監査の指摘が、残らず直っている」に当たる。どれも出力を変えず、1 直し 1 コミットで戻せる。
5. **`TranspilerConfig::new`・`LuaTranspiler::config()`・`normalize_output` の撤去**（要件 4.3〜4.5。控えに無かった素通し）
   - 仮定: 消す。
   - 推奨: 消す。控えの 4 番（`with_defaults`・`transpile_with_sink`）と同じ形の素通しで、使い手はテストだけ。公開クレートの pub API を消してよいのは 2026-10-19 の開発者の決定のとおり。
