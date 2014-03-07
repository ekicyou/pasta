#include "xtal.h"

namespace xtal{ namespace filesystem{

StreamPtr open(const StringPtr& path, const StringPtr& flags){
	return xnew<FileStream>(path, flags);
}

AnyPtr entries(const StringPtr& path){
	return xnew<Entries>(path);
}

bool is_directory(const StringPtr& path){
	return filesystem_lib()->is_directory(path->c_str());
}

}}
