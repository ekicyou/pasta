# Implementation Plan

> 進行順は design.md の Migration Strategy（P1〜P8）に従う。ただし将来仕様の brief は吸収元を参照するため、撤去（P6）より前に置く（大タスク 6）。全変更は同一ブランチに積み、統合は単一 PR で行う（途中状態を main に出さない）。各大タスクの終わりに、その時点で存在する検証コマンドが緑であることを確認する（P3 の実リポジトリに対するスキル検査と鮮度照合のみ、大タスク 4 完了まで赤でよい）。
>
> **生成対象章の執筆制約**（2.1〜2.7 共通）: 本文（最初と最後の `---` の間）に口調・コラムを置かない。リンクは `.md` 相対リンクか絶対 URL のみとし、画像・非 `.md` 相対リンク・参照形式リンク・リポジトリ内パスを書かない。

- [x] 1. 吸収台帳と仕分け（P1）
- [x] 1.1 吸収台帳の本体を作成する
  - 吸収元（doc/spec ch01–07・09–11 の節、GRAMMAR.md の節、スキル手書き文法 7 ファイル・runtime-api・shiori-handlers・pasta-toml の節、authoring-patterns の挙動事実）の見出しを 1 行ずつ列挙する
  - 各行に「収録先（章#節）／既存収録済み／除外（理由）」と「実装照合（照合したソース位置 or 不要）」を記入する。収録先は design.md の章ごとの収録先表に従う
  - スキル内ナビゲーション（runtime-api・shiori-handlers 末尾の関連リファレンス等）は「除外（スキル内ナビゲーション）」と記録する
  - 完了状態: `absorption-ledger.md` が存在し、全行の収録先列と実装照合列が空でない
  - _Requirements: 1.1, 1.2, 1.6, 3.1, 3.2, 3.3, 4.1, 10.6_

- [x] 1.2 吸収元に無い実装構文を洗い出し、バグ候補を仕分ける
  - `grammar.pest`・トランスパイラ・ランタイムを読み、吸収元のどこにも書かれていない利用者向け構文を列挙する（例: 単独 `＊` 行による継続、`％a＝0、b` の番号付け、`＄０`、末尾 `#` コメント）
  - 各行を判定基準 a〜c に照らし「収録先（章#節）」または「バグ候補（根拠 a/b/c）」で埋める
  - 完了状態: 台帳の付録「未記載の実装事実」の全行が収録先かバグ候補で埋まっている
  - _Requirements: 1.8_

- [x] 1.3 既知の食い違いを grep して訂正対象を確定する
  - design.md の食い違い表の各項目について、マニュアル全章・両 SKILL.md・スキル手書きファイルを grep する
  - 実装を読んで食い違い表の「実装（正）」を照合し、表と異なる結果は実装を正として記録する
  - 完了状態: 台帳に項目ごとの grep 結果（ファイル・行）と照合したソース位置が記録されている
  - _Requirements: 1.6, 3.3_

- [x] 1.4 将来仕様の仕分けを実装照合で確定する
  - ch08・ch12 の各項目を仕分け規則（M／B／R／除外）で分類し、(M) が実装で確認できない項目は (R) へ倒す
  - マニュアル既存の「将来変更あり」節（call-jump フィルター・words 動的単語参照・block-structure 属性・grammar/index 属性注記・actor-dictionary コードブロック）を「削除して brief へ」と「現行挙動へ書き換え」に振り分ける
  - 完了状態: 台帳に仕分け表（項目・区分・行き先）があり、B・R の全項目に行き先が決まっている
  - _Requirements: 1.3, 1.4_

- [ ] 2. マニュアル内容の移し替えと章分割（P2）
- [x] 2.1 (P) 文法章（index・markers・block-structure）を拡充・訂正する
  - 台帳に従い doc/spec・GRAMMAR.md・スキル手書きの規範内容を吸収する。block-structure にはキューコマンド行・選択肢行の節、コメント・空行・エンコーディング、Lua ブロック規則（3 個以上のバッククォート＋任意識別子）を収録する
  - 属性は「構文は受理されるが処理に反映されない」現行挙動のみの記述へ書き換える（属性行はグローバルシーン初期部のみ、ローカルシーンは宣言行への付記のみ）。grammar/index の属性注記も同様にする
  - 付録「未記載の実装事実」で収録先がこの 3 章の行を収録する
  - 生成対象章の執筆制約を守る。章末「権威的仕様」引用はこの段階では残す
  - 完了状態: 3 章が台帳の該当行をすべて収録し、食い違い表の該当誤記が残っていない
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 1.7, 1.8_
  - _Boundary: ContentMigration（grammar/index・markers・block-structure）_
  - _Depends: 1.4_

- [x] 2.2 (P) 文法章（call-jump・literals・action-line）を拡充・訂正する
  - call-jump に「特殊な呼び出し」節（チェイントーク・yield・ゴースト終了・任意式の呼び出しと nil ガード）を新設し、Call 検索 5 段・ローカル優先の候補選択・引数の区切りと位置引数を実装どおりに書く
  - call-jump の「フィルター」節を削除する（内容は brief へ回る）
  - literals に文字列の囲み多重化（エスケープ規則なし・空文字列可）と値の型解釈を、action-line に `：` 始まりの行継続と継続内空行の現行挙動を書く
  - 生成対象章の執筆制約を守る
  - 完了状態: 3 章が台帳の該当行をすべて収録し、未実装機能の節と食い違い表の該当誤記が残っていない
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 1.7, 1.8_
  - _Boundary: ContentMigration（call-jump・literals・action-line）_
  - _Depends: 1.4_

- [x] 2.3 (P) 文法章（sakura-script・variables・words・actor-dictionary）を拡充・訂正する
  - sakura-script に括弧内エスケープと主要タグ早見、variables に Lua 予約語の制約・日時変数・`＄＊` の保存先と DSL→Lua 対応表、words にシャッフル＆順次消費・複数キー・読点／カンマ区切りの単語値・単語値のさくらスクリプト、actor-dictionary に 3 段フォールバック・バルーン連携を収録する
  - words の「動的単語参照」節を削除し、actor-dictionary のアクタースコープ内コードブロックは現行挙動のみの記述へ書き換える
  - 他章からアンカー参照されている variables の見出し（日時変数・リクエスト変数（Reference）・エンジンが値を入れる変数）は維持する
  - 生成対象章の執筆制約を守る
  - 完了状態: 4 章が台帳の該当行をすべて収録し、未実装機能の節と食い違い表の該当誤記が残っていない
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 1.7, 1.8_
  - _Boundary: ContentMigration（sakura-script・variables・words・actor-dictionary）_
  - _Depends: 1.4_

- [x] 2.4 (P) 公開モジュール章（一覧・@pasta_search・@pasta_persistence・@pasta_config）を新設する
  - 旧 `lua/modules.md` の該当節とスキル runtime-api の該当節を、情報量を減らさずにモジュール別章へ移す。一覧章にはモジュール一覧・require 名・共通事項を置く
  - セレクタ節の見出しは `### set_scene_selector(...) / set_word_selector(...)` の形を保つ（スキル側アンカーの互換）
  - 各章に導入と締めを書き下ろし、生成対象章の執筆制約を守る
  - 完了状態: 4 章が存在し、台帳の該当行がすべて収録先としてこれらの章を指している
  - _Requirements: 3.1, 3.3, 1.7_
  - _Boundary: ContentMigration（lua/modules 前半 4 章）_
  - _Depends: 1.3_

- [ ] 2.5 (P) 公開モジュール章（@pasta_sakura_script・@enc・@pasta_log・mlua-stdlib）を新設する
  - 旧 `lua/modules.md` とスキル runtime-api の該当節を移し、さくらスクリプト変換のウェイト（アクター表直下のキー・既定値・挿入値・連続句読点）を実装どおりに訂正する
  - 各章に導入と締めを書き下ろし、生成対象章の執筆制約を守る
  - 完了状態: 4 章が存在し、台帳の該当行がすべて収録先としてこれらの章を指している
  - _Requirements: 3.1, 3.3, 1.7_
  - _Boundary: ContentMigration（lua/modules 後半 4 章）_
  - _Depends: 1.3_

- [ ] 2.6 (P) SHIORI イベントとハンドラの章を新設する
  - スキル shiori-handlers の全節（REG・RES・イベント一覧・シーン関数フォールバック・仮想ディスパッチャ）と、時報の 4 段フォールバック・選択肢の OnChoiceSelectEx ルーティングを収録する
  - REG ハンドラは `function(act)`、RES は実在する関数群、シーン関数フォールバックは 200 応答、OnSecondChange の既定ハンドラの役割と上書き時の注意を実装どおりに書く
  - 導入と締めを書き下ろし、生成対象章の執筆制約を守る
  - 完了状態: 章が存在し、台帳の shiori-handlers 由来の行がすべてこの章を指している
  - _Requirements: 3.2, 3.3, 1.7_
  - _Boundary: ContentMigration（lua/shiori-events）_
  - _Depends: 1.3_

- [ ] 2.7 (P) pasta.toml リファレンス章を新設する
  - スキル pasta-toml の全節（分類表・最小テンプレート・フルリファレンステンプレート・予約注記・各セクション詳細）と `pasta_patterns` の自動読み込みを収録する
  - 既定値の出典はリポジトリ内パスを書かず「実装の既定値（SSOT）」と記す。検査対象 4 キーはキー名と `` `既定値` `` を同じ表行に置く
  - 予約注記へのアンカーリンクは見出しの slug（`package-予約注記`）に合わせて直す
  - 導入と締めを書き下ろし、生成対象章の執筆制約を守る
  - 完了状態: 章が存在し、4 キーの同一行表形式が満たされ、台帳の pasta-toml 由来の行がすべてこの章を指している
  - _Requirements: 4.1, 1.7_
  - _Boundary: ContentMigration（reference/pasta-toml）_
  - _Depends: 1.3_

- [ ] 2.8 (P) 生成対象外章の誤記訂正と導入章の定義整理を行う
  - 1.3 の grep 結果に従い、生成対象外章（lua/patterns の `function(req)`・`RES.ok_with` を含む）の誤記を実装どおりに訂正し、詳細は SHIORI イベント章へ誘導する
  - 導入章の「将来変更あり」の定義を「現行挙動だが将来変わり得る箇所の注記」に限定する
  - 入門章の hello-pasta 照合ブロックは変更しない
  - 完了状態: 1.3 で記録した生成対象外章の誤記がすべて訂正済みとして台帳に記録されている
  - _Requirements: 1.3, 1.6, 3.3_
  - _Boundary: ContentMigration（生成対象外章）_
  - _Depends: 1.3_

- [ ] 2.9 章分割を目次・リンク・リダイレクト・内容検査へ結線する
  - 旧 `lua/modules.md` を削除し、目次の公開モジュール API 行を一覧章＋子 7 章へ置換、SHIORI イベント章と pasta.toml リファレンス章を目次へ追加する
  - Lua 索引章・起動シーケンス章からのリンクと、分割後章から lua/patterns への相対リンクを張り替え、Lua 索引章の章一覧へ SHIORI イベント章を追加する
  - `book.toml` に旧 URL `lua/modules.html` から `modules/index.html` へのリダイレクトを 1 行追加する
  - コンテンツ検証の B（Lua 網羅）と D（ボイス）の走査ディレクトリに lua/modules を加える
  - 完了状態: 既存のリンク切れ検査・コンテンツ検証・`mdbook build` が緑で、ビルド成果物に旧 `lua/modules.html` が転送ページとして生成され転送先が実在する
  - _Requirements: 4.2, 1.7, 3.1_
  - _Depends: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_

- [ ] 2.10 既定値整合テストの読み先をマニュアル章へ付け替える
  - テストの読み先をスキルの pasta-toml からマニュアルの pasta.toml リファレンス章へ変え、doc コメントを更新する。照合規則と失敗メッセージは変えない
  - 実行前に `NoDefaultCurrentDirectoryInExePath` 環境変数を外す（LuaJIT ビルドの既知制約）
  - 完了状態: `cargo test -p pasta_lua --test loader` が緑。章の既定値を 1 つ変えると該当キー名付きで失敗することを一度確認して戻した。テストソースに `.claude/skills` のパスが無い
  - _Requirements: 4.3, 4.4, 4.5_
  - _Boundary: ConfigDefaultsTest_
  - _Depends: 2.7_

- [ ] 3. リンク検証・生成器・内容検査のツール化（P3）
- [ ] 3.1 drift-check をリンク検証器へ改名・縮小する
  - リンク切れ検出（book 内相対 `.md`・自リポ GitHub URL・トラバーサル）を現行挙動のまま残し、ドリフト検出・未マップ検出・マニフェスト解析・ハッシュ計算を削除する
  - リンク正規表現を export し、CommonMark 準拠のフェンス判定（3 個以上の `` ` ``／`~` で開き、同じ文字で開始以上の個数・情報文字列なしの行でのみ閉じる。行数は保つ）を追加して export する
  - 自己テストはリンク検証系ケースを非回帰として残し、フェンス判定（4 連フェンス内の 3 連行を含む）のケースを追加、ドリフト・未マップ・TOML のケースを削除する
  - 旧ファイルを import しているドリフトゲート検証スクリプトを同時に削除し、検証スクリプト自己テストから該当ブロックを削除する（改名で壊れる import を残さない）
  - 完了状態: 改名後の自己テストと検証スクリプト自己テストが緑で、実リポジトリに対する book 内リンク検査が exit 0
  - _Requirements: 8.1, 8.4_

- [ ] 3.2 スキル自己完結検査と見出し slug を追加する
  - 2 スキルの全 `.md` に対し、規則 a（スキル外への脱出・実在しないファイル）・b（見出し slug と `<a id>`／`<a name>` の和集合に対するアンカー検証）・c（禁止トークン `doc/spec`・`GRAMMAR.md`・`book/src`・`crates/` を HTML コメント含む全文で検出）・d（`SKILL.md` からリンクされていない references ファイル）を検査し、違反を種別付きで報告する
  - GitHub 方式の見出し slug 関数を実装し、design.md の例（`[package] 予約注記`・全角括弧入り・`set_scene_selector(...) / set_word_selector(...)`・重複見出しの付番）を自己テストで固定する
  - 早期確認として、`mdbook build` が出力した生成対象章の見出し id と slug 関数の結果を突き合わせる。食い違う見出しがあれば、マニュアル章（ContentMigration の領域）の見出しを両方式で同じ slug になる形へ直す境界横断の作業となるため、直した見出しへの book 内アンカー参照（variables の見出しを参照する first-ghost・lua/patterns 等）も同時に直す
  - CLI は違反ありで exit 1、予期しない例外で exit 2
  - 完了状態: 自己テストが a〜d の検出と許可ケース（https・実在アンカー・`#s6-6`）をすべて通し、生成対象章の slug と mdBook id の不一致が 0
  - _Requirements: 5.3, 6.1, 6.4, 6.5, 10.2, 2.4_

- [ ] 3.3 生成器の対応表・抽出・口調判定を実装する
  - 21 エントリの対応表と、章パスから出力名を導く規則（index は直近の親ディレクトリ名付き）を定義する
  - 口調マーカー集合をコンテンツ検証から移し、衝突 3 語（`くてよ`・`ですの`・`ますの`）だけ否定先読みにし、判定関数を export する
  - 本文抽出（LF 正規化・共有フェンス判定・フェンス外の最初と最後の `---`・H1 タイトル・締め以降の破棄）と、散文部（フェンス・表の行・インラインコードを除く）への口調判定を実装し、章欠落・構造違反・口調混入を章と行番号付きで失敗させる
  - 自己テストで抽出・構造違反・口調検出（4 連フェンス内の無視、「書かなくてよい」「ですので」の非検出を含む）と、実物の対応表が 21 エントリで出力名が重複しないことを確認する
  - 完了状態: 生成器の自己テストのうち抽出・対応表・口調判定のケースが緑
  - _Requirements: 5.1, 5.2, 5.8, 5.9, 1.7_
  - _Depends: 3.1_

- [ ] 3.4 生成器のリンク書き換え・ヘッダ・書き出しと照合モードを実装する
  - リンク書き換え（同一スキル宛ては兄弟ファイル名＋アンカー、非生成章・別スキル宛ては公開 URL の `.html`＋アンカー、絶対 URL と `#anchor` は不変、画像・非 `.md`・book/src 外は失敗）を実装する
  - 固定 2 行ヘッダ（機械判定用の 1 行目と、公開 URL・編集元の案内）を付けて全エントリをメモリ上で生成し終えてから書き出す。内容が LF 正規化後に同一なら書き換えない
  - 照合モードは LF 正規化して比較し、不一致（STALE）と孤立生成物（ORPHAN）を全件列挙して再生成コマンドと共に exit 1 で報告する
  - 自己テストで書き換えの各経路、LF／CRLF 入力のバイト一致、サンドボックスでの stale（章だけ変更・生成物だけ手編集）・CRLF 化だけは一致・orphan を確認する
  - 完了状態: 生成器の自己テストが全件緑で、書き出しモード直後の照合モードが exit 0 になることをサンドボックスで確認済み。実リポジトリの全 21 章に対し書き出さずに全生成を実行して生成エラーが出ない（P2 の章に残った口調・不正リンク・構造違反をここで検出し直す）
  - _Requirements: 5.3, 5.4, 5.5, 5.6, 5.7, 7.1, 7.2, 7.3, 7.4, 7.7_

- [ ] 3.5 コンテンツ検証から旧権威前提を除き、新検査を加える
  - doc/spec 権威リンク必須の検査・マニフェスト整合検査・ローカルの口調マーカー定義を削除し、対応表と口調判定を生成器から import する
  - book 全章に `doc/spec`・`GRAMMAR.md` の文字列が無いこと、対応表の全章が目次からリンクされていることの検査を追加し、ヘッダコメントの検証範囲説明を更新する
  - 同時に、文法章末の「権威的仕様」引用、grammar/index の doc/spec 案内、導入章の権威記述、外部リンク集の doc/spec リンク群を削除する
  - 完了状態: コンテンツ検証が exit 0 で、検証スクリプト自己テストの件数閾値（50 以上）を満たす
  - _Requirements: 1.5, 4.2, 8.3, 9.7, 1.7_
  - _Depends: 3.3_

- [ ] 3.6 ドリフト機構の残置物を撤去する
  - マニフェスト（manual-sources）を削除する。3.5 でマニフェスト整合検査を外すまではコンテンツ検証が読むため、必ず 3.5 の後に行う
  - tutorial-check とその自己テストの drift-check へのコメント言及を修正する
  - 完了状態: book/tools の全自己テストとコンテンツ検証が緑で、book/tools 配下にドリフト検出への参照が残っていない
  - _Requirements: 8.1, 2.4_

- [ ] 4. スキルの生成への切替（P4）
- [ ] 4.1 生成物を書き出し、旧名の手書きファイルを削除する
  - 書き出しモードで 21 ファイルを生成する（同名の旧手書き 6 ファイルは上書き）
  - 旧名 4 ファイル（grammar-model・call-spec・runtime-api・shiori-handlers）を削除する。旧ファイル固有の内容は台帳で移設済みであることを確認してから削除する
  - 完了状態: 鮮度照合が exit 0 で、両スキルの references に旧名ファイルが無い
  - _Requirements: 5.1, 10.2_

- [ ] 4.2 (P) pasta-ghost-authoring の SKILL.md を区分表つきに更新する
  - references 一覧表（ファイル・区分・生成元章の公開 URL・用途）を全ファイル分置き、前文に「生成ファイルは編集しない・生成ファイルが正」「本文の早見表は非規範の要約」「持ち出し先は references を丸ごと置換」を明記する
  - 旧 grammar-model への 11 件・call-spec への 2 件のリンクを移動先の生成ファイルへ張り替え、アンカーを生成ファイルの見出しに合わせる。食い違い表に当たる記述を訂正し、metadata.version をバンプする
  - 完了状態: リンク検証でこのスキルの SKILL.md に起因する違反が 0
  - _Requirements: 6.1, 6.4, 6.5, 6.6, 6.7_
  - _Boundary: SkillLayout（pasta-ghost-authoring/SKILL.md）_

- [ ] 4.3 (P) authoring-patterns から挙動の規範的説明を除く
  - 時報変数・シャッフル消費・チェイントーク等の挙動説明を削り、作例と生成ファイルへの参照に置き換える。作例・ファイル分割指針・自然言語→シーン変換指針は残す
  - SKILL.md から参照されている `<a id="s6-N">` アンカーは維持する
  - 完了状態: authoring-patterns に挙動の規範的定義が残っておらず、リンク検証でこのファイルに起因する違反が 0
  - _Requirements: 6.2, 6.5_
  - _Boundary: SkillLayout（pasta-ghost-authoring/references/authoring-patterns.md）_

- [ ] 4.4 (P) pasta-lua-coding の SKILL.md と手書き 3 ファイルを更新する
  - SKILL.md に区分表と前文 3 点を置き、internal-modules を「手書き（暫定）— 将来 pasta-runtime-internals-doc で移行予定」と明記する。book への相対リンクを references の起動シーケンスファイルへ、旧 runtime-api・shiori-handlers へのリンクを新ファイルへ張り替え、早見表の `function(req)` を `function(act)` に訂正し、metadata.version をバンプする
  - testing-lint のセレクタへのアンカーリンクを pasta-search へ張り替え、リポジトリ内パスのモック記述をモジュール名表記へ直す。internal-modules 末尾の旧名リンクを張り替える
  - 完了状態: リンク検証で pasta-lua-coding に起因する違反が 0
  - _Requirements: 3.4, 6.1, 6.3, 6.4, 6.5, 6.6, 6.7_
  - _Boundary: SkillLayout（pasta-lua-coding）_

- [ ] 4.5 スキル切替の統合確認を行う
  - 両スキルを通してリンク検証と鮮度照合を実行し、残った違反（スキル間で見落とした旧名リンク・アンカー等）を解消する
  - 完了状態: 鮮度照合とリンク検証がともに exit 0（両スキルが自己完結・区分漏れなし・旧名ファイルなし）
  - _Requirements: 6.1, 6.4, 6.5, 7.1, 10.2_
  - _Depends: 4.2, 4.3, 4.4_

- [ ] 5. CI と完了ゲートの再定義（P5）
- [ ] 5.1 (P) マニュアル CI に鮮度チェックとリンク検証を組み込む
  - 起動 paths（push・pull_request の両方）に 2 スキルのディレクトリを追加する
  - Setup Node 直後に鮮度照合ステップを置き、旧ドリフト／リンク切れステップをリンク検証へ置換し、ドリフトゲート検証ステップを削除し、ヘッダコメントを更新する
  - 完了状態: ワークフロー定義に drift の語が無く、鮮度照合が npm ci より前・deploy が build に依存する構成になっていることを YAML で確認済み
  - _Requirements: 7.1, 7.2, 7.5, 7.6, 8.2, 10.4_
  - _Boundary: ManualCI_

- [ ] 5.2 (P) 完了ゲートと保守手順を workflow.md・kiro-complete に反映する
  - DoD の Manual Sync Gate を「マニュアルとスキル生成物の同期」へ再定義する（発火条件: book・2 スキルへの変更、判定: 鮮度照合とリンク検証が exit 0、解消フロー、スキップ条件）。doc/spec・ハッシュ・ドリフトの語を除く
  - スキルドキュメント更新手順を「生成ファイルはマニュアル章を直して再生成、SKILL.md と手書きは直接更新」に書き換え、更新チェックリスト・最終タスクの整合チェックリスト・保守責任表・保守ルールを「マニュアル章＋スキル再生成」へ改め、`.agents/skills` を `.claude/skills` に直す
  - kiro-complete のステップ 4 と完了チェックリストを新ゲートの発火とコマンドへ置換する（判定本体は workflow.md を正とする構造を維持）
  - 完了状態: 両ファイルに doc/spec・drift・GRAMMAR.md の語が残らず、新ゲートのコマンド 2 つが記載されている
  - _Requirements: 7.8, 8.5, 9.5_
  - _Boundary: CompletionGate_

- [ ] 6. 将来仕様の brief 2 件とロードマップ申し送りを作成する（P7 の一部を撤去前へ前倒し）
  - 吸収元（doc/spec ch08・ch12）がまだ存在するうちに、属性セマンティクス（ファイルレベル属性・Call 属性フィルターを含む）と動的単語参照 `＠＄` の brief を起票する
  - roadmap の Phase 2 配下に「将来仕様（doc/spec 廃止時の申し送り）」小節を設け、B・R 項目とバグ候補を 1 項目 1 行のキー情報で記載する。roadmap 内の doc/spec・drift-check 記述も置換する
  - 完了状態: 仕分け表の B・R 全項目が brief か roadmap のキー行に存在し、バグ候補の行数と roadmap のバグ候補キー行数が一致する
  - _Requirements: 1.3, 1.4, 1.8_
  - _Depends: 1.2, 1.4_

- [ ] 7. 旧権威を撤去する（P6）
  - doc/spec ディレクトリを削除し、GRAMMAR.md を「マニュアルへ移った」告知 1 文と公開マニュアル文法章の URL のみにし、book/CONTENT-REVIEW.md を削除する
  - 削除前に台帳の全行が埋まっていること、6 の brief が起票済みであることを再確認する
  - 完了状態: doc/spec が存在せず、GRAMMAR.md が告知と URL のみで、リンク検証とコンテンツ検証が緑のまま
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 10.2_

- [ ] 8. 現行文書の参照修正（P7）
- [ ] 8.1 (P) steering の grammar.md を非規範の要約へ縮小する
  - 「このドキュメントの役割」（権威はマニュアル・食い違い時はマニュアルが正）・マーカー一覧・よくある間違い・IR 出力のみを残し、他の節を削る。残す節も食い違い表に照らして訂正する
  - 完了状態: grammar.md が「完全参照」を名乗らず、残す 4 節のみで構成されている
  - _Requirements: 9.8, 9.1, 9.2_
  - _Boundary: ReferenceRepair（steering/grammar.md）_

- [ ] 8.2 (P) steering・ルート文書・crates 内の参照を付け替える
  - tech・structure・product の drift-check／doc/spec／GRAMMAR.md 記述を生成方式とマニュアル権威へ置換し、tech の `.agents/skills` を直す
  - README・SOUL・OPTIMIZATION の該当行を削除またはマニュアルへ付け替え、`.agents/skills` を `.claude/skills` に直し、SOUL の衝突ルールを「マニュアルを優先し README・steering・スキル手書きを修正する」へ変更する
  - crates/pasta_lua の README の pasta-toml リンクをマニュアル章へ、syntax_test の GRAMMAR.md コメントをマニュアル章へ付け替える
  - 完了状態: 対象ファイルに doc/spec・GRAMMAR.md・drift-check を現存として案内する記述が無い
  - _Requirements: 9.1, 9.2, 9.3_
  - _Boundary: ReferenceRepair（steering・ルート文書・crates コメント）_

- [ ] 8.3 (P) book/AUTHORING.md を「権威と生成」へ書き換える
  - 題名・冒頭・第 4 節を「マニュアルが唯一の権威、スキル規範ファイルは生成物で編集元はマニュアル、対象章一覧は生成器の対応表が正、再生成コマンド」へ書き換える
  - 生成対象章の規約（本文に口調・コラムを置かない、区切りは最初と最後の `---`、章外相対リンクは公開 URL になる、画像・非 `.md` 相対リンク・参照形式リンク・リポジトリ内パスの禁止、pasta.toml の同一行表形式）を追記する
  - サンプル A の権威的仕様引用、第 5 節の doc/spec チェック、流用元一覧の doc/spec・GRAMMAR.md・スキル起草元の行を削除し、「生成対象章を変更したら再生成してコミット」を追加する
  - 完了状態: AUTHORING.md にスキルを起草元とする記述と doc/spec の語が無い
  - _Requirements: 9.4, 9.1, 9.2_
  - _Boundary: ReferenceRepair（book/AUTHORING.md）_

- [ ] 8.4 (P) review-improvement-loop の今後の指示を参照修正する
  - 文書整合タスクの確認対象・ツールのテストコマンド表・未完了セルと未完了タスクの記述のみを修正し、完了記録と reports は残す
  - 完了状態: 同 spec の残存する doc/spec・GRAMMAR.md・drift-check の行がすべて完了記録か reports である
  - _Requirements: 9.1, 9.2, 9.6_
  - _Boundary: ReferenceRepair（review-improvement-loop）_

- [ ] 9. 全体検証（P8）
- [ ] 9.1 マニュアルとスキルのツール検証を通す
  - 鮮度照合・リンク検証・コンテンツ検証・book/tools の全自己テストを実行する
  - manual.yml の build ジョブのステップ（鮮度照合→依存導入→ビルドと後処理→リンク検証→チュートリアル・静的・検索・内容の各検証→自己テスト）をローカルで順に再現する
  - 完了状態: 全コマンドが exit 0 で、ビルド成果物に新章 10 個が含まれ目次から到達でき、旧 `lua/modules.html` が転送ページとして存在する
  - _Requirements: 7.1, 7.7, 8.2, 8.4, 10.4, 4.2_

- [ ] 9.2 Rust テストスイート全体を通す
  - `NoDefaultCurrentDirectoryInExePath` を外したうえで `cargo test --all` を実行する
  - 完了状態: 全テストが成功し、crates 配下の差分がテストの読み先パス・コメント・README リンクのみである
  - _Requirements: 10.3, 10.5, 4.3_

- [ ] 9.3 内容の網羅と参照修正の網羅を確認する
  - 台帳本体・付録・食い違い記録・仕分け表の全行が埋まっていること、マニュアルに未実装機能の節が残っていないことを確認する
  - 両 SKILL.md の区分表が全 references を載せ、前文 3 点と internal-modules の暫定注記があることを確認する
  - design.md の網羅 grep を実行し、残存行がすべて許容理由（廃止済みの説明・歴史的記録・スキル探索順の定義）に当たることを確認する
  - 完了状態: 上記がすべて満たされ、変更がすべて本ブランチ上にあり単一 PR で統合できる状態である
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 1.8, 2.1, 2.4, 3.1, 3.2, 4.1, 6.1, 6.2, 6.3, 8.1, 8.5, 9.1, 9.2, 9.6, 10.1, 10.2, 10.6_

## Implementation Notes
- 1.1: 台帳は design の食い違い表に無い実装差も備考に記録した（アクタースコープの `lua` コードブロックは実際に生成・A1 で到達する＝design #14 の「処理に反映されない」と食い違う、bool リテラルなし、`\]` 非対応、動的 Call は `tostring(expr)`、REG ハンドラの戻り値 `RES.ok` 二重包み疑い、`default_surface` 未参照など）。1.3・1.4・2.x は台帳の備考を実装照合の起点にすること。
- 1.1: authoring-patterns.md L144 の閉じフェンスが ```` ```lua ```` になっており、以降のフェンスが CommonMark 上ずれている（§6.5 見出しがコード扱い）。4.3 で手書きファイルを直す際に修正する。
- 1.2: 付録のバグ候補は 8 件（U06,U08,U12,U18,U19,U20,U21,U22）。バグ候補はマニュアルに書かない（10.5）。2.x は付録の「収録先」行（U11 の算術・数値変換、U19 備考のアクター名規則、U07 `＄＄` など）を収録し、U08 により本体 L148 の `\` 収録は取り消し。大タスク 6 で roadmap にバグ候補のキー行を置く（2.4 後に 16 件: U06,U08,U12,U18〜U30）。
- main 取り込み（91a00e11・actor-surface-restore #45）: `book/src/grammar/actor-dictionary.md` に節「同一スポット共有時の外見の復旧」が追加（first-ghost.md からこの見出しへアンカーリンクあり・見出し維持必須）。スキル pasta-toml.md の `default_surface` は `surface`/`dressup` に置換済み、internal-modules に `STORE.appearance` 追加。台帳 1.1 の `default_surface` 行はこの変更後の内容で照合し直すこと（1.3）。
- 1.3: 食い違い grep 記録（D01〜D18・X01〜X22）が 2.x／4.x の訂正対象リスト。各訂正タスクは自分の訂正先に割り当てられたヒットをすべて直し、台帳の該当行に「訂正済み」を記す。実装照合で判明: pasta.toml `[lua] libs` はロード時に読まれない（`RuntimeConfig::new()` のみ）、BOM 付き .pasta はパースエラー、`OnNotifyCallbackResponse` は無く `OnPastaCallBack{N}` を `CALLBACK.try_route` が REG より先に処理、OnChoiceSelectEx はローカルシーンのみ探索（グローバルへフォールバックしない）。
- 1.3→1.4: 新規バグ候補の可能性（X19 REG 戻り値の RES.ok 二重包み・D07 改行入り `"` 文字列でロード失敗・D07 単語値 `""`/`「」` が空にならない・X18 `[lua]` 未使用・D03 OnChoiceSelectEx のローカル限定）の付録追記（U23 以降）は 1.4 が担当して確定する。
- 1.4: design #14 の前提は実装照合で訂正済み。アクタースコープのコードブロックは情報文字列が小文字 `lua` ちょうどのときだけ出力され、ACTOR 関数は act ではなくアクタープロキシを引数に呼ばれ、`＠名前` のアクター単語参照からのみ到達する（`＠名前（）`・`＄x＝＠名前` では到達しない）。2.3 はこの現行挙動を書く。属性は記録されるが検索では使われない（search context の filters は常に空）。仕分け表の B1・B2（brief）と R1〜R6（roadmap キー行）とキー行文面は大タスク 6 が使う。
- 2.1: 章内の ```pasta 例は実パーサで通ること（スクラッチの `pchk` で `pasta_dsl::parse_str` を通して確認した）。行末 `#` 注記はアクション行・引用なし単語値では台詞／値になるため例に付けない。図示用断片は ```text にする。block-structure#選択肢行から SHIORI イベント章へのリンクは未設置（2.9 で lua/shiori-events.md 作成後に張る）。grammar/call-jump.md L77〜L99 の例はパース不能（コンテンツ行の後に Lua ブロック、その後に `＞`）→ 2.2 で直す。
- 2.2: 実装照合で判明: 別グローバルシーンへの Call から戻ると `act.current_scene` が復元されない（U28 として付録に追加・バグ候補）。マニュアルは「実行中のグローバルシーン」の通常挙動だけを書く。OnBoot で始めたチェイントークの残りは次の OnTalk で出力される（台帳本体 ga call-spec L68 の「後半が出ない」は誤り）。ローカル単語参照（2.3）も U28 の影響を受けうるが、マニュアルには通常挙動だけを書く。
- 2.3: grammar/variables.md は永続化の説明で `../lua/modules.md` にリンクしている。2.9（旧 lua/modules.md 削除時）に `../lua/modules/pasta-persistence.md` へ張り替える（detectBrokenLinks が検出する）。
- 2.4: lua/modules/index.md は 2.5 の章（@pasta_sakura_script・@enc・@pasta_log・mlua-stdlib）を素の文字列で載せている。2.5 で表の章セルと「章「mlua-stdlib 統合モジュール」」を各章へのリンクにする。reference/pasta-toml.md へのリンク（pasta-persistence.md#pasta.toml 設定、pasta-config.md の `[ghost]`/`[actor]` 段落から）は 2.7 で章ができた後に張る（2.7 の担当）。セレクタの整数値不使用（U29）・記号入りシーン名（U30）はバグ候補で、マニュアルには書かない。
