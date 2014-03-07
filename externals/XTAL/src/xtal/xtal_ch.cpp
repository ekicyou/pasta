#include "xtal.h"

namespace xtal{

////////////////////////////////////////////////

int_t ChCodeLib::ch_inc(const char_t* data, int_t data_size, char_t* dest, int_t dest_size){
	const char_t* p;
	int_t sz;

	if(data_size>6){
		p = data;
		sz = data_size;
	}
	else{
		uchar_t buf[8] = {0};
		std::memcpy(buf+1, data, data_size*sizeof(uchar_t));

		for(int_t i=data_size; i>=0; --i){
			buf[i]++;
			if(buf[i]==0){
				continue;
			}
			break;
		}

		if(buf[0]==0){
			p = (char_t*)buf+1;
			sz = data_size;
		}
		else{
			p = (char_t*)buf;
			sz = data_size+1;
		}
	}
	
	if(dest_size<sz){
		sz = dest_size;	
	}
	
	std::memcpy(dest, p, sz*sizeof(uchar_t));
	return sz;
}

int_t ChCodeLib::ch_cmp(const char_t* a, int_t asize, const char_t* b, int_t bsize){
	if(asize==bsize){
		for(int_t i=0; i<asize; ++i){
			if((uchar_t)a[i]<(uchar_t)b[i]){
				return -1;
			}
			else if((uchar_t)a[i]>(uchar_t)b[i]){
				return 1;
			}
		}
		return 0;
	}
	else if(asize<bsize){
		return -1;
	}
		
	return 1;
}

uint_t edit_distance(const void* data1, uint_t data_size1, const void* data2, uint_t data_size2){
	struct Helper{
		static void calc(const void* data1, uint_t size1, const void* data2, uint_t size2, uint_t* buf, uint_t k, int_t offset){
			uint_t v1 = buf[k-1+offset]+1;
			uint_t v2 = buf[k+1+offset];
			uint_t x = v1>v2 ? v1 : v2;
			uint_t y = x-k;
			while(x<size1 && y<size2 && ((u8*)data1)[x-1]==((u8*)data2)[y-1]){
				++x;
				++y;
			}
			buf[k+offset] = x;
		}
	};

	if(data_size1<data_size2){
		return edit_distance(data2, data_size2, data1, data_size1);
	}

	uint_t size1 = data_size1 + 1;
	uint_t size2 = data_size2 + 1;

	uint_t buf_size = size1+size2+6;
	uint_t* buf = (uint_t*)xmalloc(sizeof(uint_t)*buf_size);
	
	for(uint_t i=0; i<buf_size; ++i){
		buf[i] = 0;
	}

	int_t offset = size2+1, delta = size1-size2;
	for(int_t p=0;;++p){
		for(int_t k=-p; k<delta; ++k){
			Helper::calc(data1, size1, data2, size2, buf, k, offset);
		}

		for(int_t k=delta+p; k>delta; --k){
			Helper::calc(data1, size1, data2, size2, buf, k, offset);
		}

		Helper::calc(data1, size1, data2, size2, buf, delta, offset);

		if(buf[delta+offset]==size1){
			xfree(buf, sizeof(uint_t)*buf_size);
			return p+delta;
		}
	}
//	return 0;
}

}
