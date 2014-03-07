#include "breakpointview.h"

BreakpointView::BreakpointView(QWidget *parent)
	:QTreeView(parent){
	model_ = new QStandardItemModel(this);
	setModel(model_);

	model_->setHorizontalHeaderItem(0, new QStandardItem("file"));
	model_->setHorizontalHeaderItem(1, new QStandardItem("line"));
	model_->setHorizontalHeaderItem(2, new QStandardItem("condition"));

	connect(this, SIGNAL(clicked(const QModelIndex&)), this, SLOT(onClicked(const QModelIndex&)));
	connect(model_, SIGNAL(itemChanged(QStandardItem*)), this, SLOT(dataChanged(QStandardItem*)));
}

QStandardItem* BreakpointView::makeItem(const QString& text, bool editable){
	QStandardItem* ret = new QStandardItem(text);
	ret->setEditable(editable);
	return ret;
}

void BreakpointView::add(const QString& file, int line, const QString& cond){
	QString sline = QString("%1").arg(line);

	for(int i=0; i<model_->rowCount(); ++i){
		if(model_->item(i, 0)->text()==file){
			if(model_->item(i, 1)->text().toInt()==line){
				model_->setItem(i, 2, makeItem(cond, true));
				return;
			}

			if(model_->item(i, 1)->text().toInt()>line){
				model_->insertRow(i, makeItem(file));
				model_->setItem(i, 1, makeItem(sline));
				model_->setItem(i, 2, makeItem(cond, true));
				return;
			}
		}
	}

	int i = model_->rowCount();
	model_->setItem(i, 0, makeItem(file));
	model_->setItem(i, 1, makeItem(sline));
	model_->setItem(i, 2, makeItem(cond, true));
}

void BreakpointView::remove(const QString& file, int line){
	QString sline = QString("%1").arg(line);
	for(int i=0; i<model_->rowCount(); ++i){
		if(model_->item(i, 0)->text()==file && model_->item(i, 1)->text()==sline){
			model_->removeRow(i);
			return;
		}
	}
}

void BreakpointView::clear(){
	model_->setRowCount(0);
}

void BreakpointView::dataChanged(QStandardItem* item){
	int n = item->row();
	if(item->isEditable() && item->column()==2){
		emit breakpointConditionChanged(model_->item(n, 0)->text(), model_->item(n, 1)->text().toInt(), item->text());
	}
}

void BreakpointView::onClicked(const QModelIndex & index){
	emit viewBreakpoint(model_->item(index.row(), 0)->text(), model_->item(index.row(), 1)->text().toInt());
}

