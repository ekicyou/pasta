# Research & Design Decisions

---
**Feature**: ukagaka-desktop-mascot
**Discovery Scope**: Meta-Specification（全要件の子仕様への分解）
**Key Findings**:
1. 本仕様は「メタ仕様」であり、31要件+NFR-6を子仕様群に分解して統括する
2. 既存8子仕様（Phase 1作成済み）はMVPフォーカスであり、全要件カバレッジには不十分
3. 追加17子仕様の作成により全要件をカバー、年単位の開発ロードマップを構築
4. wintfはDirectComposition/D2D/bevy_ecs基盤として十分な成熟度を持つ
5. MCPプロトコルをプラットフォーム↔ゴースト間通信に採用
---

## Research Log

### wintf既存アーキテクチャ分析

- **Context**: 基盤フレームワークとして採用するwintfの現状能力を把握
- **Sources Consulted**: 
  - `crates/wintf/src/` ソースコード
  - `doc/spec/` 設計ドキュメント群
  - `.kiro/steering/` プロジェクト方針
- **Findings**:
  - **ECSアーキテクチャ**: bevy_ecs 0.17.2採用、Component/System分離が確立
  - **レイアウト**: taffy 0.9.1統合済み、Flexbox対応、DPI aware
  - **描画**: DirectComposition + Direct2D、透過ウィンドウ対応
  - **テキスト**: DirectWrite統合、縦書き対応（Label widget）
  - **ウィンドウ**: Win32 API、マルチモニター対応、メッセージループ統合
  - **現状のWidget**: Rectangle（色塗り）、Label（テキスト）のみ
- **Implications**:
  - 画像表示（Image widget）は未実装 → MVP必須機能
  - イベントシステムは設計ドキュメントのみ、実装は部分的
  - ヒットテストはドキュメント化されているが実装状況要確認

### MCPプロトコル調査

- **Context**: プラットフォーム↔ゴースト間通信の標準プロトコル選定
- **Sources Consulted**: 
  - https://modelcontextprotocol.io/
  - MCP仕様書
- **Findings**:
  - MCPは「AIアプリケーションと外部システムを接続する標準プロトコル」
  - サーバー/クライアントモデル、ツール呼び出し、リソースアクセスを標準化
  - JSONベースのRPC、stdioまたはHTTP/SSE転送
  - Rust実装（rmcp等）が存在
- **Implications**:
  - プラットフォーム = MCPサーバー（描画、イベント、ゴースト間通信を提供）
  - ゴースト（頭脳） = MCPクライアント（LLM、人格、記憶を管理）
  - 既存のLLMエコシステム（Claude、ChatGPT等）との連携が容易

### SHIORIプロトコル調査

- **Context**: 伺か互換性のためのレガシープロトコル理解
- **Sources Consulted**: 
  - ukadoc（一部アクセス不可）
  - 既存の伺か実装知識
- **Findings**:
  - SHIORIはシェル↔辞書（ゴースト頭脳）間の通信プロトコル
  - GET/NOTIFY/LOADなどのリクエストタイプ
  - DLL形式が主流（32bit）、互換性問題あり
- **Implications**:
  - 完全互換は労力に見合わない（requirements.mdで明示済み）
  - SHIORIのイベント体系は参考になる
  - MCP上でSHIORIライクなイベントマッピングを提供可能

### taffyレイアウトエンジン

- **Context**: wintfで採用済みのレイアウトエンジン能力確認
- **Sources Consulted**: 
  - https://docs.rs/taffy/latest/taffy/
  - wintf内のtaffy統合コード
- **Findings**:
  - Flexbox、Grid、Block layoutをサポート
  - High-level API（TaffyTree）とLow-level APIの両方を提供
  - measure functionによるカスタムサイズ計算（テキスト、画像等）
  - wintfはLow-level APIを使用している模様
- **Implications**:
  - 現代的レイアウトシステムとして十分な能力
  - 旧伺かの絶対座標ベースからの変換が必要

### スクリプトエンジン選定

- **Context**: ゴースト対話スクリプトの記述言語
- **Sources Consulted**: 
  - 里々（Satori）の設計思想
  - Luaエンジン（mlua等）
  - tree-sitter parser
- **Findings**:
  - 里々は「会話を自然に書ける」構文が支持された理由
  - Luaは軽量で組み込み実績豊富
  - カスタムDSLはパーサー開発コストが高い
- **Implications**:
  - MVP: 里々インスパイアのカスタムDSL（簡易版）
  - 拡張: Luaバインディング
  - LLM連携時はスクリプトとLLM応答のハイブリッド

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **ECS + MCP** | bevy_ecs基盤 + MCPプロトコル | wintf既存資産活用、LLM連携容易、責務分離明確 | MCPの学習コスト | **採用** |
| Actorモデル | ゴーストごとにActor | 並行性、分離性 | 複雑性増大、wintfとの統合困難 | 不採用 |
| モノリシック | 全機能を単一プロセス | シンプル | スケーラビリティ、プラグイン困難 | 不採用 |

### 選定理由（ECS + MCP）

1. **wintfとの一貫性**: 既存のbevy_ecsアーキテクチャを継承
2. **責務分離**: プラットフォーム（MCPサーバー）とゴースト（MCPクライアント）の明確な境界
3. **拡張性**: MCPツールとしてプラグイン機能を自然に実装可能
4. **LLM統合**: MCPは元々LLM連携のためのプロトコル

## Design Decisions

### Decision: プラットフォーム↔ゴースト通信にMCPを採用

- **Context**: ゴースト（頭脳）とプラットフォーム間の通信プロトコルが必要
- **Alternatives Considered**:
  1. SHIORI互換プロトコル — 32bit DLL問題、レガシー負債
  2. カスタムRPCプロトコル — 車輪の再発明
  3. MCP — 標準化済み、LLM連携との親和性
- **Selected Approach**: MCP
- **Rationale**: 
  - 2025年の技術としてLLM連携は必須
  - MCPはLLMとツール連携のために設計されたプロトコル
  - 標準化により、サードパーティゴーストの開発が容易
- **Trade-offs**: 
  - 学習コスト（新しいプロトコル）
  - SHIORI完全互換は断念
- **Follow-up**: MCP Rust実装（rmcp）の評価
- **Final Decision (2025-11-29)**:
  - MCPの本質は「JSON-RPCの亜種」であり、仕様変更への追従コストは許容範囲
  - SHIORI/SSTPが25年前に同等概念を先行実現しており、技術的に枯れた領域
  - LLMとの連携（頭脳パッケージ）を考慮すると、MCP準拠が将来的に有利
  - 「MCPを無視する積極的理由がない」ため採用確定
  - 実装: rmcp優先、問題あれば独自JSON-RPC（MCPサブセット）へフォールバック

### Decision: パッケージ分離（頭脳/シェル/バルーン）

- **Context**: ゴースト資産の配布・再利用性
- **Alternatives Considered**:
  1. 一体型パッケージ — シンプルだが再利用困難
  2. 分離型パッケージ — 複雑だが柔軟
- **Selected Approach**: 分離型（頭脳/シェル/バルーン独立）
- **Rationale**:
  - requirements.md Requirement 27で明示された要件
  - コミュニティ創作活動の促進
  - 「着せ替え」「バルーン交換」は伺かエコシステムの重要機能
- **Trade-offs**:
  - 依存関係管理の複雑さ
  - マニフェスト仕様の設計が必要
- **Follow-up**: manifest.toml仕様の詳細設計

### Decision: レンダリングパイプラインの責務

- **Context**: 描画の責務をどこに置くか
- **Alternatives Considered**:
  1. シェルが描画コマンドを生成 — 柔軟だがセキュリティリスク
  2. プラットフォームが描画を完全制御 — 安全だが表現力制限
- **Selected Approach**: プラットフォーム主導、シェルは素材提供のみ
- **Rationale**:
  - セキュリティ（シェルに任意コード実行を許さない）
  - 責務境界表（requirements.md）で明示
  - DirectComposition APIの直接操作はプラットフォーム責務
- **Trade-offs**:
  - カスタム描画エフェクトの制限（プラグインで対応）
- **Follow-up**: シェルフォーマット（サーフェス定義、アニメーション定義）の詳細設計

### Decision: スクリプトエンジンの段階的実装

- **Context**: ゴースト対話スクリプトの実行環境
- **Alternatives Considered**:
  1. 里々完全互換 — 実装コスト大、レガシー負債
  2. Lua/Wasm汎用スクリプト — 柔軟だが学習コスト
  3. カスタムDSL — 「会話を自然に書ける」に最適化
- **Selected Approach**: 段階的実装
  - Phase 1: カスタムDSL（里々インスパイア簡易版）
  - Phase 2: Luaバインディング
  - Phase 3: Wasmプラグイン
- **Rationale**:
  - MVPでは「会話を書ける」最小限のDSLで十分
  - 拡張性はLua/Wasmで担保
- **Trade-offs**:
  - 複数のスクリプト環境サポートの複雑さ
- **Follow-up**: カスタムDSL文法の設計

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| MCPの成熟度不足 | 中 | 低 | 標準仕様に準拠、必要に応じてフォールバック |
| パフォーマンス問題（描画） | 高 | 中 | DirectComposition活用、プロファイリング |
| スクリプトエンジンの複雑さ | 中 | 高 | MVPは最小限DSL、段階的拡張 |
| プラグインセキュリティ | 高 | 中 | サンドボックス設計、明示的権限要求 |
| wintfの機能不足 | 中 | 中 | MVP必須機能を優先実装（Image、イベント） |

## References

- [wintf設計ドキュメント](../../../doc/spec/README.md) — ECS/Visual/Layout設計
- [MCP公式サイト](https://modelcontextprotocol.io/) — プロトコル仕様
- [taffy](https://docs.rs/taffy/latest/taffy/) — レイアウトエンジン
- [bevy_ecs](https://docs.rs/bevy_ecs/) — ECSフレームワーク
- [DirectComposition](https://learn.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal) — Windows描画API

---

## 子仕様分解戦略（2025-11-29 追加）

### 目的

本仕様の目標は「全要件定義の子仕様への分解」である。MVPだけでなく、今後数か月〜年単位で駆動可能な開発ロードマップを構築する。

### 要件カバレッジ分析

**既存子仕様（Phase 1作成済み: 8件）**:
1. `wintf-image-widget` — Req 1.1, 1.3, 2.4
2. `wintf-event-system` — Req 5.1, 5.2, 5.3, 5.8
3. `wintf-typewriter` — Req 3.5, 4.7
4. `wintf-clickthrough` — Req 1.6
5. `areka-reference-ghost` — Req 4.1, 4.2, 4.4, 4.5, 4.6, 26.1-26.3
6. `areka-reference-shell` — Req 2.2, 2.7, 8.1, 8.3, 27.10-27.13
7. `areka-reference-balloon` — Req 3.4, 3.6, 27.15, 27.16
8. `areka-window-placement` — Req 1.4, 1.5, 1.7, 9.3, 16.6

**追加必要な子仕様（17件）**:

| 仕様名 | 対象要件 | 優先度 | 説明 |
|--------|---------|--------|------|
| `wintf-animation-system` | 2.1-2.8 | P0 | フレームアニメ、トランジション、連動アニメ |
| `wintf-balloon-system` | 3.1-3.10 | P0 | バルーンウィンドウ、選択肢UI |
| `wintf-dpi-scaling` | 15.1-15.5, NFR-1 | P1 | DPI対応、アクセシビリティ |
| `areka-script-engine` | 4.1-4.10, 29.6-29.8 | P0 | DSL解析、変数、制御構文 |
| `areka-timer-events` | 6.1-6.8 | P1 | タイマー、システムイベント |
| `areka-package-manager` | 7.1-7.7, 8.1-8.5, 27.1-27.27, 31.1-31.9 | P0 | パッケージ管理、メタ情報 |
| `areka-persistence` | 9.1-9.6, 30.6-30.8 | P0 | 設定、状態保存、自動保存 |
| `areka-mcp-server` | 10.1-10.5, 26.11-26.15 | P0 | MCP基盤、ゴースト間通信 |
| `areka-legacy-converter` | 11.1-11.6, 29.1-29.11 | P1 | フォーマット変換、互換プロトコル |
| `areka-devtools` | 12.1-12.7, 28.1-28.10 | P1 | デバッグ、ホットリロード |
| `areka-system-tray` | 13.1-13.5 | P0 | システムトレイ常駐 |
| `areka-presence-style` | 16.1-16.7 | P2 | 存在スタイル、控えめ〜活発 |
| `areka-memory-system` | 17.1-17.8 | P2 | 会話履歴、記憶、RAG |
| `areka-llm-integration` | 18.1-18.7, 26.7-26.10, 26.16-26.20 | P2 | LLMバックエンド、キャラ間LLM会話 |
| `areka-voice-system` | 19.1-19.8 | P3 | TTS/STT、音声対話 |
| `areka-screen-awareness` | 20.1-20.8 | P3 | 画面認識、状況理解 |
| `areka-environment-sense` | 21.1-21.7, 22.1-22.7 | P3 | 外部連携、環境認識 |
| `areka-cloud-sync` | 23.1-23.6 | P3 | マルチデバイス同期 |
| `areka-creator-tools` | 24.1-24.8 | P2 | 創作支援、テンプレート |
| `areka-privacy-security` | 25.1-25.5, NFR-3 | P2 | プライバシー、暗号化 |
| `areka-character-communication` | 26.1-26.37 | P1 | キャラクター間会話（基本〜高度） |
| `areka-ide-integration` | 28.11-28.20 | P3 | DAP/LSPサーバー |
| `areka-error-recovery` | 30.1-30.10 | P1 | クラッシュログ、状態復元 |

**統合・重複整理**:
- `areka-package-manager`: Req 7, 8, 27, 31 を統合
- `areka-character-communication`: Req 26 の全項目を包括
- `areka-environment-sense`: Req 21, 22 を統合

### 子仕様分類規則

**プレフィックス**:
- `wintf-*`: UIフレームワーク基盤（描画、イベント、レイアウト）
- `areka-*`: アプリケーション層（ゴースト、シェル、通信、設定）

**優先度**:
- **P0 (MVP)**: 最小限の動作に必須
- **P1 (リリース必須)**: 外部公開に必要
- **P2 (差別化)**: 競合優位性を生む機能
- **P3 (将来拡張)**: 長期的なビジョン

### 依存関係階層

```
Tier 0 (基盤)
├── wintf-image-widget
├── wintf-event-system
├── wintf-typewriter
└── wintf-clickthrough

Tier 1 (描画・表示)
├── wintf-animation-system (依存: Tier 0)
├── wintf-balloon-system (依存: wintf-typewriter)
├── wintf-dpi-scaling (依存: Tier 0)
└── areka-window-placement (依存: wintf-event-system)

Tier 2 (コア機能)
├── areka-script-engine (依存: Tier 1)
├── areka-package-manager (独立)
├── areka-persistence (独立)
├── areka-mcp-server (独立)
├── areka-system-tray (独立)
└── areka-error-recovery (独立)

Tier 3 (参照実装)
├── areka-reference-ghost (依存: areka-script-engine, areka-mcp-server)
├── areka-reference-shell (依存: wintf-animation-system)
└── areka-reference-balloon (依存: wintf-balloon-system)

Tier 4 (高度機能)
├── areka-timer-events (依存: areka-mcp-server)
├── areka-devtools (依存: areka-script-engine)
├── areka-legacy-converter (独立)
├── areka-presence-style (依存: areka-mcp-server)
├── areka-character-communication (依存: areka-mcp-server)
└── areka-privacy-security (依存: areka-persistence)

Tier 5 (拡張機能)
├── areka-memory-system (依存: areka-persistence)
├── areka-llm-integration (依存: areka-mcp-server)
├── areka-creator-tools (依存: areka-package-manager)
└── areka-ide-integration (依存: areka-devtools)

Tier 6 (将来機能)
├── areka-voice-system (依存: areka-llm-integration)
├── areka-screen-awareness (依存: areka-mcp-server)
├── areka-environment-sense (依存: areka-mcp-server)
└── areka-cloud-sync (依存: areka-persistence)
```

### 要件→子仕様トレーサビリティ

| Req | 子仕様 | カバー範囲 |
|-----|--------|-----------|
| 1 | wintf-image-widget, areka-window-placement, wintf-clickthrough | 全項目 |
| 2 | wintf-animation-system, wintf-image-widget, areka-reference-shell | 全項目 |
| 3 | wintf-balloon-system, wintf-typewriter, areka-reference-balloon | 全項目 |
| 4 | areka-script-engine, areka-reference-ghost, areka-llm-integration | 全項目 |
| 5 | wintf-event-system | 全項目 |
| 6 | areka-timer-events | 全項目 |
| 7 | areka-package-manager | 全項目 |
| 8 | areka-package-manager, areka-reference-shell | 全項目 |
| 9 | areka-persistence, areka-window-placement | 全項目 |
| 10 | areka-mcp-server | 全項目 |
| 11 | areka-legacy-converter | 全項目 |
| 12 | areka-devtools | 全項目 |
| 13 | areka-system-tray | 全項目 |
| 14 | (NFR、各仕様に分散) | 全項目 |
| 15 | wintf-dpi-scaling | 全項目 |
| 16 | areka-presence-style, areka-window-placement | 全項目 |
| 17 | areka-memory-system | 全項目 |
| 18 | areka-llm-integration | 全項目 |
| 19 | areka-voice-system | 全項目 |
| 20 | areka-screen-awareness | 全項目 |
| 21 | areka-environment-sense | 全項目 |
| 22 | areka-environment-sense | 全項目 |
| 23 | areka-cloud-sync | 全項目 |
| 24 | areka-creator-tools | 全項目 |
| 25 | areka-privacy-security | 全項目 |
| 26 | areka-character-communication, areka-reference-ghost, areka-llm-integration | 全項目 |
| 27 | areka-package-manager | 全項目 |
| 28 | areka-devtools, areka-ide-integration | 全項目 |
| 29 | areka-legacy-converter | 全項目 |
| 30 | areka-error-recovery | 全項目 |
| 31 | areka-package-manager | 全項目 |
| NFR-1 | wintf-dpi-scaling, (全般) | 全項目 |
| NFR-2 | areka-mcp-server, (プラグイン設計) | 全項目 |
| NFR-3 | areka-privacy-security, wintf-clickthrough | 全項目 |
| NFR-4 | (ドキュメント) | 全項目 |
| NFR-5 | (リソース設計) | 全項目 |
| NFR-6 | (パフォーマンス設計) | 全項目 |

