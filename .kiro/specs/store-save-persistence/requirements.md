# Requirements Document

## Introduction
STORE.saveテーブルの永続化機能を実装する。ランタイム起動時に保存データをロードし、ランタイム終了時（Drop時）に自動保存する。オプションで難読化をサポートし、`pasta.toml`で設定可能とする。

## Project Description (Input)
store.luaのSTORE.saveテーブルについて、ランタイムロード時に永続化ファイルの読み込み、ランタイムドロップ時に永続化ファイルへの書き込みを行うようにrust側でサポートする。

１．rustからluaへの公開関数群に、永続化ファイルのロード関数を用意。
２．STORE.save = XXXとし、XXXのところでロード関数を呼ぶ。
３．drop時にSTORE.saveを保存する処理を実装。
４．可能なら永続化ファイルの簡単な難読化可能なシリアライズクレートを導入
５．難読化するかどうかのフラグをコンフィグファイルのフラグとして追加
６．その他、永続化に必要な要件を検討して実装

## Requirements

### Requirement 1: Rust側永続化API
**Objective:** ランタイム開発者として、Rust側からLuaへ永続化関連の関数を公開したい。これにより、Lua側で明示的にファイルI/Oを実装せずにデータを永続化できる。

#### Acceptance Criteria
1. When ランタイムが初期化されるとき, the PastaLuaRuntime shall Lua側に`@pasta_persistence`モジュールを登録する
2. When `@pasta_persistence.load()`が呼び出されるとき, the persistence module shall 永続化ファイルからデータをLuaテーブルとして返す
3. When `@pasta_persistence.save(data)`が呼び出されるとき, the persistence module shall 渡されたLuaテーブルを永続化ファイルに書き込む
4. If 永続化ファイルが存在しないとき, then the persistence module shall 空テーブル`{}`を返す
5. If ファイル読み込み中にエラーが発生したとき, then the persistence module shall エラーログを出力し空テーブルを返す（起動を妨げない）

### Requirement 2: Lua側STORE.save初期化
**Objective:** スクリプト開発者として、STORE.saveが自動的に永続化データで初期化されてほしい。これにより、手動でロード処理を書く必要がなくなる。

#### Acceptance Criteria
1. When store.luaがロードされるとき, the STORE.save field shall `@pasta_persistence.load()`の結果で初期化される
2. The store.lua module shall `@pasta_persistence`モジュールをrequireする
3. While ランタイムが動作中, the STORE.save shall 通常のLuaテーブルとして読み書き可能である

### Requirement 3: ランタイムDrop時の自動保存
**Objective:** システム運用者として、ランタイム終了時に自動的にSTORE.saveが保存されてほしい。これにより、明示的な保存呼び出しなしでデータが永続化される。

#### Acceptance Criteria
1. When PastaLuaRuntimeがドロップされるとき, the runtime shall `STORE.save`テーブルを永続化ファイルに書き込む
2. When Drop実装が実行されるとき, the runtime shall Lua側の`require("pasta.store").save`を取得して保存する
3. If 保存中にエラーが発生したとき, then the runtime shall エラーログを出力する（パニックしない）
4. While ランタイムが正常に動作しているとき, the runtime shall Lua VMへの参照を保持し続ける

### Requirement 4: 難読化シリアライズ対応
**Objective:** コンテンツ開発者として、保存データを簡易的に難読化したい。これにより、カジュアルな改ざんを抑止できる。

#### Acceptance Criteria
1. Where 難読化が有効な場合, the persistence module shall データをXOR暗号化したバイナリ形式で保存する（拡張子: `.dat`）
2. Where 難読化が無効な場合, the persistence module shall データを人間可読なJSON形式で保存する（拡張子: `.json`）
3. The persistence module shall 難読化形式でもJSON形式でも読み込み可能とする（後方互換性）
4. The serialization shall 任意にネストしたテーブル、文字列、数値、ブール値、配列をサポートする

### Requirement 5: 設定ファイル対応
**Objective:** ゴースト開発者として、pasta.tomlで永続化の動作を設定したい。これにより、プロジェクトごとに異なる設定が可能となる。

#### Acceptance Criteria
1. The PastaConfig shall `[persistence]`セクションを解析する
2. When `obfuscate = true`が設定されているとき, the persistence module shall 難読化形式で保存する（デフォルト: `profile/pasta/save/save.dat`）
3. When `obfuscate = false`または未設定のとき, the persistence module shall JSON形式で保存する（デフォルト: `profile/pasta/save/save.json`）
4. The PastaConfig shall `file_path`オプションで保存先パスを変更可能とする
5. If `[persistence]`セクションが存在しないとき, then the persistence module shall デフォルト設定（難読化なし、`profile/pasta/save/save.json`）を使用する

### Requirement 6: エラーハンドリングと堅牢性
**Objective:** システム運用者として、永続化処理が失敗してもランタイム全体が停止しないでほしい。これにより、ファイルシステムの問題があっても対話が継続できる。

#### Acceptance Criteria
1. If 永続化ファイルが破損しているとき, then the persistence module shall 警告ログを出力し空テーブルで初期化する
2. If 保存先ディレクトリが存在しないとき, then the persistence module shall ディレクトリを再帰的に作成する
3. If ディスク容量不足で保存に失敗したとき, then the persistence module shall エラーログを出力し処理を継続する
4. The persistence module shall 一時ファイル書き込み後にリネームするアトミック保存を実装する
5. If リネームに失敗したとき, then the persistence module shall 元ファイルを保持し一時ファイルを削除する

### Requirement 7: テストとデバッグ支援
**Objective:** 開発者として、永続化機能を簡単にテスト・デバッグしたい。これにより、開発サイクルが効率化される。

#### Acceptance Criteria
1. The persistence module shall ユニットテストで一時ディレクトリを使用可能とする
2. When debug_mode = trueのとき, the persistence module shall 保存・読み込み時にデバッグログを出力する
3. The persistence module shall Luaテーブル↔Rust間の変換エラーを詳細に報告する
