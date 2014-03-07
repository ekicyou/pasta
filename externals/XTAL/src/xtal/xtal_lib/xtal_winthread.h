
#pragma once

#ifndef _MT
#	define _MT
#endif

#include <windows.h>
#include <process.h>

namespace xtal{

class WinMutex{
	CRITICAL_SECTION sect_;
public:
	WinMutex(){
		InitializeCriticalSection(&sect_);
	}
	
	~WinMutex(){
		DeleteCriticalSection(&sect_);
	}
	
	void lock(){
		/*for(int i=0; i<1000; ++i){
			if(TryEnterCriticalSection(&sect_)){
				return;
			}
			Sleep(1);
		}*/

		EnterCriticalSection(&sect_);
	}
	
	void unlock(){
		LeaveCriticalSection(&sect_);
	}
};

class WinEvent{
	HANDLE handle_;
public:
	WinEvent(){
		handle_ = CreateEvent(NULL, FALSE, FALSE, NULL);
	}
	
	~WinEvent(){
		CloseHandle(handle_);
	}
	
	void wait(){
		WaitForSingleObject(handle_, INFINITE);
	}
	
	void signal(){
		SetEvent(handle_); 
	}
};

class WinThread{
	HANDLE id_;
	void (*callback_)(void*);
	void* data_;

	static unsigned int WINAPI entry(void* self){
		((WinThread*)self)->begin_thread();
		return 0;
	}
	
public:

	WinThread(){
		id_ = (HANDLE)-1;
	}

	~WinThread(){
		if(id_!=(HANDLE)-1){
			CloseHandle(id_);
		}
	}

	void start(void (*callback)(void*), void* data){
		callback_ = callback;
		data_ = data;
		id_ = (HANDLE)_beginthreadex(0, 0, &entry, this, 0, 0);
	}

	void join(){
		WaitForSingleObject(id_, INFINITE);
	}

	void begin_thread(){
		callback_(data_);
	}
};

class WinThreadLib : public ThreadLib{
public:

	virtual void* new_thread(){
		void* p = xmalloc(sizeof(WinThread));
		return new(p) WinThread();
	}

	virtual void delete_thread(void* thread_object){
		((WinThread*)thread_object)->~WinThread();
		xfree(thread_object, sizeof(WinThread));
	}

	virtual void start_thread(void* thread_object, void (*callback)(void*), void* data){
		((WinThread*)thread_object)->start(callback, data);
	}

	virtual void join_thread(void* thread_object){
		((WinThread*)thread_object)->join();
	}

	virtual void* new_mutex(){
		void* p = xmalloc(sizeof(WinMutex));
		return new(p) WinMutex();
	}

	virtual void delete_mutex(void* mutex_object){
		((WinMutex*)mutex_object)->~WinMutex();
		xfree(mutex_object, sizeof(WinMutex));
	}

	virtual void lock_mutex(void* mutex_object){
		((WinMutex*)mutex_object)->lock();
	}

	virtual void unlock_mutex(void* mutex_object){
		((WinMutex*)mutex_object)->unlock();
	}

	virtual void* new_event(){ 
		void* p = xmalloc(sizeof(WinEvent));
		return new(p) WinEvent();
	}

	virtual void delete_event(void* event_object){
		((WinEvent*)event_object)->~WinEvent();
		xfree(event_object, sizeof(WinEvent));	
	}

	virtual void wait_event(void* event_object){
		((WinEvent*)event_object)->wait();
	}

	virtual void signal_event(void* event_object){
		((WinEvent*)event_object)->signal();
	}


	virtual void yield(){
		Sleep(0);
	}

	virtual void sleep(float_t sec){
		Sleep((DWORD)(1000*sec));
	}
};

}
