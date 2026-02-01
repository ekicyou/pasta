# Implementation Plan

## Task Overview
Lua 5.5日本語マニュアルを独立GitHubリポジトリに分離し、元リポジトリから削除する。

## Tasks

- [ ] 1. 新リポジトリ作成とファイル移行
  - [ ] 1.1 GitHub上に新リポジトリを作成する
    - `gh repo create lua55-manual-ja --public` でリポジトリを作成
    - 説明文を「Lua 5.5 Reference Manual - Japanese Translation」に設定
    - 作成成功を確認（GitHub URLアクセス可能）
    - _Requirements: 1_
  - [ ] 1.2 (P) ローカル作業ディレクトリを準備しファイルをコピーする
    - 一時作業ディレクトリを作成
    - `lua55-manual/` 配下の全13ファイルをルートにコピー
    - ファイル内容が完全に保持されていることを確認
    - _Requirements: 2, 3_
  - [ ] 1.3 新リポジトリにプッシュする
    - git init → add → commit（メッセージ: Initial commit: Lua 5.5 Reference Manual Japanese Translation）
    - リモート設定 → mainブランチにプッシュ
    - GitHubでドキュメントが閲覧可能なことを確認
    - _Requirements: 4_

- [ ] 2. 元リポジトリのクリーンアップ
  - [ ] 2.1 元ディレクトリを削除しコミットする
    - `crates/pasta_lua/doc/lua55-manual/` ディレクトリを削除
    - コミットメッセージに新リポジトリURLを明記
    - _Requirements: 5_
  - [ ] 2.2* (P) 関連ドキュメントの参照を更新する（オプション）
    - pasta リポジトリ内の lua55-manual への参照を検索
    - 必要に応じて新リポジトリURLに更新
    - _Requirements: 6_
