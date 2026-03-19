# Requirements Document

## Introduction

Pasta DSLの `・`（ローカルシーン）記法で定義されたシーンが、ランタイムのfinalize経路（`collect_scenes` → `register_global_raw` → `SceneTable`）を通ると名前解決に失敗する不具合を修正する。

この不具合は `act:call` からの動的呼び出しに限定されず、DSL `＞` Call文を含む**すべてのローカルシーン呼び出し**に影響する。

### 背景・根本原因

トランスパイラ（`scope_gen.rs`）はローカルシーン `・Head0` を Lua関数名 `__Head0_1__`（`__` + サニタイズ名 + `_` + カウンタ + `__`）としてコード生成する。`__` で囲む命名は本来 `__start__`（検索にマッチさせたくない特殊エントリーポイント）のための規約であり、一般のローカルシーンに適用する理由はない。

`__` ラッピングにより以下の不整合が発生する：

1. **Luaテーブルキーにマングル名が焼き付く**: `SCENE["__Head0_1__"] = func` として格納される
2. **元名が消失**: `collect_scenes` はテーブルキーを列挙するため、マングル名 `"__Head0_1__"` しか取得できない
3. **fn_nameフォーマット不整合**: finalize経路では `fn_name = "OnMouseMove1::__Head0_1__"` となり、トランスパイル時レジストリ（Path A）の `fn_name = "OnMouseMove_1::Head0_1"` と異なるフォーマットになる
4. **検索キー不一致**: `fn_name_to_search_key` が生成するキー `":OnMouseMove1:__Head0_1__"` は検索プレフィックス `":OnMouseMove1:Head0"` と前方一致しない
5. **`parse_fn_name` ダブルマングリング**: `parse_fn_name` が `local_part` に `format!("__{}__", ...)` を適用すると `"____Head0_1____"` になる

一般ローカルシーンの Lua関数名を `Head0_1`（`__` ラッピングなし）に変更すれば、全経路が自然に整合する。

### 影響範囲

| 呼び出し方式 | 影響 |
|---|---|
| DSL `＞` Call文（トランスパイラ生成の `act:call`） | ❌ 失敗 |
| Luaコードブロック内からの `act:call` | ❌ 失敗 |
| `SCENE.search` によるローカルシーン検索 | ❌ 失敗 |
| `__start__` の解決 | ✅ 正常（`__start__` は特殊名として別処理） |
| グローバルシーンの解決 | ✅ 正常（マングリング対象外） |

## Requirements

### Requirement 1: ローカルシーン名の Lua 関数命名規約修正

**Objective:** トランスパイラがローカルシーンに対して生成する Lua 関数名から不要な `__` ラッピングを除去し、`__start__` 以外のローカルシーンを `{サニタイズ名}_{カウンタ}` 形式で命名する。

#### Acceptance Criteria

1. The トランスパイラ（`scope_gen.rs`）shall ローカルシーン `・X` に対して Lua 関数名 `X_N`（`N` は1始まりの連番）を生成する。`__X_N__` 形式は使用しない。
2. The トランスパイラ shall `__start__` の命名規約を変更しない。`__start__` は引き続き `__` 付きの特殊名として扱う。
3. The `parse_fn_name`（context.rs）shall `fn_name` の `local_part` をそのまま Lua 関数名として返す。`__` ラッピングは `__start__` の場合のみ行い、それ以外は `format!("__{}__", ...)` を適用しない。

### Requirement 2: ローカルシーン検索の正常動作

**Objective:** ゴースト開発者として、ローカルシーンが DSL `＞` Call文・`act:call` 動的呼び出し・`SCENE.search` のいずれの経路でも正しく解決されることを保証したい。

#### Acceptance Criteria

1. When DSLの `＞` Call文でローカルシーンを呼び出した場合、pasta.dll shall 同一グローバルシーン内のローカルシーン関数を正しく解決して実行する。
2. When `act:call(scene_name, key, attrs)` でローカルシーンを元名で呼び出した場合、pasta.dll shall 同一グローバルシーン内のローカルシーン関数を正しく解決して実行する。
3. When 同一グローバルシーン内に複数の同名ローカルシーン（例: `・Head0` が2回定義）が存在する場合、pasta.dll shall 既存の重複シーン選択ルール（ランダム選択）と同等の動作を提供する。
4. When 検索プレフィックスでローカルシーンを呼び出した場合（例: `"Head"` で `・Head0` と `・Head1` をマッチ）、pasta.dll shall 前方一致で全候補を収集し、既存の選択ルールで1つを選んで実行する。
5. When 指定されたキーがローカルシーンとしてもグローバルシーンとしても見つからない場合、pasta.dll shall 既存のエラーログ出力動作を維持する。

### Requirement 3: 既存機能との互換性

**Objective:** ゴースト開発者として、この修正が `__start__` の解決、グローバルシーン検索、actメソッドフォールバック等の既存動作を一切破壊しないことを保証したい。

#### Acceptance Criteria

1. The pasta.dll shall `__start__` シーンの解決に影響を与えない。
2. The pasta.dll shall グローバルシーン検索を修正前と同一の動作で解決・実行する。
3. The pasta.dll shall `act:call` の Level 3〜4（`GLOBAL` テーブル参照、actメソッドフォールバック）の既存動作を変更しない。
4. The pasta.dll shall 名前付きLuaシーン関数（`function SCENE.xxx(act)` 形式）の解決動作を変更しない。

### Requirement 4: finalize経路の統合テスト

**Objective:** 今回の根本原因であるテストギャップ（finalize経路のテスト欠落）を埋め、再発を防止したい。

#### Acceptance Criteria

1. The テストスイート shall `register_global_raw` 経由で登録されたローカルシーンの前方一致検索が正しく動作することを検証するテストを含む。
2. The テストスイート shall finalize経路（`collect_scenes` → `build_scene_registry` → `SceneTable`）のラウンドトリップを検証するテストを含む。
