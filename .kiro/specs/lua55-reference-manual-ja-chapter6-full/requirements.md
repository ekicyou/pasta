# Requirements Document

## Introduction

Lua 5.5リファレンスマニュアル日本語版（`lua55-reference-manual-ja`仕様、完了済み）において、第6章「標準ライブラリ」が要約版として作成されていた。本仕様では、第6章を**完全版**として再作成し、原文の全関数・全パラメータ・全説明を網羅した日本語翻訳を提供する。

**問題点**: 現在の`06-standard-libraries.md`は約550行の要約版であり、原文HTML（約3,900行）の内容を大幅に省略している。各関数の詳細説明、パラメータの仕様、戻り値の動作、使用例が欠落している。

**目標**: 原文の11サブセクション（§6.1〜§6.11）すべてを完全に翻訳し、要約なしの完全版として再作成する。

## Project Description (Input)
lua55-reference-manual-jaについて追加の依頼。6章の文章が要約されてしまっているが、この章は要約せずに解説してもらう必要がある。6章を完全版で作り直してください。

## Requirements

### Requirement 1: 原文からの完全翻訳（徹底翻訳義務）

**Objective:** As a Lua開発者, I want 第6章のすべての関数・説明を完全な形で日本語で読めること, so that 原文を参照せずに標準ライブラリの仕様を理解できる

#### Acceptance Criteria
1. The 翻訳 shall 原文HTML（約3,900行）のすべての内容を含むこと
2. The 翻訳 shall 各関数の完全な説明（目的、パラメータ、戻り値、動作、例外）を含むこと
3. The 翻訳 shall 原文のコード例をすべて保持すること
4. When 関数説明にパラメータ表がある場合, the 翻訳 shall それを完全に翻訳すること
5. The 翻訳 shall 「詳細は原文参照」「他の関数も同様」のような省略表現を一切使用しないこと
6. **The 翻訳プロセス shall 途中で停止しないこと（トークン制限に達した場合は中断箇所を記録し、次のタスクで継続すること）**
7. **The 翻訳プロセス shall 各API関数を完全に翻訳してから次の関数に進むこと**
8. **The 翻訳単位 shall 1モジュール（§6.1-§6.11各セクション）または1API関数単位とすること**
9. **The 翻訳プロセス shall 複数関数を「同様」「他も同じ」としてまとめて省略しないこと**
10. **When 関数が多いモジュール（§6.2, §6.5, §6.8, §6.9等）, the タスク shall 関数グループ（5-10関数）単位で分割すること**

### Requirement 2: 11サブセクションの網羅性

**Objective:** As a Lua開発者, I want Lua 5.5標準ライブラリの全11サブセクションを完全に参照できること, so that どのライブラリ関数も漏れなく確認できる

#### Acceptance Criteria
1. The ドキュメント shall 以下の全11サブセクションを完全に含むこと：
   - §6.1 – Cコードでのライブラリのロード（`luaL_openlibs`, `luaL_openselectedlibs`）
   - §6.2 – 基本関数（`assert`, `collectgarbage`, `dofile`, `error`, `_G`, `getmetatable`, `ipairs`, `load`, `loadfile`, `next`, `pairs`, `pcall`, `print`, `rawequal`, `rawget`, `rawlen`, `rawset`, `select`, `setmetatable`, `tonumber`, `tostring`, `type`, `_VERSION`, `warn`, `xpcall`）
   - §6.3 – コルーチン操作（`coroutine.*`全関数）
   - §6.4 – モジュール（`require`, `package.*`全関数・変数）
   - §6.5 – 文字列操作（`string.*`全関数、パターンマッチング詳細）
   - §6.6 – UTF-8サポート（`utf8.*`全関数）
   - §6.7 – テーブル操作（`table.*`全関数）
   - §6.8 – 数学関数（`math.*`全関数・定数）
   - §6.9 – 入出力機能（`io.*`全関数、ファイルハンドルメソッド全て）
   - §6.10 – オペレーティングシステム機能（`os.*`全関数）
   - §6.11 – デバッグライブラリ（`debug.*`全関数）
2. The ドキュメント shall 各サブセクションの冒頭説明文も完全に翻訳すること
3. The ドキュメント shall 原文にある注意事項・警告文をすべて含むこと

### Requirement 3: 関数リファレンスの詳細度

**Objective:** As a Lua開発者, I want 各関数の詳細な仕様を日本語で確認できること, so that 実装時に正確な動作を把握できる

#### Acceptance Criteria
1. The 各関数説明 shall 以下の情報を含むこと（原文に存在する場合）：
   - 関数シグネチャ（`function_name (param1, param2, ...)`形式）
   - 全パラメータの説明と型
   - 戻り値の説明
   - 動作の詳細説明（複数段落にわたる場合はすべて）
   - オプショナルパラメータの動作
   - エラー時の動作
2. When 関数にオプションがある場合（例: `collectgarbage`）, the 翻訳 shall 各オプションの完全な説明を含むこと
3. The 翻訳 shall 原文の箇条書きリスト構造を保持すること
4. When 関数説明に内部リンクがある場合（例: 「§2.5を参照」）, the 翻訳 shall 対応する日本語版へのリンクを設定すること

### Requirement 4: パターンマッチングの完全解説

**Objective:** As a Lua開発者, I want 文字列パターンマッチングの完全な仕様を日本語で参照できること, so that 正規表現代替パターンを正確に使用できる

#### Acceptance Criteria
1. The §6.5 shall パターン構文の完全な説明を含むこと：
   - 文字クラス（`%a`, `%d`, `%s`, `%w`など）の全リスト
   - マジック文字（`( ) . % + - * ? [ ^ $`）の完全説明
   - 繰り返し演算子（`*`, `+`, `-`, `?`）の動作詳細
   - キャプチャ（`()`）の使用方法
   - アンカー（`^`, `$`）の説明
   - 文字セット（`[set]`, `[^set]`）の説明
2. The §6.5 shall `string.format`の書式指定子を完全に解説すること
3. The §6.5 shall `string.pack`/`string.unpack`の書式文字列を完全に解説すること

### Requirement 5: Lua 5.5固有の変更点の明示

**Objective:** As a Lua開発者, I want Lua 5.5で追加・変更された機能を明確に識別できること, so that バージョン差異を把握できる

#### Acceptance Criteria
1. The ドキュメント shall Lua 5.5で新規追加されたセクション（§6.1）を明示すること
2. The ドキュメント shall `luaL_openselectedlibs`が5.5新規関数であることを明記すること
3. The ドキュメント shall `collectgarbage`の`"param"`オプションが5.5新規であることを明記すること
4. The ドキュメント shall セクション番号の変更（5.4→5.5）を参照表で示すこと
5. Where 関数動作がLua 5.4から変更されている場合, the ドキュメント shall その変更点を注記すること

### Requirement 6: ドキュメント構造とフォーマット

**Objective:** As a ドキュメントメンテナ, I want 一貫したフォーマットで第6章が記述されていること, so that 他の章との整合性が保たれる

#### Acceptance Criteria
1. The ドキュメント shall 他の章（01-09）と同様のMarkdownフォーマットを使用すること
2. The ドキュメント shall 見出しレベルを統一すること（章: H1、セクション: H2、関数: H3）
3. The ドキュメント shall 関数シグネチャをコードブロック（`\`\`\`lua`）で表示すること
4. The ドキュメント shall 章間リンク・索引リンクを適切に設定すること
5. The ドキュメント shall ファイルヘッダー（Source, Translation, Glossary参照）を含むこと
6. The ドキュメント shall GLOSSARY.mdの用語を一貫して使用すること

### Requirement 7: 翻訳品質とソース整合性

**Objective:** As a 翻訳品質管理者, I want 翻訳が原文と完全に対応していること, so that 情報の欠落・誤訳がない

#### Acceptance Criteria
1. The 翻訳プロセス shall 仕様フォルダ内の英語原文HTML（`chapters/en/06-standard-libraries.html`および`chapters/en/standard-libraries/*.html`）を参照すること
2. The 翻訳 shall API名・関数名・予約語・定数名は原文のまま維持すること
3. The 翻訳 shall コード例は原文のまま維持すること（コメントは翻訳可）
4. The 翻訳 shall 既存のGLOSSARY.mdの用語対応表に従うこと
5. When 新しい用語が出現した場合, the 翻訳プロセス shall GLOSSARY.mdを更新すること

### Requirement 8: ファイル構成

**Objective:** As a 開発者, I want 第6章が適切なファイルサイズで管理されていること, so that 参照・編集が容易である

#### Acceptance Criteria
1. The ドキュメントシステム shall 第6章を単一ファイル（`06-standard-libraries.md`）として提供するか、または論理的にサブファイルに分割すること
2. If 単一ファイルが大きすぎる場合（目安: 5,000行超）, the ドキュメントシステム shall サブセクションごとに分割ファイルを検討すること
3. The ドキュメントシステム shall 分割する場合、親ファイル（`06-standard-libraries.md`）に目次とリンクを含むこと
4. The ドキュメント shall `crates/pasta_lua/doc/lua55-manual/`に配置すること

### Requirement 9: 実装進捗追跡と完了保証（分割戦略）

**Objective:** As a プロジェクト管理者, I want 翻訳が確実に完了することを追跡できること, so that 途中放棄を防止できる

#### Acceptance Criteria

**タスク分割戦略**:
1. **The 実装タスク shall 以下の分割単位で定義すること**：
   - 小規模モジュール（§6.1, §6.6, §6.7, §6.10, §6.11）: 1モジュール = 1タスク
   - 中規模モジュール（§6.3, §6.4）: 1モジュール = 1タスク
   - 大規模モジュール（§6.2, §6.5, §6.8, §6.9）: **関数グループ単位で分割**
2. **The 大規模モジュールの分割基準 shall 以下とすること**：
   - §6.2 基本関数: 5タスク（各5関数程度）
   - §6.5 文字列操作: 4タスク（関数群 + パターン + format + pack）
   - §6.8 数学関数: 3タスク（各10関数程度）
   - §6.9 入出力: 3タスク（io関数 + fileメソッド + 補足）

**完了管理**:
3. **The 各タスク shall 完了時に以下を記録すること**：
   - 翻訳完了関数リスト
   - 出力Markdown行数
   - GLOSSARY追加用語（あれば）
4. **When タスクが未完了のまま終了した場合, the 実装者 shall 中断箇所（関数名）と残作業を明記すること**
5. **The 全タスク完了後 shall 最終マージと品質チェックを行うこと**
6. **The 完了判定 shall 全API関数の翻訳と統合が完了した時点とすること**

**要約禁止ルール**:
7. **The 翻訳プロセス shall 以下の表現を使用しないこと**：
   - 「他の関数も同様に...」
   - 「残りの関数は省略」
   - 「詳細は原文参照」
   - 「以下同様のパターン」
   - 表形式での簡略説明（各関数に完全な説明を付けること）

## Non-Requirements (スコープ外)

以下は本仕様のスコープ外とする：

- 第6章以外の章の修正・再翻訳
- 新しい索引ファイルの作成
- 原文HTMLの取得・更新（既存のchaptersフォルダを使用）
- Lua 5.4との比較表の完全版作成

## 翻訳リソース

### 英語原文（翻訳ソース）
- `chapters/en/06-standard-libraries.html`（約3,900行、メインファイル）
- `chapters/en/standard-libraries/01-loading-the-libraries-in-c-code.html`
- `chapters/en/standard-libraries/02-basic-functions.html`（615行）
- `chapters/en/standard-libraries/03-coroutine-manipulation.html`
- `chapters/en/standard-libraries/04-modules.html`
- `chapters/en/standard-libraries/05-string-manipulation.html`
- `chapters/en/standard-libraries/06-utf-8-support.html`
- `chapters/en/standard-libraries/07-table-manipulation.html`
- `chapters/en/standard-libraries/08-mathematical-functions.html`
- `chapters/en/standard-libraries/09-input-and-output-facilities.html`
- `chapters/en/standard-libraries/10-operating-system-facilities.html`
- `chapters/en/standard-libraries/11-the-debug-library.html`

### 参考文献
- Lua 5.4日本語マニュアル: `chapters/ja/`ディレクトリ内のファイル（用語参考用）
- 用語対応表: `GLOSSARY.md`

### 出力先
- `crates/pasta_lua/doc/lua55-manual/06-standard-libraries.md`（置換）

