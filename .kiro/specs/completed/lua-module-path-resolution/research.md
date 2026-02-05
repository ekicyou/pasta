# Research & Design Decisions: lua-module-path-resolution

## Summary
- **Feature**: `lua-module-path-resolution`
- **Discovery Scope**: Extension - 既存システムの修正
- **Key Findings**:
  - 既存コードで`lua.load(r#"require(...)"#)`パターンが使用されている（runtime/mod.rs L743）
  - entry.luaとscene_dic.luaは直接ファイル読み込みで、Lua検索パスを使用していない
  - scene_dic.luaはキャッシュディレクトリに動的生成されるが、検索パス内のため解決可能

## Research Log

### requireヘルパー関数の実装方式

- **Context**: Rust側からLuaの`require()`を安全に呼び出す方法の調査
- **Sources Consulted**: 
  - mlua 0.11 ドキュメント
  - 既存コード `runtime/mod.rs L743`
- **Findings**:
  - 方式A: `lua.load(&format!("return require('{}')", module_name)).eval()`
  - 方式B: `lua.globals().get::<_, Function>("require")?.call(module_name)`
  - 既存コードは方式Aを使用
- **Implications**: 方式Bの方がエスケープ不要で直接的。設計では方式Bを採用。

### scene_dic.lua の生成場所

- **Context**: `require("pasta.scene_dic")`で解決可能にするための配置場所
- **Sources Consulted**: 
  - Lua package.path 仕様
  - `crates/pasta_lua/src/loader/cache.rs`
- **Findings**:
  - 現在: `profile/pasta/cache/lua/scene_dic.lua`
  - 必要: `profile/pasta/cache/lua/pasta/scene_dic.lua`
  - Luaは`pasta.scene_dic` → `pasta/scene_dic.lua`または`pasta/scene_dic/init.lua`を検索
- **Implications**: CacheManager::generate_scene_dicの出力パスを変更

### entry.luaの読み込み方式

- **Context**: 現在はハードコードされたパスから直接読み込み
- **Sources Consulted**: `runtime/mod.rs L549-564`
- **Findings**:
  - 現在: `scripts/pasta/shiori/entry.lua`固定
  - 変更後: `require("pasta.shiori.entry")`でpackage.pathから解決
  - ユーザーが`user_scripts/pasta/shiori/entry.lua`で上書き可能になる
- **Implications**: 設計原則に従い例外なしでrequire化

## Design Decisions

### Decision: requireヘルパー関数の実装方式

- **Context**: Rust側からLuaのrequireを呼び出す必要がある
- **Alternatives Considered**:
  1. `lua.load(&format!("return require('{}')")` - 文字列テンプレート方式
  2. `lua.globals().get::<_, Function>("require")?.call()` - 直接呼び出し方式
- **Selected Approach**: 方式B（直接呼び出し方式）
- **Rationale**: 
  - モジュール名のエスケープが不要
  - エラーハンドリングが明確
  - コードが簡潔
- **Trade-offs**: 
  - 既存コードとの一貫性は低下するが、新規実装なので問題なし
- **Follow-up**: 既存の`require("pasta.save")`呼び出しも将来的に統一検討

### Decision: entry.luaのrequire化

- **Context**: entry.luaを検索パス優先順位で上書き可能にするかどうか
- **Alternatives Considered**:
  1. 固定パスのまま維持（上書き不可）
  2. require化して上書き可能に
- **Selected Approach**: 方式2（require化）
- **Rationale**: 
  - 設計原則「例外は設けない」に従う
  - ユーザーがSHIORI処理をカスタマイズしたいケースに対応
- **Trade-offs**: 
  - ユーザーが誤って上書きするリスク → ユーザー責任として許容
- **Follow-up**: ドキュメントで上書き時の注意点を明記

### Decision: scene_dic.luaのrequire化

- **Context**: scene_dic.luaは動的生成されるが、require化すべきか
- **Alternatives Considered**:
  1. 引き続き直接ファイル読み込み
  2. require化（生成場所を`pasta/scene_dic.lua`に変更）
- **Selected Approach**: 方式2（require化）
- **Rationale**: 
  - 設計原則「例外は設けない」に従う
  - 一貫性のある設計
  - ユーザーがscene_dicを上書きすることは技術的には可能だが実用的ではない
- **Trade-offs**: 
  - CacheManagerの変更が必要
  - ユーザー上書き時は完全にユーザー責任
- **Follow-up**: 生成ディレクトリの作成処理を追加

### Decision: 初期化順序（main → entry → scene_dic）

- **Context**: main.luaで辞書登録を可能にするための順序決定
- **Alternatives Considered**:
  1. 現状維持（entry → scene_dic → main）
  2. main → scene_dic → entry
  3. main → entry → scene_dic
- **Selected Approach**: 方式3（main → entry → scene_dic）
- **Rationale**: 
  - main.luaで辞書登録APIを使えるようにする
  - entry.luaはSHIORI関数定義のみで、scene_dicに依存しない
  - scene_dic.luaはfinalize_scene()を呼び出すため最後に実行
- **Trade-offs**: 
  - entry.lua内でscene検索を使う場合は動作しない → 現状そのような使用はない
- **Follow-up**: 初期化順序をドキュメント化

### Decision: main.luaエラー時の挙動（警告のみで継続）

- **Context**: main.luaにシンタックスエラーや実行時エラーがある場合の対応
- **Alternatives Considered**:
  1. エラーで初期化失敗（厳格） - 不完全な状態で動作させない
  2. 警告のみで継続（寛容） - entry.luaと同様の扱い
- **Selected Approach**: 方式2（警告のみで継続）
- **Rationale**: 
  - main.luaはオプショナルな初期化処理
  - ユーザー辞書登録がなくても基本機能（DSLトランスパイル済みシーン）は動作すべき
  - entry.luaと一貫した挙動
- **Trade-offs**: 
  - ユーザーの意図した初期化が行われないリスク → 警告ログで通知
- **Follow-up**: Error Handlingセクションに明記

## Risks & Mitigations

- **Risk 1**: ユーザーがpasta/init.luaを上書きしてシステムが動作しなくなる
  - **Mitigation**: ユーザー責任として許容。ドキュメントで注意喚起。

- **Risk 2**: scene_dic.luaの生成場所変更によるパス解決の問題
  - **Mitigation**: 統合テストで検証

- **Risk 3**: Windows環境でのパスエンコーディング問題
  - **Mitigation**: 既存の`generate_package_path_bytes()`がANSI変換を処理済み

## References

- [Lua 5.4 Reference Manual - require](https://www.lua.org/manual/5.4/manual.html#pdf-require)
- [mlua 0.11 Documentation](https://docs.rs/mlua/0.11/)
- [gap-analysis.md](gap-analysis.md) - 実装ギャップ分析
- [requirements.md](requirements.md) - 要件定義
