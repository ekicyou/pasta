# Research & Design Decisions: ghost-authoring-skill-restructure

## Summary
- **Feature**: `ghost-authoring-skill-restructure`
- **Discovery Scope**: Extension（既存スキルドキュメントの再構成）
- **Key Findings**:
  - pasta.toml は全8セクション・約33キーを持ち、SKILL.md にはわずか6キーのみ記載（カバレッジ20%）
  - SKILL.md §6 は305行（10サブセクション: 6.1〜6.10）。内部相互参照が6箇所存在し、分離時にリンク維持が必要
  - `pasta-lua-coding` スキルに `@pasta_persistence`・SAVE モジュール・`@pasta_config` の詳細ドキュメントが既存。クロスリファレンスで重複回避可能

---

## Research Log

### pasta.toml 全キーインベントリ

- **Context**: Req 3 で全セクション・全キーを網羅するリファレンス新設が必要。権威的ソースコードから実在するキーを完全に洗い出す。
- **Sources Consulted**:
  - `crates/pasta_lua/src/loader/config.rs` — Rust 設定構造体（型付き解析）
  - `crates/pasta_lua/pasta_scripts/pasta/config.lua` — Lua 側 `@pasta_config` アクセス
  - `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` — `CONFIG.get()` 呼び出し箇所
  - `crates/pasta_lua/tests/fixtures/loader/with_actor_config/pasta.toml` — テストフィクスチャ
  - `crates/pasta_lua/README.md` — ドキュメント内サンプル

- **Findings**:

  **エンジン正式解析セクション（5種、Rust 構造体で型検証）**:

  | セクション | キー | 型 | 既定値 |
  |-----------|------|-----|--------|
  | `[loader]` | `pasta_patterns` | `string[]` | `["dic/*/*.pasta"]` |
  | `[loader]` | `lua_search_paths` | `string[]` | 5パス（下記参照） |
  | `[loader]` | `transpiled_output_dir` | string | `"profile/pasta/cache/lua"` |
  | `[loader]` | `debug_mode` | bool | `true` |
  | `[logging]` | `file_path` | string | `"profile/pasta/logs/pasta.log"` |
  | `[logging]` | `rotation_days` | integer | `7` |
  | `[logging]` | `level` | string | `"info"` |
  | `[logging]` | `filter` | string\|nil | `nil` |
  | `[persistence]` | `obfuscate` | bool | `false` |
  | `[persistence]` | `file_path` | string | `"profile/pasta/save/save.json"` |
  | `[persistence]` | `debug_mode` | bool | `false` |
  | `[lua]` | `libs` | `string[]` | `["std_all","assertions","testing","regex","json","yaml"]` |
  | `[talk]` | `script_wait_normal` | integer | `50` |
  | `[talk]` | `script_wait_period` | integer | `1000` |
  | `[talk]` | `script_wait_comma` | integer | `500` |
  | `[talk]` | `script_wait_strong` | integer | `500` |
  | `[talk]` | `script_wait_leader` | integer | `200` |
  | `[talk]` | `chars_period` | string | `"｡。．."` |
  | `[talk]` | `chars_comma` | string | `"、，,"` |
  | `[talk]` | `chars_strong` | string | `"？！!?"` |
  | `[talk]` | `chars_leader` | string | `"･・‥…"` |
  | `[talk]` | `chars_line_start_prohibited` | string | 行頭禁則文字列 |
  | `[talk]` | `chars_line_end_prohibited` | string | 行末禁則文字列 |

  **カスタムフィールドセクション（3種、`custom_fields` として Lua に透過）**:

  | セクション | キー | 型 | 既定値 | 備考 |
  |-----------|------|-----|--------|------|
  | `[package]` | `name` | string | *(必須)* | パッケージ名 |
  | `[package]` | `version` | string | *(必須)* | セマンティックバージョン |
  | `[package]` | `edition` | string | *(必須)* | エディション |
  | `[ghost]` | `talk_interval_min` | integer | `180` | ランダムトーク最小間隔（秒） |
  | `[ghost]` | `talk_interval_max` | integer | `300` | ランダムトーク最大間隔（秒） |
  | `[ghost]` | `hour_margin` | integer | `30` | OnHour マージン（秒） |
  | `[ghost]` | `spot_newlines` | number | `1.5` | スポット切替時の改行量 |
  | `[actor."名前"]` | `spot` | integer | *(なし)* | バルーン位置 (0=sakura, 1=kero) |
  | `[actor."名前"]` | `budoux` | `integer[]` | *(なし)* | 自動改行幅 (例: `[10, 12]`) |
  | `[actor."名前"]` | `default_surface` | integer | *(なし)* | デフォルトサーフェスID |

  **合計**: 33キー（エンジン解析23キー + カスタムフィールド10キー）

  **`[loader].lua_search_paths` 既定値**:
  ```
  profile/pasta/save/lua     — ユーザー保存スクリプト（最優先）
  scripts                    — ユーザーカスタムスクリプト
  pasta_scripts              — 標準ランタイム
  profile/pasta/cache/lua    — トランスパイルキャッシュ
  scriptlibs                 — 追加ライブラリ
  ```

- **Implications**:
  - `[loader]` のみ `custom_fields` に含まれない（Rust 専用）。他のセクションは全て `@pasta_config` 経由で Lua からアクセス可能
  - `[ghost]` セクションのキー (`talk_interval_min/max`, `hour_margin`, `spot_newlines`) はカスタムフィールドだが、Lua ランタイム内で `CONFIG.get()` により既定値フォールバック付きで取得される
  - 辞書制作者にとって重要度が高いのは `[ghost]`、`[actor."名前"]`、`[loader].pasta_patterns`。`[talk]` は上級者向けカスタマイズ

### SKILL.md §6 構造分析

- **Context**: Req 4 で §6 パターン集を `references/authoring-patterns.md` に分離する必要がある。
- **Sources Consulted**: SKILL.md 全文（~620行）
- **Findings**:

  **§6 行範囲**: 318行〜622行（305行）

  | サブセクション | 行数 | タイトル |
  |--------------|------|---------|
  | §6.1 | ~17行 | アクター辞書定義（actors.pasta） |
  | §6.2 | ~21行 | イベントハンドラ（boot.pasta） |
  | §6.3 | ~17行 | ランダムトーク（talk.pasta） |
  | §6.4 | ~91行 | 時報（hour.pasta / talk.pasta） — 最大セクション |
  | §6.5 | ~9行 | クリック反応（click.pasta） |
  | §6.6 | ~35行 | 単語ランダム選択（シャッフル＆順次消費方式） |
  | §6.7 | ~32行 | 継続トーク（チェイントーク） |
  | §6.8 | ~11行 | ファイル分割ガイド |
  | §6.9 | ~21行 | 自然言語→シーン変換指針 |
  | §6.10 | ~48行 | 複数キー単語定義（マルチキー） |

  **SKILL.md 内の §6 相互参照（6箇所）**:

  | 参照場所 | 参照先 | コンテキスト |
  |---------|--------|-------------|
  | §3.1（シーン定義） | §6.6 | 「シャッフル＆順次消費方式で選択される（詳細は §6.6 参照）」 |
  | §3.3（単語定義） | §6.10 | 「複数キー…（詳細は §6.10 参照）」 |
  | §3.3（単語定義） | §6.6 | 「シャッフル＆順次消費方式で1つ選択される（詳細は §6.6 参照）」 |
  | §3.5（Call文） | §6.7 | 「§6.7 参照」 |
  | §5（イベントマッピング） | §6.7 | 「継続トーク（§6.7）対応」 |
  | §5（イベントマッピング） | §6.4 | 「§6.4 参照」 |

  **外部参照（references/ からの §6 参照）**: なし

- **Implications**:
  - §6 を `references/authoring-patterns.md` に分離しても、6箇所の内部参照リンクを更新すれば外部参照の破壊はない
  - 参照先は `§6.4`, `§6.6`, `§6.7`, `§6.10` の4セクションに集中
  - 分離後の SKILL.md §6 は「要約 + リファレンスリンク」形式に置き換える。相互参照は「📖 [authoring-patterns.md §6.6](references/authoring-patterns.md#66-...)」形式に更新

### pasta-lua-coding クロスリファレンスターゲット

- **Context**: Req 1 AC3、Req 2 で pasta-lua-coding スキルへのクロスリファレンスが必要。既存ドキュメントの所在を確認。
- **Sources Consulted**:
  - `.agents/skills/pasta-lua-coding/references/runtime-api.md`
  - `.agents/skills/pasta-lua-coding/references/internal-modules.md`
- **Findings**:

  | 概念 | ドキュメント所在 | セクション |
  |------|-----------------|-----------|
  | `@pasta_persistence` | `runtime-api.md` | `## @pasta_persistence` |
  | SAVE モジュール | `internal-modules.md` | `## SAVE モジュール` |
  | SAVE キー命名規約 | `internal-modules.md` | `### キー命名規約` |
  | `@pasta_config` | `runtime-api.md` | `## @pasta_config` |

- **Implications**: 永続化メカニズムの詳細は pasta-lua-coding に完備。pasta-ghost-authoring では辞書制作者視点の要約のみ記述し、「詳細は `pasta-lua-coding` スキルを参照」で重複を回避

---

## Design Decisions

### Decision: §6 分離時のセクション番号維持方式

- **Context**: Req 4 AC3 で §1〜§6 のセクション番号体系を維持する必要がある。§6 のパターン集を丸ごと `references/authoring-patterns.md` に分離する場合、SKILL.md の §6 見出しとサブセクション番号 (§6.1〜§6.10) をどう扱うか。
- **Alternatives Considered**:
  1. SKILL.md §6 は見出しのみ残し、全サブセクションを分離先に移動
  2. SKILL.md §6 に各サブセクションの1行要約を残す
- **Selected Approach**: (2) SKILL.md §6 に要約テーブル + 代表パターン1つを残す
- **Rationale**: 要約テーブルにより LLM が §6 の内容を即座に把握でき、詳細が必要な場合のみリファレンスを読み込む2段階構成が最適
- **Trade-offs**: SKILL.md の行数は純粋な見出しだけ残すより多くなるが、LLM のコンテキスト効率は向上する

### Decision: DJ-2 — BudouX の記載カテゴリ

- **Context**: BudouX は `[actor."名前"].budoux` で設定。記載場所を決定する必要がある。
- **Selected Approach**: `references/pasta-toml.md` の `[actor."名前"]` セクションに記載（Req 3 AC4 の要件通り）
- **Rationale**: BudouX はさくらスクリプト処理の一部だが、設定は actor テーブル経由であり、pasta-toml.md が設定リファレンスとして適切。`sakura-script.md` への言及は不要（設定と効果の場所を分散させない）

### Decision: DJ-3 — `[lua]` と `[package]` の記載深度

- **Context**: `[lua].libs` と `[package]` は辞書制作者には直接関係しない（エンジン開発者向け）。
- **Selected Approach**: 「上級者向け」として簡潔に記載し、詳細は pasta-lua-coding に委譲
- **Rationale**: 網羅性と対象読者のバランス。辞書制作者が誤って変更しないよう注意書き付きで簡潔に記載

### Decision: DJ-4 — pasta-toml.md の2階層構造の表現

- **Context**: pasta.toml は2階層（エンジン正式解析 vs カスタムフィールド）に分かれる。
- **Selected Approach**: フラットに全セクションを列挙し、備考欄で解析方式の違いを注記
- **Rationale**: 辞書制作者にとって実用上の差異は小さいが、「カスタムフィールドは自由にキーを追加できる」という情報は有用。2階層の内部構造を前面に出すと混乱を招く

---

## Risks & Mitigations

- **Risk 1**: §6 分離後の相互参照リンク切れ — 6箇所の内部参照を更新リストとして管理し、実装タスクで漏れなく検証
- **Risk 2**: SKILL.md 350行目標の未達 — §6 が305行占有しているため、分離により十分な削減が見込める。§4 の pasta.toml 圧縮（~20行削減）も加算
- **Risk 3**: references/ ファイル増加による LLM コンテキスト膨張 — SKILL.md の「要約 + リンク」形式により、LLM は必要な references/ のみ読み込む設計で回避

## References

- `crates/pasta_lua/src/loader/config.rs` — pasta.toml 設定構造体の権威的ソース
- `crates/pasta_lua/pasta_scripts/pasta/config.lua` — `@pasta_config` Lua モジュール実装
- `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` — ACT オブジェクト初期化・config 取得
- `.agents/skills/pasta-lua-coding/references/runtime-api.md` — `@pasta_persistence`, `@pasta_config` ドキュメント
- `.agents/skills/pasta-lua-coding/references/internal-modules.md` — SAVE モジュール・キー命名規約
