#ifndef CALLSTACKVIEW_H
#define CALLSTACKVIEW_H

#include <QtGui>

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
using namespace xtal;

/**
  * \breif �R�[���X�^�b�N�̕\���̃c���[�r���[
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

    // �R�[���X�^�b�N��\������
    //void view(const VMachinePtr& vm);

    // �R�[���X�^�b�N�̕\����ݒ肷��
    void set(int i, const StringPtr& fun, const StringPtr& file, int line);

    // �R�[���X�^�b�N�̃��x����ݒ肷��
	void setLevel(int n);

public slots:

	void onClicked(const QModelIndex & index);

signals:

    // �R�[���X�^�b�N�ʒu�̑I���V�O�i��
	void moveCallStack(int n);

private:
    QStandardItem* makeItem(const QString& text);

private:
	QStandardItemModel* model_;
};

#endif // CALLSTACKVIEW_H
