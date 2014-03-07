/** \file src/xtal/xtal_cache.h
* \brief src/xtal/xtal_cache.h
*/

#ifndef XTAL_CACHE_H_INCLUDE_GUARD
#define XTAL_CACHE_H_INCLUDE_GUARD
#pragma once

namespace xtal{

extern uint_t member_mutate_count_;
extern uint_t is_mutate_count_;
extern uint_t cache_enable_;

void enable_cashe(uint_t);

struct MemberCacheTable{
	struct Unit{
		uint_t mutate_count;
		uint_t accessibility;
		AnyPtr target_class;
		AnyPtr primary_key;
		AnyPtr member;
	};

	enum{ CACHE_MAX = 127, CACHE_MASK = CACHE_MAX };

	Unit table_[CACHE_MAX];

	int_t hit_;
	int_t miss_;

	MemberCacheTable(){
		hit_ = 0;
		miss_ = 0;
	}

	int_t hit_count(){
		return hit_;
	}

	int_t miss_count(){
		return miss_;
	}

	const AnyPtr& cache(const AnyPtr& target_class, const IDPtr& primary_key, int_t& accessibility);

	void clear(){
		for(int_t i=0; i<CACHE_MAX; ++i){
			Unit& unit = table_[i];
			unit.target_class = null;
			unit.primary_key = null;
			unit.member = null;	
		}
	}
};

struct MemberCacheTable2{
	struct Unit{
		uint_t mutate_count;
		uint_t accessibility;
		AnyPtr target_class;
		AnyPtr primary_key;
		AnyPtr secondary_key;
		AnyPtr member;
	};

	enum{ CACHE_MAX = 127, CACHE_MASK = CACHE_MAX };

	Unit table_[CACHE_MAX];

	int_t hit_;
	int_t miss_;

	MemberCacheTable2(){
		hit_ = 0;
		miss_ = 0;
	}

	int_t hit_count(){
		return hit_;
	}

	int_t miss_count(){
		return miss_;
	}

	const AnyPtr& cache(const AnyPtr& target_class, const IDPtr& primary_key, const AnyPtr& secondary_key, int_t& accessibility);

	void clear(){
		for(int_t i=0; i<CACHE_MAX; ++i){
			Unit& unit = table_[i];
			unit.target_class = null;
			unit.primary_key = null;
			unit.secondary_key = null;
			unit.member = null;	
		}
	}
};

struct IsCacheTable{
	struct Unit{
		uint_t mutate_count;
		AnyPtr target_class;
		AnyPtr klass;
		bool result;
	};

	enum{ CACHE_MAX = 127, CACHE_MASK = CACHE_MAX };


	Unit table_[CACHE_MAX];

	int_t hit_;
	int_t miss_;

	IsCacheTable(){
		hit_ = 0;
		miss_ = 0;
	}

	int_t hit_count(){
		return hit_;
	}

	int_t miss_count(){
		return miss_;
	}

	bool cache(const AnyPtr& target_class, const AnyPtr& klass){
		uint_t itarget_class = XTAL_detail_uvalue(target_class);
		uint_t iklass = XTAL_detail_uvalue(klass);

		uint_t hash = (itarget_class>>3) ^ (iklass>>2);
		Unit& unit = table_[hash % CACHE_MASK];
		
		if(cache_enable_ && is_mutate_count_==unit.mutate_count && 
			XTAL_detail_raweq(target_class, unit.target_class) && 
			XTAL_detail_raweq(klass, unit.klass)){

			hit_++;
			return unit.result;
		}
		else{
			miss_++;

			bool ret = unchecked_ptr_cast<Class>(target_class)->is_inherited(klass);

			unit.target_class = target_class;
			unit.klass = klass;
			unit.mutate_count = is_mutate_count_;
			unit.result = ret;
			return ret;
		}
	}

	void clear(){
		for(int_t i=0; i<CACHE_MAX; ++i){
			Unit& unit = table_[i];
			unit.target_class = null;
			unit.klass = null;
		}
	}
};

}


#endif // XTAL_CACHE_H_INCLUDE_GUARD
