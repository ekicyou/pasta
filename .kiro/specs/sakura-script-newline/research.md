# ギャップ分析: sakura-script-newline

要件（requirements.md）と既存コードベースの差分を分析し、設計フェーズの実装戦略を示す。

## 分析サマリ

- 変更対象は単一モジュール `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `emit_actor_switch()` と `BUILDER.build()` に局所化されている。既存の `text_since_break`（グローバル・#21）を per-spot 化する小さな構造変更で要件を満たせる見込み。
- 呼び出し側（`act.lua` の `SHIORI_ACT_IMPL.build`）と永続状態 `STORE.actor_spots` は変更不要。`BUILDER.build()` のシグネチャは維持でき、後方互換は保たれる。
- 現行の「先出し（eager）順序」を仕様として固定しているのは **Lua ユニットテスト `sakura_builder_test.lua` のみ**。Rust 統合テスト `tests/sakura_script/*.rs` は `talk_to_script`（ウェイト挿入）を対象としており、`\p[...]` / `\n[150]` の段落区切り順序を検証していない。よって期待値更新の主戦場は Lua テスト。
- 遅延方式では「初回登場スポットには改行を出さない」ため、**単純な A→B 交替（往復なし）で従来出ていた区切り改行が消える**。これは意図した変更だが、既存 Lua テストの複数ケース（例:「先頭のサーフェス設定手番では改行を出さず、可視発話同士の間だけ改行する」）の期待値を反転させる必要がある。
- 推奨アプローチは **Option A（既存モジュールの拡張）**。ファイル規模は小さく（現状 174 行）、責務も明確で、per-spot 追跡の追加は単一責任を崩さない。

## 現状調査（Current State）

### 主要資産とレイアウト
- 変更中核: `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
  - `emit_actor_switch(buffer, actor_spots, last_spot, actor, spot_newlines, allow_break)`: 切替検出時に `allow_break and last_spot ~= nil and last_spot ~= spot` なら `\n[percent]` を**先出し**し、その後 `spot_to_tag(spot)`（`\p[N]`）を出力する。戻り値は `(spot, emitted_break)`。
  - `BUILDER.build(grouped_tokens, config, input_actor_spots)`: `actor` トークン処理でアクター切替を検出し `emit_actor_switch` を呼ぶ。ローカル変数 `text_since_break`（bool）で「前回改行以降に一般文字列が出たか」をグローバルに追跡し、非空 `talk` で真に、改行出力後に偽にリセット。`clear_spot` で `last_actor/last_spot/text_since_break` をリセット。
- 呼び出し側: `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` `SHIORI_ACT_IMPL.build`
  - `BUILDER.build(token, { spot_newlines = self._spot_newlines }, STORE.actor_spots)` として呼び出す。`spot_newlines` は `CONFIG.get("ghost", "spot_newlines", 1.5)` 由来。
- 永続状態: `crates/pasta_lua/pasta_scripts/pasta/store.lua` `STORE.actor_spots`（アクター名→スポットID）。`persist-spot-position` 仕様が所有。ビルド間で永続、`spot`/`clear_spot` トークンで更新。本仕様が導入する per-spot has-text は**ビルドローカル**でこれとは別物。

### 規約
- Lua ランタイムスクリプトは `pasta_scripts/pasta/**`。バッファは `pasta.buf`（`buffer_factory` 注入でフォールバック経路をテスト可能）。
- 一般文字列の定義は既に確立済み: 非空 `talk` のみ。`surface`/`wait`/`sakura_script`/`newline`/`clear`/`choice`/`choice_timeout`/`raw_script`/`yield` は数えない（`BUILDER.build` 内 `inner.type == "talk" and inner.text ~= "" ` 判定）。
- スポットタグは常に `\p[ID]` 形式（`spot_to_tag`、SSP ukadoc 準拠）。

### テスト配置と実行経路
- Lua ユニット: `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（`lua_test` BDD フレームワーク）。実行は `crates/pasta_lua/tests/lua_unittest_runner.rs` 経由で `cargo test` に統合。
- Rust 統合: `crates/pasta_lua/tests/sakura_script/{basic,output,edge_case,budoux}_test.rs`。段落区切り改行の順序アサーションは**存在しない**（`talk_to_script` のウェイト挿入・設定が対象）。
- 段落区切り改行の順序を固定している既存ケース（要期待値更新）:
  - `SAKURA_BUILDER - 複合シナリオ`「複数トークンを正しく連結する」: `\n[150]` の存在を確認（A→B、往復なし）。
  - `SAKURA_BUILDER - actorトークンのactor切り替え検出`「spot変更時に\n[N]を出力する」。
  - `SAKURA_BUILDER - sakura-script-newline: 改行キャンセル` の各ケース（surfaceのみ・sakura_scriptのみ・空talk・回帰・先頭サーフェス・中間さくらスクリプト・clear_spot）。
  - `SAKURA_BUILDER - 統合シナリオ`「set_spot()→talk()→spot切り替え」: `\n[150]\p[1]Kero speaks` を確認。
  - `persist-spot-position`「入力actor_spotsの値を引き継いで…」: `\n[150]` を確認。
  - `string-buffer` 系のバイト一致テスト（`\n[200]` を含む）: 出力位置が変わるためフォールバック一致は保たれるが、`native:find("\\n%[200%]")` 等の存在確認は位置変更に耐えるか要確認。

## 要件実現性（Feasibility）と要件→資産マップ

| 要件 | 必要な技術要素 | 対応資産 | ギャップ |
| --- | --- | --- | --- |
| R1 遅延出力（`\p` のみ、改行は `\p` の後） | 切替時の出力順序変更 | `emit_actor_switch` | **Constraint**: 改行出力を `spot_to_tag` の前→後へ移動。戻り値/呼び出し規約の微調整。 |
| R2 再登場スコープへ改行 | 切替先スポットの has-text 判定 | `text_since_break`（グローバル） | **Missing**: per-spot has-text マップ（切替先キーで参照）が必要。 |
| R3 終端ゴミ改行排除 | 離脱側に改行を残さない | eager 出力の除去 | **Constraint**: R1 の帰結として自動達成。要新規テスト。 |
| R4 per-spot 状態ライフサイクル | build 単位初期化・`talk` 出力時更新・`clear_spot` リセット | `BUILDER.build` の `text_since_break` 更新点・`clear_spot` 分岐 | **Missing**: グローバルフラグを per-spot テーブルへ置換。スポットキーは `actor_spots[actor.name]` 解決値。 |
| R5 全さくらスクリプト手番の抑制包摂 | #21 挙動の維持 | `text_since_break` | **Unknown**: per-spot 化で #21 を完全包摂できるか、共存が必要かの判断（下記 Research Needed）。 |
| R6 回帰防止（他トークン・spot管理・SSP見た目・空入力・バイト一致） | 既存挙動不変 | `emit_inner_token`・`spot`/`clear_spot` 分岐・`buffer_factory` | **Constraint**: SSP 目視検証（R6.3）は外部依存。 |
| R7 テスト整備 | 特性化先行・期待値更新・新規ケース | `sakura_builder_test.lua` | **Constraint**: 期待値反転を伴う更新（プロジェクト方針: 特性化テスト先行・小ステップ）。 |

### 複雑度シグナル
- アルゴリズム的ロジック（状態機械の分岐変更）。外部統合なし。UI 表現はさくらスクリプト文字列の順序のみ。純粋関数的で完全ユニットテスト可能。

## 実装アプローチ選択肢

### Option A: 既存モジュールの拡張（推奨）
**適用理由**: 機能が `sakura_builder.lua` の既存責務（グループ化トークン→さくらスクリプト変換）に自然に収まり、ファイルは小規模（174 行）。

- 変更点:
  - `BUILDER.build` のローカル `text_since_break`（bool）を `spot_has_text`（`table<spot, boolean>`）へ置換。
  - 非空 `talk` 出力時に `spot_has_text[current_spot] = true`。`clear_spot` で `spot_has_text = {}`（またはエントリ nil クリア）。
  - `emit_actor_switch` を、①先に `spot_to_tag(spot)` を出力し、②切替先スポットの has-text が真なら続けて `\n[percent]` を出力、へ変更。改行の有無判定を「離脱側 last_spot」から「切替先 spot の has-text」へ移す。
- 互換性: `BUILDER.build` の外部シグネチャ・`STORE.actor_spots` 更新規約・他トークン変換は不変。呼び出し側 `act.lua` 変更不要。
- トレードオフ: ✅ 最小差分・既存パターン踏襲・テスト容易。❌ `emit_actor_switch` の意味（改行位置）が反転するため既存テスト期待値の更新が不可避。

### Option B: 新規コンポーネント分離
**適用理由（低）**: 改行判定を別モジュール/オブジェクトへ抽出。

- トレードオフ: ✅ 責務分離。❌ 174 行のモジュールに対し過剰分割。呼び出し規約・状態受け渡しが増え、小さな順序変更に見合わない。現時点では非推奨。

### Option C: ハイブリッド
**適用理由（低〜中）**: R5 の #21 包摂が per-spot だけで成立しない場合に限り、per-spot has-text に加え補助的なグローバル抑制を一時的に共存させる段階的移行。

- トレードオフ: ✅ 移行安全。❌ 二重の抑制ロジックは一貫性リスク。設計で per-spot 単独成立を検証できれば不要。

## 工数・リスク

- **Effort: S（1〜3 日）** — 単一モジュールの局所変更＋テスト期待値更新＋新規ケース。既存パターン・依存最小。
- **Risk: Low〜Medium** — ロジック自体は低リスク（純粋関数・完全テスト可能）。Medium 要因は (1) 既存テスト期待値の反転が広範で見落としリスク、(2) R6.3 の「SSP 上の見た目不変」が外部ベースウェア描画依存で自動検証しづらい点、(3) R5 の #21 完全包摂可否が未確定。

## Research Needed（設計フェーズへ持ち越し）

1. **#21 包摂可否**: per-spot has-text 追跡が `text_since_break` グローバル抑制を完全に置換できるか、それとも共存（Option C）が必要か。既存 #21 テスト群を反例に検証する。
2. **SSP 改行遅延描画仮説の検証**: 「SSP は実際に文字がタイプされるまで改行描画を遅延する」という前提（ユーザー観察・未検証）を ukadoc / 実機 SSP で確認し、eager→lazy でも A→B→A の見た目が不変であること（R6.3）を裏付ける。
3. **改行の挿入位置（決定済み）**: 要件ディスカッション議題1で **完全遅延（fully-lazy）** を採用。`\n[N]` は `\p[spot]` 直後ではなく、当該スポットで次に出力される一般文字列の直前でフラッシュする（requirements.md R1.3・R2.2 で確定）。設計では保留（pending）フラグの状態遷移（切替でセット/破棄・一般文字列でフラッシュ）を実装する。
4. **再登場だが可視文字ゼロの手番（決定済み）**: 完全遅延の採用により解決。has-text 済みスポットへ戻る手番が surface のみで終端する場合、保留した改行はフラッシュされず破棄されるため、ゴミ改行は生じない（requirements.md R2.3・R3.3 で確定）。
5. **同一スポットを共有する複数アクター（決定済み）**: 要件ディスカッション議題2で **改行を入れる**（option b）を採用。複数キャラが同一位置で入れ代わり会話するシチュエーションは実在し、同一バルーン内でも話者交代を段落区切りする。先出し版の `last_spot == spot` 抑制ガードは**復活させず**、改行条件は「アクター切替」＋「切替先スポットの has-text」のみで決まる（requirements.md R2.5/R2.6・R6.6 で確定）。
   - **派生課題（別仕様へ分離）**: 同一スポットでアクターが変わる際、切替先アクターの立ち絵（サーフェスID・着せ替え状態）の復旧が必要。本仕様（段落区切り改行）のスコープ外とし、別仕様 `actor-surface-restore`（brief.md 作成済み・未着手）へ申し送り。ユーザー決定（要件ディスカッション議題2）。

## 設計フェーズへの推奨

- **推奨アプローチ**: Option A（既存 `sakura_builder.lua` 拡張）。per-spot has-text マップ導入 + `emit_actor_switch` の改行位置反転。
- **主要な設計判断**: (a) has-text のキー（`actor.name` か解決済み spot 番号か）、(b) `\n[N]` 挿入位置の微細仕様（Research 3/4）、(c) #21 完全包摂 vs 共存（Research 1）。
- **持ち越し検証**: SSP 見た目不変（Research 2）と #21 包摂（Research 1）を設計の検証項目に組み込む。プロジェクト方針に従い特性化テスト先行・1 抽出=1 検証=1 コミットの可逆な小ステップで実装する。
