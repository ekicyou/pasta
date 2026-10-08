# Implementation Plan

本計画は design.md（2026-10-08 の設計ディスカッションで確定）に従う。番号付きのタスク 1〜6 が `/kiro-impl` の対象で、1 つの PR（タグのコミットに追跡解除・`Cargo.lock`・ワークフロー・スクリプトがそろう形。design.md「Migration Strategy」）として main へ入れる。末尾の「マージ後の工程」は、その PR のマージ後に別セッション「Entra ID の登録」・ユーザーと調停して進める工程で、`/kiro-impl` では実行しない。

手元での検証に共通する前提:
- cargo を使う検証（ビルド・テスト・`release.ps1`）の前に、開発機の環境変数 `NoDefaultCurrentDirectoryInExePath` を外す（外さないと LuaJIT のビルドが exit 101 で失敗する）。
- 「生成物が `git status` に出ない」の検査は、無視の対象のパス（`release/`・サンプルゴーストの DLL・第三者ライセンス表示・`scripts/`）に未追跡・変更のファイルが無いことを指す。追跡中のシェルの画像（`release.ps1` の画像生成の段が作り直す）と `sample.generated.lua`（`cargo test` が改行だけ書き換える）の差分は既知のものとして `git checkout --` で戻し、コミットに混ぜない。

- [x] 1. 基盤: 成果物の追跡解除・依存の固定・検証ツール
- [x] 1.1 ビルドした成果物の git 追跡を解除し、無視の設定に加える
  - 解除の前に、追跡中の `hello-pasta.nar` のエントリ一覧（パスとサイズ）を research.md に「nar の基準（追跡解除前）」の節として記録する（2.1 で作り直した nar と比べる基準。3.3 の「今と同じ」の証拠）
  - `release/` 配下と、サンプルゴーストの DLL・第三者ライセンス表示・`scripts/` の写しを追跡から外し、無視の設定にコメント付きで加える
  - 手書きの正本（`descript.txt`・`pasta.toml`・`dic/`・`install.txt` など）とシェルの画像は追跡を続け、無視の対象に含めない
  - 完了の状態: research.md に nar の基準があり、`git ls-files` に対象の生成物が出ず、正本とシェルの画像は出る。既存の `release.ps1` を手元で実行しても、無視の対象のパスに未追跡・変更のファイルが出ない
  - _Requirements: 10.1, 10.2, 10.4, 10.5, 10.6_

- [x] 1.2 `Cargo.lock` を追跡し、依存の解決を固定する
  - 無視の設定から `Cargo.lock` の行（コメント含む）を外し、今の解決結果をコミットする
  - 完了の状態: `Cargo.lock` が追跡され、`cargo build --workspace --locked` と `cargo test --all --locked` が lock を書き換えずに通る
  - _Requirements: 3.9, 3.10_

- [x] 1.3 ワークフローの構文検査ツール actionlint を手元で使えるようにする
  - 版を固定した actionlint を、リポジトリを汚さない場所（ユーザーのツール置き場）へ導入し、導入の方法と版をワークフローの検証手順として research.md に 1 行記録する
  - 完了の状態: 手元で actionlint の版を表示でき、既存の `build.yml`・`manual.yml` を検査して結果が出る
  - _Requirements: 2.6_

- [x] 2. 配布物のビルドの調整
- [x] 2.1 (P) 手元と CI で同じ `pasta.dll.zip` を作るよう `release.ps1` を調整する
  - nar の作成の後に、x86 リリースビルドの DLL と第三者ライセンス表示の 2 エントリだけを入れた `pasta.dll.zip` を `release/` に作る段を足し、段の番号を 7 段にそろえる
  - zip の中身が 2 エントリだけであることをスクリプト自身が検査し、違えば止める
  - 最後の案内を「成果物はコミットしない。公開はリリースタグの push で CI が行う」に書き換え、`gh release create` の例と Windows PowerShell 前提の使い方の記述を外す。画像生成の段と既存の引数は変えない
  - 完了の状態: 手元で実行すると `release/pasta.dll.zip`（2 エントリ）と `release/hello-pasta.nar` ができ、nar のエントリ一覧が 1.1 で保存した基準と一致する
  - _Requirements: 3.2, 3.3, 12.3_
  - _Boundary: release.ps1_
  - _Depends: 1.1_

- [x] 2.2 (P) VSIX の WASM をリリースビルドにし、PowerShell 7 で動かす
  - VSCode 拡張の WASM ビルドのコマンドを、pwsh 経由でリリースビルドの引数を渡す形に変える（ビルドスクリプト本体は変えない）
  - 拡張の開発手順の説明に、手元で pwsh 7・wasm-pack・cargo-about が要ることを 1 行足す
  - 完了の状態: `npm run package` が非対話で成功し、できた VSIX にリリースビルドの WASM と第三者ライセンス表示が入り、ファイル名の版が `package.json` の版と一致する
  - _Requirements: 3.4_
  - _Boundary: VSIX ビルド（package.json）_

- [x] 3. リリース CI の補助スクリプト（pwsh 7・引数と環境変数だけで動き、手元で試せる）
- [x] 3.1 (P) タグと版の検査スクリプトを作る
  - タグの形（`v` + 数字 3 つ）、ワークスペースの版、拡張の `package.json` の版、タグのコミットの main からの到達性を検査し、版（`X.Y.Z`）を出力する
  - 失敗時は、どの検査で失敗したかと食い違う両方の値、「どの公開先にも公開していない」を summary（未設定なら標準出力）に書いて非 0 で終える
  - 完了の状態: v0.3.7 のコミットで成功して `version=0.3.7` を出し、`v9.9.9`（版不一致）・`v0.3`・`v1.0.0-rc.1`（形式）・main から到達できない SHA でそれぞれ失敗理由を示して失敗する
  - _Requirements: 1.2, 2.2, 2.3, 2.4, 8.3_
  - _Boundary: verify-tag.ps1_

- [x] 3.2 (P) 1 クレートの公開済み判定と公開のスクリプトを作る
  - 公開の直前に crates.io API（User-Agent 付き）で、クレート自体の有無と版の有無を問い合わせる。クレートが無ければ「初回は手で公開する」を案内して失敗、版があれば飛ばす
  - 依存先の同じ版がスパース索引に現れるまで待ち、`cargo publish --locked` を行う。索引反映待ちの時間切れは成功とみなし、依存の未解決による失敗は間隔を空けて再試行する
  - 問い合わせが 200・404 以外なら公開済みと見なさず失敗する。yank・`--no-verify` は使わない。トークンは環境変数で受け取り、`-DryRun` では判定だけを行う
  - 検証用に、問い合わせ先の API の基点を差し替える省略可能な引数（例: `-ApiBase`。既定は crates.io）を持たせる
  - status 契約どおり `status` と（失敗時）`reason`（publish / transient / not-registered）を書いてから終える
  - 完了の状態: `-DryRun` で v0.3.7 の 5 クレートが `skipped`、存在しないクレート名で `failed`・`reason=not-registered`、到達できない API 先の擬似で `failed`・`reason=transient` になる
  - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6, 7.2, 7.3, 7.4, 7.6, 9.6_
  - _Boundary: publish-crate.ps1_

- [x] 3.3 (P) Marketplace の公開済み判定と公開のスクリプトを作る
  - `vsce show` の版一覧で公開済みを判定し、無ければ Azure の資格情報（`--azure-credential`）で VSIX を公開し、公開後に版の存在を確かめる。重複公開の保険に `--skip-duplicate` を付ける
  - `show` の失敗は公開済みと見なさず失敗する。PAT を参照せず、unpublish をしない。認証エラーは `reason=auth` に分類する
  - 手元の検証の前提: `editors/vscode` で `npm ci` を済ませ、lock の版の vsce を使える状態にする
  - 完了の状態: `-DryRun` で v0.3.7 が `skipped` になり、`VSCE_PAT` を設定していない環境でもその結果になる
  - _Requirements: 5.1, 5.2, 5.3, 5.5, 7.2, 7.3, 7.4, 7.6, 9.6_
  - _Boundary: publish-vsix.ps1_

- [x] 3.4 (P) リリースノートの生成スクリプトを作る
  - 前のリリースタグ（形の合うタグのうち版で並べて直前のもの。無ければ全履歴）からタグまでの、マージを除くコミットを集める
  - 6 種の見出しに分類し、`spec` スコープを除き、空の見出しを出さない。6 種以外の type と Conventional Commits でない件名は Maintenance に入れる。末尾に Full Changelog のリンクを付ける（前のタグが無ければ省く）
  - 完了の状態: `-Tag v0.3.7` で前のタグが v0.3.6 に決まり、生成したノートの見出しと件数が v0.3.7 の Release のノートと説明のつく形で一致する。前のタグが無い場合の分岐も擬似タグで確かめる
  - _Requirements: 6.6, 6.7, 6.8, 6.9_
  - _Boundary: release-notes.ps1_

- [x] 3.5 (P) GitHub Release の作成スクリプトを作る
  - 3 つの配布物がそろっていることを先に検査し、Release を検索して、無ければ下書き → 添付 → 公開、下書きなら未完了の添付を整えて足りないものを添付して公開、公開済みで添付がそろっていれば飛ばす
  - 公開済みで添付が足りなければ足りないものだけを添付し、拒否（Immutable）されたら `reason=immutable` で失敗する。公開済みの添付を削除・上書きせず、タグを作らない
  - 題名を `pasta vX.Y.Z` とし、`status`・`reason`・`url` を出力する
  - 完了の状態: 一時ディレクトリに置いた中身の無いファイル（`pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-0.3.7.vsix`）を `-AssetDir` に渡した `-DryRun` で、v0.3.7 について `skipped`（または足りない添付の「添付する」の表示）になり、URL が出力される。ファイルが 1 つ欠けると検査で失敗する
  - _Requirements: 6.2, 6.3, 6.4, 6.5, 7.2, 7.3, 7.6, 8.2_
  - _Boundary: github-release.ps1_

- [x] 4. ワークフロー
- [x] 4.1 (P) `build.yml` を reusable workflow として呼べるようにする
  - 起動条件に入力なしの `workflow_call` を足すだけにし、job・step・matrix・artifact 名・検査内容は変えない
  - 完了の状態: `actionlint` が通り、差分が `on:` の 1 項目だけで、push・PR・手動の起動条件が残っている（PR 上で従来どおり起動することは、実装 PR を作ったときに確かめる。C-1 の前提）
  - _Requirements: 2.5, 2.6, 2.8_
  - _Boundary: build.yml_

- [x] 4.2 (P) 公開しないセットアップ確認ワークフローを作る
  - 手動起動だけで動き、確認用 environment `release-setup-check` に属し、OIDC だけの権限で Azure にログインする（変数はリポジトリ variables の 3 つ）
  - マネージド ID の Marketplace の profile ID と表示名を job summary にだけ書き、publisher のメンバー確認（`verify-pat --azure-credential`。未追加でも続行）と拡張の存在確認の結果、名前の対応表（ワークフロー名・environment 名・リポジトリ・5 クレート）を summary に書く
  - crates.io のトークン交換は行わず、どの公開先にも公開しない
  - 完了の状態: `actionlint` が通り、定義に公開のコマンド・crates.io の認証アクションが含まれず、environment 名・変数名が design.md「認証名の契約」と一致する
  - _Requirements: 1.6, 11.5, 11.7_
  - _Boundary: release-setup-check.yml_

- [x] 4.3 リリース CI の起動・関門・ビルドの job を作る
  - リリースタグ（数字 3 つの形）の push だけを起動条件にし、既定の権限を読み取りだけにし、タグごとの同時実行を抑止する。ツールの版（wasm-pack・cargo-about・Node）を `env` にまとめて固定する
  - verify（ubuntu）は 3.1 のスクリプトで版を job outputs に出し、gate は `needs: verify` で `build.yml` を呼び、build（windows、`needs: [verify, gate]`。版を読むため verify も並べる）は日本語ロケール・ツール導入の後に `release.ps1` と VSIX のビルドを配布物名の step で行う
  - build は 3 つの配布物の中身と版、`Cargo.lock` が変わっていないことを検査してから、artifact `release-assets`（90 日・ファイルが無ければエラー）に保存する。job ごとのタイムアウトを設ける
  - 完了の状態: `actionlint` が通り、タグのフィルターが `v1.2.3` に合い `v1.2`・`v1.0.0-rc.1`・`test-1`・ブランチ push・PR に合わず、verify → gate → build の `needs` の連鎖で関門の失敗が build に進まず、build が `needs.verify.outputs.version` を参照できる定義になっている
  - _Depends: 1.3, 2.1, 2.2, 3.1, 4.1_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.7, 3.1, 3.5, 3.6, 3.7, 3.8, 10.6_

- [x] 4.4 crates.io と Marketplace の公開 job を作る
  - publish-crates（windows）と publish-vsce（ubuntu）を `needs: [verify, build]`（版を読むため verify も並べる）で互いを待たずに並べ、どちらも environment `release` に属させ、OIDC の権限をこの 2 job だけに与える
  - publish-crates は依存順の 5 クレートそれぞれで Trusted Publishing のトークンを取り直してから 3.2 のスクリプトを呼ぶ。auth step の失敗は `reason=auth` として記録する。2 回目以降の auth が拒否されたときの落とし先（1 回取得 + 再実行）を定義の冒頭のコメントに書き、クレートごとの所要時間を summary に残す
  - publish-vsce は配布物を受け取り、拡張の lock で vsce を固定し、Azure にログインしてから 3.3 のスクリプトを呼ぶ
  - 両 job の末尾に常に動く集約 step を置き、公開先ごとの `status`・`reason` をその job 自身の summary に書き、job outputs として公開する。`pasta_sample_ghost`・`pasta_lsp` の step は持たない
  - 完了の状態: `actionlint` が通り、`id-token: write` を持つ job が publish-crates・publish-vsce だけで、両者に互いの `needs` が無く、environment 名・変数名が「認証名の契約」と一致する
  - _Depends: 3.2, 3.3_
  - _Requirements: 4.1, 4.4, 4.7, 5.4, 5.5, 5.6, 7.1, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

- [x] 4.5 GitHub Release と結果の報告の job を作る
  - github-release（ubuntu）は `needs: [verify, build, publish-crates]` で待ち（Marketplace は待たない。verify は版を読むため）、書き込み権限をこの job だけに与え、3.4 でノートを作って 3.5 で Release を作る。末尾の集約 step で自身の summary に結果と URL を書く
  - report（ubuntu）は常に動き、各 job の outputs と結果から公開先ごと（クレートごと）の 4 状態と理由・Release の URL・関門で失敗した job・「どの公開先にも公開していない」を 1 枚の表にする。outputs が空のものは推論せず「job の summary を参照」と出し、report 自体は成功で終える
  - 完了の状態: `actionlint` が通り、`contents: write` を持つ job が github-release だけで、report が全 job を `needs` に持ち `if: always()` で動く定義になっている
  - _Depends: 3.4, 3.5_
  - _Requirements: 6.1, 6.10, 7.1, 7.5, 8.1, 8.2, 8.3_

- [x] 5. 手順書とリリース手順の文書
- [x] 5.1 (P) 一回限りのセットアップの手順書を作る
  - 節の順を実施の順（design.md の手順書の構成 1〜12）にそろえ、ワークフローの外で 1 度だけ行うこと、ID の値をリポジトリに書かず variables に置くことを冒頭に書く
  - Azure（従量課金・無料試用版を使わない理由・予算アラート・マネージド ID・サービスプリンシパルを採らない理由）、フェデレーション資格情報 2 件、GitHub の environment 2 つと variables、main へのマージ後の確認ワークフローの実行と Members 追加、crates.io の Trusted Publisher ×5、初回リリース、初回成功後の必須手順（`trustpub_only`・2 つのトークンの失効・緊急時の戻し方）、新しいクレートの初回公開、auth の取り直しが拒否されたときの落とし先、名前の対応表を載せる
  - 完了の状態: 手順書の名前の対応表が design.md「認証名の契約」と一字一句一致し、要件 11 の 10 項目それぞれに対応する節がある
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 11.9, 11.10_
  - _Boundary: release-ci-setup.md_

- [x] 5.2 (P) サンプルゴーストのリリース手順書を新しい流れに書き換える
  - 「版を上げたコミットを main に入れる → リリースタグを push → Actions の結果を確かめる → 失敗したら失敗した job を再実行」の流れにし、前提から `gh`・`CARGO_REGISTRY_TOKEN`・`VSCE_PAT` を外す
  - 回復の手順（一時的な失敗は再実行、定義の不具合は何も公開していなければタグの付け直し・公開途中なら版を上げて出し直す）を載せ、手元の `release.ps1` は動作確認用で成果物はコミットしないことを書く。`gh release create` の手順と「タグが既に存在する」のトラブルシューティングを削る
  - 完了の状態: 手順書に手元での公開・PAT の設定の記述が残っておらず、回復の 2 つの手順が載っている
  - _Requirements: 7.5, 12.1, 12.3, 12.4, 12.6_
  - _Boundary: RELEASE.md_

- [x] 5.3 (P) スキル・README・steering のリリース関連の記述をそろえる
  - pasta-check スキルの「リリース後の手順」をリリース CI が Release を作る前提に変え、`release.bat` の位置を直す
  - サンプルゴーストの README に、生成物（DLL・ライセンス表示・`scripts/`）はコミット対象でなく `release/` は無視されることを書く
  - steering の CI/CD の記述に `release.yml`（タグ契機・OIDC・成果物）を足し、`.github/` の構成と `release.bat` の位置の記述を直す。`release-workflow` spec の本体には触れない
  - 完了の状態: 3 か所の文書に `CARGO_REGISTRY_TOKEN`・`VSCE_PAT` を開発機に置く前提と、生成物をコミットする前提が残っておらず、`.kiro/specs/release-workflow/` に差分が無い
  - _Requirements: 12.2, 12.3, 12.4, 12.5_
  - _Boundary: リリース手順の文書群_

- [ ] 6. 統合の検証とマージ前の調停
- [ ] 6.1 クリーンなチェックアウトで、追跡解除後もビルド・テスト・配布物の作成が通ることを確かめる
  - 別の作業ツリー（クリーンなチェックアウト）で `cargo test --all` と clippy（x86・x64 の `-D warnings`）を通す
  - 同じ作業ツリーで `release.ps1` と `npm run package` を実行し、`git status` に生成物が出ず `Cargo.lock` が変わらないことを確かめる
  - 3 つのワークフローを `actionlint` でまとめて検査する
  - 完了の状態: 上記がすべて成功し、結果（コマンドと終了コード）を research.md の「実装時の検証の記録」の節に残している
  - _Depends: 1.1, 1.2, 2.1, 2.2, 4.2, 4.3, 4.4, 4.5_
  - _Requirements: 3.9, 3.10, 10.2, 10.3_

- [ ] 6.2 名前の契約を実装・手順書・相手セッションの進捗と突き合わせる
  - ワークフロー 2 本・手順書・design.md「認証名の契約」の間で、ワークフローのファイル名・environment 名・変数名・リポジトリ・5 クレートの並びが一致することを確かめる
  - 「Entra ID の登録」セッションからの返事を確かめ、届いていれば research.md「一回限りのセットアップの進み具合」に名前と完了状況だけを記録する（ID 値は書かない）。名前や方式の変更の提案があれば、契約と手順書を先に更新してから返答する
  - 完了の状態: 不一致が 0 件で、research.md の進み具合が最新の返事を反映し、design.md「セッション間の分担と調停」の表の工程 1・2 の状態が分かる
  - _Depends: 4.2, 4.4, 5.1_
  - _Requirements: 9.5, 11.1, 11.6_

## マージ後の工程（セッション間の調停。`/kiro-impl` の対象外）

上の 1〜6 を含む PR が main に入った後、design.md「セッション間の分担と調停」の表に従って進める。連絡はセッション間メッセージで行い、ID の値は送らず記録しない。各工程を終えたら research.md「一回限りのセットアップの進み具合」に記録する。期限: 2026-12-01（Azure DevOps の global PAT の廃止）より前に、工程 C-4 で Marketplace へ 1 度公開できていること。

- [ ] C-1 「Entra ID の登録」セッションへ、`release-setup-check.yml` を実行できるようになったことを連絡する
  - 前提: 実装 PR 上で `build.yml` が従来どおり起動したこと（`workflow_call` の追加の影響が無いこと）、実装 PR の main へのマージ、表の工程 1（フェデレーション資格情報 2 件）・工程 2（environment と variables）の完了の返事
  - 完了の状態: 連絡を送り、research.md に送付日を記録している
- [ ] C-2 profile ID の取得と Marketplace の Members への追加の完了を受け取る
  - 前提: C-1。相手セッションが setup-check を実行し、Members に Contributor で追加し、再実行で `verify-pat` が通る（表の工程 4・5）
  - 完了の状態: 相手セッションから完了の返事を受け、research.md に記録している
- [ ] C-3 crates.io の Trusted Publisher ×5 の設定の完了を確かめる
  - 前提: なし（初回リリース前ならいつでも可。表の工程 6）。ユーザーへ手順書の該当節を案内する
  - 完了の状態: 5 クレートの設定値が名前の対応表と一致することをユーザーが確かめ、research.md に記録している
- [ ] C-4 初回のリリースを行う
  - 前提: C-2・C-3（表の工程 1・2・5・6 の完了）。版の決定と bump は `release-workflow` の手順で行い、リリースタグを push する
  - publish-crates の 2 回目以降の auth が拒否されたら、手順書の落とし先に従う。クレートごとの所要時間を確かめる
  - 完了の状態: 3 公開先が `published` になり、report に表と Release の URL が出て、同じ run の再実行ですべて `skipped` になることを確かめている
- [ ] C-5 初回リリースの結果を「Entra ID の登録」セッションへ連絡する
  - 前提: C-4
  - 完了の状態: 結果（Marketplace の Entra ID 経路での公開の成否）を送り、research.md に記録している
- [ ] C-6 初回成功後の必須手順の完了を確かめる
  - 前提: C-4 の成功。ユーザーが 5 クレートの「Trusted Publishing のみ」を有効にし、`CARGO_REGISTRY_TOKEN` と `VSCE_PAT` を失効させる（表の工程 8）
  - 完了の状態: 3 つの手順の完了を research.md に記録している

## Implementation Notes
- 1.1: nar の基準（research.md）の `pasta.toml` は 2639 バイトだが、基準の nar の後に `f245ab1f`（#59）で正本が変わったため、作り直すと 2583 バイトになる。2.1 の比較ではパスの集合を比べ、この差は既知として扱う。
- 1.2: lock はルートの `Cargo.lock` 1 つだけ（VSIX の WASM の `crates/pasta_lsp` もワークスペース内）。wasm-pack と `release.ps1` は `--locked` を付けないので、build job の `git diff --exit-code -- Cargo.lock` が lock 不変の唯一の検査になる。
- 1.3: actionlint 1.7.12（winget・ユーザー領域）。検証は引数なしの `actionlint`（PowerShell では `*.yml` が展開されず exit 3）。shellcheck は無いので `run:` の中身はシェル検査されない（仕様の要求外）。現在のシェルの PATH に無ければ `%LOCALAPPDATA%\Microsoft\WinGet\Packages\rhysd.actionlint_Microsoft.Winget.Source_8wekyb3d8bbwe\actionlint.exe` を直接呼ぶ。
- 2.1: `release.ps1` は 7 段になり、日本語の案内を含むため UTF-8（BOM 付き）にした（ルートの `release.bat` が powershell.exe 5.1 で呼ぶので BOM が要る）。`release.bat` のコメント「4-6」は古いままなので、5.3 で `release.bat` に触れるときに 7 段に合わせて直す。
- 2.2: CI の build job は `npm ci` の後に `npm run package` を呼べばよい（build:wasm は pwsh 7 経由・`-Release`）。`scripts/build-wasm.bat` と `build-wasm.ps1` の Usage コメントは今も `powershell` 前提だが、設計で変更不要とした範囲。
- 3.x: 補助スクリプトは `#Requires -Version 7`・StrictMode・BOM なし UTF-8。出力は `$GITHUB_OUTPUT`／`$GITHUB_STEP_SUMMARY`（未設定なら標準出力）へ書く関数をスクリプトごとに持つ（共通モジュールはどのタスクにも無いので作らない）。タグの形は `-cmatch` で大文字小文字を区別し ASCII 数字と `\z` で判定する（`-match` は V0.3.7 を通す）。検証ドライバーは scratchpad に置きリポジトリに入れない。
- 3.2: `publish-crate.ps1 -Crate -Version [-DependsOn] [-DryRun] [-ApiBase]`。索引待ちの上限 5 分は依存先 1 つごと（pasta_lua は最悪 10 分）、公開後の確認も最大 5 分。トークン（30 分）はクレートごとに auth の直後に取り直すので収まる。summary 行に所要秒を書く。試験は scratchpad\t32 の偽 cargo（PATH 先頭）と偽 API で行い、本物の公開はしない。
- 3.3: `publish-vsix.ps1 -VsixPath -Version -Extension [-VsceCommand] [-DryRun]`。vsce 3.7.1 は `--pat` の既定値が `VSCE_PAT` で `--azure-credential` より優先されるため、スクリプトは値を読まずに `Env:VSCE_PAT` を消す。vsce は `editors/vscode` で `npx --no-install vsce`（publish-vsce job は事前に `npm ci --ignore-scripts`）。`-VsixPath` は呼び出し元基準で解決する。公開後の確認は最大 10 分（job の 20 分に収まる）。
- 3.4: `release-notes.ps1 -Tag -Repo [-OutFile] [-WorkspaceRoot]`。範囲は `<前のタグ>..<Tag>`（タグのコミット自身を含む）、項目は type 付きの件名全体、除外は scope `spec` だけ（type `spec` は Maintenance）。github-release job の checkout は `fetch-depth: 0` と `fetch-tags: true` が要る（4.5 で確かめる）。
- 3.5: `github-release.ps1 -Tag -NotesFile -AssetDir [-Title] [-DryRun]`。リポジトリは gh の既定（checkout の remote）で決まり、認証は `GH_TOKEN`。`gh release view <tag>` は下書きもタグ名で見つける。Immutable はエラー文の一致で判定（拒否されれば失敗はする）。
- 4.2: 開発機の環境には `VSCE_PAT` が入っており、`vsce <cmd> --help` は `--pat` の既定値として PAT の値を表示する。手元で vsce を叩くときは必ず `VSCE_PAT` を外す（4.2 の実装中に会話記録へ値が出た。C-6 で失効させる PAT）。
- 4.3: release.yml は env に `WASM_PACK_VERSION`・`CARGO_ABOUT_VERSION`・`NODE_VERSION`・`RELEASE_ENVIRONMENT`（表示用。`jobs.<id>.environment` は `env` を参照できないので 4.4 では `environment: release` を直書きする）。run には `${{ }}` を埋め込まず既定の環境変数で渡す。build の artifact `release-assets` は `release/` 直下の 3 ファイル（平らに入る）。
- 4.4: job outputs は publish-crates が `<crate>_status`・`<crate>_reason`（5 クレート分）、publish-vsce が `status`・`reason`。どちらも末尾の `if: always()` の集約 step（`id: result`）が steps の outcome から決める（auth/login の失敗 → failed/auth、出力なし → failed/publish、未実行 → not-run）。publish step に continue-on-error は無いので、失敗すれば job は failure のまま。
- 4.5: report は `NEEDS_JSON: ${{ toJSON(needs) }}` を pwsh で解析する。skipped で outputs が空 → not-run、failure/cancelled で空 → 「job の summary を参照」。「どの公開先にも公開していない」は verify・gate・build のどれかが failure/cancelled のときだけ出す。
- 5.1: Marketplace のアクセストークンは、同じマネージド ID の資格情報を持つ `release-setup-check`（main からの実行）でも得られる。タグに限られるのは crates.io のトークンだけで、main を PR 経由でしか変えられないこと（ブランチ保護）が前提になる。release.yml・publish-vsix.ps1 は手順書の表を「名前の表」と呼ぶが、見出しは「名前の対応表」（どちらも design の語）。
