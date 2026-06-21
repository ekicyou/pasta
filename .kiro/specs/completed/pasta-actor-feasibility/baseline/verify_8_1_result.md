# Task 8.1 — バイト不変検証（actor-poc 無効）結果記録

Requirement 7.2: `actor-poc` 無効の release ビルド成果物が、actor-poc 導入前
ベースライン（task 1.1）と**バイト不変**であることの再現可能なエビデンス。

- 検証日時 (UTC): 2026-06-21
- toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25)` / `cargo 1.96.0`（rust-toolchain ピン無し）
- release profile: `opt-level=z, lto=true, codegen-units=1, panic=abort, strip=true`
- ベースライン commit（actor-poc 導入前ソース）: `1e94b34a`
- 検証対象（現行 feature-off）commit: `a428cc8c`（7.1 完了時点）

## 結論

**出荷成果物 `pasta.dll`（pasta_shiori cdylib）の正規化 sha256 が 1.1 ベースラインと完全一致。**
**→ R7.2 のバイト不変（出荷成果物）は GREEN。検証スクリプト `-Mode verify` は exit 0。**

検証スクリプトは初回 8.1 で rlib whole-file exact チェックにより誤って exit 1 を返していたが、
当該チェックが provably 非決定（pre-actor-poc ソースでも失敗）であったため、リメディエーション
として権威基準を出荷 `pasta.dll` normalized に一本化し、rlib を informational に降格した（§0）。
`pasta.dll` の normalized 比較・baseline 値・zeroed_ranges は無改変。

非出荷の中間 rlib 2 種は正規化 sha が一致しない（後述）。ただしこれは actor-poc の
コードが feature-off ビルドに混入したためではなく、(a) rlib の `lib.rmeta`／symbol
table に Cargo.toml の feature／optional-dep 宣言が反映されること、(b) crate の
metadata fingerprint（SVH）が宣言済み feature 集合の関数であり、feature OFF でも
symbol mangling 経由で 4 バイト変動すること、による **metadata/identity 由来の差分**
である。**出荷 DLL のコンパイル済みコードは不変**であることを object コード単位で実証済み。

## 0. 検証スクリプト修正（8.1 リメディエーション）

初回 8.1 試行では `pasta.dll` の実質的バイト不変（normalized sha 一致）は証明済みだったが、
`capture_baseline.ps1 -Mode verify` が rlib の **whole-file exact sha** を hard-fail として
exit 1 を返していた。この rlib whole-file 一致チェックは **pre-actor-poc ベースラインソース
`1e94b34a` でも失敗する**（§2a で実証）provably 非決定なチェックであり、隔離ゲートが恒久的に
RED となって R8.3（隔離前提の判定妥当性）を毀損していた。

そこで本リメディエーションで verify ロジックを以下のとおり修正した（出荷ソース・`pasta.dll`
の normalized 比較ロジック・baseline 値・zeroed_ranges は一切変更していない）:

- **権威ある不変基準を出荷成果物 `pasta.dll` の normalized sha256 に一本化。** verify の合否
  （exit code）は `authoritative=true` の成果物（= `pasta.dll`）の normalized 一致のみで決まる。
  これは R7.2 が言う「本体のリリースビルド成果物（出荷 SHIORI DLL）」のバイト不変そのもの。
- **rlib 2 種を informational に降格。** whole-file sha を報告するが exit code には算入しない。
  rlib whole-file は symbol table / crate SVH fingerprint 由来で非決定（§2a）であり、バイト
  不変の基準たりえないことを出力・`baseline.json`・スクリプトヘッダに明記した。
- `baseline.json` は `pasta.dll` を `authoritative: true`、rlib を `authoritative: false` /
  `reproducible: informational` に注記更新。`pasta.dll` の `normalized_sha256`（`adae6b39...`）と
  `zeroed_ranges` は **無改変**（git diff で確認済み）。

修正後 verify は exit 0（下記 §1）。

## 1. 出荷成果物 pasta.dll の検証（コア証拠・修正後 verify は GREEN）

`capture_baseline.ps1 -Mode verify`（PE 非決定領域＝TimeDateStamp/CheckSum/DebugDir
timestamps/RSDS GUID をゼロ埋め後の normalized sha256 で比較。合否は `pasta.dll` のみで判定）。

クリーン feature-off ビルド（`cargo build --release -p pasta_lua -p pasta_shiori`、
`--features actor-poc` 無し）後の実行結果:

```
--- Authoritative byte-invariance check (R7.2 shipped artifact) ---
[OK ] pasta.dll          normalized adae6b39055f9f819ce38f4dc483214725e6084ee0d96faa29a5783199e24b24 (baseline adae6b39055f9f819ce38f4dc483214725e6084ee0d96faa29a5783199e24b24)

--- Informational (rlib whole-file is non-deterministic: symbol table / crate SVH fingerprint; NOT a byte-invariance basis) ---
[INFO] libpasta.rlib      whole-file DIFFERS  sha256=ec90520d... (baseline d67cb3c8...)
[INFO] libpasta_lua.rlib  whole-file DIFFERS  sha256=df12d550... (baseline 6ce8c676...)

Byte-invariance verification PASSED (1 authoritative artifact(s) match normalized baseline; rlibs informational).
```

`pwsh -NoProfile -File capture_baseline.ps1 -Mode verify` の **exit code = 0**（別プロセス実行で確認）。

- size = 4031488（baseline と一致）
- 2 回のクリーン feature-off ビルドで normalized 一致を再現（reproducible）。
- ベースライン commit `1e94b34a` のソースからのビルドでも同じ normalized
  `adae6b39...` を得る（=出荷 DLL は actor-poc Cargo.toml 追加の前後で不変）。

### グリーンチェックが実変更を依然検出する理由（meaningfulness）

`pasta.dll` の normalize はリンクメタデータの **非決定 20 バイトのみ**（COFF
TimeDateStamp / Optional Header CheckSum / DebugDir TimeDateStamp / CodeView RSDS GUID）を
ゼロ埋めする。`.text`／`.rdata` をはじめコンパイル済みコードのバイト範囲は一切ゼロ埋め
しない（`zeroed_ranges` は無改変、計 6 エントリ＝上記メタのみ）。したがって出荷コードに
実変更が入れば normalized sha は必ず変動し verify は FAIL する。緑チェックは「リンク時刻
揺らぎを無視しつつ、コード実体の変化は確実に検出する」honest なゲートである。

## 2. 非出荷 rlib の差分分析（honest disclosure）

`baseline.json` は rlib を exact sha256 で比較するが、現行 feature-off ビルドは不一致:

| artifact | baseline sha (exact) | current feature-off sha | size |
|---|---|---|---|
| libpasta.rlib | d67cb3c8… | ec90520d… | 2270136 → 2270186 (+50) |
| libpasta_lua.rlib | 6ce8c676… | df12d550… | 7908898 → 7908980 (+82) |

### 2a. rlib は本環境で whole-file 再現しない（actor-poc とは無関係）

ベースライン commit `1e94b34a`（actor-poc を Cargo.toml に**一切含まない**）のソースを
独立 worktree でクリーンビルドしても、committed baseline.json の rlib sha を再現しない:

```
libpasta.rlib      baseline-src build = 001f067b…  (size 2269516)  ≠ committed d67cb3c8… (2270136)
libpasta_lua.rlib  baseline-src build = dcc531a5…  (size 7905586)  ≠ committed 6ce8c676… (7908898)
```

→ rlib の whole-file sha はビルド環境・セッションをまたぐと変動する（1.1 note の
「rlib は exact 再現」は object コードについては正しいが whole-file には当てはまらない）。
したがって rlib whole-file の不一致は actor-poc の混入を意味しない。

### 2b. object コード（COFF member）単位の比較で混入なしを実証

rlib（ar アーカイブ）の object コード member を抽出して比較:

- **libpasta.rlib の object コード member は baseline-src ビルドと feature-off ビルドで
  完全に bit 一致**（sha `d3479f95…`、size 860692）。差分は `lib.rmeta`（+670 bytes、
  feature/dep 宣言メタデータ）と symbol table のみ。
- libpasta_lua.rlib の object コード member は **4 バイトだけ**差分。差分箇所は COFF
  ファイルヘッダ offset 8 の `PointerToSymbolTable`（baseline `0x0040a1f3` → feature-off
  `0x0040a1f7`、＝+4）で、section data に 4 バイト挿入されシンボルテーブルが後方シフト
  したことを示す。挿入の実体は crate metadata fingerprint（SVH）が symbol mangling の
  `17h<hash>E` 接尾辞に反映されたもので、宣言済み feature 集合が変わると feature OFF でも
  変動する rustc の identity 由来。**actor_poc モジュールは `#[cfg(feature = "actor-poc")]`
  で除外されコードを emit しない**（feature off ビルドに actor-poc コードは無い）。

- object コード member の決定性確認: baseline-src を 3 回クリーンビルドしても object
  member sha は安定（libpasta_lua = `ef30…`、libpasta = `d3479f95…`）。
  ※ §2 の current 値はメイン worktree 側 feature-off ビルド、§2a/§2b の baseline-src 値は
  ベースライン commit の独立 worktree ビルド。

## 判定

- **R7.2（出荷成果物のバイト不変）: GREEN** — `pasta.dll` normalized sha256 が 1.1
  ベースラインと完全一致、2 回再現。`capture_baseline.ps1 -Mode verify` は **exit 0**。
- rlib whole-file の差分は (i) feature/dep 宣言メタデータ＋(ii) crate fingerprint 由来の
  metadata-only delta であり、出荷 DLL の不変性を損なわない。verify ではこれを informational
  として報告し合否に算入しない。
- ベースライン JSON の `pasta.dll` normalized 値・zeroed_ranges は無改変（pre-actor-poc
  基準として保持）。rlib エントリは再現性の基準が「informational / 非決定 whole-file」である
  ことを注記更新した（誤った "exact" 注記の訂正）。
- 隔離ゲート（R8.3 判定妥当性前提）は恒久 GREEN となり、誤検出由来の RED が解消した。
