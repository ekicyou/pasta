# Requirements Document

## Project Description (Input)
単語辞書の定義パターンに、複数キー指定ができるパターンの追加。
「＠女性、水の妖精：水無灯里、アリス・キャロル」とした場合に、「＠女性」でも、「＠水の妖精」でも、「水無灯里」or「アリス・キャロル」が単語選択肢となる。

## スコープ

- **対象クレート**: `pasta_dsl`（PEG文法 + AST変換）+ `pasta_lua`（トランスパイル・コード生成・レジストリ登録）
- **対象外**: `pasta_core`（`WordDefRegistry`/`WordTable` — 構造変更不要）、`pasta_lsp`（`span`のみ使用、`name`未参照）
- **実装制約**: 段階的にコンパイル可能な状態を維持する（各ステップで`cargo test --all`が通ること）
- **設計方針**: AST変更→文法変更→pasta_lua対応の順でインクリメンタルに進行

## Introduction

Pasta DSLの単語定義（`＠key：values`）は現在、1つのキーに対して複数の値を紐付ける構文のみをサポートしている。本仕様では、**複数のキーを同一の値リストに紐付ける**構文パターンをPEG文法・AST層・Luaトランスパイル層にわたって追加する。これにより、ゴースト作者は同一の単語候補群を複数のカテゴリ名で参照できるようになり、エンドツーエンドで動作する。

## Requirements

### Requirement 1: 複数キー単語定義の文法拡張

**Objective:** ゴースト作者として、1行の単語定義で複数のキーを指定したい。それにより、同一の値リストを異なるキー名で参照でき、辞書の重複記述を削減できる。

#### Acceptance Criteria

1. When `＠key1、key2：value1、value2` 形式の単語定義行がパースされた場合, the pasta_dsl parser shall コロン（`：`/`:`）の左側をカンマ区切りの複数キーとして認識し、右側を値リストとして認識する。
2. When コロン左側にキーが1つだけ指定された場合（既存の `＠key：values` 形式）, the pasta_dsl parser shall 従来どおり単一キーの単語定義として正常にパースする（後方互換性）。
3. When コロン左側に3つ以上のキーがカンマ区切りで指定された場合（`＠k1、k2、k3：values`）, the pasta_dsl parser shall すべてのキーを正しく認識する。
4. The pasta_dsl parser shall 全角カンマ（`、`）・半角カンマ（`,`）・全角コンマ（`，`）のいずれもキー区切りとして受け付ける（既存の値区切りと同一のカンマ仕様に準拠）。
5. The pasta_dsl parser shall ファイルレベル（`file_word_line`）・グローバルシーンスコープ（`global_scene_word_line`）・アクター辞書（`actor_scope_item`）のいずれの文脈でも複数キー構文を受け付ける（`key_words` ルール共有による自動波及）。

### Requirement 2: AST表現の拡張

**Objective:** DSL利用者（`pasta_lua`トランスパイラ等）として、パース結果のASTから複数キー情報を取得したい。それにより、後続の登録・検索処理で各キーに対して同一の値リストを紐付けられる。

#### Acceptance Criteria

1. When 複数キー単語定義がパースされた場合, the pasta_dsl parser shall AST上で各キー名を保持する構造体を出力する。
2. The `KeyWords` AST構造体 shall 複数キー情報を表現できるフィールドを持つ（具体的な設計はOption A/B/C から設計フェーズで決定）。
3. When 単一キーの従来形式がパースされた場合, the `KeyWords` AST構造体 shall 既存ASTとの意味的互換性を維持する。
4. The `KeyWords` AST構造体 shall 各キーが定義された元のソース位置情報（`Span`）を保持する。
5. The `KeyWords` AST構造体 shall `pasta_lua`のトランスパイラ層がキーごとにレジストリ登録を呼び出せるよう、キーリストを列挙可能な形式で提供する。
6. The `KeyWords` AST構造体 shall 値リスト（`words: Vec<String>`）は複数キー間で共有される単一リストとして保持する（キーごとに値を複製しない）。

### Requirement 3: エラーハンドリング

**Objective:** ゴースト作者として、構文エラーがある場合に明確なエラーメッセージを受け取りたい。

#### Acceptance Criteria

1. If コロンが存在しない `＠key1、key2` 形式が入力された場合, the pasta_dsl parser shall 既存の動作に従いパースエラーとして処理する（PEG文法が `kv_marker` を要求するため自動的にマッチ失敗する。破壊的変更を起こさない）。
2. If キー部分に空文字のキーが含まれる場合（`＠、key2：values` や `＠key1、、key2：values`）, the pasta_dsl parser shall PEG文法の `id` ルールにより自動的にマッチ失敗し、パースエラーとなる。

### Requirement 4: pasta_lua トランスパイル・レジストリ登録対応

**Objective:** ゴースト作者として、複数キーで定義した単語をいずれのキー名でも参照したい。それにより、`＠女性`でも`＠水の妖精`でも同じ候補リストから選択される。

#### Acceptance Criteria

1. When 複数キーの`KeyWords` ASTがトランスパイルされた場合, the pasta_lua transpiler shall 各キーに対して`WordDefRegistry`への登録（`register_global` / `register_local` / `register_actor`）を実行する。
2. When 複数キーの`KeyWords` ASTからLuaコードが生成された場合, the pasta_lua code generator shall 各キーに対して`PASTA.create_word(key):entry(...)` / `SCENE:create_word(key):entry(...)` / `ACTOR:create_word(key):entry(...)` を出力する。
3. When 単一キーの従来形式がトランスパイルされた場合, the pasta_lua transpiler shall 既存と同一の出力を生成する（後方互換性）。
4. The pasta_lua transpiler shall `WordDefRegistry` / `WordTable` の構造変更なしに、同一`values`を複数`key`で登録することで動作する。

## 備考

- **`pasta_core`**: `WordDefRegistry`/`WordTable`は構造変更不要。同一`values`を複数`key`で`register_*`すれば既存の前方一致検索がそのまま動作する。
- **`pasta_lsp`**: `visit_keywords`は`word.span`のみ使用しており`name`フィールド未参照。本仕様では変更不要。
- **実装順序制約**: AST拡張→PEG文法変更→pasta_lua対応の順で、各ステップでコンパイルが通る状態を維持する。
