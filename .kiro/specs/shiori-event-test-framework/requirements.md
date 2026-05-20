# Requirements Document

## Project Description (Input)

**対象ユーザー**: pasta ゴースト開発者 / pasta エンジン開発者（テスト実装者）

**現状の課題**:
- `pasta_lua` のテストでは `@pasta_persistence`, `@pasta_search`, `@pasta_sakura_script` 等のRustバックエンドモジュールを各テストファイルで手動 `package.loaded` モック設定する必要があり、ボイラープレートが多い
- `pasta_shiori` のテストでは `parse_request()` が常に `now_local()` を呼ぶため時刻を固定できず、OnHour・OnTalk 等の時刻依存イベントのテストが書けない
- SHIORI レスポンスの検証が生文字列マッチ（`response.contains("200 OK")`）のみで、status コードや Value フィールドを構造化検証できない
- 上記の結果、`shiori-async-talk` で必要なマルチステップ SHIORI 往復テストを書く基盤がない

**あるべき姿**:
- Layer 1 (pasta_lua): SHIORI プロトコルに依存せず、Lua モック一括注入でイベントディスパッチテストが書ける
- Layer 2 (pasta_shiori): RAW SHIORI リクエスト文字列 + `X-Pasta-Time` ヘッダー（RFC 3339）で時刻を固定し、構造化レスポンス検証が書ける
- 両層とも `shiori-async-talk` のマルチステップ往復テストの前提条件を満たす

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
