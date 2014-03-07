/** \file src/xtal/xtal_threadspace.h
* \brief src/xtal/xtal_threadspace.h
*/

#ifndef XTAL_THREADSPACE_H_INCLUDE_GUARD
#define XTAL_THREADSPACE_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class ThreadSpace{
public:
	
	ThreadSpace(){}

	void initialize(ThreadLib* lib){
		thread_lib_ = lib;

		thread_enabled_ = false;
		registered_thread_ = false;
		thread_count_ = 1;

		mutex_ = XNew<Mutex>(); 

		thread_enabled_ = true;

		register_vmachine();
		xlock();
	}

	void uninitialize(){
		join_all_threads();

		unregister_vmachine();
		xunlock();
		
		thread_enabled_ = false;
		thread_lib_ = 0;
		mutex_ = null;
	}

	void join_all_threads(){
		while(thread_count_!=1){
			XTAL_UNLOCK{
				thread_lib_->yield();
			}
		}
	}

	void register_vmachine(){
		set_vmachine(vmachine_take_over());
	}

	void unregister_vmachine(){
		vmachine_take_back(set_vmachine(nul<VMachine>()));
	}

	void xlock(){
		if(!thread_enabled_){
			return;
		}

		mutex_->rawlock();
	}

	void xunlock(){
		if(!thread_enabled_){
			return;
		}

		mutex_->unlock();
	}

	void register_thread(Environment* environment){
		// まずロックを獲得する
		thread_lib_->lock_mutex(mutex_->impl());

		set_environment(environment); // 環境へのポインタをスレッドローカル変数にコピーする

		registered_thread_ = true;
		thread_count_++;
		register_vmachine();
	}

	void unregister_thread(Environment* environment){
		unregister_vmachine();
		thread_count_--;
		xunlock();
	}

	void yield_thread(){
		if(thread_count_==1 && !registered_thread_){
			return;
		}

		XTAL_UNLOCK{
			thread_lib_->yield();
		}
	}

	void sleep_thread(float_t sec){
		XTAL_UNLOCK{
			thread_lib_->sleep(sec);
		}
	}

	bool thread_enabled(){
		return thread_enabled_;
	}

private:
	int thread_count_;

	MutexPtr mutex_;

	bool thread_enabled_;
	bool registered_thread_;

	ThreadLib* thread_lib_;
};

}

#endif // XTAL_THREADSPACE_H_INCLUDE_GUARD
