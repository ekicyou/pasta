# Implementation Plan

> 凡例: `(P)` = 直前の同位タスクと並列実行可能。`_Boundary:_` = 主担当コンポーネント。`_Depends:_` = 順序だけでは表せない依存。

- [x] 1. Foundation: シーン同一性索引の基盤データ構造
- [x] 1.1 シーン同一性索引の型と位置→シーン解決ロジックを実装する
  - `.pasta` の (ファイル, 行範囲) → シーン識別子 の対応を保持するインメモリ索引を用意する（行範囲は「宣言行〜次の同レベル以上のシーン宣言の直前」と定義）
  - global ⊃ local の入れ子を表現し、包含が複数該当する場合は最内（local 優先）を選ぶ
  - 包含が無い場合はクリック行と同じか下方の最近接有効シーンを選ぶ（後方フォールバック）、下方に無ければ未検出を返す
  - lookup は行範囲を昇順保持し O(log n) 級で引けること（GET ブロックを延ばさない）
  - 単体テストで「包含1件／最内local選択／後方フォールバック／最終シーン後は未検出／空ファイルは未検出」を観測できる
  - _Requirements: 2.1, 2.2, 2.5, 2.6, 3.2_
  - _Boundary: SceneIdentityIndex_

- [x] 1.2 (P) ソースマップ sink にシーン記録口をデフォルト no-op で追加する
  - `SourceMapSink` に「突合キー＋span を記録する」メソッドをデフォルト実装 no-op で追加し、既存実装を後方非破壊にする
  - 既存の行マッピング記録（`record_line` / `record`）の内容・順序を一切変更しない
  - 突合キーは code_gen と finalize の双方で決定的に再現できる値（base 名＋出現順）とし、連番付き最終識別子は記録しない
  - 既存 sink 実装が新メソッド未実装でもコンパイル・動作することをテストで観測できる
  - _Requirements: 3.1, 3.4_
  - _Boundary: SourceMapSink_

- [x] 2. Core: ビルド側の索引構築（code_gen 記録 → finalize 突合）
- [x] 2.1 code_gen のシーン生成箇所でシーン記録を呼ぶ
  - global/local シーン生成箇所（既存 span 記録の近傍）で、突合キー（base 名＋出現順）と `.pasta` span を sink へ流す
  - シーン範囲終端（次の同レベル以上宣言の直前／chunk 末尾）と入れ子レベルを受け渡す
  - 連番のサニタイズ規則を code_gen 側で再実装しない（形式ドリフト回避）
  - transpile 後、各シーンの (突合キー, 行範囲) が sink に蓄積されていることをテストで観測できる
  - _Requirements: 3.1, 3.3_
  - _Depends: 1.2_
  - _Boundary: code_gen scope_gen_

- [x] 2.2 finalize でランタイム実識別子と span を突合し索引を構築する
  - finalize がランタイム実シーン識別子を列挙する箇所（`collect_scenes` が `(global_name, local_name)` を返す・`会話1`/`挨拶_1` 形式）で、同一の突合キーで span 側と突合し、(ファイル, 行範囲) → (scene_id, parent) の索引を確定する
  - 各シーンを **(scene_id, parent)** で保持する：global → `(会話1, None)`、local → `(挨拶_1, 会話1)`。task 1.1 の `SceneIdentityIndex`/`SceneSpan` を **parent: Option<String> を持つよう拡張**し、`scene_at` が最内（level 最大）の (scene_id, parent) を返せること
  - join：join_key の親参照 `会話#1`→runtime `会話1` を (base, 出現順) で対応付け、local `L:会話#1:挨拶_1`→`(parent=会話1, fn_name=挨拶_1)` を `collect_scenes` の `(会話1, 挨拶_1)` と突合（Implementation Notes「local シーン kick の方式決定」「2.x join_key 契約」参照）
  - chunk ごとの索引を集約してロード済みソースマップ（実行中エンジンが保持する読み取り専用状態）へ同梱する
  - 索引が返す識別子値が、kick のシーン解決の解決対象と一致すること（global は `会話1`、local は parent 付きで local 分岐解決＝`SceneRegistry` の別形式を SSOT にしない）
  - `.pasta` を transpile→ロードした後、既知のシーン宣言行（global／local 双方）から引いた (scene_id, parent) がランタイム実シーンに一致することをテストで観測できる
  - **通常モード非破壊**：sink 未接続の本番トランスパイルでは索引を構築せず、生成バイト・行マッピング挙動が不変であることを確認
  - _Requirements: 3.1, 3.2, 3.3, 7.1_
  - _Depends: 1.1, 1.2, 2.1_
  - _Boundary: finalize, build_source_map, SceneIdentityIndex_

- [x] 3. Core: ランタイム位置→シーン解決器
- [x] 3.1 位置→シーン解決器を実装する
  - 入力 (uri, line) を `.pasta` ファイルパス＋行番号へ正規化する。正規化は `std::path::absolute` を用い `fs::canonicalize` は使わない（CI の 8.3 短縮名パス不具合回避）
  - VSCode 側 uri→path と索引キー生成を同一規則に揃える
  - ロード済み索引で位置を解決し、確定時は (scene_id, parent)、解決不能時は未検出を返す（最内local優先 > 後方フォールバック、下方に無ければ未検出）
  - 確定した (scene_id, parent) で既存 kick 取次点（`KickSink`／`KickRequest`）を呼び、kick セマンティクスを継承する（取次は fire-and-forget・索引 lookup は読み取り専用で GET ブロックを延ばさない）
  - **kick transport の local 対応（composite-string 方式・debug 限定・通常モード無改修・`KickRequest`/pasta_shiori 不変）**：resolver は確定 (scene_id, parent) を `KickRequest.scene` の search-key 形式へ変換する（global → `会話1`、local → `:会話1:挨拶_1`）。kick.lua `try_dispatch` のみ改修し、先頭 `:` を local-composite として分解→`SCENE.search(local, parent)` の local 分岐でコルーチン化。Implementation Notes「local シーン kick の方式決定」「3.1 への波及（最終方式＝composite-string）」参照
  - 単体テストで「正規化済みパスの一致／global 確定→取次呼出（scene=`会話1`）／local 確定→取次呼出（scene=`:会話1:挨拶_1`）／未検出→取次しない」を観測できる。kick.lua の `:`-分岐は Lua 側テストで local func 解決を観測
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 7.1, 7.6, 8.1, 8.3_
  - _Depends: 1.1_
  - _Boundary: PositionResolver, kick.lua_

- [ ] 4. Core: DAP トランスポート拡張（位置ベース実行＋リロード）
- [ ] 4.1 位置ベース実行リクエストを既存 DAP チャネルに追加する
  - 既存デバッグ用カスタムリクエストチャネルに「位置 (uri, line) を入力とするシーン実行要求」を追加し、uri/line を厳格にパースする
  - 受理時に位置→シーン解決器を呼び、確定→成功応答、未検出→理由付きエラー応答を返す
  - 内部 kick 取次点（`KickSink`）は撤去せず再利用する（確定識別子で既存 sink を呼ぶ）
  - kick 経路の内部で `\![reload,shiori]` を自動送出しない
  - 統合テストで「成功要求→成功応答＋取次呼出」「未検出要求→エラー応答」を観測できる
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 2.3, 2.6, 5.5, 7.3, 8.1, 8.2, 8.3_
  - _Depends: 3.1_
  - _Boundary: decode, wiring/inbound_

- [ ] 4.2 旧シーン名ベースの外部トランスポートを撤去する
  - 旧シーン名入力のシーン実行外部リクエスト（デコード arm・関連デコード状態・外部受理ヘルパ）を削除し、外部口を位置ベース一本に統一する
  - 統合テストで「旧名前ベース arm 不在」「外部からのシーン名ベース要求が受理されない」ことを観測できる
  - _Requirements: 5.4_
  - _Depends: 4.1_
  - _Boundary: decode, wiring/inbound_

- [ ] 4.3 SHIORI リロードリクエストを追加しエンジンからリロード用さくらスクリプトを出力する
  - 既存デバッグチャネルに「SHIORI リロード要求」を追加する
  - 受理時、エンジンが `\![reload,shiori]`（SHIORI のみ再読み込み・非同期）をさくらスクリプト出力に載せ、SSP が次の GET で受け取れるようにする（SSP への直送はしない）
  - 統合テストで「リロード要求受理 → 次応答に `\![reload,shiori]` が含まれる」ことを観測できる
  - _Requirements: 9.2_
  - _Depends: 4.2_
  - _Boundary: decode, wiring/inbound_

- [ ] 5. Core: VSCode UI 動線（カーソル実行・リロード・旧動線撤去）
- [ ] 5.1 カーソル位置からのシーン実行コマンドと右クリック動線を実装する
  - `.pasta` エディタのコンテキストメニュー最上段（ナビゲーション群先頭）に「▶ シーンを実行」を常時表示する（`.pasta` 言語のとき・デバッグ接続有無に依存しない）
  - コマンドはアクティブエディタのカーソル位置 (uri, 行) を取得し、シーン名の手入力を要求せず位置ベース実行要求を送る
  - 未接続時は要求を送らず警告し、「デバッグ開始」アクションを提示する。誘導はワークスペースの `pasta` 構成優先・無ければ既定アタッチ構成（`127.0.0.1:9276`）でデバッグセッションを開始する
  - バッファが未保存（dirty）なら、実行中ゴーストと実態がずれ得る旨を警告し保存＋リロードを促す
  - エンジンからのエラー応答を作者へ提示する
  - 旧シーン名入力コマンド・関連 pure helper・メニュー貢献を削除し、位置ベースを唯一の作者向け動線にする
  - 右クリックで最上段に項目が出て、シーン本文上で実行するとライブ SSP に当該シーンが再生されることを E2E 補助テスト／手動確認で観測できる
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4, 7.2, 7.5_
  - _Boundary: VSCode RunSceneAtCursor_
  - _Depends: 4.1_

- [ ] 5.2 SHIORI リロードコマンドと自動再アタッチを実装する
  - `.pasta` 右クリックメニューとデバッグツールバーの両方に「SHIORIリロード」を提供し、いずれも接続中（`debugType == 'pasta'`）のみ表示・有効化する
  - 実行時、dirty バッファがあれば未保存変更がリロードに反映されない旨を提示し保存を促す
  - リロード要求を送り、デタッチ後に数秒待機して既定 `pasta` アタッチ構成で再アタッチし、失敗時は短間隔リトライ（上限/タイムアウトあり）する。アタッチ構成解決は 5.1 の誘導と共通化する
  - 再アタッチ完了後にシーンキックを自動再実行しない（手動再キック）
  - リロード実行→デタッチ→自動再アタッチで接続状態が復帰することを E2E 補助テスト／手動確認で観測できる
  - _Requirements: 7.4, 9.1, 9.3, 9.4, 9.5_
  - _Depends: 4.3, 5.1_
  - _Boundary: VSCode ReloadShiori_

- [ ] 6. Integration & Validation
- [ ] 6.1 ビルド→ランタイムの一致と後方非破壊を検証する
  - `.pasta` を transpile→索引構築→ロード→位置解決した結果が、ランタイム実シーン識別子およびシーン名検索の解決対象に一致することを統合テストで確認する
  - 既存の行マッピング機能（双方向 resolve）の挙動が索引追加後も不変であることを回帰テストで確認する
  - 一致テストと回帰テストがともに緑になることを観測できる
  - _Requirements: 3.1, 3.3, 3.4, 7.1_
  - _Depends: 2.2, 3.1_

- [ ] 6.2 トランスポート往復と kick セマンティクス継承を検証する
  - 位置ベース実行要求の往復（成功→応答＋既存 kick 取次呼出／未検出→エラー応答）を統合テストで確認する
  - 確定シーンが既存 kick 取次点（同一 sink）経由で起動され、co_scene 設置・preempt 等のセマンティクスを継承することを確認する
  - 往復テストと sink 経由確認がともに緑になることを観測できる
  - _Requirements: 4.2, 4.3, 8.1_
  - _Depends: 4.1_

- [ ] 6.3 uri 正規化の特性化テストを追加する
  - VSCode uri → 索引キーの正規化が、Windows パス・URI エンコード・CI の 8.3 短縮名パス（例 `RUNNER~1`）でミスマッチせず正しく解決することを特性化テストで固定する
  - `fs::canonicalize` 非使用・`std::path::absolute` 統一の前提を崩すと失敗するテストになっていることを観測できる
  - _Requirements: 2.1, 7.1_
  - _Depends: 3.1_

- [ ] 6.4 VSCode UI の検証を行う（自動 E2E ＋ 手動 SSP ゲート）
  - 自動 E2E／UI テスト（デバッグセッションをモック）で、右クリック最上段表示（`.pasta` 言語時）、未接続時の警告＋デバッグ開始誘導の呼び出し、dirty 時の保存促し、コマンド登録／メニュー貢献を確認する
  - 実 SSP を要する項目（シーン位置実行→ライブ SSP 再生＝1.5、SHIORIリロード→デタッチ→自動再アタッチ＝9.3）は、自動化が困難なため手動検証ゲートとして手順を明記し実施する
  - 自動テストが緑になり、手動ゲート項目がチェックリストで確認済みになることを観測できる
  - _Requirements: 1.1, 1.5, 6.1, 6.2, 7.4, 9.1, 9.3_
  - _Depends: 5.1, 5.2_

## Implementation Notes

- 1.1: `SceneIdentityIndex` の行範囲は実装上 **両端 inclusive** `[start_line, end_line]`（`contains` = `start <= line <= end`）。design.md:297 は半開区間記法 `[start_line, end_line)` で書かれているが、`end_line` を「次の同レベル以上宣言の**直前行**」と定義しているため具体行のメンバシップは等価。task 2.2 の構築側は **「次宣言行 − 1」（直前行）を inclusive な end_line として投入**すること（次宣言行そのものを渡さない）。空ファイルは `insert_file` で空エントリを作れば未検出を返す。
- 1.2: `SourceMapSink::record_scene(&mut self, scene_join_key: &str, span: Span)` はデフォルト no-op の **2引数**シグネチャ（design.md:341 が SSOT）。`end_line` と入れ子 `level` は**この trait メソッドの引数に含めない**。task 2.1（scope_gen 記録）/2.2（finalize join）の **builder 側**で算出する想定：`end_line` は次の同レベル以上宣言検出時または finish 時に確定（design.md:349）、`level` は scope_gen の呼び出し文脈（global=0/local=1…）から builder が把握。よって 2.1/2.2 で `MapBuilderSink` 内にシーン蓄積（join_key→(start,end,level)）を実装する際、必要なら `record_scene` を builder 内部で受けて level/end をローカルに track する設計とし、trait シグネチャは増やさない方針。

- 2.x join_key 契約（**設計の緊張を解決**: design.md:341 の2引数 trait と task 2.1「終端と入れ子レベルを受け渡す」の不整合を、design.md SSOT 優先で解決）：
  - **trait は2引数のまま**。終端・レベルは trait 引数で渡さず、**`scene_join_key` 文字列に決定的に符号化**して 2.2（finalize/builder）が復元する。
  - 符号化が担うべき情報（2.2 が復元に要する）：(a) global/local 種別とネスト level、(b) 親 global への参照、(c) 出現順（global は **per-base 出現順＝ランタイム `create_scene` 連番順**に一致させること。これがランタイム実 identity `会話1` との突合の鍵）。
  - global counter は**ランタイム付与**（scope_gen は base 名のみ出力・`scope_gen.rs:119-121`）。local counter は**code_gen 算出**（`scope_gen.rs:154-162` per-name HashMap、`fn_name = {sanitize}_{counter}`）。よって join_key で global は (base, 出現順) を、local は (親base, 親出現順, local fn_name) を符号化する想定。
  - `SceneRegistry::sanitize_name` の連番を code_gen 側で**二重実装しない**（design.md:329/350 形式ドリフト回避）。
  - 2.1 は scope_gen の global=`scope_gen.rs:139`／local=`scope_gen.rs:222`（既存 `record_span` 近傍）で `record_scene` を呼ぶところまで。蓄積・join・level/end 復元・runtime identity 突合は 2.2。
  - **2.1 確定 join_key 形式（2.2 が consume する）**: global=`G:{base}#{counter}`、local=`L:{parent_base}#{parent_counter}:{fn_name}`。`base`=`SceneRegistry::sanitize_name(name)`、`counter`=per-base 出現順（=ランタイム `create_scene` 連番。`G:会話#1`↔runtime `会話1` を pasta_core `increment_counter`／`scene.lua:48-52` まで遡り検証済）、`fn_name`=`{sanitize}_{counter}` または `__start__`。`G`/`L` 接頭辞が種別＋level、local の `{parent_base}#{parent_counter}` が親参照。形式は `scope_gen.rs` の `generate_global_scene` 内コメントに明記済。
  - **2.2 への注意（レビュー指摘 #6・2.1 由来でなく既存性質）**: `increment_counter` は **raw** `scene.name` をキーにするが、ランタイム counter は **sanitized** `base_name` をキーにする。よって「異なる raw 名が同一 base へ正規化される」ケースでは join_key の出現順とランタイム連番がズレうる。2.2 の突合実装時にこの境界を**特性化テストで固定**するか、join を sanitized-base ベースの出現順に揃えること（`increment_counter` のキー基準を確認）。

- **local シーン kick の方式決定（ユーザー承認済・要件2.2「最内 local 優先」の実現方式）**：
  - **背景の調査結論**：既存 kick 取次（`KickRequest{scene:String}` → `co_exec(act, name)` → `find_scene` → `find_handler` L5 → `SCENE.search(name, nil)`）は **local を再生できない**。理由：(1) `search_scene` の **global 分岐（parent=nil）が `context.rs:107-109` で local_name を捨て `"__start__"` を強制**するため、`resolve_scene_id(":会話1:挨拶_1")` が local の SceneId に当たっても返り値が `(会話1, __start__)` に潰れ、global の頭が再生される。(2) `act.lua:432/435` の `find_scene` は `global_scene_name` を**未使用**として破棄するため、co_exec に parent を渡しても効かない。→ 「無改修の単一文字列で local を引く」案（🅲）は**不成立**。
  - **採用方式（🅰・debug 限定・通常モード無改修）**：local を狙うには `search_scene` の **local 分岐**（parent あり → `resolve_scene_id_unified(parent, name)` → local_name を正しく保持）を通す。実装は kick.lua 側で「parent あり時は `SCENE.search(local_name, parent)` で直接解決し func をコルーチン化」する小改修。**通常 SSP 動作では kick 要求が来ず `kick_pending` も立たないため完全に inert**（kick 経路・索引ともに debug opt-in）。**この『通常モード非破壊』は 2.2/3.1/4.1/6.2 のレビュー必須検証項目**。
  - **2.2 索引が格納する識別子**：`collect_scenes` が返す `(global_name, local_name)`（例 `(会話1, 挨拶_1)`）をそのまま使い、各シーンを **(scene_id, parent)** で保持する。global → `scene_id=会話1, parent=None, level=0`。local → `scene_id=挨拶_1, parent=会話1, level=1`。よって task 1.1 の `SceneIdentityIndex`/`SceneSpan` を **parent: Option<String> を持つよう拡張**（2.2 境界内）。`scene_at` は最内（level 最大）を返し、呼び出し側へ (scene_id, parent) を渡せること。
  - **2.2 の join（local 含む）**：join_key の親参照 `会話#1`（base=会話・出現順1）→ runtime global_name `会話1` を (base, 出現順) で対応付け、local は `L:会話#1:挨拶_1` → `(parent=会話1, local fn_name=挨拶_1)` を `collect_scenes` の `(会話1, 挨拶_1)` と突合する。
  - **3.1 への波及（最終方式＝composite-string・KickRequest と pasta_shiori は不変）**：当初案の「`KickRequest{scene,parent}` フィールド追加」は **採らない**。理由：parent を別フィールドにすると配送鎖（`KickRequest`→`lifecycle.rs kick_into_mailbox`→`ActorMsg::Kick{scene}`→`thread.rs shiori.kick(scene)`→`entry.lua SHIORI.kick`→`KICK.install`）の **全段（pasta_shiori 跨ぎ）に波及**する。
    - 代わりに **resolver が `KickRequest.scene` に search-key 形式を載せる**：global → `会話1`（従来通り）、local → `:会話1:挨拶_1`（= `:{parent}:{local_fn_name}`）。`KickRequest{scene:String}` 契約は**不変**（R5.5「既存取次点を再利用」に最も忠実）、**pasta_shiori も無改修**（scene は不透明文字列として `shiori.kick(&scene)` まで素通し・`thread.rs:214`／`entry.lua:99-100` 確認済）。
    - **kick.lua `try_dispatch` のみ改修**：`scene_name` が先頭 `:` なら local-composite として `:parent:local` を分解し、`SCENE.search(local, parent)`（local 分岐 → `resolve_scene_id_unified` → local_name 保持）で func を取得しコルーチン化（`choice_select.lua` の `create_scene_coroutine` パターン／`co_exec` のラッパを踏襲）。先頭 `:` でなければ従来の `co_exec(act, scene)`（global・不変）。
    - **`:` 識別子の安全性**：`SceneRegistry::sanitize_name`（`scene_registry.rs:228`）が英数字・`_` 以外を `_` 置換するため、global 実名は決して `:` を含まない＝衝突しない。
    - **debug 限定・通常モード非破壊**：kick 経路は debug opt-in、`kick_pending` が立つ時のみ作用。通常 SSP は無影響。
    - index は引き続き (scene_id, parent) を格納（2.2 済）。resolver がそれを composite-string へ変換するだけ。`KickRequest`／`ActorMsg`／`SHIORI.kick` の signature は触らない。

- **検証の教訓（全タスク共通・2.1 で露呈）**：`cargo test -p pasta_lua --lib` だけでは **`tests/` 配下の integration test 目標を検証しない**。task 2.1 は `generate_local_scene` に `parent_ref` を追加したが `tests/transpiler/record_wiring_scope_test.rs` の呼び出し側を更新せず、`--lib` 緑のままコミットした（2.2 で inline 修正済）。**以降の実装・レビューは `cargo test -p pasta_lua`（全目標）で回帰確認すること**。`pub` 関数のシグネチャ変更時は `tests/` の呼び出し側も grep して更新する。
- **2.2 索引の attachment 機構（3.1 resolver が読む口）**：`SourceMap` に `scene_index: OnceLock<SceneIdentityIndex>`（write-once 内部可変）。join site は `runtime/factory.rs`（finalize 後・`source_map.is_some()` で debug-gated）で `build_scene_index(&lua, source_map)` を呼び `set_scene_index`。読み取りは `SourceMap::scene_at(file, line) -> Option<SceneIdentity{scene_id, parent}>`。`__start__` は level-1 として索引せず global 本体領域は `(会話N, None)` が覆う（global kick が `__start__` を強制する挙動と整合）。named local のみ level-1+parent。
