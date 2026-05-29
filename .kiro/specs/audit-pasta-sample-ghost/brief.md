# Brief: audit-pasta-sample-ghost

## Problem
pasta_sample_ghostはサンプルゴースト「hello-pasta」のビルド・画像生成を行う小規模クレート（~300行）。image/imageprocによる画像処理を含む。publish=falseのため直接配布はされないが、リリースNARの元データを生成するため品質は重要。

## Current State
- ~300行のソースコード（src/ 5ファイル）
- image 0.25 / imageproc 0.26 による画像生成
- build.rs でのビルド時処理
- ghosts/ 配下にゴーストデータ

## Desired Outcome
- 画像処理ライブラリ使用箇所の安全性確認
- build.rs のファイルI/O安全性検証
- デッドコード除去、冗長表現削減
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。画像処理→build.rs→デッドコードの順に調査する。小規模のため短時間で完了見込み。

## Scope
- **In**: pasta_sample_ghost/src/ 全ファイルの脆弱性調査、build.rs の安全性確認、デッドコード除去
- **Out**: ゴーストデータ（ghosts/）の内容変更、画像デザインの変更、新しいサンプルゴースト追加

## Boundary Candidates
- 画像生成ロジック
- build.rs ビルドスクリプト
- ゴーストデータ読み込み

## Out of Boundary
- image/imageproc クレートの内部実装
- ゴーストデータのコンテンツ

## Upstream / Downstream
- **Upstream**: なし（独立クレート）
- **Downstream**: なし（publish=false、リリースNARの元データ）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: release-workflow（NARビルドで参照）

## Constraints
- 外部振る舞い（生成画像・ゴーストデータ）不変
- 既存テスト全パス必須
