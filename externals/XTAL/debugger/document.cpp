#include "document.h"
#include <QtXml>
#include <QDomDocument>

void Document::init(){
    paths_.clear();
    breakpoints_.clear();
	evalExprs_.clear();
    emit changed();
}

bool Document::save(const QString& filename){
	QFile file(filename);
	if(!file.open(QIODevice::WriteOnly)){
		 return false;
	}

	QTextStream ds(&file);
	QDomDocument doc;

	QDomElement root = doc.createElement("project");
	doc.appendChild(root);

	QDomElement files = doc.createElement("sourcePathList");
	root.appendChild(files);

    for(int i=0; i<paths_.size(); ++i){
		QDomElement node1 = doc.createElement("path");
        QDomText node2 = doc.createTextNode(paths_[i]);
		node1.appendChild(node2);
		files.appendChild(node1);
	}

	QDomElement evalExprs = doc.createElement("evalExprList");
	root.appendChild(evalExprs);
	for(int i=0; i<evalExprs_.size(); ++i){
		QDomElement node1 = doc.createElement("evalExpr");
		QDomText node2 = doc.createTextNode(evalExprs_[i]);
		node1.appendChild(node2);
		evalExprs.appendChild(node1);
	}

	QDomElement breakpoints = doc.createElement("breakpointList");
	root.appendChild(breakpoints);
    for(int i=0; i<breakpoints_.size(); ++i){
        BreakpointInfo* f = &breakpoints_[i];
        QDomElement node1 = doc.createElement("breakpoint");
        node1.setAttribute("file", f->file);
        node1.setAttribute("lineno", f->lineno);
        node1.setAttribute("condition", f->condition);
        breakpoints.appendChild(node1);
	}

	doc.save(ds, 4);

	return true;
}

bool Document::load(const QString& filename){
	QFile file(filename);
	if(!file.open(QIODevice::ReadOnly)){
		 return false;
	}

	init();

	QDomDocument doc;
	QString errorMessage;
	int errorLine;
	int errorColumn;
	if(!doc.setContent(&file, true, &errorMessage, &errorLine, &errorColumn)){
		return false;
	}

	QDomElement root = doc.documentElement();
	if(root.tagName()!="project"){
		return false;
	}

	for(QDomNode node=root.firstChild(); !node.isNull(); node=node.nextSibling()){
		if(node.toElement().tagName()=="sourcePathList"){
			for(QDomNode node2 = node.toElement().firstChild(); !node2.isNull(); node2=node2.nextSibling()){
				QDomNode node3 = node2.toElement().firstChild();
				if(node3.nodeType()==QDomNode::TextNode){
                    paths_.push_back(node3.toText().data());
				}
			}
		}

		if(node.toElement().tagName()=="evalExprList"){
			for(QDomNode node2 = node.toElement().firstChild(); !node2.isNull(); node2=node2.nextSibling()){
				QDomNode node3 = node2.toElement().firstChild();
				if(node3.nodeType()==QDomNode::TextNode){
					evalExprs_.push_back(node3.toText().data());
				}
			}
		}

		if(node.toElement().tagName()=="breakpointList"){
			for(QDomNode node2 = node.toElement().firstChild(); !node2.isNull(); node2=node2.nextSibling()){
				QDomElement elem = node2.toElement();
                BreakpointInfo bi;
                bi.file = elem.attribute("file");
                bi.lineno = elem.attribute("lineno").toInt();
                bi.condition = elem.attribute("condition");
                breakpoints_.push_back(bi);
			}
		}
	}

    emit changed();
    return true;
}

QString Document::path(int i){
    if(i<0 || i>=paths_.size()){
        return "";
	}
    return paths_[i];
}

int Document::pathCount(){
    return paths_.size();
}

void Document::insertPath(int n, const QString& path){
    paths_.insert(n, path);
    emit changed();
}

void Document::setPath(int n, const QString& path){
    if(n<paths_.size()){
        if(path!=""){
            paths_[n] = path;
            emit changed();
        }
        else{
            removePath(n);
            emit changed();
        }
    }
    else{
        if(path!=""){
            paths_.push_back(path);
            emit changed();
        }
    }
}

void Document::removePath(int n){
    paths_.remove(n);
}

BreakpointInfo Document::breakpoint(int i){
    return breakpoints_[i];
}

int Document::breakpointCount(){
    return breakpoints_.size();
}

QString Document::breakpointCondition(const QString& file, int lineno){
    for(int i=0; i<breakpoints_.size(); ++i){
        if(breakpoints_[i].file==file && breakpoints_[i].lineno==lineno){
            return breakpoints_[i].condition;
        }
    }
}

void Document::addBreakpoint(const QString& file, int lineno){
    for(int i=0; i<breakpoints_.size(); ++i){
        if(breakpoints_[i].file==file && breakpoints_[i].lineno==lineno){
            return;
        }
    }

    BreakpointInfo bi;
    bi.file = file;
    bi.lineno = lineno;
    breakpoints_.append(bi);
    emit changed();
}

void Document::addBreakpoint(const QString& file, int lineno, const QString& cond){
    for(int i=0; i<breakpoints_.size(); ++i){
        if(breakpoints_[i].file==file && breakpoints_[i].lineno==lineno){
            breakpoints_[i].condition = cond;
            return;
        }
    }

    BreakpointInfo bi;
    bi.file = file;
    bi.lineno = lineno;
    bi.condition = cond;
    breakpoints_.append(bi);
    emit changed();
}

void Document::removeBreakpoint(const QString& file, int lineno){
    for(int i=0; i<breakpoints_.size(); ++i){
        if(breakpoints_[i].file==file && breakpoints_[i].lineno==lineno){
            breakpoints_.erase(breakpoints_.begin()+i);
            emit changed();
            return;
        }
    }
}
