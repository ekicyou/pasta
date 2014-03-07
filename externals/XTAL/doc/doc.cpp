
/**
* \biref xtalのライブラリ
*/
namespace lib{

    /**
    * \brief ビルトインのクラスや関数
    */
    namespace builtin{
    
        /**
        * \brief ファイルシステムのクラスや関数
        */
        namespace filesystem{}
        
        /**
        * \brief デバッグのクラスや関数
        */
        namespace debug{}
        
        /**
        * \brief xpegのクラスや関数
        */
        namespace xpeg{}
    }
}




/** \mainpage Programing Xtal
Xtalは、主にゲーム製作用途のために、C++に組み込み、連携させて使うために開発された言語です。\n
基本的にXtalを単体で使うことは無く、C++と組み合わせて使います。\n
このドキュメントも、C++を使用できることを前提に記述されています。\n
\n
XtalはRuby, Python, Lua, NewtonScript, ECMAScript等、とても多くの言語を参考にして作られました。\n
\n

\section feature 特徴

- クラスベースオブジェクト指向 
- C++ likeな構文 
- C++に容易に組み込める 
- 動的な型 
- クロージャ 
- ファイバー (コルーチン、マイクロスレッド) 
- byte codeによる仮想マシン駆動 
- native thread対応 
- MultiVM 

\section xindexsec Xtal
- \subpage tutorial
- \subpage syntax
- \subpage xtalapis
\section cppindexsec C++
- \subpage cppapis
- \subpage usepage
*/

/** \page xtalapis Xtalの標準クラスや関数
以下を参照してください。\n
- \link lib::builtin Xtalの標準クラスや関数 \endlink
*/

/** \page cppapis C++のAPI
以下を参照してください。\n
- \link xtal C++のAPI \endlink
*/

/** \page syntax Xtal言語構文
- \subpage comment
- \subpage identifier
- \subpage reserved
- \subpage defvar
- \subpage expr
- \subpage sentence
- \subpage overloadable_operator
*/

/** \page expr 式
- \subpage pseudovar
- \subpage numberliteral
- \subpage string
- \subpage array_map
- \subpage fun
- \subpage method
- \subpage lambda
- \subpage fiber
- \subpage class
- \subpage singleton
- \subpage operator
*/

/** \page operator 演算子
- \subpage call
- \subpage method_call
- \subpage refmember
- \subpage secondary_key
- \subpage basic_operator
- \subpage cmp_operator
- \subpage etc_operator
- \subpage operator_priority
*/

/** \page sentence 文
- \subpage expr_sentence
- \subpage assign
- \subpage op_assign
- \subpage branch
- \subpage loop
- \subpage except
*/

/** \page tutorial Xtal言語クイックチュートリアル
\section secdebugprint デバッグプリント
\code
100.p;
"test".p;
\endcode
\section secvar 変数
\code
foo: 100;
bar: 200;
(foo - bar).p; //=> -100
foo = 10;
bar = 20;
(foo - bar).p; //=> -10
\endcode

\section secmvar 多重定義・多重代入
\code
foo, bar: 100, 200;
(foo - bar).p; //=> -100
foo, bar = 10, 20;
(foo - bar).p; //=> -10
\endcode

\section secarray 配列
\code
array: [];
array.push_back(100);
array.push_back(200);
array.p; //=> [100, 200]
\endcode

\section secmap 連想配列
\code
map: [:];
map["test"] = 100;
map["test"].p; //=> 100
\endcode

\section secblock ブロック
\code
array: [10, 20, 30];
array{
    it.p;
}
//=> 10
//=> 20
//=> 30
\endcode

\section secfun 関数
\code
fun foo(a, b){
    return a - b;
}
foo(15, 5).p; //=> 10
\endcode

\section seclambda ラムダ
\code
lambda: |a, b| a + b;
lambda(10, 20); //=> 30
\endcode

\section secclass クラス
\code
class Bar{
    _boo;
    
    initialize(boo){
        _boo = boo;
    }

    say{
        _boo.p;
    }
}

bar: Bar("boo!");
bar.say; //=> boo! 
\endcode

*/


/** \page comment コメント
コメントはC++と同一の記述が可能です。\n
\n
それに加え、\#!で始める一行コメントも可能です。\n
\code
#! this is comment
\endcode
#単体ではコメントとならないことに注意してください。\n
*/

/** \page identifier 識別子
変数名はアルファベット、数字、アンダースコアを使えます。 ただし数字で始まる変数名は認められません。\n
文字コードの設定が適切であれば、日本語も使用することができます。\n
アンダースコアで始まる名前はインスタンス変数にしか使えません。\n
予約語と同じ識別子は使用することが出来ません。\n
\n
ただし、.演算子と::演算子の直後の場合は、予約語でも使用することが出来ます。\n

\code
test: 100; // ok
foo123: 200; // ok
alpha_0_test: 300; // ok
\endcode

\code
class: 400; // bad! classは予約語
if: 500; // bad! ifは予約語
\endcode

\code
100.class; // ok .演算子の後なので予約語が使える
Int::if: 600; // ok ::演算子の後なので予約語が使える
\endcode

また、文字コードの設定が正しく行われているならば、日本語の識別子も使うことが出来ます。 \n

\code
あいう: 100; // ok
\endcode

*/

/** \page reserved 予約語
予約語の一覧です。 \n

- if 
- for 
- else 
- fun 
- method 
- do 
- while 
- continue 
- break 
- fiber 
- yield 
- return 
- once 
- null 
- undefined 
- false 
- true 
- xtal 
- try 
- catch 
- finally 
- throw 
- class 
- callee 
- this 
- current_context 
- dofun 
- is 
- in 
- assert 
- nobreak 
- switch 
- case 
- default 
- singleton 
- debug 
- public 
- protected 
- private
*/

/** \page defvar 変数定義
\section local ローカル変数の定義

現在のスコープに変数を定義する文です。\n
初期値を伴わない変数の定義は出来ません。\n 
変数には静的な型はなく、なんの種類のデータの参照でも保持出来ます。\n 
\code
foo: 111;
bar: "test";
foobar: null;
\endcode

\section defmultivar ローカル変数の多重定義 

右辺に多値を返す式がある場合、それをばらして受け取るために、ローカル変数の多重定義ができます。\n
もし右辺の値の方が多い場合、左辺最後の変数に多値として代入されます。\n
右辺の値の方が少ない場合、undefinedで埋められます。\n
\code
{
    a, b: 0, 1;
    a.p; //=> 0
    b.p; //=> 1
}

{
    foo: fun(){
        return 5, 10;
    }
    a, b: foo();
    a.p; //=> 5
    b.p; //=> 10
}
\endcode
*/

/** \page pseudovar 擬似変数
変数参照のように扱えるオブジェクトです。

\section null 
何も無いことを表すオブジェクトです。\n
if文などの条件式で、nullは「偽」となります。\n

\section true 
「真」を表すオブジェクトです。\n
ただし、nullとfalseとundefined以外の全てのオブジェクトは「真」であるので、他に適当な値が無い場合にだけこれを使用することになるでしょう。\n

\section false 
「偽」を表すオブジェクトです。 \n
if文などの条件式で、falseは「偽」となります。 \n

\section undefined 
無値を表すオブジェクトです。
例えば ::?演算子 や .?演算子 で、存在しないメンバを取り出そうとしたときに返されるオブジェクトです。\n
if文などの条件式で、undefinedは「偽」となります。\n

\section callee 
今実行している関数自身を指してます。 よって次のようにすることで再帰呼び出しが可能です。\n
\code
callee();
\endcode

\section lib 
ライブラリのロード、登録を司るシングルトンクラスオブジェクトです。\n
lib::foo とすると、foo.xtalファイルを検索し、コンパイル、実行して、そのファイルでreturnされた値を保存します。\n
二度目以降のアクセスはその保存された値が返されます。\n
\code
// foo.xtalを読み込み、returnされた値をfoo変数を定義
foo: lib::foo; 

// 既にfoo.xtalは読み込まれているので、保存された値でhoge変数を定義する。
hoge: lib::foo; 
\endcode

上のようなファイル読み込み以外にも、libオブジェクトにメンバを直接定義することも可能です。\n
\code
lib::foo: "foo!";
lib::foo.p; //=> foo!
\endcode

append_load_pathメソッドを使うと、検索パスの追加が出来ます。\n
\code
lib.append_load_path("library")
\endcode

\section filelocal
そのファイル固有ののトップレベルを指す名前空間的に扱われるシングルトンクラスオブジェクトです。 \n
静的に見つからない変数参照は、filelocalから検索されます。\n
また、filelocalはbuiltinオブジェクトをinheritしているので、builtinに定義されている関数またはクラスも自動的に検索対象となります。\n
\code
println(foo); // filelocal::println(filelocal::foo); と同義
\endcode
*/

/*
\section defmember メンバ定義文 

書式:
\code
クラスを返す式 :: 識別子 : 式 ; 
\endcode

既存のクラスにメソッドなどを定義する構文です。\n
例:
\code
// 整数のクラスに自身を2倍の数を返すメソッドを追加
Integer::x2: method(){
    return this*2;
}
100.x2.p; //=> 200
\endcode

既に同名のメンバが定義されている場合、例外が発生します。 \n
*/

/** \page numberliteral 数値リテラル
\section integer 整数リテラル 
整数を生成する式は次のように記述します。\n
\code
0
123
58325
\endcode
見やすいように途中にアンダーバーを入れることも出来ます。\n
\code
1_234_567
\endcode

\subsection hex 16進数整数リテラル 
\code
0x3f
0Xfd_2A
\endcode

\subsection bin 2進数整数リテラル
\code
0b1111
0B1111_0011
\endcode

\section float 浮動小数点数リテラル
浮動小数点数を生成する式は次のように記述します。\n

\code
12.3
0.56
\endcode

eまたはEを使い、10を底とする指数表記も出来ます。 eは大文字小文字問われません。\n
また、ピリオド(.)の直後にeを置くことは出来ません。 \n
それはメンバ参照演算子としてパースされます。 eの前に0を置いてください。\n

\code
50.0e-5
15.5E+2
56.0e10
\endcode

\section suffix サフィックス
整数リテラルと浮動小数点数リテラルは次のサフィックスをつけることが出来ます。\n
- f 浮動小数点数リテラルとして扱われる 
- F 同上 

\code
10.5f
255f
\endcode

数値はどれも変更不能オブジェクトです。 \n
そのため += 演算子などは a = a + b; と同じ意味となります。\n
*/

/** \page string 文字列リテラル
文字列を生成する式はダブルクォート(")で文字列を挟んで記述します。\n
\code
"String"
\endcode

使用できるエスケープシーケンスは以下のとおりです。\n
- \\b バックスペース 
- \\f 改ページ 
- \\n 改行 
- \\r 復帰 
- \\t 水平タブ 
- \\v 垂直タブ 
- \\\\ バックスラッシュ (円記号) 
- \\" 引用符 

文字列は変更不能オブジェクトです。 \n
そのため a ~= b; という連結代入演算子は、 a = a ~ b; と同じ意味となります。\n
\n
文字は一文字の文字列を使って表します。文字リテラルは存在しません。\n

\section parcent %記法 

Rubyのような%記法も存在します。\n
\code
%!This is a "pen"! // "This is a \"pen\"" と等しい
\endcode

!の部分には好きなASCIIの記号文字を使用できます。\n
始まりの記号文字として、[ ( < { のどれかを使った場合、終りの記号に対応する ] > ) } を使います。\n
括弧を使用した場合、内部でまた同じ括弧があるとネストします。\n

\code
%[ai[u]eo] // "ai[u]eo" と等しい
\endcode

また、%%記法では、%%と始まりの記号文字の間の文字によって特別な意味が付与されます。\n
- f フォーマット文字列 
- t get text文字列 

\subsection フォーマット文字列 
%%の後に f を入れると、それはフォーマット文字列となります。\n

\code
format_object: %f!This %s a %s.!;
// format_objectは二つの値を受け取る関数となる

format_object("is", "pen").p; // This is a pen. 
\endcode

Pythonのように、%(name)sという風に記述することで、名前付きにすることも出来ます。\n
\code
format_object: %f[x=%(x)d, y=%(y)d];
format_object(y: 10, x: 20).p; // x=20, y=10
\endcode

\subsection text get text文字列 
%%の後に t を入れると、それはget text文字列となります。\n

*/


/** \page array_map 配列、連想配列

\section secarray 配列生成式 
配列を生成する式は次のように記述します。\n
\code
[]
[0, 1, 2]
["str", 5, 5.6, ]
\endcode
カンマに区切られた値を要素とした配列オブジェクトを生成します。 \n
配列の要素はどんなデータが混在してもかまいません。 \n
最後のカンマは付けても付けなくてもかまいません。\n

\section secmap 連想配列生成式 
連想配列を生成する式は次のように記述します。\n
\code
[:]
["key":"value", 5:100]
["one":1, "two":2, ]
\endcode
キーと値を:で区切ったものをカンマ(,)区切りにすると連想配列オブジェクトの生成となります。\n 
最後のカンマは付けても付けなくてもかまいません。\n
\n
配列、連想配列はどれも変更可能オブジェクトです。 \n
そのため += 演算子などはaを破壊的に変更します。\n
*/

/** \page fun 関数生成式
関数を生成する式は次のように記述します。\n
\code
foo: fun(name1, name2){    
    return name1 + name2;
}
foo(10, 20).p; //=> 30
\endcode

fun()の後をブロックではなく式を記述した場合、自動的にこの式の結果を返す関数となります。\n
\code
foo: fun(name1, name2) name1 + name2;
foo(10, 20).p; //=> 30
\endcode

fun, method, fiber, class, singletonでは、変数定義と同時に生成式で初期化をしたい場合、\n
\code
foo: fun(){}
bar: class{}
\endcode
このような記述を、C++に似た記述となる次のような書き方が許されます。\n
\code
fun foo(){}
class bar{}
\endcode

2つの値のうち、大きい方を返す関数maxは次のように書けます。\n
\code
fun max(a, b){
    if(a<b){
        return b;
    }else{
        return a;
    }
}

// または
// fun max(a, b) a<b ? b : a;
\endcode

引数を一つも取らない場合、()は省略することも可能です。\n
\code
foo: fun{
    "foo!".p;
}

fun bar{
    "bar!".p;
}
\endcode

\section arguments デフォルト引数・名前付き引数 
引数名:値 という書式でデフォルト値の指定が出来ます。\n
デフォルト引数を付加した引数は自動的に名前付き引数となり、呼び出すときに引数名:値と指定します。\n
\code
foo foo_with_default_value(a: 0, b: 10){
    (a + b).p;
}

foo_with_default_value(); //=> 10
foo_with_default_value(a: 50); //=> 60
\endcode

\section argarg 可変引数 
関数の引数の最後の名前の前に...をつけると、可変引数を取る関数定義となります。\n
\code
foo foo(a, ...b){
    a.p;
    b[0].p; // 引数bに可変長引数オブジェクトが入っている
    b["named"].p;
    bar(...b); // bar関数に可変引数部分全てを渡す
}

foo(5, 10, 20, named:"test"); 
//=> a 5
//=> 0 10
//=> named test
\endcode

可変引数オブジェクトはArgumentsクラスのインスタンスであり、Argumentsクラスのメソッドを呼ぶことが出来ます。
\code
fun foo(...a){
    // 順番指定引数を全てプリントする
    a.each_ordered{
        it.p;
    }
}
\endcode

可変長引数を展開して渡したい場合、...argsのように、...を前置します。\n
\code
fun foo(...args){
    bar(args); // 可変長引数オブジェクトをひとつの引数として関数呼び出しをする
    bar(...args); // 可変長オブジェクトを展開して複数の引数として関数呼び出しをする
}
\endcode

*/

/** \page lambda ラムダ生成式
lambda式は引数の受け取り方が多重代入と同じルールの関数生成式です。\n
\code
bar: |a, b| { return a + b; }
bar(10, 20); //=> 30

foo: |a, b| a + b;
foo(10, 20).p; //=> 30


// 渡すほうが少ない場合、多重代入のように、最後の多値が展開される。
hoge: |a, b| a + b;
hoge( (5, 6) ).p; //=> 30 

// 渡すほうが多い場合、多重代入のように、多値にまとめられる。
bar: |a| a;
bar(7, 8).p; //=> (7, 8)
\endcode

lambda式はその特性上、ならびに構文の都合上、デフォルト・キーワード引数を持つことは出来ません。\n
*/

/** \page method メソッド生成式
メソッドを生成する式の書式は、関数生成式のfunの部分をmethodに変えるだけです。\n
\code
m: method(name1, name2){
    return name1 + name2;
}
m(10, 20).p; //=> 30
\endcode

クラスのスコープの中では、メソッドの定義はかなり簡略化した記述が可能です。\n
\code
class Foo{
    bar(a, b){
        
    }
    
    // これは下と等しい
    
    // bar: method(a, b){
    //    
    // }
}
\endcode

funで関数を生成することの違いは、擬似変数thisの扱われ方だけです。 \n
funの場合thisは、その外側のスコープのthisがそのまま使われます。 \n
しかしmethodの場合、thisが何を指すかは、呼び出し側で決定されます。\n
methodが obj.foo() 形式で呼び出されたとき、thisはobjとなります。 \n
そうでなく、ただ foo() と呼ばれたときは、その呼び出した関数のthisがそのままthisとして使われます。\n

\code
class Foo{
    return_fun{
        return fun{
            this.boo();
        }
    }

    return_method{
        return method{
            this.boo();
        }
    }

    boo{
        "boo!".p;
    }
}

// Fooクラスをインスタンス化
foo: Foo();

// return_funメソッドが返す関数をfに入れる
f: foo.return_fun();

// ここでfを呼び出すと boo! と表示される。
f();

// このように、funは関数が生成されたときのthisを記憶しているので、
// methodから返された場合でも、thisが正しくオブジェクトを指しています。


// methodを返す場合どうなるか

// return_methodメソッドが返す関数をmに入れる
m: foo.return_method();

// ここでfを呼び出すと例外が送出される。
f();

// なぜなら、methodは関数が作られた場所のthisを記憶せず、呼び出し元のthisを使用するためです。
// トップレベルではthisは、filelocalですので、filelocalがthisとして渡されていることになります。
\endcode
*/

/** \page fiber ファイバー生成式
ファイバーを生成する式の書式は、関数生成式のfunの部分をfiberに変えるだけです。 \n
ファイバーはyieldを使うことで、実行を一時中断して、また途中から再実行が出来る特殊な関数です。 \n
\n
他の言語ではコルーチン、マイクロスレッドなどと呼称されているものです。\n
\code
fib: fiber(){
    1.p;
    yield;
    2.p;
}

fib(); //=> 1
fib(); //=> 2
\endcode

yieldで値を返すことも出来ます。
\code
fib: fiber(){
    yield 10;
    yield 20;
    yield 30;
}

fib().p; //=> 10
fib().p; //=> 20
fib().p; //=> 30
\endcode

yieldに値を渡すこともできます。
\code
fib: fiber(){
    a: yield; 
    a.p;
    b: yield;
    b.p;
}

fib(); // 最初のyieldまで実行される
fib(10); // yield式から返される値を渡す //=> 10
fib(20); //=> 20

fiberのthisの扱われ方はfunと同じです。\n
\n
また、fiberはIteratorオブジェクトとしても扱えます。 つまり、ブロック文の適用が可能です。\n
\code
fib{ |val|
    val.p;
}
\endcode

ファイバーは親子関係が明確に存在しており、スレッドのような並列の関係を持つことは出来ない、と思うかもしれません。\n
しかし、次のような機構を書けば、並列の関係を持たせることが可能です。\n
\code
class FiberScheduler{
    _fibers: [];

    step{
        for(i: 0; i<_fibers.length; ++i){
            it: _fibers[i];
            it();
            if(it.is_finished()){
                _fibers.erase(i);
                i--;
            }
        }
    }

    run{
        while(!_fibers.is_empty())
            step();
    }

    register(f){
        _fibers.push_back(f);
    }
}

fs: FiberScheduler();

fs.register(fiber{
    i: 0;
    while(i<10){
        i.p;
        i++;
        yield;
    }
});

fs.register(fiber{
    i: 0;
    while(i<10){
        i+5 .p;
        i++;
        yield;
    }
});

fs.run;
\endcode
*/

/** \page expr_sentence 式文
式の後に;(セミコロン)を置くと文となります。\n

\code
100;
foo.bar;
println(10);
\endcode

関数生成式やクラス生成式など、いくつかの}で終わる式は、}の後に;があるかのように動作します。\n
したがって、}で終わる式は、その後に;は必要ありません。\n
その代わり、右に式を続けて書くことが出来ません。\n
\code
foo: class{ 
    foo: fun(){
    
    }
}::foo(); // コンパイルエラー
\endcode
*/

/** \page class クラス生成式
classを生成する式です。\n
\code
// TestClassという変数にクラスを入れる
TestClass: class{
// ↑これはclass TestClass という書き方も許される
    
    // インスタンス変数は _ (アンダースコア) で始めなければならない。
    _value; // インスタンス変数_valueを宣言
    _test: 100; // このような初期化の仕方も可能。

    // initializeという名前のメソッドはインスタンス生成時に呼び出される
    initialize{ 
        _value = 0; // インスタンス変数_valueを0で初期化
    }
    
    add(val){ // メソッドaddを定義
        _value += val; // インスタンス変数_valueにvalを足す
    }

    print{
        println(_value);
    }
}

// classオブジェクトに対し、関数呼び出し式を使うとそのインスタンスを生成できる
t: TestClass(); 
t.add(10);
t.print(); // 10が出力される
\endcode
    
インスタンス変数は外部から参照することが出来ません。\n
\code
t._value
\endcode
という式は不正です。\n

また、インスタンス変数は、そのclass生成式のスコープの中にあるメソッドからしかアクセスすることが出来ません。 \n
つまり、サブクラスの中からでも継承元のインスタンス変数は不可視です。\n
\n
また、Xtalではclassの定義後にメソッドを追加することが可能ですが、 その中でもインスタンス変数に触ることは出来ません。\n
\code
method TestClass::access(){
    _value = 10; // エラー！
}
\endcode
そのため継承先に同名のインスタンス変数があったとしても競合することはありません。 それらは完全に別のモノとして扱われます。\n

\section 可触性

メンバ定義の際、\n
何もつけないかまたはpublicをつけるとpublicメンバ\n
protectedをつけるとprotectedメンバ\n
privateを付けるとprivateメンバとなります。\n
\code
class Foo{
  // public
  test0{}
  public test1{}

  // protected
  protected test2{}
    
  // private
  private test3{}
}
\endcode
publicはどこからでもアクセスできるメンバ\n
protectedは自身のクラスかまたは継承先のクラスのメソッドからしかアクセスできないメンバ\n
privateは自身のクラスからしかアクセスできないメンバです。\n


\section propety プロパティ

引数を取らないメソッド呼び出しは()を省略できるため、値の取得に関してはただ()を省くだけです。
\code
obj.foo;
\endcode

値の設定は obj.foo = 100; と書きます。これは obj.set_foo(100) のシンタックスシュガーです。 \n
つまり、次のようにメソッドを定義すれば、プロパティの実装が出来ます。\n

class Foo{
    _hoge;

    hoge{
        return _hoge;
    }

    set_hoge(v){
        _hoge = v;
    }
}

foo: Foo();
foo.hoge = 100;
foo.hoge.p; //=> 100

foo.hoge += 10;
foo.hoge.p; //=> 110
\endcode

単純にインスタンス変数を返したり、インスタンス変数に設定したりするだけなら、次のような簡易的な記述が出来ます。\n
\code
class Foo{
    public _hoge;
}

foo: Foo();
foo.hoge = 100;
foo.hoge.p; //=> 100

foo.hoge += 10;
foo.hoge.p; //=> 110
\endcode
インスタンス変数定義文で、可触性の修飾子をつけると、コンパイラがhogeメソッドと、set_hogeメソッドを、その可触性で定義します。\n
getterとsetterで可触性を変えたい場合、メソッドを自分で定義してください。\n

\section adddef クラス定義後の追加メンバ定義
クラスの定義後でもメンバの追加定義が可能です。\n
\code
class Foo{}
Foo::hoge: 10;
\endcode
Fooクラスにhogeメンバを追加定義します。\n
既に同名のメンバがある場合、エラーとなります。\n

文字列で動的に名前を決定することもできます。\n
\code
name: "test"
Foo::("set_" ~ name): method(v){

}
\endcode

\section inherit 継承
classの継承は次のように書きます。\n
\code
class A{
    // 初期化関数
    initialize(x){
        
    }
    
    foo{

    }
}

class B(A){
    // 初期化関数
    initialize(x){
        // Aの初期化関数を呼び出す
        A::initialize(x);
    }
    
    // fooをオーバーライドする
    foo{
        // A::fooを呼び出す
        A::foo();
    }
}
\endcode

\section minherit 多重継承 
Xtalではclassの多重継承をサポートします。カンマで区切って指定します。\n
\code
class C(Foo, Bar){

}
\endcode

\section poly ポリモーフィズムを期待したメソッド呼び出し
クラスのメンバ関数の中で、ポリモーフィズムを期待したメソッド呼び出しをしたい場合。\n
this.と前置した呼び出しをしなければなりません。\n
\code
class Foo { 
    value{
        return 1;
    }
    
    show{ 
        this.value().p; 
    } 
} 

class Bar(Foo) { 
    value{ 
        return 1; 
    } 
} 
\endcode
これを単純にvalue().pとした場合、valueは静的に解決されるため、Foo::valueが呼ばれます。
*/

/** \page singleton シングルトンクラス生成式
シングルトンクラスとは、唯一のインスタンスが自分自身となる特別なクラスのことです。\n
自身がクラスであるため、メンバの関数を object::member() でも、object.member でも呼び出すことが可能です。 \n
\code
foo: singleton{
    bar{
        callee.p;
    }
}

(foo.class===foo).p; //=> true
foo::bar(); //=> foo::bar
foo.bar(); //=> foo::bar
\endcode
Xtalでは、C++の名前空間と同じような用途には、このシングルトンクラスを使います。\n
*/

/** \page assign 代入文
変数の参照先を変えるのは代入文を使います。\n

\code
// 書式:
ident = expr ; 

// 例:
name: "key";
name.p; //=> key
name = "test";
name.p; //=> test
\endcode

Xtalでは代入は文で、値を返さないため、\n
\code
if(name = 0){}
\endcode
という記述はコンパイルエラーとなります。\n

\section massign 多重代入文 
Xtalは多重変数代入もサポートします。
\code
a, b = 0, 1;
\endcode

次のように記述すれば、変数のスワップができます。\n
\code
a, b = b, a;
\endcode
*/

/** \page op_assign 演算代入文
\section inc インクリメント文
\code
i++;
++i;
\endcode
iをインクリメントします。\n
インクリメント文は、次の形式のシンタックスシュガーです。\n
\code
i = i.op_inc();
\endcode

\section dec デクリメント文 
\code
i--;
--i;
\endcode
iをデクリメントします。\n
デクリメント文は、次の形式のシンタックスシュガーです。\n
\code
i = i.op_dec();
\endcode

\section add_assign 加算代入文 
\code
a += b
\endcode
aにbを加算した結果をaに代入します。\n
これは、a = a.op_add_assign(b)の省略形です。\n

\section sub_assign 減算代入文 
\code
a -= b
\endcode
aからbを減算した結果をaに代入します。\n
これは、a = a.op_sub_assign(b)の省略形です。\n

\section cat_assign 連結代入文 
\code
a ~= b
\endcode
aにbを連結した結果をaに代入します。\n
これは、a = a.op_cat_assign(b)の省略形です。\n

\section mul_assign 乗算代入文 
\code
a *= b
\endcode
aにbを掛け合わせます。\n
これは、a = a.op_mul_assign(b)の省略形です。\n

\section div_assign 除算代入文 
\code
a /= b
\endcode
aをbで除算した結果をaに代入します。\n
これは、a = a.op_div_assign(b)の省略形です。\n

\section mod_assign 剰余代入文 
\code
a %= b
\endcode
aをbで割ったあまりをaに代入します。\n
これは、a = a.op_mod_assign(b)の省略形です。\n

\section and_assign bitwise and代入文 
\code
a &= b
\endcode
aとbのbitwise andした結果をaに代入します。\n
これは、a = a.op_and_assign(b)の省略形です。\n

\section or_assign bitwise or代入文 
\code
a |= b
\endcode
aとbのbitwise orした結果をaに代入します。\n
これは、a = a.op_or_assign(b)の省略形です。\n

\section xor_assign bitwise xor代入文 
\code
a ^= b
\endcode
aとbのbitwise xorした結果をaに代入します。\n
これは、a = a.op_xor_assign(b)の省略形です。\n

\section shl_assign 左シフト代入文 
\code
a <<= b
\endcode
aをbビット左シフトした結果をaに代入します。\n
これは、a = a.op_shl_assign(b)の省略形です。\n

\section shr_assign 算術右シフト代入文 
\code
a >>= b
\endcode
aをbビット右シフトした結果をaに代入します。\n
これは、a = a.op_shr_assign(b)の省略形です。\n

\section ushr_assign 論理的右シフト代入文 
\code
a >>>= b
\endcode
aをbビット論理的右シフトした結果をaに代入します。\n
これは、a = a.op_ushr_assign(b)の省略形です。\n

*/

/** \page overloadable_operator 再定義可能な演算子の一覧

\section secassign 演算子系
<TABLE>
<TR><TD>演算子</TD><TD>意味</TD></TR>
<TR><TD>+a</TD><TD>a.op_pos()</TD></TR>
<TR><TD>-a</TD><TD>a.op_neg()</TD></TR>
<TR><TD>~a</TD><TD>a.op_com()</TD></TR>
<TR><TD>a[]</TD><TD>a.op_to_array()</TD></TR>
<TR><TD>a[:]</TD><TD>a.op_to_map()</TD></TR>
<TR><TD>a + b</TD><TD>a.op_add(b)</TD></TR>
<TR><TD>a - b</TD><TD>a.op_sub(b)</TD></TR>
<TR><TD>a ~ b</TD><TD>a.op_cat(b)</TD></TR>
<TR><TD>a * b</TD><TD>a.op_mul(b)</TD></TR>
<TR><TD>a / b</TD><TD>a.op_div(b)</TD></TR>
<TR><TD>a % b</TD><TD>a.op_mod(b)</TD></TR>
<TR><TD>a & b</TD><TD>a.op_and(b)</TD></TR>
<TR><TD>a | b</TD><TD>a.op_or(b)</TD></TR>
<TR><TD>a ^ b</TD><TD>a.op_xor(b)</TD></TR>
<TR><TD>a << b</TD><TD>a.op_shl(b)</TD></TR>
<TR><TD>a >> b</TD><TD>a.op_shr(b)</TD></TR>
<TR><TD>a >>> b</TD><TD>a.op_ushr(b)</TD></TR>
<TR><TD>a[b]</TD><TD>a.op_at(b)</TD></TR>
<TR><TD>a .. b</TD><TD>!a.op_range(b, Range::CLOSED)</TD></TR>
<TR><TD>a ..< b</TD><TD>!a.op_range(b, Range::LEFT_CLOSED_RIGHT_OPEN)</TD></TR>
<TR><TD>a <.. b</TD><TD>!a.op_range(b, Range::LEFT_OPEN_RIGHT_CLOSED)</TD></TR>
<TR><TD>a <..< b</TD><TD>!a.op_range(b, Range::OPEN)</TD></TR>
<TR><TD>a == b</TD><TD>a.op_eq(b)</TD></TR>
<TR><TD>a != b</TD><TD>!a.op_eq(b)</TD></TR>
<TR><TD>a < b</TD><TD>a.op_lt(b)</TD></TR>
<TR><TD>a > b</TD><TD>b.op_lt(a)</TD></TR>
<TR><TD>a <= b</TD><TD>!b.op_lt(a)</TD></TR>
<TR><TD>a >= b</TD><TD>!a.op_lt(b)</TD></TR>
<TR><TD>a in b</TD><TD>a.op_in(b)</TD></TR>
<TR><TD>a !in b</TD><TD>!a.op_in(b)</TD></TR>
</TABLE>

\section secassign 代入演算文系
<TABLE>
<TR><TD>演算子</TD><TD>意味</TD></TR>
<TR><TD>i++;</TD><TD>i = i.op_inc();</TD></TR>
<TR><TD>i--;</TD><TD>i = i.op_dec();</TD></TR>
<TR><TD>a[b] = c;</TD><TD>a = a.op_set_at(b, c);</TD></TR>
<TR><TD>a += b;</TD><TD>a = a.op_add_assign(b);</TD></TR>
<TR><TD>a -= b;</TD><TD>a = a.op_sub_assign(b);</TD></TR>
<TR><TD>a ~= b;</TD><TD>a = a.op_cat_assign(b);</TD></TR>
<TR><TD>a *= b;</TD><TD>a = a.op_mul_assign(b);</TD></TR>
<TR><TD>a /= b;</TD><TD>a = a.op_div_assign(b);</TD></TR>
<TR><TD>a %= b;</TD><TD>a = a.op_mod_assign(b);</TD></TR>
<TR><TD>a &= b;</TD><TD>a = a.op_and_assign(b);</TD></TR>
<TR><TD>a |= b;</TD><TD>a = a.op_or_assign(b);</TD></TR>
<TR><TD>a ^= b;</TD><TD>a = a.op_xor_assign(b);</TD></TR>
<TR><TD>a <<= b;</TD><TD>a = a.op_shl_assign(b);</TD></TR>
<TR><TD>a >>= b;</TD><TD>a = a.op_shr_assign(b);</TD></TR>
<TR><TD>a >>>= b;</TD><TD>a = a.op_ushr_assign(b);</TD></TR>
</TABLE>

*/

/** \page branch 分岐制御文
\section if if文
\code
if(a.is_empty){
    a.p;
}

if(i < 5){
    "true".p;
}
else{
    "false".p;
}
\endcode

条件式の中で変数を定義することも出来ます。
\code
if(val: foo()){
    val.p;
}
\endcode

\section switch switch文
switchはC言語と少し構文が異なります。
xtalのswitchは次のような形となります。

\code
switch(value){
    // valueが"test"と等しい場合に実行される
    case("test"){
        "test".p; 
    }

    // valueが10と等しい、または20と等しい場合に実行される
    case(10, 20){
        10.p;
    }
    
    // どのcaseとも等しくない場合に実行される
    default{
        "default".p;
    }
}
\endcode
このようにC言語ではcase部分が「case 10:」という風になりますが、Xtalでは「case(10){}」となります。\n
またswitch内のbreakもswitchを抜け出す意味とはなりません。\n
\n
case文に指定したオブジェクトは同一演算子(===)で比較されます。\n

\code
while(true){
    switch(value){
        case("test"){
            break; // 外側のwhileを抜け出す
        }
    }
}
\endcode

*/

/** \page loop 繰り返し制御文
\section for for文
\code
for(i: 0; i<10; ++i){
    println(i);
}
\endcode

\section while while文
i: 0;
while(i < 5){
    i++;
}

while文はif文と同じように条件式の中で変数を定義することも出来ます。
\code
while(i: foo()){
    println(i);
}
\endcode

\section block ブロック文
\code 
ary: [5, 4, 3];
ary.each{ |value|
    println(value);
}
\endcode

ary.eachは自身の内容を反復するIteratorオブジェクトを返すメソッドです。\n
Iteratorオブジェクトに対しブロック{}を適用するとブロック文となります。\n
{ の後に |value, value2| と書くと、Iteratorが反復した要素が順々に代入されます。}\n
\n
ブロック文の中では、暗黙的に it という名前の変数が定義されるので、それを使うことも出来ます。\n
\code
ary: [5, 4, 3];
ary.each{
    println(it);
}
\endcode

ブロック文は次のように書くのと同義です。\n
\code
ary: [5, 4, 3];
_has_next, value: ary.each.block_first();
try{
    while(_has_next){
        println(value);
        _has_next, value = _has_next.block_next();
    }
}
finally{
    _has_next.?block_break();
}
\endcode

また、配列のようにblock_firstメソッドを持つ場合、eachでIteratorを取得するまでもなくブロックが適用できます。つまり\n
\code
ary: [5, 4, 3];
ary.each{
    it.p;
}
\endcode
は次のように書いてもOKです。\n
\code
ary: [5, 4, 3];
ary{ 
    it.p;
}
\endcode


\section else else節
各ループ文にはelse節をつけることが出来ます。\n
else節は、条件式が一度も真とならなかった場合に通ります。\n
\code
for(i: 10; i<5; i++){

}
else{
    // 上のi<5は一度も真とならないので、ここを通る
}
\endcode

\section nobreak nobreak節
各ループ文にはnobreak節をつけることが出来ます。\n
nobreak節は、breakで中止しなかった場合に通ります。\n
条件式が一度も真とならない場合も同様に通ります。\n
\code
for(i: 0; i<10; ++i){
    if(foo())
        break;
}
nobreak{
    // foo()がtrueを返さなければ、ここを通る

}
\endcode
    
else節とnobreak節はどちらか一つだけつけることが可能です。\n


\section break break文
ループ文から途中で脱出できます。\n
\code
while(true){
    break; // 中断して抜ける
}
\endcode

ラベルを使い、複数のループ文から一気に脱出することもできます。\n
\code
outer: while(true){
    array.each{
        break outer; // 外側のループから抜け出す。
    }
}
\endcode

\section continue continue文
ループ文の残りの部分をスキップします。\n
\code
for(i: 0; i<10; i++){
    if(i%2 == 1){
        continue;
    }
    i.p;
}
\endcode
上の例は奇数だけを出力します。\n
    \n
breakと同じように、ラベルを利用して、特定のループ文に対して処理することが出来ます。\n
\code
outer: for(i: 0; i<10; i++){
    for(j: 0; j<10; j++){
        if(i%2 == 1){
            continue outer; 
        }
        i.p;
    }
}
\endcode
    
\section first_step first_step
各ループ文の内部では、first_stepという変数が定義されます。\n
first_stepは最初のステップ時はtrue、それ以降のステップではfalseとなります。\n
\code
for(i: 0; i<3; ++i){
    if(!first_step){
        print(",");
    }
    print(i);
}
//=> 0, 1, 2
\endcode
*/
    
/** \page basic_operator 基本的な演算子

\section pos 単項+演算子
\code
+a
\endcode
これは、a.op_pos()の省略形です。\n

\section neg 単項-演算子
\code
-a
\endcode
符号を反転します。\n
これは、a.op_neg()の省略形です。\n

\section com 単項~演算子
\code
~a
\endcode
ビットを反転します。\n
これは、a.op_com()の省略形です。\n

\section add 加算演算子
\code
a + b
\endcode
aにbを加算します。\n
これは、a.op_add(b)の省略形です。\n

\section sub 減算演算子
\code
a - b
\endcode
aからbを減算します。\n
これは、a.op_sub(b)の省略形です。\n

\section cat 連結演算子
\code
a ~ b
\endcode
aにbを連結します。\n
これは、a.op_cat(b)の省略形です。\n

\section mul 乗算演算子
\code
a * b
\endcode
aにbを掛け合わせます。\n
これは、a.op_mul(b)の省略形です。\n

\section div 除算演算子
\code
a / b
\endcode
aをbで除算します。\n
これは、a.op_div(b)の省略形です。\n

\section mod 剰余演算子
\code
a % b
\endcode
aをbで割ったあまりをaに代入します。\n
これは、a.op_mod(b)の省略形です。\n

\section and bitwise and演算子
\code
a & b
\endcode
aとbのbitwise andします。\n
これは、a.op_and(b)の省略形です。\n

\section or bitwise or演算子
\code
a | b
\endcode
aとbのbitwise orします。\n
これは、a.op_or(b)の省略形です。\n

\section xor bitwise xor演算子
\code
a ^ b
\endcode
aとbのbitwise xorします。\n
これは、a.op_xor(b)の省略形です。\n

\section shl 左シフト演算子
\code
a << b
\endcode
aをbビット左シフトします。\n
これは、a.op_shl(b)の省略形です。\n

\section shr 右シフト演算子
\code
a >> b
\endcode
aをbビット右シフトします。\n
これは、a.op_shr(b)の省略形です。\n

\section ushr 論理的右シフト演算子
\code
a >>> b
\endcode
aをbビット論理的右シフトします。\n
これは、a.op_ushr(b)の省略形です。\n
*/

    
/** \page cmp_operator 比較演算子
\section not 単項!演算子
\code
!a
\endcode
真値をfalseに、偽値をtrueに変換します。\n

\section eq ==演算子
\code
a == b
\endcode
aとbが等しいか調べます。\n
これは、a.op_eq(b)の省略形です。\n
    
\section ne !=演算子
\code
a != b
\endcode
==演算子の否定です。\n
これは、!a.op_eq(b)の省略形です。\n

\section lt <演算子
\code
a < b
\endcode
aがb未満か調べます。\n
これは、a.op_lt(b)の省略形です。\n

\section gt >演算子
\code
a > b
\endcode
aよりbが大きい値か調べます。\n
これは、b.op_lt(a)の省略形です。\n

\section ge <=演算子
\code
a <= b
\endcode
aがb以下か調べます。\n
これは、!b.op_lt(a)の省略形です。\n
    
\section le >=演算子
\code
a >= b
\endcode
aがb以上か調べます。\n
これは、!a.op_lt(b)の省略形です。\n

\section eql ===演算子
\code
a === b
\endcode
aとbが同一のオブジェクトであるか調べます。\n
    
\section neq !==演算子
\code
a !== b
\endcode
===演算子の否定です。\n
    
\section is is演算子
\code
a is b
\endcode
aのがbのクラスのインスタンスか調べます。\n
    
\section nis !is演算子
\code
a !is b
\endcode
is演算子の否定です。\n
    
\section in in演算子
\code
a in b
\endcode
bがaを含んでいるか調べます。\n
これはa.op_in(b)の省略系です。
    
\section nin !in演算子
\code
a !in b
\endcode
in演算子の否定です。\n
これは!a.op_in(b)の省略系です。

*/

/** \page call 関数呼び出し演算子
\code
foo(a, b, c)
\endcode
関数呼び出しはfoo.op_call(a, b, c)の省略形です。\n

関数呼び出しは引数が無いとしても、括弧を省略することはできません。 \n
括弧を省略した場合、単なる変数の参照になります。\n

Xtalはキーワード引数も対応しています。
\code
foo(key: "value", key2: 100);
\endcode
*/
    
/** \page method_call メソッド呼び出し演算子
\code
obj.member();
obj.member(100);
\endcode
引数の渡し方は、関数呼び出し演算子と同じです。\n
\n
メソッド呼び出し演算子は、プロパティサポートのため、メソッド呼び出しは引数が無いとき、括弧を省略することができます。\n 
\code
obj.member;
\endcode
プロパティの値設定の方は次のように書けます。\n
\code
obj.member = 100;
\endcode
これは obj.set_member(100); のようにコンパイルされます。\n

メソッド呼び出し式の.を.?とすると、そのメソッドが存在する場合だけ呼び出すという動作になります。 \n
もし存在しなかった場合、式の値はundefinedオブジェクトとなります。\n
\code
obj.?member;
obj.?member(100);
\endcode
    
\section dynamic_method_call 動的メソッド呼び出し
\code
obj.("test" ~ "2");
name: "aaaa";
obj.(name)(100);
ojb.?("key");
\endcode
.の直後に()で囲むことで、識別子の部分を動的な文字列にて指定することができます。\n
*/

/** \page refmember メンバ取得演算子
\code
class Foo{
    roo: 100;
}

Foo::roo.p; //=> 100
\endcode

クラスオブジェクトに定義されているメンバを取り出す演算子です。\n
::? の方を使うと、そのメンバが存在しない場合、undefinedオブジェクトが返る演算子です。\n
\code
if(Foo::?hoge === undefined){
  // Foo::hogeが無いならここを通る
}else{
  // Foo::hogeがあるならここを通る
}
\endcode

\section dynamic_refmeber 動的メンバ取得演算子
\code
class Foo{
    roo: 100;
}

Foo::("ro" ~ "o").p; //=>100
Foo::?("key").p; //=> undefined
\endcode
::の直後に()で囲むことで、識別子の部分を文字列で指定し、取り出すメンバを動的に決定します。\n
*/

/** \page secondary_key セカンダリキー
メンバの定義や取得などに、任意のオブジェクトをセカンダリキーとして使うことができます。\n
\code
class Foo{
    // 識別子hogeをプライマリキー
    // 整数555をセカンダリキーに
    bar#555: 100;
    
    // 識別子hogeをプライマリキー
    // クラスオブジェクトStringをセカンダリキーに
    hoge#String{
        "hoge#String".p;
    }
}

Foo::bar#555.p; //=> 100

f: Foo();
f.hoge#String; //=> hoge#String
\endcode

クラスオブジェクトをセカンダリキーにした場合、そのクラスのサブクラスでも検索がヒットします。\n
\code
class A{}
class B(A){}

class Foo{
    bar#A{
        "bar#A".p;
    }
}

f: Foo();
f.bar#B; //=> "bar#A"
\endcode
*/
    
/** \page etc_operator その他の演算子

\section 要素取得演算子
\code
a[index]
\endcode
indexに対応する要素を返します。\n
これは、a.op_at(index)の省略形です。\n

\section 要素設定演算子
\code
a[index] = value
\endcode
indexに対応する要素を設定します。\n
これは、a.op_set_at(index, value)の省略形です。\n

\section 配列化演算子
\code
a[]
\endcode
aを配列に纏めた結果を返します。\n
これは、a.op_to_arrayの省略形です。\n

\section 連想配列化演算子
\code 
a[:]
\endcode
aを連想配列に纏めた結果を返します。\n
これは、a.op_to_mapの省略形です。\n

\section oror 論理和演算子
\code
a || b
\endcode
aを評価して、それが偽でないなら左辺の値を返します。 偽なら右辺を返します。\n

\section andand 論理積演算子 
\code
a && b
\endcode
aを評価して、それが偽でないなら、右辺を返します。 偽ならそのまま左辺を返します。\n

\section _3term 三項演算子
値を返すif文です。\n
\code
a<10 ? 1: 2
\endcode

\section once once演算子 
once演算子は、式の評価を最初の一度だけする演算子です。\n
\code
fun foo(){
    return once [0, 1, 2, 3];
}
\endcode
もしonceを付けない場合、この関数は呼び出すたびに違う配列オブジェクトを生成して返しますが、 onceで一度しか評価しないとしているため、何度呼び出しても同じ配列オブジェクトを返します。\n

\section range 範囲生成演算子
範囲オブジェクトを生成します。\n
\code
a .. b  // 範囲[expr, expr]
a ..< b  // 範囲[expr, expr)
a <.. b  // 範囲(expr, expr]
a <..< b  // 範囲(expr, expr)
\endcode
これは、a.op_range(b, kind)の省略系です。\n
両辺の式が整数の場合、IntRangeクラスのオブジェクトが、\n
浮動小数点数の場合、FloatRangeクラスのオブジェクトが、\n
長さ1の文字列の場合、ChRangeクラスのオブジェクトが生成されます。\n
*/

/** \page operator_priority 演算子の優先順位
<TABLE>
<TR><TD>優</TD><TD>結</TD><TD>演算子</TD><TD>意味</TD></TR> 
<TR><TD>15</TD><TD>左</TD><TD>a.b</TD><TD>オブジェクトメンバアクセス</TD></TR>  
<TR><TD>15</TD><TD>左</TD><TD>a[b]</TD><TD>配列要素アクセス</TD></TR>  
<TR><TD>15</TD><TD>左</TD><TD>a()</TD><TD>関数呼び出し</TD></TR>  
<TR><TD>15</TD><TD>左</TD><TD>a..b</TD><TD>範囲演算子</TD></TR>  
<TR><TD>15</TD><TD>左</TD><TD>a[]</TD><TD>配列化演算子</TD></TR> 
<TR><TD>15</TD><TD>左</TD><TD>a[:]</TD><TD>連想配列化演算子</TD></TR> 
<TR><TD>14</TD><TD>右</TD><TD>+a</TD><TD></TD></TR>  
<TR><TD>14</TD><TD>右</TD><TD>-a</TD><TD>符号反転</TD></TR>  
<TR><TD>14</TD><TD>右</TD><TD>~a</TD><TD>ビット単位の反転</TD></TR>  
<TR><TD>14</TD><TD>右</TD><TD>!a</TD><TD>論理否定</TD></TR>  
<TR><TD>13</TD><TD>左</TD><TD>a*b</TD><TD>乗算</TD></TR>  
<TR><TD>13</TD><TD>左</TD><TD>a/b</TD><TD>除算</TD></TR>  
<TR><TD>13</TD><TD>左</TD><TD>a%b</TD><TD>余り</TD></TR>  
<TR><TD>12</TD><TD>左</TD><TD>a+b</TD><TD>加算</TD></TR>  
<TR><TD>12</TD><TD>左</TD><TD>a-b</TD><TD>減算</TD></TR>  
<TR><TD>11</TD><TD>左</TD><TD>a<<b</TD><TD>左シフト</TD></TR>  
<TR><TD>11</TD><TD>左</TD><TD>a>>b</TD><TD>符号付右シフト</TD></TR>  
<TR><TD>11</TD><TD>左</TD><TD>a>>>b</TD><TD>符号無右シフト</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a==b</TD><TD>等しい</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a!=b</TD><TD>等しくない</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a===b</TD><TD>同一</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a!==b</TD><TD>同一ではない</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a is b</TD><TD>aはbクラスのインスタンスか</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a !is b</TD><TD>aはbクラスのインスタンスではないか</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a<b</TD><TD>より小さい</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a>b</TD><TD>より大きい</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a<=b</TD><TD>より小さいか等しい</TD></TR>  
<TR><TD>10</TD><TD>左</TD><TD>a>=b</TD><TD>より大きいか等しい</TD></TR>  
<TR><TD>09</TD><TD>左</TD><TD>a&b</TD><TD>ビット単位の論理積</TD></TR>  
<TR><TD>08</TD><TD>左</TD><TD>a^b</TD><TD>ビット単位の排他的論理和</TD></TR>  
<TR><TD>07</TD><TD>左</TD><TD>a|b</TD><TD>ビット単位の論理和</TD></TR>  
<TR><TD>06</TD><TD>左</TD><TD>a&&b</TD><TD>論理積</TD></TR>  
<TR><TD>05</TD><TD>左</TD><TD>a||b</TD><TD>論理和</TD></TR>  
<TR><TD>00</TD><TD>右</TD><TD>once a</TD><TD>最初の一度しか評価しない</TD></TR>  
</TABLE>
    
Xtalは優先順位と空白がマッチしていない場合、コンパイルエラーとします。\n
\code
10 + 5*6 // ok
10+5*6 // ok
10+5 * 6 // error +より*の方が優先順位が高いのに、見た目はそうなっていない。
\endcode

\subsection sppri 特別な優先順位0の演算子 
onceなどの優先順位0の演算子をくっつけて記述した場合、 優先順位が一番高くなる、というルールが適用されます。\n
例えば\n
\code
once(10)+10
\endcode
は\n
\code
((once 10) + 10)
\endcode
となります。\n
\code
once (10)+10
\endcode
と離すと\n
\code
(once (10 + 10))
\endcode
となります。\n
*/
    
    
/** \page except 例外制御文
\section try_catch try catch文
\code
try{
    hoge();
    // ...
}
catch(e){ // try節で例外が発生した場合に実行される節
   // 変数eに例外オブジェクトが代入される
}
\endcode

try節の中で送出された例外を捕まえる構文です。 \n

\section try_finally try finally文
\code
try{

}
finally{

}
\endcode
try節から抜け出したときに必ずfinally節に書かれた節を実行する構文です。\n
これは普通に{}の中を抜け出したときはもちろん、\n
break、continue、return、throwなどで抜け出したときにも実行されます。\n

\section try try catch finally文 
\code
try{

}
catch(e){

}
finally{

}
\endcode
try catch文とtry finally文の複合文です。\n

\section throw throw文
\code
throw "error";
\endcode
例外を送出します。\n
投げられるオブジェクトは、文字列化可能なオブジェクトか、Excetionクラスかそのサブクラスのインスタンスオブジェクトである必要があります。\n
*/
