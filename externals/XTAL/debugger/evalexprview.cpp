#include "evalexprview.h"
#include "mainwindow.h"

EvalExprView::EvalExprView(QWidget *parent)
	:QTreeView(parent){
	model_ = new QStandardItemModel(this);
	model_->setRowCount(0);
	model_->setRowCount(256);
	setModel(model_);

	model_->setHorizontalHeaderItem(0, new QStandardItem("expr"));
	model_->setHorizontalHeaderItem(1, new QStandardItem("value"));
	model_->setHorizontalHeaderItem(2, new QStandardItem("class"));

	connect(model_, SIGNAL(itemChanged(QStandardItem*)), this, SLOT(dataChanged(QStandardItem*)));

	dataChanged_ = false;
	exprCount_ = 0;
}

void EvalExprView::dataChanged(QStandardItem* item){
	if(item->isEditable() && item->column()==0){
		if(exprCount_<=item->row()){
			exprCount_ = item->row()+1;
		}

		emit exprChanged(item->row(), item->text());
	}
}

void EvalExprView::setExpr(int n, const QString& expr){
	if(exprCount_<=n){
		exprCount_ = n+1;
	}

	model_->invisibleRootItem()->setChild(n, 0, makeItem(expr, true));
}

QString EvalExprView::expr(int n){
	if(model_->item(n, 0)){
		return model_->item(n, 0)->text();
	}
	return QString("");
}

void EvalExprView::setExprResult(int n, const ArrayPtr& value){
	setChild(model_->invisibleRootItem(), n, "", value);
}

void EvalExprView::setChild(QStandardItem* item, int n, const QString& key, const ArrayPtr& value){
	bool primary = item==model_->invisibleRootItem();

	if(value && value->size()>=3){
		if(!primary){ item->setChild(n, 0, makeItem(key)); }
		item->setChild(n, 1, makeItem(value->at(1)->to_s()->c_str()));
		item->setChild(n, 2, makeItem(value->at(0)->to_s()->c_str()));

		if(value->at(2)){
			setChild(item->child(n, 0), value->at(2));
		}
		else{
			item->child(n, 0)->removeRows(0, item->child(n, 0)->rowCount());
		}
	}
	else{
		if(item->child(n, 0)){
			item->child(n, 0)->removeRows(0, item->child(n, 0)->rowCount());
		}
		item->setChild(n, 1, makeItem(""));
		item->setChild(n, 2, makeItem(""));
	}
}

void EvalExprView::setChild(QStandardItem* item, const AnyPtr& children){
	if(ArrayPtr ary = ptr_cast<Array>(children)){
		int i = 0;
		Xfor(val, ary){
			ArrayPtr it = ptr_cast<Array>(val);
			setChild(item, i, QString("%1").arg(i), it);
			++i;
		}
	}
	else if(MapPtr map = ptr_cast<Map>(children)){
		int i = 0;
		Xfor2(key, val, map){
			ArrayPtr it = ptr_cast<Array>(val);
			setChild(item, i, key->to_s()->c_str(), it);
			++i;
		}
	}
	else if(StringPtr str = ptr_cast<String>(children)){
		item->setChild(0, 0, makeItem(str->c_str()));
	}
}

QStandardItem* EvalExprView::makeItem(const QString& text, bool editable){
	QStandardItem* ret = new QStandardItem(text);
	ret->setEditable(editable);
	return ret;
}
