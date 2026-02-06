# Requirements Document

## Project Description (Input)
スポット指定が無い場合に前回スポットを引き継ぐ
スポット指定が無くても前回のスポット位置で再生を継続する

1. コンフィグでspotを登録できるように
2. 前回のspot値はactorオブジェクトに残り続けるはず、基本は引き継ぐ
3. トランスパイル時、act:clear_spot()は「％さくら、うにゅう」など、act:set_spotを出力しないときは出力しない。％とセットで出力する。
4. サンプルゴーストのコンフィグ設定で、アクターにspotを設定する（0,1）

## はじめに

本仕様は、アクターのスポット位置（`\p[ID]`）をシーン間で継続保持する機能を定義する。
現行実装では、すべてのシーンの`__start__`で`act:clear_spot()`が呼ばれ、スポット状態がリセットされる。
これにより、アクター行（`％さくら、うにゅう`）が省略されたシーンでも前回のスポット位置が失われてしまう。

本機能により、以下を実現する：
- コンフィグ（`pasta.toml`）でアクターごとのデフォルトspot値を設定可能にする
- `％`アクター行のないシーンでは前回のスポット位置を引き継ぐ
- `act:clear_spot()` は `act:set_spot()` と常にセットで出力し、単独では出力しない

## 要件

### 要件 1: コンフィグでアクターのデフォルトspot値を設定

**目的:** ゴースト制作者として、`pasta.toml`の設定でアクターごとのデフォルトスポット位置を定義したい。これにより、スクリプト中に毎回`％`行を書かなくてもアクターの表示位置が決まるようにしたい。

#### 受入基準
1. When `pasta.toml`に`[actor]`セクションが定義されている場合, the pasta system shall そのセクションからアクター名とデフォルトspot値のマッピングを読み込む
2. The pasta system shall `[actor]`セクションが未定義の場合でもエラーなく動作し、従来どおりの挙動を維持する
3. When `[actor]`セクションで定義されたアクターが使用される場合, the pasta system shall そのデフォルトspot値をアクターオブジェクトの初期状態として設定する

### 要件 2: スポット位置の継続保持とトークン出力制御

**目的:** ゴースト制作者として、アクター行（`％`）を省略したシーンでも前回のスポット位置で再生を継続したい。これにより、アクター構成が変わらない連続会話で冗長な`％`行を省略できるようにしたい。

**設計決定（議題1の結論）:**
- `STORE.actor_spots = {}` フィールドを追加してセッション全体でスポット状態を保持
- `store.lua` 初期化時に `CONFIG.actor[name].spot` から初期値を転送
- `sakura_builder.build()` は純粋関数を維持し、入力として `actor_spots` を受け取り、更新後の値を戻り値で返す
- `SHIORI_ACT:build()` が STORE との入出力を仲介

#### 受入基準
1. When `store.lua`が初期化される場合, the pasta system shall `CONFIG.actor`の各アクターの`spot`値を`STORE.actor_spots`に転送する
2. When `sakura_builder.build()`が呼ばれる場合, the builder shall 入力として受け取った`actor_spots`マップを使用してアクターのスポット位置を決定する
3. When `％`アクター行が明示的に記述されたシーンの場合, the pasta system shall `act:clear_spot()`と`act:set_spot()`をセットで出力し、`STORE.actor_spots`を更新する
4. When `％`アクター行が省略されたシーンの場合, the pasta system shall `STORE.actor_spots`の前回値を維持したまま`sakura_builder.build()`に渡す
5. The code generator shall `act:clear_spot()`を`act:set_spot()`と常にセットで生成し、`act:set_spot()`が1つも出力されない場合は`act:clear_spot()`も出力しない（**実装済み**: `code_generator.rs` L292-302で`!actors.is_empty()`ガード済み）

### 要件 3: サンプルゴーストのコンフィグ設定

**目的:** ゴースト制作者として、サンプルゴーストでアクターのデフォルトspot設定の実例を確認したい。これにより、自作ゴーストのコンフィグ設定の参考にできるようにしたい。

#### 受入基準
1. The sample ghost configuration shall `pasta.toml`の`[actor]`セクションでメインキャラクター（女の子）にspot=0を設定する
2. The sample ghost configuration shall `pasta.toml`の`[actor]`セクションでサブキャラクター（男の子）にspot=1を設定する
3. When サンプルゴーストが起動された場合, the pasta system shall コンフィグで指定されたデフォルトspot値に基づいてアクターのスポット位置を初期化する
