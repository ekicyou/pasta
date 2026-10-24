# Requirements Document

## Project Description (Input)
pasta の Lua ランタイムは mlua 0.11（0.11.6。LuaJIT 2.1、features `luajit52`・`vendored`・`serialize`）の上にある。mlua は 2026-07-05 に 0.12.0 を出し、2026-10-03 の 0.12.2 が最新で、0.11 系は 2026-01-27 の 0.11.6 から止まっている。0.11 に留まるほど上げるときの差が大きくなり、不具合の直しや LuaJIT の取り込み（`luajit-src` の版は mlua-sys が決めるので、今は `210.6.6` に止まっている）が 0.12 の側にしか入らなくなる。0.12 は Rust 2024 edition で pasta と揃う。

上げるのを止めていた mlua-stdlib は `mlua-stdlib-removal` で外れ、mlua の serde の値の変換（`to_value`・`from_value`）は `toml-serialization` で使う所が無くなった（残るのは配列の印 `array_metatable` だけ）。この spec では、ルートの `Cargo.toml` の mlua を 0.12 系に上げ、0.12 の破壊的変更を pasta の使い方に当ててまとめて直す。Lua から見える挙動（エラーの文言・コルーチンの扱い・`require` の探し方・配列の印）は変えない。変わるところは、決めたうえでマニュアルに書く。x86・x64 のビルドとテスト、`pasta_lsp` の WASM のビルドを通す（詳しくは brief.md）。

## Introduction

この spec は、pasta の Lua ランタイムの土台を mlua 0.11 から 0.12 系へ上げる。ゴーストの作者から見て、何も変わらないことが目標である。

主な利用者は 3 人である。

- **ゴーストの作者**: `.pasta` の辞書と `scripts/` の Lua を書き、SSP で pasta.dll を動かす人。版上げの前と後で、同じ辞書が同じように動くことを期待する。
- **pasta の開発者**: 依存を管理し、今後の mlua の更新を `/update` の小さな直しで済ませたい人。
- **後に続く spec**: `shiori4-v3-portable`・`ponytail-lua-*` など、mlua 0.12 の上の main でギャップ分析をやり直す spec。

版上げの前と後で、次のような観察が変わらないことを確かめる。

```
（作者の辞書）
＊OnBoot
　さくら：こんにちは。

（SSP に返る値）  \0こんにちは。\e  ← 前後で同じ
```

```lua
-- scripts/ の Lua
local ok, err = pcall(require, "no_such_module")
-- err の文言（"module 'no_such_module' not found:" に続く探した場所の並び）が前後で同じ
local co = coroutine.create(function() coroutine.yield(1) end)
coroutine.resume(co)        -- true, 1
coroutine.status(co)        -- "suspended"  ← 前後で同じ
```

```toml
# save/ のセーブファイル（@pasta_persistence）
空の配列 = []      # 配列の印の付いた空の表は、前後とも [] と書く
```

## Boundary Context

- **In scope**
  - ルートの `Cargo.toml` の mlua の版（0.12 系）と `Cargo.lock`（mlua に引かれて上がる mlua-sys・`luajit-src`・`lua-src` を含む）
  - mlua を参照する 4 つのクレート（`pasta_lua`・`pasta_shiori`・`pasta_check`・`pasta_sample_ghost`）の、0.12 に合わせる直し（移った型の参照・使わなくなった名前の置き換え）
  - Lua から見える挙動が変わらないことの確かめ（直す前に、まだ押さえていない所の特性化テストを足す）
  - x86（`i686-pc-windows-msvc`）・x64 のビルド・テスト・clippy、`pasta_lsp` の WASM のビルド、`pasta_check` のライブラリの WASM の確かめ
  - `tech.md` の「主要依存関係」と「言語・ランタイム」、mlua の版に触れているマニュアル・ソースの説明
  - x86 のビルドが今の main で通らない不具合（`toml-serialization` が入れたセーブの整数の変換が、x86 の Lua の整数（32 ビット）を考えていない。mlua の版に関わらず起きる）の直し。**前提（仮定）**: x86 のビルドとテストを通すことがこの spec の Desired Outcome なので、この spec で直す。別の直しとして先に main へ入れるかは、要件のディスカッションで決める（OPEN QUESTION 1）
- **Out of scope**
  - mlua の features を変えること（`luajit52`・`vendored`・`serialize` のまま。Luau・Lua 5.4・5.5 へは移らない）
  - 0.12 で増えた機能（`#[derive(UserData)]`・スレッドの作成と再開のコールバック・`Thread::is_resumable` など）を使うための作り直し
  - Lua ランタイムの設計の変更（`ponytail-lua-*` の各 spec が持つ）
  - LuaJIT そのものの版を mlua と別に選ぶこと（mlua-sys が選ぶ `luajit-src` の版に従う）
  - LuaJIT に無い `coroutine.close`・`<close>` を足すこと（mlua の版に関わらず無い）
- **Adjacent expectations**
  - `mlua-stdlib-removal`・`toml-serialization`・`ponytail-check-release` は入っている前提で、その後の main の上で直す（`self_deploy_integration_test.rs` は `mod common;` を使う今の形の上で直す）。
  - ルートの `Cargo.toml`・`Cargo.lock` は、この spec が走る間はこの spec だけが触る。`shiori4-v3-portable`・`pest-2-9-4-upgrade`・`ponytail-editor-tooling`・`ponytail-core-registry` は同じウェーブに置かない。
  - 今後の mlua の更新は `/update` が扱う（この spec の後に続く spec は無い）。

## Requirements

### Requirement 1: mlua 0.12 系への依存の更新
**Objective:** As a pasta の開発者, I want pasta が mlua 0.12 系の上でビルドされる, so that 0.12 の側にだけ入る不具合の直しと LuaJIT の取り込みを受け取れる

#### Acceptance Criteria
1. The pasta ワークスペース shall mlua を 0.12 系（0.12.2 以上・0.13 未満）に依存する。**前提（仮定）**: 下限は 0.12.2（OPEN QUESTION 4）。
2. The pasta ワークスペース shall mlua の features を `luajit52`・`vendored`・`serialize` のまま保つ。**前提（仮定）**: `serialize` は配列の印のためだけに要るが、残す（OPEN QUESTION 3）。
3. The pasta ワークスペース shall crates.io に公開できる形の依存だけで mlua を引く（git やパスの依存を使わない）。
4. When `Cargo.lock` を更新するとき, the pasta ワークスペース shall mlua に引かれて上がる mlua-sys・`luajit-src`・`lua-src` 以外の依存の版を変えない。
5. The pasta.dll shall 静的 CRT のリンクのまま、mlua が組み込む LuaJIT を同じビルドに含める。
6. The pasta ワークスペース shall mlua を参照するすべてのクレートが、同じ 1 つの mlua の版を使う。

### Requirement 2: ビルドと検査の関門
**Objective:** As a pasta の開発者, I want 版上げの後もいつもの関門がすべて通る, so that 0.4.3 のリリースを版上げのせいで止めない

#### Acceptance Criteria
1. When x64（`x86_64-pc-windows-msvc`）と x86（`i686-pc-windows-msvc`）のそれぞれでワークスペースをビルドするとき, the pasta ワークスペース shall 誤りなくビルドを終える。
2. When x64 と x86 のそれぞれでワークスペースのテストをすべて走らせるとき, the pasta ワークスペース shall すべてのテストを通す（版上げの前に x64 で通っていたテストを、期待を変えずに通すことを含む）。
3. When clippy を警告を誤りとして扱って全ターゲットに走らせるとき, the pasta ワークスペース shall 警告を 1 つも出さない（mlua 0.12 で使わなくなった名前の警告を含む）。
4. When `pasta_lsp` を WASM（`wasm32-unknown-unknown`）向けにビルドし、`pasta_check` のライブラリを既定の features 無しで WASM 向けに確かめるとき, the pasta ワークスペース shall 誤りなく終える。
5. When Lua のスクリプトの単体テスト（lua_test。Rust のテストから pasta のランタイムの上で走る）を走らせるとき, the pasta ワークスペース shall 版上げの前と同じ結果を出す。
6. If 版上げの前に通っていたテストが版上げの後に落ちるとき, the pasta の開発者 shall そのテストの期待を書き換えて通すのではなく、Requirement 4 に従って挙動の差として扱う。

### Requirement 3: Lua から見える挙動の保存
**Objective:** As a ゴーストの作者, I want 版上げの前と後で同じ辞書と Lua が同じように動く, so that 版上げのために辞書やスクリプトを書き直さずに済む

#### Acceptance Criteria
1. When 同じ辞書に同じ SHIORI の要求を送るとき, the pasta.dll shall 版上げの前と同じ応答（さくらスクリプトの値）を返す。
2. When Lua のコードが実行時の誤り・構文の誤りで失敗するとき, the pasta ランタイム shall 版上げの前と同じ文言（`runtime error: `・`syntax error: ` の接頭辞の有無、止まるエラーのバルーンに出す 1 行の形を含む）で知らせる。
3. When `require` でモジュールを探すとき, the pasta ランタイム shall 版上げの前と同じ順で同じ場所を探し、見つからないときと読めないときに同じ文言で失敗する。
4. If Lua ファイルの searcher を置き換える前提（`package.loaders` が 4 つの要素を持つ）が崩れているとき, the pasta ランタイム shall 版上げの前と同じく、別の searcher を黙って上書きせずに VM の構築を失敗させる。
5. When シーンやチェイントークのコルーチンを作り、`yield` し、再開し、捨てるとき, the pasta ランタイム shall 版上げの前と同じ状態（`coroutine.status` の値・再開の結果・捨てた後の扱い）を示す。
6. When `@pasta_persistence` が配列の印の付いた表（空の表を含む）を保存し、読み戻すとき, the pasta ランタイム shall 版上げの前と同じく、空の配列を `[]` と書き、読んだ配列に配列の印を付ける。
7. When セーブやログの表が表の中身を読むとき, the pasta ランタイム shall 版上げの前と同じく、表の `__pairs` を呼ばずに生の中身だけを読む。
8. When 作者の Lua が `@toml` の `decode`・`@enc` の文字コードの変換・`ffi.load` を使うとき, the pasta ランタイム shall 版上げの前と同じ結果と、同じ失敗の文言を返す。
9. The pasta ランタイム shall Lua の標準ライブラリの有効・無効を決める 13 の `std_` の名前を、版上げの前と同じ意味で受ける（pasta.toml の書き方を変えない）。

### Requirement 4: 挙動が変わるときの扱い
**Objective:** As a ゴーストの作者, I want 版上げで変わる挙動があるなら知らされる, so that 黙って変わった挙動に驚かされない

#### Acceptance Criteria
1. If mlua 0.12 または新しい LuaJIT で Lua から見える挙動が変わるとき, the pasta の開発者 shall pasta の側で前の挙動に戻せるなら戻す。
2. If 前の挙動に戻せない、または戻さないと決めた挙動の差があるとき, the pasta の開発者 shall その差と理由をマニュアルの該当の章に書き、リリースノートの「動きが変わる変更」に載せる。
3. The pasta の開発者 shall 変わった挙動を、特性化テストの期待として新しい挙動のまま固定し、その差を書いた所を指す。
4. The pasta の開発者 shall 挙動の差が 1 つも無かったことも、確かめた結果として spec の記録に残す。

### Requirement 5: 特性化テストの先行
**Objective:** As a pasta の開発者, I want 版上げの前に、変わりうる挙動をテストで押さえる, so that 版上げで変わったかどうかを推測でなく結果で判断できる

#### Acceptance Criteria
1. The pasta の開発者 shall Requirement 3 の各項目について、既存のテストで押さえているかを確かめ、押さえていない項目のテストを、mlua の版を上げる前の main で書いて通す。
2. When 特性化テストを足したとき, the pasta の開発者 shall mlua 0.11 の上でそのテストが通ること（x86 は Requirement 7 の直しの後）を確かめてから版を上げる。
3. The 特性化テスト shall 版上げの後も同じ期待のまま通る（Requirement 4 で変わると決めた項目を除く）。
4. The 特性化テスト shall `PASTA_DEBUG` などの環境変数に左右されずに通る。

### Requirement 6: デバッガの働きの保存
**Objective:** As a ゴーストの作者, I want VS Code のデバッガ（`PASTA_DEBUG=1` のときの DAP）が版上げの後も同じように止まり、変数を見せる, so that 版上げでデバッグの手段を失わない

#### Acceptance Criteria
1. While デバッグが有効のとき, when ブレークポイントの行に来たとき, the pasta ランタイム shall 版上げの前と同じくその行で止まる。
2. While シーンのコルーチンの中で止まっているとき, the pasta ランタイム shall 版上げの前と同じく、そのコルーチンの本体の呼び出しの積み重なりとローカル変数・上位値を示す。
3. When コルーチンが `yield` して再開したとき, the pasta ランタイム shall 同じコルーチンを同じものとして扱い、ステップ実行を続けられる。
4. The pasta ランタイム shall デバッグの有効・無効に関わらず、Lua の `debug` ライブラリをスクリプトに開かない。

### Requirement 7: x86 の pasta.dll のセーブの整数
**Objective:** As a x86 の pasta.dll を使うゴーストの作者, I want セーブの整数が x64 と同じ値で保存され、読み戻される, so that 32 ビットの環境でも保存した数が化けたり、読み込みで失われたりしない

#### Acceptance Criteria
1. When x86 の pasta.dll が 32 ビットに収まる整数の値をセーブに書くとき, the pasta ランタイム shall x64 と同じ TOML の整数として書く。
2. When x86 の pasta.dll が 32 ビットに収まらない整数の値をセーブに書くとき, the pasta ランタイム shall 値を変えずに書く。**前提（仮定）**: x86 ではその値を Lua の側が浮動小数として持つので、x64 では `3000000000`、x86 では `3000000000.0` と書いた文字が違うが、読み戻した Lua の値は同じになる。書いた文字まで揃えるかは OPEN QUESTION 2。
3. When x86 の pasta.dll が 32 ビットに収まる整数をセーブから読むとき, the pasta ランタイム shall x64 と同じ値の Lua の数として渡す。
4. If x86 の pasta.dll が 32 ビットに収まらない整数をセーブから読むとき, the pasta ランタイム shall 値を黙って切り詰めず、Lua の数（浮動小数）として、表せる範囲で同じ値を渡す。**前提（仮定）**: LuaJIT の数はもともと倍精度の浮動小数なので、2^53 までは値を失わない。2^53 を超える整数は、x64 と同じく丸めて渡す（`toml-serialization` の確定した設計判断 6。マニュアル `pasta-persistence.md`・`toml.md` に書いてある）。
5. When 配列の印を判定するために表の整数のキーを数えるとき, the pasta ランタイム shall x86 と x64 で同じ表を同じ配列・同じ表として判定する。
6. The pasta の自動テスト shall x86 と x64 のそれぞれで、セーブの整数の書き出しと読み戻しを確かめる（x64 だけで意味を持つ `i64` の端の値の試験は、x86 での期待を別に持つ）。

### Requirement 8: 文書の更新
**Objective:** As a pasta の開発者, I want 依存の版を書いた文書が今の姿を示す, so that 後に続く spec と作者が古い版を前提にしない

#### Acceptance Criteria
1. The `tech.md` shall 「言語・ランタイム」と「主要依存関係」に mlua 0.12 と書く。
2. The pasta の開発者 shall マニュアルとソースの説明のうち、mlua の版（0.11・0.11.6・mlua-sys 0.10 など）を前提に書いた所を、0.12 系で確かめ直した内容に改める。
3. Where 第三者のライセンスの表示に LuaJIT の著作権の文を固定で書いているとき, the pasta の開発者 shall 新しい `luajit-src` の著作権の文と食い違わないことを確かめる。
4. When 版上げが入ったとき, the pasta の開発者 shall `roadmap.md` に、`pasta_lua` を触る後の spec がギャップ分析をやり直す前提が満ちたことを書く。
5. When 0.4.3 のリリースノートを書くとき, the pasta の開発者 shall `pasta_lua` が公開する `mlua`（`pasta_lua::mlua`）の版が 0.12 に変わったことを、Rust のライブラリとして使う人に向けて書く。

## 未決の論点（OPEN QUESTIONS）

要件のディスカッションで決める。上の要件は、各項目の「前提（仮定）」で下書きしてある。

1. **x86 のビルドの不具合を、どこで直すか**: 今の main は x86 のビルドが通らない（CI の build-x86 が `e221ebf3`（`toml-serialization`）から赤）。前提（仮定）はこの spec で直す（Requirement 7）。先に小さな直しとして main へ入れる道もある。
2. **x86 と x64 で、セーブに書いた文字を揃えるか**: 32 ビットに収まらない整数を、x86 は `3000000000.0`、x64 は `3000000000` と書く（読み戻した値は同じ）。表に入った `i64::MIN` も、x64 は整数、x86 は `-9.223372036854776e18` と書く（版上げの試しで、既存の往復の試験 2 本が x86 だけこの差で落ちた）。揃えるなら、整数の値を持つ浮動小数を整数として書く決まりが要る（Requirement 7.2）。
3. **features の `serialize` を残すか**: 使うのは配列の印（`array_metatable`）だけ。前提（仮定）は残す（brief の推奨。範囲を広げない。Requirement 1.2）。
4. **mlua の版の下限**: 前提（仮定）は 0.12.2 以上（0.12.1・0.12.2 のコルーチン・フック・コールバックの直しを確実に受ける。Requirement 1.1）。`0.12` とだけ書くかは設計で決めてもよい。
5. **`pasta_lua::mlua` の版が変わることを、0.4.3（パッチの版）で出してよいか**: `pasta_lua` は crates.io に出しており、`pub use mlua;` で mlua を公開の口に含む。0.4.3 にはすでに破壊的変更（`mlua-stdlib-removal` の `RuntimeConfig` など）が入っているので、前提（仮定）は 0.4.3 のまま出してリリースノートに書く（Requirement 8.5）。
