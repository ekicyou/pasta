# 実装計画

- [ ] 1. 基盤: 注入文法テストのための結線基盤を整備する
- [ ] 1.1 テスト専用 vendored lua 文法を導入する
  - VS Code 組み込み `source.lua` がテスト環境に存在しないため、`source.lua` として登録可能なテスト専用の最小 Lua 文法フィクスチャを editors/vscode 配下に追加する
  - vendored Lua 文法のライセンス表記ファイルを併せて追加する（出所・ライセンス条項を明記）
  - フィクスチャは Lua のキーワード・文字列・コメントを区別する最小スコープを付与し、ランタイムには同梱しない（テスト専用）
  - 完了条件: フィクスチャと LICENSE ファイルが配置され、`print("hello")` を Lua 系スコープへトークン化できることをフィクスチャ単体で確認できる
  - _Requirements: 1.2_
  - _Boundary: Lua Injection Grammar_

- [ ] 1.2 grammar テストの registry を注入対応へ拡張する
  - vscode-textmate の `Registry` を `getInjections('source.pasta') → ['pasta-lua.injection']` を返すよう構成し、注入が実際に行使される結線にする
  - `loadGrammar` が `source.pasta`・`pasta-lua.injection`・`source.lua`（1.1 の vendored fixture）の3文法を解決できるよう拡張する
  - 既存の単一文法ベースの `tokenizeLine` ヘルパ経路を壊さず、注入を伴う行トークナイズが可能な検証経路を用意する
  - 完了条件: 拡張後の registry で `source.pasta` をロードし、注入文法と vendored lua が解決され、注入なし構成では出ない `source.lua` 系スコープが本文に現れることを最小確認できる
  - _Requirements: 1.1_
  - _Boundary: Lua Injection Grammar_
  - _Depends: 1.1_

- [ ] 2. コア: Lua ブロック本文への source.lua 注入文法を実装する
- [ ] 2.1 注入文法ファイルを新規作成し拡張へ登録する
  - `meta.embedded.block.lua.content` スコープ配下にのみ `source.lua` を注入する VS Code 注入文法ファイルを新規作成する（`injectionSelector` は content 限定・`L:` 左優先）
  - 拡張マニフェストの文法コントリビューションへ、当該注入文法を `injectTo: ["source.pasta"]` で登録する
  - 共有 SSOT 文法（pasta 本体）およびセマンティックトークン凡例の登録は一切変更しない（読み取り標的のみ）
  - 完了条件: 拡張ロード時に注入文法が `source.pasta` へ注入登録され、SSOT 文法ファイルとマニフェストの既存文法・凡例エントリが差分なしであることを確認できる
  - _Requirements: 1.1, 1.3, 1.4, 2.1, 2.2_
  - _Boundary: Lua Injection Grammar_
  - _Depends: 1.2_

- [ ] 2.2 注入着色とフェンス保持・非注入境界を検証するテストを追加する
  - ```` ```lua ```` ブロック本文の `print("hello")` に `source.lua` 系スコープが付与されることを検証する（本文 Lua 着色）
  - 言語名なしフェンス（```` ``` ````）で開始するブロック本文にも Lua 着色が注入されることを検証する
  - 開始/終了フェンス行に pasta スコープ（フェンス句読点・言語名スコープ）が保持されることを検証する
  - アクション行内のインライン Lua（`＠func()`）に `source.lua` が注入されないことを検証する
  - 完了条件: 上記4観点のアサーションを含む grammar テストが追加され、注入結線下でグリーンになる
  - _Requirements: 1.1, 1.2, 2.1, 2.2, 5.2_
  - _Boundary: Lua Injection Grammar_
  - _Depends: 2.1_

- [ ] 3. コア: セマンティックトークン codeBlock をフェンス行限定へ縮小する
- [ ] 3.1 (P) コードブロックトークン出力をフェンス行のみへ変更する
  - Lua ブロックのトークン生成を、ブロック全域を覆う単一 `codeBlock` 出力から、開始フェンス行と終了フェンス行のみへ `codeBlock` を出力する実装へ変更する
  - 本文行（開始フェンス行と終了フェンス行の間）には `codeBlock` を一切出力しない
  - 他の visitor・他トークン種別・凡例（種別の並び順・`CODE_BLOCK` の凡例位置）には一切手を加えない
  - Lua ブロックが存在しない入力では新たなコードパスを通らず従来出力を維持する
  - 完了条件: ビルドが通り、本文行に `codeBlock` が出力されずフェンス2行のみに出力される実装に置き換わっている
  - _Requirements: 3.1, 3.2, 3.3, 4.2, 4.4_
  - _Boundary: CodeBlock Token Narrowing_

- [ ] 3.2 (P) フェンス限定化と無回帰を検証するユニットテストを追加する
  - 開始/終了フェンス行に `codeBlock` トークンが出力されることを検証する
  - 単数および複数本文行のブロックで、すべての本文行に `codeBlock` が無くフェンス2行のみに出力されることを検証する
  - Lua ブロックと pasta 要素（シーン・アクター・単語）が混在する文書で、`codeBlock` 以外のトークン列が変更前と同一であることを検証する
  - 凡例不変ガードとして、トークン種別の並び順と `CODE_BLOCK` の凡例位置が不変であることを検証する
  - 完了条件: 上記観点を含むユニットテストが追加され、3.1 の実装に対しグリーンになる
  - _Requirements: 3.1, 3.2, 4.1, 4.2, 4.4_
  - _Boundary: CodeBlock Token Narrowing_
  - _Depends: 3.1_

- [ ] 4. 検証: 無回帰と合成可視性を最終確認する
- [ ] 4.1 既存スイートと book ハイライト無回帰を確認する
  - pasta_lsp の `cargo test` と VS Code 拡張の grammar/unit/e2e テストスイートを全実行し、全パスを確認する
  - book ハイライタ（tokenizer / scope-map / highlight-html）のテストを実行し、SSOT 文法・book 未改変により出力が不変であることを確認する
  - Lua ブロック外の pasta 固有ハイライトおよび Lua ブロック無し文書のセマンティックトークン出力が導入前と同一であることを、既存スイートのグリーンで担保する
  - 範囲外操作（手動トグル・インライン Lua 注入・言語サービス）の不提供が、新規コマンド/機能追加の不在として保たれていることを確認する
  - 完了条件: 全テストスイートがグリーンで、book テストが期待値変更なしにパスする
  - _Requirements: 4.1, 4.2, 4.3, 4.5, 5.1, 5.3_
  - _Depends: 2.2, 3.2_

- [ ] 4.2 実 VS Code での合成可視性を目視確認する（DoD）
  - 実 VS Code で `.pasta` ファイルを開き、Lua ブロック本文が Lua 文法の色（キーワード・文字列・コメント・数値・関数名の区別）で表示されることを目視確認する
  - フェンス外の pasta 固有ハイライトが本機能導入前と変化していないことを目視確認する
  - 追加のコマンド実行やモード切替なしに着色が適用されていることを目視確認する
  - 完了条件: 上記の合成可視性が実エディタ上で確認され、本機能の完了条件（DoD）を満たす
  - _Requirements: 1.3, 3.3_
  - _Depends: 4.1_
