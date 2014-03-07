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

    // 評価式ビューを更新する
	void updateExprView();

    // コールスタックビューを更新する
	void updateCallStackView();

public slots:

    // プロジェクトを初期化
	void initProject();

    // プロジェクトを新しく作る
	void newProject();

    // プロジェクトを保存
	void saveProject();

    // プロジェクトを読み込む
	void loadProject();

    // 現在編集中のファイルを保存する
    void saveFile();

    void publish();

    // オプションダイアログを表示
	void viewOption();

    // ソースを表示
    void viewSource(const QString& file);

    // 評価式を変更
    void changeExpr(int i, const QString& expr);

    // ブレークポイントを変更
    void changeBreakpoint(const QString& path, int line, bool b);

    // ブレークポイントの条件を変更
    void changeBreakpointCondition(const QString& path, int line, const QString& cond);

    // ブレークポイントの条件式を変更
    void viewBreakpoint(const QString& path, int line);

    // ブレークポイントの削除
    void eraseBreakpoint(const QString& path, int line);

    void viewPath(int n);

    void addPath(const QString& path);

    void modifiedPath();

public slots:

    // デバッグ開始
    void run();

    void pause();

    // ステップオーバーする
    void stepOver();

    // ステップイントゥする
    void stepInto();

    // ステップアウトする
    void stepOut();

public slots:

    // ブレークした
	void breaked();

    // ファイル要求があった
    void required();

    // 実機と接続された
	void connected();

    // 実機と接続が切れた
	void disconnected();
    //
	void onUpdate();

    // コールスタックを移動する
    void moveCallStack(int n);

    // メッセージを表示する
	void print(const QString& mes);

public slots:

    // ウインドウ周りの再表示
    void showProjectDock(){ projDockWidget_->show(); }
	void showEvalExprDock(){ exprDockWidget_->show(); }
	void showCSDock(){ csDockWidget_->show(); }
	void showBreakpointDock(){ breakpointDockWidget_->show(); }
	void showMessageDock(){ mesDockWidget_->show(); }

	void showAboutQt(){
		QMessageBox::aboutQt(this, "Xtal Debugger");
	}

private:
    // パスを環境変数の値を使って変換する
    QString convertPath(const QString& path);

    // 相対パスを絶対パスに変換する
    QString toXtalPath(const QString& str);

    // 絶対パスを相対パスに変換する
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
