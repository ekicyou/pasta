# Requirements Document

## Introduction

おしゃべり頻度（`talk_interval_min` / `talk_interval_max`）を `pasta.save`（SAVE テーブル）に永続化し、`pasta.toml` の値をデフォルトフォールバックとして扱う仕様を定義する。ユーザーがゴーストとの対話中におしゃべり頻度を変更した場合、その設定がセッションを跨いで保持される。

### 現状

- `talk_interval_min` / `talk_interval_max` は `pasta.toml` の `[ghost]` セクションで定義
- `virtual_dispatcher.lua` の `get_config()` がモジュールロード時に 1 回だけ読み込み、セッション中キャッシュ
- 値の変更手段・永続化手段は存在しない

### 用語

| 用語 | 定義 |
|------|------|
| おしゃべり頻度 | `talk_interval_min` / `talk_interval_max` の組（秒単位の乱数区間） |
| SAVE テーブル | `pasta.save` ファイルに永続化されるグローバルLuaテーブル |
| デフォルト値 | `pasta.toml` の `[ghost]` セクションに記述された設定値 |

## Requirements

### Requirement 1: SAVE テーブルからのおしゃべり頻度読み出しと優先順位

**Objective:** ゴースト作者として、おしゃべり頻度設定がセッションを跨いで保持されるようにしたい。ユーザーが調整した頻度が次回起動時にも反映され、優先順位が予測可能であるため。

#### Acceptance Criteria

1. The virtual_dispatcher shall 以下の優先順位でおしゃべり頻度を決定する: (1) SAVE テーブルの値 → (2) `pasta.toml` `[ghost]` セクションの値 → (3) ハードコードデフォルト値（`talk_interval_min=180`, `talk_interval_max=300`）。
2. When SAVE テーブルに `talk_interval_min` が存在する場合, the virtual_dispatcher shall SAVE テーブルの値をおしゃべり頻度の最小値として使用する。
3. When SAVE テーブルに `talk_interval_max` が存在する場合, the virtual_dispatcher shall SAVE テーブルの値をおしゃべり頻度の最大値として使用する。
4. If SAVE テーブルに `talk_interval_min` が存在しない場合, the virtual_dispatcher shall `pasta.toml` の `[ghost].talk_interval_min` の値をデフォルトとして使用する。
5. If SAVE テーブルに `talk_interval_max` が存在しない場合, the virtual_dispatcher shall `pasta.toml` の `[ghost].talk_interval_max` の値をデフォルトとして使用する。

### Requirement 2: おしゃべり頻度の実行時変更

**Objective:** ゴースト作者として、Luaスクリプトからおしゃべり頻度を動的に変更できるようにしたい。ユーザー操作や対話イベントに応じて頻度を調整するため。

#### Acceptance Criteria

1. When Luaスクリプトから SAVE テーブルの `talk_interval_min` / `talk_interval_max` を変更した場合, the virtual_dispatcher shall 次回のトーク間隔計算から変更後の値を反映する。

### Requirement 3: 値のバリデーション

**Objective:** ゴースト作者として、不正な値が設定された場合でもランタイムが安定して動作するようにしたい。ランタイムエラーを防止するため。

#### Acceptance Criteria

1. If SAVE テーブルの `talk_interval_min` が数値でない場合, the virtual_dispatcher shall その値を無視し、次の優先順位の値にフォールバックする。
2. If SAVE テーブルの `talk_interval_max` が数値でない場合, the virtual_dispatcher shall その値を無視し、次の優先順位の値にフォールバックする。
3. If `talk_interval_min` が `talk_interval_max` より大きい場合, the virtual_dispatcher shall 両方の値を `talk_interval_min` と同じ値として扱う。
