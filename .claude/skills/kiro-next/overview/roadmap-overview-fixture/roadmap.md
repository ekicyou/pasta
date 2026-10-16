# Roadmap — 見本（roadmap-overview.ps1 -SelfTest の入力・本物の roadmap.md の難所を写した断片）

> この断片は道具の自分の検査だけが読む。中身は架空の spec で、本物の roadmap.md とは関係しない。

## 概要

`alpha-one` の話（棚卸の節より前の節はテーマにしない）。

## 完了フェーズ（要約）

| Phase | 主題 | spec |
| ----- | ---- | ---- |
| 1–2 | 古い土台（括弧の。は切らない）。後ろの文は一言に入らない | old-base, done-spec（v0.0.1 で公開・`not-a-name-x` は注） |
| — | 二つ目の主題 | old-two |

### 常駐 spec（節の 2 つ目の表＝読まない）

| Phase | 主題 | spec |
| -- | -- | -- |
| x | 読まない表 | zz-ignored |

## 0.9.0 までの道筋（2026-01-01 開発者の決定）

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 完了 | done-spec | Fable | 13（実績） | 2026-01-01 に完了 |
| 1（高優先度） | alpha-one | Opus | 10〜13 | 今すぐ（`a|b` と \| を書いても割れない） |
| 2 | beta-two | Opus（見直し） | 5 | `alpha-one` の後 |
| 3 | 0.9.0 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

| spec | 子孫 |
| ---- | ---- |
| zz-ignored | 見出しが「順｜spec」でない表は帯にしない |

## 棚卸（2026-01-02・main `0123abcd`）

棚卸の本文（`gamma-three` が出てもテーマにしない）。

### ウェーブ（2026-01-02）

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| alpha-one | 約 1 | 機能 | 10〜13 | Opus |
| `gamma-three` | 0 | 文書 | 3 | Opus |

```
Wave 2本
Opus
/kiro-start alpha-one
/kiro-start gamma-three
```

### 開発者の判断が要るもの

- なし。

## Phase 8 の残り: 土台（注は外す）

| Wave | spec（並走可） |
| ---- | -------------- |
| 次 | gamma-three |

`alpha-one` は境界の話。`beta-two` の名前も出るが、居場所は次の節の箇条。

## Phase 9: 見本の夢（1.0.0）

### 順番

| 順 | spec | 要件定義（見当） | 規模（見当） | 先に要るもの |
| -- | ---- | ---------------- | ------------ | ------------ |
| 1 | gamma-three（既存） | Fable | 15〜18 | なし |
| 2 | 1.0.0 のリリース | Opus | — | 上の全部 |

- `beta-two` — 鎖の 2 本目（ここが居場所）。

`eps-done` の話もここに置く。

## 名前の境目の節

`delta-free-old` は別の名前（名前の境目で当てるので、この節は台帳の 4 本目のテーマにならない）。

## Specs (dependency order)

- [ ] alpha-one -- 一言は台帳の何をするかから取り、六十字を越える長い文は六十字で切って末尾に省略の印を足して見せる、どの帯にも置かれていない spec も同じ決まりで一言を作る（括弧の注は外す）。後ろの文。Dependencies: none（`beta-two` は注）
- [ ] beta-two -- 鎖の 2 本目を張る。Dependencies: alpha-one, done-spec（注の omega-x は数えない）
- [ ] gamma-three -- 見本の「夢」を叶える。Dependencies: ghost-of-past
- [ ] delta-free -- 台帳の文に `</script>` が在っても壊れない。Dependencies: none
- [x] eps-done -- 済んだがまだ畳んでいない。Dependencies: alpha-one

## Existing Spec Updates

- [ ] alpha-one -- 申し送りの行（読まない）。Dependencies: zz-ignored
