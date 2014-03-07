#include "xtal.h"
#include "xtal_macro.h"

#include "xtal_serializer.h"

namespace xtal{

uint_t Stream::on_tell(){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(tell), null));
	return 0;
}

void Stream::put_s(const AnyPtr& str){
	put_s(str->to_s());
}

void Stream::put_s(const char_t* str, uint_t length){
	write(str, length*sizeof(char_t));
}

uint_t Stream::on_write(const void*, uint_t){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(write), null));
	return 0;
}

uint_t Stream::on_read(void*, uint_t){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(read), null));
	return 0;
}

void Stream::on_seek(uint_t){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(seek), null));
}

uint_t Stream::on_available(){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(available), null));
	return 0;
}


StringPtr Stream::on_get_s(uint_t length){
	if(eos()){
		return empty_string;
	}

	if(length==1){
		return get_ch();
	}

	MemoryStreamPtr ms = xnew<MemoryStream>();
	for(uint_t i=0; i<length; ++i){
		ms->put_s(get_s(1));
	}

	return ms->to_s();
}

StringPtr Stream::get_ch(){
	char_t ch;
	ChMaker chm;
	while(!chm.is_completed()){
		read(&ch, sizeof(char_t));
		chm.add(ch);
	}
	return chm.to_s();
}

StringPtr Stream::on_get_s_all(){
	if(eos()){
		return empty_string;
	}

	MemoryStreamPtr ms = xnew<MemoryStream>();
	while(!eos()){
		ms->put_s(get_s(1));
	}

	return ms->to_s();
}

uint_t Stream::on_read_charactors(AnyPtr* buffer, uint_t max){
	int nn = 0;
	for(uint_t i=0; i<max; ++i){
		if(eos()){
			return i;
		}

		char_t ch;
		ChMaker chm;
		while(!chm.is_completed()){
			read(&ch, sizeof(char_t));
			chm.add(ch);
		}

		XTAL_ASSERT(i<max);
	
		buffer[i] = chm.to_s();
		nn++;
	}
	return max;
}

void Stream::read_strict(void* p, uint_t size){
	if(size==0){
		return;
	}

	uint_t read = 0;
	while(true){
		int temp = this->read((char*)p+read, size-read);

		if(temp<=0){
			XTAL_SET_EXCEPT(cpp_class<EOSError>()->call(Xt("XRE1033")));
			return;
		}

		read += temp;
		if(read==size){
			break;
		}
	}
}

void Stream::print(const AnyPtr& value){
	put_s(value);
}

void Stream::println(const AnyPtr& value){
	put_s(value);
	put_s(XTAL_STRING("\n"));
}

void Stream::printf(const StringPtr& format_string, const ArgumentsPtr& args){
	put_s(format_string->call(args));
}

uint_t Stream::on_pour(const StreamPtr& in_stream, uint_t size){
	XMallocGuard umg(size*sizeof(xtal::u8));
	xtal::u8* buf = (xtal::u8*)umg.get();
	uint_t len = in_stream->read(buf, size);
	write(buf, len);
	return len;
}

uint_t Stream::on_pour_all(const StreamPtr& in_stream){
	uint_t size = 1024*10, len, sum = 0;
	XMallocGuard umg(size*sizeof(xtal::u8));
	xtal::u8* buf = (xtal::u8*)umg.get();
	do{
		len = in_stream->read(buf, size);
		sum += len;
		write(buf, len);
	}while(len==size);
	return sum;
}
	
void Stream::skip_bom(){
#ifdef XTAL_USE_WCHAR
	char_t head[1] = {0};
	int_t pos = tell();
	int_t n = read(head, 1*sizeof(char_t));

	if(n==1*sizeof(char_t) && (uchar_t)head[0]==(uchar_t)0xFEFF || (uchar_t)head[0]==(uchar_t)0xFFFE){
		return;
	}	

	seek(pos);

#else

	char_t head[3] = {0};
	int_t pos = tell();
	int_t n = read(head, 3*sizeof(char_t));

	if(n==3*sizeof(char_t) && (uchar_t)head[0]==(uchar_t)239 && (uchar_t)head[1]==(uchar_t)187 && (uchar_t)head[2]==(uchar_t)191){
		return;
	}	

	seek(pos);
#endif	
}

uint_t Stream::on_size(){
	XTAL_SET_EXCEPT(unsupported_error(get_class(), Xid(size), null));
	return 0;
}

void Stream::block_first(const VMachinePtr& vm){
	vm->return_result(to_smartptr(this), to_smartptr(this));
}

void Stream::block_next(const VMachinePtr& vm){
	vm->return_result(null, null);
	close();
}

void Stream::block_break(const VMachinePtr&){
	close();
}

void Stream::serialize(const AnyPtr& obj){
	Serializer s(to_smartptr(this));
	s.serialize(obj);
}

AnyPtr Stream::deserialize(){
	Serializer s(to_smartptr(this));
	return s.deserialize();
}

void Stream::put_u16be(u16 v){
	struct{ u8 data[2]; } data;
	data.data[0] = (u8)(v>>8);
	data.data[1] = (u8)(v>>0);
	write(data.data, 2);
}

void Stream::put_u16le(u16 v){
	struct{ u8 data[2]; } data;
	data.data[1] = (u8)(v>>8);
	data.data[0] = (u8)(v>>0);
	write(data.data, 2);
}

void Stream::put_u32be(u32 v){
	struct{ u8 data[4]; } data;
	data.data[0] = (u8)(v>>24);
	data.data[1] = (u8)(v>>16);
	data.data[2] = (u8)(v>>8);
	data.data[3] = (u8)(v>>0);
	write(data.data, 4);
}

void Stream::put_u32le(u32 v){
	struct{ u8 data[4]; } data;
	data.data[3] = (u8)(v>>24);
	data.data[2] = (u8)(v>>16);
	data.data[1] = (u8)(v>>8);
	data.data[0] = (u8)(v>>0);
	write(data.data, 4);
}

void Stream::put_u64be(u64 v){
	struct{ u8 data[8]; } data;
	data.data[0] = (u8)(v>>56);
	data.data[1] = (u8)(v>>48);
	data.data[2] = (u8)(v>>40);
	data.data[3] = (u8)(v>>32);
	data.data[4] = (u8)(v>>24);
	data.data[5] = (u8)(v>>16);
	data.data[6] = (u8)(v>>8);
	data.data[7] = (u8)(v>>0);
	write(data.data, 8);
}

void Stream::put_u64le(u64 v){
	struct{ u8 data[8]; } data;
	data.data[7] = (u8)(v>>56);
	data.data[6] = (u8)(v>>48);
	data.data[5] = (u8)(v>>40);
	data.data[4] = (u8)(v>>32);
	data.data[3] = (u8)(v>>24);
	data.data[2] = (u8)(v>>16);
	data.data[1] = (u8)(v>>8);
	data.data[0] = (u8)(v>>0);
	write(data.data, 8);
}

u8 Stream::get_u8(){
	struct{ u8 data[1]; } data = {{0}};
	read_strict(data.data, 1);
	return (u8)data.data[0];
}

u16 Stream::get_u16be(){
	struct{ u8 data[2]; } data = {{0}};
	read_strict(data.data, 2);
	return (u16)((data.data[0]<<8) | data.data[1]);
}

u16 Stream::get_u16le(){
	struct{ u8 data[2]; } data = {{0}};
	read_strict(data.data, 2);
	return (u16)((data.data[1]<<8) | data.data[0]);
}

u32 Stream::get_u32be(){
	struct{ u8 data[4]; } data = {{0}};
	read_strict(data.data, 4);
	return (u32)((data.data[0]<<24) | (data.data[1]<<16) | (data.data[2]<<8) | data.data[3]);
}

u32 Stream::get_u32le(){
	struct{ u8 data[4]; } data = {{0}};
	read_strict(data.data, 4);
	return (u32)((data.data[3]<<24) | (data.data[2]<<16) | (data.data[1]<<8) | data.data[0]);
}

u64 Stream::get_u64be(){
	struct{ u8 data[8]; } data = {{0}};
	read_strict(data.data, 8);
	return (u64)(((u64)data.data[0]<<56) | ((u64)data.data[1]<<48) | ((u64)data.data[2]<<40) | ((u64)data.data[3]<<32) | (data.data[4]<<24) | (data.data[5]<<16) | (data.data[6]<<8) | data.data[7]);
}

u64 Stream::get_u64le(){
	struct{ u8 data[8]; } data = {{0}};
	read_strict(data.data, 8);
	return (u64)(((u64)data.data[7]<<56) | ((u64)data.data[6]<<48) | ((u64)data.data[5]<<40) | ((u64)data.data[4]<<32) | (data.data[3]<<24) | (data.data[2]<<16) | (data.data[1]<<8) | data.data[0]);
}

/////////////////////////////////////////////////////////////////

PointerStream::PointerStream(const void* data, uint_t size){
	static u8 temp = 0;
	if(data){
		data_ = (u8*)data;
	}
	else{
		data_ = &temp;
	}
	size_ = size;
	pos_ = 0;
}
	
uint_t PointerStream::on_tell(){
	return pos_;
}

uint_t PointerStream::on_read(void* p, uint_t size){
	if(pos_+size>size_){ 
		size = size_-pos_;
	}
	
	if(size>0){
		std::memcpy(p, &data_[pos_], size);
	}

	pos_ += size;
	return size;
}

void PointerStream::on_seek(uint_t offset){
	pos_ = offset;
}

StringPtr PointerStream::on_get_s(uint_t length){
	if(pos_ >= size_)
		return empty_string;

	if(length==1){
		return get_ch();
	}

	uint_t saved = pos_;

	uint_t slen = 0;
	char_t ch;
	ChMaker chm;
	while(slen<length){
		chm.clear();
		while(!chm.is_completed()){
			read(&ch, sizeof(char_t));
			chm.add(ch);
		}
		slen += 1;
	}

	return xnew<String>((char_t*)&data_[saved], (pos_ - saved)/sizeof(char_t));	
}

StringPtr PointerStream::on_get_s_all(){
	if(pos_ >= size_)
		return empty_string;

	char_t* data = (char_t*)data_;
	StringPtr ret = xnew<String>((char_t*)&data[pos_], size_ - pos_);
	pos_ = size_;
	return ret;
}

bool PointerStream::on_eos(){
	return pos_>=size_;
}

///////////////////////////////////////////////////////////////

MemoryStream::MemoryStream(){
	pos_ = 0;
	capa_ = 0;
}

MemoryStream::MemoryStream(const void* data, uint_t data_size){
	pos_ = 0;
	capa_ = 0;
	resize(data_size);
	std::memcpy((void*)data_, data, data_size);
}

MemoryStream::~MemoryStream(){
	if(capa_){
		xfree((void*)data_, capa_);
	}
}

uint_t MemoryStream::on_write(const void* p, uint_t size){
	if(pos_+size>capa_){
		resize(pos_+size);
	}
	else{
		size_ = pos_+size;
	}

	std::memcpy((void*)&data_[pos_], p, size);

	pos_ += size;

	return size;	
}

uint_t MemoryStream::on_pour(const StreamPtr& in_stream, uint_t size){
	if(size==0){
		return 0;
	}

	resize(pos_+size);

	uint_t len = in_stream->read((void*)&data_[pos_], size);
	resize(size_ - (size - len));
	pos_ += len;
	return len;
}

uint_t MemoryStream::on_pour_all(const StreamPtr& in_stream){
	uint_t size = 1024*10, len, sum = 0;
	do{
		if(size_ <= pos_ + size){
			resize(pos_+size);
		}

		len = in_stream->read((void*)&data_[pos_], size);
		sum += len;
	}while(len==size);
	resize(size_ - (size - len));
	pos_ += sum;
	return sum;
}

const StringPtr& MemoryStream::to_s(){
	str_ = XNew<String>((char_t*)data_, size_/sizeof(char_t));
	return str_;
}

void MemoryStream::clear(){
	seek(0);
	resize(0);
}

void MemoryStream::resize(uint_t size){
	if(size>capa_){
		uint_t newcapa = size + capa_*2;
		void* newp = xmalloc(newcapa);
		std::memcpy(newp, data_, size_);
		if(capa_){
			xfree((void*)data_, capa_);
		}
		data_ = (u8*)newp;
		capa_ = newcapa;
	}
	size_ = size;
}

//////////////////////////////////////////////////////////////////////////

StringStream::StringStream(const StringPtr& str)
:str_(str ? str : empty_string){
	data_ = (u8*)str_->data();
	size_ = str_->data_size()*sizeof(char_t);
	pos_ = 0;
}

//////////////////////////////////////////////////////////////////////////

void FileStream::open(const StringPtr& path, const StringPtr& aflags){
	close();

	const char_t* flags = aflags->data();
	char_t flags_temp[256];
	bool text = false;
	uint_t i = 0;
	uint_t sz = aflags->data_size();
	for(; flags[i]!=0 && i<sz; ++i){
		if(flags[i]=='t'){
			text = true;
		}
		else{
			flags_temp[i] = flags[i];
		}
	}

	if(!text){
		flags_temp[i++] = 'b';
	}
	flags_temp[i++] = 0;

	impl_ = filesystem_lib()->new_file_stream(path->c_str(), flags_temp);
}

///////////////////////////////////////////////////////////////////////////

class LZEncoder{
public:

	LZEncoder(const StreamPtr& out){
		out_ = out;
		state_ = 0;
	}

	void finish(){
		while(state_>=0){
			encode(0, 0);
		}
	}

	void encode(u8* first, u8* last);

	enum{
		RING_BUF_SIZE = 4096,
		RING_BUF_MASK = RING_BUF_SIZE-1,
		NO_COMPRESS_SIZE = 3,
		MAX_MATCH_LEN = 15+NO_COMPRESS_SIZE,
		NIL = RING_BUF_SIZE
	};	

private:
	enum{
		TEXT_SIZE = RING_BUF_SIZE+MAX_MATCH_LEN-1,
		DAD_SIZE = RING_BUF_SIZE+1,
		LSON_SIZE = RING_BUF_SIZE+1,
		RSON_SIZE = RING_BUF_SIZE+256+1
	};

	int state_;

	u8 code_[17];
	int code_count_;
	unsigned int mask_;

	int s_;
	int r_;
	int len_;
	int i_;
	int lastmatchlen_;

	StreamPtr out_;

	u8 text_[RING_BUF_SIZE+MAX_MATCH_LEN-1];
	int parent_node_[RING_BUF_SIZE+1];
	int left_node_[RING_BUF_SIZE+1];
	int right_node_[RING_BUF_SIZE+256+1];
	int matchpos_;
	int matchlen_;
	
	void insert(int r);
	void erase(int p);
};

void LZEncoder::encode(u8* first, u8* last){
	if(first==0){
		switch(state_){
		case 0: goto finish0;
		case 1: goto finish1;
		case 2: goto finish2;
		}
	}
	else{
		if(first==last){ return; }
		switch(state_){
		case 0: goto yield0;
		case 1: goto yield1;
		case 2: goto yield2;
		}
	}

finish0:
yield0:

	matchpos_ = 0;
	matchlen_ = 0;
	
	for(int i=RING_BUF_SIZE; i<=RING_BUF_SIZE+256; ++i){
		right_node_[i] = NIL;
	}

	for(int i=0; i<RING_BUF_SIZE; ++i){
		parent_node_[i] = NIL;
	}

	std::fill(text_, text_+TEXT_SIZE, 0);
	
	code_[0] = 0;

	mask_ = 1;
	code_count_ = 1;

	s_ = 0;
	r_ = RING_BUF_SIZE-MAX_MATCH_LEN;

	for(len_ = 0; len_<MAX_MATCH_LEN; len_++){
		if(first==last){
			state_ = 1;
			return;
			finish1:
			break;
			yield1:;
		}

		text_[r_+len_] = *first++;
	}

	for(int i=1; i<=MAX_MATCH_LEN; i++){
		insert(r_-i);
	}

	insert(r_);

	do{
		if(matchlen_>len_){
			matchlen_ = len_;
		}

		if(matchlen_<NO_COMPRESS_SIZE){
			matchlen_ = 1;
			code_[0] |= mask_;
			code_[code_count_++] = text_[r_];
		}
		else{
			code_[code_count_++] = (u8)matchpos_;
			code_[code_count_++] = (u8)(((matchpos_>>4) & 0xf0) | (matchlen_-NO_COMPRESS_SIZE));
		}

		mask_ = (mask_<<1) & 0xff;
		if(mask_==0){
			out_->write(code_, code_count_);
			mask_ = code_count_ = 1;
			code_[0] = 0;
		}
		
		lastmatchlen_ = matchlen_;
		for(i_ = 0; i_<lastmatchlen_; i_++){
			if(first==last){
				state_ = 2;
				return;
				finish2:
				break;
				yield2:;
			}
			u8 c = *first++;

			erase(s_);	
			text_[s_] = c;

			if(s_<MAX_MATCH_LEN-1){
				text_[s_+RING_BUF_SIZE] = c;
			}

			s_ = (s_+1) & RING_BUF_MASK;	
			r_ = (r_+1) & RING_BUF_MASK;
			insert(r_);
		}

		while(i_++<lastmatchlen_){
			erase(s_);
			s_ = (s_+1) & RING_BUF_MASK;	
			r_ = (r_+1) & RING_BUF_MASK;
			if(--len_){
				insert(r_);
			}
		}

	}while(len_>0);

	if(code_count_>1){
		out_->write(code_, code_count_);
	}

	state_ = -1;
}

void LZEncoder::insert(int r){		
	int cmp = 1;
	u8* key = &text_[r];
	int p = RING_BUF_SIZE + 1 + key[0];
	right_node_[r] = left_node_[r] = NIL;
	matchlen_ = 0;
	while(true){
		if(cmp>=0){
			if(right_node_[p]!=NIL){
				p = right_node_[p];
			}
			else{	
				right_node_[p] = r;	
				parent_node_[r] = p;
				return;	
			}
		}
		else{
			if(left_node_[p]!=NIL){
				p = left_node_[p];
			}
			else{	
				left_node_[p] = r;	
				parent_node_[r] = p;
				return;	
			}
		}

		int i = 1;
		for(; i<MAX_MATCH_LEN; i++){
			cmp = key[i]-text_[p+i];
			if(cmp!=0){
				break;
			}
		}

		if(i>matchlen_){
			matchpos_ = p;
			matchlen_ = i;
			if(matchlen_>=MAX_MATCH_LEN){	
				break;
			}
		}
	}

	parent_node_[r] = parent_node_[p];
	left_node_[r] = left_node_[p];
	right_node_[r] = right_node_[p];
	parent_node_[left_node_[p]] = r;
	parent_node_[right_node_[p]] = r;
	
	if(right_node_[parent_node_[p]] == p){
		right_node_[parent_node_[p]] = r;
	}
	else{				
		left_node_[parent_node_[p]] = r;
	}

	parent_node_[p] = NIL;
	
}

void LZEncoder::erase(int p){
	int q;
	
	if(parent_node_[p]==NIL){
		return;
	}

	if(right_node_[p]==NIL){
		q = left_node_[p];
	}
	else if(left_node_[p]==NIL){
		q = right_node_[p];
	}
	else{
		q = left_node_[p];
		if(right_node_[q]!=NIL){
			q = right_node_[q];
			while(right_node_[q]!=NIL){
				q = right_node_[q];
			}

			right_node_[parent_node_[q]] = left_node_[q];
			parent_node_[left_node_[q]] = parent_node_[q];
			left_node_[q] = left_node_[p];
			parent_node_[left_node_[p]] = q;
		}
		right_node_[q] = right_node_[p];
		parent_node_[right_node_[p]] = q;
	}
	
	parent_node_[q] = parent_node_[p];
	
	if(right_node_[parent_node_[p]]==p){	
		right_node_[parent_node_[p]] = q;
	}
	else{						
		left_node_[parent_node_[p]] = q;
	}

	parent_node_[p] = NIL;
}

CompressEncoder::CompressEncoder(const StreamPtr& stream){
	LZEncoder* p = new(object_xmalloc<LZEncoder>()) LZEncoder(stream);
	impl_ = p;
}

CompressEncoder::~CompressEncoder(){
	destroy();
}

uint_t CompressEncoder::on_write(const void* data, uint_t size){
	if(impl_){
		LZEncoder* p = (LZEncoder*)impl_;
		p->encode((u8*)data, (u8*)data + size);
		return size;
	}
	return 0;
}

void CompressEncoder::on_close(){
	if(impl_){
		LZEncoder* p = (LZEncoder*)impl_;
		p->finish();

		destroy();
	}
}

void CompressEncoder::destroy(){
	if(impl_){
		LZEncoder* p = (LZEncoder*)impl_;
		p->~LZEncoder();
		object_xfree<LZEncoder>(p);
		impl_ = 0;
	}
}

class LZDecoder{
public:

	enum{
		RING_BUF_SIZE = 4096,
		RING_BUF_MASK = RING_BUF_SIZE-1,
		NO_COMPRESS_SIZE = 3,
		MAX_MATCH_LEN = 15+NO_COMPRESS_SIZE
	};	

	LZDecoder(const StreamPtr& in){
		in_ = in;
		state_ = 0;
	}

	u8* decode(u8* first, u8* last);

private:

	StreamPtr in_;

	int state_;
	u8 text_[RING_BUF_SIZE];
	u8 c_;
	int r_;
	unsigned int flags_;
	int pos_;
	int len_;
	int i_;
};

u8* LZDecoder::decode(u8* first, u8* last){
	unsigned int c1 = 0;
	unsigned int c2 = 0;

	if(first==last){ return first; }
	switch(state_){
	case -1: return first;
	case 0: goto yield0;
	case 1: goto yield1;
	case 2: goto yield2;
	}

yield0:
	std::fill(text_, text_+RING_BUF_SIZE, 0);
	r_ = RING_BUF_SIZE - MAX_MATCH_LEN;
	flags_ = 0;

	while(true){
		flags_ >>= 1;
		if((flags_&(1<<8))==0){
			u8 c;
			if(in_->read(&c, 1)!=1){
				state_ = -1;
				return first;
			}

			flags_ = (int)c | 0xff00;
		}

		if(flags_ & 1){
			if(in_->read(&c_, 1)!=1){
				state_ = -1;
				return first;
			}

			if(first==last){
				state_ = 1;
				return first;
				yield1:;
			}
			*first++ = text_[r_] = c_;
			r_ = (r_+1) & RING_BUF_MASK;
		}
		else{
			u8 cc[2];
			if(in_->read(cc, 2)!=2){
				state_ = -1;
				return first;
			}

			c1 = cc[0];
			c2 = cc[1];

			pos_ = c1 | ((c2 & 0xf0)<<4);
			len_ = (c2 & 0x0f) + NO_COMPRESS_SIZE;

			for(i_=0; i_<len_; i_++){
				if(first==last){
					state_ = 2;
					return first;
					yield2:;
				}

				*first++ = text_[r_] = text_[(pos_+i_) & RING_BUF_MASK];
				r_ = (r_+1) & RING_BUF_MASK;
			}
		}
	}

	return first;
}

CompressDecoder::CompressDecoder(const StreamPtr& stream){
	LZDecoder* p = new(object_xmalloc<LZDecoder>()) LZDecoder(stream);
	impl_ = p;
}

CompressDecoder::~CompressDecoder(){
	destroy();
}

uint_t CompressDecoder::on_read(void* data, uint_t size){
	if(impl_){
		LZDecoder* p = (LZDecoder*)impl_;
		u8* out = p->decode((u8*)data, (u8*)data + size);
		return out - (u8*)data;
	}
	return 0;
}

void CompressDecoder::destroy(){
	if(impl_){
		LZDecoder* p = (LZDecoder*)impl_;
		p->~LZDecoder();
		object_xfree<LZDecoder>(p);
		impl_ = 0;
	}
}

void CompressDecoder::on_close(){
	destroy();
}

///////////////////////////////////////////////////////////////////////////

XNew<MemoryStream>::XNew(){ init(); }
XNew<MemoryStream>::XNew(const void* data, uint_t data_size){ init(data, data_size); }

}
