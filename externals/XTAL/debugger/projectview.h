#ifndef PROJECTVIEW_H
#define PROJECTVIEW_H

#include <QtGui>
#include "document.h"

class ProjectViewModel : public QAbstractListModel{
    Q_OBJECT
public:
    ProjectViewModel(Document* doc, QObject* parent = 0)
        :QAbstractListModel(parent), doc_(doc){
        connect(doc, SIGNAL(changed()), this, SLOT(updateView()));
    }

    int rowCount(const QModelIndex &parent = QModelIndex()) const{
        return doc_->pathCount()+1;
    }

    QVariant data(const QModelIndex &index, int role) const{
        QString str;
        if(index.row()<doc_->pathCount()){
            str = doc_->path(index.row());
        }

        //if(index.column() == 0 && role == Qt::DecorationRole) return color;
        if(index.column() == 0 && role == Qt::EditRole){ return str; }
        if(index.column() == 0 && role == Qt::DisplayRole){ return str; }

        return QVariant();
    }

    Qt::ItemFlags flags(const QModelIndex &index) const{
        return QAbstractItemModel::flags(index) | Qt::ItemIsEditable | Qt::ItemIsDragEnabled | Qt::ItemIsDropEnabled;
    }

    bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole){
        if(role == Qt::EditRole || role == Qt::DisplayRole){
            doc_->setPath(index.row(), value.toString());
            emit dataChanged(index, index);
            reset();
            return true;
        }
        return true;
    }

    bool insertRows(int row, int count, const QModelIndex &parent = QModelIndex()){
        beginInsertRows(QModelIndex(), row, row + count - 1);
        for(int r = 0; r < count; ++r){
            doc_->insertPath(row, QString());
        }
        endInsertRows();

        return true;
    }

    bool removeRows(int row, int count, const QModelIndex &parent = QModelIndex()){
        beginRemoveRows(QModelIndex(), row, row + count - 1);
        for(int r = 0; r < count; ++r){
            doc_->removePath(row);
        }
        endRemoveRows();

        return true;
    }

public slots:
    void updateView(){
        reset();
    }

private:
    Document* doc_;
};

class ProjectView : public QTreeView{
    Q_OBJECT
public:
    ProjectView(Document* doc, QWidget* parent = 0);

public:
    void dragEnterEvent(QDragEnterEvent *event);

    void dropEvent(QDropEvent *event);

    void dragMoveEvent(QDragMoveEvent *event);

public slots:
    void onClicked(const QModelIndex & index);

signals:
    void pathAdded(const QString& path);

    void pathSelected(int n);

private:
    ProjectViewModel* model_;
};

#endif // PROJECTVIEW_H
