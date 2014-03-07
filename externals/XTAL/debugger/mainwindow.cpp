#include "mainwindow.h"
#include "ui_mainwindow.h"

/**
  * \brief 相対パスに直す
  */
QString makeRelative(QString absolutePath, QString relativeTo, bool bIsFile = false){
    //qDebug() << "absolutePath:" << absolutePath;
    //qDebug() << "relativeTo:" << relativeTo;
    QStringList absoluteDirectories = absolutePath.split('/', QString::SkipEmptyParts);
    QStringList relativeDirectories = relativeTo.split('/', QString::SkipEmptyParts);

    int length = absoluteDirectories.count() < relativeDirectories.count() ? absoluteDirectories.count() : relativeDirectories.count();

    int lastCommonRoot = -1;
    int index;
    for(index = 0; index < length; index++){
        if(absoluteDirectories[index] == relativeDirectories[index]){
            lastCommonRoot = index;
        }
        else{
            break;
        }
    }

    if(lastCommonRoot == -1){
        return relativeTo;
    }

    QString relativePath;

    for(index = lastCommonRoot + 1; index < absoluteDirectories.count() - (bIsFile?1:0); index++){
        if(absoluteDirectories[index].length() > 0){
            relativePath.append("../");
        }
    }

    if(lastCommonRoot + 1 < relativeDirectories.count()){
        for(index = lastCommonRoot + 1; index < relativeDirectories.count() - 1; index++){
            relativePath.append(relativeDirectories[index]).append("/");
        }

        relativePath.append(relativeDirectories[relativeDirectories.count() - 1]);
    }
    else{
        if(relativePath[relativePath.size()-1]=='/'){
            relativePath.resize(relativePath.size()-1);
        }
    }

    //qDebug() << "ret:" << relativePath;
    return relativePath;
}

MainWindow::MainWindow(QWidget *parent)
	: QMainWindow(parent){

	codeEditor_ = new CodeEditor(this);
    setCentralWidget(codeEditor_);
    connect(codeEditor_, SIGNAL(breakpointChanged(const QString&,int,bool)), this, SLOT(changeBreakpoint(const QString&,int,bool)));

	state_ = STATE_NONE;

    createProjectView();
    createExprView();
    createCallStackView();
    createBreakpointView();
    createMessageView();

	createActions();

	//evalexpr_->addExpr(0, "filelocal");

	QSettings settings("xtal", "debugger");
	QByteArray geoData = settings.value("geometry").toByteArray();
	QByteArray layoutData = settings.value("windowState").toByteArray();

    projectFilename_ = settings.value("lastProject").toString();

	if(geoData.size() > 1){
		restoreGeometry(geoData);
	}

	if(layoutData.size() > 1){
		restoreState(layoutData);
	}

    if(projectFilename_.size()!=0 && QFileInfo(projectFilename_).exists()){
        loadProject(projectFilename_);
    }
}

MainWindow::~MainWindow(){
}

void MainWindow::closeEvent(QCloseEvent *event){
	QSettings settings("xtal", "debugger");
	settings.setValue("geometry", saveGeometry());
	settings.setValue("windowState", saveState());
    settings.setValue("lastProject", projectFilename_);

    if(projectFilename_.size()!=0 && QFileInfo(projectFilename_).exists()){
        QMessageBox mb(this);
        mb.setText("close");
        mb.setInformativeText("Do you overwrite the file under edit?");
        mb.setStandardButtons(QMessageBox::Yes | QMessageBox::No | QMessageBox::Cancel);
        mb.setDefaultButton(QMessageBox::Cancel);
        int ret = mb.exec();

        if(ret==QMessageBox::Yes){
            saveProject();
        }
        else if(ret==QMessageBox::No){
        }
        else{
             event->ignore();
        }
    }
}

void MainWindow::createActions(){
	fileMenu_ = this->menuBar()->addMenu(tr("&File"));
	QAction* newProjectAction = fileMenu_->addAction(tr("&New Project"));
	QAction* loadProjectAction = fileMenu_->addAction(tr("&Load Project"));
	QAction* saveProjectAction = fileMenu_->addAction(tr("&Save Project"));
    fileMenu_->addSeparator();

    QAction* saveFileAction = fileMenu_->addAction(tr("&Save File"));
    saveFileAction->setShortcut(QKeySequence::Save);

    fileMenu_->addSeparator();
    QAction* publishAction = fileMenu_->addAction(tr("&Publish"));

	//fileMenu_->addSeparator();
    //QAction* addFileAction = fileMenu_->addAction(tr("add file"))
    //QMenu* optionMenu_ = this->menuBar()->addMenu(tr("&Tool"));
    //QAction* optionAction = optionMenu_->addAction(tr("&Option"));

	toolBar_ = this->addToolBar(tr("debug"));
	toolBar_->setObjectName("debug");
	runAction_ = toolBar_->addAction(QIcon("data/run.png"), tr("run"));
    pauseAction_ = toolBar_->addAction(QIcon("data/pause.png"), tr("pause"));
    stepIntoAction_ = toolBar_->addAction(QIcon("data/step_into.png"), tr("step into"));
	stepOverAction_ = toolBar_->addAction(QIcon("data/step_over.png"), tr("step over"));
	stepOutAction_ = toolBar_->addAction(QIcon("data/step_out.png"), tr("step out"));
	updateAction_ = toolBar_->addAction(QIcon("data/update.png"), tr("update"));

	connect(newProjectAction, SIGNAL(triggered()), this, SLOT(newProject()));
	connect(saveProjectAction, SIGNAL(triggered()), this, SLOT(saveProject()));
	connect(loadProjectAction, SIGNAL(triggered()), this, SLOT(loadProject()));
    connect(saveFileAction, SIGNAL(triggered()), this, SLOT(saveFile()));
    connect(publishAction, SIGNAL(triggered()), this, SLOT(publish()));
//	connect(addFileAction, SIGNAL(triggered()), this, SLOT(addFile()));
    //connect(optionAction, SIGNAL(triggered()), this, SLOT(viewOption()));

	connect(runAction_, SIGNAL(triggered()), this, SLOT(run()));
    connect(pauseAction_, SIGNAL(triggered()), this, SLOT(pause()));
    connect(stepIntoAction_, SIGNAL(triggered()), this, SLOT(stepInto()));
	connect(stepOverAction_, SIGNAL(triggered()), this, SLOT(stepOver()));
	connect(stepOutAction_, SIGNAL(triggered()), this, SLOT(stepOut()));
	connect(updateAction_, SIGNAL(triggered()), this, SLOT(onUpdate()));

	QMenu* windowMenu = this->menuBar()->addMenu(tr("&Window"));
	connect(windowMenu->addAction(tr("Project")), SIGNAL(triggered()), this, SLOT(showProjectDock()));
	connect(windowMenu->addAction(tr("Expr")), SIGNAL(triggered()), this, SLOT(showEvalExprDock()));
	connect(windowMenu->addAction(tr("CallStack")), SIGNAL(triggered()), this, SLOT(showCSDock()));
	connect(windowMenu->addAction(tr("Breakpoint")), SIGNAL(triggered()), this, SLOT(showBreakpointDock()));
	connect(windowMenu->addAction(tr("Message")), SIGNAL(triggered()), this, SLOT(showMessageDock()));

	QMenu* helpMenu_ = this->menuBar()->addMenu(tr("&Help"));
	QAction* aboutqtAction = helpMenu_->addAction(tr("About Qt"));
	connect(aboutqtAction, SIGNAL(triggered()), this, SLOT(showAboutQt()));

	connect(&debugger_, SIGNAL(breaked()), this, SLOT(breaked()));
	connect(&debugger_, SIGNAL(required()), this, SLOT(required()));
	connect(&debugger_, SIGNAL(connected()), this, SLOT(connected()));
	connect(&debugger_, SIGNAL(disconnected()), this, SLOT(disconnected()));

	setActionsEnabled(false);
	setGuiEnabled(false);
 }

void MainWindow::setGuiEnabled(bool b){
    projectView_->setEnabled(b);
	evalexpr_->setEnabled(b);
	callstack_->setEnabled(b);
	codeEditor_->setEnabled(b);
	breakpoint_->setEnabled(b);
	messages_->setEnabled(b);

    foreach(QDockWidget* p, projDockWidgetList_){
        p->setEnabled(b);
    }
}

void MainWindow::setActionsEnabled(bool b){
	runAction_->setEnabled(b);
    pauseAction_->setEnabled(b);
	stepIntoAction_->setEnabled(b);
	stepOverAction_->setEnabled(b);
	stepOutAction_->setEnabled(b);
	updateAction_->setEnabled(b);
}

void MainWindow::setStepActionsEnabled(bool b){
	runAction_->setEnabled(b);
    pauseAction_->setEnabled(!b);
	stepIntoAction_->setEnabled(b);
	stepOverAction_->setEnabled(b);
	stepOutAction_->setEnabled(b);
}

void MainWindow::updateExprView(){
	for(int i=0; i<evalexpr_->exprCount(); ++i){
        evalexpr_->setExprResult(i, debugger_.eval_expr_result(qstr2xstr(evalexpr_->expr(i))));
	}
}

void MainWindow::updateCallStackView(){
	callstack_->clear();
    for(int i=0; i<debugger_.call_stack_size(); ++i){
		callstack_->set(i,
            debugger_.call_stack_fun_name(i),
            debugger_.call_stack_file_name(i),
            debugger_.call_stack_lineno(i));
	}
	callstack_->setLevel(debugger_.level());
}

void MainWindow::modifiedPath(){
    enum{
        NONE,
        ADD,
        ERASE,
        NOTCHANGE
    };

    QMap<QString, int> map;
    for(int i=0; i<document_.pathCount(); ++i){
        map[document_.path(i)] = ADD; // ドキュメントにあるものは1を立てる
    }

    foreach(QString key, projDockWidgetList_.keys()){
        if(projDockWidgetList_[key]!=0){
            if(map[key]==ADD){
                map[key] = NOTCHANGE;
            }
            else{
                map[key] = ERASE;
            }
        }
    }

    foreach(QString key, map.keys()){
        int n = map[key];
        if(n==ADD){
            addPathView(key);
        }
        else if(n==ERASE){
            delete projDockWidgetList_[key];
            projDockWidgetList_[key] = 0;
        }
    }

}

void MainWindow::addPathView(const QString& path){    
    if(!projDockWidgetList_[path]){
        QString path2 = convertPath(path);
        if(QDir(path2).exists()){
            QTreeView* tree_view = new QTreeView(this);
            QFileSystemModel* dir_model = new LimitedFileSystemModel(tree_view);

            dir_model->setRootPath(path2);
            dir_model->setFilter(QDir::NoDotAndDotDot | QDir::AllDirs | QDir::AllEntries);
            dir_model->setNameFilters(QStringList("*.xtal"));
            dir_model->setNameFilterDisables(false);
            tree_view->setModel(dir_model);
            tree_view->setRootIndex(dir_model->index(path2));
            tree_view->setHeaderHidden(true);

            QDockWidget* dock = new QDockWidget("path: \""+path+"\"", this);
            dock->setObjectName("path: \""+path+"\"");
            dock->setWidget(tree_view);
            dock->setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
            addDockWidget(Qt::LeftDockWidgetArea, dock);

            CllickMapper* cm = new CllickMapper(dir_model, this);
            connect(tree_view, SIGNAL(clicked(const QModelIndex&)), cm, SLOT(map(const QModelIndex&)));
            connect(cm, SIGNAL(clicked(const QString&)), this, SLOT(viewSource(const QString&)));

            projDockWidgetList_[path] = dock;
        }
    }

    fromXtalPath("first");
}

void MainWindow::clearPathView(){
    qDeleteAll(projDockWidgetList_);
    projDockWidgetList_.clear();
}

void MainWindow::createProjectView(){
    projectView_ = new ProjectView(&document_, this);
    projDockWidget_ = new QDockWidget(tr("Project"), this);
    projDockWidget_->setObjectName("Project");
    projDockWidget_->setWidget(projectView_);
    projDockWidget_->setAllowedAreas(Qt::BottomDockWidgetArea);
    addDockWidget(Qt::BottomDockWidgetArea, projDockWidget_);

    connect(projectView_, SIGNAL(pathAdded(const QString&)), this, SLOT(addPath(const QString&)));
    connect(projectView_, SIGNAL(pathSelected(int)), this, SLOT(viewPath(int)));
    connect(&document_, SIGNAL(changed()), this, SLOT(modifiedPath()));
}

void MainWindow::createMessageView(){
    messages_ = new QTextEdit(this);
    mesDockWidget_ = new QDockWidget(tr("Message"), this);
    mesDockWidget_->setObjectName("Message");
    mesDockWidget_->setWidget(messages_);
    mesDockWidget_->setAllowedAreas(Qt::BottomDockWidgetArea);
    addDockWidget(Qt::BottomDockWidgetArea, mesDockWidget_);
}

void MainWindow::createExprView(){
    evalexpr_ = new EvalExprView(this);
    exprDockWidget_ = new QDockWidget(tr("Expr"), this);
    exprDockWidget_->setObjectName("Expr");
    exprDockWidget_->setWidget(evalexpr_);
    exprDockWidget_->setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
    addDockWidget(Qt::RightDockWidgetArea, exprDockWidget_);

    connect(evalexpr_, SIGNAL(exprChanged(int, const QString&)), this, SLOT(changeExpr(int, const QString&)));
}

void MainWindow::createCallStackView(){
    callstack_ = new CallStackView(this);
    csDockWidget_ = new QDockWidget(tr("Call Stack"), this);
    csDockWidget_->setObjectName("Call Stack");
    csDockWidget_->setWidget(callstack_);
    csDockWidget_->setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
    addDockWidget(Qt::RightDockWidgetArea, csDockWidget_);

    connect(callstack_, SIGNAL(moveCallStack(int)), this, SLOT(moveCallStack(int)));
}

void MainWindow::createBreakpointView(){
    breakpoint_ = new BreakpointView(this);
    breakpointDockWidget_ = new QDockWidget(tr("Breakpoint"), this);
    breakpointDockWidget_->setObjectName("Breakpoint");
    breakpointDockWidget_->setWidget(breakpoint_);
    breakpointDockWidget_->setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
    addDockWidget(Qt::RightDockWidgetArea, breakpointDockWidget_);

    connect(breakpoint_, SIGNAL(breakpointConditionChanged(const QString&,int,const QString&)), this, SLOT(changeBreakpointCondition(const QString&,int,const QString&)));
    connect(breakpoint_, SIGNAL(viewBreakpoint(const QString&,int)), this, SLOT(viewBreakpoint(const QString&,int)));
}

void MainWindow::initProject(){
	document_.init();

	evalexpr_->init();
	callstack_->init();
	codeEditor_->init();
	breakpoint_->init();

    clearPathView();

	setActionsEnabled(false);
	setGuiEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::newProject(){
    QString str = QFileDialog::getSaveFileName(this, tr("New Project"), QString(), tr("xtal project(*.xproj)"));
    if(str.length()==0){
        return;
    }

    initProject();

    projectFilename_ = str;
	if(document_.save(projectFilename_)){
		QDir::setCurrent(QFileInfo(projectFilename_).dir().absolutePath());
		setGuiEnabled(true);
		this->setWindowTitle(projectFilename_ + " - xtal debugger");
	}
}

void MainWindow::saveProject(){
	document_.save(projectFilename_);
	codeEditor_->saveAll();
}

void MainWindow::loadProject(){
    QString str = QFileDialog::getOpenFileName(this, tr("Load Project"), QString(), tr("xtal project(*.xproj)"));
    if(str.length()==0){
        return;
    }

    loadProject(str);
}

void MainWindow::loadProject(const QString& filename){
    initProject();

    projectFilename_ = filename;
    if(document_.load(projectFilename_)){
        // プロジェクトのある位置をカレントディレクトリとする
        QDir::setCurrent(QFileInfo(projectFilename_).dir().absolutePath());

        for(int i=0; i<document_.pathCount(); ++i){
            QString path = document_.path(i);
            addPathView(path);
        }

        for(int i=0; i<document_.breakpointCount(); ++i){
            BreakpointInfo f = document_.breakpoint(i);

            breakpoint_->add(f.file, f.lineno, f.condition);
            codeEditor_->setBreakpoint(f.file, f.lineno);
        }

        for(int i=0; i<document_.evalExprCount(); ++i){
            QString expr = document_.evalExpr(i);
            evalexpr_->setExpr(i, expr);
            debugger_.add_eval_expr(qstr2xstr(expr));
        }

        setGuiEnabled(true);
        this->setWindowTitle(projectFilename_ + " - xtal debugger");
        this->setWindowTitle(QDir::current().absolutePath());
    }
}

void MainWindow::saveFile(){
    codeEditor_->saveFile();
}

void MainWindow::publish(){
    xtal::debug::disable();
    xtal::debug::disable_debug_compile();

    QString str = QFileDialog::getSaveFileName(this, tr("PublishedFile"), QString(), "");
    if(str.length()==0){
        return;
    }

    MapPtr code_map = xnew<Map>();

    for(int i=0; i<document_.pathCount(); ++i){
        MapPtr ret = publish(QDir(document_.path(i)));

        ret->append(code_map);
        code_map = ret;
    }

    StreamPtr fs = xnew<CompressEncoder>(xnew<FileStream>(qstr2xstr(str), "w"));
    fs->serialize(code_map);
    fs->close();

    xtal::debug::enable();
    xtal::debug::enable_debug_compile();
}

MapPtr MainWindow::publish(const QDir& dir){
    print("\n\nPublishing...");

    MapPtr code_map = xnew<Map>();

    foreach(QString file, dir.entryList()){
        if(file[0]=='.'){
            continue;
        }

        QString path = dir.absolutePath() + "/" + file;

        QDir d(path);
        if(d.exists()){
            code_map->set_at(qstr2xstr(path), publish(d));
        }
        else{
            QFileInfo fi(path);
            if(fi.suffix()=="xtal"){
                print(QString("compile %1").arg(path));
                if(CodePtr code = compile_file(qstr2xstr(path))){
                    code_map->set_at(qstr2xstr(fi.baseName()), code);
                }
                else{
                    XTAL_CATCH_EXCEPT(e){
                        print(xstr2qstr(e->to_s()));
                    }
                }
            }
        }
    }

    print("- Published");
    return code_map;
}

void MainWindow::viewSource(const QString& file){
    QString path = fromXtalPath(toXtalPath(file));
    addPage(path);
}

void MainWindow::addPage(const QString& file){
    codeEditor_->addPage(file);

    for(int i=0; i<document_.breakpointCount(); ++i){
        BreakpointInfo bi = document_.breakpoint(i);
        if(bi.file==file){
             codeEditor_->setBreakpoint(bi.file, bi.lineno);
        }
    }
}

void MainWindow::viewOption(){
	OptionDialog* dialog = new OptionDialog(this);
	dialog->exec();
}

void MainWindow::changeExpr(int i, const QString& expr){
    if(i<document_.evalExprCount()){
        debugger_.remove_eval_expr(qstr2xstr(document_.evalExpr(i)));
    }

    document_.setEvalExpr(i, expr);
    debugger_.add_eval_expr(qstr2xstr(expr));
	if(debugger_.isConnected()){
		if(state_==STATE_BREAKING){
            debugger_.nostep();
		}
	}
}

QString MainWindow::convertPath(const QString& path){
    QString file = path;
    QStringList env = QProcess::systemEnvironment();
    QRegExp envreg("(\\w+)=(.+)");

    foreach(QString str, env){
        int pos = envreg.indexIn(str);
        if(pos > -1){
            QString key = envreg.cap(1);
            QString value = envreg.cap(2);
            QRegExp keyreg(QString("\\$\\(") + key + "\\)");
            file.replace(keyreg, value);
        }
    }

    return file;
}

QString MainWindow::toXtalPath(const QString& str){
    for(int i=0; i<document_.pathCount(); ++i){
        QString path = convertPath(document_.path(i));
        QString relative = makeRelative(QFileInfo(path).absoluteFilePath(), str, true); // 相対パスに直す
        QFileInfo fi(path+"/"+relative);

        if(fi.exists()){
            QFileInfo rfi(relative);
            QString folder = makeRelative(path, rfi.path());
            if(folder.isEmpty() || (folder.size()==1 && folder==".")){
                return rfi.baseName();
            }
            else{
                return folder + "/" + rfi.baseName();
            }
        }
    }
    return str;
}

QString MainWindow::fromXtalPath(const QString& str){
    for(int i=0; i<document_.pathCount(); ++i){
        QString path = convertPath(document_.path(i));
        QString fname = path+"/"+str+".xtal";
        if(QFileInfo(fname).exists()){
            return fname;
        }
    }
    return str;
}

void MainWindow::breaked(){
    QString file = xstr2qstr(debugger_.call_stack_file_name());
    if(QFileInfo(file).exists()){
        addPage(file);
        codeEditor_->setPos(file, debugger_.call_stack_lineno());
    }

	updateExprView();
	updateCallStackView();

	state_ = STATE_BREAKING;
	setStepActionsEnabled(true);

	activateWindow();
	raise();
}

void MainWindow::required(){
    print(QString("Required %1 : %2")
          .arg(xstr2qstr(debugger_.required_file()))
          .arg(fromXtalPath(xstr2qstr(debugger_.required_file()))));

    QString file = fromXtalPath(xstr2qstr(debugger_.required_file()));

    if(QFileInfo(file).exists()){
        if(CodeEditorPage* p = codeEditor_->findPage(file)){
            if(CodePtr code = compile(qstr2xstr(p->toPlainText()))){
                qDebug() << "plane code->set_source_file_name " << file;
                code->set_source_file_name(qstr2xstr(file));
                debugger_.required_source(code);
                debugger_.start();
				return;
			}
			else{
				XTAL_CATCH_EXCEPT(e){
					print(e->to_s()->c_str());
                    print("コンパイルエラーが発生しました。修正してUpdateボタンを押してください");
				}
			}
		}
        else if(CodePtr code = compile_file(qstr2xstr(file))){
            qDebug() << "code->set_source_file_name " << file;
            code->set_source_file_name(qstr2xstr(file));
            debugger_.required_source(code);
            debugger_.start();
			return;
		}
		else{
			XTAL_CATCH_EXCEPT(e){
				print(e->to_s()->c_str());
				print("コンパイルエラーが発生しました。修正してUpdateボタンを押してください");
			}
		}
	}
	else{
        debugger_.required_source(null);
        print(QString("Not found %1").arg(xstr2qstr(debugger_.required_file())));
		return;
    }

	state_ = STATE_REQUIRING;
	setStepActionsEnabled(false);
}

void MainWindow::connected(){
	setActionsEnabled(true);
	setStepActionsEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::disconnected(){
	setActionsEnabled(false);
	codeEditor_->clearCursorLine();
}

void MainWindow::run(){
	codeEditor_->clearCursorLine();
	debugger_.run();
	setStepActionsEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::pause(){
    debugger_.pause();
}

void MainWindow::stepOver(){
	codeEditor_->clearCursorLine();
    debugger_.step_over();
	setStepActionsEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::stepInto(){
	codeEditor_->clearCursorLine();
    debugger_.step_into();
	setStepActionsEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::stepOut(){
	codeEditor_->clearCursorLine();
    debugger_.step_out();
	setStepActionsEnabled(false);
	state_ = STATE_NONE;
}

void MainWindow::onUpdate(){
	if(state_==STATE_REQUIRING){
		state_ = STATE_NONE;
		setStepActionsEnabled(true);
		required();
	}
    else{
		if(CodeEditorPage* p=codeEditor_->currentPage()){
            if(CodePtr code = compile(qstr2xstr(p->toPlainText()))){
                code->set_source_file_name(qstr2xstr(p->sourcePath()));
                debugger_.update_source(code);
				return;
			}
			else{
				XTAL_CATCH_EXCEPT(e){
					print(e->to_s()->c_str());
				}
			}
		}
    }
}

void MainWindow::changeBreakpoint(const QString& path, int line, bool b){
    if(b){
        document_.addBreakpoint(path, line);

        QString cond = document_.breakpointCondition(path, line);
        breakpoint_->add(path, line, cond);
        qDebug() << "changeBreakpoint " << path;
        debugger_.add_breakpoint(qstr2xstr(path), line, qstr2xstr(cond));
    }
    else{
        breakpoint_->remove(path, line);
        document_.removeBreakpoint(path, line);
        debugger_.remove_breakpoint(qstr2xstr(path), line);
    }
}

void MainWindow::changeBreakpointCondition(const QString& path, int line, const QString& cond){
    debugger_.add_breakpoint(qstr2xstr(path), line, qstr2xstr(cond));
    document_.addBreakpoint(path, line, cond);
}

void MainWindow::viewBreakpoint(const QString& path, int line){
	codeEditor_->setPos(path, line);
}

void MainWindow::eraseBreakpoint(const QString& path, int line){
    changeBreakpoint(path, line, false);
}

void MainWindow::viewPath(int n){
    QString path = document_.path(n);
    if(projDockWidgetList_[path]){
        projDockWidgetList_[path]->show();
    }
}

void MainWindow::addPath(const QString& apath){
    QString path = makeRelative(QDir::currentPath(), apath);
    if(!projDockWidgetList_[path]){
        document_.setPath(document_.pathCount(), path);
        projectView_->update();
        addPathView(path);
    }
}

void MainWindow::print(const QString& mes){
	messages_->append(mes);
}

void MainWindow::moveCallStack(int n){
	if(debugger_.isConnected()){
        debugger_.move_call_stack(n);
	}
}
