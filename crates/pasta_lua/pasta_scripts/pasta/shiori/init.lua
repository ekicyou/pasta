--- @module pasta.shiori
--- SHIORI拡張モジュール
---
--- SHIORI/3.0プロトコル固有の機能を提供する。
--- 将来の拡張ポイント: リクエスト解析、レスポンス構築、ヘッダー操作等

local SHIORI = {}

-- 将来の拡張ポイント（未実装）:
-- - parse_request(text): リクエスト解析
-- - build_response(status, headers, body): レスポンス構築
-- - get_header(request, name): ヘッダー取得
-- - set_header(response, name, value): ヘッダー設定
--
-- 注意: 本モジュールはリポジトリ内 require 0 件の空テーブルだが、
-- pasta_scripts は zip 出荷物であり外部ゴーストが require しうる
-- 公開面のため維持する（3.47 ct.lua と同方針の既知負債）。

return SHIORI
