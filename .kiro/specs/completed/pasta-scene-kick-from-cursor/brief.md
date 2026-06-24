# Brief: pasta-scene-kick-from-cursor

## Problem
現状の scene-kick（`pasta-scene-kick`、完了済み）は、VSCode のコマンドパレット／デバッグツールバーから作者が **シーン名を手入力**（`showInputBox`）してキックする設計になっている。しかしシーン名は `.pasta` の `＊`（global）/`・`（local）宣言を `pasta_core::SceneRegistry` がサニタイズして生成する**内部識別子**（例: `会話_1`）であり、作者が知り得る情報ではない。
→ 「唯一持っていない情報（シーン名）を、唯一それを知らない者（作者）に要求している」状態であり、動線として破綻している。

## Current State
- **`pasta-scene-kick`（完了）**: executor スレッド上で対象シーンを `co_scene` として設置し、preempt-and-abort（進行中会話を中断・前 `co_scene` を閉じ自動復帰しない）、初回ビートのみ抑制ゲートをワンショット突破、以降は OnSecondChange シーン継続機構でビート単位配信。transport は debug DAP の `pasta/playScene`{scene 名}。**kick 実行本体はそのまま再利用可能**。
- **VSCode 拡張**: `pasta.debug.playScene` コマンドが `showInputBox` でシーン名を要求（`when: debugType == 'pasta'` ＝ デバッグセッション中のみ表示）。
- **`pasta-source-map`（完了・部分実装）**: `.pasta` 行 ↔ 生成 `.lua` 行 の双方向マップ。`resolve_lua_to_pasta` / `resolve_pasta_to_lua` あり。**「`.pasta` 位置 → シーン名」の逆引きは存在しない**。
- **LSP（`pasta_lsp`）**: scene marker の semantic token はあるが、document symbol / scene range など「この位置のシーンは何か」を答える API は無い。

## Desired Outcome
作者は `.pasta` を開き、**任意のシーン位置で右クリック → コンテキストメニュー最上段の「▶ シーンを実行」→ 即時実行 → ライブ SSP でそのシーンが再生**される。シーン名の手入力は不要・**廃止**。これがシーンキックの**唯一の動線**となる。

旧 `pasta.debug.playScene` コマンド（`showInputBox` でシーン名を手入力 / `when: debugType == 'pasta'` ＝ コマンドパレット・デバッグツールバー gated）は**存在自体を廃止（リジェクト）**する。

## Approach
- **採用（案 B・エンジン解決）**: エディタは **位置 (uri + line) のみ**を送る。実行中エンジンが**ロード済みソースマップ**で位置 → シーンを確定し、既存 kick backend へ取り次ぐ。実際に動いているゴーストが権威。
  - 理由: エディタはファイルと行しか持ち得ない。行からシーンを確定するのは、トランスパイラと同一の `SceneRegistry` 由来情報を持つエンジン側でしか確定的に行えない。編集中バッファとロード済み実態のずれを「動いているゴースト基準」で吸収できる。
- **必要拡張**:
  1. ソースマップに**シーン同一性索引**を追加（`.pasta` (file, 行範囲) → シーン ID）。トランスパイル時に `SceneRegistry` 情報を source-map sink 経由で受け渡す（現状は span のみ）。
  2. 新 custom request `pasta/playSceneAt`{uri, line}（既存 DAP チャネル一般化の続き）。
  3. VSCode `.pasta` エディタの**ネイティブ `editor/context` メニュー貢献**。新コマンド `pasta.runSceneAtCursor`（仮）を `group: navigation@1` で**右クリック最上段**に配置（ラベル例「▶ シーンを実行」、`when: resourceLangId == pasta` ＋ 要デバッグセッション）。コマンドはアクティブエディタのカーソル位置 (uri + line) を取得して `playSceneAt` を送るだけの薄い glue。
- **却下（案 A・LSP/エディタ側解決）**: 編集中バッファとロード済み実態がずれる、サニタイズ規則の重複実装で名前ドリフトの恐れ。
- **却下（Code Runner 連携）**: 「Run Code」は VSCode 標準ではなく第三者拡張 `formulahendry.code-runner`。(1) カーソル行を渡せずファイル単位実行のみ ＝「カーソル下のシーン」を表現不可（本仕様の中核要件と矛盾）、(2) サブプロセス → ターミナル stdout モデルで、既に動いているゴーストへの注入・ライブ SSP 描画ができない、(3) 全作者に第三者拡張の導入・設定を強制。ネイティブ `editor/context` 貢献で同等の見た目・最上段配置を依存ゼロで実現できるため不採用。

## Scope
- **In**: `.pasta` エディタのネイティブ `editor/context` 右クリック最上段コマンド（`pasta.runSceneAtCursor`）、カーソル位置 (uri + line) 送信、新 transport `playSceneAt`、エンジン側「位置 → シーン」解決、ソースマップへのシーン同一性索引追加、既存 kick backend 再利用、**旧 `pasta.debug.playScene`（シーン名入力）コマンドと showInputBox フローの廃止**。
- **Out**: kick 実行本体（co_scene 設置 / preempt-and-abort / OnSecondChange 継続）の再設計（`pasta-scene-kick` を流用）、LSP でのシーン解決（案 A 不採用）、Code Runner 連携（不採用）、SSTP / 別プレビュー画面、`*.pasta` 編集ウィンドウからのキック（別境界・将来）。

## Boundary Candidates
- ソースマップのシーン同一性索引（build-side: `SceneRegistry` → source-map sink）
- エンジン側「位置 → シーン」解決 + kick 取次（runtime）
- 新 transport `pasta/playSceneAt`（DAP custom request 一般化の続き）
- VSCode `.pasta` ネイティブ `editor/context` 右クリック UI（`group: navigation` 最上段）

## Out of Boundary
- kick 実行セマンティクス（preempt-and-abort 等）の変更
- ライブ SSP 以外の出力先

## Upstream / Downstream
- **Upstream**: `pasta-scene-kick`（kick backend）、`pasta-source-map`（マップ基盤）、`pasta-vscode-lua-debug`（DAP チャネル）
- **Downstream**: `pasta-authoring-window`（将来・編集ウィンドウからのキック）

## Existing Spec Touchpoints
- **Extends**: `pasta-scene-kick`（トリガーをシーン名入力 → 位置ベースへ置換）、`pasta-source-map`（シーン同一性索引を追加）
- **Adjacent**: `pasta-debug-lua-view-toggle`（DAP custom request + VSCode コマンドの前例）、`pasta-language-server`

## Constraints / 要件・設計フェーズで詰める論点
- **staleness / リロード整合**: エンジンはロード済み辞書で解決するため、`.pasta` がディスク上で新しい（未リロード／未保存）場合に行ずれで誤解決し得る。リロード要求 or ベストエフォート＋警告 など、解決失敗時の挙動を要件で定義する。
- **解決規則**: カーソルが属する最内シーン（local 優先）を選ぶ。どのシーンにも属さない位置（シーン間・空行等）の扱い（最近接シーン or エラー提示）を定義する。
- **transport 形**: `playSceneAt` 新設か `playScene` の引数拡張か。
- **後方互換**: 旧 `playScene`{scene 名} と `showInputBox` フローを完全廃止するか残すか（「唯一の動線」方針 ＝ 廃止寄り。`pasta.debug.playScene` コマンドはリジェクト確定）。
- **メニュー表示条件**: 右クリック項目を `resourceLangId == pasta` で常時出すか、要デバッグセッション（`debugType == 'pasta'`）で gate するか。常時表示ならセッション未接続時の挙動（自動アタッチ案内 or 警告）を定義する。
- SHIORI/3.0 整合・OnSecondChange ≤1 秒・GET ブロックは短く（`pasta-scene-kick` の制約を継承）。
