# Requirements Document

## Project Description (Input)

pasta の利用者（ゴースト作者）向け情報（Pasta DSL 文法・公開 Lua API・`pasta.toml` 設定）が、`doc/spec/`・`book/src/`（mdBook）・`GRAMMAR.md`・スキル `references/` に重複して書かれ、権威の所在が混乱している。Lua API はスキル側の方が詳しく、`pasta.toml` リファレンスはスキル側にしか無く、`book/AUTHORING.md` はスキルを mdBook の「起草元」と位置付けており、権威の向きが逆転している。

本仕様は **mdBook マニュアルを利用者向け情報の唯一の権威**にする。`doc/spec/` と `GRAMMAR.md` を mdBook へ吸収して廃止し、スキル `references/` のうち利用者向け規範部分は mdBook から自動生成する（スキルは別リポジトリへコピーして使うため自己完結を保つ）。作例・パターン・コーディング規約・テスト/lint などの AI 作業手順はスキル手書きを権威として残す。権威の移行と生成への切替は 1 spec で一括完了し、過渡状態を出荷しない。

（詳細は `.kiro/specs/manual-ssot-authority/brief.md` を参照）

## Introduction

本仕様は、文書の「権威の所在」を再編するドキュメント基盤の仕様である。ランタイムの挙動・文法・API そのものは変更しない。成果は次の 3 点で観測できる。

1. 利用者向けの規範的記述は mdBook マニュアルにのみ手書きで存在する。
2. スキル `references/` の利用者向け規範ファイルは mdBook から生成され、その鮮度は CI で保証される。
3. 旧権威（`doc/spec/`・`GRAMMAR.md`・drift-check 機構）とそれらへの参照がリポジトリの現行文書から消えている。

想定読者（ロール）:

- **ゴースト作者**: 公開マニュアルを読んで辞書・スクリプトを書く利用者。
- **スキル利用者（AI エージェント／ゴースト開発者）**: スキルを別リポジトリへコピーしてゴースト制作に使う者。
- **メンテナ**: pasta リポジトリで文書とスキルを保守する開発者・AI エージェント。

## Boundary Context

- **In scope**:
  - `doc/spec/` の内容の mdBook への吸収と `doc/spec/` の廃止
  - `GRAMMAR.md` の廃止（マニュアルへの案内のみ残す）
  - mdBook Lua API 章の拡充（スキル `runtime-api`・`shiori-handlers` 相当の内容）
  - mdBook `pasta.toml` リファレンス章の新設と、既定値整合テストの参照先の付け替え
  - mdBook → スキル `references/` の生成と、CI による生成物鮮度チェック
  - スキル内の生成／手書き区分の明示（ファイル単位）と `SKILL.md` の更新
  - drift-check 機構（`drift-check`・`verify-drift-gate`・`manual-sources.toml`・関連 CI ステップ・完了ゲート記述）の撤去
  - `doc/spec/`・`GRAMMAR.md`・drift-check を参照する現行文書（steering・README・SOUL.md・OPTIMIZATION.md・スキル・`book/AUTHORING.md`・進行中 spec 等）の参照修正
- **Out of scope**:
  - ランタイム内部設計の解説、および `internal-modules` の権威移動（`pasta-runtime-internals-doc` の領域）
  - 文法・API・設定の挙動そのものの変更（記述の移し替えのみ）
  - `.kiro/specs/completed/` 内の歴史的記述の書き換え
  - マニュアルのデザイン・シンタックスハイライトの変更
  - 既に別リポジトリへ持ち出されたスキルのコピーの更新（持ち出し先での再コピーは各リポジトリの責任）
  - `pasta-check` スキル（brief が挙げる対象スキルは `pasta-ghost-authoring` と `pasta-lua-coding` のみ）
- **Adjacent expectations**:
  - 上流 `pasta-user-manual` / `pasta-manual-syntax-highlight` / `pasta-manual-debugging` が確立した mdBook 基盤（静的サイト・章の文体構造・公開パイプライン）はそのまま維持される。
  - 下流 `pasta-runtime-internals-doc` は、本仕様が確立する「mdBook 権威＋スキル生成」方式を再利用する。本仕様は `internal-modules` を手書きのまま残し、暫定である旨を明示する。
  - 進行中 spec `review-improvement-loop` は `doc/spec/`・`GRAMMAR.md`・drift-check を参照しているため、本仕様で参照のみ修正する（同 spec のプロセス内容は変更しない）。

> **未確定事項の扱い**: brief.md で解決できない論点は、各要件に「**仮定**」として明示し、末尾「Open Questions」に列挙した。要件ディスカッションで確定する。

## Requirements

### Requirement 1: 文法の権威を mdBook へ一本化する

**Objective:** As a ゴースト作者, I want Pasta DSL 文法の規範的な記述がマニュアルの 1 箇所にまとまっていること, so that どの文書が正しいか迷わずに辞書を書ける

#### Acceptance Criteria

1. The マニュアル shall `doc/spec/` の実装済み章（ch01〜07・ch09〜11）が定めていた規範的内容（構文定義・規則・制約・例外）を、文法パートの章に欠落なく収録する。
2. The マニュアル shall `GRAMMAR.md` またはスキルの手書きファイル（`authoring-patterns.md` 等）にのみ存在していた利用者向けの挙動の事実（例: チェイントーク、単語のシャッフル＆順次消費、時報変数、キューコマンド行・選択肢定義の詳細）を、マニュアル内のいずれかの章に収録する。
3. The マニュアル shall 現行実装が受理・処理する挙動のみを規範として収録し、未実装・将来仕様（`doc/spec/` ch08 の未実装セマンティクス、ch12 の未確定事項・将来仕様）を収録しない。属性のように「構文は受理されるが処理に反映されない」ものは、その現行挙動のみを記述する。
4. The リポジトリ shall `doc/spec/` ch08・ch12 にあった未実装・将来仕様の内容を黙って破棄せず、具体度に応じて行き先を分ける。brief を書ける程度に具体的な項目は個別 spec の brief（`.kiro/specs/{feature}/brief.md`）として起票し、ロードマップ（`.kiro/steering/roadmap.md`）には各項目のキー情報（名称・要旨・brief への参照）のみを記載する。brief に満たない未確定事項はロードマップにキー情報の粒度で記載する。（どの項目を brief 化するかの仕分けは設計で決定）
5. The マニュアル shall 文法の規範的記述について、マニュアル外の文書（`doc/spec/`・`GRAMMAR.md`）を「権威的仕様」として案内するリンクや注記を含まない。
6. When 吸収前後で文法記述を比較した場合, the マニュアル shall 現行実装の挙動と矛盾する記述を含まない（記述の移し替えのみで、文法の挙動は変更しない）。
7. The マニュアル shall 既存の章構成規約（キャラ口調の導入 → 規範的本文 → キャラ口調の締め、コードブロック・表・構文定義にキャラ口調を入れない）を、吸収・追加した内容でも維持する。

### Requirement 2: `doc/spec/` と `GRAMMAR.md` を廃止する

**Objective:** As a メンテナ, I want 旧権威の文書がリポジトリから無くなること, so that 重複管理と「どちらを直すか」の判断が発生しない

#### Acceptance Criteria

1. When 本仕様が完了した時点で, the リポジトリ shall `doc/spec/` ディレクトリを持たない（案内用スタブも残さない）。
2. When 本仕様が完了した時点で, the `GRAMMAR.md` shall 文法の説明本文を含まず、公開マニュアルの該当箇所への案内のみを含む。
3. If 利用者が旧 `GRAMMAR.md` を開いた場合, then the `GRAMMAR.md` shall 文法リファレンスがマニュアルへ移ったことと、その行き先（公開マニュアルの URL）を示す。
4. The リポジトリ shall `doc/spec/` および `GRAMMAR.md` の内容に依存するテスト・検証・ビルド手順を持たない。

### Requirement 3: 公開 Lua API の権威を mdBook へ移す

**Objective:** As a ゴースト作者, I want 公開 Lua API と SHIORI イベントハンドラの完全なリファレンスをマニュアルで読めること, so that スキルを持たない人間の読者でも同じ情報に到達できる

#### Acceptance Criteria

1. The マニュアル shall 現行スキル `runtime-api` が記載している公開ランタイムモジュール（`@pasta_search`・`@pasta_persistence`・`@pasta_config`・`@pasta_sakura_script`・`@enc`・`@pasta_log`・mlua-stdlib 統合モジュール）の API 記述を、情報量を減らさずに収録する。
2. The マニュアル shall 現行スキル `shiori-handlers` が記載している内容（REG 登録・RES 応答生成・主要 SHIORI イベント一覧・シーン関数フォールバック・仮想ディスパッチャ）を、情報量を減らさずに収録する。
3. When 現行のマニュアル Lua 章とスキル側の記述が食い違う場合, the マニュアル shall 現行実装の挙動に一致する記述を採用する。
4. The マニュアル shall ランタイム内部モジュール（`internal-modules` 相当）の解説を本仕様では収録対象としない。

### Requirement 4: `pasta.toml` リファレンスをマニュアルに新設する

**Objective:** As a ゴースト作者, I want `pasta.toml` の全セクション・全キーのリファレンスをマニュアルで読めること, so that 設定方法を公開マニュアルだけで把握できる

#### Acceptance Criteria

1. The マニュアル shall `pasta.toml` リファレンスの章を持ち、現行スキル `pasta-toml` が記載している内容（分類表・最小テンプレート・フルリファレンステンプレート・予約注記・各セクション詳細）を情報量を減らさずに収録する。
2. The マニュアル shall `pasta.toml` リファレンスの章を目次から到達可能にする。
3. The 既定値整合テスト shall マニュアルの `pasta.toml` リファレンスに記載された既定値と、実装の既定値が一致することを検証する。
4. If マニュアルに記載された `pasta.toml` の既定値が実装の既定値と食い違った場合, then the 既定値整合テスト shall 失敗し、食い違ったキーを報告する。
5. The 既定値整合テスト shall スキル配下のファイルを権威の読み先として参照しない。

### Requirement 5: スキル `references/` の利用者向け規範部分を mdBook から生成する

**Objective:** As a スキル利用者, I want スキルの文法・公開 Lua API・`pasta.toml` リファレンスがマニュアルと常に同じ内容であること, so that 別リポジトリへ持ち出したスキルだけで正しいゴーストを書ける

#### Acceptance Criteria

1. The スキル生成機構 shall 文法・公開 Lua API（`runtime-api`・`shiori-handlers` 相当）・`pasta.toml` リファレンスのスキル `references/` ファイルを、マニュアルの対応章から生成する。
2. The スキル生成機構 shall 生成物に規範的本文のみを出力し、章のキャラ口調の導入段落および章末の締め段落を含めない。
3. The 生成されたスキルファイル shall リポジトリ内パスやマニュアル章への相対リンクなど、スキルディレクトリの外を指す参照を含まない（スキルを別リポジトリへコピーしても参照が切れない）。
4. When マニュアル章が他の章を参照している場合, the スキル生成機構 shall 生成物内でその参照をスキル内で解決可能な形にするか、参照切れを残さない形で出力する。
5. When 同一のマニュアル内容から生成を繰り返した場合, the スキル生成機構 shall 実行環境（OS・作業コピーの改行コード）によらず同一内容の生成物を出力する。
6. The 生成されたスキルファイル shall 「マニュアルから生成されたものであり手で編集しない」旨と、編集すべき元がマニュアルであることをファイル内に明示する。
7. The スキル生成機構 shall メンテナがリポジトリ内で 1 回の操作により全生成物を再生成できる手段を提供する。
8. The スキル生成機構 shall 既存のマニュアルビルド用ツールチェーンの範囲で動作し、新たなエコシステム依存を要求しない。
9. If 生成対象として指定されたマニュアル章が存在しない、または期待する章構造（導入と本文の区切り）を満たさない場合, then the スキル生成機構 shall 生成を失敗させ、該当章を報告する。

### Requirement 6: スキル内の生成／手書き区分を明示する

**Objective:** As a メンテナ, I want スキルのどのファイルが生成物でどれが手書きかをファイル単位で判別できること, so that 誤って生成物を手で直したり、手書きの権威を上書きしたりしない

#### Acceptance Criteria

1. The スキル（`pasta-ghost-authoring`・`pasta-lua-coding`）shall `references/` の各ファイルについて、「マニュアルから生成」か「スキル手書き（権威）」かをファイル単位で `SKILL.md` に明示する。
2. The スキル shall 作例・パターン・コーディング規約・テスト/lint のファイルを手書き（スキルが権威）として保持する。ただし手書きファイルは文法・公開 Lua API・`pasta.toml` の挙動に関する規範的事実を独自に定義せず、作例・手順・規約のみを持つ（挙動の事実はマニュアルが権威で、生成ファイル経由でスキルに入る）。
3. The スキル shall `internal-modules` を手書きとして保持し、暫定の位置付けであること（将来 `pasta-runtime-internals-doc` で扱うこと）を明示する。
4. The `SKILL.md` shall 生成後の `references/` のファイル構成と一致するリンクのみを持ち、存在しないファイルを参照しない。
5. The スキル shall 手書きファイルを含むスキル全体として、スキルディレクトリ外（リポジトリ内パス・`doc/spec/`・`GRAMMAR.md`・`book/`）への参照を含まない。
6. While 生成ファイルと手書きファイルが同じ事実を扱う場合, the スキル shall 生成ファイルの記述を正とする旨を `SKILL.md` に明示する。

### Requirement 7: 生成物の鮮度を CI で保証する

**Objective:** As a メンテナ, I want マニュアルとスキル生成物の乖離が自動で検出されること, so that 「mdBook が権威なのにスキルが古い」状態がマージされない

#### Acceptance Criteria

1. When マニュアル章が変更され、対応するスキル生成物が再生成されていない変更が提出された場合, the 鮮度チェック shall 失敗する。
2. When スキルの生成ファイルが手で編集され、マニュアルからの生成結果と一致しない変更が提出された場合, the 鮮度チェック shall 失敗する。
3. If 鮮度チェックが失敗した場合, then the 鮮度チェック shall 不一致のファイルと、解消手順（再生成の方法）を報告する。
4. The 鮮度チェック shall 改行コードの差（CRLF / LF）のみを不一致として扱わない。
5. The 鮮度チェック shall マニュアル側のみ・スキル側のみ・生成機構のみのいずれの変更に対しても CI で実行される。
6. While 鮮度チェックが失敗している間, the マニュアル公開パイプライン shall マニュアルを公開しない。
7. The 鮮度チェック shall メンテナがローカルでも同じ判定を再現できる手段を提供する。
8. The spec 完了ワークフロー（完了ゲート）shall マニュアルまたはスキル生成対象に触れる変更について、鮮度チェックの成功を完了条件に含める。

### Requirement 8: drift-check 機構を撤去する

**Objective:** As a メンテナ, I want `doc/spec/` 追従のためのドリフト検出機構が残っていないこと, so that 存在しない権威ソースを前提とした検査や手順に惑わされない

#### Acceptance Criteria

1. When 本仕様が完了した時点で, the リポジトリ shall `doc/spec/` とマニュアル章のハッシュ対応表、およびそれを用いるドリフト検出・ドリフトゲート検証を持たない。
2. The マニュアル公開パイプライン shall ドリフト検出に依存せず、マニュアルのビルド・検証・公開を完了する。
3. The マニュアル検証 shall 文法章に対して「`doc/spec/` への権威リンクを持つこと」「ハッシュ対応表に登録されていること」を要求しない。
4. The マニュアル検証 shall drift-check が担っていたマニュアル内リンク切れの検出を、撤去後も引き続き行う。
5. The spec 完了ワークフロー（完了ゲート）shall ドリフト検出を実行条件・完了条件として記述しない。

### Requirement 9: 現行文書の参照を修正する

**Objective:** As a メンテナ, I want 現行文書が廃止済みの `doc/spec/`・`GRAMMAR.md`・drift-check を権威として案内しないこと, so that 文書の指示に従っても存在しない場所へ誘導されない

#### Acceptance Criteria

1. When 本仕様が完了した時点で, the リポジトリの現行文書（steering・`README.md`・`SOUL.md`・`OPTIMIZATION.md`・スキル・`book/` 配下の執筆規約と章・進行中 spec・ソースコード内コメント）shall `doc/spec/` を現存する文書として参照しない。
2. When 本仕様が完了した時点で, the リポジトリの現行文書 shall `GRAMMAR.md` を文法の参照先・同期対象として案内しない。
3. The 現行文書 shall 利用者向け情報（文法・公開 Lua API・`pasta.toml`）の権威がマニュアルであること、仕様衝突時にマニュアルを優先することを一貫して記述する。
4. The `book/AUTHORING.md` shall スキルを章の「起草元／流用元」とする記述を持たず、マニュアルが権威でスキルの規範部分はマニュアルから生成されることを記述する。
5. The ドキュメント保守手順（steering の保守ルール・更新チェックリスト）shall 文法・API・設定の変更時に更新すべき対象として、マニュアルとスキル再生成を示す。
6. The リポジトリ shall `.kiro/specs/completed/` 配下の歴史的記述を本仕様で書き換えない。
7. The マニュアル shall 廃止された `doc/spec/` を指すリンク（外部リンク集・導入章を含む）を含まない。

### Requirement 10: 一括完了と既存品質の維持

**Objective:** As a メンテナ, I want 権威の移行と生成への切替が同時に完了すること, so that 「mdBook が権威なのにスキルが古い手書き」という過渡状態が出荷されない

#### Acceptance Criteria

1. The 本仕様の成果 shall Requirement 1〜9 のすべてを満たした状態でのみ既定ブランチへ統合される（一部のみを先行して出荷しない）。
2. When 本仕様が完了した時点で, the リポジトリ shall 利用者向け規範内容について、マニュアル以外に手書きの写し（`doc/spec/`・`GRAMMAR.md` 本文・スキルの手書き規範リファレンス）を持たない。
3. The リポジトリ shall 本仕様の変更後も既存のテストスイート全体が成功する。
4. The マニュアル公開パイプライン shall 本仕様の変更後もマニュアルを静的サイトとしてビルド・公開できる。
5. The pasta ランタイム shall 本仕様の変更によって、文法・公開 Lua API・`pasta.toml` の解釈に関する外部から観測可能な挙動を変えない。
6. If 吸収元（`doc/spec/`・`GRAMMAR.md`・スキル手書きリファレンス）にあった規範的内容がマニュアルに収録できないと判明した場合, then the 本仕様 shall 当該内容を黙って破棄せず、扱い（収録・明示的な除外理由の記録）を決定してから完了する。

## Open Questions

要件ディスカッションで確定すべき論点。各項目は上記要件では「仮定」として暫定的に扱っている。

4. **`SKILL.md` の文法要約の扱い**（R6）: `SKILL.md` 自体が文法の要約（マーカー表等）を手書きで持つ。これは許容する重複か、縮小・生成対象にするか。
7. **`steering/grammar.md` の位置付け**（R9）: 「AI 向け完全参照（doc/spec 準拠）」として文法を要約しており、事実上 5 箇所目の写しである。参照先の付け替えのみとするか、案内のみに縮小するか。
9. **生成対象の範囲**（R5.1）: `pasta-lua-coding/SKILL.md` は既に `book/src/reference/startup.md` をリポジトリ内パスで参照しており自己完結に反している。起動シーケンス章（およびデバッグ章など他の利用者向け章）を生成対象に含めるか、参照を削るだけにするか。

### 設計フェーズへ先送りした事項

以下は how の判断であり、`research.md` §6 の申し送りに従って設計で決定する。

- スキル `references/` のファイル構成（章と 1:1 か現行ファイル名維持か）
- 章中のキャラ口調コラムの扱いと、生成時に落とす範囲の執筆規約化
- リンク切れ検出の存続方法
- 鮮度チェックの CI 配置
