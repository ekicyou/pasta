# Brief: audit-pasta-core

## Problem
pasta_coreクレートは長期開発の中で成長してきたが、脆弱性の体系的な調査やコード簡素化が行われていない。デッドコード、冗長な表現、不要な`#[allow(dead_code)]`が蓄積している可能性がある。

## Current State
- ~600行のソースコード（src/ 3ファイル）
- レジストリ層（WordTable、SceneTable等）の純粋なデータ構造
- 外部依存: `thiserror`, `fast_radix_trie`, `rand`, `tracing`
- 他クレートからの依存が多い基盤クレート

## Desired Outcome
- 全脆弱性カテゴリ（入力検証、エラーハンドリング、パニック安全性）の調査完了
- デッドコード除去、冗長表現削減によるコード量削減
- 既存テスト（950+全体）がリグレッションなく全パス
- 外部振る舞い不変

## Approach
クレート内完結型監査。APIは変更せず、内部実装のみを対象に脆弱性回避とコード簡素化を行う。

## Scope
- **In**: pasta_core/src/ 全ファイルの脆弱性調査、デッドコード除去、冗長表現削減、アルゴリズム改善
- **Out**: APIシグネチャの変更、新機能追加、他クレートへの変更

## Boundary Candidates
- WordTable / SceneTable のデータ構造操作
- レジストリ登録・検索ロジック
- エラー型定義

## Out of Boundary
- pasta_dsl以降のクレートの変更
- レジストリの設計変更（新しいテーブル種追加など）
- fast_radix_trie の内部実装

## Upstream / Downstream
- **Upstream**: なし（基盤クレート）
- **Downstream**: pasta_dsl, pasta_lua, pasta_shiori が依存

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: audit-pasta-dsl, audit-pasta-lua（依存先として影響を受ける可能性）

## Constraints
- 外部振る舞い（公開API）不変
- 既存テスト全パス必須
- 性能劣化禁止
