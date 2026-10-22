# pasta_check

ゴースト（「伺か」デスクトップマスコット）の配布パッケージ（`.nar`）と
SSP ネットワーク更新ファイル（`updates.txt`）を作成し、
SSP を起動せずにゴーストの辞書（`.pasta`）と `pasta.toml` を検査する CLI ツールです。

ゴースト名やパスに依存しない汎用ツールであり、任意のゴーストに対して使えます。

## インストール

GitHub の Release から `pasta_check.zip` を落として、好きなフォルダに展開します。最新版は次の固定のリンクで落とせます。

<https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip>

中身は `pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` です。`pasta_check.exe` は 64 ビット版の Windows 向けで、署名が無いため、初めて起動するときに Windows の SmartScreen の警告が出ることがあります。展開した `pasta_check.exe` はパスで指定して呼べます。

```powershell
C:\tools\pasta_check\pasta_check.exe validate --target ghosts\hello-pasta
```

Rust を使っている場合は、`cargo install` でも入れられます。

```bash
cargo install pasta_check
```

## 使い方

### 基本構文

```
pasta_check <command> [options]
pasta_check --help
pasta_check --version
```

### validate サブコマンド

ゴースト開発フォルダーの辞書と `pasta.toml` を、SSP を起動せずに検査します。ファイルは読むだけで、作成・変更・削除はしません。

```
pasta_check validate --target <path>
```

| オプション | 必須 | 説明 |
|-----------|------|------|
| `--target <path>` | ✅ | ゴースト開発フォルダー（`ghost/master` を含むルート。`ghost/master` そのものは不可） |

- 見つけた問題を 1 件 1 行（`パス:行:列: 誤り|注意: 理由`）で標準出力に出し、最後に辞書ファイルの数・誤りと注意の件数をまとめた行を出します
- 終了コードは、誤りが 1 件以上あれば 1、無ければ 0（注意だけなら 0）。使い方の誤りと `--target` のフォルダの誤り（無い・フォルダでない・`ghost/master` を含まない）も 1 で、`Error: ` で始まる理由を標準エラー出力に出します
- 検査の項目・各報告の意味と直し方・読むだけでは決まらない範囲は、マニュアルの [辞書を検査する（pasta_check validate）](https://ekicyou.github.io/pasta/debug/validate.html) を参照してください

```powershell
pasta_check validate --target path/to/ghost
```

### release サブコマンド

ゴースト開発フォルダーからリリースパッケージを作成します。辞書の検査（validate と同じ規則）を関門にしています。

```
pasta_check release --target <path> --release <path> --nar <path> [--copy <path>]...
```

| オプション | 必須 | 説明 |
|-----------|------|------|
| `--target <path>` | ✅ | ゴースト開発フォルダー（`ghost/master` 等を含むルート） |
| `--release <path>` | ✅ | リリース出力先フォルダー（毎回クリーンされる） |
| `--nar <path>` | ✅ | 出力 NAR ファイルパス（親ディレクトリは自動作成） |
| `--copy <path>` | | 上書きコピー元フォルダー（複数指定可、後勝ち） |

#### 実行フロー（6ステップ）

```
[1/6] Preparing release folder   ← --release を削除して新規作成、--nar の位置の前回の nar を削除
[2/6] Copying target files       ← --target → --release に再帰コピー
[3/6] Applying overlay copies    ← --copy（指定があれば）上書きコピー
[4/6] Validating dictionaries    ← --copy 上書き後の --release/ghost/master の辞書を validate と同じ規則で検査（関門）
[5/6] Generating update files    ← 同梱バルーンを判定し、ゴースト用・バルーン用の updates.txt を自動生成
[6/6] Creating NAR archive       ← --release を ZIP 圧縮して --nar に出力
```

#### 辞書の検査の関門（段 4）

- 段 4 は validate と同じ形で問題とまとめの行を出します
- 誤りが 1 件でもあると `Error: dictionary validation found N error(s); update files and NAR were not created` を出し、`updates.txt` と NAR を作らずに終了コード 1 で終わります。段 1 で前回の NAR を消しているので、`--nar` の位置に NAR は残りません
- 配布フォルダに `ghost/master` が無いときも段 4 でエラーになります
- 注意だけのときは注意を出したうえで段 5 に進みます
- 検査を省くオプションはありません

#### 使用例

```powershell
# 最小構成
pasta_check release `
  --target path/to/ghost `
  --release release/my-ghost `
  --nar release/my-ghost.nar

# オーバーレイ付き（ビルド成果物を上書き）
pasta_check release `
  --target path/to/ghost `
  --release release/my-ghost `
  --nar release/my-ghost.nar `
  --copy path/to/build-output `
  --copy path/to/extra-files
```

## 仕様メモ

### updates.txt（SSP ネットワーク更新・Version 3 形式）

- 1 行目: `charset,UTF-8`
- 以降: `file,<path>\x01<md5>\x01size=<bytes>\x01date=<YYYY-MM-DDTHH:MM:SS>\x01` （CRLF 区切り）
- エントリはパスの辞書順でソートされる（決定的出力）
- `ghost/master/` が存在する場合は同内容の `updates.txt` をそこにもコピーする
- 一覧から除外: ディレクトリ `profile/`・`var/`、ファイル `updates2.dau`・`updates.txt`・`developer_options.txt`、同梱バルーンのフォルダ配下

### 同梱バルーン

- 段 5 は `--copy` 上書き後の配布フォルダ直下の `install.txt` を読む。`install.txt` が無ければ同梱バルーンは無いものとする
- `install.txt` があるときは、同梱バルーンの有無にかかわらず UTF-8 で、1 行目（BOM があればその直後）が `charset,UTF-8` でなければならない（文字コードの変換はしない）
- バルーン指定（`balloon[N].source.directory`・`balloon[N].directory`。番号なし → `balloon0` → `balloon1` … の順）が指す配布フォルダ内のフォルダを同梱バルーンとする。置き場所は任意で、階層付きの値は `source.directory` にだけ書ける
- 同梱バルーンのフォルダ配下はゴースト用 `updates.txt` から除き、フォルダ直下にバルーン用 `updates.txt`（パスはバルーンのフォルダが基準・`ghost/master/` への複製なし・除外規則はゴースト用と同じ）を生成して NAR に封入する
- 同梱バルーンのフォルダ直下には `descript.txt` が必要で、`install.txt` と同じ UTF-8 の規則を適用する。`homeurl` が無いか値が空なら警告を出す（NAR は作成する）
- バルーン指定の不正（キーの重複・欠番の後ろの番号、値が空・絶対パス・`..`・`profile`/`var` 等）、フォルダが無い、`ghost/master` や別の同梱バルーンとの入れ子、UTF-8 の規則違反はエラーで止まり、NAR を作らない

### 前回の NAR の削除

- 段 1 で配布フォルダを準備した後に `--nar` の位置の前回の NAR を削除する（`--nar` がフォルダなど削除できないときは段 1 でエラー）
- 段 6 で失敗したときは作りかけの NAR も削除する。削除した後の段でエラーになったときは `--nar` の位置に NAR が残らない（古い NAR を最新と誤って配らないため）

### NAR（ZIP）封入規則

- `profile/` ディレクトリは封入しない（ユーザーデータ・フレームワーク自己展開先の保護）
- シンボリックリンクはコピー・封入ともにスキップする
- ZIP エントリ名の `..` コンポーネントは常時実行時検査で拒否する（パストラバーサル防御）

## ソース構成

クレートはライブラリ（検査の核）とバイナリ（CLI）の 2 つの顔を持ちます。検査の核はファイルシステムに触れず、ファイルの一覧と中身を受け取って報告を返すので、バイナリと VSCode 拡張（`pasta_lsp` の WASM 経由）の両方から同じ規則で使えます。

```
pasta_check/
├── Cargo.toml
├── src/
│   ├── lib.rs           # ライブラリの入口（validate モジュールを公開）
│   ├── validate/        # 検査の核（ファイルシステムに触れない・WASM でも動く）
│   │   ├── mod.rs              # 入口（辞書ファイルの選び出し・検査）
│   │   ├── input.rs            # 入力の形（JS からも同じ形で渡せる）
│   │   ├── config.rs           # pasta.toml の [loader]・[scene] の検査
│   │   ├── discovery.rs        # pasta.dll と同じ辞書ファイル・辞書モジュールの選び出し
│   │   ├── syntax.rs           # 構文の検査・読めないファイル
│   │   ├── transpile_rules.rs  # Lua への変換で失敗する書き方
│   │   ├── module_name.rs      # モジュール名の重なり
│   │   ├── scenes.rs           # シーンの登録・シーン名の重なり
│   │   ├── builtins.rs         # 組み込みの飛び先の名前
│   │   ├── references.rs       # Call・選択肢の飛び先の検査
│   │   └── report.rs           # 報告の種類・分類・文言・並び順・出力
│   ├── main.rs          # CLI 引数解析（lexopt）・ディスパッチ
│   ├── validate_cmd.rs  # validate サブコマンド（ghost/master のファイルを集めて核を呼ぶ・release の段 4 も使う）
│   ├── release.rs       # release サブコマンド（6ステップパイプライン）
│   ├── copy.rs          # 再帰コピー・release フォルダー初期化
│   ├── balloon.rs       # 同梱バルーン判定（install.txt・descript.txt の読み取り）
│   ├── update_files.rs  # updates.txt 生成（MD5・ISO 8601 日時）
│   └── nar.rs           # NAR（ZIP）アーカイブ作成
└── tests/
    ├── cli_test.rs               # バイナリ経由の CLI 統合テスト（終了コード・入出力契約）
    ├── validate_cli_test.rs      # validate のバイナリ経由のテスト
    ├── validate_config_test.rs   # pasta.toml の検査
    ├── validate_rules_test.rs    # 構文・変換・モジュール名の検査
    ├── validate_scenes_test.rs   # シーン名・飛び先の検査
    ├── validate_report_test.rs   # 報告の文言・並び順・出力の形
    └── validate_parity_test.rs   # pasta.dll（pasta_lua）との一致テスト
```

## 依存クレート

| クレート | 用途 |
|---------|------|
| `pasta_dsl` | 辞書の構文解析（pasta.dll と同じパーサー） |
| `pasta_core` | シーンの登録（pasta.dll と同じレジストリ） |
| `toml` | `pasta.toml` の読み取り |
| `serde` | 検査の入力・報告の形（WASM 経由の JS とのやりとり） |
| `glob` | `pasta_patterns` の照合 |
| `lexopt` | CLI 引数パーサー（feature `cli`） |
| `md5` | updates.txt の MD5 ハッシュ（SSP 仕様準拠のファイル変更検出用途・非暗号学的。feature `cli`） |
| `zip` | NAR（ZIP）アーカイブ作成（feature `cli`） |

- 既定の feature `cli` はバイナリだけが使う依存（`lexopt`・`md5`・`zip`）を有効にします。`pasta_lsp` は `default-features = false` で依存し、ライブラリ（検査の核）だけを WASM にビルドします
- 開発時の依存 `pasta_lua` は一致テスト（検査の結果が pasta.dll の読み込みと一致すること）に使います（版を書かないので `cargo publish` では外れます）

## ライセンス

MIT
