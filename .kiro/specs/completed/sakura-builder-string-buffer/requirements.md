# Requirements Document

## Introduction
pasta Lua ランタイムのさくらスクリプト組み立ての中核 `pasta.shiori.sakura_builder` の
`BUILDER.build` は、現在 `local buffer = {}` への `table.insert` 蓄積と `table.concat` 結合で
文字列を生成している。トークン数の多いトークでは中間テーブルの成長と GC 負荷が無視できない。

本仕様では、再利用可能なバッファ抽象モジュール `pasta.buf` を新設して `new()` を公開し、
LuaJIT String Buffer Library（`string.buffer`）が利用可能ならそれを採用、利用不可な環境では
最小実装フォールバックを返す。そのうえで `sakura_builder.build` をこのバッファ経由の組み立てへ
置き換える。受け入れの中核は **さくらスクリプト出力のバイト一致（外部振る舞い不変）** であり、
高速化はその制約下で期待する効果と位置づける。

ランタイムは mlua 0.11（`luajit52` / `vendored`、LuaJIT 2.1）。既存テスト
`tests/lua_specs/sakura_builder_test.lua` が振る舞い回帰の検証基盤となる。

## Boundary Context
- **In scope**:
  - `pasta.buf` モジュールの新設（`new()` + 最小実装フォールバック）
  - `sakura_builder.build` のバッファ化（外部振る舞い不変）
  - 既存テストでの回帰検証と、最小実装フォールバック経路の出力同一性検証
  - 対象ランタイム（mlua `luajit52`/`vendored`）で String Buffer 高速パスが採用されることの実機検証
- **Out of scope**:
  - `act.lua` / `scene.lua` など他モジュールの文字列組み立てバッファ化
  - `pasta.buf` の高度 API（`get`/`set`/`reserve`/`skip`/`encode`/`decode`/FFI 連携）
  - 性能数値のベンチマーク測定・速度閾値による合否判定（高速パスの**採用有無**は検証するが、**速度の数値測定**は行わない）
- **Adjacent expectations**:
  - LuaJIT String Buffer Library（mlua vendored 同梱）が利用可能な環境では、その `new` 実体が
    バッファ生成を担うことを前提とする
  - `pasta.shiori.act`（`BUILDER.build` の呼び出し元）は、出力契約（`\e` 終端・従来と同一文字列）に
    依存し続ける。本仕様はこの契約を変更しない

## Requirements

### Requirement 1: バッファ抽象モジュール `pasta.buf` の提供
**Objective:** As a pastaランタイムのスクリプト開発者, I want 文字列を効率的に連結できる再利用可能なバッファ抽象, so that さくらスクリプト等の組み立てで中間テーブルと GC 負荷を避けられる

#### Acceptance Criteria
1. The buf モジュール shall `new()` 関数を公開する
2. When `new()` が呼ばれたとき, the buf モジュール shall 追記メソッドと結合取り出しメソッドを備えたバッファオブジェクトを返す
3. While LuaJIT String Buffer Library が利用可能な環境, the buf モジュール shall その `new` をバッファ生成の実体として採用する
4. When バッファに複数の文字列が順に追記されたとき, the buf モジュール shall 追記順を保持して蓄積する
5. When 結合取り出しが要求されたとき, the buf モジュール shall それまでに追記された全文字列を追記順どおりに連結した単一文字列を返す

### Requirement 2: 最小実装フォールバック
**Objective:** As a pastaランタイムのスクリプト開発者, I want LuaJIT String Buffer が存在しない環境でも同じ呼び出し方で動くフォールバック, so that 実行環境に依存せず buf を安全に利用できる

#### Acceptance Criteria
1. If LuaJIT String Buffer Library が利用不可能な環境, then the buf モジュール shall 最小実装のバッファを `new()` の戻り値として提供する
2. If String Buffer Library の検出に失敗したとき, then the buf モジュール shall 例外を送出せず最小実装へフォールバックする
3. The 最小実装バッファ shall `sakura_builder.build` が使用するメソッド（追記・結合取り出し）を LuaJIT String Buffer と同一のシグネチャで提供する
4. While 最小実装が使用されている状態, when 同一の追記列に対して結合取り出しが行われたとき, the buf モジュール shall LuaJIT String Buffer 使用時と同一の連結結果を返す

### Requirement 3: `sakura_builder.build` のバッファ化と出力不変性
**Objective:** As a ゴースト作者, I want トーク生成が従来と同じ出力のまま効率化されること, so that 既存の辞書がそのまま動き、トークの表示結果が一切変わらない

#### Acceptance Criteria
1. The sakura_builder モジュール shall `BUILDER.build` 内部のさくらスクリプト組み立てに buf モジュールのバッファを使用する
2. When 任意の grouped_tokens 入力に対して `BUILDER.build` が呼ばれたとき, the sakura_builder モジュール shall バッファ化前と完全に同一（バイト一致）のさくらスクリプト文字列を返す
3. The sakura_builder モジュール shall 出力文字列を従来どおり `\e` で終端する
4. When 空の grouped_tokens（要素なし）に対して `BUILDER.build` が呼ばれたとき, the sakura_builder モジュール shall `\e` のみの文字列を返す
5. While LuaJIT String Buffer が利用可能・利用不可能のいずれの環境でも, when 同一の grouped_tokens に対して `BUILDER.build` が呼ばれたとき, the sakura_builder モジュール shall 同一の出力文字列を返す

### Requirement 4: 回帰検証と検証可能性
**Objective:** As a メンテナ, I want 振る舞い不変が自動テストで保証されること, so that バッファ化による退行を確実に防げる

#### Acceptance Criteria
1. The sakura_builder モジュール shall 既存の sakura_builder テストスイートを全て成功させ続ける
2. Where 最小実装フォールバックが提供される, the テストスイート shall 最小実装経路でも `BUILDER.build` 出力の同一性を検証する
3. The テストスイート shall `pasta.buf` の `new()` と最小実装バッファの追記・結合取り出し挙動を直接検証する

### Requirement 5: 対象ランタイムでの String Buffer 採用検証
**Objective:** As a メンテナ, I want 対象ランタイムで高速パス（String Buffer）が実際に採用されることを実機検証, so that 「高速化」という目的が空洞化せず、常時フォールバックを見逃さない

#### Acceptance Criteria
1. The テストスイート shall 対象ランタイム（mlua `luajit52`/`vendored`）で `require("string.buffer")` が成功し、`pasta.buf.new()` が String Buffer 実体を採用することを確認する
2. If 対象ランタイムで String Buffer Library が利用不可能と判明したとき, then the メンテナ shall それを finding として記録し、最小実装フォールバックを黙って合格としない
3. When String Buffer 利用不可が検出されたとき, then the 検証 shall finding に `pasta.lua_version` の数値（実行ランタイム識別）を含める

### Requirement 6: Lua ランタイムバージョンの数値取得
**Objective:** As a pastaランタイムのスクリプト開発者, I want 実行中ランタイムの種別とバージョンを単一の数値で取得, so that LuaJIT/標準Lua を正確に判別でき、診断（R5 finding）や条件分岐に使える

#### Acceptance Criteria
1. The lua_version モジュール shall 実行中ランタイムを表す単一の整数を返す関数を公開する
2. While LuaJIT 上で実行中, the lua_version モジュール shall `200 + major×10 + minor`（LuaJIT のバージョン）を返す（例: LuaJIT 2.1 → 221、2.0 → 220）
3. While 標準 Lua 上で実行中, the lua_version モジュール shall `100 + major×10 + minor`（Lua のバージョン）を返す（例: Lua 5.4 → 154、5.5 → 155）
4. When ランタイム種別を判定するとき, the lua_version モジュール shall `jit` テーブルの有無で LuaJIT を判定する（`_VERSION` は LuaJIT を "Lua 5.1" と誤報するため不可）
5. The lua_version モジュール shall 例外を送出せず常に整数を返す
