#include "stdafx.h"
#include "PipeTest.h"

static int runTest(int argc, char **argv)
{
	std::cout << "Running UnitTest main() from main.cpp\n";
	
	testing::InitGoogleTest(&argc, argv);
	int rc = RUN_ALL_TESTS();
	if(rc == 0){
		ATLTRACE2(_T("TEST SUCCESS.\n"));
	}
	else{
		ATLTRACE2(_T("TEST FAULD.\n"));
		std::cout << "Please Any Key ..\n";
		char c[1];
		std::cin.read(c,1);
	}
	return rc;
}

int main(int argc, char **argv) {
	if( argc<2 ) return	runTest(argc, argv);
	return				runPipeServer(argc, argv);
}
