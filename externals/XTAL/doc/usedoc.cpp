
/** \page usepage C++組み込みサンプル
- \subpage useinitialize
- \subpage useexcept
- \subpage usehelloworld
- \subpage usestring
- \subpage useanyptr
- \subpage usefuncall
- \subpage usesend
- \subpage usenew
- \subpage userefmember
- \subpage useat
- \subpage useiter
- \subpage usereturn
- \subpage usecppclass
- \subpage usebytecode
*/

/** \page usemain
Luaなどのスクリプト言語は、C/C++とのやり取りが煩雑であるものが多い。\n
そのため言語実装者以外にスクリプト言語とC++との間を取り持つ、「バインダ」と呼ばれるものがよく作られる。\n
\n
Xtalでは最初からスクリプト言語と密接したバインダを内蔵しているため、\n
わざわざバインダを探すことも、またオリジナルのバインダを作る必要も無く、すぐに組み込むことができる。\n
*/

/** \page useinitialize Xtalの初期化
ここでは例としてWindowsでの初期化方法を示す。\n
例として、Windowsでの初期化方法を説明します。\n
\code
#include <xtal.h>
#include <xtal_macro.h> // Xidなど便利なマクロが定義されている

#include <xtal_lib/xtal_cstdiostream.h> // CStdioStdStreamLibのため
#include <xtal_lib/xtal_winthread.h> // WinThreadLibのため
#include <xtal_lib/xtal_winfilesystem.h> // WinFilesystemLibのため
#include <xtal_lib/xtal_chcode.h> // SJISChCodeLibのため
#include <xtal_lib/xtal_errormessage.h> // bind_error_message()のため

int main(int argc, char** argv){
    using namespace xtal;

    CStdioStdStreamLib cstd_std_stream_lib; // stdin, stdout, stderrはCの標準ライブラリを使う
    WinThreadLib win_thread_lib; // Windowsのスレッドを使う
    WinFilesystemLib win_filesystem_lib; // Windowsのファイルシステムを使う 
    SJISChCodeLib sjis_ch_code_lib; // SJISを使う

    // 環境依存である機能についてどれを使うかを設定
    Setting setting; 
    setting.thread_lib = &win_thread_lib;
    setting.std_stream_lib = &cstd_std_stream_lib;
    setting.filesystem_lib = &win_filesystem_lib;
    setting.ch_code_lib = &sjis_ch_code_lib;
    
    // ここで指定したthread_libなどのポインタが示す先のオブジェクトは、
    // uninitializeを呼び出すまで存在している必要があります。

    // Xtalを初期化
    initialize(setting);

    // エラーメッセージをバインド
    bind_error_message();

	{
	    // ここでスクリプトの実行を行う
	}
	
    // Xtalを破棄
    uninitialize();

    return 0;
}
\endcode

Xtalでは環境依存の機能は簡単に取り替えることができるようになっている。\n
WindowsとLinuxについては、最初から機能が用意されているため、それを使うことができる。\n
各実装はxtal_libディレクトリ内にクラスで提供されている。\n
使いたい機能をincludeし、initializeからuninitializeまで生存する変数として定義し、\n
Settingの各メンバにそのポインタを代入し、initailize関数に渡すこと。\n
\n
bind_error_messageでエラーメッセージのバインドができる。\n
これにより、XRE1001といったエラーメッセージが、意味のあるテキストとして得られるようになる。\n
リリース時など、エラーメッセージが必要ないのであればこれを呼び出さないことで若干のメモリ節約になる。\n
\n
uninitialize時に参照カウントを掴んだままのオブジェクトがあるとassertで停止するため、\n
スクリプトの実行の部分はブロックで囲むか、別関数で行い、ローカル変数で定義したAnyPtrなどが確実にデストラクタされているようにしよう。\n
*/



/** \page usehelloworld ファイルに書かれているXtalを実行
次のようなHello, Worldを出力するスクリプト、HelloWorld.xtalがあるとする\n
\code
// HelloWorld.xtal
println("Hello, World");
\endcode
これをC++から呼び出すにはload関数を使う。\n
\code
// C++
load(XTAL_STRING("HelloWorld.xtal"));

// xtal_macro.hをincludeしている場合、次のように記述してもよい
// load(Xs("HelloWorld.xtal"))

// コンパイルエラーや実行時エラーがあったら捕捉する。
XTAL_CATCH_EXCEPT(e){
    stderr_stream()->println(e); // stderrに例外オブジェクトを出力する
}
\endcode
*/

/** \page useexcept Xtalで発生した例外を捕捉する
XTAL_CATCH_EXCEPT(e)マクロを使うことで、Xtal処理系で発生した例外を補足することができる。\n
\code
any->call();

XTAL_CATCH_EXCEPT(e){
    // call内で例外がセットされたらここを通る
    // eに例外オブジェクトがセットされる。
	stderr_stream()->println(e); // stderrに例外オブジェクトを出力する
}
else{
    // 例外が発生していないならここを通る
}
\endcode
*/

/** \page usestring 文字列のXtalソースを実行
ファイルとしてではなく、文字列としてソースを保持していてそれを実行したい場合は、compile関数を使い、結果を呼び出す。 \n
\code
const char* source = " println(\"Hello, World\"); ";

// コンパイルエラーがあれば、codeはnullが返る
if(CodePtr code = compile(source)){
    code->call(); // コンパイルしたコードを実行する
}

XTAL_CATCH_EXCEPT(e){
    stderr_stream()->println(e); // stderrに例外オブジェクトを出力する
}
\endcode
*/

/** \page useanyptr AnyPtr
AnyPtrはあらゆるXtalのオブジェクトを保持できる、スマートポインタ型である。 \n   
AnyPtrは、正確にはSmartPtr<Any>型のtypedefとなっている。 \n

\section int2anyptr 整数や浮動小数点数型とAnyPtrを相互変換する 
AnyPtrは全ての整数や浮動小数点数を受け取るコンストラクタが定義されているので、C++の整数からAnyPtrへの変換は自動的に行われる。\n
\code
AnyPtr any = 100;
AnyPtr value = 10.0f;
\endcode
逆にAnyPtr型に入れたものを整数に変換したい場合、Any::to_iメンバ関数やAny::to_fメンバ関数を使う。\n
\code
int i = any->to_i();
float f = value->to_f();
\endcode
これは、オブジェクトによっては、Xtalレベルでto_iメソッドが呼ばれるため、例外が発生するかもしれないことに注意する。\n

\section str2anyptr 文字列とAnyPtrを相互変換する 
AnyPtrはconst char_t*を受け取るコンストラクタが定義されているため、文字列リテラルからAnyPtrへの変換は自動的に行われる。\n
ただし、UNICODEなどの可搬性を高めるために、XTAL_STRINGマクロで囲む方が望ましい。\n
\code
AnyPtr str = "string";
AnyPtr str2 = XTAL_STRING("string");
\endcode
逆にAnyPtrからC++の文字列に直すには、Any::to_sメンバ関数を使います。\n
to_sはStringPtrを返してくるので、StringPtrで受け取ります。\n
StringPtrはSmartPtr<String>のtypedefです。\n    
\code
StringPtr s = str->to_s();
\endcode
そして、String::c_str関数で、const char_t*として受け取れます。\n
\code
const char* ccp = s->c_str();
\endcode

\section anyptr2any AnyPtr型を他の型に変換する
Stringを保持しているAnyPtrをStringPtr型に変換する場合、to_sメソッドを使うということを説明したが、
その他にもptr_cast関数を使うことでString型に変換することができる。\n
ptr_cast関数は次のような関数である。    \n

\code
template<class T>
const SmartPtr<T>& ptr_cast(const AnyPtr& from);
\endcode
dynamic_cast等のC++のキャストと同じ形式で呼び出すようにデザインされている。\n
\code
AnyPtr astr = XTAL_STRING("string");
StringPtr str = ptr_cast<String>(astr);
\endcode
もしキャストに失敗したら、nullを返す。\n  
\n
絶対にこれはStringPtrだ、と確定している場合、unchecked_ptr_castが使える。\n
ptr_castは継承関係を調べるため若干時間がかかるが、unchecked_ptr_castは高速に動作する。\n
\code
AnyPtr astr = XTAL_STRING("string");
StringPtr str = unchecked_ptr_cast<String>(astr);
\endcode
*/

/** \page usenew C++のオブジェクトをXtalが管理できるオブジェクトとして作成する
xnew<T>テンプレート関数を使うと、C++のオブジェクトをXtalでも扱えるオブジェクトとして作成できる。\n  
xnew<T>関数は、SmartPtr<T>型を返します。\n
\n
例えば、組み込みの配列のArrayクラスはC++で記述されているが、これをC++で生成するには次のように書く。\n
\code
// ArrayPtr は SmartPtr<Array> のtypedef
ArrayPtr ary = xnew<Array>(10); //長さ10の配列を生成
ary->push_back(10);
\endcode
 
ユーザーが定義したクラスもxnewで生成できる。\n
\code
class Foo{
pubic:
    void bar(){}
};
SmartPtr<Foo> foo = xnew<Foo>();
foo->bar();
\endcode
これらのオブジェクトは、すべてのスマートポインタが参照しなくなれば、\n
自動的にガーベージコレクションで削除されるので、明示的なdeleteは必要無い。\n
*/

/** \page usefuncall 関数の呼び出し
AnyPtrに格納されているのがC++の関数かXtalの関数かに関わらず、Any::callを使う。\n
\code
// fooに格納されている関数を引数(100, 200)で呼び出す
AnyPtr ret = foo->call(100, 200);
\endcode
名前つき引数の呼び出しも下のように手軽に記述できる。\n
\code
// fooに格納されている関数を引数(10, hoge:50)で呼び出す
AnyPtr ret = foo->call(10, Xnamed(hoge, 50));
\endcode
*/

/** \page usesend メソッド呼び出し
AnyPtrに格納されているのがC++のオブジェクトかXtalのオブジェクトかに関わらず、Any::sendを使う。\n
\code
int len = obj->send(Xid(length))->to_i();
AnyPtr ret = obj->send(Xid(slice), 0, 2);
printf("%s\n", ret->to_s()->c_str());
\endcode
Xidというのは、Xtalのintern済み文字列を簡単に生成するためのマクロである。\n
\n
セカンダリキー付で呼び出すにはAny::send2を使う。\n
*/

/** \page useiter イテレータの巡回
配列や連想配列、またはイテレータを巡回する場合、Xforマクロを使う。\n
\code
Xfor(val, array){
    val.p;
}

Xfor2(key, val, map){
    key.p;
    val.p;
}
\endcode
このXforの中では普通にbreakなどで抜け出すことができる。\n
また、first_stepという、ループの一番最初かどうかをしめすローカル変数も定義されていて使うことができる。\n
\n
セカンダリキー付で呼び出すにはAny::send2を使う。\n
*/


/** \page userefmember メンバの取得
これにはAny::memberメンバ関数を使います。\n
取得したいメンバの名前を第一引数として渡す。\n    
もし取得したいメンバが無い場合、undefinedが返る。\n
メンバの取得に関してはC++レベルでは例外はセットされない。\n  
\code 
// クラスのfooメンバを取り出す
AnyPtr foo = cls->member(Xid(foo));

// クラスのbarメンバを取り出す
AnyPtr bar = lib()->member(Xid(bar)):
\endcode
*/

/** \page useat 配列、連想配列の要素取得、設定
Array::at, Map::atを使って、要素の取得を行う。\n
\code
ret = ary->at(0);
ret = map->at("key");
\endcode
 
値の設定は、Array::set_at, Map::set_atを行う。\n
\code
any->set_at(0, 10);
map->set_at("test", 5);
\endcode
*/

/** \page usereturn Xtalからオブジェクトを受け取る
Xtalから値をC++に返すにはトップレベルでreturn文を使う。\n
\code
// test.xtal
return 100 + 20;
\endcode
 
\code
// C++
// test.xtalファイルを実行し、returnされたオブジェクトを得る
AnyPtr ret = load("test.xtal");

// 例外が発生していないかチェック
XTAL_CATCH_EXCEPT(e){
    stderr_stream()->println(e);
}
else{
    int value = ret->to_i();
    printf("%d\n", value);
}
\endcode

もしreturn文が無い場合、toplevelオブジェクトが自動的に返される。\n
\code
// test.xtal
hoo: 100;

class Vec{
	public _x; 
	public _y;
}

\endcode
 
\code
// C++
// test.xtalファイルを実行し、returnされたオブジェクトを得る
AnyPtr ret = load("test.xtal");

// 例外が発生していないかチェック
XTAL_CATCH_EXCEPT(e){
    stderr_stream()->println(e);
}
else{
	// toplevelのhooを取得する
	int value = ret->member(Xid(hoo))->to_i();
    printf("%d\n", value);
}
\endcode
*/

/** \page usecppclass C++のクラスをバインドする
\section secvec2d 2次元のベクトルクラスをXtalで使えるようにする例
\code
// test.cpp
class Vector2D{
public:
    float x, y;
    
    Vector2D(float x = 0, float y = 0)
        :x(x), y(y){}
    
    float length() const{
        return sqrt(x*x + y*y);
    }
    
    void normalize(){
        float len = length();
        x /= len;
        y /= len;
    }
};

// XTAL_PREBINDの中で継承関係の登録、コンストラクタの登録、所属する名前空間を指定する
XTAL_PREBIND(Vector2D){
	Xregister(Lib); // libにVector2Dという名前で登録する
    
    // コンストラクタの登録
	Xdef_ctor2(float, float);
		Xparam(x, 0); // オプショナル引数x
		Xparam(y, 0); // オプショナル引数y
}

// XTAL_BINDの中でメンバ関数の登録を行う
XTAL_BIND(Vector2D){
    // itはClassPtrである。
    // it->でClassクラスのメンバ関数が呼べる

    Xdef_var(x); // メンバ変数xのセッタ、ゲッタを登録
    Xdef_var(y); // メンバ変数yのセッタ、ゲッタを登録
    Xdef_method(length); // メンバ関数lengthを登録
    Xdef_method(normalize); // メンバ関数lengthを登録
}
\endcode

\code
// test.xtal
vec: lib::Vector2D(10, 20);
vec.length.p;
vec.normalize;
vec.length.p;
\endcode

\section secfoo 継承関係があるクラスをバインドする
C++のクラスに継承関係がある場合、Class::inheritを使って、継承関係を登録する必要があります。\n
これをしなければ、キャストに失敗します。\n
\code
class Foo{
public:
    int a;
    Foo(int value = 0){ a = value; }
    int add(int b){ return a+=b; }
};

class SubFoo : public Foo{
public:
    SubFoo(){}
};

XTAL_PREBIND(Foo){
	Xregister(Lib); // libにFooという名前で登録する

    Xdef_ctor1(int);
		Xparam(value, 0);
}

XTAL_BIND(Foo){
    Xdef_method(add);
}

XTAL_PREBIND(SubFoo){
 	Xregister(SubFoo); // libにFooという名前で登録する
	
   // Xinheritで継承関係を結ぶ
    Xinherit(Foo);
	
	// コンストラクタを定義する
    Xdef_ctor0();
}

void test(){
    // 実行して結果を受け取る
    AnyPtr ret = load("test.xtal");
    
    // ここでretにはSubFooクラスのインスタンスが入っている
    
    // ptr_castでSubFooとして取り出せる
    SmartPtr<SubFoo> subfooref = ptr_cast<SubFoo>(ret);
    
    // SubFooと継承関係を結んだFooとしても取り出せる
    SmartPtr<Foo> fooref = ptr_cast<Foo>(ret);
}
\endcode

\code
// test.xtal
// libオブジェクトに登録された、SubFooクラスを取り出してインスタンスを生成する。
subfoo: lib::SubFoo(); 
return subfoo; // subfooをreturnする
\endcode
*/

/** \page usebytecode バイトコードにコンパイルする
\code
// ファイルをコンパイルする
if(CodePtr code = compile_file("test.xtal")){
    // test.xtalcにバイトコードを保存する
    FileStreamPtr fs = xnew<FileStream>("test.xtalc", "w");
    fs->serialize(code);
}
\endcode

\code
// バイトコードを読み出す
FileStreamPtr fs = xnew<FileStream>("test.xtalc", "r");
if(CodePtr code = ptr_cast<Code>(fs->deserialize())){
    // 実行する
    code->call();
}
\endcode
*/

