# Requirements Document

## Introduction

本仕様は、`pasta_lua`クレートに**Pasta DSL → Lua トランスパイル結果のキャッシュファイル生成機能**を実装することを目的とする。

現在、トランスパイルは実行時に都度行われているが、本機能により以下を実現する：
- ファイルタイムスタンプに基づく増分トランスパイル（変更検出）
- キャッシュされたLuaファイルによる起動時間短縮
- `scene_dic.lua`によるシーンモジュールの一括ローディング

**スコープ境界**：`finalize_scene()`関数の実体実装は本仕様のスコープ外とする。本仕様では呼び出しの仕組みのみを整備する。

## Requirements

### Requirement 1: ファイル変更検出

**Objective:** ゴースト開発者として、変更されたPastaファイルのみをトランスパイルしたい。これにより、起動時間を短縮し、開発効率を向上させたい。

#### Acceptance Criteria

1. When `dic/**/*.pasta` ファイルが検出された時, the CacheTranspiler shall 対応するキャッシュLuaファイル (`{transpiled_output_dir}/pasta/scene/**/*.lua`) の存在を確認する
2. When Pastaファイルのタイムスタンプがキャッシュファイルより新しい場合, the CacheTranspiler shall そのファイルをトランスパイル対象としてマークする
3. When キャッシュファイルが存在しない場合, the CacheTranspiler shall そのPastaファイルをトランスパイル対象としてマークする
4. When Pastaファイルのタイムスタンプがキャッシュファイルより古いまたは同じ場合, the CacheTranspiler shall そのファイルのトランスパイルをスキップする
5. The CacheTranspiler shall ファイルタイムスタンプ比較においてミリ秒精度で判定を行う

### Requirement 2: キャッシュファイル出力

**Objective:** システム管理者として、トランスパイル結果を決められたディレクトリ構造で永続化したい。これにより、再起動時に再トランスパイルを回避できる。

#### Acceptance Criteria

1. When トランスパイル対象のPastaファイルが処理される時, the CacheTranspiler shall `{transpiled_output_dir}/pasta/scene/` 配下にLuaファイルを出力する
2. When `dic/scene1.pasta` がトランスパイルされる時, the CacheTranspiler shall `{transpiled_output_dir}/pasta/scene/scene1.lua` として出力する
3. When `dic/subdir/scene2.pasta` がトランスパイルされる時, the CacheTranspiler shall `{transpiled_output_dir}/pasta/scene/subdir/scene2.lua` として出力する
4. When 出力先ディレクトリが存在しない場合, the CacheTranspiler shall 必要なディレクトリ階層を自動作成する
5. The CacheTranspiler shall 出力ファイルのエンコーディングをUTF-8とする

**Note**: `{transpiled_output_dir}` はデフォルト値 `profile/pasta/cache/lua` を想定。実際の出力先は `profile/pasta/cache/lua/pasta/scene/` となる。

### Requirement 3: シーン辞書ファイル生成

**Objective:** ランタイムシステムとして、すべてのシーンモジュールを一括でロードしたい。これにより、シーンテーブルの初期化を一元化できる。

#### Acceptance Criteria

1. When キャッシュトランスパイル処理が完了した時, the CacheTranspiler shall `{transpiled_output_dir}/pasta/scene_dic.lua` を常に再生成する
2. The scene_dic.lua shall 全てのキャッシュ済みLuaモジュールに対する `require` 文を含む
3. When `dic/scene1.pasta` がキャッシュに存在する場合, the scene_dic.lua shall `require "pasta.scene.scene1"` を含む
4. When `dic/subdir/scene2.pasta` がキャッシュに存在する場合, the scene_dic.lua shall `require "pasta.scene.subdir.scene2"` を含む
5. The scene_dic.lua shall 末尾で `require("pasta").finalize_scene()` を呼び出す
6. When 新規Pastaファイルが追加された場合, the scene_dic.lua shall 次回生成時にその require 文を含める
7. When Pastaファイルが削除された場合, the scene_dic.lua shall 次回生成時にそのモジュールの require 文を含めない

### Requirement 4: モジュール命名規則

**Objective:** Luaモジュールシステムとして、一貫したモジュール名でシーンにアクセスしたい。これにより、予測可能なモジュール構造を維持できる。

#### Acceptance Criteria

1. The CacheTranspiler shall Pastaファイルパスを `pasta.scene.<relative_path>` 形式のモジュール名に変換する
2. When ファイルパスにサブディレクトリが含まれる場合, the CacheTranspiler shall パス区切りを `.` に変換する
3. When ファイル名に日本語が含まれる場合, the CacheTranspiler shall そのまま日本語をモジュール名に使用する
4. The CacheTranspiler shall `.pasta` 拡張子を除去してモジュール名を生成する
5. When ファイル名にハイフンが含まれる場合, the CacheTranspiler shall アンダースコアに変換する
6. The CacheTranspiler shall ディレクトリ階層を再現した物理ファイル配置を行う（`dic/subdir/scene.pasta` → `{transpiled_output_dir}/pasta/scene/subdir/scene.lua`）

**Design Decision**: ディレクトリ階層を再現する方式を採用。Luaの標準的なモジュール解決（`require "pasta.scene.subdir.scene"` → `pasta/scene/subdir/scene.lua`）に準拠する。

### Requirement 5: ローダー統合

**Objective:** ゴースト実行環境として、トランスパイル後にシーン辞書を自動的にロードしたい。これにより、手動のロード処理が不要になる。

#### Acceptance Criteria

1. When トランスパイル処理が完了した時, the PastaLoader shall `pasta.scene_dic` モジュールを require する
2. If scene_dic.lua のロード中にエラーが発生した場合, the PastaLoader shall エラー詳細とファイルパスを含むエラーメッセージを報告する
3. The PastaLoader shall scene_dic.lua のロードを、他のユーザーモジュールのロードより先に実行する
4. While debug_mode が有効な場合, the PastaLoader shall トランスパイル対象ファイル数とスキップファイル数をログ出力する

**Design Decision**: Rust側（PastaLuaRuntime::from_loader）で明示的に `require "pasta.scene_dic"` を実行する方式を採用。これにより、シーン構築の失敗を早期に検出し、確実なエラーハンドリングを実現する。

### Requirement 6: エラーハンドリング

**Objective:** ゴースト開発者として、トランスパイルエラーの原因を特定したい。これにより、DSL記述ミスを素早く修正できる。

#### Acceptance Criteria

1. If Pastaファイルのパースに失敗した場合, the CacheTranspiler shall ファイルパス、行番号、エラー内容を含むエラーを報告する
2. If トランスパイル処理に失敗した場合, the CacheTranspiler shall 該当ファイルのキャッシュを更新せず、既存キャッシュを保持する
3. If ファイル書き込みに失敗した場合, the CacheTranspiler shall 書き込み対象パスと OS エラーを含むエラーを報告する
4. When 一部のファイルでエラーが発生した場合, the CacheTranspiler shall 他のファイルの処理を継続する
5. The CacheTranspiler shall 処理完了時にエラーが発生したファイルの一覧をサマリーとして報告する

### Requirement 7: パス解決とディレクトリ構成

**Objective:** システム管理者として、pasta.toml の設定に基づいてパスを解決したい。これにより、カスタマイズ可能なディレクトリ構成をサポートできる。

#### Acceptance Criteria

1. The CacheTranspiler shall `pasta.toml` の `[loader].pasta_patterns` からソースパターンを読み取る
2. The CacheTranspiler shall `pasta.toml` の `[loader].transpiled_output_dir` からキャッシュ出力先を読み取る
3. When `transpiled_output_dir` が設定されていない場合, the CacheTranspiler shall デフォルト値 `profile/pasta/cache/lua` を使用する
4. The CacheTranspiler shall ベースディレクトリからの相対パスを絶対パスに解決する
5. If pasta.toml が存在しない場合, the CacheTranspiler shall デフォルト設定で動作する
