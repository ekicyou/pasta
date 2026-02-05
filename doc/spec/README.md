# Pasta DSL 言語仕様書

このディレクトリは、Pasta DSL の正式な言語仕様書を章別に管理します。

## 概要

**Pasta DSL** は、「伺か」デスクトップマスコットやシナリオ型ゲーム向けの対話スクリプト言語です。  
本仕様書は、パーサー・トランスパイラー・ランタイムの実装判断における**権威的ソース**です。

---

## 章一覧

| 章番号 | ファイル                                         | 内容                             |
| ------ | ------------------------------------------------ | -------------------------------- |
| 1      | [01-grammar-model.md](01-grammar-model.md)       | 文法モデルの基本原則             |
| 2      | [02-markers.md](02-markers.md)                   | キーワード・マーカー定義         |
| 3      | [03-block-structure.md](03-block-structure.md)   | 行とブロック構造                 |
| 4      | [04-call-spec.md](04-call-spec.md)               | Call の詳細仕様                  |
| 5      | [05-literals.md](05-literals.md)                 | リテラル型                       |
| 6      | [06-action-line.md](06-action-line.md)           | アクション行（Action Line）      |
| 7      | [07-sakura-script.md](07-sakura-script.md)       | Sakura スクリプト仕様            |
| 8      | [08-attributes.md](08-attributes.md)             | 属性（Attribute）                |
| 9      | [09-variables.md](09-variables.md)               | 変数・スコープ                   |
| 10     | [10-words.md](10-words.md)                       | 単語定義（Word Definition）      |
| 11     | [11-actor-dictionary.md](11-actor-dictionary.md) | アクター辞書（Actor Dictionary） |
| 12     | [12-future.md](12-future.md)                     | 未確定事項・検討中の仕様         |

---

## よくある参照パターン

AI/開発者が仕様を参照する際の典型的なケースと、どの章を参照すべきかのガイドです。

| 知りたいこと                                 | 参照する章                                         |
| -------------------------------------------- | -------------------------------------------------- |
| マーカー（`＊`、`・`、`＠`等）の意味         | [Chapter 2: マーカー定義](02-markers.md)           |
| Call文の動作・引数渡し                       | [Chapter 4: Call詳細仕様](04-call-spec.md)         |
| アクション行の構文                           | [Chapter 6: アクション行](06-action-line.md)       |
| さくらスクリプト（`\\s[]`等）の仕様          | [Chapter 7: Sakuraスクリプト](07-sakura-script.md) |
| 変数スコープ（ローカル/グローバル/システム） | [Chapter 9: 変数・スコープ](09-variables.md)       |
| 単語定義と参照                               | [Chapter 10: 単語定義](10-words.md)                |
| アクター固有の辞書                           | [Chapter 11: アクター辞書](11-actor-dictionary.md) |
| コードブロック（Lua）の使い方                | [Chapter 3: ブロック構造](03-block-structure.md)   |

---

## 関連ドキュメント

- [GRAMMAR.md](../../GRAMMAR.md) - 人間向けクイックリファレンス（例文豊富な学習用資料）
- [SOUL.md](../../SOUL.md) - プロジェクト憲法（コアバリュー・設計原則）
- [README.md](../../README.md) - プロジェクト概要

**役割分担**:
- **本仕様書** (`doc/spec/`): 実装判断の権威的ソース（What/How）
- **GRAMMAR.md**: 利用者向け学習資料（読みやすさ優先）
- **steering/grammar.md**: AI向け完全参照（本仕様書に準拠）

---

## 参考資料

### 外部仕様

| 項目             | 参照先                                                                                          |
| ---------------- | ----------------------------------------------------------------------------------------------- |
| Unicode識別子    | [UAX #31](https://unicode.org/reports/tr31/) - XID_START / XID_CONTINUE                         |
| さくらスクリプト | [ukadoc さくらスクリプトリスト](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html) |

### ディレクトリ構成

```
doc/spec/
├── README.md                  # このファイル（インデックス）
├── 01-grammar-model.md        # 章1: 文法モデルの基本原則
├── 02-markers.md              # 章2: キーワード・マーカー定義
├── 03-block-structure.md      # 章3: 行とブロック構造
├── 04-call-spec.md            # 章4: Call の詳細仕様
├── 05-literals.md             # 章5: リテラル型
├── 06-action-line.md          # 章6: アクション行
├── 07-sakura-script.md        # 章7: Sakura スクリプト仕様
├── 08-attributes.md           # 章8: 属性
├── 09-variables.md            # 章9: 変数・スコープ
├── 10-words.md                # 章10: 単語定義
├── 11-actor-dictionary.md     # 章11: アクター辞書
└── 12-future.md               # 章12: 未確定事項・検討中の仕様
```
