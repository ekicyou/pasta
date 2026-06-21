# actor-poc 撤去手順と検証エビデンス（task 8.3 / R7.5）

> 本書は使い捨て PoC（`actor-poc` feature・default off）の**撤去手順の確定版**と、その手順を
> 適用したときに本体出荷物がバイト不変（1.1 ベースライン）へ戻ることの**検証エビデンス**である。
>
> **重要**: 本 PoC は本書の手順で「いつでも痕跡なく除去できる」ことを保証するためのものであり、
> 実際の除去は後続実装仕様 `pasta-actor-runtime` の移行完了時に行う（R7.5）。
> 本検証時点では PoC コードは作業ブランチに**保持**する。本検証は使い捨ての別 git worktree 上で
> 実施し、メインの作業ツリーからは actor_poc を一切削除していない。

## 1. 撤去対象の完全な集合

design.md「File Structure Plan / Modified Files」とリポジトリ実状（`grep -ri "actor.poc|actor_poc|wintf" crates/`）を
突き合わせて確定した、削除すべき actor-poc 追加物の完全な集合は以下のとおり。

### 1.1 モジュールディレクトリ（ディレクトリごと削除）

- `crates/pasta_lua/src/actor_poc/`（全ファイル）
  - `mod.rs`, `actor_thread.rs`, `r1_probe.rs`, `teardown.rs`, `mailbox.rs`, `responder.rs`,
    `coroutine_probe.rs`, `kick_harness.rs`, `sim_driver.rs`, `latency.rs`, `verdict.rs`, `test_isolation.rs`
- `crates/pasta_shiori/src/actor_poc/`（全ファイル）
  - `mod.rs`, `ffi_marshal.rs`

### 1.2 feature-gated テストファイル（ファイルごと削除）

- `crates/pasta_lua/tests/actor_poc_*.rs`（12 ファイル）
  - `actor_poc_isolation.rs`, `actor_poc_actor_thread.rs`, `actor_poc_teardown.rs`, `actor_poc_r1_probe.rs`,
    `actor_poc_responder.rs`, `actor_poc_coroutine_probe.rs`, `actor_poc_sim_driver.rs`,
    `actor_poc_kick_harness.rs`, `actor_poc_kick_latency.rs`, `actor_poc_latency.rs`,
    `actor_poc_verdict_stage.rs`, `actor_poc_integration.rs`
- `crates/pasta_shiori/tests/actor_poc_*.rs`（1 ファイル）
  - `actor_poc_marshal.rs`

### 1.3 `lib.rs` の cfg-mod 宣言（行削除）

- `crates/pasta_lua/src/lib.rs` — 以下 2 行を削除:
  ```rust
  #[cfg(feature = "actor-poc")]
  pub mod actor_poc;
  ```
- `crates/pasta_shiori/src/lib.rs` — 以下 2 行を削除:
  ```rust
  #[cfg(feature = "actor-poc")]
  pub mod actor_poc;
  ```

### 1.4 `Cargo.toml` の feature／依存（セクション・エントリ削除）

- `crates/pasta_lua/Cargo.toml`
  - `[features]` セクションごと削除（コメント＋ `actor-poc = ["dep:wintf-winmsg-executor", "windows-sys/Win32_System_Threading"]`）。
    これにより task 2.2 で `actor-poc` に追加された `windows-sys/Win32_System_Threading` feature も同時に除去される。
  - `[dependencies]` の optional dep 行を削除:
    ```toml
    # actor-poc feature 有効時のみリンクする使い捨て PoC 用 executor（公開フォーク）
    wintf-winmsg-executor = { version = "0.0.3", optional = true }
    ```
- `crates/pasta_shiori/Cargo.toml`
  - `[features]` セクションごと削除（コメント＋ `actor-poc = ["pasta_lua/actor-poc"]` 伝播）。

### 1.5 撤去対象から除外するもの（理由付き）

- **`dev-dependencies` の `ctor = "0.2"`（両クレート）は削除しない**。
  `ctor` は actor-poc 導入（commit `c5faac8a`）より前から両クレートの dev-dependencies に存在しており
  （debug 系テストの `#[ctor]` env ガードで使用）、actor-poc 追加物ではない。git 履歴で確認済み。
- **`.kiro/specs/pasta-actor-feasibility/` 配下（`baseline/`・`verdict-document.md`・`removal-procedure.md` を含む）は撤去対象ではない**。
  これらは PoC エンジンコードではなく、**本仕様の記録（spec artifact）**である。design.md が
  「本仕様は判定文書を成果物とし、本番実装は持たない」と定めるとおり、`verdict-document.md`（段階判定 GO+ の
  結論）と `baseline/`（R7.2 バイト不変の検証基盤）は後続 `pasta-actor-runtime` が着手前提として参照する
  spec の永続記録であり、エンジンコード撤去後も保持する。撤去＝「`actor_poc` エンジンコードと feature/依存を
  痕跡なく除去」であって「spec 記録の消去」ではない。

### 1.6 撤去後に確認すべき自動再生成物

- `Cargo.lock` は本リポジトリでは git 追跡対象外（gitignore）。`wintf-winmsg-executor` 等のロック
  エントリは次回ビルドで再解決され、追跡ファイルとしての痕跡は残らない。
- workspace ルート `Cargo.toml` には actor-poc 由来の記述は無い（確認済み）。

## 2. 撤去手順（コマンド列）

メイン作業ツリーを汚さないため、**現在のブランチ HEAD で使い捨ての linked worktree** を作って適用する。
（実際の本番撤去時はメインツリーで同じ編集を行う。）

```sh
# 1. 使い捨て worktree を現在の HEAD に作成（メインツリー配下を避ける）
git worktree add C:/home/maz/git/pasta/.actorpoc-removal-check <HEAD-sha>

cd C:/home/maz/git/pasta/.actorpoc-removal-check

# 2. モジュールディレクトリ・テストを削除
rm -rf crates/pasta_lua/src/actor_poc crates/pasta_shiori/src/actor_poc
rm -f  crates/pasta_lua/tests/actor_poc_*.rs crates/pasta_shiori/tests/actor_poc_*.rs

# 3. lib.rs ×2 から cfg-mod 宣言を削除（§1.3）
# 4. Cargo.toml ×2 から [features] と wintf optional dep を削除（§1.4）
#    （ctor は残す。§1.5）
```

## 3. 検証エビデンス

throwaway worktree（`C:/home/maz/git/pasta/.actorpoc-removal-check`、現ブランチ HEAD `1698be8b`）に
上記手順を適用して検証した結果。

### 3.1 痕跡ゼロ（grep clean）

`grep -ri "actor.poc|actor_poc|wintf" crates/` を throwaway worktree で実行 → **エンジンコードに一致ゼロ**。

```
(crates/ 配下に actor-poc / actor_poc / wintf の一致は 0 件)
```

worktree 全体でのヒットは全て `.kiro/`（spec ドキュメント・steering）配下のみで、エンジンコードには痕跡が残らない。
`Cargo.lock` は git 追跡外（撤去後に再生成され痕跡なし）、ルート `Cargo.toml` にも actor-poc 記述なし。

### 3.2 クリーン release ビルド成功

```
Remove-Item Env:\NoDefaultCurrentDirectoryInExePath -ErrorAction SilentlyContinue;
cargo build --release -p pasta_lua -p pasta_shiori
→ Finished `release` profile [optimized] target(s) in 55.71s   （エラー・警告なし）
```

撤去後のツリーが整合（actor_poc 参照の取りこぼしなし）であることを実証。

### 3.3 バイト不変検証（pasta.dll が 1.1 ベースラインへ復帰）

`capture_baseline.ps1 -Mode verify` を throwaway worktree の `target/release` に対して実行:

```
--- Authoritative byte-invariance check (R7.2 shipped artifact) ---
[OK ] pasta.dll  normalized adae6b39055f9f819ce38f4dc483214725e6084ee0d96faa29a5783199e24b24
                  (baseline  adae6b39055f9f819ce38f4dc483214725e6084ee0d96faa29a5783199e24b24)

Byte-invariance verification PASSED (1 authoritative artifact(s) match normalized baseline; rlibs informational).
プロセス終了コード = 0
```

- **出荷成果物 `pasta.dll` の正規化 sha256 が 1.1 ベースライン `adae6b39…` と完全一致**（authoritative・exit 0）。
- rlib（`libpasta.rlib` / `libpasta_lua.rlib`）の whole-file sha は差分ありだが、これは
  symbol table / crate SVH fingerprint 由来の非決定差分であり（baseline.json / `verify_8_1_result.md` に既述）、
  informational 扱いで合否に算入しない。`pasta.dll` の `.text`/`.rdata` は逐語ハッシュであり、実コード変更が
  あれば必ず検出されるが、撤去後も一致している＝actor-poc が出荷コードへ一切漏れていないことの証左。

## 4. 結論

撤去手順（§1〜§2）を適用後:

- **(a)** エンジンコードから actor-poc 関連の痕跡が完全に消える（grep ゼロ）。
- **(b)** 撤去後のツリーがクリーン release ビルドに成功する（整合性あり）。
- **(c)** 出荷物 `pasta.dll` の正規化ダイジェストが task 1.1 のベースライン（`adae6b39…`）へバイト不変で復帰し、
  `capture_baseline.ps1 -Mode verify` が exit 0／`pasta.dll [OK]` を返す。

以上より **R7.5（使い捨て・恒久統合を残さない）** および **R7.2（無効時バイト不変＝撤去後の復帰）** が
満たされることを確認した。検証は使い捨て worktree で実施し、検証完了後に worktree を除去。メイン作業ツリーの
`actor_poc` コードは未変更で保持されている（実際の除去は `pasta-actor-runtime` 移行完了時に実施・R7.5）。
