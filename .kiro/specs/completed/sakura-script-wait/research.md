# Research & Design Decisions: sakura-script-wait

---
**Purpose**: 設計フェーズにおける調査結果・アーキテクチャ選定根拠の記録

---

## Summary
- **Feature**: `sakura-script-wait`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  1. 既存`@pasta_*`モジュールパターン（enc, persistence, search）が確立されており、新モジュール追加は低リスク
  2. `PastaConfig`は`custom_fields`経由で任意TOMLセクションを扱う設計。`[talk]`追加は既存パターン踏襲
  3. `regex`クレートをCargo.tomlに追加する必要あり（Rust側でのさくらスクリプトタグ検出用）

## Research Log

### モジュール登録パターン調査
- **Context**: 新規Luaモジュール`@pasta_sakura_script`の登録方法調査
- **Sources Consulted**:
  - `crates/pasta_lua/src/runtime/enc.rs` - `@enc`モジュール実装
  - `crates/pasta_lua/src/runtime/persistence.rs` - `@pasta_persistence`実装
  - `crates/pasta_lua/src/search/mod.rs` - `@pasta_search`実装
- **Findings**:
  - 全モジュールが`register(lua: &Lua) -> LuaResult<Table>`パターンを採用
  - `lua.create_function()`で関数を作成し、Tableに登録
  - `_VERSION`と`_DESCRIPTION`メタデータを含む規約
- **Implications**: 新モジュールも同一パターンで実装可能。配置は`src/sakura_script/`として独立

### 設定管理パターン調査
- **Context**: `[talk]`セクションの追加方法
- **Sources Consulted**:
  - `crates/pasta_lua/src/loader/config.rs` - `PastaConfig`実装
- **Findings**:
  - `PastaConfig`は`loader`専用フィールドと`custom_fields: toml::Table`で構成
  - `logging()`, `persistence()`, `lua()`等のアクセサが`custom_fields.get("section")`パターン
  - 各設定構造体は`serde::Deserialize`と`Default`を実装
- **Implications**: `TalkConfig`構造体を定義し、`PastaConfig::talk()`アクセサを追加

### 正規表現クレート調査
- **Context**: さくらスクリプトタグ検出用正規表現の実装方法
- **Sources Consulted**:
  - `crates/pasta_lua/Cargo.toml` - 現在の依存関係
  - Rust `regex`クレートドキュメント
- **Findings**:
  - 現在`regex`クレートは未使用（`mlua-stdlib`経由でLua側のみ利用可能）
  - パターン`\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?`はRust `regex`で問題なく動作
  - `regex`はUnicode対応済み
- **Implications**: `regex = "1"`をCargo.tomlに追加

## Architecture Pattern Evaluation

| Option            | Description                            | Strengths                        | Risks / Limitations | Notes                |
| ----------------- | -------------------------------------- | -------------------------------- | ------------------- | -------------------- |
| A: runtime/拡張   | `src/runtime/sakura_wait.rs`として追加 | 既存配置に統合                   | runtime/の責務拡大  | シンプルな追加時向け |
| B: 独立モジュール | `src/sakura_script/`として独立         | 明確な責務分離、テスト独立性     | ファイル数増加      | 将来の拡張性重視     |
| C: ハイブリッド   | 独立モジュール + 設定はconfig.rs       | 責務分離と既存パターン維持の両立 | なし                | **推奨**             |

**選択**: Option C（ハイブリッドアプローチ）

## Design Decisions

### Decision: モジュール配置
- **Context**: 新機能のファイル配置決定
- **Alternatives Considered**:
  1. `src/runtime/sakura_wait.rs` - 単一ファイル追加
  2. `src/sakura_script/` - 独立ディレクトリ
- **Selected Approach**: `src/sakura_script/`として独立ディレクトリ
- **Rationale**: 
  - トークナイザーとウェイト挿入ロジックの分離が可能
  - 将来の拡張（`validate_tags()`, `parse_tags()`等）に対応
  - テストファイルの独立配置が可能
- **Trade-offs**: ファイル数増加だが、保守性向上
- **Follow-up**: 設定アクセサは`loader/config.rs`に追加

### Decision: トークナイザー実装方式
- **Context**: 入力文字列を文字種別ごとに分解する方法
- **Alternatives Considered**:
  1. 正規表現ベース - 全パターンを正規表現でマッチ
  2. 文字イテレータ - `chars()`で1文字ずつ判定
  3. ハイブリッド - さくらスクリプトは正規表現、他は文字判定
- **Selected Approach**: ハイブリッド方式
- **Rationale**:
  - さくらスクリプトタグは可変長パターン（`\h`, `\s[0]`, `\_w[100]`等）
  - 文字種別判定は単純な文字セット照合で十分
  - 正規表現は`\`で始まるタグ検出時のみ使用
- **Trade-offs**: 実装複雑度は中程度だが、パフォーマンスと正確性のバランス良好

### Decision: ウェイト値フォールバック戦略
- **Context**: actor設定 → pasta.toml → ハードコードデフォルトの優先順位
- **Selected Approach**: 3段階フォールバック
  1. `actor.script_wait_*` (Luaテーブル)
  2. `pasta.toml [talk].script_wait_*`
  3. ハードコードデフォルト値
- **Rationale**: キャラクター固有設定 > ゴースト全体設定 > システムデフォルト

### Decision: モジュール初期化タイミング
- **Context**: runtime/mod.rsからの登録タイミングと設定受け渡し方法
- **Selected Approach**: `PastaRuntime::from_loader()`内で`TalkConfig`をOption渡し
- **Rationale**: 
  - `PastaConfig::talk()`はOptionを返すため、`register()`もOption受け入れに対応
  - pasta.toml不在時もハードコードデフォルトで動作可能
  - 既存persistenceモジュール（設定必須）とencモジュール（設定不要）の中間パターン

### Decision: Unicode結合文字の扱い
- **Context**: 絵文字や結合文字（`👨‍👩‍👧‍👦`, `が`=`か`+`゛`）の処理方針
- **Selected Approach**: Rust標準`chars()`イテレータの挙動に従う
- **Rationale**:
  - 日本語会話テキストで絵文字・結合文字は稀
  - grapheme cluster処理は`unicode-segmentation`クレート追加が必要
  - 将来問題報告時に改善検討（現時点ではオーバーエンジニアリング）

### Decision: 正規表現コンパイルタイミング
- **Context**: tokenize()呼び出しごとの正規表現コンパイルコスト削減
- **Selected Approach**: `register()`時点（PastaConfig確定後）にTokenizer構造体を初期化し、Regexコンパイル
- **Rationale**:
  - ローダープロセス完了後、ランタイムで設定変更なし
  - lazy_static/OnceCell不要（初期化タイミングが明確）
  - Tokenizer構造体に`sakura_tag_regex: Regex`と`char_sets: CharSets`を保持
  - Lua関数クロージャ経由でTokenizerインスタンスを渡す

## Risks & Mitigations

| Risk                         | Likelihood | Impact | Mitigation                                           |
| ---------------------------- | ---------- | ------ | ---------------------------------------------------- |
| Unicode文字分類の不備        | Low        | Medium | 文字セットを設定可能にし、ユーザーがカスタマイズ可能 |
| さくらスクリプトタグの見逃し | Low        | High   | 正規表現パターンをテストケースで網羅的に検証         |
| 性能影響（長文処理）         | Low        | Low    | 文字列処理は1パスで完結、O(n)計算量                  |

## References

- [Rust regex crate](https://docs.rs/regex/latest/) - 正規表現ライブラリ
- [mlua documentation](https://docs.rs/mlua/) - Lua 5.5バインディング
- [UKAGAKA さくらスクリプト仕様](http://ssp.shillest.net/ukadoc/manual/list_sakura_script.html) - タグ形式参考
