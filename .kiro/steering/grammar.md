# Pasta DSL文法ステアリング

## このドキュメントの役割

**対象**: AI開発支援ツール（GitHub Copilot等）

**役割**:
- Pasta DSL の**AI向け完全参照**ドキュメント
- `doc/spec/`（章別分割仕様書）に準拠
- 高頻度参照情報（マーカー一覧、基本パターン）を展開

**人間向けとの役割分離**:
- **[GRAMMAR.md](../../GRAMMAR.md)**: 人間向け読みやすさ優先の学習用マニュアル
- **[doc/spec/](../../doc/spec/)**: 権威的仕様書（章別分割）
- **このファイル**: AI向け完全参照（高頻度パターン展開）

---

## 権威的仕様書

**開発時は必ず参照**: `doc/spec/` ディレクトリ（章別分割）

| 章番号                                                        | 参照用途                     |
| ------------------------------------------------------------- | ---------------------------- |
| [01-grammar-model.md](../../doc/spec/01-grammar-model.md)     | 行指向文法・式サポート       |
| [02-markers.md](../../doc/spec/02-markers.md)                 | マーカー・演算子定義         |
| [03-block-structure.md](../../doc/spec/03-block-structure.md) | ブロック構造                 |
| [04-call-spec.md](../../doc/spec/04-call-spec.md)             | Call仕様・スコープ解決       |
| [06-action-line.md](../../doc/spec/06-action-line.md)         | アクション行・インライン要素 |
| [09-variables.md](../../doc/spec/09-variables.md)             | 変数スコープ                 |
| [10-words.md](../../doc/spec/10-words.md)                     | 単語定義・参照               |

パーサー・トランスパイラー・ランタイムの実装判断はすべてこの仕様書に基づく。

---

## マーカー一覧（全角/半角両対応）

| マーカー         | 全角 | 半角 | 用途                         | 参照                                                        |
| ---------------- | ---- | ---- | ---------------------------- | ----------------------------------------------------------- |
| グローバルシーン | `＊` | `*`  | シーン定義                   | [Chapter 2](../../doc/spec/02-markers.md#22-シーンマーカー) |
| ローカルシーン   | `・` | `-`  | サブシーン定義               | [Chapter 2](../../doc/spec/02-markers.md#22-シーンマーカー) |
| 属性             | `＆` | `&`  | メタデータ                   | [Chapter 8](../../doc/spec/08-attributes.md)                |
| 単語/関数        | `＠` | `@`  | 単語定義・参照・関数呼び出し | [Chapter 10](../../doc/spec/10-words.md)                    |
| 変数             | `＄` | `$`  | 変数宣言・参照               | [Chapter 9](../../doc/spec/09-variables.md)                 |
| Call             | `＞` | `>`  | シーン呼び出し               | [Chapter 4](../../doc/spec/04-call-spec.md)                 |
| コメント         | `＃` | `#`  | コメント行                   | [Chapter 2](../../doc/spec/02-markers.md#210-コメント)      |
| コロン           | `：` | `:`  | 区切り                       | [Chapter 2](../../doc/spec/02-markers.md#コロンcolon)       |
| アクター辞書     | `％` | `%`  | アクター辞書定義             | [Chapter 11](../../doc/spec/11-actor-dictionary.md)         |

---

## ドメイン概念

### シーン（Label）
- **グローバルシーン**: `＊シーン名` - プロジェクト全体からアクセス可能
- **ローカルシーン**: `・シーン名` - 親シーン内でのみアクセス可能
- **重複シーン**: 同名で複数定義可能、実行時にランダム選択
- **前方一致検索**: `LabelTable::find_by_prefix`でランダム候補選択

### 変数スコープ
- **ローカル変数**: `＄変数名` - シーン内のみ有効
- **グローバル変数**: `＄＊変数名` - セッション中有効

### 制御フロー
- **Call文** (`＞label`): サブルーチン呼び出し、実行後復帰
- **Luaブロック**: 複雑なロジックをLua言語で記述

---

## 基本パターン

```pasta
＊会話                    # グローバルシーン
＆priority：10            # 属性
  ＠greeting：こんにちは おはよう  # ローカル単語定義
  Alice：＠greeting！\n    # アクション行（単語参照 + Sakura改行）
  ＄count：1              # 変数代入
  ＞次の会話              # Call（ローカルシーン）
  ・次の会話              # ローカルシーン
    Bob：続きです
```

### Luaブロック
````pasta
＊計算
```lua
function calculate(ctx)
    local result = 10 + 20
    return result
end
```
  Alice：結果は ＠calculate() です
````

参照: [Chapter 3: ブロック構造](../../doc/spec/03-block-structure.md#luaブロックの配置)

---

## IR出力（ScriptEvent）

| イベント                             | 用途             |
| ------------------------------------ | ---------------- |
| `Talk { speaker, content }`          | 発言             |
| `Wait { duration }`                  | ウェイトマーカー |
| `BeginSync`, `SyncPoint`, `EndSync`  | 同期制御マーカー |
| `SetVariable { scope, name, value }` | 変数設定         |
| `Error { message }`                  | エラー           |

**設計原則**: Wait/Sync関連はマーカーのみ、areka層が制御を実装。

---

## さくらスクリプト

Pastaは以下をそのままIR出力に含める（解釈はareka層）：
- `\\s[表情ID]`: 表情変更
- `\\w数字`: ウェイト
- `\\n`: 改行
- `\\_w[数字]`: 待機

参照: [Chapter 7: Sakuraスクリプト仕様](../../doc/spec/07-sakura-script.md)

---

## 破壊的変更（2025-12 適用済み）

1. **Jump（？）廃止** → Call（＞）を使用
2. **Sakura エスケープ**: 半角 `\` のみ（全角 `＼` 不可）
3. **Sakura 括弧**: 半角 `[]` のみ（全角 `［］` 不可）
4. **Rune → Lua**: コードブロックはLua言語に移行済み
