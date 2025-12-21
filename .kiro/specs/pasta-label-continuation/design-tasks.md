# pasta-label-continuation 設計分析後の作業項目と議題進行

## フェーズ: 設計（承認前）

設計ドキュメント及び設計分析レポートを踏まえて、修正点・疑問点・不安点を収拾します。自明な修正はコミット、確認が必要な項目は議題として1つずつディスカッション進行します。

---

## 設計検証結果サマリー（参考）

**GO決定**: 設計は実装準備完了。要件を完全にカバーし、アーキテクチャ整合性を維持。

**改善提案（任意）**:
1. Error Handling セクションにエラーメッセージの例を追加
2. Testing Strategy で後方互換性テストを強調

---

## 自明な修正（コミット対象）

現時点で自明な修正候補を抽出中...

### 修正1: エラーメッセージ例の追加
**内容**: Error Handling セクションに具体的なエラーメッセージ例を追加し、実装時の参考とする。  
**理由**: 設計検証で「例があると実装時の判断がスムーズ」との指摘あり。  
**対象箇所**: design.md「Error Handling」セクション（L346以降）  
**修正方針**: 既に記載されている「先頭無名」「名前付きシーン未出現時」のエラー説明を、具体例として明確化。

### 修正2: 後方互換性テストの強調
**内容**: Testing Strategy セクションに既存DSLスクリプトの回帰テスト方針を明示。  
**理由**: 設計検証で「後方互換性が重要なので、テスト戦略で明記すべき」との指摘あり。  
**対象箇所**: design.md「Testing Strategy」セクション（L368以降）  
**修正方針**: Integration Tests に「既存DSL fixtures回帰テスト」項目を追加。

---

## 議題（開発者確認必要）

現時点で確認が必要な議題を抽出中...

### 議題候補の評価

#### 候補A: parse_global_scene のシグネチャ変更影響
**内容**: `parse_global_scene` に `last_scene_name: Option<String>` パラメータを追加するが、この関数が他モジュールから呼ばれている可能性。  
**調査必要性**: 現在のコードベースで `parse_global_scene` の呼び出し元を確認し、影響範囲を特定。  
**判定**: 実装時調査で十分（設計フェーズでの議題化は不要）。

#### 候補B: Pest 2.8 の `label_name?` 構文サポート確認
**内容**: 設計では `label_name?` でオプション化するとしているが、Pest 2.8 で正式にサポートされているか。  
**調査必要性**: Pest ドキュメント確認、または簡易な文法テストで検証。  
**判定**: 自明（Pest PEG では `?` はオプショナル修飾子として標準サポート）。議題化不要。

#### 候補C: エラーメッセージの日英混在方針
**内容**: design.md では日本語メッセージ例を示しているが、コードベースではエラーメッセージは英語か日本語か。  
**調査必要性**: 既存の `PastaError::ParseError` メッセージを確認し、統一方針を把握。  
**判定**: 既存コード確認後に判断。

---

## 調査結果: 既存エラーメッセージの言語方針

**確認完了**: `src/error.rs` および `src/parser/mod.rs` を確認した結果、既存のエラーメッセージはすべて英語で記述されている。

**例**:
- `"Scene name '{}' is reserved for system use. scene names starting and ending with '__' are not allowed."`
- `"Parse error at {file}:{line}:{column}: {message}"`

**影響**: design.md のエラーメッセージ例も英語で統一すべき。

---

## 自明な修正実行完了（コミット準備完了）

### 修正1: エラーメッセージ例を英語で追加 ✅
**実行内容**: design.md L352-358（Error Handling セクション）にエラーメッセージの具体例を英語で追加。  
**変更内容**:
- 先頭無名「＊」エラー: `"Unnamed global scene at the start of file. The first scene definition must have an explicit name. Consider adding a scene name after the '＊' marker."`
- 名前付きシーン未出現時の無名「＊」エラー: `"Unnamed global scene with no prior named scene for continuation. A named global scene must appear before using unnamed '＊' markers."`
- 既存コード（`src/parser/mod.rs`）のメッセージパターン（構造的説明 + 提案）に合わせた表現を使用

### 修正2: 後方互換性テストを明示的に強調 ✅
**実行内容**: design.md L383-386（Testing Strategy セクション）に「Backward Compatibility Validation」項目を追加。  
**変更内容**:
- 既存DSLファイル（`tests/fixtures/`全ファイル）が引き続き正常にパース可能であることを検証
- 既存テストスイート（`cargo test --all`）が全て通過することを確認
- 名前付き「＊」のみのDSLスクリプトが、パーサー拡張後も同一のASTを生成することを検証

---

## 議題リスト（開発者確認必要）

**結論**: 開発者確認が必要な議題は現時点で存在しない。
