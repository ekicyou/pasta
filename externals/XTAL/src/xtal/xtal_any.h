/** \file src/xtal/xtal_any.h
* \brief src/xtal/xtal_any.h
*/

#ifndef XTAL_ANY_H_INCLUDE_GUARD
#define XTAL_ANY_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \brief すべてのクラスの基底クラス
*/
class Any{
public:

	/**
	* \brief 関数オブジェクトとみなし、関数呼び出しをする。
	* 引数や戻り値はvmを通してやり取りする。
	*/
	void rawcall(const VMachinePtr& vm) const;
	
	/**
	* \brief メソッド呼び出しをする
	* 引数や戻り値はvmを通してやり取りする。
	* \param vm 仮想マシン
	* \param primary_key メンバ名
	* \param secondary_key セカンダリキー
	* \param inherited_too 継承元クラスからもメソッド検索をするかどうか
	* \param q もし見つからないなら例外を設定するならtrue
	*/
	void rawsend(const VMachinePtr& vm, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, bool q) const;
	void rawsend(const VMachinePtr& vm, const IDPtr& primary_key) const;
	void rawsend(const VMachinePtr& vm, const IDPtr& primary_key, const AnyPtr& secondary_key) const;
	void rawsend(const VMachinePtr& vm, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too) const;

	/**
	* \brief メンバを取得する。
	* \param primary_key メンバ名
	* \param secondary_key セカンダリキー
	* \param inherited_too 継承元クラスからもメソッド検索をするかどうか
	* \retval undefined そのメンバは存在しない
	* \retval 非undefined nameに対応したメンバ  
	*/
	const AnyPtr& member(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility) const;
	const AnyPtr& member(const IDPtr& primary_key) const;
	const AnyPtr& member(const IDPtr& primary_key, const AnyPtr& secondary_ke) const;
	const AnyPtr& member(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too) const;

	/**
	* \brief メンバを定義する。
	* \param primary_key メンバ名
	* \param secondary_key セカンダリキー
	*/
	void def(const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility) const;
	void def(const IDPtr& primary_key, const AnyPtr& value) const;
	void def(const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key) const;

	/**
	* \xbind
	* \brief このオブジェクトが所属するクラスを返す。
	*/
	const ClassPtr& get_class() const;

	/**
	* \brief 整数に変換して返す。
	*/
	int_t to_i() const;
	
	/**
	* \brief 浮動小数点数に変換して返す。
	*/
	float_t to_f() const;
	
	/**
	* \brief 文字列に変換して返す。
	*/
	StringPtr to_s() const;

	/**
	* \brief 配列に変換して返す。
	*/
	ArrayPtr to_a() const;

	/**
	* \brief 連想配列に変換して返す。
	*/
	MapPtr to_m() const;

	/**
	* \brief klassクラスのインスタンスか調べる。
	*/
	bool is(const AnyPtr& klass) const;

	bool is(CppClassSymbolData* key) const;
		
	/**
	* \brief klassクラスを継承しているか調べる
	*/
	bool is_inherited(const AnyPtr& klass) const;
	
	/**
	* \brief このオブジェクトがメンバとなっている親のクラスを返す。
	*/
	const ClassPtr& object_parent() const;
	
	/**
	* \brief このオブジェクトに親を設定する。
	* 親を持てないオブジェクトや、前に付けられた親の方が強い場合無視される。
	* \param parent 親
	*/
	void set_object_parent(const ClassPtr& parent) const;

	/**
	* \xbind
	* \brief オブジェクトの名前を返す。
	*/
	StringPtr object_name() const;

	/**
	* \xbind
	* \brief オブジェクトの名前のリストを返す
	* 一番最上位の親からの名前のリストを返す
	*/
	ArrayPtr object_name_list() const;

	/**
	* \xbind
	* \brief 自身を文字列化してprintlnする。
	* 文字列化にはto_sメソッドが呼ばれる。
	* \return 自身を返す。
	*/
	const AnyPtr& p() const;

public:

	MapPtr s_save() const;

	void s_load(const MapPtr& v) const;

	AnyPtr save_instance_variables(const ClassPtr& cls) const;

	void load_instance_variables(const ClassPtr& cls, const AnyPtr& v) const;

	void visit_members(Visitor& m) const;

	StringPtr ask_object_name_to_parent() const;

	InstanceVariables* instance_variables() const;

	bool op_eq( const AnyPtr& v ) const;

public:

	/// \brief primary_keyメソッドを呼び出す
	AnyPtr send(const IDPtr& primary_key) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key) const;

	/// \brief 関数を呼び出す
	AnyPtr call() const;

//{REPEAT{{
/*
	/// \brief primary_keyメソッドを呼び出す
	template<class A0 #COMMA_REPEAT#class A`i+1`#>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  #COMMA_REPEAT#const A`i+1`& a`i+1`#) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 #COMMA_REPEAT#class A`i+1`#>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  #COMMA_REPEAT#const A`i+1`& a`i+1`#) const;

	/// \brief 関数を呼び出す
	template<class A0 #COMMA_REPEAT#class A`i+1`#>
	AnyPtr call(const A0& a0  #COMMA_REPEAT#const A`i+1`& a`i+1`#) const;
*/

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 >
	AnyPtr send(const IDPtr& primary_key, const A0& a0  ) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 >
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  ) const;

	/// \brief 関数を呼び出す
	template<class A0 >
	AnyPtr call(const A0& a0  ) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1>
	AnyPtr call(const A0& a0  , const A1& a1) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12) const;

	/// \brief primary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, class A13>
	AnyPtr send(const IDPtr& primary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12, const A13& a13) const;

	/// \brief primary_key#secondary_keyメソッドを呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, class A13>
	AnyPtr send2(const IDPtr& primary_key, const AnyPtr& secondary_key, const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12, const A13& a13) const;

	/// \brief 関数を呼び出す
	template<class A0 , class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, class A13>
	AnyPtr call(const A0& a0  , const A1& a1, const A2& a2, const A3& a3, const A4& a4, const A5& a5, const A6& a6, const A7& a7, const A8& a8, const A9& a9, const A10& a10, const A11& a11, const A12& a12, const A13& a13) const;

//}}REPEAT}

private:

	// call(VMachinePtr)がコンパイルエラーとなるように
	struct cmpitle_error{ cmpitle_error(const VMachinePtr& vm); };
	void call(cmpitle_error) const;

private:

	StringPtr defined_place_name(const CodePtr& code, int_t pc, int_t name_number) const;

protected:

	const ClassPtr& get_class_except_base() const;

	struct noinit_t{};

	void init(RefCountingBase* p);

public: // 性能確保のためpublicにするが、基本的に直接触ることはよろしくない

	enum{
		USER_FLAG_SHIFT = TYPE_SHIFT+1,

		STRING_SIZE_SHIFT = USER_FLAG_SHIFT,
		STRING_SIZE_MASK = ~((1<<STRING_SIZE_SHIFT)-1),

		CPP_CLASS_INDEX_SHIFT = USER_FLAG_SHIFT,
		CPP_CLASS_INDEX_MASK = ~((1<<CPP_CLASS_INDEX_SHIFT)-1)
	};

	struct AnyRawValue{
	public:

		void init_primitive(PrimitiveType t){ type = t; ivalue = 0; }

		void init_null(){ type = 0; ivalue = 0; }

		void init_int(char v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(signed char v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(unsigned char v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(short v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(unsigned short v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(int v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(unsigned int v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(long v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(unsigned long v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(long long v){ type = TYPE_INT; ivalue = (int_t)v; }
		void init_int(unsigned long long v){ type = TYPE_INT; ivalue = (int_t)v; }

		void init_float(float v){ type = TYPE_FLOAT; fvalue = (float_t)v; }
		void init_float(double v){ type = TYPE_FLOAT; fvalue = (float_t)v; }
		void init_float(long double v){ type = TYPE_FLOAT; fvalue = (float_t)v; }

		void init_bool(bool b){ type = TYPE_FALSE + (int)b; ivalue = 0; }

		void init_base(const Base* v){  
			type = TYPE_BASE; 
			pvalue = const_cast<Base*>(v); 
		}

		void init_rcbase(int t, const RefCountingBase* v){ 
			type = t; 
			rcpvalue = const_cast<RefCountingBase*>(v); 
		}

		void init_long_lived_string(const char_t* str, uint_t size){ 
			type = TYPE_LONG_LIVED_STRING | (size<<STRING_SIZE_SHIFT); 
			spvalue = str; 
		}

		void init_small_string(uint_t size){ 
			type = TYPE_SMALL_STRING | (size<<STRING_SIZE_SHIFT);
			ivalue = 0;	
		}

		void init_small_string(const char_t* str, uint_t size){ 
			type = TYPE_SMALL_STRING | (size<<STRING_SIZE_SHIFT);
			ivalue = 0;
			for(uint_t i=0; i<size; ++i){
				svalue[i] = str[i];
			}
		}

		void init_interned_string(const char_t* str, uint_t size){ 
			type = TYPE_INTERNED_STRING | (size<<STRING_SIZE_SHIFT);
			spvalue = str; 
		}

		void init_pointer(const void* v, uint_t i){ 
			type = TYPE_POINTER | (i<<CPP_CLASS_INDEX_SHIFT); 
			vpvalue = const_cast<void*>(v);
		}

		void init_stateless_native_method(const param_types_holder_n* v){  
			type = TYPE_STATELESS_NATIVE_METHOD; 
			pthvalue = v; 
		}

		void init_immediate_value(int_t value1, int_t value2){
			type = TYPE_IMMEDIATE_VALUE | (value1<<8);		
			ivalue = value2;
		}

		void init_immediate_value(int_t value1, float_t value2){
			type = TYPE_IMMEDIATE_VALUE | (value1<<8);		
			fvalue = value2;
		}

		void init_immediate_value(int_t value1, void* value2){
			type = TYPE_IMMEDIATE_VALUE | (value1<<8);		
			vpvalue = value2;
		}

		void init_instance_variable_getter(int_t number, void* info){
			type = TYPE_IVAR_GETTER | (number<<8);		
			vpvalue = info;
		}

		void init_instance_variable_setter(int_t number, void* info){
			type = TYPE_IVAR_SETTER | (number<<8);		
			vpvalue = info;
		}

	public:
		uint_t string_size() const{ return (type & STRING_SIZE_MASK)>>STRING_SIZE_SHIFT; }
		uint_t cpp_class_index() const{ return (type & CPP_CLASS_INDEX_MASK)>>CPP_CLASS_INDEX_SHIFT; }

		int_t immediate_first_value() const{ return (int_t)(type>>8); }
		int_t immediate_second_ivalue() const{ return ivalue; }
		float_t immediate_second_fvalue() const{ return fvalue; }
		void* immediate_second_vpvalue() const{ return vpvalue; }

	public: // 性能確保のためpublicにするが、基本的に直接触るのはよろしくない

		union{
			int_t type;
			uint_t utype;
		};

		union{
			int_t ivalue;
			uint_t uvalue;
			float_t fvalue;
			Base* pvalue;
			RefCountingBase* rcpvalue;
			const char_t* spvalue;
			char_t svalue[SMALL_STRING_MAX];
			void* vpvalue;
			const param_types_holder_n* pthvalue;
		};
	};

	AnyRawValue value_;
};

//////////////////////////////////////////////////////

#define XTAL_detail_rawvalue(v) ((v).value_)
#define XTAL_detail_rawtype(v) (XTAL_detail_rawvalue(v).type)
#define XTAL_detail_urawtype(v) (XTAL_detail_rawvalue(v).utype)
#define XTAL_detail_user_flags(v) (XTAL_detail_rawvalue(v).utype)

#define XTAL_detail_type(v) (XTAL_detail_urawtype(v) & ::xtal::TYPE_MASK)
#define XTAL_detail_ivalue(v) (XTAL_detail_rawvalue(v).ivalue)
#define XTAL_detail_uvalue(v) (XTAL_detail_rawvalue(v).uvalue)
#define XTAL_detail_fvalue(v) (XTAL_detail_rawvalue(v).fvalue)
#define XTAL_detail_chvalue(v) (XTAL_detail_rawvalue(v).svalue[0])
#define XTAL_detail_pvalue(v) (XTAL_detail_rawvalue(v).pvalue)
#define XTAL_detail_rcpvalue(v) (XTAL_detail_rawvalue(v).rcpvalue)
#define XTAL_detail_vpvalue(v) (XTAL_detail_rawvalue(v).vpvalue)
#define XTAL_detail_pthvalue(v) (XTAL_detail_rawvalue(v).pthvalue)
#define XTAL_detail_svalue(v) (XTAL_detail_rawvalue(v).svalue)
#define XTAL_detail_spvalue(v) (XTAL_detail_rawvalue(v).spvalue)
#define XTAL_detail_ssize(v) (XTAL_detail_rawvalue(v).string_size())

#define XTAL_detail_is_undefined(v) (XTAL_detail_urawtype(v)==::xtal::TYPE_UNDEFINED)
#define XTAL_detail_is_null(v) (XTAL_detail_urawtype(v)==::xtal::TYPE_NULL)
#define XTAL_detail_is_true(v) (XTAL_detail_urawtype(v)>::xtal::TYPE_FALSE)
#define XTAL_detail_is_ivalue(v) (XTAL_detail_urawtype(v)==::xtal::TYPE_INT)
#define XTAL_detail_is_fvalue(v) (XTAL_detail_urawtype(v)==::xtal::TYPE_FLOAT)
#define XTAL_detail_is_pvalue(v) (XTAL_detail_type(v)==::xtal::TYPE_BASE)
//#define XTAL_detail_is_rcpvalue(v) (XTAL_detail_type(v)>=::xtal::TYPE_BASE)
#define XTAL_detail_is_rcpvalue(v) (XTAL_detail_rawtype(v)&::xtal::TYPE_BASE)

inline void XTAL_detail_copy(Any& a, const Any& b){ (XTAL_detail_rawvalue(a) = XTAL_detail_rawvalue(b)); }
inline void XTAL_detail_swap(Any& a, Any& b){ (std::swap(XTAL_detail_rawvalue(a), XTAL_detail_rawvalue(b))); }

inline uint_t XTAL_detail_rawbitxor(const Any& a, const Any& b){ return (((XTAL_detail_urawtype(a)^XTAL_detail_urawtype(b))&TYPE_MASK) | (XTAL_detail_ivalue(a)^XTAL_detail_ivalue(b))); }
inline uint_t XTAL_detail_raweq(const Any& a, const Any& b){ return (XTAL_detail_rawbitxor(a, b)==0); }
inline uint_t XTAL_detail_rawhash(const Any& v){ return (XTAL_detail_uvalue(v) ^ XTAL_detail_type(v) ^ (XTAL_detail_uvalue(v)>>3)); }

//////////////////////////////////////////////////////

class AnyArg : public Any{
public:
	template<int N> AnyArg(const LongLivedStringN<N>& str){ value_.init_long_lived_string(str.str(), str.size()); }
	AnyArg(const LongLivedString& str){ value_.init_long_lived_string(str.str(), str.size()); }
	AnyArg(bool v){ value_.init_bool(v); }
	AnyArg(char v){ value_.init_int(v); }
	AnyArg(signed char v){ value_.init_int(v); }
	AnyArg(unsigned char v){ value_.init_int(v); }
	AnyArg(short v){ value_.init_int(v); }
	AnyArg(unsigned short v){ value_.init_int(v); }
	AnyArg(int v){ value_.init_int(v); }
	AnyArg(unsigned int v){ value_.init_int(v); }
	AnyArg(long v){ value_.init_int(v); }
	AnyArg(unsigned long v){ value_.init_int(v); }
	AnyArg(long long v){ value_.init_int(v); }
	AnyArg(unsigned long long v){ value_.init_int(v); }
	AnyArg(float v){ value_.init_float(v); }
	AnyArg(double v){ value_.init_float(v); }
	AnyArg(long double v){ value_.init_float(v); }
	AnyArg(const Any& v){ value_ = v.value_; }
public:
	operator const AnyPtr&() const{
		return *reinterpret_cast<const AnyPtr*>(this);
	}
};

/////////////////////////////////////////////////////////

}

#endif // XTAL_ANY_H_INCLUDE_GUARD
