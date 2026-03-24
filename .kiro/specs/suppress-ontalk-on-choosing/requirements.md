# Requirements Document

## Introduction

SSP（SHIORI Server Protocol）は、SHIORI リクエストの `Status` ヘッダーでゴーストの現在状態を通知する。ユーザーが選択肢（`\q[]` タグ）を提示されている間、`Status` ヘッダーには `choosing` が含まれる。この状態中にランダムトーク（OnTalk）や時報（OnHour）が発動すると、選択肢UIが消失しユーザー体験を損なう。本仕様は `choosing` 状態を検出し、仮想イベント発動を抑制する機能を定義する。

### 背景情報

- SSP の `Status` ヘッダーはカンマ区切りで複数状態を同時送出する（例: `talking,choosing,balloon(0=2)`）
- 既存実装では `talking` 状態での抑制は完全一致（`==`）で行われているが、`Status` ヘッダーの実際の値は `talking,balloon(0=0)` のようにカンマ区切りで複合される場合がある
- `choosing` は常に `talking` と併記される傾向にあるが、仕様上は単独出現の可能性もある

### 参考

- SSP SHIORI/3.0 仕様: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html

## Project Description (Input)

Statusに「choosing」が出ている間、OnTalk発動を抑制すること。

## Requirements

### Requirement 1: choosing 状態での OnTalk 抑制

**Objective:** ゴースト開発者として、ユーザーが選択肢を選んでいる最中にランダムトークが割り込まないようにしたい。選択肢が意図せず消えてしまうことを防ぐためである。

#### Acceptance Criteria

1. While SHIORI リクエストの `Status` ヘッダーに `choosing` が含まれている, the virtual_dispatcher shall OnTalk の発動をスキップする（`nil` を返す）。
2. While SHIORI リクエストの `Status` ヘッダーに `choosing` が含まれている, the virtual_dispatcher shall 次回トーク時刻の再計算を行わない（タイマーを消費しない）。

### Requirement 2: choosing 状態での OnHour 抑制

**Objective:** ゴースト開発者として、ユーザーが選択肢を選んでいる最中に時報が割り込まないようにしたい。OnTalk と同じ理由で選択肢消失を防ぐためである。

#### Acceptance Criteria

1. While SHIORI リクエストの `Status` ヘッダーに `choosing` が含まれている, the virtual_dispatcher shall OnHour の発動をスキップする（`nil` を返す）。
2. While `choosing` 状態で OnHour がスキップされた場合, the virtual_dispatcher shall 次の正時タイムスタンプを更新しない（正時到達後に choosing が解除されれば発火可能とする）。

### Requirement 3: Status ヘッダーのカンマ区切り対応（choosing・talking 共通）

**Objective:** ゴースト開発者として、`Status` ヘッダーが `talking,choosing,balloon(0=2)` のようにカンマ区切りで複合された場合でも正しく状態を検出したい。既存の完全一致（`==`）判定を「含む」（部分一致）判定に統一し、choosing 新規対応と既存 talking 判定の CSV 不具合を同時に解消するためである。

> **決定事項（ディスカッション #1）**: 既存 talking 判定の CSV 不具合修正（旧 Req 4）を本 spec に含めることを承認。  
> **理由**: 変更対象が同一ファイル・同一ガード節であり、検出方式を「含む」パターンに統一することで choosing・talking を一貫して扱える。  

#### Acceptance Criteria

1. When `Status` ヘッダーが `choosing` 単独の場合, the virtual_dispatcher shall choosing 状態として検出する。
2. When `Status` ヘッダーが `talking,choosing,balloon(0=2)` のようにカンマ区切りで複合されている場合, the virtual_dispatcher shall choosing 状態として検出する。
3. When `Status` ヘッダーが `talking` のみの場合, the virtual_dispatcher shall choosing 状態として検出しない（talking 抑制のみ適用）。
4. When `Status` ヘッダーが `talking,balloon(0=0)` のようにカンマ区切りで複合されている場合, the virtual_dispatcher shall talking 状態として検出し OnTalk / OnHour をスキップする。

### Requirement 4: テストカバレッジ

**Objective:** 開発者として、choosing 抑制および CSV 対応 talking 抑制の挙動を自動テストで保証したい。リグレッションを防止するためである。

#### Acceptance Criteria

1. The Lua テストスイート shall choosing 状態での OnTalk スキップを検証するテストケースを含む。
2. The Lua テストスイート shall choosing 状態での OnHour スキップを検証するテストケースを含む。
3. The Lua テストスイート shall カンマ区切り Status（例: `talking,choosing,balloon(0=2)`）での choosing 正常検出を検証するテストケースを含む。
4. The Lua テストスイート shall カンマ区切り Status（例: `talking,balloon(0=0)`）での talking 正常検出を検証するテストケースを含む。
