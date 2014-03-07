#ifndef BREAKPOINTVIEW_H
#define BREAKPOINTVIEW_H

#include <QtGui>

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
using namespace xtal;

/**
  * \brief ブレークポイントの表示ツリービュー
  */
class BreakpointView : public QTreeView{
	Q_OBJECT
public:
	BreakpointView(QWidget* parent = 0);

    void init(){
        model_->setRowCount(0);
    }

	void add(const QString& file, int line, const QString& cond);

	void remove(const QString& file, int line);

	void clear();

public slots:
	void dataChanged(QStandardItem* item);

	void onClicked(const QModelIndex & index);

signals:
    // ブレークポイントの条件式の変更シグナル
	void breakpointConditionChanged(const QString& file, int line, const QString& cond);

    // ブレークポイント場所の表示シグナル
	void viewBreakpoint(const QString& file, int line);

    // ブレークポイントの消去シグナル
	void eraseBreakpoint(const QString& file, int line);

protected:
	QStandardItem* makeItem(const QString& text, bool editable = false);

private:
	QStandardItemModel* model_;
};

#endif // BREAKPOINTVIEW_H
