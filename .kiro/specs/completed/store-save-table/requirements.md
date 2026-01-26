# Requirements Document

## Introduction
このフィーチャーは、`pasta.store`モジュールに永続変数用の`save`テーブルを追加し、`CTX.new`メソッドで`STORE.save`を環境コンテキストの`ctx.save`として自動的に注入する機能を実装する。これにより、セッションを跨いで保持される永続変数の一元管理が実現される。

## Project Description (Input)
store.luaに、saveテーブルを追加し、CTX.newメソッドで、ctx.save引数として渡す。ctx.save = STORE.saveです。

## Requirements

### Requirement 1: STOREモジュールへのsaveテーブル追加
**Objective:** ランタイム開発者として、永続変数を一元管理したい。これにより、セッション間でデータを保持し、循環参照なしで他モジュールからアクセスできるようになる。

#### Acceptance Criteria
1. The pasta.store module shall have a `save` field of type `table<string, any>` for persistent variables.
2. When `STORE.reset()` is called, the STORE module shall reset `STORE.save` to an empty table.
3. The pasta.store module shall include LuaDoc annotation `@field save table<string, any>` in the `@class Store` definition.

### Requirement 2: CTX.newでのSTORE.save自動注入
**Objective:** シーン作成者として、CTX作成時にsaveが自動設定されるようにしたい。これにより、手動でsaveを渡す必要がなくなり、シンプルな呼び出しが可能になる。

#### Acceptance Criteria
1. When `CTX.new()` is called, the CTX module shall set `ctx.save` to `STORE.save`.
2. The CTX module shall require `pasta.store` to access `STORE.save`.
3. The `CTX.new()` function shall not accept `save` parameter (simplified interface).

### Requirement 3: 循環参照回避の維持
**Objective:** アーキテクト として、循環参照回避パターンを維持したい。これにより、モジュール間の依存関係が一方向に保たれる。

#### Acceptance Criteria
1. The pasta.store module shall not require any other pasta modules (maintain zero dependencies).
2. The pasta.ctx module shall require pasta.store to obtain `STORE.save` reference.
