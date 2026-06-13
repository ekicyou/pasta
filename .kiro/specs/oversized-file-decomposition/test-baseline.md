# テスト棚卸しベースライン（oversized-file-decomposition）

> **目的**: 本仕様は純粋リファクタリング（振る舞い不変）であり、唯一の安全網は「テスト集合の不変性」である。
> 本ファイルは Task 1.1 で捕捉した**着手前ベースライン**を記録する。C1/C2 の各移動ステップ後にこのベースラインと差分ゼロ
> （`cargo test --workspace -- --list` のテスト関数集合・総数が不変）であることを確認すること。
> 唯一の許容される後続追加は C4 の順序固定特性化テスト **ちょうど 1 本**（requirements.md 4.5 / design.md「Characterization Test」）。
>
> - 捕捉日: 2026-06-14
> - ワークツリー / ブランチ: `claude/upbeat-newton-4b1cc6`
> - 捕捉時 HEAD: `8d1fce7`
> - 対応要件: requirements.md 1.7・5.1・5.3・5.4／design.md「V — Staged Verification」「Test Inventory Baseline」「File Structure Plan」／research.md §2.1

---

## 1. 環境ワークアラウンド（requirements.md 5.4・必須前提）

LuaJIT / mlua-sys ビルドのため、**すべての `cargo` 実行の前に環境変数 `NoDefaultCurrentDirectoryInExePath` を無効化（unset）する**こと。
無効化しないと mlua-sys / LuaJIT ビルドが exit 101 で失敗する（本環境では当該変数が SET 済み）。

### カノニカル検証コマンド（段階的検証 V で毎ステップ実行）

`/usr/bin/bash`（Bash ツール）で実行する。各コマンドは `unset` を前段に置く:

```bash
# ビルド検証
unset NoDefaultCurrentDirectoryInExePath; cargo build --workspace

# テスト検証（全ワークスペース・クレート単位への簡略化禁止: requirements.md 5.1）
unset NoDefaultCurrentDirectoryInExePath; cargo test --workspace

# テスト集合スナップショット（移動不変の差分確認用・C1/C2 各ステップ後に再取得）
unset NoDefaultCurrentDirectoryInExePath; cargo test --workspace -- --list
```

**確認済み（2026-06-14）**: `unset` 後に `cargo build --workspace` / `cargo test --workspace` の両方が green。
`unset` を省略すると LuaJIT ビルドが exit 101 で失敗する（既知の落とし穴）。

---

## 2. ベースライン総数（不変条件・最重要）

### 2.1 `cargo test --workspace -- --list` の集計

| 指標 | 値 |
|---|---:|
| **`--list` が列挙するテスト関数の総数（不変条件）** | **2021** |
| うちユニーク leaf テスト名（最終 `::` セグメント） | 1997 |
| テストバイナリ数（ハッシュ除去後の安定名） | 63 |
| benchmark 行 | 0 |

> **不変条件**: この **2021** という総数は、本仕様の全工程（C4 の特性化テスト 1 本追加を除く）を通じて不変でなければならない。
> 兄弟ファイル外出し（C1）は単一移動ではフルパス（`mod` パス）も保たれるが、クラスタ分割は `mod` 名プレフィクスが変わる。
> よって **leaf テスト関数名**（最終 `::` セグメント）と**総数**の双方を記録し、後続で差分する
> （総数は不変・leaf 名集合も不変。後続で唯一許容される追加は C4 の特性化テスト 1 本）。

### 2.2 `cargo test --workspace`（実行）の結果 — green エビデンス

| 指標 | 値 |
|---|---:|
| passed | 2010 |
| failed | **0** |
| ignored（`#[ignore]`） | 11 |
| **実行ステータス** | **ok（全 65 テストバイナリ green・exit 0）** |

> `--list` の 2021 = 実行時の 2010 passed + 11 ignored。`--list` は `#[ignore]` 付きテストも列挙するため総数 2021 に含まれる。
> ignored 11 本の内訳: あるバイナリで 2 本、別バイナリで 9 本（`test result: ok. … N ignored` で観測）。

### 2.3 doc-test（参考・`--list` 非対象・本仕様の移動対象外）

`cargo test --workspace` は別途 doc-test も実行する（`--list` には現れない）:

| クレート | doc-test 数 |
|---|---:|
| pasta_core | 1 |
| pasta_dsl | 6 |
| pasta_lsp | 0 |
| pasta_lua | 12 |
| pasta_sample_ghost | 0 |
| pasta | 0 |
| **合計** | **19** |

> doc-test はソース外出し/クラスタ分割の対象外（本番 doc コメント内）であり、再配置で増減しない想定。差分監視の主対象は §2.1 の 2021 関数集合。

---

## 3. テストバイナリ別テスト数（安定名・差分の基準表）

> バイナリ名末尾の `-<16桁ハッシュ>` はビルド毎に変わるため除去した安定名。後続ステップで `cargo test --workspace -- --list` を
> 再取得し、本表の件数（合計 2021）と照合する。移動はバイナリ内・バイナリ間で件数を保存する（純粋移動）。

| # | 安定バイナリ名 | テスト数 |
|---:|---|---:|
| 1 | pasta_lua（unittests src/lib.rs） | 614 |
| 2 | runtime | 175 |
| 3 | loader | 140 |
| 4 | transpiler | 120 |
| 5 | pasta（unittests src/lib.rs） | 94 |
| 6 | pasta_core（unittests src/lib.rs） | 78 |
| 7 | shiori | 73 |
| 8 | cue_cmd_test | 63 |
| 9 | sakura_script | 59 |
| 10 | pasta_check（unittests src/main.rs） | 59 |
| 11 | log | 33 |
| 12 | search | 32 |
| 13 | partial_parse_test | 31 |
| 14 | lua_request_test | 29 |
| 15 | word_table_test | 26 |
| 16 | shiori_test_env_test | 24 |
| 17 | pasta_sample_ghost（lib 20 + main 4） | 24 |
| 18 | pasta_lsp（unittests src/lib.rs） | 22 |
| 19 | ast_test | 20 |
| 20 | property_scope_test | 16 |
| 21 | parser_test | 16 |
| 22 | error_api_test | 16 |
| 23 | choice_line_test | 16 |
| 24 | analysis_test | 15 |
| 25 | expr_parse_test | 14 |
| 26 | span_byte_offset_test | 13 |
| 27 | utf16_conversion_test | 12 |
| 28 | async_callback_integration_test | 12 |
| 29 | var_set_token_test | 10 |
| 30 | property_scope_codegen_test | 10 |
| 31 | integration_test | 10 |
| 32 | cue_command_token_test | 10 |
| 33 | shiori_response_test | 9 |
| 34 | semantic_token_test | 9 |
| 35 | var_set_none_test | 8 |
| 36 | sakura_symbol_tag_test | 7 |
| 37 | japanese_identifier_test（pasta_lsp 5 + pasta_lua 2） | 7 |
| 38 | dynamic_call_test | 7 |
| 39 | partial_token_test | 6 |
| 40 | diagnostics_test | 6 |
| 41 | code_block_token_test | 6 |
| 42 | cli_test | 6 |
| 43 | shiori_lifecycle_test | 5 |
| 44 | property_token_preservation_test | 5 |
| 45 | loader_source_map_build_test | 5 |
| 46 | fullwidth_halfwidth_test | 5 |
| 47 | pasta_dsl（unittests src/lib.rs） | 4 |
| 48 | lsp_lifecycle_test | 4 |
| 49 | document_sync_test | 4 |
| 50 | digit_id_var_test | 4 |
| 51 | crash_recovery_test | 4 |
| 52 | build_determinism_test | 4 |
| 53 | var_set_multibyte_panic_test | 3 |
| 54 | ucid_test | 3 |
| 55 | self_deploy_integration_test | 3 |
| 56 | actor_code_block_test | 3 |
| 57 | shiori_sample_ghost_test | 2 |
| 58 | string_buffer_availability_test | 1 |
| 59 | ontalk_probe_test | 1 |
| 60 | lua_unittest_runner | 1 |
| 61 | dist_src_validation_test | 1 |
| 62 | chunk_name_validation_test | 1 |
| 63 | analyze_robustness_test | 1 |
| | **合計** | **2021** |

> 注: ハッシュ除去で 2 つのバイナリ名が統合された — `pasta_sample_ghost`（lib 20 + main 4 = 24）と
> `japanese_identifier_test`（pasta_lsp の binary 5 + pasta_lua の binary 2 = 7）。実体は別バイナリだが、
> 合計件数 2021 の差分監視には影響しない。

---

## 4. テスト識別子（leaf 名）の捕捉方法と再取得手順

完全な識別子リスト（バイナリ修飾の 2021 行・leaf 名の 1997 種マルチセット）は、下記コマンドで決定論的に再生成できる
（生データを本ファイルに丸写しせず、再現可能な抽出手順を SSOT として残す）:

```bash
# 0) 前提: env 無効化
unset NoDefaultCurrentDirectoryInExePath

# 1) --list を stdout+stderr 結合で取得（"Running ... .exe)" 行と "ID: test" 行を順序保存で取り込む）
cargo test --workspace -- --list > /tmp/list.txt 2>&1

# 2) 各テストを所属バイナリへ帰属（バイナリ名 \t テストID）
awk '
  /Running / { s=$0; p=index(s,"deps"); if(p>0)s=substr(s,p+5); q=index(s,".exe)"); if(q>0)bin=substr(s,1,q-1); next }
  /Doc-tests / { next }
  /: test$/ { id=$0; sub(/: test$/,"",id); print bin "\t" id }
' /tmp/list.txt > /tmp/by_binary.txt

# 3) ハッシュ除去 → 安定名化
sed -E 's/^([^\t]*)-[0-9a-f]{16}\t/\1\t/' /tmp/by_binary.txt > /tmp/by_binary_stable.txt

# 4) 総数（=2021 であること） / バイナリ別件数 / leaf 名マルチセット
wc -l < /tmp/by_binary_stable.txt                                  # → 2021
cut -f1 /tmp/by_binary_stable.txt | sort | uniq -c | sort -rn      # → §3 の表
sed -E 's/.*:://' /tmp/by_binary_stable.txt | sort | uniq -c       # → leaf 名マルチセット（1997 種・合計 2021）
```

### 後続ステップでの差分手順（C1/C2 各移動後）

1. 上記 1〜3 を再実行し `/tmp/by_binary_stable.txt` を再生成。
2. **総数チェック**: `wc -l` が **2021** のまま（C4 特性化テスト追加後は 2022）であること。
3. **leaf 名集合チェック**: `sed -E 's/.*:://' … | sort | uniq -c` の出力が移動前と一致すること
   （クラスタ分割では `mod` プレフィクスが変わるが leaf 名と件数は不変）。
4. **件数の合計不変**: §3 の合計 2021 が保たれること（バイナリ内/間の移動は合計を保存）。
5. 差分が出た場合は「移動先 `mod` 登録漏れ」「テスト改名/欠落」を疑い、当該ステップ内で是正（requirements.md 5.2）。

> leaf 名マルチセットで件数 ≥2 の代表例（重複 leaf 名・別バイナリ/別 mod に同名が存在）:
> `test_transpile_basic_scene`（×8）、`test_create_runtime_with_finalize_succeeds`（×8）、
> `test_search_word_deterministic_with_mock_selector`（×2）、`test_default_config`（×2）等。
> 重複は正常（別モジュール/別バイナリの同名テスト）。差分はマルチセット（件数付き）で判定すること。

---

## 5. 是正対象ファイル集合（requirements.md 1.7・research.md §2.1 再スキャン照合）

### 5.1 照合結果

着手時の独立再スキャン（`crates/*/src/**` で**トップレベル `#[cfg(test)] mod NAME { … }`（インライン）を持つファイル** ＝ 外出し対象、
および 600 行超の純本番ファイル ＝ 責務分割対象）を実施し、research.md §2.1 の確定インベントリと突合した。

- **600 行超ファイルの再スキャン結果は research.md §2.1 の 28 件と完全一致**（ファイル・行数とも。drift 反映済みの `transport.rs` 1340・`debug/mod.rs` 1288・`debug_integration_test.rs` 758 を含む）。§2.1 は 2026-06-14 時点で正確。
- **インライン `#[cfg(test)] mod` を持つ src 本番ファイルは全 64 件**（うち外出し済み `mod tests;` 形式は 0 件 — `debug/` 含め全件がインライン保持）。前例の `scene_table.rs`・`shiori.rs` は既に `#[path]` 外出し済みのため本集合には含まれない。
- C3 純本番対象 `visitors.rs`(996)・`loader/mod.rs`(718)・`runtime/mod.rs`(635) は**インライン `#[cfg(test)]` を 0 件保持**（純本番）であることを確認。

### 5.2 是正対象集合（和集合 = §2.1 最低保証 ∪ 再スキャン検出）

#### C1 — インラインテスト外出し対象（src 本番・600 行超／規約準拠観点で同居解消）

| 行数 | ファイル | 備考 |
|---:|---|---|
| 5539 | `crates/pasta_lua/src/debug/wiring.rs` | トップレベル `#[cfg(test)] mod` × **11**（C1+C4。research §2.3 のクラスタ群） |
| 3090 | `crates/pasta_lua/src/debug/session.rs` | |
| 1818 | `crates/pasta_lua/src/debug/dap.rs` | C1+C3 |
| 1474 | `crates/pasta_lua/src/debug/source_map.rs` | |
| 1426 | `crates/pasta_lua/src/debug/inspect.rs` | |
| 1340 | `crates/pasta_lua/src/debug/transport.rs` | drift +496（§2.1 で更新済） |
| 1288 | `crates/pasta_lua/src/debug/mod.rs` | C1+C3（drift +180） |
| 981 | `crates/pasta_lua/src/code_gen/element_gen.rs` | |
| 798 | `crates/pasta_lua/src/loader/config.rs` | |
| 766 | `crates/pasta_lua/src/transpiler.rs` | |
| 759 | `crates/pasta_lua/src/debug/hook.rs` | |
| 665 | `crates/pasta_lua/src/code_gen/scope_gen.rs` | |
| 654 | `crates/pasta_lua/src/loader/discovery.rs` | |
| 648 | `crates/pasta_shiori/src/windows.rs` | |
| 605 | `crates/pasta_lua/src/loader/extract.rs` | |

> 600 行未満だが規約未準拠（インライン同居）の任意外出し候補: `debug/breakpoints.rs`(591)・`debug/types.rs`(567) 他多数。
> requirements.md 1.6 のバイナリ完了判定は「是正対象クレートの src 本番にインライン `#[cfg(test)] mod` 残存 0」であり、
> design.md「File Structure Plan」が確定対象を規定する。全 64 インライン保持ファイルの一覧は §4 の再スキャン手順で随時再取得可能。

#### C2 — 巨大テストファイル分割対象（600 行超・`tests/` ＋ 既外出し `#[path]`）

| 行数 | ファイル |
|---:|---|
| 1612 | `crates/pasta_lua/tests/runtime/runtime_toggle_e2e_test.rs` |
| 1023 | `crates/pasta_shiori/src/shiori_tests.rs`（既 `#[path]`） |
| 961 | `crates/pasta_dsl/tests/cue_cmd_test.rs` |
| 817 | `crates/pasta_shiori/tests/async_callback_integration_test.rs` |
| 808 | `crates/pasta_core/src/registry/scene_table_tests.rs`（既 `#[path]`） |
| 804 | `crates/pasta_lua/tests/loader/config_test.rs` |
| 758 | `crates/pasta_lua/tests/runtime/debug_integration_test.rs`（§2.1 で追加された drift） |
| 739 | `crates/pasta_shiori/tests/lua_request_test.rs` |
| 635 | `crates/pasta_lua/tests/transpiler/record_wiring_test.rs` |
| 605 | `crates/pasta_lua/tests/shiori/virtual_event_config_test.rs`（僅少超過・据え置き候補） |

#### C3 — 純本番責務分割対象（600 行超・インライン `#[cfg(test)]` 0）

| 行数 | ファイル |
|---:|---|
| 996 | `crates/pasta_lsp/src/analysis/visitors.rs` |
| 718 | `crates/pasta_lua/src/loader/mod.rs` |
| 635 | `crates/pasta_lua/src/runtime/mod.rs` |
| （C1 後確定） | `crates/pasta_lua/src/debug/dap.rs` 本番残・`crates/pasta_lua/src/debug/mod.rs` 本番残 |

#### C4 — `handle_inbound` 解体対象

| ファイル | 備考 |
|---|---|
| `crates/pasta_lua/src/debug/wiring.rs` | `handle_inbound`（free fn @ ~280 行）＋特性化テスト 1 本（唯一の許容追加） |

> **着手時注意（research §2.3 / §6.4）**: 行番号は drift する。各ステップ着手時に `wiring.rs` の `mod` 行番号と
> `setBreakpoints` 原子境界の行範囲を再取得すること（命名は安定）。本ベースラインの行数は 2026-06-14 / HEAD `8d1fce7` 時点。

---

## 6. まとめ（Task 1.1 成果）

- env ワークアラウンド（`NoDefaultCurrentDirectoryInExePath` 無効化）を確認・文書化（§1）。
- ベースライン総テスト関数数 = **2021**（実行: 2010 passed / 0 failed / 11 ignored・全 green）を捕捉（§2）。
- バイナリ別件数表（合計 2021）と leaf 名集合の決定論的再取得手順を確立（§3・§4）。
- research.md §2.1 確定インベントリを再スキャンで照合し、和集合を是正対象集合として確定（§5）。28 件の 600 行超リストは §2.1 と完全一致。
