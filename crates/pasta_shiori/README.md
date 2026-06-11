# pasta_shiori

「伺か」SHIORI DLL インターフェースを提供するクレートです。

## 概要

`pasta_shiori` は pasta_lua を使用して SHIORI/3.0 プロトコルを実装し、
従来の伺かユーザー向けのDLLインターフェースを提供します。

Windows DLL（`pasta.dll`）として出力され、伺かベースウェアから呼び出されます。

## アーキテクチャ

```
pasta_shiori
├── SHIORI Protocol    # SHIORI/3.0 プロトコル実装
│   ├── load           # 初期化
│   ├── request        # イベント処理
│   └── unload         # 終了
├── Lua Integration    # pasta_lua ランタイムとの統合
│   ├── PastaLoader    # スクリプトロード（pasta_lua 提供）
│   └── lua_request    # SHIORI リクエスト→Lua テーブル変換
└── Windows DLL        # C FFI エクスポート（windows.rs）
    └── load / unload / request  # SHIORI DLL エントリポイント（+ DllMain）
```

## ディレクトリ構成

```
pasta_shiori/
├── Cargo.toml
└── src/
    ├── lib.rs           # クレートエントリーポイント
    ├── error.rs         # エラー型定義
    ├── shiori.rs        # PastaShiori - SHIORI プロトコル実装
    ├── shiori_tests.rs  # SHIORI テストモジュール（#[path]属性による例外的配置）
    ├── lua_request.rs   # Lua リクエスト処理
    ├── windows.rs       # Windows DLL エクスポート（#[cfg(windows)]）
    └── util/            # ユーティリティ
        ├── hglobal/     # HGLOBAL 文字列・ANSI/UTF-8 エンコーディング（#[cfg(windows)]）
        └── parsers/     # SHIORI リクエストパーサー（pest 文法）
```

## SHIORI プロトコル

### プロトコルフロー

```
ベースウェア → shiori32.dll
                    ↓
               load(hinst, load_dir)     # 初期化
                    ↓
               request(SHIORI/3.0)       # イベント処理（繰り返し）
                    ↓
               unload()                  # 終了
```

### サポートイベント

| イベント  | 説明                                          |
| --------- | --------------------------------------------- |
| `load`    | 初期化。`pasta.toml` 読み込み、ランタイム起動 |
| `request` | SHIORI/3.0 リクエスト処理                     |
| `unload`  | 終了処理。リソース解放                        |

### SHIORI/3.0 リクエスト形式

```
GET SHIORI/3.0
Charset: UTF-8
Sender: SSP
SecurityLevel: local
ID: OnFirstBoot
Reference0: 1

```

### レスポンス形式

```
SHIORI/3.0 200 OK
Charset: UTF-8
Value: \0\s[0]初めまして！\e

```

### FFI 境界の安全性

`windows.rs` の DLL エントリポイントは、SHIORI ホスト（SSP 等）を巻き込む
クラッシュを防ぐため、次の安全性保証を実装しています。

- **パニック封じ込め**: `load` / `request` / `unload` の各ディスパッチは
  `catch_unwind` で全包囲され、パニックは SHIORI エラー契約へ縮退します
  （`load`→`false`、`request`→`500` + `X-ERROR-REASON`、`unload`→`true`）。
  FFI 境界を越えるパニックは未定義動作であるためです。
- **入力 HGLOBAL の解放保証**: 入力 HGLOBAL の所有権は SHIORI 契約により
  DLL 側へ移転します。長さ 0・未初期化などのガードパスを含む全経路で解放されます。
- **アロケーション失敗の明示エラー化**: `GlobalAlloc` 失敗は null ポインタの
  まま流通させず、明示的なエラーとして処理されます。
- **反復パース**: SHIORI リクエストヘッダのパースは反復実装であり、
  ホスト制御のヘッダ数によるスタック枯渇は発生しません。

## 公開API

### PastaShiori

| メソッド                | 説明                                       |
| ----------------------- | ------------------------------------------ |
| `load(hinst, load_dir)` | ランタイム初期化                           |
| `request(request)`      | SHIORI リクエスト処理                      |
| `runtime()`             | ロード済み Lua ランタイム参照（テスト用） |
| `Default::default()`    | 新規インスタンス作成                       |

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
use pasta_shiori::{PastaShiori, Shiori};

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

```
ghost/
└── master/                  # load_dir（SHIORIのload_dir）
    ├── pasta.toml           # 設定ファイル（必須）
    ├── dic/                 # Pasta DSL ソース
    │   └── *.pasta
    ├── scripts/             # Lua スクリプト
    │   └── pasta/
    │       └── shiori/
    │           └── entry.lua # SHIORI エントリーポイント
    └── profile/             # ランタイム生成
        └── pasta/
            ├── save/        # 永続化データ
            ├── cache/       # キャッシュ
            └── logs/        # ログ
```

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

### Windows 専用

| クレート    | バージョン | 用途                              |
| ----------- | ---------- | --------------------------------- |
| windows-sys | 0.61       | Windows API（メモリ、文字コード） |

### 開発用（dev-dependencies）

| クレート | バージョン | 用途                     |
| -------- | ---------- | ------------------------ |
| tempfile | 3          | テスト用一時ディレクトリ |

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

- [pasta_dsl](../pasta_dsl/README.md) - DSLパーサー
- [pasta_core](../pasta_core/README.md) - レジストリ
- [pasta_lua](../pasta_lua/README.md) - Luaバックエンド
- [プロジェクト概要](../../README.md) - pasta プロジェクト全体

## ライセンス

プロジェクトルートの [LICENSE](../../LICENSE) ファイルを参照してください。
