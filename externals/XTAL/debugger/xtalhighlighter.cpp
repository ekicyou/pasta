#include "xtalhighlighter.h"

XtalHighlighter::XtalHighlighter(QTextDocument *parent)
	: QSyntaxHighlighter(parent){
	HighlightingRule rule;

	keywordFormat_.setForeground(Qt::blue);
	keywordFormat_.setFontWeight(QFont::Bold);
	QStringList keywordPatterns;

	keywordPatterns
		<< "if"
		<< "for"
		<< "else"
		<< "fun"
		<< "method"
		<< "do"
		<< "while"
		<< "continue"
		<< "break"
		<< "fiber"
		<< "yield"
		<< "return"
		<< "once"
		<< "null"
		<< "undefined"
		<< "false"
		<< "true"
		<< "xtal"
		<< "try"
		<< "catch"
		<< "finally"
		<< "throw"
		<< "class"
		<< "callee"
		<< "this"
		<< "dofun"
		<< "is"
		<< "in"
		<< "assert"
		<< "nobreak"
		<< "switch"
		<< "case"
		<< "default"
		<< "singleton"
		<< "public"
		<< "protected"
		<< "private";

	foreach(const QString &pattern, keywordPatterns) {
		rule.pattern = QRegExp("\\b" + pattern + "\\b");
		rule.format = keywordFormat_;
		highlightingRules_.append(rule);
	}

	classFormat_.setFontWeight(QFont::Bold);
	classFormat_.setForeground(Qt::darkMagenta);
	rule.pattern = QRegExp("\\b[A-Z][A-Za-z0-9_]+\\b");
	rule.format = classFormat_;
	highlightingRules_.append(rule);

	numberFormat_.setFontWeight(QFont::Bold);
	numberFormat_.setForeground(Qt::darkBlue);
	rule.pattern = QRegExp("\\b[0-9]([0-9_]|\\.[0-9])*[fF]?\\b");
	rule.format = numberFormat_;
	highlightingRules_.append(rule);

	singlelineCommentFormat_.setForeground(Qt::darkGreen);
	rule.pattern = QRegExp("//[^\n]*");
	rule.format = singlelineCommentFormat_;
	highlightingRules_.append(rule);

	multilineCommentFormat_.setForeground(Qt::darkGreen);

	quotationFormat_.setForeground(Qt::darkRed);
	rule.pattern = QRegExp("\".*\"");
	rule.format = quotationFormat_;
	highlightingRules_.append(rule);

	commentStartExpression_ = QRegExp("/\\*");
	commentEndExpression_ = QRegExp("\\*/");
}

void XtalHighlighter::highlightBlock(const QString &text){
	foreach(const HighlightingRule &rule, highlightingRules_){
		QRegExp expression(rule.pattern);
		int index = expression.indexIn(text);
		while(index >= 0){
			int length = expression.matchedLength();
			setFormat(index, length, rule.format);
			index = expression.indexIn(text, index + length);
		}
	}
	setCurrentBlockState(0);

	int startIndex = 0;
	if(previousBlockState() != 1){
		startIndex = commentStartExpression_.indexIn(text);
	}

	while(startIndex >= 0){
		int endIndex = commentEndExpression_.indexIn(text, startIndex);
		int commentLength;
		if(endIndex == -1){
			setCurrentBlockState(1);
			commentLength = text.length() - startIndex;
		}
		else{
			commentLength = endIndex - startIndex + commentEndExpression_.matchedLength();
		}

		setFormat(startIndex, commentLength, multilineCommentFormat_);
		startIndex = commentStartExpression_.indexIn(text, startIndex + commentLength);
	}
}

