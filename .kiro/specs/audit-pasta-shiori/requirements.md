# Requirements Document

## Introduction

pasta_shioriはSHIORI DLLインターフェース層であり、プロジェクト内でunsafeコードが最も集中する箇所（13ブロック）を含む。Windows HGLOBALメモリ管理、extern "C" FFI境界（DllMain + 3関数）、SHIORIリクエストパーサーを含む約1,500行のコードベースに対し、外部振る舞いを不変に保ちながら以下を実施する：

1. 全unsafeブロックの安全性検証とSAFETYコメントによるドキュメント化
2. FFI境界の入力検証強化
3. プロダクションコード内のパニック除去（Result伝搬への変換）
4. デッドコード除去（`#[allow(dead_code)]` 18箇所の精査）
5. 冗長表現の削減

## Boundary Context
- **In scope**: pasta_shiori/src/ 配下の全ソースファイルに対する脆弱性調査、unsafe安全性検証、FFI境界強化、パーサー堅牢化、デッドコード除去、冗長表現削減
- **Out of scope**: SHIORIプロトコル仕様の変更、Windows API（windows-sys）クレートの内部、新しいSHIORI機能の追加、pasta_luaランタイムの変更
- **Adjacent expectations**: pasta_coreおよびpasta_luaの公開APIは変更しない。既存テスト（pasta_shiori/tests/、pasta_shiori/src/shiori_tests.rs、hglobal内テスト、req_parser内テスト）は全パスを維持する

## Requirements

### Requirement 1: unsafeブロックの安全性検証とドキュメント化

**Objective:** As a メンテナー, I want 全unsafeブロックに安全性の根拠を示すSAFETYコメントが付与されていること, so that コードレビュー時にunsafe使用の妥当性を即座に判断できる

#### Acceptance Criteria
1. The pasta_shiori shall SAFETYコメントを全unsafeブロック（`unsafe impl Send/Sync`、`unsafe`ブロック、`#[unsafe(no_mangle)]`）に付与する
2. When unsafeブロックが安全性の前提条件を満たさない場合, the pasta_shiori shall 前提条件を満たすためのガード処理を追加するか、安全なAPIで置き換える
3. The pasta_shiori shall unsafeブロックの数を必要最小限に維持する（ゼロにはできないが、不要なunsafeを除去する）

### Requirement 2: FFI境界の入力検証強化

**Objective:** As a SHIORIホスト（SSP等）, I want FFI関数（`load`、`unload`、`request`）がNULLポインタやゼロ長入力に対して安全に動作すること, so that 不正な入力がメモリ安全性違反を引き起こさない

#### Acceptance Criteria
1. When `load`関数にNULLポインタまたはゼロ長のHGLOBALが渡された場合, the pasta_shiori shall パニックせずにfalseを返す
2. When `request`関数にNULLポインタが渡された場合, the pasta_shiori shall パニックせずにNULLポインタとゼロ長を返す
3. When HGLOBALのデータが不正なUTF-8またはANSI文字列の場合, the pasta_shiori shall エラーログを出力し、適切なエラー応答を返す
4. The pasta_shiori shall 全FFI関数のドキュメントコメントに安全性の前提条件（Safetyセクション）を明記する

### Requirement 3: プロダクションコードのパニック除去

**Objective:** As a ゴースト作者, I want SHIORIリクエスト処理中にパニックが発生しないこと, so that 不正なリクエストを受けてもDLLがクラッシュせずにエラー応答を返す

#### Acceptance Criteria
1. When SHIORIリクエストのパースでkey_valueペアが不正な場合, the pasta_shiori shall パニックではなくResult::Errを返す
2. When SHIORI2バージョン番号のパースに失敗した場合, the pasta_shiori shall パニックではなくResult::Errを返す
3. The pasta_shiori shall lua_request.rs内の全`unwrap()`呼び出しをResult伝搬（`?`演算子）またはデフォルト値に置き換える
4. The pasta_shiori shall req.rs内の全`unwrap()`呼び出しおよび`panic!()`呼び出しをResult伝搬に置き換える
5. While テストコード内にある場合, the pasta_shiori shall `unwrap()`および`unwrap_or_else(|e| panic!())`の使用を許容する

### Requirement 4: デッドコード除去

**Objective:** As a メンテナー, I want 使用されていないコードが除去されていること, so that コードベースの理解・保守コストが削減される

#### Acceptance Criteria
1. When `#[allow(dead_code)]`が付与されたアイテムが外部から実際に使用されていない場合, the pasta_shiori shall そのアイテムを除去する
2. When `#[allow(dead_code)]`が付与されたアイテムがテストまたはFFI境界から使用されている場合, the pasta_shiori shall `#[allow(dead_code)]`をより具体的な属性（`#[cfg(test)]`等）に置き換えるか、SAFETYコメントで使用理由を明記する
3. When ファイル全体が空（res.rs）の場合, the pasta_shiori shall そのファイルとmod宣言を除去する
4. The pasta_shiori shall MyError::Others バリアントの使用状況を調査し、未使用であれば除去する

### Requirement 5: 冗長表現の削減

**Objective:** As a メンテナー, I want コードの冗長な表現が簡潔に書き直されていること, so that 可読性と保守性が向上する

#### Acceptance Criteria
1. The pasta_shiori shall ShioriRequest構造体（req.rs）とlua_request.rs内のparse_key_value関数の重複パースロジックを識別し、統合可能な部分を統合する
2. The pasta_shiori shall error.rs内のFrom実装で冗長なパターンを簡潔化する
3. While 外部振る舞い（SHIORI API応答内容・エラーフォーマット）が変わらない限り, the pasta_shiori shall リファクタリングを適用する

### Requirement 6: 既存テストの全パス維持

**Objective:** As a 開発者, I want 監査による変更後も全既存テストがパスすること, so that 外部振る舞いが不変であることが検証される

#### Acceptance Criteria
1. The pasta_shiori shall `cargo test -p pasta_shiori`の全テストをパスする
2. The pasta_shiori shall `cargo test`（ワークスペース全体）でリグレッションが発生しない
3. The pasta_shiori shall `cargo clippy -p pasta_shiori`で新しい警告が発生しない
4. While パフォーマンスに影響する変更を行った場合, the pasta_shiori shall 変更前と同等以上の性能を維持する
