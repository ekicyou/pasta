# Requirements Document

## Introduction
Lua 5.5 リファレンスマニュアル日本語翻訳ドキュメント（`crates/pasta_lua/doc/lua55-manual/`）を、独立したGitHubリポジトリ `ekicyou/lua55-manual-ja` として分離する。これにより、翻訳ドキュメントの独立した管理・公開・コントリビューションが可能になる。

## Project Description (Input)
「lua55-manual」一式を、githubの別リポジトリ「lua55-manual-ja」を新規に作ってそちらにpushする。「lua55-manual」階層がそのまま新リポジトリのルートになる感じ。pushできたら本リポジトリから「lua55-manual」一式を削除する。

## Requirements

### Requirement 1: 新規リポジトリ作成
**Objective:** As a プロジェクトオーナー, I want Lua 5.5日本語マニュアルを独立リポジトリとして管理したい, so that 翻訳ドキュメントの単独公開とコントリビューションが容易になる

#### Acceptance Criteria
1. When リポジトリ作成を実行した場合, the GitHub shall `ekicyou/lua55-manual-ja` リポジトリを作成する
2. The リポジトリ shall 公開（public）設定で作成される
3. The リポジトリ shall 適切な説明文（Lua 5.5リファレンスマニュアル日本語翻訳）を持つ

### Requirement 2: ディレクトリ構造の移行
**Objective:** As a 開発者, I want lua55-manualの内容がそのまま新リポジトリのルートになるようにしたい, so that 既存のドキュメント構造を維持しつつ独立リポジトリとして機能する

#### Acceptance Criteria
1. When 移行を実行した場合, the 新リポジトリ shall 以下のファイルをルートに配置する:
   - `README.md`
   - `LICENSE.md`
   - `ABOUT.md`
   - `GLOSSARY.md`
   - `01-introduction.md` ～ `09-complete-syntax.md`
2. The 新リポジトリ shall 元の `lua55-manual/` ディレクトリ階層を持たない（フラット構造）
3. The ファイル内容 shall 完全に保持される（改変なし）

### Requirement 3: Gitヒストリの取り扱い
**Objective:** As a プロジェクト管理者, I want 移行方法を明確にしたい, so that ヒストリ保持の要否を判断できる

#### Acceptance Criteria
1. The 移行作業 shall 単純コピー方式（ヒストリなし）で実行する
2. If ヒストリ保持が必要な場合, then the 作業者 shall git filter-branch等の別手法を検討する（本仕様スコープ外）

### Requirement 4: 新リポジトリへのプッシュ
**Objective:** As a 開発者, I want 新リポジトリにドキュメントをプッシュしたい, so that GitHubで公開される

#### Acceptance Criteria
1. When プッシュを実行した場合, the Git shall mainブランチに全ファイルをコミットする
2. The コミットメッセージ shall 「Initial commit: Lua 5.5 Reference Manual Japanese Translation」とする
3. When プッシュが成功した場合, the GitHub shall リポジトリページでドキュメントが閲覧可能になる

### Requirement 5: 元リポジトリからの削除
**Objective:** As a プロジェクト管理者, I want プッシュ成功後に元ディレクトリを削除したい, so that 重複管理を避ける

#### Acceptance Criteria
1. While 新リポジトリへのプッシュが成功している場合, when 削除を実行した場合, the pasta リポジトリ shall `crates/pasta_lua/doc/lua55-manual/` ディレクトリを削除する
2. The 削除 shall Gitコミットとして記録される
3. The コミットメッセージ shall 新リポジトリへの移行を明記する

### Requirement 6: ドキュメント参照の更新（オプション）
**Objective:** As a 開発者, I want 関連ドキュメントの参照を更新したい, so that リンク切れを防ぐ

#### Acceptance Criteria
1. If pasta リポジトリ内に lua55-manual への参照がある場合, then the 作業者 shall 新リポジトリURLへの更新を検討する
2. Where ステアリングファイルに参照がある場合, the 参照 shall 新リポジトリURLに更新される
