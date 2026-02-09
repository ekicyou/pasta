# Rust LSP + WASM 互換性 技術リサーチレポート

> 調査日: 2026-02-09  
> 対象: `wasm32-unknown-unknown` ターゲットでの Rust LSP サーバー構築可能性

---

## 1. pest crate WASM 互換性

### 結論: ✅ **互換性あり（`no_std` + `default-features = false`）**

| 項目                     | 詳細                              |
| ------------------------ | --------------------------------- |
| バージョン               | pest 2.8.x                        |
| `no_std` サポート        | ✅ 公式サポート（README に明記）   |
| `wasm32-unknown-unknown` | ✅ コンパイル可能                  |
| 条件                     | `default-features = false` が必要 |

**設定例:**
```toml
[dependencies]
pest = { version = "2.8", default-features = false }
pest_derive = { version = "2.8", default-features = false }
```

**注意点:**
- `pest` と `pest_derive` の両方で `default-features = false` を指定すること
- `std` feature はファイルI/O関連の機能を提供するが、パーサー自体は `no_std` で動作する
- `pest_meta`（文法の解析・最適化）も pure Rust で OS 依存なし
- **pasta_dsl で使用中の pest はそのまま WASM で利用可能**

---

## 2. thiserror crate WASM 互換性

### 結論: ✅ **互換性あり（`no_std` ネイティブ対応）**

| 項目                     | 詳細                                              |
| ------------------------ | ------------------------------------------------- |
| バージョン               | thiserror 2.x                                     |
| `no_std` サポート        | ✅ `#![no_std]` で宣言済み                         |
| `wasm32-unknown-unknown` | ✅ コンパイル可能                                  |
| カテゴリ                 | crates.io で「No standard library」カテゴリに登録 |

**根拠:**
- `thiserror` 2.x の `src/lib.rs` は `#![no_std]` で宣言されている
- `core::error::Error` を使用（Rust 1.81+ で安定化）
- `std` feature はオプショナルで、`Path`/`PathBuf` のDisplay対応のみ
- `tests/no-std/test.rs` で `#![no_std]` テストが存在
- `build.rs` で `core::error::Error` の安定化バージョンを検出する仕組みあり
- proc-macro である `thiserror-impl` はコンパイル時のみ動作し、ターゲットに依存しない

**設定例:**
```toml
[dependencies]
thiserror = { version = "2", default-features = false }
```

---

## 3. tower-lsp WASM 互換性

### 結論: ✅ **互換性あり（`runtime-agnostic` feature 使用）**

| 項目                       | 詳細                                      |
| -------------------------- | ----------------------------------------- |
| バージョン                 | tower-lsp 0.20.0                          |
| WASM サポート              | ✅ 公式に対応（CHANGELOG v0.16.0 で追加）  |
| `runtime-agnostic` feature | ✅ WASM に必須                             |
| 実績                       | tower-lsp-web-demo プロジェクトで実証済み |

**CHANGELOG v0.16.0 からの引用:**
> - Add compatibility with WASM (PR #309).
> - Support alternative async runtimes other than `tokio` when enabling the `runtime-agnostic` feature (PR #309).

**設定例:**
```toml
[dependencies.tower-lsp]
version = "0.20"
default-features = false
features = ["runtime-agnostic"]
```

**アーキテクチャのポイント:**
- `runtime-agnostic` feature は `tokio` 依存を削除し、`futures` の `AsyncRead`/`AsyncWrite` trait を使用
- transport 層で `async-codec-lite` の `FramedRead`/`FramedWrite` を使用
- `Server` は `tokio::spawn` を使わない独自の並行処理機構を内蔵（WASM 互換のため）
- ドキュメントに明記: *"exotic targets currently incompatible with `tokio`, such as WASM"* をサポート

**制約:**
- stdio は直接使えない → カスタムトランスポート（Web Streams等）が必要
- WebSocket 経由 または `wasm-bindgen` 経由のメッセージパッシングが必要
- `tokio::io::stdin()`/`stdout()` の代わりにカスタム `AsyncRead`/`AsyncWrite` を提供する

**参考プロジェクト: [tower-lsp-web-demo](https://github.com/silvanshade/tower-lsp-web-demo)**
- Rust (tower-lsp) + TypeScript (Monaco Editor) の完全な実装例
- プロジェクト構造:
  - `crates/browser/` — WASM エントリポイント
  - `crates/server/` — tower-lsp LanguageServer 実装
  - `packages/app/` — Monaco Editor + LSP クライアント
  - カスタム codec（demuxer, headers parser 等）でメッセージをブリッジ

---

## 4. lsp-server WASM 互換性

### 結論: ❌ **互換性なし（WASM 非対応）**

| 項目          | 詳細                                           |
| ------------- | ---------------------------------------------- |
| バージョン    | lsp-server 0.7.9                               |
| WASM サポート | ❌ 不可能                                       |
| 根本的問題    | `crossbeam-channel`、`std::thread`、stdio 依存 |

**非互換の原因:**

1. **`crossbeam-channel` 依存** — スレッドベースのチャネル実装で `wasm32-unknown-unknown` では動作しない
2. **`std::thread::spawn`** — I/O スレッドを生成（`IoThreads`）。WASM にはスレッド API がない
3. **stdio 直接使用** — `std::io::stdin()`/`std::io::stdout()` で LSP メッセージを送受信。WASM には標準入出力がない
4. **同期 API 設計** — `Connection` は同期的な `crossbeam::select!` マクロでメッセージを待機

**lsp-server のソースコード (socket.rs) より:**
```rust
fn make_write(mut stream: TcpStream) -> (...) {
    let writer = thread::spawn(move || { ... });
    ...
}
```

**代替策:** `lsp-server` の型定義（`Request`, `Response`, `Notification`, `Message`）自体は `serde` ベースで WASM 互換だが、`Connection` と `IoThreads` は完全に非互換。

---

## 5. Alternative approaches for WASM LSP

### 5.1 実世界のアプローチパターン

#### パターン A: tower-lsp + `runtime-agnostic` + `wasm-bindgen`（推奨）

**実績:** tower-lsp-web-demo

```
┌─────────────────────────────────────────────┐
│  VSCode Web Extension (TypeScript)          │
│  ├── Monaco Editor / VSCode API             │
│  ├── LSP Client (vscode-languageclient)     │
│  └── Web Streams ←→ WASM Bridge             │
│       ↕ (postMessage / ReadableStream)      │
│  ┌─────────────────────────────────────┐    │
│  │  WASM Module (Rust)                 │    │
│  │  ├── wasm-bindgen entrypoint        │    │
│  │  ├── tower-lsp (runtime-agnostic)   │    │
│  │  │   └── LanguageServer trait impl  │    │
│  │  └── 言語解析エンジン (pest等)       │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

**利点:**
- tower-lsp の `LanguageServer` trait で型安全な LSP 実装
- 公式 WASM サポートあり
- `Server::new(read, write, socket)` にカスタム AsyncRead/AsyncWrite を渡せる

**欠点:**
- tower-lsp は 2023年8月以降メンテナンスが停滞気味（v0.20.0 が最新、2年以上前）
- async runtime の選択に注意が必要

#### パターン B: プロトコル層と解析層の分離（最も柔軟）

```
┌────────────────────────────────────────┐
│  共通コア (WASM + Native)              │
│  ├── pasta_dsl (pest パーサー)         │
│  ├── 言語解析・診断エンジン             │
│  └── lsp-types (型定義のみ)            │
├────────────────────────────────────────┤
│  Native バイナリ用                     │
│  ├── lsp-server OR tower-lsp          │
│  └── stdio / TCP トランスポート        │
├────────────────────────────────────────┤
│  WASM 用                               │
│  ├── wasm-bindgen エントリポイント      │
│  ├── カスタム JSON-RPC ハンドラ         │
│  └── postMessage トランスポート         │
└────────────────────────────────────────┘
```

**利点:**
- 最大の柔軟性 — トランスポート層を完全に制御
- コア解析ロジックの再利用性が高い
- ネイティブ版は lsp-server (rust-analyzer 実績) を使い、WASM 版は軽量な自前実装
- テスト容易性 — コアロジックはプラットフォーム非依存

**欠点:**
- トランスポート層の自前実装コスト
- JSON-RPC のハンドリングを自分で書く必要がある（ただし単純）

#### パターン C: 完全自前実装

- `lsp-types` で型定義を使用
- `serde_json` で JSON-RPC メッセージをパース
- `wasm-bindgen` でトランスポートを実装
- async ランタイムなし or `wasm-bindgen-futures`

**利点:** 依存関係が最小限、完全制御  
**欠点:** LSP プロトコルの実装コストが高い

### 5.2 よく使われるクレート構成

| クレート               | 用途                                 | WASM 互換              |
| ---------------------- | ------------------------------------ | ---------------------- |
| `lsp-types`            | LSP 型定義                           | ✅                      |
| `tower-lsp`            | LSP サーバーフレームワーク           | ✅ (`runtime-agnostic`) |
| `lsp-server`           | LSP サーバースキャフォールド         | ❌                      |
| `pest` / `pest_derive` | PEG パーサー                         | ✅ (`no_std`)           |
| `thiserror`            | エラー定義                           | ✅ (`no_std`)           |
| `serde` / `serde_json` | シリアライゼーション                 | ✅                      |
| `wasm-bindgen`         | JS ⇔ WASM ブリッジ                   | ✅（WASM専用）          |
| `wasm-bindgen-futures` | JS Promise ⇔ Rust Future             | ✅（WASM専用）          |
| `js-sys` / `web-sys`   | Web API バインディング               | ✅（WASM専用）          |
| `ropey`                | テキストロープ（効率的テキスト操作） | ✅                      |

---

## 6. lsp-types WASM 互換性

### 結論: ✅ **互換性あり（問題なし）**

| 項目          | 詳細                              |
| ------------- | --------------------------------- |
| バージョン    | lsp-types 0.97.0                  |
| WASM サポート | ✅ 完全互換                        |
| 依存関係      | `serde`, `serde_json`, `url` のみ |

**根拠:**
- ほぼ全てが `struct` / `enum` の型定義 + `Serialize`/`Deserialize` derive
- OS 依存の機能なし
- `serde` と `serde_json` は `wasm32-unknown-unknown` で動作実績多数
- `url` クレート（URL パース）も WASM 互換
- tower-lsp-web-demo で lsp-types が WASM で使用されている実績あり
- rust-analyzer も内部で lsp-types を型定義として利用

---

## 推奨アーキテクチャ（pasta-language-server 向け）

### 推奨: パターン B — プロトコル層と解析層の分離

```
crates/
├── pasta_lsp_core/          # プラットフォーム非依存コア
│   ├── src/
│   │   ├── analysis.rs      # 言語解析エンジン
│   │   ├── diagnostics.rs   # 診断生成
│   │   ├── completion.rs    # 補完候補生成
│   │   ├── hover.rs         # ホバー情報
│   │   └── lib.rs
│   └── Cargo.toml           # deps: pasta_dsl, lsp-types, thiserror
│
├── pasta_lsp_server/        # ネイティブ LSP サーバー
│   ├── src/
│   │   ├── main.rs          # stdio/TCP エントリポイント
│   │   └── transport.rs     # lsp-server or tower-lsp ベース
│   └── Cargo.toml           # deps: pasta_lsp_core, tower-lsp
│
└── pasta_lsp_wasm/          # WASM LSP サーバー
    ├── src/
    │   └── lib.rs            # wasm-bindgen エントリポイント
    └── Cargo.toml            # deps: pasta_lsp_core, wasm-bindgen, tower-lsp (runtime-agnostic)
```

### 理由:

1. **pasta_dsl (pest) は WASM 互換** → コアパーサーをそのまま再利用
2. **thiserror は WASM 互換** → エラー型をそのまま再利用
3. **lsp-types は WASM 互換** → 型定義をそのまま再利用
4. **tower-lsp の `runtime-agnostic`** → WASM 対応済み、実績あり
5. **解析コアの分離** → テスト容易性、ネイティブ/WASM 両対応

### WASM ターゲットでの Cargo 設定例:

```toml
# pasta_lsp_core/Cargo.toml
[dependencies]
pasta_dsl = { path = "../pasta_dsl" }
lsp-types = "0.97"
thiserror = { version = "2", default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# pasta_lsp_wasm/Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
pasta_lsp_core = { path = "../pasta_lsp_core" }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"

[dependencies.tower-lsp]
version = "0.20"
default-features = false
features = ["runtime-agnostic"]
```

---

## リスクと注意事項

| リスク                       | 影響 | 緩和策                                         |
| ---------------------------- | ---- | ---------------------------------------------- |
| tower-lsp のメンテナンス停滞 | 中   | コアを分離し、tower-lsp への依存を薄く保つ     |
| WASM バイナリサイズ          | 中   | `wasm-opt` で最適化、不要機能の削除            |
| pest の WASM バイナリサイズ  | 低   | Unicode テーブルが大きい。必要なものだけ有効化 |
| 非同期ランタイムの選択       | 中   | WASM では `wasm-bindgen-futures` 一択          |
| デバッグ困難性               | 高   | WASM 版は `console_log` crate でログ出力       |

---

## 4. 部分パース戦略（議題3調査結果）

### 背景

Pasta DSLの文法は**行指向**であり、各スコープ（`file_scope`, `global_scene_scope`, `actor_scope`）は独立してパース可能。パースエラー時も成功した部分のセマンティックトークンを提供することで、編集中のUX（シンタックスハイライト維持）を向上できる。

### 文法の行指向性

grammarのトップレベル構造:
```pest
file = _{ SOI ~ ( file_scope | global_scene_scope | actor_scope )* ~ s ~ EOI }
```

各スコープの開始マーカー（行頭インデントなし）:

| スコープ | 開始マーカー | 例 |
|---|---|---|
| `file_scope` | `&`/`＆` (attr) or `@`/`＠` (word) | `&タイトル：テスト` |
| `global_scene_scope` | `*`/`＊` (global_marker) | `＊挨拶` |
| `actor_scope` | `%`/`％` (actor_marker) | `％Alice` |

**重要な性質**: すべての行は `eol` (`NEWLINE`) で終端する。つまり、各行は独立してパース試行可能。

### 部分パース実装戦略

#### Phase 1: 全体パース試行（現行動作）
```rust
match PastaParser2::parse(Rule::file, source) {
    Ok(pairs) => build_full_ast(pairs), // 成功 → 完全AST返却
    Err(_) => { /* Phase 2へ */ }
}
```

#### Phase 2: スコープ境界分割
行頭マーカーでソースをスコープチャンクに分割:
- `＊`/`*` → `global_scene_scope`開始
- `％`/`%` → `actor_scope`開始
- `＆`/`&` または `＠`/`@` → `file_scope`開始

各チャンクを独立してパース試行:
```rust
for chunk in split_by_scope_markers(source) {
    match PastaParser2::parse(chunk.rule, chunk.text) {
        Ok(pairs) => partial_items.extend(build_ast(pairs)),
        Err(e) => {
            errors.push(e);
            // Phase 3: 失敗したスコープ内を行単位パース
            partial_items.extend(parse_line_by_line(chunk));
        }
    }
}
```

#### Phase 3: 行単位フォールバック
スコープパースが失敗した場合、各行を個別Ruleで試行:
```rust
fn parse_line_by_line(chunk: &ScopeChunk) -> Vec<PartialItem> {
    let mut items = vec![];
    for line in chunk.text.lines() {
        // 各行の先頭パターンから適切なRuleを推論
        let rule = infer_rule_from_line_prefix(line);
        if let Ok(pairs) = PastaParser2::parse(rule, line) {
            items.push(build_partial_item(pairs));
        }
        // 失敗した行はスキップ（Diagnosticに報告）
    }
    items
}
```

#### 行からRuleを推論
```rust
fn infer_rule_from_line_prefix(line: &str) -> Rule {
    let trimmed = line.trim_start();
    match trimmed.chars().next() {
        Some('＊')|Some('*') => Rule::global_scene_line,
        Some('％')|Some('%') => Rule::actor_line,
        Some('＆')|Some('&') => Rule::file_attr_line,
        Some('@')|Some('＠') => Rule::file_word_line,
        _ if trimmed.starts_with("  ") => {
            // インデントあり → アクション行/変数設定行等
            if trimmed.contains('：') || trimmed.contains(':') {
                Rule::action_line
            } else {
                Rule::var_set_line
            }
        }
        _ => Rule::blank_line,
    }
}
```

### 技術的課題と対策

| 課題 | 対策 |
|---|---|
| **`pad`（インデント）の扱い** | スコープ内行は`pad`（インデント必須）を前提。行単位パース時もインデント付きで試行 |
| **コンテキスト依存性** | `global_scene_continue_line`（`＊`単体）は前シーン名に依存。部分パースでは無名シーン扱いで十分 |
| **`code_block`の複数行** | バッククォートフェンスで囲まれたコードブロックは複数行。スコープ分割時に正しく抽出 |
| **Span整合性** | 行単位パース時、byte offsetはソース全体からの相対位置に補正が必要 |

### pasta_dslへの影響

部分パース機能は**pasta_dslクレート側の新規API**として実装:

```rust
// pasta_dsl/src/parser/mod.rs
pub struct PartialParseResult {
    pub file: PastaFile,        // パース成功した部分のAST
    pub errors: Vec<ParseError>, // 各行/スコープのエラー
}

pub fn parse_str_partial(source: &str, filename: &str) -> PartialParseResult {
    // Phase 1→2→3の実装
}
```

### 要件への反映

- **R3-5**: pasta_dslに`parse_str_partial()`を追加
- **R2-6**: エラー時は成功行のトークンのみ提供し、失敗行はDiagnosticとして報告

### 期待効果

- ✅ 編集中もマーカー構造が視認可能（全トークン消失を回避）
- ✅ pest文法の単一管理維持（正規表現フォールバックを回避）
- ✅ 将来的にpest内error recoveryへの移行が容易

