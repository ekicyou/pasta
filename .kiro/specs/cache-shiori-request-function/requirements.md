# Requirements Document

## Introduction
PastaShiori における SHIORI Lua 関数呼び出しの最適化仕様。現在、`request()` メソッド内で毎回 `globals.get("SHIORI")` → `shiori_table.get("request")` を実行しており、オーバーヘッドが発生している。`load()` 時に関数参照をキャッシュすることで、リクエスト処理のパフォーマンスを向上させる。

## Project Description (Input)
PastaShiori::request内で、「let request_fn: Function = match shiori_table.get("request")」などとやっている個所があるが、Functionは毎回同じ関数を呼び出すので無駄な処理をしていると思う。 PastaShiori::loadで、request_fnを取得しておくように変更できるか？Functionは`aなどがついてないので普通にできるはず。

**追加要件**: load_fn / unload_fn もキャッシュして欲しい。

## Requirements

### Requirement 1: SHIORI 関数キャッシュ構造
**Objective:** As a pasta_shiori 開発者, I want SHIORI.load / SHIORI.request / SHIORI.unload 関数への参照を load 時にキャッシュする, so that リクエスト処理時の関数ルックアップオーバーヘッドを削減できる

#### Acceptance Criteria
1. The PastaShiori shall store optional `Function` references for `SHIORI.load`, `SHIORI.request`, and `SHIORI.unload` as struct fields
2. When `load()` is called, the PastaShiori shall retrieve and cache all available SHIORI functions from the Lua runtime
3. When functions are not found in Lua, the PastaShiori shall set the corresponding cache field to `None`
4. The PastaShiori shall clear all cached function references when runtime is released or reloaded

### Requirement 2: request() 最適化
**Objective:** As a SHIORI クライアント, I want request() がキャッシュされた関数を使用する, so that 各リクエストでの関数ルックアップが不要になる

#### Acceptance Criteria
1. When `request()` is called and `request_fn` cache is `Some`, the PastaShiori shall use the cached function directly
2. When `request()` is called and `request_fn` cache is `None`, the PastaShiori shall return the default 204 response
3. The PastaShiori shall not access `globals.get("SHIORI")` during request processing

### Requirement 3: load 関数呼び出しの最適化
**Objective:** As a pasta_shiori 開発者, I want SHIORI.load 呼び出しもキャッシュを利用する, so that call_shiori_load 内の関数ルックアップも削減できる

#### Acceptance Criteria
1. When `call_shiori_load()` is invoked, the PastaShiori shall use the cached `load_fn` if available
2. If `load_fn` cache is `None`, the PastaShiori shall skip the SHIORI.load call and return success

### Requirement 4: unload 関数サポート
**Objective:** As a SHIORI 仕様準拠開発者, I want SHIORI.unload 関数をサポートする, so that ゴーストのアンロード時にクリーンアップ処理を実行できる

#### Acceptance Criteria
1. When `PastaShiori` is dropped and `unload_fn` cache is `Some`, the PastaShiori shall call the cached unload function
2. When `unload_fn` call fails, the PastaShiori shall log the error but not propagate it
3. The PastaShiori shall call unload before clearing the runtime

### Requirement 5: 既存フラグとの整合性
**Objective:** As a pasta_shiori 開発者, I want 既存の `has_shiori_load` / `has_shiori_request` フラグを関数キャッシュで置き換える, so that 冗長なフラグ管理を削減できる

#### Acceptance Criteria
1. The PastaShiori shall remove `has_shiori_load` and `has_shiori_request` boolean fields
2. The PastaShiori shall use `Option<Function>` cache fields to determine function availability (`.is_some()`)
3. When checking function availability, the PastaShiori shall use the cache field's `is_some()` method

### Requirement 6: テスト互換性
**Objective:** As a pasta_shiori テスト担当者, I want 既存のテストが引き続き動作する, so that リファクタリングによるリグレッションを防止できる

#### Acceptance Criteria
1. The PastaShiori shall pass all existing unit tests after the refactoring
2. When runtime is loaded multiple times, the PastaShiori shall correctly update cached functions for each load
3. The PastaShiori shall maintain independent function caches across multiple instances
