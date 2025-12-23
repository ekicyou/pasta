# リサーチ & 設計決定: parser2-filescope-bug-fix

---
**目的**: 設計決定の根拠と発見事項を記録し、技術設計を補完する。
---

## サマリー
- **Feature**: `parser2-filescope-bug-fix`
- **Discovery Scope**: Extension（既存システムの修正）
- **Key Findings**:
  - grammar.pest `file = ( file_scope | global_scene_scope )*` は交互出現を許容
  - 現行バグ: `file.file_scope = ...` の上書き代入で複数file_scopeが消失
  - 3バリアント FileItem 設計（FileAttr, GlobalWord, GlobalSceneScope）で順序保持

## リサーチログ

### Topic 1: grammar.pest との整合性分析
- **Context**: parser2が文法仕様に違反しているバグを修正するため、文法定義を精査
- **Sources Consulted**:
  - `src/parser2/grammar.pest:222` - `file = _{ SOI ~ ( file_scope | global_scene_scope )* ~ s ~ EOI }`
  - Pest 2.8 公式ドキュメント（`*` 演算子 = 0回以上）
- **Findings**:
  - `( file_scope | global_scene_scope )*` は交互出現・任意回数を許容
  - 各要素（file_scope, global_scene_scope）は0回以上出現可能
  - 順序情報を保持するAST設計が必要
- **Implications**: Vec<FileItem> で順序を保持し、3バリアント（FileAttr, GlobalWord, GlobalSceneScope）で表現

### Topic 2: 既存コード影響分析
- **Context**: AST構造変更によるコンパイルエラー範囲を特定
- **Sources Consulted**:
  - `src/parser2/ast.rs:62-73` - PastaFile構造体
  - `src/parser2/mod.rs:135-138` - バグのあるパーサーループ
  - `tests/parser2_integration_test.rs` - 6箇所で file.file_scope への直接アクセス
- **Findings**:
  - 影響範囲は parser2 モジュールとそのテストのみ
  - 既存 transpiler（src/transpiler/）は parser::PastaFile を使用（別型、影響なし）
  - transpiler2 は未実装のため影響なし（むしろ新AST構造を期待）
- **Implications**: 破壊的変更は限定的、コンパイルエラーで修正漏れ防止

### Topic 3: FileItem 3バリアント設計の根拠
- **Context**: 要件定義レビューで開発者から「FileAttr・GlobalWord・GlobalSceneScope の3バリアント」が指示された
- **Sources Consulted**:
  - 要件定義ディスカッション記録（2025-12-23）
  - transpiler2-layer-implementation 仕様のブロッキング依存
- **Findings**:
  - file_scope 内の attrs と words を個別バリアントとして分離
  - transpiler2 Pass 1 でシーケンシャル処理を簡潔化
  - FileScope コンテナを廃止し、フラットな構造に
- **Implications**: `enum FileItem { FileAttr(Attr), GlobalWord(KeyWords), GlobalSceneScope(GlobalSceneScope) }`

## アーキテクチャパターン評価

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 3バリアントFileItem | FileAttr/GlobalWord/GlobalSceneScope | 順序完全保持、transpiler2対応、シンプル | 破壊的変更 | **推奨** |
| B: FileScope累積 | Vec<FileScope> + Vec<GlobalSceneScope> | 変更最小 | 順序保持不可（要件2違反） | 非推奨 |
| C: ハイブリッド | 段階的移行 | リスク分散 | 工数増大、複雑化 | 非推奨 |

## 設計決定

### Decision: FileItem 3バリアント列挙型の採用
- **Context**: file_scope と global_scene_scope の交互出現順序を保持しつつ、transpiler2 での処理を簡潔化
- **Alternatives Considered**:
  1. Option A: 3バリアント FileItem（FileAttr, GlobalWord, GlobalSceneScope）
  2. Option B: FileScope コンテナを維持（2バリアント: FileScope, GlobalSceneScope）
  3. Option C: Vec<FileScope> + Vec<GlobalSceneScope> 分離
- **Selected Approach**: Option A - 3バリアント FileItem
- **Rationale**:
  - 開発者からの明確な指示（FileScope コンテナ廃止）
  - transpiler2 Pass 1 での処理簡潔化（attrs/words を順次処理）
  - grammar.pest `( file_scope | global_scene_scope )*` の意図を忠実に反映
- **Trade-offs**:
  - (+) 順序保持が自明（Vec の順序 = ファイル記述順序）
  - (+) パターンマッチが直感的
  - (-) FileScope 型は parse_file_scope() の一時的な戻り値のみに使用
- **Follow-up**: parse_file_scope() の戻り値型を維持するか、attrs/words を直接返すか検討

### Decision: ヘルパーメソッドの提供
- **Context**: 破壊的変更後の利便性確保
- **Selected Approach**: `file_attrs()`, `words()`, `global_scene_scopes()` の3メソッド提供
- **Rationale**: transpiler2 で型別にアイテムを取得するユースケースが頻出

## リスク & 緩和策
- **Risk 1**: テストコード修正漏れ → **Mitigation**: フィールド廃止によりコンパイルエラー発生、漏れ検出
- **Risk 2**: transpiler2 との API ミスマッチ → **Mitigation**: transpiler2 仕様と事前整合確認済み
- **Risk 3**: パフォーマンス低下 → **Mitigation**: Vec push は O(1) 償却、影響なし

## 参考資料
- [Pest 2.8 Documentation](https://pest.rs/book/) - 文法演算子の仕様
- `src/parser2/grammar.pest` - Pasta DSL 文法定義
- `.kiro/specs/parser2-filescope-bug-fix/gap-analysis.md` - 詳細ギャップ分析
