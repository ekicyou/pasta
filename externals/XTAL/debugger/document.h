#ifndef DOCUMENT_H
#define DOCUMENT_H

#include <QtGui>
#include <QPlainTextEdit>
#include <QObject>

struct BreakpointInfo{
    QString file;
    int lineno;
    QString condition;
};


/**
  * \brief プロジェクトの情報を保持するクラス
　 * ソースパス、ブレークポイントの位置、評価式を保持している
  */
class Document : public QObject{
    Q_OBJECT
public:

    /**
      * \brief ドキュメントを初期化する
      */
	void init();

    /**
      * \brief ドキュメントを保存する
      */
	bool save(const QString& filename);

    /**
      * \brief ドキュメントを読み込む
      */
	bool load(const QString& filename);

signals:
    void changed();

public:

	/**
      * \brief i番目のパス情報を取り出す
	  */
    QString path(int i);

	/**
	  * \brief ファイル情報が何個あるか返す
	  */
    int pathCount();

    void setPath(int n, const QString& path);

    void insertPath(int n, const QString& path);

    void removePath(int n);

public:

    /**
      * \brief i番目のパス情報を取り出す
      */
    BreakpointInfo breakpoint(int i);


    /**
      * \brief ファイル情報が何個あるか返す
      */
    int breakpointCount();

    /**
      * \brief
      */
    void addBreakpoint(const QString& file, int lineno);

    void addBreakpoint(const QString& file, int lineno, const QString& cond);

    QString breakpointCondition(const QString& file, int lineno);

    void removeBreakpoint(const QString& file, int lineno);

public:

    /**
      * \brief 評価式を設定する
      */
	void setEvalExpr(int n, const QString& expr){
		if(n>=evalExprs_.size()){
			evalExprs_.resize(n+1);
		}

		evalExprs_[n] = expr;
	}

    /**
      * \breif 評価式の数を取得する
      */
	int evalExprCount(){
		return evalExprs_.size();
	}

    /**
      * \brief 評価式を取得する
      */
	QString evalExpr(int n){
		return evalExprs_[n];
	}

    /**
      * \brief n番目の評価式を削除する
      */
	void removeEvalExpr(int n);

    /**
      * \brief n番目に評価式を追加する
      */
	void insertEvaExpr(int n);

private:
    // パスのリスト
    QVector<QString> paths_;

    // ブレークポイントのリスト
    QVector<BreakpointInfo> breakpoints_;

    // 評価式のリスト
	QVector<QString> evalExprs_;
};


#endif // DOCUMENT_H
