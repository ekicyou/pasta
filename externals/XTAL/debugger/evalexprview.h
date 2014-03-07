#ifndef EVALEXPRVIEW_H
#define EVALEXPRVIEW_H

#include <QtGui>

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
using namespace xtal;

/**
  * \brief 変数の表示ツリービュー
  */
class EvalExprView : public QTreeView{
	Q_OBJECT
public:

	EvalExprView(QWidget *parent);

	void init(){
		model_->setRowCount(0);
		model_->setRowCount(256);
	}

    // 式を設定する
    void setExpr(int n, const QString& expr);

    // 式を評価した結果を設定する
    void setExprResult(int n, const ArrayPtr& value);

    // 式を取得する
    QString expr(int n);

    // 式の個数を取得する
    int exprCount(){ return exprCount_; }

public slots:

	void dataChanged(QStandardItem* item);

signals:

    // 式が変更されたシグナル
	void exprChanged(int i, const QString& expr);

private:
	void setChild(QStandardItem* item, int n, const QString& key, const ArrayPtr& value);

	void setChild(QStandardItem* item, const AnyPtr& children);

	QStandardItem* makeItem(const QString& text, bool editable = false);

private:
	QStandardItemModel* model_;
	int exprCount_;
	bool dataChanged_;
};

#endif // EVALEXPRVIEW_H
