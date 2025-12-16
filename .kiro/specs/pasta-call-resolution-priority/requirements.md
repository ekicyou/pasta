# Requirements Document

## Project Description (Input)

pasta エンジンにおいて、単語辞書の登録と呼び出しに対応して欲しい。pasta の DSL レベルではすでに対応されている可能性がある。トランスパイラー側および生成後の rune-rust エンジンにて対応が必要。

### グローバル単語の登録
トランスパイラーの1pass目にて、インデント無しで「＠」から始まる行について、グローバル単語登録とする。

```pasta
＠単語：単語１　単語２　単語３
＠単語：単語４　単語５
```

この時、以下のように登録する。ここで、HashMap<String, Vec[String]>

- key: `単語:`
- values: [`単語１`, `単語２`, `単語３`, `単語４`, `単語５`]

### ローカル単語の登録
トランスパイラーの1pass目にて、インデント有りで「＠」から始まる行について、ローカル単語登録とする。

```pasta
＊グローバルラベル

　＠単語：単語６　単語７
```

この時、以下のように登録する。

- key: `グローバルラベル_1:単語:`
- values: [`単語６`, `単語７`]

ここで、「グローバルラベル」は`mod グローバルラベル_1`にトランスパイラー解決されているものとする。

### 単語の変換
トランスパイラーの1pass目にて、会話行のインライン要素として「＠単語（空白が続く場合、区切り文字として任意文字数の空白を許容）」が現れたとき、単語検索関数word()に置換して欲しい。

```pasta
　さくら：今日は＠単語　を覚えるよ。
```

```rune
  // さくら：今日は
  ...
  // ＠単語
  for a in word("グローバルラベル_1", "単語") { yield a; }
  // を覚えるよ。
  ...
```


### fn pasta::word(global, local) の処理
pub fn pasta::word(global, local)を定義する。この関数はrust側で公開する単語検索エンジンを所有するクロージャを呼び出して結果を一つyieldする。

### 単語検索ルール
検索条件`"グローバルラベル_1", "単語"`が与えられた時の単語検索ルール。
1. 検索キー`グローバルラベル_1:単語:`を組み立てる
2. 検索キーでキャッシュを検索する。
  - キャッシュが無ければ要素を抽出し（後述）、キャッシュに登録し、vecを返す。
  - キャッシュがあればキャッシュvecを返す。
3. vecから要素を一つ取り出して返す。vec要素が０になったらキャッシュから削除。

### 単語抽出ルール
検索条件`"グローバルラベル_1", "単語"`が与えられた時の単語抽出ルール。以下のルールで検索・リストアップしたすべての単語が単語選択の対象となる。

1. `単語:`で検索した候補
2. `グローバルラベル_1:単語:`で検索した候補

## Introduction

本仕様は、pasta スクリプト内で定義される「単語辞書」の登録と、会話行からの単語呼び出しを、トランスパイル後の rune 実行時に解決できるようにする。

用語:

- **グローバルラベル**: `＊` で始まるラベル行により導入されるスコープ名。トランスパイラーにより `mod グローバルラベル_1` のようなモジュール名へ解決される。
- **グローバル単語登録行**: 先頭に空白が無い `＠` 始まりの行。
- **ローカル単語登録行**: 先頭に空白がある `＠` 始まりの行。
- **単語名**: `＠` の直後から `：`（全角コロン）までの文字列。

## Requirements

### Requirement 1: 単語辞書の登録（グローバル/ローカル）
**Objective:** As a pasta スクリプト作者, I want 単語辞書を宣言できる, so that 会話の表現を簡潔に揺らせる。

#### Acceptance Criteria
1. When 入力にグローバル単語登録行が含まれるとき, the Pasta Transpiler shall 単語名を抽出し、キーを`<単語名>:`としてグローバル単語辞書に登録する。
2. When 入力にグローバル単語登録行が含まれるとき, the Pasta Transpiler shall `：`以降に並ぶトークン列を値リストとして抽出し、同一キーの既存値リストが存在する場合は末尾に追加する。
3. When 入力にローカル単語登録行が含まれるとき, the Pasta Transpiler shall 現在のグローバルラベル（解決後モジュール名）を用いて、キーを`<グローバルラベル_解決後>:<単語名>:`としてローカル単語辞書に登録する。
4. If ローカル単語登録行が現在のグローバルラベル文脈なしで現れたとき, then the Pasta Transpiler shall 単語登録を行わず、診断可能なエラーを報告する。
5. If 単語登録行に`：`が存在しない、または単語名が空のとき, then the Pasta Transpiler shall 単語登録を行わず、診断可能なエラーを報告する。

### Requirement 2: 会話行の単語呼び出しの変換
**Objective:** As a pasta スクリプト作者, I want 会話行内で単語を呼び出せる, so that 辞書により会話表現を差し替えできる。

#### Acceptance Criteria
1. When 会話行内にインライン単語参照`＠<単語名>`が現れたとき, the Pasta Transpiler shall 対応する位置に単語検索関数呼び出し（引数: `<グローバルラベル_解決後>`, `<単語名>`）へ変換する。
2. When インライン単語参照の直後に任意文字数の空白が続くとき, the Pasta Transpiler shall その空白を区切りとして扱い、単語名に含めない。
3. If 会話行内でインライン単語参照が現れ、かつグローバルラベル文脈が確定できないとき, then the Pasta Transpiler shall 変換を行わず、診断可能なエラーを報告する。

### Requirement 3: `pasta::word(global, local)` の提供
**Objective:** As a runtime, I want `pasta::word`で単語を取り出せる, so that トランスパイル後のコードが辞書を参照できる。

#### Acceptance Criteria
1. The Pasta Runtime shall `pasta::word(global, local)` を、生成された rune コードから呼び出し可能な形で提供する。
2. When `pasta::word(global, local)` が呼び出されたとき, the Pasta Runtime shall 単語辞書の検索結果から要素を1つずつ返却（yield）できるようにする。
3. If `pasta::word(global, local)` の検索結果が空のとき, then the Pasta Runtime shall 例外やパニックを発生させず、要素を返却しない。

### Requirement 4: 単語検索・抽出・キャッシュ
**Objective:** As a runtime, I want 検索結果をキャッシュしつつランダムに取り出せる, so that 同一参照の連続呼び出しで効率よく多様な結果を得られる。

#### Acceptance Criteria
1. When `pasta::word(global, local)` が呼び出されたとき, the Pasta Runtime shall 検索キー`<global>:<local>:`を組み立てる。
2. When 検索キーに対応するキャッシュが存在しないとき, the Pasta Runtime shall 候補リストを抽出し、抽出結果をキャッシュに登録する。
3. When 候補リストを抽出するとき, the Pasta Runtime shall グローバルキー`<local>:`に紐づく値リストとローカルキー`<global>:<local>:`に紐づく値リストの両方を探索し、存在する全要素を候補としてリストアップする。
4. When 候補リストをキャッシュ登録するとき, the Pasta Runtime shall 候補リストの順序をシャッフルする。
5. When 検索キーに対応するキャッシュが存在するとき, the Pasta Runtime shall 抽出処理を再実行せず、キャッシュされた候補リストを使用する。
6. When 候補リストから要素を返却するとき, the Pasta Runtime shall キャッシュされた候補リストから要素をちょうど1つ取り出し、その要素を返却する。
7. When 候補リストの要素数が0になったとき, the Pasta Runtime shall 当該検索キーのキャッシュを削除する。
8. The Pasta Runtime shall グローバル/ローカル両方から同一文字列が候補として得られた場合でも、重複を除去せず候補として扱う。

### Requirement 5: 入力の堅牢性
**Objective:** As a developer, I want 不正入力でも安定して診断できる, so that スクリプトの誤りを容易に修正できる。

#### Acceptance Criteria
1. If 単語登録行の値リストが空のとき, then the Pasta Transpiler shall 空リストとして登録するか、または診断可能なエラーを報告する（いずれの場合もクラッシュしない）。
2. If 単語名またはグローバルラベルに検索キーとして不正な文字が含まれているとき, then the Pasta Transpiler shall 変換/登録を行わず、診断可能なエラーを報告する。
