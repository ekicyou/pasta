
#pragma once

namespace xtal{

inline void bind_error_message(){
	const char_t* messages[] = {
		XTAL_L("XCE1001"), XTAL_L("XCE1001:構文エラーです。"),
		XTAL_L("XCE1002"), XTAL_L("XCE1002:'%(required)s'を期待しましたが'%(char)s'が検出されました。"),
		XTAL_L("XCE1003"), XTAL_L("XCE1003:';' がありません。"),
		XTAL_L("XCE1004"), XTAL_L("XCE1004:不正な関数の仮引数です。"),
		XTAL_L("XCE1005"), XTAL_L("XCE1005:非名前付き引数が名前付き引数の後にあります。"),
		XTAL_L("XCE1006"), XTAL_L("XCE1006:不正なbreak文、またはcontinue文です。"),
		XTAL_L("XCE1008"), XTAL_L("XCE1008:不正な多重代入文です。"),
		XTAL_L("XCE1009"), XTAL_L("XCE1009:定義されていない変数 '%(name)s' に代入しようとしました 。"),
		XTAL_L("XCE1010"), XTAL_L("XCE1010:不正な数字リテラルのサフィックスです。"),
		XTAL_L("XCE1011"), XTAL_L("XCE1011:文字列リテラルの途中でファイルが終わりました。"),
		XTAL_L("XCE1012"), XTAL_L("XCE1012:不正な代入文の左辺です。"),
		XTAL_L("XCE1013"), XTAL_L("XCE1013:比較演算式の結果を演算しようとしています。"),
		XTAL_L("XCE1014"), XTAL_L("XCE1014:不正な浮動小数点数リテラルです。"),
		XTAL_L("XCE1015"), XTAL_L("XCE1015:不正な%(n)d進数値リテラルのサフィックスです。"),
		XTAL_L("XCE1016"), XTAL_L("XCE1016:assert文の引数の数が不正です。"),
		XTAL_L("XCE1017"), XTAL_L("XCE1017:不正な%%記法リテラルです。"),
		XTAL_L("XCE1018"), XTAL_L("XCE1018:default節が重複定義されました。"),
		XTAL_L("XCE1019"), XTAL_L("XCE1019:'%(name)s'は代入不可能です。"),
		XTAL_L("XCE1021"), XTAL_L("XCE1021:コメントの途中でファイルが終わりました。"),
		XTAL_L("XCE1022"), XTAL_L("XCE1022:関数から返せる多値の最大は255個です。"),
		XTAL_L("XCE1023"), XTAL_L("XCE1023:定義されていないインスタンス変数名 '%(name)s' を参照しています。"),
		XTAL_L("XCE1024"), XTAL_L("XCE1024:同名のインスタンス変数名 '%(name)s' が既に定義されています。"),
		XTAL_L("XCE1025"), XTAL_L("XCE1025:比較演算式の結果を再比較しようとしています。"),
		XTAL_L("XCE1026"), XTAL_L("XCE1026:同じスコープ内で、同じ変数名 '%(name)s' が既に定義されています。"),
		XTAL_L("XCE1027"), XTAL_L("XCE1027:コードが大きすぎて、バイトコードの生成に失敗しました。"),
		XTAL_L("XCE1028"), XTAL_L("XCE1028:演算子の前後の空白と演算子の優先順位が一致していません。想定している優先順位と異なっている可能性があります。"),
		XTAL_L("XCE1029"), XTAL_L("XCE1029:a&1というように、bitwise andを条件式とすることは安全のため禁止されています。(a&1)!=0という形で比較してください。"),
		XTAL_L("XCE1030"), XTAL_L("XCE1012:不正な定義文の左辺です。"),
		
		XTAL_L("XRE1001"), XTAL_L("XRE1001:'%(object)s' 関数呼び出しの '%(no)s'番目の引数の型が不正です。'%(required)s'型を要求していますが、'%(type)s'型の値が渡されました。"),
		XTAL_L("XRE1002"), XTAL_L("XRE1002:ソースのコンパイル中、コンパイルエラーが発生しました。"),
		XTAL_L("XRE1003"), XTAL_L("XRE1003:不正なインスタンス変数の参照です。"),
		XTAL_L("XRE1004"), XTAL_L("XRE1004:型エラーです。 '%(required)s'型を要求していますが、'%(type)s'型の値が渡されました。"),
		XTAL_L("XRE1005"), XTAL_L("XRE1005:'%(object)s' 関数呼び出しの引数の数が不正です。'%(min)s'以上の引数を受け取る関数に、%(value)s個の引数を渡しました。"),
		XTAL_L("XRE1006"), XTAL_L("XRE1006:'%(object)s' 関数呼び出しの引数の数が不正です。'%(min)s'以上、'%(max)s'以下の引数を受け取る関数に、'%(value)s'個の引数を渡しました。"),
		XTAL_L("XRE1007"), XTAL_L("XRE1007:'%(object)s' 関数呼び出しの引数の数が不正です。引数を取らない関数に、'%(value)s'個の引数を渡しました。"),
		XTAL_L("XRE1008"), XTAL_L("XRE1008:'%(object)s'はシリアライズできません。"),
		XTAL_L("XRE1009"), XTAL_L("XRE1009:不正なコンパイル済みXtalファイルです。"),
		XTAL_L("XRE1010"), XTAL_L("XRE1010:コンパイルエラーが発生しました。"),
		XTAL_L("XRE1011"), XTAL_L("XRE1011:%(object)s :: '%(name)s' は既に定義されています。"),
		XTAL_L("XRE1012"), XTAL_L("XRE1012:yieldがfiberの非実行中に実行されました。"),
		XTAL_L("XRE1013"), XTAL_L("XRE1013:%(object)s にコンストラクタが登録されていないため、インスタンスを生成できません。"),
		XTAL_L("XRE1014"), XTAL_L("XRE1014:ファイル '%(name)s' を開けません。"),
		XTAL_L("XRE1015"), XTAL_L("XRE1015:%(object)s は定義されていません。"),
		XTAL_L("XRE1016"), XTAL_L("XRE1016:ファイル '%(name)s' のコンパイル中、コンパイルエラーが発生しました。"),
		XTAL_L("XRE1017"), XTAL_L("XRE1017:%(object)s :: %(primary_key)s # %(secondary_key)sは %(accessibility)s です。"),
		XTAL_L("XRE1018"), XTAL_L("XRE1018:既に閉じられたストリームです。"),
		XTAL_L("XRE1019"), XTAL_L("XRE1019:C++で定義されたクラスの多重継承は出来ません。"),
		XTAL_L("XRE1020"), XTAL_L("XRE1020:配列の範囲外アクセスです。"),
		XTAL_L("XRE1021"), XTAL_L("XRE1021:%(object)s は定義されていません。'%(pick)s'と間違えている可能性があります。"),
		XTAL_L("XRE1023"), XTAL_L("XRE1023:1より長い文字列は範囲演算子に指定できません。"),
		XTAL_L("XRE1024"), XTAL_L("XRE1024:0除算エラーです。"),
		XTAL_L("XRE1025"), XTAL_L("XRE1025:ChRangeは閉区間である必要があります。"),
		XTAL_L("XRE1026"), XTAL_L("XRE1026:xpeg要素に変換できません。"),
		XTAL_L("XRE1027"), XTAL_L("XRE1027:cap関数の引数が不正です。cap(name: value)というように名前付き引数にするか、cap(\"name\"), value)というように呼んでください。"),
		XTAL_L("XRE1028"), XTAL_L("XRE1028:finalマークが付けられたクラス'%(name)s'を継承しようとしました。"),
		XTAL_L("XRE1029"), XTAL_L("XRE1029:C++で定義されたクラス'%(name)s'は、クラス生成時のみ継承が可能です。"),
		XTAL_L("XRE1030"), XTAL_L("XRE1030:暗黙の変数参照があります。%(name)s"),
		XTAL_L("XRE1031"), XTAL_L("XRE1031:シングルトンクラスはシングルトンクラスでないと継承できません。"),
		XTAL_L("XRE1032"), XTAL_L("XRE1032:ファイル'%(name)s'が開けません。"),
		XTAL_L("XRE1033"), XTAL_L("XRE1033:ストリームの終端以降を読み取ろうとしました。"),
		XTAL_L("XRE1034"), XTAL_L("XRE1034:無限ループが発生する可能性があるxpeg要素を実行しようとしました"),
		XTAL_L("XRE1035"), XTAL_L("XRE1035:実行中で非yield中のファイバーに対する不正な操作です"),	
		XTAL_L("XRE1036"), XTAL_L("XRE1036:'%(object)s' 関数呼び出しの引数の名前が不正です。関数側で必要とされていない名前付き引数'%(name)s'が渡されました"),	
	};
	
	for(unsigned int i=0; i<sizeof(messages)/sizeof(*messages)/2; ++i){
		IDPtr key(*(LongLivedString*)messages[i*2+0]);
		text_map()->set_at(key, *(LongLivedString*)messages[i*2+1]);
	}
}
	
}
