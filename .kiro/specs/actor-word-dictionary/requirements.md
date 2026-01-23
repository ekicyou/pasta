# Requirements Document

## Introduction

本仕様は、Pasta DSLにおける**アクター単語辞書機能**を定義する。現行のアクター属性（1対1キー・バリュー形式）を拡張し、複数値からのランダム選択機能を実装することで、より豊かなランダムトーク生成を実現する。

### 設計哲学

> **シーンに影響を与える要素 ＝ アクター**

台本DSLとして、登場人物だけでなく「ト書き」「ナレーション」「背景」「効果音」なども すべてアクターとして統一的に扱う。

### 対象クレート
- **pasta_core**: grammar.pest（構文解析）、AST定義
- **pasta_lua**: トランスパイラ（Lua出力）、ランタイム（word.lua）

---

## Requirements

### Requirement 1: DSL構文 - 複数値アクター属性（実装済み）

> **注**: grammar.pestの`words`ルールで既に対応済み。本要件は既存実装の文書化。

**Objective:** DSL作成者として、アクター属性に複数の値をカンマ区切りで定義できるようにしたい。これにより、ランダムトーク生成時に多様な出力が可能になる。

#### Acceptance Criteria

1. When アクター属性行にカンマ区切りの複数値が記述された場合, the Pasta Parser shall 各値を個別要素として認識しパースする
2. When カンマ文字として全角「、」「，」または半角「,」が使用された場合, the Pasta Parser shall いずれも区切り文字として認識する
3. When 値が「」（鉤括弧）で囲まれている場合, the Pasta Parser shall 囲み内のカンマをエスケープとして扱い、値の一部として保持する
4. When アクター属性に単一値のみが定義された場合, the Pasta Parser shall 単一要素の配列として扱う（後方互換性維持）
5. The Pasta Parser shall 空の値（連続カンマ）を無視して有効な値のみを収集する

#### 構文例

```pasta
％さくら
　＠通常：\s[0]、\s[100]、\s[200]
　＠照れ：\s[1]
　＠挨拶：こんにちは、やっほー、「やあ、元気？」
```

---

### Requirement 2: トランスパイル - Lua配列出力

**Objective:** pasta_luaトランスパイラとして、アクター属性を常にLua配列形式で出力したい。これにより、ランタイムでのランダム選択が統一的に行える。

#### Acceptance Criteria

1. When アクター属性がトランスパイルされる場合, the Lua Transpiler shall 値を `{ [=[値1]=], [=[値2]=] }` 形式のLua配列として出力する
2. When 単一値のアクター属性がトランスパイルされる場合, the Lua Transpiler shall 単一要素の配列 `{ [=[値]=] }` として出力する
3. The Lua Transpiler shall 文字列リテラルに `[=[...]=]` 形式を使用し、エスケープを不要にする
4. When 鉤括弧エスケープされた値がトランスパイルされる場合, the Lua Transpiler shall 鉤括弧を除去し、内部文字列のみを出力する

#### 出力例

**入力（DSL）:**
```pasta
％さくら
　＠通常：\s[0]、\s[100]
　＠照れ：\s[1]
　＠挨拶：「やあ、元気？」、こんにちは
```

**出力（Lua）:**
```lua
do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR.通常 = { [=[\s[0]]=], [=[\s[100]]=] }
    ACTOR.照れ = { [=[\s[1]]=] }
    ACTOR.挨拶 = { [=[やあ、元気？]=], [=[こんにちは]=] }
end
```

---

### Requirement 3: ランタイム - word関数によるランダム選択

**Objective:** ランタイムとして、単語参照時に配列からランダムに値を選択したい。これにより、毎回異なる出力でトークの多様性を実現する。

#### Acceptance Criteria

1. When `ACTOR:word(act, key)` が呼び出され、キーが配列を指す場合, the Pasta Runtime shall 配列からランダムに1要素を選択して返す
2. When `ACTOR:word(act, key)` が呼び出され、キーが関数を指す場合, the Pasta Runtime shall 関数を実行しその戻り値を返す
3. When `ACTOR:word(act, key)` が呼び出され、キーが存在しない場合, the Pasta Runtime shall nilを返しフォールバック処理へ移行する
4. The Pasta Runtime shall `math.random()` を使用してランダム選択を行う

#### 実装仕様

```lua
function ACTOR:word(act, key, ...)
    -- 1. 同名関数があれば呼び出し
    if type(self[key]) == "function" then
        return self[key](act, ...)
    end
    -- 2. 配列（単語辞書）ならランダム選択
    if type(self[key]) == "table" then
        return self[key][math.random(#self[key])]
    end
    -- 3. なければnil（フォールバックへ）
    return nil
end
```

---

### Requirement 4: フォールバック検索 - 階層的単語解決

**Objective:** ランタイムとして、単語が現在のスコープで見つからない場合に上位スコープを検索したい。これにより、グローバル単語のオーバーライドとデフォルト値の両方を実現する。

#### Acceptance Criteria

1. When 単語参照 `＠キー` が実行される場合, the Pasta Runtime shall アクター → シーン → グローバル の順序で検索する
2. When 検索中に関数が見つかった場合, the Pasta Runtime shall 完全一致で関数を実行する
3. When 検索中に単語辞書が見つかった場合, the Pasta Runtime shall 前方一致でマッチしたキーからランダム選択する
4. When すべてのスコープで単語が見つからない場合, the Pasta Runtime shall nilを返す（またはエラー処理）
5. The Pasta Runtime shall 最初にマッチしたスコープで検索を終了し、上位スコープへフォールスルーしない

#### 検索優先順位

| 優先度 | 検索対象 | マッチ方式 |
|--------|----------|------------|
| 1 | アクターの同名関数 | 完全一致 |
| 2 | アクターの単語辞書 | 前方一致 |
| 3 | シーンの同名関数 | 完全一致 |
| 4 | シーンの単語辞書 | 前方一致 |
| 5 | グローバルの同名関数 | 完全一致 |
| 6 | グローバルの単語辞書 | 前方一致 |

#### 動作例

```pasta
％さくら
　＠天気：晴れ、曇り

＠天気：雨、雪、台風    # グローバル単語

＊シーン
　％さくら、うにゅう
　
　　さくら：＠天気　だね。    # → 「晴れ」or「曇り」（アクター辞書）
　うにゅう：＠天気　やね。    # → 「雨」or「雪」or「台風」（グローバル）
```

---

### Requirement 5: アクター内Lua関数定義

**Objective:** DSL作成者として、アクター定義内にLua関数を直接記述したい。これにより、動的な単語生成（時刻、状態依存など）が可能になる。

#### Acceptance Criteria

1. When アクター定義内にluaコードブロックが記述された場合, the Pasta Parser shall コードブロックを関数定義として認識する
2. When luaコードブロックがトランスパイルされる場合, the Lua Transpiler shall コードブロック内容をそのままアクター定義内に展開する
3. The Lua Transpiler shall 関数シグネチャ `function ACTOR.関数名(act, ...)` を維持する
4. When 関数が `ACTOR:word()` から呼び出される場合, the Pasta Runtime shall 関数を実行し戻り値を単語として使用する

#### 構文例

````pasta
％さくら
　＠通常：\s[0]
```lua
function ACTOR.時刻(act, ...)
    local hour = os.date("%H")
    if hour < 12 then
        return "おはよう"
    elseif hour < 18 then
        return "こんにちは"
    else
        return "こんばんは"
    end
end
```
````

---

### Requirement 6: グローバル単語定義（既存機能）

> **注**: word.luaの`create_global`で既に対応済み。本要件は既存実装の文書化。

**Objective:** DSL作成者として、アクターに属さないグローバル単語を定義したい。これにより、プロジェクト全体で共有される単語辞書を作成できる。

#### Acceptance Criteria

1. When `＠キー：値1、値2` がアクター定義外（トップレベルまたはシーン直下）に記述された場合, the Pasta Parser shall グローバル単語定義として認識する
2. When グローバル単語がトランスパイルされる場合, the Lua Transpiler shall グローバルスコープの単語テーブルに登録する
3. When 単語参照がアクター・シーンで見つからない場合, the Pasta Runtime shall グローバル単語テーブルを検索する

---

### Requirement 7: 後方互換性

**Objective:** 既存のPastaスクリプト作成者として、既存のアクター定義が変更なしで動作し続けることを保証したい。

#### Acceptance Criteria

1. When 既存の単一値アクター属性がパースされる場合, the Pasta Parser shall 正常にパースし単一要素配列として扱う
2. When 既存のLua出力形式（単一文字列）を使用するスクリプトがある場合, the Pasta Runtime shall 配列形式に移行後もランダム選択で同一動作を維持する
3. The Pasta System shall 既存のアクター定義構文 `＠キー：値` を引き続きサポートする

---

## 非機能要件

### 性能

1. The Pasta Runtime shall 単語辞書のランダム選択をO(1)の時間計算量で実行する
2. The Pasta Parser shall 複数値パースのオーバーヘッドを最小限に抑える

### 保守性

1. The Pasta System shall 単語辞書機能を既存のパーサー・トランスパイラー構造に統合する
2. The Pasta System shall 新機能のユニットテストを提供する

---

## 将来の拡張可能性

現時点では `％` をアクター専用として維持するが、将来的に別種の辞書が必要になった場合は `＊種類：名前` 構文の導入を検討する（本仕様のスコープ外）。

---

## 改訂履歴

| 日付 | 内容 |
|------|------|
| 2026-01-23 | 初版作成 - 要件生成 |
