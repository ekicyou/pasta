# Requirements Document

## Project Description (Input)
`.pasta` ソースレベルデバッグの利用者（pasta ゴースト作者）が、ブレークポイントで停止後に F5（Continue）を押しても、同じ `.pasta` 行から抜け出せず再び同じ行で停止してしまう不具合を修正する。`.pasta` 行 BP は「`.pasta` 行を訪問するごとに1回だけ」発火し、Continue は同じ `.pasta` 行に対応する残りの `.lua` 行を消化して次の停止点まで進むようにする。`SourceMode::Lua`（`.lua` 粒度）時は従来どおり `.lua` 行ごとに停止する。詳細は `brief.md` を参照。

## Introduction

pasta の `.pasta` ソースレベルデバッグでは、1つの `.pasta` 行がトランスパイル後の複数の `.lua` 行へ展開される。現状、`.pasta` 行に張ったブレークポイントは対応する全 `.lua` 行で命中判定されるため、Continue（F5）で再開しても同じ `.pasta` 行を指す次の `.lua` 行で即座に再ブレークし、利用者は1回の Continue で当該行から抜け出せない。

本仕様は、Pasta 提示モード（`.pasta` 粒度）での **ブレークポイント命中と Continue の再開挙動を `.pasta` 行粒度へ揃える**。`.pasta` 行 BP は「その `.pasta` 行を訪問するたびに高々1回」発火し、Continue は同じ `.pasta` 行に属する残りの `.lua` 行を消化して次の異なる停止点へ進む。ループや再帰で同じ `.pasta` 行へ改めて到達した場合は、新たな訪問として再び停止する。Lua 提示モード・ソースマップ非在・デバッグ無効時の挙動は不変とする。

利用者から観測できる対象は「VSCode 等の DAP クライアント上での停止位置・停止回数・停止理由」であり、内部のフック実装や停止状態機械の構造は本仕様の関心外（design フェーズで扱う）。

## Boundary Context

- **In scope**:
  - Pasta 提示モードにおける、`.pasta` 行ブレークポイント命中の `.pasta` 行粒度化（1訪問あたり高々1停止）。
  - Continue（F5）再開時の、同一 `.pasta` 行に属する残り `.lua` 行の消化（再ブレーク抑制）と、次の異なる停止点までの進行。
  - ループ・再帰による同一 `.pasta` 行の再訪時の再停止。
- **Out of scope**:
  - 提示モード（`.pasta`/`.lua`）の実行時トグルや VSCode UI（隣接仕様 `pasta-debug-lua-view-toggle` が所有）。
  - 条件付きブレークポイント・ログポイント等、新規ブレークポイント種別の追加。
  - ソースマップ生成（`code_gen`）の変更。`.pasta` 1行 → 複数 `.lua` 行という多対1対応は正常仕様として前提する。
  - 同一 `.pasta` 行への**直接再帰**・同一 `.pasta` 行の**別コルーチン実行**における訪問ごと再停止の*厳密保証*（R2.3 のベストエフォート扱い。保証対象はループ再訪 R2.2）。
- **Adjacent expectations**:
  - ソースマップは正しい（`.lua` 行 ↔ `.pasta` 位置の双方向対応が確定的）ことを前提とする。
  - 既存のステップ実行（over/into/out）は既に `.pasta` 行粒度で「同一 `.pasta` 行を消化」する挙動を持つ。Continue の新挙動はこれと一貫させ、ステップ挙動を退行させない。
  - 隣接仕様 `pasta-debug-lua-view-toggle` でモードが `.lua` に切り替わった場合、本仕様の集約は作用しない（`.lua` 粒度に従う）。

## Requirements

### Requirement 1: Continue による現 `.pasta` 行からの離脱
**Objective:** As a `.pasta` をデバッグするゴースト作者, I want ブレーク停止後に Continue を1回押すと現在の `.pasta` 行から確実に抜けること, so that 何度も F5 を押さずにステップを前進できる

#### Acceptance Criteria
1. While Pasta 提示モードかつソースマップが有効である, when 利用者が `.pasta` 行ブレークポイントでの停止中に Continue を発行した, the Pasta デバッグバックエンド shall 実行を再開し、停止していた `.pasta` 行と同じ `.pasta` 行に対応する `.lua` 行では再停止しない。
2. While 上記の Continue による再開中である, when 実行が停止元と異なる `.pasta` 行に到達し、その行に有効なブレークポイントがある, the Pasta デバッグバックエンド shall その `.pasta` 行で停止する。
3. When 同一 `.pasta` 行の消化後に有効なブレークポイントもステップ目標も存在しない, the Pasta デバッグバックエンド shall 余計な再停止なく次の自然な停止点または実行完了まで継続する。

### Requirement 2: `.pasta` 行ブレークの1訪問1停止と再訪時の再停止
**Objective:** As a `.pasta` をデバッグするゴースト作者, I want `.pasta` 行 BP がその行の訪問ごとに1回だけ止まり、ループで戻れば再び止まること, so that 停止回数が `.pasta` のソース行と直感的に一致する

#### Acceptance Criteria
1. While 実行が同一 `.pasta` 行に対応する複数の `.lua` 行を通過している, the Pasta デバッグバックエンド shall その訪問について高々1回だけブレークポイント停止を報告する。
2. When 実行が当該 `.pasta` 行を離れて別の対応 `.pasta` 行へ移った後、ループで同じ `.pasta` 行へ改めて到達した, the Pasta デバッグバックエンド shall その新たな訪問で再びブレークポイント停止を報告する。
3. When 実行が同一 `.pasta` 行へ直接再帰した、または同一 `.pasta` 行が別コルーチンで実行された, the Pasta デバッグバックエンド should 可能な範囲で新たな訪問として再停止する（厳密な再停止保証は要求しない。R2.2 のループ再訪が保証対象）。
4. When 実行が同一 `.pasta` 行へ初めて到達した, the Pasta デバッグバックエンド shall その `.pasta` 行で1回停止し、停止位置を当該 `.pasta` ソース・行として報告する。

### Requirement 3: 停止位置・停止理由の一貫性
**Objective:** As a DAP クライアントを使う利用者, I want 停止位置と停止理由が従来どおり正しく提示されること, so that デバッガ UI 上で停止の意味を取り違えない

#### Acceptance Criteria
1. When Pasta デバッグバックエンドが `.pasta` 行ブレークポイントで停止する, the Pasta デバッグバックエンド shall 当該 `.pasta` ソース・行を停止位置として提示し、ブレークポイント由来の停止理由を報告する。
2. When 集約により同一 `.pasta` 行の後続 `.lua` 行での再ブレークを抑制した, the Pasta デバッグバックエンド shall 利用者に対して追加の停止イベントを発生させない。

### Requirement 4: 提示モードの直交性と後方互換
**Objective:** As a 既存の Lua レベルデバッグ利用者, I want 本修正が Lua 提示やデバッグ無効時の挙動を変えないこと, so that 既存のデバッグワークフローが退行しない

#### Acceptance Criteria
1. While 提示モードが `.lua` である, the Pasta デバッグバックエンド shall `.lua` 行ごとの停止粒度を維持し、`.pasta` 行集約を適用しない。
2. Where ソースマップが存在しない、またはデバッグが無効である, the Pasta デバッグバックエンド shall 本仕様導入前の既存挙動を変更しない。
3. The Pasta デバッグバックエンド shall デバッグ無効（OFF）経路の実行を本仕様導入前とバイト不変に保つ。

### Requirement 5: ステップ実行との一貫性
**Objective:** As a `.pasta` をステップ実行する利用者, I want Continue の新挙動が既存のステップ挙動と一貫すること, so that ステップと Continue で「現在の `.pasta` 行」の概念がぶれない

#### Acceptance Criteria
1. The Pasta デバッグバックエンド shall 既存の `.pasta` 粒度ステップ（over/into/out。同一 `.pasta` 行・未対応行の消化、コルーチン跨ぎ含む）の挙動を退行させない。
2. When 利用者が `.pasta` 行ブレークポイント停止からステップまたは Continue のいずれかで再開する, the Pasta デバッグバックエンド shall いずれの場合も停止元の `.pasta` 行を離れるまで同一 `.pasta` 行で再停止しない一貫した挙動を示す。

### Requirement 6: 無回帰と検証可能性
**Objective:** As a プロジェクト保守者, I want 本修正が再現可能な証跡で検証されること, so that 修正の有効性と無回帰を確証できる

#### Acceptance Criteria
1. The Pasta デバッグバックエンド shall 既存の Lua レベルデバッグおよび既存の自動テストに対して無回帰であること。
2. The Pasta デバッグバックエンド shall Requirement 1 および Requirement 2 の振る舞いを実 DAP-over-TCP の端到端（E2E）試験で検証可能とすること。当該試験は最低限、(a) 1つの `.pasta` 行が複数の `.lua` 行へ展開される構成、および (b) 同一 `.pasta` 行をループで再訪する構成を網羅する。
