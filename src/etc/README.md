スクリプト・辞書置き場
======================

スクリプト置き場。
あと、雑多なメモを保管しておく場所。





SHIORI/3.0 リクエスト
---------------------





SHIORI/3.0 レスポンス例
-----------------------
SHIORI/3.0 400 Bad Request
Charset: UTF-8
Sender: SHIORI-BASIC-2
X-SHIORI-BASIC-Reason: 
(空行)


ステータスコード
----------------

### 2xx - 処理完了
  * 200 OK          正常に終了した
  * 204 No Content  正常に終了したが、返すべきデータがない

### 3xx - 処理完了、追加アクション要求
  * 310 Communicate     - deprecated -
  * 311 Not Enough      TEACH リクエストを受けたが、情報が足りない
  * 312 Advice  TEACH リクエスト内の最も新しいヘッダが解釈不能

### 4xx - リクエストエラー
  * 400 Bad Request     リクエスト不備

### 5xx - サーバエラー
  * 500 Internal Server Error   サーバ内でエラーが発生した
