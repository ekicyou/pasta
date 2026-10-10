# pasta_shiori

「伺か」SHIORI DLL インターフェースを提供するクレートです。

## 概要

`pasta_shiori` は pasta_lua を使用して SHIORI/3.0 プロトコルを実装し、
従来の伺かユーザー向けのDLLインターフェースを提供します。

Windows DLL（`pasta.dll`）として出力され、伺かベースウェアから呼び出されます。

## アーキテクチャ

ベースウェアは pasta.dll のエクスポート関数 `loadu`（設置パスを UTF-8 で受け取る）または `load`（ANSI で受け取る）・`request`・`unload` を呼びます。FFI 入口（`windows.rs`）は Lua VM を直接触らず、アクターランタイム（`actor/`）が起こした専用スレッドへ mailbox 経由でメッセージを送ります。そのスレッド上の `PastaShiori`（`shiori.rs`）が `pasta_lua` の `PastaLoader` で VM を構築し、リクエスト文字列を Lua の表に変換して（`lua_request.rs`）、Lua 側の `SHIORI.request` を呼びます。

詳細は [内部設計: SHIORI 層](https://ekicyou.github.io/pasta/internals/shiori.html#構成要素) を参照してください。ファイルごとの役割は [Rust 側（pasta_shiori）](https://ekicyou.github.io/pasta/internals/shiori.html#rust-側pasta_shiori) にあります。

## SHIORI プロトコル

`loadu`・`load` でアクタースレッドを起こしてランタイムを構築し、`request` は GET を同期で、NOTIFY を即時の 204 応答で処理し、`unload` でアクターを終了します。HGLOBAL の所有と解放、panic の封じ込め、応答を返せないときの 204 などの FFI 境界の約束も内部設計の章にまとめています。

- loadu・load・request・unload の流れ: [内部設計: 処理とデータの流れ](https://ekicyou.github.io/pasta/internals/shiori.html#処理とデータの流れ)
- FFI 境界の約束: [内部設計: 不変条件と制約](https://ekicyou.github.io/pasta/internals/shiori.html#不変条件と制約)
- SHIORI イベントの一覧: [主要イベント](https://ekicyou.github.io/pasta/lua/shiori-events.html#主要イベント)
- Lua に渡るリクエストの表と応答の組み立て: [act.req](https://ekicyou.github.io/pasta/lua/shiori-events.html#actreq)・[RES](https://ekicyou.github.io/pasta/lua/shiori-events.html#res)

## 公開API

ライブラリ名は DLL 名に合わせて `pasta` です。Rust からは `pasta::` で参照します（`use pasta::{PastaShiori, Shiori};`）。

### PastaShiori

| メソッド                | 説明                                                          |
| ----------------------- | ------------------------------------------------------------- |
| `load(hinst, load_dir)` | ランタイム初期化（`Shiori` トレイト）                         |
| `request(request)`      | SHIORI リクエスト処理（`Shiori` トレイト）                    |
| `runtime()`             | ロード済み Lua ランタイム参照（テスト用）                     |
| `kick(scene)`           | Lua 側の `SHIORI.kick` を保護呼び出しする（デバッグ通信用）  |
| `Default::default()`    | 新規インスタンス作成                                          |

このほか、アクターランタイムの `actor`・エラー型の `error`・リクエスト解析の `lua_request` の各モジュールと、Windows ではエクスポート関数 `load`・`loadu`・`request`・`unload` を公開しています（統合テスト用）。

### Shiori トレイト

```rust
pub trait Shiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool>;
    fn request<S: AsRef<str>>(&mut self, request: S) -> MyResult<String>;
}
```

`MyResult` は `error` モジュールの `MyError` を用いた Result 型エイリアスです。

## 使用例

### Rust からの利用（テスト用）

```rust
use pasta::{PastaShiori, Shiori};

let mut shiori = PastaShiori::default();

// 初期化
let success = shiori.load(0, "path/to/ghost/master").unwrap();
assert!(success);

// リクエスト送信
let request = "GET SHIORI/3.0\r\nID: OnBoot\r\n\r\n";
let response = shiori.request(request).unwrap();
println!("Response: {}", response);
```

### ゴーストディレクトリ構成

`load_dir` にはゴーストの `ghost/master/` を渡します。必須なのは設定ファイル `pasta.toml` で、辞書は `dic/`、ゴースト作者の Lua スクリプトは `scripts/` に置きます。SHIORI のエントリ（`pasta.shiori.entry`）は pasta.dll に同梱されており、ゴースト側に置く必要はありません。実行時には `profile/pasta/` に同梱スクリプトの展開先・キャッシュ・保存データ・ログが作られます。

フォルダ構成は [ゴーストの最小一式を置く](https://ekicyou.github.io/pasta/getting-started/setup.html#ゴーストのフォルダ構成)、起動の流れとモジュール検索パスは [起動シーケンスとモジュール解決](https://ekicyou.github.io/pasta/reference/startup.html) を参照してください。

## 依存関係

バージョンはすべてワークスペース（ルート `Cargo.toml`）で一元管理されています。

| クレート    | バージョン | 用途                                              |
| ----------- | ---------- | ------------------------------------------------- |
| pasta_lua   | workspace  | Luaランタイム（pasta_dsl経由でDSLパーサーに依存） |
| time        | 0.3        | タイムスタンプ処理                                |
| tracing     | 0.1        | ロギング                                          |
| thiserror   | 2          | エラー型定義                                      |
| pest        | 2.8        | SHIORI リクエストパース                           |
| pest_derive | 2.8        | pest パーサー導出マクロ                           |
| flume       | 0.12       | アクタースレッドの mailbox（チャネル）            |
| wintf-winmsg-executor | 0.0.3 | アクタースレッドのメッセージループ           |
| arc-swap    | 1.9        | mailbox の送信端の差し替え                        |

### Windows 専用

| クレート    | バージョン | 用途                              |
| ----------- | ---------- | --------------------------------- |
| windows-sys | 0.61       | Windows API（メモリ、文字コード、ハンドル数の計測） |

### ビルド用（build-dependencies）

| クレート       | バージョン | 用途                                       |
| -------------- | ---------- | ------------------------------------------ |
| embed-resource | 3          | pasta.dll へのバージョン情報の埋め込み     |

### 開発用（dev-dependencies）

| クレート | バージョン | 用途                     |
| -------- | ---------- | ------------------------ |
| tempfile | 3          | テスト用一時ディレクトリ |
| ctor     | 0.2        | テスト前の環境変数の中和 |
| serde_json | 1        | デバッグ通信のテスト     |
| tracing-test | 0.2    | ログ出力のテスト         |

## ビルド

### Windows DLL

```bash
cargo build --release -p pasta_shiori
# 出力: target/release/pasta.dll
```

### ライブラリ（テスト用）

```bash
cargo build -p pasta_shiori
cargo test -p pasta_shiori
```

## 外部仕様参照

- [SHIORI/3.0 仕様](http://usada.sakura.vg/contents/specification.html) - SHIORI プロトコル仕様
- [伺か](http://usada.sakura.vg/) - デスクトップマスコット基盤

## 関連クレート

- [`pasta_dsl`](https://crates.io/crates/pasta_dsl) - DSLパーサー
- [`pasta_core`](https://crates.io/crates/pasta_core) - レジストリ
- [`pasta_lua`](https://crates.io/crates/pasta_lua) - Luaバックエンド
- [プロジェクト概要](https://github.com/ekicyou/pasta) - pasta プロジェクト全体
- [pasta マニュアル](https://ekicyou.github.io/pasta/) - ゴースト作者向けの使い方とコントリビュータ向けの内部設計

## ライセンス

プロジェクトルートの [LICENSE](https://github.com/ekicyou/pasta/blob/main/LICENSE) ファイルを参照してください。
