/** \file src/xtal/xtal_thread.h
* \brief src/xtal/xtal_thread.h
*/

#ifndef XTAL_THREAD_H_INCLUDE_GUARD
#define XTAL_THREAD_H_INCLUDE_GUARD

#pragma once

namespace xtal{

void yield_thread();
void sleep_thread(float_t sec);

/**
* \xbind
* \xinherit lib::builtin::Any
* \brief スレッド
*/
class Thread : public Base{
public:
	/**
	* \brief スレッドオブジェクトを生成する
	*/
	Thread(){
		thread_lib_ = thread_lib();
		impl_ = thread_lib_->new_thread();
	}

	~Thread(){
		thread_lib_->delete_thread(impl_);
	}

public:

	/**
	* \brief スレッドを開始する
	*/
	void start(const AnyPtr& callback){
		callback_ = callback;
		env_ = environment();
		thread_lib_->start_thread(impl_, &trampoline, this);
	}

	/**
	* \brief スレッドが終わるまで待機する
	*/
	void join(){
		XTAL_UNLOCK{
			thread_lib_->join_thread(impl_);
		}
	}

public:

	/**
	* \brief スレッドを切り替える
	*/
	static void yield(){
		yield_thread();
	}
		
	/**
	* \brief カレントスレッドをsec秒眠らせる
	*/
	static void sleep(float_t sec){
		sleep_thread(sec);
	}

public:

	void on_visit_members(Visitor& m){
		Base::on_visit_members(m);
		m & callback_;
	}

protected:

	static void trampoline(void* data){
		((Thread*)data)->begin_thread();
	}

	void begin_thread(){
		register_thread(env_);

		VMachinePtr vm = setup_call(0);
		callback_->rawcall(vm);
		vm->cleanup_call();
		callback_ = null;
		vm->reset();
		vm = null;

		unregister_thread(env_);
	}

protected:

	AnyPtr callback_;
	void* impl_;
	Environment* env_;
	ThreadLib* thread_lib_;
};

/**
* \xbind
* \xinherit lib::builtin::Any
* \brief ミューテックス
*/
class Mutex : public Base{
public:

	Mutex(){
		thread_lib_ = thread_lib();
		impl_ = thread_lib_->new_mutex();
	}

	~Mutex(){
		thread_lib_->delete_mutex(impl_);
	}

	/**
	* \brief ロックする
	*/
	void lock(){
		XTAL_UNLOCK{
			thread_lib_->lock_mutex(impl_);
		}
	}

	/**
	* \brief アンロックする
	*/
	void unlock(){
		thread_lib_->unlock_mutex(impl_);
	}

	void rawlock(){
		thread_lib_->lock_mutex(impl_);
	}

public:

	void* impl(){
		return impl_;
	}

private:
	void* impl_;
	ThreadLib* thread_lib_;
};
}

#endif // XTAL_THREAD_H_INCLUDE_GUARD
