# Implementation Plan

- [x] 1. おしゃべり頻度永続化コアロジックの実装
- [x] 1.1 SAVE/toml/ハードコードの3段フォールバックロジック実装
  - `cached_config` モジュールローカル変数と早期リターンを除去し、毎回 SAVE テーブルを読み直す方式に変更する
  - ローカル関数 `resolve(save_key, toml_key, default)` を `get_config()` 内に定義し、`pasta.save`（SAVE テーブル）→ `@pasta_config`（pasta.toml `[ghost]` セクション）→ ハードコードデフォルトの優先順位で値を解決する
  - `type(sv) == "number"` ガードで SAVE 値および toml 値が数値でない場合を無視し、次の優先順位にフォールバックする
  - `talk_interval_min` 解決後、`talk_interval_max` を解決し、`min > max` のとき `max = min` に補正する
  - `hour_margin` は従来通り toml のみ参照（永続化対象外）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 3.1, 3.2, 3.3_

- [x] 1.2 テスト用モジュール公開インターフェースの整理
  - `M._reset()` から `cached_config = nil` 行を除去する（キャッシュ変数廃止）
  - `M._get_internal_state()` の返却テーブルから `cached_config` フィールドを除去する
  - `M._get_config = get_config` を新規追加し、テストコードが設定解決結果を直接検証できるようにする
  - _Requirements: 1.1, 2.1_

- [x] 2. 既存設定取得テストのキャッシュ参照更新
  - `test_config_default_values`: `state.cached_config` による検証を、`dispatcher._get_config()` 呼び出し結果を検証する方式に置き換える
  - `test_module_state_reset`: `state_after.cached_config == nil` の検証を、リセット後に `_get_config()` がデフォルト値を返すことの検証に置き換える
  - `test_internal_state_getter`: `state.cached_config == nil` の検証を、`cached_config` フィールドが存在しないことの検証に更新する
  - タスク 1.2 の `M._get_config` 追加完了が前提
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 3. 新規フォールバック検証テストの追加
- [x] 3.1 SAVE優先・実行時変更・部分設定テストの追加
  - `test_save_priority_over_toml_and_default`: SAVE テーブルに `pasta_talk_interval_min` / `pasta_talk_interval_max` を設定し、`_get_config()` が SAVE 値を返すことを検証する
  - `test_runtime_change_reflected_immediately`: dispatch 呼び出し後に SAVE 値を変更し、次回 `_get_config()` 呼び出しで変更後の値が使用されることを検証する（キャッシュ廃止の暗黙的検証）
  - `test_partial_save_configuration`: `pasta_talk_interval_min` のみ SAVE に設定し、min は SAVE 値・max はデフォルト (300) が返ることを検証する
  - _Requirements: 1.1, 1.2, 1.3, 2.1_

- [x] 3.2 tomlフォールバック・ハードコードデフォルトテストの追加
  - `test_toml_fallback_values`: テスト冒頭でインラインモック `package.loaded["@pasta_config"] = { ghost = { talk_interval_min = 60, talk_interval_max = 90 } }` を挿入し（案B方式）、SAVE 未設定時に toml 値が使用されることを検証する
  - `test_hardcoded_default_values`: SAVE・toml ともに未設定（デフォルトランタイム）で `_get_config()` が min=180, max=300 を返すことを検証する
  - `create_runtime_with_pasta_path()` は変更しない；インラインモックは各テスト関数のランタイムインスタンスにのみ有効であることを確認する
  - _Requirements: 1.1, 1.4, 1.5_

- [x] 3.3 バリデーション（非数値・min>max補正）テストの追加
  - `test_non_numeric_save_values_fallback`: SAVE テーブルに文字列値を設定し（例: `pasta_talk_interval_min = "fast"`）、無視されてデフォルト値が返ることを検証する
  - `test_min_greater_than_max_correction`: SAVE テーブルに `pasta_talk_interval_min = 500`, `pasta_talk_interval_max = 100` を設定し、返却値が両方 500 になることを検証する
  - _Requirements: 3.1, 3.2, 3.3_

- [x] 4. (P) `pasta-lua-coding` スキルへの SAVE キー命名規約追記
  - `.agents/skills/pasta-lua-coding/SKILL.md` の §3 Coding Conventions に「SAVE キー命名規約」セクションを追加する：エンジン予約キー（`pasta_` プレフィックス付き; 例: `pasta_talk_interval_min`）とゴースト固有キー（任意命名; `pasta_` プレフィックス使用禁止）の区別を明記する
  - `.agents/skills/pasta-lua-coding/references/internal-modules.md` の `pasta.save` セクションに同命名規約を追記する
  - `SKILL.md` の `metadata.version` をバンプする
  - タスク 1～3 とは独立して実施可能（異なるファイルのみ変更）
  - _Requirements: 4.1, 4.2_
