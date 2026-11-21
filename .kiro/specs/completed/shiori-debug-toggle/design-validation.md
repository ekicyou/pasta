# 設計の検証: shiori-debug-toggle

- 対象: `design.md`（2026-11-21 生成）・`requirements.md`（Requirement 1〜10）・`research.md`
- 方法: `kiro-validate-design`（`rules/design-review.md`）の手順。設計の主張を今のコードで確かめた
- 実行: 非対話（問いは出さず、議題は設計のディスカッションへ送る）

## Design Review Summary

判断の場所と時期（ローダの Phase 5.5・Lua の前）を変えずに、純粋な `decide`／`save_switch` と保存ファイルを読む閉包を足す設計で、要件 1〜10 のすべてに部品と試験が対応している。コードで確かめた主張（`build_filter_from` が `pasta::load_entry=info` を最後に足す・印は `spawn_actor` の後に FFI の入口で出る・`load_from_file` は読むだけ・リロードは古いランタイムの Drop で保存してから読み込む）はどれも正しい。残る弱みは、道具が頼る「ログの行の契約」の決め方と試験、recipe の状態の戻し方の 2 か所で、どちらも設計の小さな手直しで閉じられる。

## 確かめたコードの事実

| 設計の主張 | 確かめた所 | 結果 |
|------------|------------|------|
| 判断は `resolve_enabled` 1 か所・Phase 5.5 で 1 回 | `debug/config.rs`（`pick`）・`loader/mod.rs` 228 行 | 正しい |
| 有効の行は `Startup sequence completed` の直前・target は `pasta_lua::loader` | `loader/mod.rs` 252〜255 行・`scene_play_event_test.rs` 380 行 | 正しい（試験は直前であることを確かめている） |
| `pasta::load_entry` の info は `level`・`filter`・`PASTA_LOG` のどれでも通る | `logging/tracing_init.rs` `build_filter_from` と試験 4 本 | 正しい（作者が `pasta::load_entry=off` と書いても通る） |
| 印は FFI の入口が `spawn_actor` の後に出す | `pasta_shiori/src/windows.rs` 172〜189 行・`v3.rs` 66 行 | 正しい（ローダの行は必ず先に並ぶ） |
| 場所の確かめは `register` の中に直に書かれている | `runtime/persistence.rs` 120〜135 行 | 正しい（Drop の保存は確かめずに `join` する） |
| `load_from_file` は gzip を判別して読むだけ | `runtime/persistence.rs` 258 行 | 正しい |
| `ReloadOutcome` の `done` を作るのは `reloadAndWait` だけ | `editors/` の検索 | 正しい（試験は `as Extract<…>` で受けるだけ） |
| パレットに出すには `editor/context` に足さないだけでよい | `editors/vscode/package.json`（`commandPalette` の指定なし） | 正しい |
| 登録の数は 4 → 6 | `devCommands.ts`（出力の欄 1＋コマンド） | 正しい（5 コマンド＋1） |
| 予約の名前とぶつかる既存の利用 | リポジトリ全体で `SAVE.debug`・`＄＊debug`・`SAVE.デバッグ` を検索 | 0 件 |

## Critical Issues

### Critical Issue 1: 有効・無効の行を `pasta::load_entry` の target に相乗りさせる契約が曖昧

- **Concern**: `LOAD_ENTRY_TARGET` は「SHIORI の読み込みの終わりの行の target」として doc comment・`pasta_shiori` の 2 本の試験の意図（「読み込みの終わりの行のほかの info は絞られたまま」）・マニュアルの `[logging]` に書かれている。ローダの判断の行をこの target で出すと、名前と意味がずれ、「どの行がログの設定を越えて出るか」の決まりが暗黙に広がる。
- **Impact**: 後で誰かが `pasta::load_entry` を「印の行だけ」に絞ると、道具は黙って「確かめられなかった」に落ちる（5.4 が崩れる）。Revalidation Triggers に挙げてはいるが、名前から読めない。
- **Suggestion**: 専用の target（例 `pasta::debug_state`）を `logging` に定数で置き、`build_filter_from` で同じく info を強制で通す 1 行を足し、`tracing_init.rs` の試験に `level = "warn"`・`PASTA_LOG`・作者の `off` で通ることを足す。相乗りを残すなら、`LOAD_ENTRY_TARGET` の doc comment とマニュアルの決まりを「読み込みの終わりの行と、有効・無効の行」と明記する。OPEN QUESTION 1 と一緒に決める。
- **Traceability**: 5.4・5.5・3.3
- **Evidence**: design.md「LoaderPhase55」の行の表・「Revalidation Triggers」・OPEN QUESTIONS 1

### Critical Issue 2: 道具が頼る「有効・無効の行は印より前」が FFI を越えた試験で守られていない

- **Concern**: `reloadAndWait` は印で読むのを止めるので、トグルの正しさは「ローダの行が印より前に、ログを絞っても 1 行出る」ことに懸かる。設計はこれを `pasta_lua` の結合テスト（ローダだけ）と TS の偽の入出力で確かめ、FFI の入口を通す `ffi_loadu_test.rs`・`v3_load_entry_log_test.rs` は「変えずに通る」としている。
- **Impact**: 入口の順（`spawn_actor` の後に印）やログの書き手の並びが変わっても、どの試験も落ちない。道具は実機でだけ「確かめられなかった」になる。
- **Suggestion**: 絞ったゴースト（`level = "warn"`）を読む上の 2 本に、「無効の行がちょうど 1 行あり、`SHIORI load entry completed` より前にある」を足す（有効のゴーストでも 1 本）。Testing Strategy の Integration Tests に書き足す。
- **Traceability**: 5.4・10.3
- **Evidence**: design.md「LoaderPhase55」の Implementation Notes（`ffi_loadu_test.rs`・`v3_load_entry_log_test.rs` の注）・「Testing Strategy」

### Critical Issue 3: recipe の状態が、開けなかったゴーストの切り替えでも「まだ確かめていない」に戻る

- **Concern**: 設計は `setGhost(info)` で「master が替わったら `#debugOn = false`」とする。今の `main.ts`（144〜152 行）は切り替えの初めに `setGhost(null)` を呼び、開けなかったら `setGhost(before)` で同じゴーストへ戻す。null を挟むので、別のゴーストを開いていないのに状態が消える。
- **Impact**: 7.2（最後に確かめた状態が有効なら「無効にする」と示す）と 7.5（別のゴーストを開いたときだけ戻す）に食い違う。押すと既に有効なゴーストへもう一度 `true` を書くだけで害は小さいが、作者には逆の向きのボタンに見える。
- **Suggestion**: 状態に「確かめたゴーストの master」を添え（`#debugOn` を `{ master, on }` に）、ボタンの向きは今の master と一致するときだけ有効と読む。`setGhost(null)` では消さない。`devActions.test.ts` に「切り替えに失敗して同じゴーストへ戻ると向きを保つ」を足す。
- **Traceability**: 7.2・7.3・7.5・10.3
- **Evidence**: design.md「RecipeToggle」の State model と `setGhost` の注・「recipe のボタンの向き」の状態図

## Design Strengths

1. **今の部品をそのまま伸ばす最小の形**: 判断は `decide(env, save, file)` の純粋な関数のまま、保存ファイルは `PASTA_DEBUG` で決まらないときだけ閉包で読む。シーンマップの門・`@pasta_debug` の閉じ込め・Lua の起動順・`factory.rs`・`module_registry.rs` に触れないので、無効のときの `.lua` の不変（3.4）と、`shiori4-v4-host` とのファイルの重なりの無さが構造で守られる。27 通り・16 通りの表の単体テストも、環境変数を読まない形で 10.2 を満たす。
2. **「確かめられなかった」を正面から扱う**: ログの設定で行が落ちる穴（research の穴 1）を、ログの設定を越える target と `debug: boolean | undefined` の 3 値、`debugStateOf` の 1 か所の決めで閉じている。古い pasta.dll・リロードの失敗・時間切れがどれも「無効」と誤って言わない（5.5）。トグル・リロード・▶・「値を入れて実行」が同じ `ReloadOutcome` から向きを決める一般化も筋がよい。

## Final Assessment

- **Decision**: GO
- **Rationale**: 構造の食い違いや要件の抜けは無く、設計の主張はコードと合っている。3 つの指摘はどれも契約の明記・試験の追加・状態の持ち方の小さな手直しで、実装の道を変えない。
- **Next Steps**:
  1. 設計のディスカッション（`/kiro-design-discussion shiori-debug-toggle`）で、上の 3 つを design.md の 7 つの OPEN QUESTIONS と一緒に扱う（Issue 1 は OPEN QUESTION 1 と同じ議題）。
  2. 決めたことを design.md に入れてから `/kiro-spec-tasks shiori-debug-toggle` へ進む。

## 指摘にしなかった小さな気づき

- `ReloadOutcome` に欄を足しても、`done` を作るのは `reloadAndWait` だけなので型の壊れは出ない。`deepStrictEqual` で結果を丸ごと比べる試験があれば期待に `debug` を足す。
- 真偽値でない値の警告は既定の target（`pasta_lua::debug::config`）で出るので、作者が `filter = "pasta_lua=error"` に絞ると出ない。ほかの警告と同じ扱いで、要件の範囲内。
- 無効の行は配った先のゴーストの pasta.log にも、読み込みごとに 1 行増える（`level = "warn"` でも出る）。OPEN QUESTION 1 の判断材料。
