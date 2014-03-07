#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QtGui>
#include <QPlainTextEdit>
#include <QObject>
#include <QMenu>
#include <QPair>
#include <QVector>

#include <vector>
#include <map>

#include "codeeditor.h"
#include "evalexprview.h"
#include "callstackview.h"
#include "breakpointview.h"
#include "codeeditor.h"
#include "debugger.h"
#include "document.h"
#include "projectview.h"

Q_DECLARE_METATYPE(AnyPtr);

QT_BEGIN_NAMESPACE
class QDragEnterEvent;
class QDropEvent;
QT_END_NAMESPACE

class LineNumberArea;

class AlertDialog : public QDialog{
	Q_OBJECT
public:

	AlertDialog(const QString& mes, QWidget* parent = 0)
		:QDialog(parent){
		QHBoxLayout* mainLayout = new QHBoxLayout();
		setLayout(mainLayout);

		QTextEdit* edit = new QTextEdit();
		QPushButton* cancel = new QPushButton(tr("cancel"));

		mainLayout->addWidget(edit);
		mainLayout->addWidget(cancel);
		connect(cancel, SIGNAL(clicked()), this, SLOT(close()));

		edit->setText(mes);
	}
};

inline void alert(const QString& mes){
	AlertDialog* alert = new AlertDialog(mes);
	alert->exec();
}

inline void alert(int i){
	AlertDialog* alert = new AlertDialog(QString("%1").arg(i));
	alert->exec();
}

class OptionDialog : public QDialog{
	Q_OBJECT
public:

	OptionDialog(QWidget* parent = 0)
		:QDialog(parent){
		QHBoxLayout* mainLayout = new QHBoxLayout();
		setLayout(mainLayout);

		QLabel* label = new QLabel(tr("ip address"));
		QLineEdit* lineEdit = new QLineEdit();
		label->setBuddy(lineEdit);

		QPushButton* ok = new QPushButton(tr("ok"));
		QPushButton* cancel = new QPushButton(tr("cancel"));

		mainLayout->addWidget(label);
		mainLayout->addWidget(lineEdit);
		mainLayout->addWidget(ok);
		mainLayout->addWidget(cancel);

		ok->setDefault(true);
		connect(cancel, SIGNAL(clicked()), this, SLOT(close()));
	}
};

class LimitedFileSystemModel : public QFileSystemModel{
    Q_OBJECT
public:
    LimitedFileSystemModel(QWidget* parent)
        :QFileSystemModel(parent){}

    virtual int	columnCount(const QModelIndex & parent = QModelIndex()) const{
        return 1;
    }
};

class CllickMapper : public QObject{
    Q_OBJECT
public:
    CllickMapper(QFileSystemModel* model, QObject* parent = 0)
        :QObject(parent), model_(model){}

public slots:
    void map(const QModelIndex& index){
        emit clicked(model_, index);
        emit clicked(model_->filePath(index));
    }

signals:
    void clicked(QFileSystemModel* model, const QModelIndex& index);
    void clicked(const QString& file);

private:
    QFileSystemModel* model_;
};

class PathListModel : public QAbstractListModel{
public:
};

class MainWindow : public QMainWindow{
	Q_OBJECT

public:
	MainWindow(QWidget *parent = 0);

	~MainWindow();

private:
    void createProjectView();
    void addPathView(const QString& path);
    void clearPathView();

    void createMessageView();
	void createActions();
    void createExprView();
    void createCallStackView();
    void createBreakpointView();

    //void view(const VMachinePtr& vm){
    //	callstack_->view(vm);
    //}

protected:
	void closeEvent(QCloseEvent *event);

	void setGuiEnabled(bool b);

	void setActionsEnabled(bool b);

	void setStepActionsEnabled(bool b);

    // �]�����r���[���X�V����
	void updateExprView();

    // �R�[���X�^�b�N�r���[���X�V����
	void updateCallStackView();

public slots:

    // �v���W�F�N�g��������
	void initProject();

    // �v���W�F�N�g��V�������
	void newProject();

    // �v���W�F�N�g��ۑ�
	void saveProject();

    // �v���W�F�N�g��ǂݍ���
	void loadProject();

    // ���ݕҏW���̃t�@�C����ۑ�����
    void saveFile();

    void publish();

    // �I�v�V�����_�C�A���O��\��
	void viewOption();

    // �\�[�X��\��
    void viewSource(const QString& file);

    // �]������ύX
    void changeExpr(int i, const QString& expr);

    // �u���[�N�|�C���g��ύX
    void changeBreakpoint(const QString& path, int line, bool b);

    // �u���[�N�|�C���g�̏�����ύX
    void changeBreakpointCondition(const QString& path, int line, const QString& cond);

    // �u���[�N�|�C���g�̏�������ύX
    void viewBreakpoint(const QString& path, int line);

    // �u���[�N�|�C���g�̍폜
    void eraseBreakpoint(const QString& path, int line);

    void viewPath(int n);

    void addPath(const QString& path);

    void modifiedPath();

public slots:

    // �f�o�b�O�J�n
    void run();

    void pause();

    // �X�e�b�v�I�[�o�[����
    void stepOver();

    // �X�e�b�v�C���g�D����
    void stepInto();

    // �X�e�b�v�A�E�g����
    void stepOut();

public slots:

    // �u���[�N����
	void breaked();

    // �t�@�C���v����������
    void required();

    // ���@�Ɛڑ����ꂽ
	void connected();

    // ���@�Ɛڑ����؂ꂽ
	void disconnected();
    //
	void onUpdate();

    // �R�[���X�^�b�N���ړ�����
    void moveCallStack(int n);

    // ���b�Z�[�W��\������
	void print(const QString& mes);

public slots:

    // �E�C���h�E����̍ĕ\��
    void showProjectDock(){ projDockWidget_->show(); }
	void showEvalExprDock(){ exprDockWidget_->show(); }
	void showCSDock(){ csDockWidget_->show(); }
	void showBreakpointDock(){ breakpointDockWidget_->show(); }
	void showMessageDock(){ mesDockWidget_->show(); }

	void showAboutQt(){
		QMessageBox::aboutQt(this, "Xtal Debugger");
	}

private:
    // �p�X�����ϐ��̒l���g���ĕϊ�����
    QString convertPath(const QString& path);

    // ���΃p�X���΃p�X�ɕϊ�����
    QString toXtalPath(const QString& str);

    // ��΃p�X�𑊑΃p�X�ɕϊ�����
    QString fromXtalPath(const QString& str);

    void loadProject(const QString& filename);

    MapPtr publish(const QDir& dir);

    void addPage(const QString& file);

private:
    ProjectView* projectView_;
	EvalExprView* evalexpr_;
	CallStackView* callstack_;
	CodeEditor* codeEditor_;
	BreakpointView* breakpoint_;
	QTextEdit* messages_;
	Debugger debugger_;

private:
	Document document_;
	QString projectFilename_;

private:
    QDockWidget* projDockWidget_;
    QMap<QString, QDockWidget*> projDockWidgetList_;
	QDockWidget* exprDockWidget_;
	QDockWidget* csDockWidget_;
	QDockWidget* mesDockWidget_;
	QDockWidget* breakpointDockWidget_;

private:
	QToolBar* toolBar_;
	QAction* runAction_;
    QAction* pauseAction_;
	QAction* stepIntoAction_;
	QAction* stepOverAction_;
	QAction* stepOutAction_;
	QAction* updateAction_;

	QMenu* fileMenu_;

private:
	int stoppedLine_;

	enum{
		STATE_NONE,
		STATE_REQUIRING,
		STATE_BREAKING
	};

	int state_;
};


#endif // MAINWINDOW_H
