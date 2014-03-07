#ifndef DEBUGGER_H
#define DEBUGGER_H

#include <QTcpSocket>
#include <QTcpServer>

#include "xtal_src.h"

class TCPStream : public Stream{
public:

	TCPStream(QTcpSocket* socket = 0){
		socket_ = socket;
	}

	virtual ~TCPStream(){
        on_close();
	}

    bool is_open(){
        return socket_!=0 && socket_->isOpen();
	}

    virtual void on_close(){
        if(socket_){
			socket_ = 0;
		}
	}

    virtual uint_t on_available(){
        if(!is_open()){
            return 0;
        }

        return socket_->bytesAvailable();
    }

    virtual uint_t on_read(void* dest, uint_t size){
        if(!is_open()){
            return 0;
        }

		uint_t read = 0;

		while(true){
			int temp = socket_->read((char*)dest+read, size-read);
			if(temp<0){
				close();
				return read;
			}
			read += temp;
			if(read==size){
				break;
			}

			socket_->waitForReadyRead();
		}

		return read;
	}

    virtual uint_t on_write(const void* src, uint_t size){
        if(!is_open()){
            return 0;
        }

		int temp = socket_->write((char*)src, size);
		if(temp<0){
			close();
			return 0;
		}
		return temp;
	}

    virtual bool on_eos(){
        return !is_open();
    }

	QTcpSocket* rawsocket(){
		return socket_;
	}

private:
	QTcpSocket* socket_;
};

class Debugger : public QObject, public debug::CommandSender{
	Q_OBJECT
public:

	Debugger();

	void setSource(const QString& source);

public:
	bool isConnected();

signals:
	// リモートデバック先と接続された際のシグナル
	void connected();

	// リモートデバック先と接続された際のシグナル
	void disconnected();

	// ブレークした際のシグナル
	void breaked();

	// ブレークした際のシグナル
	void required();

	// コンパイルエラーが起こった際のシグナル
	void compileError();

	// キャッチされない例外が起こった際のシグナル
	void uncatchedException();

protected:
    virtual void on_breaked(){
        emit breaked();
    }

    virtual void on_required(){
        emit required();
    }

    virtual void on_compile_error(){
        emit compileError();
    }

    virtual void on_uncatched_exception(){
        emit uncatchedException();
    }

protected slots:

	void onRecv();
	void onClosed();
	void onConnected();

private:

	int socket_;
	QTcpServer server_;
	QString source_;

	int state_;

	enum{
		STATE_NONE,
		STATE_CONNECTED,
	};
};

#endif // DEBUGGER_H
