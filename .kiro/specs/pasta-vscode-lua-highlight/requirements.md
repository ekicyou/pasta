# Requirements Document

## Project Description (Input)
pasta ゴースト作者が `.pasta` ファイルを VS Code で編集する際、Lua コードブロック（```` ```lua ... ``` ````）の中身が Lua として色付けされず、単色（`codeBlock`）で塗りつぶされている。複雑なロジックを Lua で記述する作者にとって、キーワード・文字列・関数・コメントの視認性が無く、可読性と編集効率が低い。

本仕様は、`.pasta` の複数行 Lua コードブロック内部を VS Code 組み込みの Lua 文法（`source.lua`）で自動的にシンタックスハイライトすることを目的とする。実現手段は「TextMate 埋め込み言語注入 ＋ セマンティックトークン範囲調整」（ユーザー承認済み 2026-06-12）。TextMate 文法 `lua-code-block` の content に `source.lua` を注入し、同時に pasta_lsp の `visit_code_block` が出力する `codeBlock` セマンティックトークンが Lua ブロック本文全域を上書きする問題を解消する。Lua ブロック外の pasta 固有ハイライトおよび既存セマンティックトークンは外部観測挙動として無回帰であることを保証する。

## Introduction
本要件は、VS Code 上で `.pasta` ファイルを編集するゴースト作者に対し、複数行 Lua コードブロックの内部を Lua 言語として可視化（シンタックスハイライト）する機能を定義する。作者は追加操作なしに、Lua のキーワード・文字列・コメント・関数などが色分けされた状態でコードブロックを編集できるようになる。本機能はハイライト（見た目の色付け）のみを対象とし、Lua ブロック内の診断・補完・定義ジャンプ等の言語サービス機能は範囲外とする。

## Boundary Context
- **In scope（本仕様が担当する）**:
  - `.pasta` の複数行 Lua コードブロック（```` ``` ```` または ```` ```lua ```` で開始するフェンス区間）本文を、VS Code 組み込み Lua 文法で自動ハイライトする。
  - フェンスマーカー（```` ``` ````, `lua`）が pasta 側のスコープで識別され続けること。
  - Lua ブロック本文を覆っていた `codeBlock` セマンティックトークンが、埋め込み Lua ハイライトを隠さないように範囲を調整されること。
  - 上記変更が Lua ブロック外の pasta 固有ハイライトおよび既存セマンティックトークンに回帰を起こさないこと。
  - SSOT TextMate 文法（`pasta.tmLanguage.json`）の改変に伴い、同一文法ファイルを共有する **mdBook 側の静的ハイライタ**（マニュアル）の出力を無回帰に保つための調停。本仕様の文法変更が book の二段トークナイズ前提を壊さないよう book 側を整合させ、book ハイライトの外部観測挙動を保持すること。
- **Out of scope（本仕様は担当しない）**:
  - ユーザー操作でハイライトの有効/無効を切り替えるトグルコマンドやボタン（手動切替は将来仕様）。本仕様は自動切替のみ。
  - インライン Lua（`＠func()` 形式の関数呼び出し）への Lua 文法注入。本仕様は複数行 Lua ブロックに限定する。
  - Lua ブロック内の診断・補完・定義ジャンプ等の言語サービス拡張。
  - book/ マニュアルのハイライト**機能仕様そのもの**（章コンテンツの色付け方針・対象範囲・二段トークナイズ設計）は別仕様 pasta-manual-syntax-highlight が担当する。ただし、本仕様による SSOT 文法改変がもたらす book ハイライタの無回帰調停は In scope（上記参照）。
  - pasta DSL 文法そのものの変更。
- **Adjacent expectations（隣接仕様・前提への期待）**:
  - VS Code 組み込みの Lua 文法（`source.lua`）が常時利用可能であること（追加 grammar の同梱は不要）。
  - 既存の pasta-vscode-extension（TextMate 文法・セマンティックトークン基盤）および pasta-language-server（LSP/WASM 解析基盤）が稼働済みであること。
  - セマンティックトークンが TextMate ハイライトより優先される VS Code の合成順序を前提とすること。

## Requirements

### Requirement 1: Lua コードブロック内部の自動シンタックスハイライト
**Objective:** ゴースト作者として、`.pasta` の Lua コードブロックを編集するとき、その内部が Lua として色分け表示されてほしい。そうすれば複雑な Lua ロジックの可読性と編集効率が上がる。

#### Acceptance Criteria
1. When 作者が `.pasta` ファイル内で複数行 Lua コードブロックを開き、その本文に Lua コードを記述したとき、the pasta VS Code 拡張 shall ブロック本文を VS Code 組み込み Lua 文法に基づいてシンタックスハイライトする。
2. The pasta VS Code 拡張 shall Lua ブロック内の Lua キーワード・文字列・コメント・数値・関数名を、それぞれ Lua 文法のスコープに従って区別可能な色で表示する。
3. While Lua コードブロックがハイライト対象として認識されている間、the pasta VS Code 拡張 shall 作者による追加のコマンド実行やモード切替を要求せずにハイライトを適用する。
4. When 作者が Lua ブロックの本文を編集（追加・削除・変更）したとき、the pasta VS Code 拡張 shall 編集後の内容に対しても Lua シンタックスハイライトを維持する。

### Requirement 2: フェンスマーカーの pasta スコープ保持
**Objective:** ゴースト作者として、Lua ブロックのフェンス行（```` ``` ````、`lua`）が pasta 側で識別され続けてほしい。そうすればブロックの境界が pasta ファイルの文脈で一貫して認識できる。

#### Acceptance Criteria
1. When Lua コードブロックの開始フェンス行および終了フェンス行が表示されるとき、the pasta VS Code 拡張 shall フェンスマーカーを pasta 側のスコープで識別する。
2. The pasta VS Code 拡張 shall フェンスマーカーの識別と本文の Lua ハイライトを、ブロックの開始・終了境界で一貫して区別する。

### Requirement 3: コードブロックを覆うセマンティックトークンの範囲調整
**Objective:** ゴースト作者として、Lua ブロック本文に注入された Lua ハイライトが、pasta 側のトークンによって隠されずに見えてほしい。そうすればハイライトが実際に画面へ反映される。

#### Acceptance Criteria
1. While `.pasta` ファイルに対しセマンティックトークンが生成されている間、the pasta 解析機能 shall Lua コードブロック本文を覆う `codeBlock` セマンティックトークンを、本文の Lua ハイライトが可視となる範囲に限定する。
2. When Lua コードブロックが解析されたとき、the pasta 解析機能 shall ブロック本文全域を単一の `codeBlock` トークンで覆わない。
3. The pasta VS Code 拡張 shall セマンティックトークンと Lua ハイライトの合成結果として、Lua ブロック本文が Lua 文法の色で表示される状態を提供する。

### Requirement 4: Lua ブロック外およびセマンティックトークンの無回帰
**Objective:** ゴースト作者として、本機能の導入によって Lua ブロック以外の既存ハイライト、および同一 SSOT 文法を共有するマニュアル（mdBook）側のハイライトが変化しないでほしい。そうすれば既存の編集体験とマニュアル表示の双方が損なわれない。

#### Acceptance Criteria
1. The pasta VS Code 拡張 shall Lua コードブロック外の pasta 固有ハイライト（シーン・アクター・単語・変数・Call 文・Talk 文・さくらスクリプト等）を本機能導入前と同一の外部観測挙動で保持する。
2. The pasta 解析機能 shall Lua コードブロック以外の要素に対するセマンティックトークン出力を本機能導入前と同一に保持する。
3. When `.pasta` ファイルに Lua コードブロックが含まれないとき、the pasta VS Code 拡張 shall 本機能導入前と同一のハイライト結果を表示する。
4. The pasta 解析機能 shall 既存のセマンティックトークン種別および修飾子の定義（種別の並び順・凡例）を変更しない。
5. While 本機能が SSOT TextMate 文法を改変している間、the pasta 構文ハイライト基盤 shall 同一文法を共有するマニュアル（mdBook）側の静的ハイライト出力を、本機能導入前と同一の外部観測挙動で保持する。

### Requirement 5: 範囲外操作の不提供
**Objective:** ゴースト作者として、本仕様の範囲外機能が誤って提供されないでほしい。そうすれば仕様境界が明確に保たれ、将来仕様との責務衝突を避けられる。

#### Acceptance Criteria
1. The pasta VS Code 拡張 shall 本仕様の一部として、ハイライトの有効/無効を切り替える手動トグルコマンドまたはボタンを提供しない。
2. The pasta VS Code 拡張 shall インライン Lua（`＠func()` 形式の関数呼び出し）に対する Lua 文法注入を本仕様の一部として行わない。
3. The pasta VS Code 拡張 shall Lua コードブロック内の診断・補完・定義ジャンプ等の言語サービス機能を本仕様の一部として提供しない。
