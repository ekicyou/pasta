# Requirements Document

## Project Description (Input)
store.luaのSTORE.saveテーブルについて、ランタイムロード時に永続化ファイルの読み込み、ランタイムドロップ時に永続化ファイルへの書き込みを行うようにrust側でサポートする。

１．rustからluaへの公開関数群に、永続化ファイルのロード関数を用意。
２．STORE.save = XXXとし、XXXのところでロード関数を呼ぶ。
３．drop時にSTORE.saveを保存する処理を実装。
４．可能なら永続化ファイルの簡単な難読化可能なシリアライズクレートを導入
５．難読化するかどうかのフラグをコンフィグファイルのフラグとして追加
６．その他、永続化に必要な要件を検討して実装

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
