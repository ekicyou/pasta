#include "codeeditor.h"

CodeEditorPage::CodeEditorPage(QWidget *parent)
:QPlainTextEdit(parent){
	blockCount_ = 0;

	lineNumberArea = new LineNumberArea(this);

	connect(this, SIGNAL(blockCountChanged(int)), this, SLOT(onBlockCountChanged(int)));
	connect(this, SIGNAL(blockCountChanged(int)), this, SLOT(updateLineNumberAreaWidth(int)));
	connect(this, SIGNAL(updateRequest(const QRect &, int)), this, SLOT(updateLineNumberArea(const QRect &, int)));
	//connect(this, SIGNAL(cursorPositionChanged()), this, SLOT(highlightCurrentLine()));

	updateLineNumberAreaWidth(0);
	highlightCurrentLine();

	lineData_.clear();

	text_ = new QTextDocument();
	highlighter_ = new XtalHighlighter(text_);
	text_->setDocumentLayout(new QPlainTextDocumentLayout(text_));
	setDocument(text_);
	setLineWrapMode(QPlainTextEdit::NoWrap);

	QFont font = text_->defaultFont();
	font.setFamily("Courier");
	font.setFixedPitch(true);
	font.setKerning(false);
	font.setPointSize(10);
	text_->setDefaultFont(font);

	setTabStopWidth(20);

	connect(this, SIGNAL(textChanged()), this, SLOT(onTextChanged()));
	changed_ = false;

}

int CodeEditorPage::lineNumberAreaWidth(){
	int space = 3 + fontMetrics().width(QLatin1Char('9')) * 5;
	return space;
}

void CodeEditorPage::updateLineNumberAreaWidth(int /* newBlockCount */){
	setViewportMargins(lineNumberAreaWidth(), 0, 0, 0);
}

void CodeEditorPage::updateLineNumberArea(const QRect &rect, int dy){
	if(dy){
		lineNumberArea->scroll(0, dy);
	}
	else{
		lineNumberArea->update(0, rect.y(), lineNumberArea->width(), rect.height());
	}

	if(rect.contains(viewport()->rect())){
		updateLineNumberAreaWidth(0);
	}
}

void CodeEditorPage::resizeEvent(QResizeEvent *e){
	QPlainTextEdit::resizeEvent(e);

	QRect cr = contentsRect();
	lineNumberArea->setGeometry(QRect(cr.left(), cr.top(), lineNumberAreaWidth(), cr.height()));
}

void CodeEditorPage::highlightCurrentLine(){
	QList<QTextEdit::ExtraSelection> extraSelections;

	if(!isReadOnly()){
		QTextEdit::ExtraSelection selection;

		QColor lineColor = QColor(Qt::yellow).lighter(120);

		selection.format.setBackground(lineColor);
		selection.format.setProperty(QTextFormat::FullWidthSelection, true);
		selection.cursor = textCursor();
		selection.cursor.clearSelection();
		extraSelections.append(selection);
	}

	setExtraSelections(extraSelections);
}

void CodeEditorPage::lineNumberAreaPaintEvent(QPaintEvent *event){
	QPainter painter(lineNumberArea);
	painter.fillRect(event->rect(), Qt::lightGray);

	QTextBlock block = firstVisibleBlock();
	int blockNumber = block.blockNumber();
	int top = (int) blockBoundingGeometry(block).translated(contentOffset()).top();
	int bottom = top + (int) blockBoundingRect(block).height();

	while(block.isValid() && top <= event->rect().bottom()){
		if(block.isVisible() && bottom >= event->rect().top()){
			QString number = QString::number(blockNumber + 1);
			painter.setPen(Qt::black);
			painter.drawText(0, top, lineNumberArea->width(), fontMetrics().height(), Qt::AlignRight, number);

			if(block.blockNumber()<lineData_.size()){
				LineData& data = lineData_[block.blockNumber()];
				if(data.breakpoint){
					painter.setPen(Qt::red);
					painter.setBrush(Qt::red);
					painter.drawEllipse(QPoint(4, (top+bottom)/2-2), 4, 4);
				}
			}
		}

		block = block.next();
		top = bottom;
		bottom = top + (int) blockBoundingRect(block).height();
		++blockNumber;
	}
}

void CodeEditorPage::onTextChanged(){
	if(!changed_){
		emit textChanged(this);
		changed_ = true;
	}
}

QTextBlock CodeEditorPage::block(int line){
	QTextBlock block = firstVisibleBlock();
	while(block.blockNumber()+1<line && block.isValid()){
		block = block.next();
	}
	return block;
}

void CodeEditorPage::setCursorLine(int line){
	QTextBlock b = text_->findBlockByLineNumber(line-1);
	setTextCursor(QTextCursor(b));
	highlightCurrentLine();
}

void CodeEditorPage::clickEvent(QMouseEvent* event){
	QTextBlock block = firstVisibleBlock();
	int blockNumber = block.blockNumber();
	int y = this->mapFromGlobal(event->globalPos()).y();
	int top = (int) blockBoundingGeometry(block).translated(contentOffset()).top();
	int bottom = top + (int) blockBoundingRect(block).height();

	while(block.isValid()) {
		if(block.isVisible() && top<y && y<bottom) {
			flipBreakpoint(block.blockNumber()+1);
			return;
		}

		block = block.next();
		top = bottom;
		bottom = top + (int) blockBoundingRect(block).height();
		++blockNumber;
	}
}

void CodeEditorPage::setBreakpoint(int n, bool b){
	lineData_[n-1].breakpoint = b;
	this->update();
	//emit breakpointChanged(sourcePath_, n, lineData_[n-1].breakpoint);
}

void CodeEditorPage::flipBreakpoint(int n){
	lineData_[n-1].breakpoint = !lineData_[n-1].breakpoint;
	this->update();
	emit breakpointChanged(sourcePath_, n, lineData_[n-1].breakpoint);
}

void CodeEditorPage::save(){
	QFile data(sourcePath_);
	if(data.open(QFile::WriteOnly | QFile::Truncate)){
		QTextStream out(&data);
		out << text_->toPlainText();
		changed_ = false;
	}
}

void CodeEditorPage::onBlockCountChanged(int blockCount){
	/*
	QList<QTextEdit::ExtraSelection> sel = this->extraSelections();
	if(lineData_.size()!=0){
		if(sel.size()>0){
			QTextCursor cur = textCursor();
			int diff = blockCount - lineData_.size();
			int n = cur.block().blockNumber();
			if(diff>0){
				lineData_.insert(lineData_.begin()+n+1-diff, diff, LineData());
			}
			else{
				lineData_.erase(lineData_.begin()+n+1, lineData_.begin()+n+1-diff);
			}

		}
	}
	*/

	lineData_.resize(blockCount);
}

//////////////////////

CodeEditor::CodeEditor(QWidget *parent)
	:QTabWidget(parent){
	CodeEditorPage* page = new CodeEditorPage();
	setTabsClosable(true);
	page->setEnabled(false);

	connect(this, SIGNAL(tabCloseRequested(int)), this, SLOT(closePage(int)));
}

void CodeEditor::closePage(int index){
    this->removeTab(index);
}

void CodeEditor::onTextChanged(CodeEditorPage* p){
	int i = index(p);
	this->setTabText(i, "*" + this->tabText(i));
}

void CodeEditor::onBreakpointChanged(const QString& path, int n, bool b){
	emit breakpointChanged(path, n, b);
}

void CodeEditor::addPage(const QString& path){
    if(CodeEditorPage* p = findPage(path)){
		this->setCurrentWidget(p);
		return;
	}

	CodeEditorPage* page = new CodeEditorPage();
	page->setSourcePath(path);
    addTab(page, QFileInfo(path).fileName());
	this->setCurrentWidget(page);

	QFile f(path);
	if(f.open(QIODevice::ReadOnly | QIODevice::Text)){
		QTextStream ts(&f);
		page->setPlainText(ts.readAll());
	}

	connect(page, SIGNAL(textChanged(CodeEditorPage*)), this, SLOT(onTextChanged(CodeEditorPage*)));
	connect(page, SIGNAL(breakpointChanged(const QString&,int,bool)), this, SLOT(onBreakpointChanged(const QString&,int,bool)));
}

CodeEditorPage* CodeEditor::page(int index){
	return (CodeEditorPage*)QTabWidget::widget(index);
}

CodeEditorPage* CodeEditor::currentPage(){
	return (CodeEditorPage*)currentWidget();
}

bool CodeEditor::setPos(const QString& filename, int lineno){
	if(setSourceFile(filename)){
		setCursorLine(lineno);
		return true;
	}
	return false;
}

void CodeEditor::setCursorLine(int n){
	if(currentPage()){
		currentPage()->setCursorLine(n);
	}
}

CodeEditorPage* CodeEditor::findPage(const QString& path){
	for(int i=0; i<this->count(); ++i){
		CodeEditorPage* p = (CodeEditorPage*)widget(i);
		if(p->sourcePath()==path){
			return p;
		}
	}
	return 0;
}

int CodeEditor::index(CodeEditorPage* a){
	for(int i=0; i<this->count(); ++i){
		CodeEditorPage* p = (CodeEditorPage*)widget(i);
		if(p==a){
			return i;
		}
	}
	return -1;
}

bool CodeEditor::setSourceFile(const QString& path){
	if(CodeEditorPage* p = findPage(path)){
		this->setCurrentWidget(p);
		return true;
	}
	else{
		addPage(path);
		return true;
	}
}

void CodeEditor::clearCursorLine(){
	for(int i=0; i<this->count(); ++i){
		CodeEditorPage* p = (CodeEditorPage*)widget(i);
		p->setExtraSelections(QList<QTextEdit::ExtraSelection>());
	}
}

void CodeEditor::saveAll(){
	for(int i=0; i<this->count(); ++i){
		CodeEditorPage* p = (CodeEditorPage*)widget(i);
		p->save();

		if(this->tabText(i).left(1)=="*"){
			this->setTabText(i, this->tabText(i).right(this->tabText(i).size()-1));
		}
	}
}

void CodeEditor::saveFile(){
    if(currentPage()){
        CodeEditorPage* p = currentPage();
        p->save();
        int i = index(p);
        if(this->tabText(i).left(1)=="*"){
            this->setTabText(i, this->tabText(i).right(this->tabText(i).size()-1));
        }
    }
}

