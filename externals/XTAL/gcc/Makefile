
CC = g++
OPT += -fno-rtti -fno-exceptions
OPT += -O2 
OPT +=-DNDEBUG -DXTAL_USE_PTHREAD_TLS
OPT += -Wall -Wno-unused-parameter -Wno-unused
OPT += -Wstrict-aliasing=2
#OPT += -fno-strict-aliasing

LOPT = -lpthread -lstdc++

.SUFFIXES: .cpp .o 

SRCDIR = ../src/xtal
BINDIR = ../bin
LIBDIR = ../lib

SRC = $(shell ls $(SRCDIR)/xtal_*.cpp)
OBJ = $(patsubst $(SRCDIR)/%.cpp,%.o,$(SRC))
HED = $(shell ls $(SRCDIR)/xtal_*.h)

all: libxtal xtal ixmake

%.o: $(SRCDIR)/%.cpp
	$(CC) -o $@ -c $< $(OPT)

depend:
	g++ -MM -MG $(SRC) >makefile.depend -DXTAL_NO_THREAD

$(SRCDIR)/xtal.h.gch: $(HED) 
	$(CC) $(SRCDIR)/xtal.h $(OPT)

$(LIBDIR)/libxtal.a: $(SRCDIR)/xtal.h.gch $(OBJ)
	ar r $(LIBDIR)/libxtal.a $(OBJ)
	ranlib $(LIBDIR)/libxtal.a

libxtal: $(LIBDIR)/libxtal.a
	

$(BINDIR)/xtal.exe: $(LIBDIR)/libxtal.a xtal.cpp
	$(CC) -o $(BINDIR)/xtal.exe ./xtal.cpp $(LIBDIR)/libxtal.a $(OPT) $(LOPT) 
	strip --strip-all $(BINDIR)/xtal.exe

xtal: $(BINDIR)/xtal.exe
	

$(BINDIR)/ix.exe: $(LIBDIR)/libxtal.a ix.cpp
	$(CC) -o $(BINDIR)/ix.exe ./ix.cpp $(LIBDIR)/libxtal.a $(OPT) $(LOPT) 
	strip --strip-all $(BINDIR)/ix.exe

ixmake: $(BINDIR)/ix.exe


$(BINDIR)/test.exe: $(LIBDIR)/libxtal.a test.cpp
	$(CC) -o $(BINDIR)/test.exe ./test.cpp $(LIBDIR)/libxtal.a $(OPT) $(LOPT) 

test: $(BINDIR)/test.exe
	$(BINDIR)/test.exe

.PHONY: clean
	

clean:
	rm *.o $(LIBDIR)/libxtal.a $(BINDIR)/xtal.exe $(BINDIR)/ix.exe $(BINDIR)/test.exe $(BINDIR)/prof.exe $(BINDIR)/bench.exe $(BINDIR)/all_src.exe $(SRCDIR)/xtal.h.gch


$(BINDIR)/bench.exe: $(LIBDIR)/libxtal.a bench.cpp
	$(CC) -o $(BINDIR)/bench.exe ./bench.cpp $(LIBDIR)/libxtal.a $(OPT) $(LOPT) 

bench: $(BINDIR)/bench.exe
	$(BINDIR)/bench.exe


$(BINDIR)/prof.exe: $(LIBDIR)/libxtal.a bench.cpp
	$(CC) -o $(BINDIR)/prof.exe ./bench.cpp $(LIBDIR)/libxtal.a $(OPT) $(LOPT)  

prof: override OPT += -pg
prof: $(LIBDIR)/libxtal.a $(BINDIR)/prof.exe
	$(BINDIR)/prof.exe
	gprof $(BINDIR)/prof.exe gmon.out -p > prof.txt
	
all_src: 
	$(CC) -o $(BINDIR)/all_src.exe ../all_src/all_src.cpp $(OPT) $(LOPT) 

run: xtal
	$(BINDIR)/xtal.exe 

checkdll:
	$ objdump -p $(BINDIR)/xtal.exe | grep "DLL Name"
	$ objdump -p $(BINDIR)/ix.exe | grep "DLL Name"
	
-include makefile.depend

