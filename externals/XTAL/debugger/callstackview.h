#ifndef CALLSTACKVIEW_H
#define CALLSTACKVIEW_H

#include <QtGui>

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
using namespace xtal;

/**
  * \breif コールスタックの表示のツリービュー
  */
class CallStackView : public QTreeView{
	Q_OBJECT
public:

	CallStackView(QWidget *parent = 0);

public:

	void init(){
		clear();
    }

    void clear();

    // コールスタックを表示する
    //void view(const VMachinePtr& vm);

    // コールスタックの表示を設定する
    void set(int i, const StringPtr& fun, const StringPtr& file, int line);

    // コールスタックのレベルを設定する
	void setLevel(int n);

public slots:

	void onClicked(const QModelIndex & index);

signals:

    // コールスタック位置の選択シグナル
	void moveCallStack(int n);

private:
    QStandardItem* makeItem(const QString& text);

private:
	QStandardItemModel* model_;
};

#endif // CALLSTACKVIEW_H
