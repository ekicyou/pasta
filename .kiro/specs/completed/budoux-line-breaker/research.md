# Research & Design Decisions: budoux-line-breaker

## Summary
- **Feature**: `budoux-line-breaker`
- **Discovery Scope**: Extension（既存 `sakura_script` モジュールへの機能追加）
- **Key Findings**:
  - `budoux::parse(&model, &text) -> Vec<String>` は平文ワード列を返す。日本語モデルは `budoux::models::default_japanese_model()` で取得
  - 既存 `Tokenizer::SAKURA_TAG_PATTERN` 正規表現をそのまま再利用してタグ分離が可能
  - TOML パススルーにより `[actor."名前"].budoux = [10, 12]` 設定は追加コード不要で `CONFIG.actor` に自動伝搬済み

## Research Log

### budoux クレート API 調査

- **Context**: 要件 3（日本語分割）の実現に必要な外部 API 確認
- **Sources Consulted**: [docs.rs/budoux/0.1.1](https://docs.rs/budoux/0.1.1/budoux/)
- **Findings**:
  - `budoux::parse(model: &Model, input: &str) -> Vec<String>` — 分割結果をワード列で返す
  - `budoux::models::default_japanese_model() -> Model` — 日本語モデル（`HashMap<String, i32>`）
  - `Model` 型は `HashMap<String, i32>` のエイリアス。モデルデータはコンパイル時に静的生成されるため、ランタイム I/O 不要
  - ライセンス: Apache-2.0（プロジェクトの MIT/Apache-2.0 デュアルライセンスと互換）
  - 最新バージョン: 0.1.1（安定、更新頻度低＝API安定の証左）
- **Implications**:
  - モデルは `&Model` 参照で渡せるため、`Arc` 経由で共有可能。初回モジュール登録時に一度だけ生成し `SakuraScriptState` に保持する設計が自然
  - `parse()` は O(n) の線形時間処理。トーク文字列の一般的長さ（数百文字）では性能問題なし

### unicode-width クレート API 調査

- **Context**: 要件 4（CJK 文字幅計算）の実現に必要な外部 API 確認
- **Sources Consulted**: [docs.rs/unicode-width/0.2.2](https://docs.rs/unicode-width/0.2.2/unicode_width/)
- **Findings**:
  - `UnicodeWidthChar::width_cjk(self) -> Option<usize>` — CJK 互換幅を返す
  - `UnicodeWidthStr::width_cjk(&self) -> usize` — 文字列全体の CJK 幅を返す
  - CJK 全角文字は幅 2、ASCII は幅 1 を返す
  - ライセンス: MIT/Apache-2.0（完全互換）
  - 最新バージョン: 0.2.2（安定）
- **Implications**:
  - `str::width_cjk()` を使えばワード単位でまとめて幅計算できる。文字単位のループは不要
  - budoux 分割後のワード列に対して逐次幅加算し、閾値超過時に改行を挿入する設計が適切

### さくらスクリプトタグ分離アルゴリズム調査

- **Context**: 要件 2（透過処理）の核心アルゴリズム選定
- **Sources Consulted**: 既存コードベース `tokenizer.rs`, `wait_inserter.rs`
- **Findings**:
  - 既存 `Tokenizer::SAKURA_TAG_PATTERN` = `r"\\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?"` はタグ検出に十分
  - 既存 `Tokenizer::tokenize()` はウェイト挿入用の文字分類を含むため、budoux 用途にはオーバースペック
  - タグ分離には正規表現の `find_iter` / `split` で十分。`Tokenizer` インスタンスではなくパターン定数を再利用するのが最小差分
  - `wait_inserter` が挿入する `\_w[N]` タグ自体も `SAKURA_TAG_PATTERN` にマッチする → budoux 処理はウェイト挿入**後**に安全に適用可能
- **Implications**:
  - 正規表現 `find_iter` でタグ位置を走査し、平文を1文字ずつ `PlainChar { ch, trailing }` に分解する方式を採用（Design Decisions 参照）
  - 既存 `Tokenizer::tokenize()` はウェイト挿入向けの文字分類を含みオーバースペックなため、パターン定数のみ再利用

### Lua 統合ポイント調査

- **Context**: 要件 6/7 のパイプライン統合方式決定
- **Sources Consulted**: `sakura_builder.lua`, `mod.rs`, `module_registry.rs`
- **Findings**:
  - `sakura_builder.lua` の `BUILDER.build()` 内で `SAKURA_SCRIPT.talk_to_script(actor, inner.text)` を呼出
  - budoux 呼出を追加する場合、`talk_to_script` の結果に対して `SAKURA_SCRIPT.break_lines(result, actor.budoux)` を後続呼出するのが最小変更
  - `actor.budoux` は TOML パススルーにより配列テーブル（Lua table）として自動伝搬済み
  - `SakuraScriptState` に budoux モデルを追加すれば、`Arc` 共有パターンで Lua 関数クロージャからアクセス可能
- **Implications**:
  - Lua 側変更は `sakura_builder.lua` の 2 箇所（`talk` / `sakura_script` type）のみ
  - Rust 側変更は `mod.rs` の `register()` に `break_lines` 関数登録 + `line_breaker.rs` 新規ファイル

### テスト戦略調査

- **Context**: 要件 8 のテスト配置方式決定
- **Sources Consulted**: `tests/sakura_script/main.rs`, `tests/common/mod.rs`
- **Findings**:
  - `tests/sakura_script/main.rs` に `mod basic_test; mod output_test;` で統合テスト管理
  - `common::create_sakura_test_runtime()` で `@pasta_sakura_script` モジュール登録済みの Lua ランタイム生成
  - Rust 純粋関数の単体テストは `line_breaker.rs` 内部の `#[cfg(test)] mod tests` に配置
  - 統合テスト（Lua 経由呼出）は `tests/sakura_script/` 配下に `budoux_test.rs` を追加
- **Implications**:
  - 既存テストパターンに完全に準拠可能。`main.rs` に `mod budoux_test;` を1行追加するだけ

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存モジュール拡張 | `@pasta_sakura_script` に `break_lines` 追加 | 最小差分、Tokenizer 直接利用、パターン踏襲 | モジュール責務増加 | 推奨候補 |
| B: 独立モジュール | `@pasta_budoux` 新設 | 単一責任原則 | 新モジュール登録、可視性調整、Lua require 追加 | 過剰設計 |
| C: ハイブリッド | ロジックは独立ファイル、公開は既存モジュール | A のメリット + ファイル分離 | モジュール関数増加（軽微） | **採用** |

**決定**: Option C（ハイブリッド）。ロジックを `sakura_script/line_breaker.rs` に独立ファイルとして実装し、Lua 公開は `@pasta_sakura_script` モジュールへの関数追加とする。

## Design Decisions

### Decision: タグ位置マッピング方式

- **Context**: さくらスクリプトタグを含む文字列から平文を抽出し、budoux 分割後にタグを復元する必要がある
- **Alternatives Considered**:
  1. バイトオフセットマッピング — 正規表現でタグ位置を記録、平文を連結して budoux へ渡し、オフセットで改行位置を元文字列にマッピング
  2. トークン列ベース — 既存 `Tokenizer::tokenize()` でトークン化し、SakuraScript 以外を連結して budoux へ
- **Selected Approach**: PlainChar トークン構造によるタグ分離（方式 c）
- **Rationale**:
  - 入力を `PlainChar { ch: char, trailing: &str }` の列に分解し、各平文文字に直後のタグ群を紐付ける
  - 改行挿入が構造的に「trailing の後、次の PlainChar の前」になるため、ウェイトタグの所属が自明
  - budoux 分割境界は文字単位で、PlainChar の char index と直接対応するためマッピングが単純
  - `&'a str` 参照のみでゼロアロケーション（出力 String 確保のみ）
  - 方式 (a) バイトオフセットマッピングの利点を維持しつつ、タグ境界での改行挿入順序問題を構造的に解消
  - 処理が直線的で理解しやすく、テストも容易
- **Trade-offs**: 正規表現コンパイルが必要だが、`Regex` は `SakuraScriptState` に保持済みなので追加コスト0
- **Follow-up**: ウェイトタグ（`\_w[50]`）がワード境界を跨ぐケースのテストケースを要作成

### Decision: budoux モデル保持方式

- **Context**: budoux 日本語モデルの初期化タイミングと保持場所
- **Alternatives Considered**:
  1. 関数呼出時に毎回 `default_japanese_model()` を呼ぶ
  2. `SakuraScriptState` に保持して `Arc` 共有
- **Selected Approach**: `SakuraScriptState` に保持
- **Rationale**:
  - モデルは `HashMap<String, i32>` であり、初期化コストが発生する
  - 既存 `Arc<SakuraScriptState>` パターンに追加するだけで対応可能
  - モジュール登録時に一度だけ初期化し、全 `break_lines` 呼出で共有
- **Trade-offs**: `SakuraScriptState` のメモリ消費がモデル分だけ増加するが、モデルサイズは数十KBで無視できる

### Decision: Lua 側統合ポイント

- **Context**: budoux 処理をパイプラインのどこに配置するか
- **Alternatives Considered**:
  1. `talk_to_script()` 内部に組み込む — ウェイト挿入と改行挿入を一体化
  2. `sakura_builder.lua` で外部呼出 — `talk_to_script` の結果に対して後続適用
- **Selected Approach**: `sakura_builder.lua` での外部呼出（方式 2）
- **Rationale**:
  - `talk_to_script` は汎用的なウェイト挿入関数であり、budoux 依存を持ち込むべきでない
  - budoux 適用の有無はアクター設定（`actor.budoux`）に依存し、`sakura_builder.lua` が actor 情報を持っている
  - パイプライン順序が明示的: `talk_to_script` → `break_lines` → buffer 蓄積
  - `break_lines` は独立関数であり、将来的に `talk_to_script` 以外のコンテキストでも利用可能
- **Trade-offs**: Lua 側の呼出コードが2行増加するが、パイプラインの透明性が向上

### Decision: 改行候補の貪欲法による閾値チェック

- **Context**: budoux 分割ワード列から実際の改行位置を決定するアルゴリズム
- **Alternatives Considered**:
  1. 動的計画法（最適改行位置）
  2. 貪欲法（先頭からワードを積み、閾値超過時に改行）
- **Selected Approach**: 貪欲法
- **Rationale**:
  - トーク文字列は通常数十〜数百文字で、最適化の恩恵が小さい
  - 人間の読み感覚に近い「できるだけ詰めて改行」が自然
  - 実装が単純で検証容易
  - 幅閾値スライス（行ごとに異なる幅）との相性が良い
- **Trade-offs**: 局所最適であり、文末が極端に短い行になる可能性があるが、budoux の分割が自然な区切りを保証するため実用上問題なし

## Risks & Mitigations

- **budoux 分割がウェイトタグで不自然になるリスク** → budoux には平文（タグ除去済み）のみ渡すため、分割精度への影響なし
- **unicode-width の CJK 幅が環境依存になるリスク** → `width_cjk()` は Unicode Annex #11 準拠の固定テーブルであり、環境非依存
- **既存テスト破壊のリスク** → `break_lines` は新規追加関数であり、既存 `talk_to_script` に変更なし。Lua 側変更も条件付き呼出（`actor.budoux` 存在チェック）のため、budoux 未設定の既存ゴーストに影響なし

## References

- [budoux 0.1.1 API ドキュメント](https://docs.rs/budoux/0.1.1/budoux/) — `parse()`, `models::default_japanese_model()`
- [unicode-width 0.2.2 API ドキュメント](https://docs.rs/unicode-width/0.2.2/unicode_width/) — `UnicodeWidthStr::width_cjk()`
- [BudouX 本家 (Google)](https://github.com/google/budoux) — 機械学習モデルの原理
- [Unicode Annex #11](https://www.unicode.org/reports/tr11/) — East Asian Width 定義
