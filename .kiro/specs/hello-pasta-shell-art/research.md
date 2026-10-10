# ギャップ分析: hello-pasta-shell-art

作成: 2026-10-10（main `ea40c363`・ブランチ `claude/hello-pasta-shell-art-b407df`）。対象: `requirements.md`（R1〜R10）と既存のコードベースの差。方針は示すが、最終判断は要件ディスカッションと設計に委ねる。

## 1. 分析の要約

- 絵は `crates/pasta_sample_ghost` の `src/image_generator.rs`（572 行）が描き、`generate_ghost()`（`src/lib.rs`）が `shell/master/` に 18 枚と `surfaces.txt` を書く。`release.ps1` の Step 2（`cargo run -p pasta_sample_ghost`）が手元でもリリース CI（`release.yml` L119-120、スイッチ無しで呼ぶ）でも毎回上書きする。**この段を外すことが、素材への切り替えの核**。
- `.gitignore`（L38-42）は「シェルの画像は追跡を続ける」と明記済みで、`shell/master/` の 20 ファイルは追跡中。追跡の変更は不要（`release-ci` R10.5）。
- `surfaces.txt` は `element0,overlay,surfaceN.png,0,0` の 18 ブロックのみで、`collision` が無い。UKADOC の書式は `collisionN,始点X,始点Y,終点X,終点Y,ID`（矩形）と `collisionexN,ID,タイプ,座標...`（rect/ellipse/circle/polygon）。8 段目（`dic/08-touch.pasta` L11・L15・L20）は `＄ｒ４` を台詞にそのまま差し込む。`STAGES.md` L37 の検証の表 `4=Head` を `tests/tutorial_stages_test.rs` が読んで `Reference4=Head` を送る。
- 絵の生成を前提にしたテストは 4 箇所に散る: `tests/integration_test.rs`（L14-65・L271-286・吹き出し位置の決め打ち L251-266）、`src/lib.rs`（19 ファイル）、`src/config_templates.rs`（18 ブロック逐語一致）、`src/image_generator.rs`（L399-572）。コミットした実ファイルの PNG 寸法や `surfaces.txt` の書式を検証するテストは**無い**。
- ライセンス表記は 3 系統に割れている: ルート `LICENSE`（MIT 全文のみ）、`Cargo.toml` L12 と 4 クレートの README・`tech.md` L165（MIT OR Apache-2.0）、`about.hbs` L4/L8 と `editors/vscode/package.json` L7（MIT 単独）。`LICENSE-APACHE` は無い。**2026-10-10 の議論で MIT 単独に確定**（このリポジトリの開発物は MIT。`Cargo.toml`・README 4 つ・`tech.md` を「MIT」に直す）。素材の出どころの記録の先例は `book/src/img/claudia/LICENSE.txt`（Unlicense の顔アイコン 16 枚の出典）。
- 推奨: **Option C（ハイブリッド）**。絵・`surfaces.txt`・`descript.txt` は素材として手で置く（生成コードは削除）。検証は既存の `tests/integration_test.rs` の絵のテストを素材検証に置き換え、`release.ps1`・`release.yml` の検査は最小の差分で足す。規模 M・リスク Medium（未知は fal.ai の出力品質とドリフト。コード側は既知）。

## 2. 現状の調査

### 2.1 関係するファイルと役割

| ファイル | 現状 | 本仕様での扱い |
| --- | --- | --- |
| `crates/pasta_sample_ghost/src/image_generator.rs` | 572 行。`Character`・`Expression`・`generate_surfaces()`・`generate_surface()`。`image`/`imageproc` に依存 | 削除（R5.2） |
| `src/lib.rs` | `pub mod image_generator; pub mod config_templates;`・`GhostError`（`image::ImageError` を包む）・`generate_ghost()`・単体テスト 2 本 | `generate_ghost`・`GhostError::ImageError`・単体テストを削除。`scripts` モジュール（辞書の検証テスト）は残す |
| `src/main.rs` | CLI。`generate_ghost()` を呼び、ファイル数を表示。テスト 5 本 | 削除（`[[bin]]` も外す。R5.2・Q10） |
| `src/config_templates.rs` | `generate_surfaces_txt()`（18 ブロック）とテスト 3 本 | 削除。`surfaces.txt` は手書きの正本になる（R3.2・R4） |
| `src/scripts.rs` | 辞書の検証テスト（`03-face.pasta` の対応・`08-touch.pasta` の `＄ｒ４`） | 変更なし |
| `build.rs` | `pasta_shiori/src` の監視と `ghosts/` 不在時の案内（`cargo run` を案内。L35-42） | 案内文を直す（R5.6）。監視は残すか設計で判断 |
| `Cargo.toml`（クレート） | `[[bin]]`・`image.workspace`・`imageproc.workspace`・`thiserror` | `[[bin]]`・`image`・`imageproc` を外す。`thiserror` は `GhostError` 削除なら不要 |
| `Cargo.toml`（ルート）L62-63 / `Cargo.lock` | `image = "0.25"`・`imageproc = "0.26"`。このクレートしか使っていない | 外す（R5.5）。`release.yml` L182 の `git diff --exit-code -- Cargo.lock` があるので lock は同じ変更でコミットする |
| `ghosts/hello-pasta/shell/master/` | PNG 18 枚（各 2〜3 KB、128×256）・`surfaces.txt`（989 B）・`descript.txt`（230 B、`sakura/kero.balloon.offsetx,64`） | 全面差し替え（R1・R4・R9） |
| `release.ps1` | 7 段。Step 2 が `cargo run -p pasta_sample_ghost`。Step 7 は `.nar` の MB 表示のみで上限無し | Step 2 を外し 6 段に（R6.1）。上限の検査を足すかは設計で判断 |
| `release.bat`（ルート）L7-9 | 段の説明「1-3. Build pasta.dll, generate images, copy DLL/scripts」 | 説明を直す（R6.2） |
| `README.md`（クレート） | L12「自己完結型: シェル画像を Rust で自動生成（外部素材不要）」、L31-34 のツリー、L71-80 の 7 ステップ、L109-118 の `generate_ghost` API、L156-157 の `[gen]` | 書き直す（R5.6・R5.7・R6.2・R8.3） |
| `RELEASE.md` | リリース手順（CI 側の手順。画像生成の記述は無し） | 段の数の記述があれば合わせる（R6.2） |
| `STAGES.md` L37・L43 | 検証の表 `4=Head`、「`Head` は仮の部位名」 | 注記を確定に改める（R4.9）。7 段目の `＞ゴースト終了` は触らない（Q13） |
| `tests/integration_test.rs`（306 行） | 絵のテスト 4 本（L14-65・L271-286）、`descript.txt` の決め打ち（L251-266）、`common::copy_pasta_shiori_dll` | 絵のテストを素材検証に置き換え（R10）、吹き出しの値を更新（R9.3） |
| `tests/dist_src_validation_test.rs` | `shell/master/descript.txt` の存在のみ | 変更なし |
| `tests/tutorial_stages_test.rs` | `STAGES.md` の表から `Reference4=Head` を送る | 部位名が `Head` のままなら変更なし |
| `.github/workflows/release.yml` L119-120・L130-179 | `release.ps1` をそのまま呼ぶ。検査は `.nar` の 5 エントリのみ（シェルの検査無し・大きさの検査無し） | 検査に `shell/master/surfaces.txt` と絵を足す（R6.4）。リリース実行中は触らない（R6.6） |
| `.github/workflows/manual.yml` L135 | `cargo test -p pasta_sample_ghost` を実行 | 置き換え後のテストが CI で走る |
| `LICENSE`・`Cargo.toml` L12・`about.hbs`・`editors/vscode/package.json` | 表記の食い違い（§1） | R8.4（Q1 確定: MIT 単独。`about.hbs`・`package.json` は既に MIT） |
| `.kiro/steering/tech.md` L17・L77・L120、`structure.md` L198・L207・L264 | 「画像生成」「image/imageproc」 | 完了処理で同期（R5.8） |

### 2.2 既存の慣例

- **SSOT 方式**: テキスト系（`descript.txt`・`pasta.toml`・`dic/`・`install.txt`）は `ghosts/hello-pasta/` の手書きが正本。絵だけが「生成物だが追跡」という中途半端な状態で、本仕様はこれを「手書き（手置き）の正本」側に揃える。
- **検証はテストで**: 辞書の正本は `src/scripts.rs`・`tests/tutorial_stages_test.rs` が内容を検証する。絵と `surfaces.txt` にも同じ型（実ファイルを読んで検証）を当てるのが自然。
- **`.nar` の中身の検査は CI に**: `release.yml` の「配布物の検査」が zip のエントリ名を列挙して確かめる。絵の検査を足すならここ。
- **素材の出典の記録**: `book/src/img/claudia/LICENSE.txt`（出典 URL・ライセンス・取得日）が先例。`THIRD_PARTY_LICENSES.txt` は `cargo about` が Rust 依存だけから作るので、絵は載らない（載せる場所でもない）。
- **`release.ps1` の副作用**: Step 2 の再生成は `surfaces.txt` に CRLF 差分だけを残し、`release-ci` のタスクで `git checkout --` の運用になっていた（`release-ci/research.md` L454-457）。Step 2 を外せば消える。
- **テストはネットワーク非依存**: `PASTA_DEBUG` の中和（`ctor`）など、`cargo test` は環境に依存しない設計。素材の検証も同じ（R10.5）。

### 2.3 統合面

- **SSP の仕様（UKADOC）**: `surfaces.txt` の `collisionN,x1,y1,x2,y2,ID`／`collisionexN,ID,rect|ellipse|circle|polygon,...`。`seriko.use_self_alpha,1` が `descript.txt` にあるので PNG のアルファがそのまま使われる。`\![enter,collisionmode]` で当たり判定の枠を実機で可視化できる（検証に使える）。
- **SHIORI イベント**: `OnMouseDoubleClick` の `Reference4` = 当たり判定の ID。pasta 側は `＞transfer_req_to_var` → `＄ｒ４`。実機確認は MCP（`mcp__ssp__raise_event`・`get_log`）で `Reference4` を観測できる。
- **fal.ai（MCP）**: `fal-ai/qwen-image-edit-2511`（本線・seed と寸法を固定可）、`fal-ai/nano-banana-pro/edit`（比較用・SynthID 入り）、`fal-ai/birefnet/v2`（切り抜き）。有料。brief の調査は 2026-10-06 時点で未実行。

## 3. 要件の実現性

### 3.1 要件 → 資産の対応（ギャップのタグ: Missing／Unknown／Constraint）

| 要件 | 既存の資産 | ギャップ |
| --- | --- | --- |
| R1 立ち絵 18 枚 | 無し（ピクトグラムのみ） | **Missing**: 生成そのもの。**Unknown**: モデルの出力品質・キャラの一貫性・透かしの有無（qwen-image-edit は「見つかっていない」止まり） |
| R2 ずれない | 無し | **Unknown**: ドリフトの実測値（試作で測る）。**Constraint**: 方式 2 案（顔以外を合成し直す／element で重ねる）は後処理と `surfaces.txt` の形を変える |
| R3 番号の維持 | `03-face.pasta`・`scripts.rs` のテストが固定 | ギャップ無し。ファイル名を変えなければ満たす |
| R4 当たり判定 | `surfaces.txt` に無し。UKADOC 書式は既知 | **Missing**: 18 ブロックへの `collision` 行、座標（絵が決まってから）。**Unknown**: 部位名（`Head` 以外）。**Constraint**: `08-touch.pasta` と `08-touch.md` は逐語照合（辞書を変えると `book/` を触る） |
| R5 素材へ切り替え | `.gitignore` 済み | **Constraint**: `generate_ghost` を消すと `lib.rs`・`main.rs`・`config_templates.rs`・`image_generator.rs`・テスト 4 箇所・README・`build.rs` が連動。`Cargo.lock` の更新を同じ変更に含める |
| R6 配布スクリプト | `release.ps1`・`release.yml`・`release.bat`・README | **Missing**: Step 2 除去と段の番号振り直し、CI の検査にシェルを足す。**Constraint**: リリース実行中は触らない（`release-workflow` の席） |
| R7 大きさ | `release.ps1` L253-262 が MB 表示、`pasta_check` が KB 表示。上限の検査は無し | **Missing**: 上限の数字（Q5）と、それを確かめる場所（テスト／`release.ps1`／CI）。現行 `.nar` 2,055,580 B（v0.3.8）、絵は約 50 KB |
| R8 記録・ライセンス | `claudia/LICENSE.txt` の先例。`LICENSE` は MIT のみ | **Missing**: 生成の記録（新規）、表記を MIT に揃える修正（`Cargo.toml`・README 4 つ・`tech.md`）。**Unknown**: 日本法での AI 生成物の扱い（brief で未確認） |
| R9 吹き出し | `descript.txt` offsetx 64（幅 128 の半分）、テストが決め打ち | **Missing**: 新寸法に合わせた値。**Unknown**: 実機で自然に見える位置（絵が決まってから） |
| R10 テスト | 生成前提のテストのみ。PNG を読む部品 `image` は外す予定 | **Constraint**: PNG の寸法・アルファ・四隅の透明の検証に `image` を使うなら dev-dependency として残す。使わないなら PNG ヘッダ（IHDR: 幅・高さ・color type）を手で読む最小の実装（数十行）で足りる。四隅の透明までは IHDR だけでは分からない |

### 3.2 技術的に必要なもの

- **生成パイプライン**（手作業＋MCP）: 基準画像 1 枚 → 表情の編集 ×9 → 切り抜き → 縮小 → PNG 最適化。全部リポジトリ外の作業で、結果（PNG）だけコミット。手順は文書（R8.1）。
- **`surfaces.txt` の手書き**: 18 ブロック × （element 1 行 ＋ collision 2〜3 行）。element で重ねる方式なら胴体 2 枚 ＋ 顔 18 枚の 20 枚構成になり、ブロックの element が 2 行になる。
- **素材の検証テスト**: ファイルの存在・PNG ヘッダ・大きさ・`surfaces.txt` の構文（簡易パーサ）・`collision` の `Head`。
- **スクリプトの整理**: `release.ps1`（Step 2 除去、`[n/6]`、`-SkipSetup` の説明「steps 1-2」）、`release.bat` のコメント、README。
- **CI の検査**: `release.yml` L148 のエントリ一覧に `shell/master/surfaces.txt` と `shell/master/surface0.png`〜（18 枚）を足す。

### 3.3 複雑さの信号

- コード側は「削除と置き換え」で単純（生成コード 700 行超の削除、テスト 100 行程度の追加）。
- 外部統合（fal.ai）が本仕様の主要な不確実性。品質が出るまでの試行回数と費用が読めない。
- ワークフロー: `release.ps1` の変更は `release-workflow` の実行と重ねられない（タイミングの制約）。

## 4. 実装アプローチの選択肢

### Option A: 生成コードを残し、素材を保護する

`generate_ghost()` を残したまま、`shell/master/` に絵があれば上書きしない（または出力先を変える）。`release.ps1` の Step 2 は残す。

- 変更: `lib.rs` に「存在すれば書かない」分岐、テストの調整。
- ✅ 差分が最小、`cargo run` の案内がそのまま使える。
- ❌ 丸と三角の生成コードが残り、README の「Rust で生成」も半分残る。素材と生成物の二重の正本になり、brief の「生成物ではなく素材」と食い違う。`image`/`imageproc` も残る。
- 評価: brief の方針（Q2 で「変える」）に反する。採らない。

### Option B: 生成コードを全部消し、素材を正本にする（新規は記録と検証テスト）

`image_generator.rs`・`config_templates.rs`・`main.rs`・`generate_ghost`・`GhostError` を削除。`surfaces.txt`・PNG・`descript.txt` は手書き／手置きの正本。素材検証テストと生成の記録を新設。

- 変更: 上の削除 ＋ `Cargo.toml` ×2 ＋ `Cargo.lock` ＋ `release.ps1` ＋ `release.bat` ＋ README ＋ `build.rs` ＋ テスト ＋ `release.yml` の検査 ＋ 記録文書 ＋ ライセンス表記を MIT に揃える修正。
- ✅ 正本が 1 つ。クレートは「辞書と配布物の検証テストの置き場」に純化（`scripts.rs`・`tests/`）。依存が減る。
- ❌ 触るファイルが多い（ただし各ファイルの変更は単純）。PNG を読む部品が無くなるので、検証テストは PNG ヘッダの手読みか `image` の dev-dependency 化のどちらかを選ぶ。
- 評価: brief と一致。

### Option C: B を段階に分けるハイブリッド（推奨）

B の内容を、独立に検証・revert できる段に分ける。

1. **素材の作成**（リポジトリ外）: 試作（1 キャラ × 3 表情）→ 実測の記録 → 本番 18 枚 → 後処理。成果物と記録だけコミット。
2. **シェルの更新**: PNG 18 枚・`surfaces.txt`（collision 付き）・`descript.txt`（吹き出し）。実機で表情・当たり判定・吹き出しを確認。
3. **クレートの整理**: 生成コードの削除・依存の除去・`Cargo.lock`・テストの置き換え・README・`build.rs`。
4. **配布の整理**: `release.ps1`・`release.bat`・`release.yml` の検査・大きさの確認。リリースの実行と重ならない時期にまとめて入れる。
5. **ライセンスと steering**: 表記を MIT に統一・`tech.md`/`structure.md` の同期。

- ✅ 2 と 3 の順序を入れ替えられる（絵が先に無くても 3 は進む。ただし 3 のテストは 2 の素材を読むので、同じ PR か 2 を先に）。4 はタイミングの制約を単独で管理できる。
- ❌ 段の間は「新しい絵 ＋ 旧生成コード」の中間状態がありうる。`release.ps1` を実行すると絵が戻るので、2 と 4（Step 2 の除去）を離さない、または 3 を 2 に含める。
- 評価: 推奨。**2 と「Step 2 の除去」は同じ変更に含める**のが安全。

## 5. 工数とリスク

- **工数: M（3〜7 日）**。コードは削除中心で既知の作業だが、絵の生成と後処理の試行、実機確認、文書が時間を取る。brief の見積り 12〜14 タスクと整合。
- **リスク: Medium**。
  - High 寄りの要素: fal.ai の出力でキャラの一貫性・ドリフトが要件（R2.1 の 2 px）に収まるか未検証。収まらなければ element で重ねる方式か、許容値の見直しが要る。
  - Low の要素: クレート・スクリプト・CI の変更はすべて既知のファイルで、`release-ci` が追跡の境界を済ませている。

## 6. 設計フェーズへの推奨

### 6.1 推奨の方針と主要な判断

- Option C。素材は手置きの正本、生成コードは削除、検証はテスト（`tests/integration_test.rs` の置き換え）と `release.yml` の検査の 2 層。
- ドリフト対策は試作の実測で選ぶ: 2 px 以内に収まるなら「顔以外を基準画像から合成し直す」（`surfaces.txt` は 1 ブロック 1 element のまま、テストの 18 枚前提も変わらない）。収まらないなら element で重ねる（20 枚構成。R1.1 の「18 枚」とテストの前提を改める）。
- 当たり判定は `collisionex,ID,ellipse`（頭）または矩形の `collision`。座標は絵が決まってから、全表情で同一（R4.4）。
- 吹き出しの位置は、新しい幅の半分を起点に実機で調整（現行 64 = 幅 128 の半分）。
- PNG の検証に `image` を dev-dependency で残すか、IHDR の手読みにするか。四隅の透明（R10.2）まで見るなら `image`（または `png` クレート）が楽。ワークスペースの `[workspace.dependencies]` から外す要件（R5.5）とは両立する（クレートの dev-dependency に直接書ける）が、`Cargo.lock` には残る。

### 6.2 設計へ持ち越す調査項目（Research Needed）

1. `fal-ai/qwen-image-edit-2511` の出力に透かし（可視・不可視）が入るか、出力の利用条件（商用・再配布）の一次資料。brief は「見つかっていない（未確認）」。
2. 試作の実測: 表情の編集で髪・服・輪郭が何 px 動くか、`birefnet/v2` の切り抜き後に 150〜300 px へ縮小したときの縁（ハロー・ジャギー）。
3. PNG の最適化で 1 枚 100 KB 以下に入るか（寸法 × 色数）。入らなければ Q5 の数字を見直す。
4. 日本法での AI 生成物の著作権の扱い（記録に書く注記の文面）。
5. （解決）`about.hbs`・`package.json` は既に MIT。Q1 は MIT 単独で確定。
6. `build.rs` の `pasta_shiori/src` 監視は、生成を消した後も意味があるか（無ければ `build.rs` ごと削除できる）。

### 6.2a 画風の手本の調査（2026-10-10・要件ディスカッション）

画風と構図は「悪役令嬢クローディア」のシェルに揃えると決めた（Q15）。手本のリポジトリ（`https://github.com/ponapalt/claudia`、確認したコミット `02cbd4f5`）で確かめた事実。

- **構図と寸法**: 全身・正面・約 3 頭身のちびキャラ。`shell/master/surface0.png`（令嬢）と `surface10.png`（執事）はどちらも 333×500 px。紹介ページ用の `site/img/s0.png` は 256×477 px・168 KB。
- **絵の作り方**: 制作後記（`https://ponadocs.shillest.net/claudia/`）に「シェル画像の生成: ponapalt（Claudia と相談しながら gpt-image-2.5-flare で生成）」とある。brief の本線（`fal-ai/qwen-image-edit-2511`）とは別のモデル。
- **表情の作り方**: 同じポーズの表情は `element0,base,surface0.png` に目だけの画像（`claudia_eyes_N.png`）を `element1,overlay` で重ねる。ポーズが違う表情（扇を上げるなど）だけ別の 1 枚絵を持つ。brief のドリフト対策の 2 案目（element で重ねる）の実例で、ずれが構造上起きず、ファイルも小さい。
- **当たり判定**: `collisionexN,ID,rect|ellipse|polygon,...` の形。令嬢は `Head`・`Face`・`Bust`・`Hair`・`Foot`・`Skirt`・`Fan`、執事は `Head`・`Face`・`Body`・`Foot`・`Tray`。全サーフェスに同じ名前を置き、ポーズが違う絵だけ座標を変えている。
- **吹き出し**: `sakura.balloon.offsetx,50`・`offsety,15`、`kero.balloon.offsetx,70`・`offsety,220`、`balloon.alignment,none`。
- **ライセンス**: 辞書・シェル画像を含めすべて Unlicense。画風の参照画像として生成に渡せる。

設計で調べること:

1. 手本と同じモデル（gpt-image-2.5-flare）が使えるか、出力の利用条件と透かしの有無。使えない場合に、brief の本線のモデルで手本の画風をどこまで再現できるか（手本の絵を参照画像として渡す）。
2. 大きさ: 手本と同じ 333×500 px で 18 枚を 1 枚絵で持つと約 3 MB（168 KB × 18）の見積り。**確定（Q5・Q11）**: キャンバス 333×500 px、絵 1 枚 ≤ 250 KB・合計 ≤ 4.5 MB・`.nar` ≤ 7 MB。目だけ重ねる構成なら大幅に減る。

### 6.2b 要件ディスカッションから設計へ持ち越した判断（2026-10-10）

要件では結果だけを定め、方式は設計で選ぶと決めた項目。

1. ドリフト対策の方式（Q6）: 顔以外を基準画像から合成し直す／`surfaces.txt` の element で胴体と顔を重ねる。試作の実測（R2.4）で選ぶ。R2.1 の許容値 2 px も同じ実測で見直す。
2. 立ち絵の寸法の具体値（Q11）: 幅 150〜300 px の範囲で、試作の縮小後の縁の品質と大きさの上限（R7）から決める。
3. 絵の検証テストの置き場所と PNG の読み方（Q9）: `tests/integration_test.rs` の置き換えを仮定。PNG の寸法・アルファ・四隅の透明の検証に `image`（または `png`）を dev-dependency で残すか、IHDR を手で読むか。
4. 大きさの上限を確かめる場所（R7・R10.4）: テスト／`release.ps1`／`release.yml` の検査のどこで見るか。
5. 当たり判定の形と座標（R4）: `collision`（矩形）か `collisionex`（ellipse など）か。重なる部位の優先順位（UKADOC の定義順の扱い）を確かめる。
6. `build.rs` の扱い（6.2 の 6 と同じ）。

### 6.3 要件ディスカッションで決める事項（requirements.md の Q1〜Q14 と対応）

- Q1 ライセンス表記の解消方法 → **確定: MIT 単独**（2026-10-10）
- Q2 外部素材への方針転換（転換を仮定）
- Q3 生成元を同梱物に記録するか（リポジトリのみを仮定）
- Q4 部位名（`Head` 固定。`Face`・`Bust` は台詞で読んで不自然でないか）
- Q5 大きさの上限 → **確定**: 1 枚 250 KB・合計 4.5 MB・`.nar` 7 MB（Q11 のキャンバス 333×500 px と併せて 2026-10-10）
- Q6 ドリフト対策の方式（設計で選ぶを仮定）
- Q7 再現手順を残すか → **確定**: 文書のみ・スクリプト無し。設定画像（2 人の基準の絵・縮小前）だけはコミットする。置き場所は設計で決める（`ghosts/hello-pasta/` の外。`.nar` に入らない場所。クレートは `publish = false` なので crates.io には出ない）
- Q8 `image`/`imageproc` の除去（除去を仮定。検証テストでの dev-dependency は別論点）
- Q9 絵のテストの置き場所（`tests/integration_test.rs` を仮定）
- Q10 CLI と `generate_ghost` の削除（削除を仮定）
- Q11 寸法（幅 150〜300 px・2 人同寸を仮定）
- Q12 費用の上限 → **確定**: 試作 3 ドル・全体 40 ドル。残高は前払いで、開発者が 40 ドルを先に入れる
- Q13 `STAGES.md` 7 段目の `＞ゴースト終了`（触らないを仮定）
- Q14 絵のライセンス → **確定**: 絵だけ Unlicense（手本と同じ）。配布物には表示を足さない（Q3 も確定）。`manual-shell-guide` へ申し送る
