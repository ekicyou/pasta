
#pragma once

#include <pthread.h>
#include <unistd.h>
#include <sys/time.h>

namespace xtal{

class PThreadMutex{
public:
	pthread_mutex_t mutex_;

	PThreadMutex(){
		pthread_mutexattr_t attr;
		pthread_mutexattr_init(&attr);
		pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
		pthread_mutex_init(&mutex_, &attr);
		pthread_mutexattr_destroy(&attr);
	}
	
	~PThreadMutex(){
		pthread_mutex_destroy(&mutex_);
	}
	
	void lock(){
		pthread_mutex_lock(&mutex_);
	}
	
	void unlock(){
		pthread_mutex_unlock(&mutex_);
	}
};

class PEvent{
	PThreadMutex mutex_;
	pthread_cond_t cond_;
	volatile int signal_;
public:
	PEvent(){
		pthread_cond_init(&cond_, NULL);
		signal_ = 0;
	}
	
	~PEvent(){
		pthread_cond_destroy(&cond_);
	}
	
	void wait(){
		mutex_.lock();
		if(signal_==0){
			pthread_cond_wait(&cond_, &mutex_.mutex_);
		}

		signal_ = 0;
		mutex_.unlock();
	}
	
	void signal(){
		mutex_.lock();
		signal_ = 1;
        pthread_cond_signal(&cond_);
		mutex_.unlock();
	}
};

class PThread{
	bool start_;
	pthread_t id_;
	void (*callback_)(void*);
	void* data_;

	static void* entry(void* self){
		((PThread*)self)->begin_thread();
		return 0;
	}
	
public:

	PThread(){
          start_ = false;
	}

	virtual ~PThread(){
		if(start_){
			pthread_detach(id_);
		}
	}

	virtual void start(void (*callback)(void*), void* data){
		callback_ = callback;
		data_ = data;
		start_ = true;
		pthread_attr_t attr;
		pthread_attr_init(&attr);
		//pthread_attr_setstacksize(&attr, 1024*32);
		pthread_create(&id_, &attr, &entry, this);
		pthread_attr_destroy(&attr);
	}

	virtual void join(){
		if(start_){
			void* p = 0;
			pthread_join(id_, &p);
		}
	}

	virtual void begin_thread(){
		if(start_){
			callback_(data_);
		}
	}
};

class PThreadLib : public ThreadLib{
public:

	virtual void* new_thread(){
		void* p = xmalloc(sizeof(PThread));
		return new(p) PThread();
	}

	virtual void delete_thread(void* thread_object){
		((PThread*)thread_object)->~PThread();
		xfree(thread_object, sizeof(PThread));
	}

	virtual void start_thread(void* thread_object, void (*callback)(void*), void* data){
		((PThread*)thread_object)->start(callback, data);
	}

	virtual void join_thread(void* thread_object){
		((PThread*)thread_object)->join();
	}

	virtual void* new_mutex(){
		void* p = xmalloc(sizeof(PThreadMutex));
		return new(p) PThreadMutex();
	}

	virtual void delete_mutex(void* mutex_object){
		((PThreadMutex*)mutex_object)->~PThreadMutex();
		xfree(mutex_object, sizeof(PThreadMutex));
	}

	virtual void lock_mutex(void* mutex_object){
		((PThreadMutex*)mutex_object)->lock();
	}

	virtual void unlock_mutex(void* mutex_object){
		((PThreadMutex*)mutex_object)->unlock();
	}

	virtual void* new_event(){ 
		void* p = xmalloc(sizeof(PEvent));
		return new(p) PEvent();
	}

	virtual void delete_event(void* event_object){
		((PEvent*)event_object)->~PEvent();
		xfree(event_object, sizeof(PEvent));	
	}

	virtual void wait_event(void* event_object){
		((PEvent*)event_object)->wait();
	}

	virtual void signal_event(void* event_object){
		((PEvent*)event_object)->signal();
	}

	virtual void yield(){
		sched_yield();
	}

	virtual void sleep(float_t sec){
		usleep((useconds_t)(sec*1000*1000));
	}
};

}
