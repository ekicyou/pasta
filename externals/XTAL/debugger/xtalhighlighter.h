#ifndef XTALHIGHLIGHTER_H
#define XTALHIGHLIGHTER_H

#include <QtGui>
#include <QPlainTextEdit>
#include <QObject>

/**
  * \brief Xtalのシンタックスハイライタ
  */
class XtalHighlighter : public QSyntaxHighlighter{
	Q_OBJECT

public:

	XtalHighlighter(QTextDocument *parent = 0);

protected:

	void highlightBlock(const QString &text);

private:

	struct HighlightingRule{
		QRegExp pattern;
		QTextCharFormat format;
	};

	QVector<HighlightingRule> highlightingRules_;

	QRegExp commentStartExpression_;
	QRegExp commentEndExpression_;

	QTextCharFormat keywordFormat_;
	QTextCharFormat classFormat_;
	QTextCharFormat singlelineCommentFormat_;
	QTextCharFormat multilineCommentFormat_;
	QTextCharFormat quotationFormat_;
	QTextCharFormat numberFormat_;
};

#endif // XTALHIGHLIGHTER_H
