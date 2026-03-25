# 実装タスク: ghost-authoring-skill-restructure

## 実装計画

<!-- フェーズ構成:
  Phase 1（並列可）: Task 1(C-VR)、Task 2(C-PT)、Task 3(C-AP) — ファイルが独立しており相互依存なし
  Phase 2: Task 4(C-SM) — Phase 1 完了後に実施
  Phase 3: Task 5（検証）— Task 4 完了後に実施
-->

- [x] 1. (P) variables.md に永続化メカニズムと SAVE テーブル情報を追記する
- [x] 1.1 既存スコープ表のグローバル変数説明を更新する
  - `references/variables.md` の変数スコープ一覧表にある「グローバル変数 `＄＊変数名`」行の「永続的に有効」を「SAVE テーブル経由でセッション間にわたりファイルに永続化される（JSON 保存）」に変更する
  - _Requirements: 1, 2_
- [x] 1.2 「永続化と SAVE テーブル」セクションをファイル末尾に追記する
  - 既存の「自動設定される日時変数」セクションの後に `---` 区切りを挟んで `## 永続化と SAVE テーブル` セクションを追加する
  - 永続化メカニズムの説明として、`＄変数名`（ローカル、シーン終了で消滅）と `＄＊変数名`（SAVE テーブル経由、`profile/pasta/save/save.json` に JSON 保存）の比較表を記載する
  - `pasta-lua-coding` スキルの `references/runtime-api.md`（`@pasta_persistence`）および `references/internal-modules.md`（SAVE モジュール）へのクロスリファレンスリンクを記載する
  - `pasta_` プレフィックスをエンジン予約キーとして定義し、`pasta_talk_interval_min`（既定値 `180` 秒）と `pasta_talk_interval_max`（既定値 `300` 秒）の用途・影響・3段フォールバック（SAVE テーブル > pasta.toml > ハードコード既定値）を表形式で記載する
  - ユーザー独自変数に `pasta_` プレフィックスを使用しないよう警告文を追記する
  - _Requirements: 1, 2_

- [x] 2. (P) references/pasta-toml.md リファレンスファイルを新設する
- [x] 2.1 ファイル骨格と概要セクションを作成する
  - `references/pasta-toml.md` を新規作成し、タイトル・導入文・最小構成のサンプルを記載する
  - エンジン正式解析セクション（Rust 構造体で型検証）とカスタムフィールドセクション（Lua に透過）の2種類があることを簡潔に説明する
  - _Requirements: 3_
- [x] 2.2 エンジン正式解析5セクションの全キーを記載する
  - `[loader]` セクション（`pasta_patterns`・`lua_search_paths`・`transpiled_output_dir`・`debug_mode` の4キー）を記載する — `lua_search_paths` の5パス既定値（profile/pasta/save/lua、scripts、pasta_scripts、profile/pasta/cache/lua、scriptlibs）を列挙する
  - `[logging]` セクション（`file_path`・`rotation_days`・`level`・`filter` の4キー）を記載する
  - `[persistence]` セクション（`obfuscate`・`file_path`・`debug_mode` の3キー）を記載する
  - `[lua]` セクション（`libs` の1キー）を「上級者向け」として簡潔に記載する
  - `[talk]` セクション（`script_wait_*` 5キー + `chars_*` 7キー、計12キー）を記載する
  - 各キーにつき、型・既定値・説明・使用例を記載する
  - _Requirements: 3_
- [x] 2.3 カスタムフィールド3セクションの全キーと BudouX 設定を記載する
  - `[package]` セクション（`name`・`version`・`edition` の3キー）を「上級者向け」注記とともに簡潔に記載する
  - `[ghost]` セクション（`talk_interval_min`・`talk_interval_max`・`hour_margin`・`spot_newlines` の4キー）を記載する — SAVE テーブルの `pasta_talk_interval_min/max` による上書きが可能な点を注記する
  - `[actor."名前"]` セクション（`spot`・`budoux`・`default_surface` の3キー）を記載する — `budoux` キーについては配列形式 `[行1文字幅, 行2以降文字幅]` の意味と BudouX 自動改行動作を詳細に説明し、設定例（例: `budoux = [10, 12]`）を含める
  - _Requirements: 3_

- [x] 3. (P) references/authoring-patterns.md を新設して §6 パターン集を移動する
- [x] 3.1 ファイルを新設し §6.1〜§6.5 のパターンを移動する
  - `references/authoring-patterns.md` を新規作成し、ファイルヘッダー（タイトル、SKILL.md §6 から移動した旨の説明）を記載する
  - SKILL.md の §6.1（アクター辞書定義）の内容をそのまま移動する — 見出し直前に `<a id="s6-1"></a>` アンカーを付与する
  - SKILL.md の §6.2（イベントハンドラ）の内容をそのまま移動する — 見出し直前に `<a id="s6-2"></a>` アンカーを付与する
  - SKILL.md の §6.3（ランダムトーク）の内容をそのまま移動する — 見出し直前に `<a id="s6-3"></a>` アンカーを付与する
  - SKILL.md の §6.4（時報）の内容をそのまま移動する — 見出し直前に `<a id="s6-4"></a>` アンカーを付与する
  - SKILL.md の §6.5（クリック反応）の内容をそのまま移動する — 見出し直前に `<a id="s6-5"></a>` アンカーを付与する
  - _Requirements: 4_
- [x] 3.2 §6.6〜§6.10 のパターンを移動する
  - SKILL.md の §6.6（単語ランダム選択・シャッフル＆順次消費方式）の内容をそのまま移動する — 見出し直前に `<a id="s6-6"></a>` アンカーを付与する
  - SKILL.md の §6.7（継続トーク・チェイントーク）の内容をそのまま移動する — 見出し直前に `<a id="s6-7"></a>` アンカーを付与する
  - SKILL.md の §6.8（ファイル分割ガイド）の内容をそのまま移動する — 見出し直前に `<a id="s6-8"></a>` アンカーを付与する
  - SKILL.md の §6.9（自然言語→シーン変換指針）の内容をそのまま移動する — 見出し直前に `<a id="s6-9"></a>` アンカーを付与する
  - SKILL.md の §6.10（複数キー単語定義・マルチキー）の内容をそのまま移動する — 見出し直前に `<a id="s6-10"></a>` アンカーを付与する
  - _Requirements: 4_

- [x] 4. SKILL.md を「要約 + リファレンスリンク」形式に再構成する（Task 1・2・3 完了後に実施）
- [x] 4.1 §3.4 Variables の永続化説明を更新しクロスリファレンスリンクを追記する
  - §3.4 の「グローバル変数 `＄＊変数名`: 永続的に有効」を「SAVE テーブル経由でセッション間にわたりファイルに永続化される（JSON 保存）」に更新する
  - §3.4 末尾に `📖` 形式で `references/variables.md` の「永続化と SAVE テーブル」セクションへのリンクを1行追記する
  - _Requirements: 1_
- [x] 4.2 §4 Project Structure の pasta.toml 記述を要約テーブルとリファレンスリンクに置き換える
  - §4 内の pasta.toml インライン記述（サンプル toml コードブロック + キー説明）を削除する
  - design.md §C-SM の §4 設計に従い、8セクションを辞書制作者向け重要度付きで一覧する要約テーブルに置き換える
  - テーブル直後に `📖 全セクション・全キーの詳細: references/pasta-toml.md` へのリンクを追記する
  - セクション見出し `### pasta.toml（ゴースト設定）` は維持する
  - _Requirements: 3_
- [x] 4.3 §6 Authoring Patterns を要約テーブルと代表パターンに再構成する
  - §6 のパターン集本文（305行）を削除する
  - design.md §C-SM の §6 設計に従い、§6.1〜§6.10 を内容と代表ファイルで一覧する要約テーブルを配置する
  - §6.3 ランダムトークの代表コード例（同名シーン `＊OnTalk` を複数定義する最小例）を1つ掲載する
  - `📖 全パターンの詳細・応用例: references/authoring-patterns.md` へのリンクを追記する
  - セクション見出し `## §6 Authoring Patterns（辞書制作パターン集）` は維持する
  - _Requirements: 4_
- [x] 4.4 §3・§5 の §6 内部相互参照を authoring-patterns.md アンカーリンクに更新し、バージョンをバンプする
  - §3.1 内の「§6.6 参照」を `[authoring-patterns.md §6.6](references/authoring-patterns.md#s6-6)` に更新する
  - §3.3 内の「§6.10 参照」を `[authoring-patterns.md §6.10](references/authoring-patterns.md#s6-10)` に、「§6.6 参照」を `[authoring-patterns.md §6.6](references/authoring-patterns.md#s6-6)` に更新する
  - §3.5 内の「§6.7 参照」を `[authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)` に更新する
  - §5 内の「§6.7」を `[authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)` に、「§6.4 参照」を `[authoring-patterns.md §6.4](references/authoring-patterns.md#s6-4)` に更新する（計6箇所）
  - YAML フロントマターの `metadata.version` を `1.4.0` から `1.5.0` に更新する
  - _Requirements: 4, 5_

- [x] 5. 変更内容の整合性と行数目標を検証する
- [x]* 5.1 SKILL.md の総行数とリンク到達可能性を確認する
  - SKILL.md の総行数が 350行以内であることを確認する（設計目標 ~341行）
  - SKILL.md から `references/variables.md`・`references/pasta-toml.md`・`references/authoring-patterns.md` への各リンクが実在するファイルを指していることを確認する
  - `authoring-patterns.md` に `<a id="s6-1"></a>` 〜 `<a id="s6-10"></a>` のアンカーが全て存在することを確認する
  - _Requirements: 4_
- [x]* 5.2 要件ごとの変更内容を照合して矛盾・漏れがないことを確認する
  - Req 1〜5 の受け入れ基準（AC）を1件ずつ確認し、対応する変更が実施されていることを照合する
  - SKILL.md と references/ の間に記述の矛盾がないことを確認する（矛盾があれば references/ を正として SKILL.md を修正する）
  - _Requirements: 1, 2, 3, 4, 5_
