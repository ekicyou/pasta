SHIORI
=======================================

SHIORIはゴーストの人格を表現することを目的とした、ベースウェアとSHIORI共有ライブラリの通信・動作仕様である。

SHIORIクライアントであるベースウェアとSHIORIサーバーであるSHIORI共有ライブラリによるクライアント・サーバー方式である。

SHIORI共有ライブラリがさまざまな動作をするので、ゴースト作者には一見SHIORI共有ライブラリがわがままなクライアントであるように見えるかもしれないが、
SHIORI共有ライブラリがいくら望んでもベースウェアが許可しない限り、基本的な1秒毎の応答さえ実現できない。

SHIORIはおおまかに3層の仕様に分かれていると見ることができる。

1. SHIORI共有ライブラリ仕様
2. SHIORIプロトコル仕様
3. SHIORIイベント仕様

この3層が一体となってバージョンが付加されている。いわゆるSHIORI/2.xやSHIORI/3.xである。

SHIORI/1.xは削除され、ほぼ駆逐された。新規のベースウェアやゴースト作者はSHIORI/1.xについては考えなくてよいといえる。

SHIORI共有ライブラリ仕様
---------------------------------------

SHIORI共有ライブラリ仕様はベースウェアとSHIORI共有ライブラリの通信経路を確保するための仕様である。

SHIORI「共有ライブラリ」そのものの仕様を定めるものである。

大雑把にOSI参照モデルのトランスポート層のようなものと考えてよい。

ゴースト作者はこの仕様について全てを理解する必要はない。

SHIORI共有ライブラリの役割は、ゴースト(人格)固有の動作を表現するためにGUIに任意の指示を出すことである。

SHIORI共有ライブラリはGUIと高速に通信する独立したネイティブプログラムである。

SHIORI/2.xとSHIORI/3.xでこの部分は共通である。

### WindowsにおけるSHIORI共有ライブラリ

WindowsにおけるSHIORI共有ライブラリはDLLとして実装される。

これはmateriaやSSP等のWindows上で動作するベースウェアが利用する。

SHIORI DLLは以下の関数load(), unload(), request()をエクスポートする。

    extern "C" __declspec(dllexport) BOOL __cdecl load(HGLOBAL h, long len);
    extern "C" __declspec(dllexport) BOOL __cdecl unload();
    extern "C" __declspec(dllexport) HGLOBAL __cdecl request(HGLOBAL h, long *len);

- シングルサーバ／シングルクライアントであり、SHIORI DLLは複数のクライアントの接続を受け入れることは想定しなくてよい。
- ベースウェアはリクエスト後レスポンスが返るまでブロッキングする。スレッドセーフであることは考慮しなくてよい。

DLLロード時にload(), アンロード時にunload()、その他全ての実際的な通信にrequest()が呼ばれる。

#### BOOL load(HGLOBAL h, long len)

DLLロード時に呼ばれ、DLLが存在するディレクトリパスが渡される。
これは常にパスセパレーター(`\`)で終わることが保障される。

SHIORI DLLの初期化処理等を行うことが期待される。

hは`GlobalAlloc(GMEM_FIXED, len)`で確保された`len`の長さを持つ`const char*`へのポインタである。

したがって`strncpy(str, (const char*)h, len)`等でコピーできる。

この文字列は`\0`を末尾に持たないことに注意すべきである。

hはSHIORI DLL側で`GlobalFree((HGLOBAL)h)`で開放しなければならない。

成功した場合true、失敗した場合falseを返さなくてはならない。

#### BOOL unload()

DLLアンロード時に呼ばれる。

SHIORI DLLの終了処理等を行うことが期待される。

成功した場合true、失敗した場合falseを返さなくてはならない。

#### HGLOBAL request(HGLOBAL h, long *len)

SHIORIプロトコルのリクエストを渡されて呼ばれ、同レスポンスを返すための関数である。

hは`GlobalAlloc(GMEM_FIXED, *len)`で確保された`*len`の長さを持つ`const char*`へのポインタである。

したがって渡されたリクエスト文字列は`strncpy(str, (const char*)h, *len)`等でコピーできる。

この文字列は`\0`を末尾に持たないことに注意すべきである。

hはSHIORI DLL側で`GlobalFree((HGLOBAL)h)`で開放しなければならない。

`*len`はリクエスト文字列を取得した後開放せずに、新たにレスポンス文字列の長さを代入する。

レスポンス文字列はSHIORI DLL側で`GlobalAlloc(GMEM_FIXED, *len)`で確保して`memcpy(rh, str, *len)`等で文字列を格納し、そのポインタを返す

#### ベースウェア

ベースウェアはこれらの共有ライブラリを`LoadLibrary("shiori.dll")`等で呼び出し、`GetProcAddress(dllh, "load")`等で関数をインポートする。

実行中は適切に引数確保や返り値の処理をし、切り離し時は`FreeLibrary(dllh)`で開放する。

これは通常のDLLの扱いと特に変わりはない。

### UNIX like OSにおけるSHIORI共有ライブラリ

UNIX like OSにおけるSHIORI共有ライブラリはDLLとして実装できないので、代替手段として共有ライブラリ(*.so)として実装する。

これはninix-aya等のUNIX like OS上で動作するベースウェアが利用する。

これは公式に仕様化はされておらず、オープンソースのSHIORI共有ライブラリが準拠するデファクトスタンダードであるが、同様の方式で普及する利益をかんがみてここで紹介する。

UNIX like OSにおけるSHIORI共有ライブラリはWindowsにおけるSHIORI DLLのメモリ利用のための関数を単に以下のように置き換えたものである。

- HGLOBAL -> char*
- GlobalAlloc(GMEM_FIXED, len) -> malloc(len)
- GlobalFree((HGLOBAL)h) -> free((void *)h)

同時にエクスポートする関数は以下のように置き換えられる。

    extern "C" bool load(char* h, long len);
    extern "C" bool unload();
    extern "C" char* request(char* h, long *len);

各関数の仕様に変化はないが、load()のみ、パスセパレータが`\`から`/`に変更されるであろう。

### ブラウザ環境におけるSHIORI共有ライブラリ

ブラウザ環境におけるSHIORI共有ライブラリはUNIX like OS上のそれをemscriptenにより移植したものとして存在する。

これは如何か等のブラウザ環境上で動作するベースウェアが利用する。

これは公式に仕様化はされておらず、如何かの実装による仕様であるが、同様の方式で普及する利益をかんがみてここで紹介する。

SHIORI共有ライブラリとしての全ての仕様は、UNIX like OS上のそれと変更が無い。

SHIORIプロトコル仕様
---------------------------------------

SHIORIプロトコル仕様はベースウェアとSHIORI共有ライブラリの文字列通信の書式の仕様である。

OSI参照モデルのアプリケーション層にあたる。

ゴースト作者はこの仕様について全てを理解する必要はない。

SHIORI/2.xとSHIORI/3.xでこの部分は見方によっては差異があるが、パーサを共通にしても処理できる程度の差しかない。

### SHIORIリクエスト

    GET Version SHIORI/2.0
    Sender: Nobody
    Charset: Shift_JIS
    

SHIORIプロトコルはHTTPに似ている。

1行目がコマンド行であり、2行目以降がヘッダ行である。それぞれの行はCR+LFでターミネートされ、リクエスト全体はCR+LFが2つ続いた時点でターミネートされる。

コマンド行は`コマンド SHIORI/バージョン[CRLF]`である。SHIORI/2.xではコマンド文字列中に空白が存在するので空白をセパレータとしてはならない。

ヘッダ行は`ヘッダ名: ヘッダ内容[CRLF]`であり、`:`の後の空白は省略不可能である。最大数は無限大である。

    GET SHIORI/3.0[CRLF]
    Sender: Ikagaka[CRLF]
    Charset: Shift_JIS[CRLF]
    ID: OnFirstBoot[CRLF]
    Reference0: 0[CRLF]
    [CRLF]

HTTPに対しての書式の相違点としては、BODYにあたる部分が存在しないこと、SHIORI/2.xではコマンド文字列中に空白が存在することなどである。

### SHIORIレスポンス

    SHIORI/2.5 200 OK
    String: http://sakura.mikage.to/
    

リクエストと同様にレスポンスもHTTPと似ている。

1行目がステータス行であり、2行目以降がヘッダ行である。それぞれの行はCR+LFでターミネートされ、レスポンス全体はCR+LFが2つ続いた時点でターミネートされる。

コマンド行は`SHIORI/バージョン ステータスコード ステータス文字列[CRLF]`である。

ヘッダ行は`ヘッダ名: ヘッダ内容[CRLF]`であり、`:`の後の空白は省略不可能である。

    SHIORI/3.0 200 OK[CRLF]
    Charset: Shift_JIS[CRLF]
    Sender: サンプル[CRLF]
    SecurityLevel: local[CRLF]
    Value: \0\s[0]おはこんばんちは。[CRLF]
    [CRLF]

ステータスコードとステータス文字列は以下のとおり。

- 2xx - 処理完了
    - 200 OK	正常に終了した
    - 204 No Content	正常に終了したが、返すべきデータがない
- 3xx - 処理完了、追加アクション要求
    - <del>310 Communicate</del>	- deprecated -
    - 311 Not Enough	TEACH リクエストを受けたが、情報が足りない
    - 312 Advice	TEACH リクエスト内の最も新しいヘッダが解釈不能
- 4xx - リクエストエラー
    - 400 Bad Request	リクエスト不備
- 5xx - サーバエラー
    - 500 Internal Server Error	サーバ内でエラーが発生した

### SHIORI/2.xとSHIORI/3.xの相違点

基礎的な書式は共通であるが、SHIORI/2.xとSHIORI/3.xはいずれもその書式内で実現できる記述のサブセットのみをとる。

第一の相違点としては、ヘッダの命名がある。

SHIORI/2.xでは意味に即したヘッダ名がつけられる。

GET SentenceへのレスポンスはSentenceヘッダであり、GET StringへのレスポンスはStringヘッダである。

SHIORI/3.xでは文字コードやSender等の基本的なもの以外のヘッダ名は引数の位置を表すものとして統一されている。

全てのレスポンスはValueヘッダであらわされ、Toヘッダなどは消滅した。
このためレスポンスにおける「コミュニケート」の相手先はSHIORI/2.xではTo、SHIORI/3.xではReference0ヘッダで指定する。

SHIORIイベント仕様
---------------------------------------

SHIORIイベント仕様はベースウェアのアプリケーションの動作を規定する仕様である。

このイベント仕様を参照して、SHIORI共有ライブラリは適切な応答を返す必要がある。

ゴースト作者が主に参照すべきはこの仕様部分である。

ここではmateria583の従う固定された仕様をSHIORI/3.0とし、現在SSP等が拡張を続けているLiving Standardは便宜上SHIORI/3.1として区別する。

SHIORI/3.0が定めるイベントは多くにおいてSHIORI/2.xのスーパーセットであり、一部コミュニケート等に非互換の仕様が存在することを除けば容易に変換が可能である。

SHIORI/3.1はSHIORI/3.0の完全なスーパーセットである。

SHIORIプロトコル・イベント仕様
---------------------------------------

これら3つの部分仕様のうち、プロトコル・イベント仕様はSHIORI仕様のバージョンによって結合的に定義が変化している。

現在主に利用されているのはSHIORI/3.1だが、依然としてSHIORI/2.xを利用するゴーストも存在する。

SHIORI/3.1で新たに定められたイベントがSHIORI/2.xのイベントと類似した書式でSHIORI共有ライブラリに通知されることは起こりうるが、それは本質的にはSHIORI/2.xではなくSHIORI/3.1のイベントである。

この点は後日記述する。

参考までにSHIORI/3.1の各イベントは[UKADOC SHIORI Eventリスト](http://ssp.shillest.net/ukadoc/manual/list_shiori_event.html)に詳しい。

注記
---------------------------------------

この文書は未完成である。特にSHIORIプロトコル・イベント仕様について後日の追記が必須である。
