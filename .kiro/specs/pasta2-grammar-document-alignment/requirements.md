# Requirements Document

## Project Description (Input)
パスタスクリプトの新たな文法定義として、「pasta2.pest」を定義した。本定義ファイルを憲法（変更不可）として、まずはドキュメント「SPECIFICATION.md」「GRAMMAR.md」「comprehensive_control_flow2.pasta」を、文法に一致した記述に更新する。

## Introduction
本仕様は、新たに定義された`pasta2.pest`文法を権威的基準（憲法）として、既存のドキュメントとサンプルスクリプトを完全に整合させることを目的とする。`pasta2.pest`はPestパーサー文法として厳密に定義されており、変更不可である。既存の3ファイル（SPECIFICATION.md、GRAMMAR.md、comprehensive_control_flow2.pasta）は、この文法定義と矛盾や乖離があってはならない。

---

## Requirements

### Requirement 1: pasta2.pest文法の権威性確立
**Objective:** パスタプロジェクトの開発者として、pasta2.pest文法定義を唯一の正式な文法仕様として扱えるようにし、ドキュメントとの不整合を排除したい。

#### Acceptance Criteria
1. When ドキュメントやサンプルスクリプトを参照する場合、the Pasta Documentation System shall pasta2.pestと矛盾する記述を含まない
2. If pasta2.pestと既存ドキュメントの間に矛盾が発見された場合、then the Documentation Update Process shall pasta2.pestの定義を優先し、ドキュメント側を修正する
3. The SPECIFICATION.md shall pasta2.pestで定義されたすべての構文要素（マーカー、識別子、式、文、スコープ）を正確に反映する
4. The GRAMMAR.md shall pasta2.pestで定義された文法規則に基づいて、学習者向けの説明を提供する
5. The comprehensive_control_flow2.pasta shall pasta2.pestで認識可能な有効な構文のみを使用する

---

### Requirement 2: SPECIFICATION.mdの文法整合性
**Objective:** 仕様書の執筆者として、SPECIFICATION.mdがpasta2.pestの文法定義と完全に一致するよう更新し、実装者が正確な仕様を参照できるようにしたい。

#### Acceptance Criteria
1. When SPECIFICATION.mdのマーカー定義セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのマーカー定義（hash, at, amp, ast, local_marker, dollar, gt等）と完全に一致した記述を含む
2. When SPECIFICATION.mdの識別子定義セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのid定義（reserved_id, identifier, XID_START, XID_CONTINUE）と一致した説明を含む
3. When SPECIFICATION.mdの式（expr）定義を読む場合、the SPECIFICATION.md shall pasta2.pestのexpr, term, bin, bin_op規則と一致した構文説明を含む
4. When SPECIFICATION.mdの変数参照・代入セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのvar_ref, var_set規則（ローカル/グローバル区別）と一致する
5. When SPECIFICATION.mdの関数呼び出しセクションを読む場合、the SPECIFICATION.md shall pasta2.pestのfn_call, args, key_arg, positional_arg規則と一致する
6. When SPECIFICATION.mdの文字列リテラルセクションを読む場合、the SPECIFICATION.md shall pasta2.pestのstring_literal, strfence（「」および"）規則と一致する
7. When SPECIFICATION.mdの単語辞書セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのwords, word_nofenced, comma_sep規則と一致する
8. When SPECIFICATION.mdの属性セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのattr, key_attr, attr_value規則と一致する
9. When SPECIFICATION.mdのシーン定義セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのglobal_scene_line, local_scene_line, scene規則と一致する
10. When SPECIFICATION.mdのアクション行セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのaction_line, continue_action_line, actions, action規則と一致する
11. When SPECIFICATION.mdのCall文セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのcall_scene, call_scene_local, call_scene_global規則と一致する
12. When SPECIFICATION.mdのさくらスクリプトセクションを読む場合、the SPECIFICATION.md shall pasta2.pestのsakura_script, sakura_marker（半角\\のみ）規則と一致する
13. When SPECIFICATION.mdのRuneコードブロックセクションを読む場合、the SPECIFICATION.md shall pasta2.pestのcode_block, code_open, code_contents, code_close規則と一致する
14. When SPECIFICATION.mdのスコープ構造セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのfile_scope, global_scene_scope, local_scene_scope規則と一致する
15. The SPECIFICATION.md shall pasta2.pestで廃止された構文（全角\\、全角[]等）を使用例として記載しない

---

### Requirement 3: GRAMMAR.mdの学習者向け文法整合性
**Objective:** 学習者として、GRAMMAR.mdを読んでpasta2.pestで定義された正確な文法を理解し、誤った構文を学習しないようにしたい。

#### Acceptance Criteria
1. When GRAMMAR.mdのマーカー一覧を参照する場合、the GRAMMAR.md shall pasta2.pestで定義されたマーカーの全角/半角両対応を正確に示す
2. When GRAMMAR.mdのシーン定義セクションを読む場合、the GRAMMAR.md shall pasta2.pestのglobal_scene_line, local_scene_line構文に従った例を提供する
3. When GRAMMAR.mdのアクション行の説明を読む場合、the GRAMMAR.md shall pasta2.pestのaction_line, continue_action_line構文に従った例を提供する
4. When GRAMMAR.mdの変数セクションを読む場合、the GRAMMAR.md shall pasta2.pestのvar_set（ローカル/グローバル）構文に従った例を提供する
5. When GRAMMAR.mdの単語定義と参照セクションを読む場合、the GRAMMAR.md shall pasta2.pestのword_ref, key_words構文に従った例を提供する
6. When GRAMMAR.mdの関数呼び出しセクションを読む場合、the GRAMMAR.md shall pasta2.pestのfn_call, args構文に従った例を提供する
7. When GRAMMAR.mdのさくらスクリプトセクションを読む場合、the GRAMMAR.md shall sakura_marker（半角\\のみ）を正確に説明し、全角\\は使用不可と明記する
8. When GRAMMAR.mdの文字列リテラルセクションを読む場合、the GRAMMAR.md shall pasta2.pestのstring_fenced（「」または"）構文を正確に説明する
9. When GRAMMAR.mdのRuneコードブロックセクションを読む場合、the GRAMMAR.md shall pasta2.pestのcode_block構文（```で囲む）を正確に説明する
10. The GRAMMAR.md shall pasta2.pestで廃止された構文を「使用可能」として記載しない

---

### Requirement 4: comprehensive_control_flow2.pastaのパース可能性
**Objective:** パーサー開発者として、comprehensive_control_flow2.pastaがpasta2.pestで完全にパース可能であることを保証し、テストフィクスチャとして使用したい。

#### Acceptance Criteria
1. When comprehensive_control_flow2.pastaをpasta2.pestパーサーで解析する場合、the Pasta Parser shall 構文エラーを出力しない
2. When comprehensive_control_flow2.pastaにグローバルシーン定義が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのglobal_scene_line構文に準拠する
3. When comprehensive_control_flow2.pastaにローカルシーン定義が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのlocal_scene_line構文に準拠する
4. When comprehensive_control_flow2.pastaにアクション行が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのaction_line, continue_action_line構文に準拠する
5. When comprehensive_control_flow2.pastaに変数代入が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのvar_set_line構文（$id=exprではなく$id=exprまたは$id：expr）に準拠する
6. When comprehensive_control_flow2.pastaにCall文が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのcall_scene_line構文（>labelまたは>*label）に準拠する
7. When comprehensive_control_flow2.pastaに単語定義が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのkey_words構文（カンマ区切り）に準拠する
8. When comprehensive_control_flow2.pastaに属性定義が含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのattr構文（&key:value）に準拠する
9. When comprehensive_control_flow2.pastaにさくらスクリプトが含まれる場合、the comprehensive_control_flow2.pasta shall sakura_marker（半角\\のみ）を使用する
10. When comprehensive_control_flow2.pastaにRuneコードブロックが含まれる場合、the comprehensive_control_flow2.pasta shall pasta2.pestのcode_block構文（```rune...```）に準拠する
11. The comprehensive_control_flow2.pasta shall pasta2.pestで廃止された構文（全角\\、全角[]、Jump文等）を使用しない

---

### Requirement 5: ドキュメント間の一貫性維持
**Objective:** プロジェクトメンテナーとして、SPECIFICATION.md、GRAMMAR.md、comprehensive_control_flow2.pastaの3ファイル間で、用語・構文・例示の一貫性を維持し、学習者の混乱を防ぎたい。

#### Acceptance Criteria
1. When マーカーの全角/半角対応を説明する場合、the Documentation System shall すべてのドキュメントで同一の対応表を使用する
2. When 変数スコープ（ローカル/グローバル）を説明する場合、the Documentation System shall すべてのドキュメントで同一の記法（$var vs $*var）を使用する
3. When シーン定義の構文を示す場合、the Documentation System shall すべてのドキュメントで同一の形式（*scene vs ・scene）を使用する
4. When さくらスクリプトのマーカーを示す場合、the Documentation System shall すべてのドキュメントで半角\\のみを使用し、全角\\は記載しない
5. When 文字列リテラルの囲み文字を示す場合、the Documentation System shall すべてのドキュメントで「」および"を正確に記載する

---

### Requirement 6: 廃止構文の完全削除
**Objective:** 開発者として、pasta2.pestで廃止された古い構文（Jump文、全角さくらマーカー等）がドキュメントやサンプルに残存しないようにし、誤った実装を防ぎたい。

#### Acceptance Criteria
1. The SPECIFICATION.md shall Jump文（？マーカー）に関する記述を削除または「廃止」と明記する
2. The SPECIFICATION.md shall 全角\\（さくらスクリプトマーカー）を使用例として記載しない
3. The SPECIFICATION.md shall 全角[]（さくらスクリプト引数括弧）を使用例として記載しない
4. The GRAMMAR.md shall Jump文（？マーカー）を使用可能な構文として記載しない
5. The GRAMMAR.md shall 全角\\を有効なさくらスクリプトマーカーとして記載しない
6. The GRAMMAR.md shall 全角[]を有効なさくらスクリプト引数括弧として記載しない
7. The comprehensive_control_flow2.pasta shall Jump文（？マーカー）を使用しない
8. The comprehensive_control_flow2.pasta shall 全角\\（さくらスクリプトマーカー）を使用しない
9. The comprehensive_control_flow2.pasta shall 全角[]（さくらスクリプト引数括弧）を使用しない

---

### Requirement 7: 新規構文の完全反映
**Objective:** 開発者として、pasta2.pestで新たに定義された構文要素（式のサポート、グローバル変数・関数呼び出し等）がすべてのドキュメントに正確に記載されるようにしたい。

#### Acceptance Criteria
1. When SPECIFICATION.mdの式（Expression）セクションを読む場合、the SPECIFICATION.md shall pasta2.pestのexpr, term, bin, bin_op規則を詳細に説明する ✅ **[RESOLVED] 決定: pasta2.pestは式を正式採用。SPECIFICATION.md 1.3節「式の制約」を「式（Expression）のサポート」に改名し、expr, term, bin, bin_op規則を詳細説明に変更。**
2. When SPECIFICATION.mdのグローバル変数セクションを読む場合、the SPECIFICATION.md shall $*id構文（var_ref_global, var_set_global）を説明する ⏳ **[議題2: グローバル参照仕様確認中]**
3. When SPECIFICATION.mdのグローバル関数呼び出しセクションを読む場合、the SPECIFICATION.md shall @*id(args)構文（fn_call_global）を説明する ⏳ **[議題2: グローバル参照仕様確認中]**
4. When SPECIFICATION.mdのグローバル単語参照セクションを読む場合、the SPECIFICATION.md shall @*id構文（word_ref_global）を説明する ⏳ **[議題2: グローバル参照仕様確認中]**
5. When SPECIFICATION.mdのグローバルCall文セクションを読む場合、the SPECIFICATION.md shall >*id構文（call_scene_global）を説明する ⏳ **[議題2: グローバル参照仕様確認中]**
6. When GRAMMAR.mdの式セクションを読む場合、the GRAMMAR.md shall pasta2.pestで定義された式の構文例を提供する ✅ **[RESOLVED] 決定と同時に対応可能**
7. When GRAMMAR.mdのグローバル参照セクションを読む場合、the GRAMMAR.md shall $*id, @*id, >*id構文の使用例を提供する ⏳ **[議題2: グローバル参照仕様確認中]**
8. When comprehensive_control_flow2.pastaにグローバル参照が必要な場合、the comprehensive_control_flow2.pasta shall $*id, @*id, >*id構文を正確に使用する ⏳ **[議題2・3: 後続議題で確認]**

---

## Summary
本仕様は、pasta2.pest文法定義を唯一の権威として、既存ドキュメント（SPECIFICATION.md、GRAMMAR.md）とサンプルスクリプト（comprehensive_control_flow2.pasta）を完全に整合させる。すべての構文要素（マーカー、識別子、式、変数、関数、シーン、アクション、さくらスクリプト、Runeブロック）がpasta2.pestと矛盾なく記載され、廃止構文が削除され、新規構文が完全に反映される。これにより、開発者と学習者は正確な文法仕様を参照でき、実装とドキュメントの乖離が排除される。

