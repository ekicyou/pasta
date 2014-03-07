#ifndef CODEEDITOR_H
#define CODEEDITOR_H

#include "xtalhighlighter.h"

class LineData{
public:
	LineData(){ breakpoint = false; }
	bool breakpoint;
};

/**
  * \brief コードエディタ部分
  */
class CodeEditorPage : public QPlainTextEdit{
	Q_OBJECT
public:
	CodeEditorPage(QWidget *parent = 0);

	void lineNumberAreaPaintEvent(QPaintEvent *event);

	int lineNumberAreaWidth();

	QTextBlock block(int line);

	void setCursorLine(int line);

	void clickEvent(QMouseEvent* event);

	void setBreakpoint(int n, bool b = true);

	void flipBreakpoint(int n);

	void save();

	int lineCount(){
		return lineData_.size();
	}

	bool breakpoint(int n){
		return lineData_[n-1].breakpoint;
	}

	const QString& sourcePath(){
		return sourcePath_;
	}

	void setSourcePath(const QString& file){
		sourcePath_ = file;
	}

	void setPlainText(const QString& text){
		text_->setPlainText(text);
		changed_ = false;
	}

protected:
	void resizeEvent(QResizeEvent *event);

private slots:

	void updateLineNumberAreaWidth(int newBlockCount);
	void highlightCurrentLine();
	void updateLineNumberArea(const QRect &, int);

	void onTextChanged();

	void onBlockCountChanged(int blockCount);

signals:

	void textChanged(CodeEditorPage* p);

	void breakpointChanged(const QString& path, int n, bool b);

private:
	QTextDocument* text_;
	QWidget *lineNumberArea;
	QString sourcePath_;
	QVector<LineData> lineData_;
	int blockCount_;
	bool changed_;
	XtalHighlighter* highlighter_;
};

class CodeEditor : public QTabWidget{
   Q_OBJECT
public:

	CodeEditor(QWidget *parent = 0);

	void init(){
		this->clear();
	}

public slots:

	void closePage(int index);

	void onTextChanged(CodeEditorPage* p);

	void onBreakpointChanged(const QString& path, int n, bool b);

signals:

	void breakpointChanged(const QString& path, int n, bool b);

public:

	CodeEditorPage* page(int index);

	int index(CodeEditorPage* page);

	CodeEditorPage* findPage(const QString& path);

	void setBreakpoint(const QString& path, int line, bool b = true){
		if(CodeEditorPage* page = findPage(path)){
			page->setBreakpoint(line, b);
		}
	}

    void addPage(const QString& path);

	CodeEditorPage* currentPage();

	bool setPos(const QString& path, int lineno);

	void setCursorLine(int n);

	bool setSourceFile(const QString& path);

	void clearCursorLine();

	void saveAll();

    void saveFile();
};

class LineNumberArea : public QWidget{
public:
	LineNumberArea(CodeEditorPage *editor) : QWidget(editor) {
		codeEditor = editor;
	}

	QSize sizeHint() const {
		return QSize(codeEditor->lineNumberAreaWidth(), 0);
	}

protected:

	void paintEvent(QPaintEvent *event) {
		codeEditor->lineNumberAreaPaintEvent(event);
	}

	void mouseReleaseEvent(QMouseEvent* event){
		codeEditor->clickEvent(event);
	}

private:
	CodeEditorPage *codeEditor;
};

#endif // CODEEDITOR_H
