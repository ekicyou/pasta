/** \file src/xtal/xtal_iterator.h
* \brief src/xtal/xtal_iterator.h
*/

#ifndef XTAL_ITERATOR_H_INCLUDE_GUARD
#define XTAL_ITERATOR_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class DelegateToIterator : public Base{
	IDPtr member_;
public:
	
	DelegateToIterator(const IDPtr& name)
		:member_(name){}

	void on_rawcall(const VMachinePtr& vm);
};

/**
* \xbind lib::builtin
* \brief イテレータの便利メソッドを提供するためのクラス
*/
class Iterator{
public:

	//void scan(pattern);

	/**
	* \xbind
	* \brief 要素をn個デバッグプリントする
	* 代わりのイテレータを返す。
	*/
	AnyPtr ip(int_t n = 3);

	/**
	* \xbind
	* \brief 要素を纏めて配列にして返す
	*/
	ArrayPtr op_to_array();

	/**
	* \xbind
	* \brief 要素を纏めて連想配列にして返す
	* 二つ以上の値を列挙するイテレータである必要がある
	*/
	MapPtr op_to_map();

	/**
	* \xbind
	* \brief 要素を文字列化し、さらにそれらをsepで連結した文字列を返す
	* \code
	* [1, 2, 3].join("::").p;
	* //=> 1::2::3
	* \endcode
	*/
	StringPtr join(const StringPtr& sep = XTAL_STRING(""));

	/**
	* \xbind
	* \brief (index, value)を返すイテレータを返す
	* ループごとにindexはインクリメントされる。
	* \code
	* [45, 12, 33].with_index{ |index, value|
	*     %f(%d-%d)(index, value).p;
	* }
	* //=> 0-45
	* //=> 1-12
	* //=> 2-33
	* \endcode
	*/
	AnyPtr with_index(int_t start = 0);

	/**
	* \xbind
	* \brief 要素をconv関数で変換した要素を返すイテレータを返す
	* mapメソッドと同じ意味である
	*/
	AnyPtr collect(const AnyPtr& conv);

	/**
	* \xbind
	* \brief 要素をconv関数で変換した要素を返すイテレータを返す
	* \code
	* [10, 20, 30].map(|x| x*2){
    *	it.p;
	* }
	* //=> 20
	* //=> 40
	* //=> 60
	* \endcode
	*/
	AnyPtr map(const AnyPtr& conv);

	/**
	* \xbind
	* \brief 選別された要素を返すイテレータを返す
	* filterメソッドと同一の意味である
	*/
	AnyPtr select(const AnyPtr& pred);

	/**
	* \xbind
	* \brief 選別された要素を返すイテレータを返す
	* \code
	* [1, 2, 3].filter(|x| x%2==1){
	*	it.p;
	* }
	* //=> 1
	* //=> 3
	* \endcode
	*/
	AnyPtr filter(const AnyPtr& pred);

	/**
	* \xbind
	* \brief pred関数がtrueを返すまでイテレートするイテレータを返す
	* \code
	* [7, 5, 3, 2, 1, 3].break_if(|x| x%2!=1){
	*     it.p;
	* }
	* //=> 7
	* //=> 5
	* //=> 3
	* \endcode
	*/
	AnyPtr break_if(const AnyPtr& pred);

	/**
	* \xbind
	* \brief times回イテレートするイテレータを返す
	* \code
	* [5, 3, 7].cycle.take(5).join(",").p;
	* //=> 5,3,7,5,3
	* \endcode
	*/
	AnyPtr take(int_t times);

	/**
	* \xbind
	* \brief 並列にイテレートするイテレータを返す
	*/
	AnyPtr zip(...);

	/**
	* \xbind
	* \brief 直列にイテレートするイテレータを返す
	*/
	AnyPtr chain(...);

	/**
	* \xbind
	* \brief イテレートする要素がなくなったら、最初から繰り返すイテレータを返す
	* \code
	* [4, 8, 11].map(|x| x*3).cycle{
	*    it.p;
	* }
	* //=> 12
	* //=> 24
	* //=> 33
	* //=> 12
	* //=> 24
	* //=> 33
	* // 以下繰り返し
	* \endcode
	*/
	AnyPtr cycle();

	/**
	* \xbind
	* \brief イテレートする要素の最大の値を返す
	* \param pred 同一要素か判定する述語関数オブジェクト
	* 二つの引数を受け取り、一つ目より二つ目の方が大きいならtrueを返す関数を渡すこと。
	* nullを指定すると < 演算子で判定される。
	*/
	AnyPtr max_element(const AnyPtr& pred = null);

	/**
	* \xbind
	* \brief イテレートする要素の最小の値を返す
	* \param pred 同一要素か判定する述語関数オブジェクト
	* 二つの引数を受け取り、一つ目より二つ目の方が小さいならtrueを返す関数を渡すこと。
	* nullを指定すると > 演算子で判定される。
	*/
	AnyPtr min_element(const AnyPtr& pred = null);

	/**
	* \xbind
	* \brief イテレートする要素のpred関数がtrueを返す関数を返す
	* \param pred 探している要素かを判定する述語関数オブジェクト
	* 一つの引数を受け取り、探している要素ならtrueを返す関数を渡すこと。
	*/
	AnyPtr find(const AnyPtr& pred);

	/**
	* \xbind
	* \brief inject
	*/
	AnyPtr inject(const AnyPtr& init, const AnyPtr& fn);

	/**
	* \xbind
	* \brief (前回の要素, 今回の要素)を要素をするイテレータを返す
	* 最初の一回の前回の要素にはundefinedが入る
	*/
	AnyPtr with_prev();

};

/**
* \xbind lib::builtin
* \brief イテレート可能なコンテナのための実装を提供するためのクラス。
* これをinheritしたクラスのオブジェクトは、eachメソッドを経由してlib::builtin::Iteratorが持つメソッドを呼び出せる。
*/
class Iterable{};

class ZipIter : public Base{
public:

	ZipIter(const VMachinePtr& vm);

	void common(const VMachinePtr& vm, const IDPtr& id);

	void block_first(const VMachinePtr& vm);
	
	void block_next(const VMachinePtr& vm);

	void block_break(const VMachinePtr& vm);

	void on_visit_members(Visitor& m);

	ArrayPtr next_;
};

struct BlockValueHolder1{
	
	BlockValueHolder1(const AnyPtr& tar, bool& not_end);
	BlockValueHolder1(const ArrayPtr& tar, bool& not_end);
	~BlockValueHolder1();

	AnyPtr target;
	AnyPtr values[1];

	SmartPtr<ArrayIter> it;
};

struct BlockValueHolder2{
	
	BlockValueHolder2(const AnyPtr& tar, bool& not_end);
	BlockValueHolder2(const MapPtr& tar, bool& not_end);
	~BlockValueHolder2();

	AnyPtr target;
	AnyPtr values[2];

	SmartPtr<MapIter> it;
};

struct BlockValueHolder3{
	BlockValueHolder3(const AnyPtr& tar, bool& not_end);
	~BlockValueHolder3();

	AnyPtr target;
	AnyPtr values[3];
};

bool block_next(BlockValueHolder1& holder, bool first);
bool block_next(BlockValueHolder2& holder, bool first);
bool block_next(BlockValueHolder3& holder, bool first);
void block_break(AnyPtr& target);

}

#endif // XTAL_ITERATOR_H_INCLUDE_GUARD
