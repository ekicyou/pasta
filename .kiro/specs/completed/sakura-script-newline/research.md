# Research & Design Decisions: sakura-script-newline

## Summary

- **Feature**: `sakura-script-newline`
- **Discovery Scope**: Extension（既存 `sakura_builder.lua` の改行出力ロジック変更）
- **Key Findings**:
  - per-spot has-text 追跡＋完全遅延 pending は、#21 の `text_since_break` グローバル抑制を**完全に包摂**できる（既存 #21 テスト全ケースで検証済み。Option C ハイブリッド不要、グローバルフラグは削除可能）。
  - SSP の「文字がタイプされるまで改行描画を遅延する」仮説は **ukadoc に記載なし**（`\n` / `\n[パーセント]` はタグ意味論のみ）。設計はこの仮説に依存しない（fully-lazy 出力は描画実装非依存の正規形）。検証は実機 SSP 目視で実施する。
  - 先出し順序を固定しているテストは Lua ユニットに加え、`shiori_act_test.lua`（1件）と Rust `startup_test.rs`（1件: `\n[200]` config 検証）にも存在する。単純 A→B シナリオは遅延方式で改行ゼロになるため、これらは A→B→A 往復へ書き換えてアサーション対象を保持する。

## Research Log

### #21 `text_since_break` グローバル抑制の包摂可否（Research 1）

- **Context**: R5.3 が「グローバル抑制と per-spot 追跡の統合または共存の方式は設計フェーズで確定する」と規定。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`（現行実装）、`crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` の「改行キャンセル」スイート全7ケース。
- **Findings**（新方式 = 切替時に切替先 spot の has-text で pending をセット、非空 talk 直前でフラッシュ、切替/clear_spot/ビルド終端で破棄）:

  | #21 既存テストケース | 旧期待値 | 新方式の出力 | 判定 |
  | --- | --- | --- | --- |
  | surfaceのみ手番→切替（改行なし） | break 0 | 切替先 spot has-text 偽 → pending なし → 0 | ✅ 包摂 |
  | sakura_scriptのみ手番→切替 | break 0 | 同上 → 0 | ✅ 包摂 |
  | 空talk＋surface手番→切替 | break 0 | 同上 → 0 | ✅ 包摂 |
  | A(talk)→B(talk) 回帰ケース | break 1 | B の spot は初登場 → 0 | ⚠️ 期待値反転（意図した変更: R3.1） |
  | 先頭サーフェス手番×2→A→B | break 1（A末尾） | A・B とも各 spot 初テキスト → 0 | ⚠️ 期待値反転（旧出力の `\n[150]` は離脱側ゴミ改行そのもの） |
  | 中間さくらスクリプト手番を挟む往復 | break 1 | 「またね」は spot1 初テキスト → 0（spot0 の pending は surface のみ手番のため破棄） | ⚠️ 期待値反転 |
  | clear_spot でフラグリセット | break 0 | clear_spot で has-text/pending 全リセット → 0 | ✅ 包摂 |

- **包摂の論証**: #21 が改行を抑制するのは「前回改行以降に一般文字列ゼロ」のとき。新方式で `\n[N]` が出るのは「切替先 spot が同一ビルド内で一般文字列出力済み、かつ pending が破棄されずに非空 talk へ到達」したときのみ。切替先 spot に一般文字列があれば直近の手番列に一般文字列が存在するため、#21 が抑制するケース（全さくらスクリプト手番からの切替）で新方式が改行を出すことはない（pending は切替ごとに破棄・再評価されるため、全さくらスクリプト手番を挟んだ場合も R2.3 で破棄される）。逆方向の差分（#21 が出していた改行を新方式が出さない）はすべて「離脱側末尾のゴミ改行」または「初登場スポットへの不要改行」であり、本仕様が意図的に除去するもの（R1.1/R3.1）。
- **Implications**: `text_since_break` は削除し `spot_has_text` テーブル＋`pending_break` フラグへ置換する。**Option C（ハイブリッド共存）は不要**。R5.3 の判断は「完全包摂・置換」で確定。

### SSP 改行遅延描画仮説の検証（Research 2）

- **Context**: brief.md の制約「SSP の改行遅延描画仮説は設計フェーズで検証する」。R6.3（修正前後で SSP 上の見た目不変）の裏付け。
- **Sources Consulted**: ukadoc MCP（`\n`、`\n[パーセント]`、`\n[half]`、`\_n`、バルーン descript `wordwrappoint.x`）、里々 Wiki（`＄スコープ切り換え時`）。
- **Findings**:
  - ukadoc の `\n[パーセント]` は「通常のパーセント分改行する。負の値を指定すると戻る（SSP 2.3.97+）」とタグ意味論のみを規定し、**描画遅延（タイプされるまで改行を描画しない）挙動は文書化されていない**。仮説は文書からは確認も反証もできず「ユーザー観察・未検証」のまま。
  - 里々は `＄スコープ切り換え時 = \n[half]` をスコープ切替時に**自動挿入**する同型の先出し設計であることを Wiki で確認（ゴースト「ポストと狛犬」でデフォルト設定）。里々製ゴーストが SSP 上で末尾ゴミ改行だらけに見えていない事実は、SSP が末尾改行を可視化しない（遅延描画または末尾クリップ）挙動の**状況証拠**となる。
- **Implications**:
  - 設計は仮説の真偽に**依存しない**: fully-lazy 出力は「改行が必要な位置にのみ改行がある」正規形であり、SSP が遅延描画してもしなくても見た目は同一になる。
  - 検証計画（実装フェーズの手動検証項目として設計に組込み）: 実機 SSP で (a) A→B 終了トーク（両バルーンに空行なし）、(b) A→B→A 往復（戻り側段落先頭に約1.5行の区切り＝修正前と同見た目）、(c) 同一スポット話者交代（段落区切りが入る）を目視確認する。自動化はしない（外部ベースウェア依存のため）。

### 先出し順序を固定している既存テストの全数調査

- **Context**: R7.2 の期待値更新対象の確定。ギャップ分析時点の調査を design 向けに precise 化。
- **Sources Consulted**: `crates/pasta_lua/tests/` 配下の grep（`\n[150]` / `\n[200]` / `spot_newlines`）。
- **Findings**（更新必須のアサーション）:
  1. `lua_specs/sakura_builder_test.lua`:
     - 複合シナリオ「複数トークンを正しく連結する」: `\n[150]` 存在確認（A→B）→ 遅延方式では改行ゼロ。
     - 「spot変更時に\n[N]を出力する」（A→B）→ A→B→A へ書き換えて N 算出検証を維持。
     - 改行キャンセル「直前の会話に一般文字列があれば従来どおり改行を出力する（回帰）」: 1→0 反転、または A→B→A 化。
     - 「先頭のサーフェス設定手番では…」: count 1→0、パターン `A\n[150]\p[1]B` → `A\p[1]B`。
     - 「さくらスクリプトのみの手番を挟んでも…」: count 1→0（「またね」は spot1 初テキスト）。
     - 統合シナリオ「set_spot()→talk()→spot切り替え」: `\n[150]\p[1]Kero speaks` → `\p[1]Kero speaks`。
     - persist-spot-position「入力actor_spotsの値を引き継いで…」: `\n[150]` 存在確認 → 削除または A→B→A 化。
     - string-buffer「spot 変更・改行を含む入力でバイト一致する」: `\n[200]` 存在確認 → シナリオを往復化して `\n[200]` カバレッジを保持（バイト一致検証自体は影響なし）。
  2. `lua_specs/shiori_act_test.lua`（行167付近）: `act:talk(sakura)→act:talk(kero)` で `\n[150]` 存在確認 → A→B→A へ書き換え。
  3. `loader/startup_test.rs` `test_shiori_act_uses_config_spot_newlines`: A→B で `\n[200]` を確認（config 値伝搬の検証）→ A→B→A へ書き換えて `\n[200]` の観測を維持。
  4. Rust 統合テスト `tests/sakura_script/*.rs`: 段落区切り順序のアサーションなし（`talk_to_script` のウェイト挿入が対象）。変更不要。
- **Implications**: 期待値更新は Lua 2ファイル＋Rust 1ファイルに閉じる。「改行が消える」方向の反転が多いため、単なる期待値反転ではなく **A→B→A 往復への書き換えでアサーション対象（N 算出・config 伝搬・バイト一致）を保持**する方針を採る。

### 呼び出し側・永続状態の互換性確認

- **Context**: 後方互換の確認（R6.1/R6.2/R6.5）。
- **Sources Consulted**: `pasta_scripts/pasta/shiori/act.lua`（`SHIORI_ACT_IMPL.build`）、`pasta_scripts/pasta/store.lua`。
- **Findings**: 呼び出しは `BUILDER.build(token, { spot_newlines = self._spot_newlines }, STORE.actor_spots)` の1箇所のみ。`spot_newlines` は `CONFIG.get("ghost", "spot_newlines", 1.5)` 由来。バッファは `pasta.buf`（`config.buffer_factory` 注入でフォールバック検証可能）。
- **Implications**: `BUILDER.build` の外部シグネチャ・`STORE.actor_spots` 直接変更方式・バッファ抽象は不変のまま実装可能。呼び出し側変更ゼロ。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| --- | --- | --- | --- | --- |
| A: 既存モジュール拡張（採用） | `sakura_builder.lua` 内で `text_since_break` を `spot_has_text`＋`pending_break` へ置換 | 最小差分、呼び出し側不変、純関数的で完全テスト可能 | 既存テスト期待値の反転が広範 | ファイル174行・単一責務のまま |
| B: 改行判定の別モジュール分離 | pending 状態機械を新モジュールへ抽出 | 責務分離 | 174行のモジュールに過剰分割、状態受け渡し増 | 却下 |
| C: per-spot＋グローバル抑制ハイブリッド | #21 フラグを残し共存 | 移行安全 | 二重抑制ロジックの一貫性リスク | **Research 1 で完全包摂を確認したため不要・却下** |

## Design Decisions

### Decision: #21 グローバル抑制の完全置換（共存させない）

- **Context**: R5.3 の設計判断持ち越し。
- **Alternatives Considered**: 1) 完全置換（`text_since_break` 削除） 2) Option C ハイブリッド共存
- **Selected Approach**: 完全置換。`spot_has_text` と `pending_break` のみで改行判定を行う。
- **Rationale**: Research 1 の全ケース検証で、#21 が抑制する全ケースを新方式も抑制することを確認。共存は二重ロジックの保守リスクのみ増やす。
- **Trade-offs**: 既存 #21 テストのうち3ケースは期待値反転が必要（いずれも本仕様が意図する挙動変更）。
- **Follow-up**: 実装時、反転ケースは「意図した変更」であることをテストコメントに明記する。

### Decision: has-text マップのキーは解決済みスポットID（アクター名ではない）

- **Context**: ギャップ分析の設計判断 (a)。
- **Alternatives Considered**: 1) `actor.name` キー 2) 解決済み spot 番号キー
- **Selected Approach**: `spot_has_text: table<integer, boolean>`、キーは `actor_spots[actor.name]` 解決後（フォールバック 0 適用後）の spot 番号。
- **Rationale**: 段落区切りの意味論は「バルーン（スポット）に既にテキストがあるか」であり、アクター単位ではない。R2.5/R2.6（同一スポット共有アクターの交代で改行）はスポットキーでのみ自然に成立する（アクターキーだと切替先アクターの has-text が偽になり改行が出ない）。
- **Trade-offs**: なし（要件がスポット単位を明示）。

### Decision: pending は単一 boolean（per-spot テーブルにしない）

- **Context**: 保留状態のデータ構造選定。
- **Alternatives Considered**: 1) `pending_break: boolean`（現在スコープ専用） 2) `table<spot, boolean>`
- **Selected Approach**: 単一 boolean。不変条件「pending は常に現在スコープ（`last_spot`）に対する保留である」を維持する。アクター切替のたびに旧 pending は暗黙に破棄され、切替先の has-text で再評価される（R2.3）。
- **Rationale**: 保留がフラッシュされ得るのは現在スコープの一般文字列直前のみ（R1.3/R2.2）。スコープを離脱した保留は要件上必ず破棄されるため、複数スポットの保留が同時に生存することはない。
- **Trade-offs**: なし。テーブル化は死んだ状態を持ち込むだけ。

### Decision: フラッシュ位置は「非空 talk の変換出力の直前」

- **Context**: R1.3「`\p` より後、次の一般文字列の直前」の実装位置。
- **Selected Approach**: inner トークンループ内で、非空 `talk` を `talk_to_script` へ渡す**直前**に pending を判定し `\n[N]` を出力する。`talk_to_script` の変換結果（ウェイトタグ埋込み済み）全体の前に置く。
- **Rationale**: 切替直後の surface/wait 等の非テキストトークンは改行より先に出力される（R1.3）。同一アクター継続グループでの初テキストにも自然に対応する（pending は切替時のみ破棄されるため、同一スコープの後続グループまで生存する）。
- **Trade-offs**: `\n[N]` とテキストの間にウェイトタグが入らない（`\n[150]\_w[...]テキスト` ではなく `\n[150]テキスト...`）。要件と整合。

### Decision: `emit_actor_switch` の責務縮小

- **Context**: 現行 `emit_actor_switch(buffer, actor_spots, last_spot, actor, spot_newlines, allow_break) -> (spot, emitted_break)` は改行判定を内包する。
- **Selected Approach**: スポット解決（フォールバック 0＋warn 維持）と `\p[spot]` 出力のみに縮小し、`(spot)` を返す。pending のセット・フラッシュは `BUILDER.build` のループ側で行う。
- **Rationale**: 改行判定の入力（`spot_has_text`・pending）はビルドループの状態であり、関数へ引き回すより呼び出し側で完結させる方が凝集する。
- **Trade-offs**: 関数名は switch タグ出力に特化した意味へ変わる（doc コメント更新）。

## Risks & Mitigations

- **既存テスト期待値の反転見落とし** — 全数調査（Research Log 第3項）で対象を3ファイルに確定済み。実装は特性化ベースライン（現行スイート green 確認）→ 実装＋期待値更新を小ステップコミットで行う。
- **R6.3（SSP 見た目不変）が自動検証不能** — 実機 SSP 目視の手動検証チェックリスト（Research 2 の a/b/c）を実装フェーズの検証項目として design に明記。出力正規形は仮説非依存のため、目視は最終確認の位置づけ。
- **actor が nil のグループ・初回切替前の talk というエッジ** — 現行実装でも `\p` タグなしで出力される既存挙動。設計では「`last_spot == nil` の間は has-text 追跡・フラッシュとも行わない（pending は切替前に存在し得ない）」と定義し挙動を固定する。

## References

- [ukadoc: \n[パーセント]](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html) — `\n[150]` の意味論（描画遅延の記載なし）
- [里々Wiki: ＄スコープ切り換え時](https://soliton.sub.jp/satori/?%E7%89%B9%E6%AE%8A%E5%A4%89%E6%95%B0) — 里々の先出し `\n[half]` 自動挿入の慣習（状況証拠）
- #21 先行修正（9ac91f82）: `text_since_break` グローバル抑制の導入コミット
- `.kiro/specs/sakura-script-newline/requirements.md` — 本設計の要件（R2.5/R2.6 同一スポット交代、fully-lazy は要件ディスカッションで確定済み）

---

## 付録: ギャップ分析（要件フェーズ実施分・要約）

- 変更対象は `sakura_builder.lua` の `emit_actor_switch()` と `BUILDER.build()` に局所化。呼び出し側（`act.lua`）と永続状態 `STORE.actor_spots` は変更不要。`BUILDER.build()` シグネチャ維持。
- 要件→資産マップ: R1=出力順序変更（Constraint）、R2=per-spot has-text マップ（Missing）、R3=R1の帰結（要新規テスト）、R4=状態ライフサイクル（`text_since_break` 置換）、R5=#21包摂（→本設計で「完全包摂」確定）、R6=回帰防止（SSP目視は外部依存）、R7=テスト整備。
- 工数 S（1〜3日）、リスク Low〜Medium（期待値反転の見落とし・SSP目視・#21包摂可否→本設計で解消）。
- 推奨 Option A を採用（上記 Architecture Pattern Evaluation 参照）。
