#include "debugger.h"

Debugger::Debugger(){
	connect(&server_, SIGNAL(newConnection()), SLOT(onConnected()));
	server_.listen(QHostAddress::Any, 13245);
	state_ = STATE_NONE;
}

void Debugger::setSource(const QString& source){
	source_ = source;
}

bool Debugger::isConnected(){
	return state_!=STATE_NONE;
}

void Debugger::onRecv(){
    qDebug() << "onRecv";
    update();
}

void Debugger::onClosed(){
    qDebug() << "onClosed";
    state_ = STATE_NONE;
	emit disconnected();
}

void Debugger::onConnected(){
    qDebug() << "onConnected";
    SmartPtr<TCPStream> s = xnew<TCPStream>(server_.nextPendingConnection());
    connect(s->rawsocket(), SIGNAL(readyRead()), SLOT(onRecv()));
    connect(s->rawsocket(), SIGNAL(readChannelFinished()), SLOT(onClosed()));
    state_ = STATE_CONNECTED;
    start(s);
    emit connected();
}


