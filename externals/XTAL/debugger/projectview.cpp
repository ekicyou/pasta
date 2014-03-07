#include "projectview.h"

ProjectView::ProjectView(Document* doc, QWidget *parent)
    :QTreeView(parent){
    model_ = new ProjectViewModel(doc, this);
    setModel(model_);

    //model_->setHorizontalHeaderItem(0, new QStandardItem("path"));

    connect(this, SIGNAL(clicked(const QModelIndex&)), this, SLOT(onClicked(const QModelIndex&)));

    setDragEnabled(true);
    setDragDropMode(QAbstractItemView::DragDrop);
    setHeaderHidden(true);
}

void ProjectView::onClicked(const QModelIndex & index){
    emit pathSelected(index.row());
}

void ProjectView::dragEnterEvent(QDragEnterEvent *event){
	dragMoveEvent(event);
}

void ProjectView::dropEvent(QDropEvent *event){
	QList<QUrl> urls = event->mimeData()->urls();
	if(urls.empty()){
		return;
	}

	foreach(QUrl url, urls){
		QString file = url.toLocalFile();

        if(QDir(file).exists()){
            emit pathAdded(file);
        }
	}
}

void ProjectView::dragMoveEvent(QDragMoveEvent *event){
	if(event->mimeData()->hasFormat("text/uri-list")){
		event->acceptProposedAction();
	}
}
