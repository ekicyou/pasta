#include <QtGui/QApplication>
#include <QDir>
#include <QFile>

#include "mainwindow.h"

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_lib/xtal_errormessage.h"

namespace xtal{

class QtFindNextFile{
	QDir dir_;
	QStringList list_;
	int n_;
public:

	QtFindNextFile(const char_t* path){
		dir_ = QDir(path);
		list_ = dir_.entryList();
		n_ = 0;
	}

	~QtFindNextFile(){

	}

	const char_t* next(){
		if(n_>=list_.size()){
			return 0;
		}

		return list_[n_++].toStdString().c_str();
	}

	void stop(){

	}
};

class QtFilesystemLib : public FilesystemLib{
public:

	virtual bool is_directory(const char_t* path){
		return QDir(path).exists();
	}

	virtual void* new_file_stream(const char_t* path, const char_t* flags){
		QFile* f = new QFile(path);

		QFile::OpenMode om = QFile::Text;
		while(*flags){
			if(*flags=='r'){
				om |= QFile::ReadOnly;
			}

			if(*flags=='w'){
				om |= QFile::WriteOnly;
			}

			if(*flags=='a'){
				om |= QFile::Append;
			}

			if(*flags=='b'){
				om ^= QFile::Text;
			}

			++flags;
		}

		if(f->open(om)){
			return f;
		}

		delete f;
		return 0;
	}

	virtual void delete_file_stream(void* file_stream_object){
		delete ((QFile*)file_stream_object);
	}

	virtual uint_t read_file_stream(void* file_stream_object, void* dest, uint_t size){
		return ((QFile*)file_stream_object)->read((char*)dest, size);
	}

	virtual uint_t write_file_stream(void* file_stream_object, const void* src, uint_t size){
		return ((QFile*)file_stream_object)->write((char*)src, size);
	}

	virtual void seek_file_stream(void* file_stream_object, uint_t offset){
		((QFile*)file_stream_object)->seek(offset);
	}

	virtual uint_t tell_file_stream(void* file_stream_object){
		return ((QFile*)file_stream_object)->pos();
	}

	virtual bool end_file_stream(void* file_stream_object){
		return ((QFile*)file_stream_object)->atEnd();
	}

	virtual uint_t size_file_stream(void* file_stream_object){
		return ((QFile*)file_stream_object)->size();
	}

	virtual void flush_file_stream(void* file_stream_object){
		((QFile*)file_stream_object)->flush();
	}

	virtual void* new_entries(const char_t* path){
		void* p = xmalloc(sizeof(QtFindNextFile));
		return new(p) QtFindNextFile(path);
	}

	virtual void delete_entries(void* entries_object){
		((QtFindNextFile*)entries_object)->~QtFindNextFile();
		xfree(entries_object, sizeof(QtFindNextFile));
	}

	virtual const char_t* next_entries(void* entries_object){
		return ((QtFindNextFile*)entries_object)->next();
	}

	virtual void break_entries(void* entries_object){
		((QtFindNextFile*)entries_object)->stop();
	}

};

}

int main(int argc, char *argv[]){
	xtal::Setting setting;
	xtal::QtFilesystemLib flib;
	setting.filesystem_lib = &flib;
	xtal::initialize(setting);
	xtal::bind_error_message();

    xtal::debug::enable();
    xtal::debug::enable_debug_compile();

    {
        QApplication a(argc, argv);
        QTextCodec::setCodecForCStrings(QTextCodec::codecForName("SJIS"));
        MainWindow w;
        w.show();

        a.exec();
    }

    xtal::uninitialize();

	return 0;
}
