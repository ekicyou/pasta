#include "callstackview.h"

CallStackView::CallStackView(QWidget *parent)
	:QTreeView(parent){
	model_ = new QStandardItemModel(this);
	setModel(model_);

	model_->setHorizontalHeaderItem(0, new QStandardItem("function"));
	model_->setHorizontalHeaderItem(1, new QStandardItem("file"));
	model_->setHorizontalHeaderItem(2, new QStandardItem("line"));

	connect(this, SIGNAL(doubleClicked(const QModelIndex&)), this, SLOT(onClicked(const QModelIndex&)));
}

QStandardItem* CallStackView::makeItem(const QString& text){
	QStandardItem* ret = new QStandardItem(text);
	ret->setEditable(false);
	return ret;
}

/*
void CallStackView::view(const VMachinePtr& vm){
	for(int i=0;; ++i){
		if(debug::CallerInfoPtr caller = vm->caller(i)){
			model_->setItem(i, 0, makeItem(caller->fun_name()->c_str()));
			model_->setItem(i, 1, makeItem(caller->file_name()->c_str()));
			if(caller->lineno()){
				model_->setItem(i, 2, makeItem(QString("%1").arg(caller->lineno())));
			}
		}
		else{
			break;
		}
	}
}
*/
void CallStackView::setLevel(int n){
	model_->item(n, 0)->setIcon(QIcon("data/right.png"));
}

void CallStackView::set(int i, const StringPtr& fun, const StringPtr& file, int line){
	model_->setItem(i, 0, makeItem(fun->c_str()));
	model_->setItem(i, 1, makeItem(file->c_str()));
	if(line){
		model_->setItem(i, 2, makeItem(QString("%1").arg(line)));
	}
}

void CallStackView::clear(){
	model_->setRowCount(0);
}

void CallStackView::onClicked(const QModelIndex & index){
	emit moveCallStack(index.row());
}

