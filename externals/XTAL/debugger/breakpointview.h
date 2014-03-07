#ifndef BREAKPOINTVIEW_H
#define BREAKPOINTVIEW_H

#include <QtGui>

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
using namespace xtal;

/**
  * \brief �u���[�N�|�C���g�̕\���c���[�r���[
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
    // �u���[�N�|�C���g�̏������̕ύX�V�O�i��
	void breakpointConditionChanged(const QString& file, int line, const QString& cond);

    // �u���[�N�|�C���g�ꏊ�̕\���V�O�i��
	void viewBreakpoint(const QString& file, int line);

    // �u���[�N�|�C���g�̏����V�O�i��
	void eraseBreakpoint(const QString& file, int line);

protected:
	QStandardItem* makeItem(const QString& text, bool editable = false);

private:
	QStandardItemModel* model_;
};

#endif // BREAKPOINTVIEW_H
