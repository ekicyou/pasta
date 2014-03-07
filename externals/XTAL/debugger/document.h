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
  * \brief �v���W�F�N�g�̏���ێ�����N���X
�@ * �\�[�X�p�X�A�u���[�N�|�C���g�̈ʒu�A�]������ێ����Ă���
  */
class Document : public QObject{
    Q_OBJECT
public:

    /**
      * \brief �h�L�������g������������
      */
	void init();

    /**
      * \brief �h�L�������g��ۑ�����
      */
	bool save(const QString& filename);

    /**
      * \brief �h�L�������g��ǂݍ���
      */
	bool load(const QString& filename);

signals:
    void changed();

public:

	/**
      * \brief i�Ԗڂ̃p�X�������o��
	  */
    QString path(int i);

	/**
	  * \brief �t�@�C����񂪉����邩�Ԃ�
	  */
    int pathCount();

    void setPath(int n, const QString& path);

    void insertPath(int n, const QString& path);

    void removePath(int n);

public:

    /**
      * \brief i�Ԗڂ̃p�X�������o��
      */
    BreakpointInfo breakpoint(int i);


    /**
      * \brief �t�@�C����񂪉����邩�Ԃ�
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
      * \brief �]������ݒ肷��
      */
	void setEvalExpr(int n, const QString& expr){
		if(n>=evalExprs_.size()){
			evalExprs_.resize(n+1);
		}

		evalExprs_[n] = expr;
	}

    /**
      * \breif �]�����̐����擾����
      */
	int evalExprCount(){
		return evalExprs_.size();
	}

    /**
      * \brief �]�������擾����
      */
	QString evalExpr(int n){
		return evalExprs_[n];
	}

    /**
      * \brief n�Ԗڂ̕]�������폜����
      */
	void removeEvalExpr(int n);

    /**
      * \brief n�Ԗڂɕ]������ǉ�����
      */
	void insertEvaExpr(int n);

private:
    // �p�X�̃��X�g
    QVector<QString> paths_;

    // �u���[�N�|�C���g�̃��X�g
    QVector<BreakpointInfo> breakpoints_;

    // �]�����̃��X�g
	QVector<QString> evalExprs_;
};


#endif // DOCUMENT_H
