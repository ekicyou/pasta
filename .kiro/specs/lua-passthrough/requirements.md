# Requirements Document

## Project Description (Input)
*.pastaを置くディレクトリ（辞書ディレクトリ）に、*.luaがあったら、トランスパイルせずにキャッシュディレクトリにコピーし、そのままscene_dic.luaから読み込むようにする。luaコードを記述するための簡便な方法。

## Introduction

辞書ディレクトリ（`dic/*/`）は現在 `.pasta` ファイルのみを対象としており、Luaコードを直接記述するにはLuaブロック（` ```lua ` ）を`.pasta`ファイル内に埋め込む必要がある。本機能は辞書ディレクトリに直接`.lua`ファイルを配置し、トランスパイルなしでキャッシュにコピーして`scene_dic.lua`から読み込む仕組みを提供する。これにより、Pasta DSLでは表現しにくい高度なLuaロジックを、`.pasta`ファイルのボイラープレートなしに直接記述できる簡便な手段を実現する。

## Requirements

### Requirement 1: Luaファイルの検出

**Objective:** ゴースト開発者として、辞書ディレクトリに `.lua` ファイルを置くだけで自動的に認識されるようにしたい。手動設定なしにワークフローを効率化するため。

#### Acceptance Criteria
1. When `.lua` ファイルが辞書ディレクトリ（`dic/*/`）に配置された場合, the Loader shall `.pasta` ファイルと同様にそのファイルを検出する
2. The Loader shall 既存の `pasta_patterns` 設定に加えて、辞書ディレクトリ内の `.lua` ファイルを検出する
3. The Loader shall `profile/` ディレクトリ内の `.lua` ファイルを検出対象から除外する
4. When 同名の `.pasta` ファイルと `.lua` ファイルが同じディレクトリに存在する場合, the Loader shall `.pasta` ファイルを優先し、`.lua` ファイルを無視する
5. If 同名衝突により `.lua` ファイルが無視された場合, the Loader shall 警告をログに出力する

### Requirement 2: Luaファイルのパススルー処理

**Objective:** ゴースト開発者として、辞書ディレクトリの `.lua` ファイルがトランスパイルなしに直接キャッシュディレクトリにコピーされ、`scene_dic.lua` から自動的に読み込まれるようにしたい。Pasta DSLのボイラープレートなしにLuaコードを直接記述できるようにするため。

#### Acceptance Criteria
1. The Loader shall `.lua` ファイルを `pasta_core::parse_str()` および `LuaTranspiler::transpile()` に渡さない
2. When 辞書ディレクトリ内の `.lua` ファイルが検出された場合, the Loader shall そのファイルの内容をキャッシュディレクトリへコピーする
3. The CacheManager shall `.lua` ファイルに対して `.pasta` ファイルと同じディレクトリ構造（`cache/pasta/scene/`配下）とモジュール命名規則（`pasta.scene.<path>`）を適用する
4. When `.lua` ファイルのソースがキャッシュより新しい場合, the CacheManager shall ファイルを再コピーする（インクリメンタル更新）
5. While キャッシュが最新の場合, the CacheManager shall `.lua` ファイルのコピーをスキップする
6. When `scene_dic.lua` が生成される場合, the CacheManager shall `.lua` 由来のモジュールを `.pasta` 由来のモジュールと同列にrequireエントリとして含める
7. If `.lua` ファイルのコピーに失敗した場合, the Loader shall 警告をログに出力し処理を継続する（致命的エラーとしない）

> **設計判断**: 統計情報のカテゴリ分け（`copied` を新設するか `skipped` に含めるか）は設計フェーズで決定する。

### Requirement 3: 孤立キャッシュの管理

**Objective:** ゴースト開発者として、削除された `.lua` ファイルの孤立キャッシュが正しく検出されるようにしたい。キャッシュの一貫性を維持するため。

#### Acceptance Criteria
1. When ソースの `.lua` ファイルが削除された場合, the CacheManager shall 対応するキャッシュファイルを孤立キャッシュとして検出する
2. The CacheManager shall `.lua` 由来のキャッシュファイルを `.pasta` 由来のキャッシュファイルと同じ孤立検出ロジックで処理する
