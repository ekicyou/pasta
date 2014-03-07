#include "xtal.h"

#ifndef XTAL_NO_PARSER

namespace xtal{

const ExprPtr& ep(const AnyPtr& a){
	if(XTAL_detail_type(a)==TYPE_TREE_NODE){
		return unchecked_ptr_cast<Expr>(a);
	}
	return nul<Expr>();
}

TreeNode::TreeNode()
	:tag_(null), lineno_(0){}

TreeNode::TreeNode(int_t tag, int_t lineno)	
	:tag_(tag), lineno_(lineno){}

TreeNode::TreeNode(const AnyPtr& tag, int_t lineno)
	:tag_(tag), lineno_(lineno){}

const AnyPtr& TreeNode::at(int_t i){
	if(i>=0){
		if(size()<=(uint_t)i){
			resize(i+1);
		}
		return Array::at(i);
	}
	return Array::op_at(i);
}

void TreeNode::set_at(int_t i, const AnyPtr& v){
	if(i>=0){
		if(size()<=(uint_t)i){
			resize(i+1);
		}
		Array::set_at(i, v); 
	}
	Array::set_at(i, v);
}

}

#endif
