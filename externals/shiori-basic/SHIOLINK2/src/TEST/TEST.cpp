#include "stdafx.h"

static int test1(const int val){
	return val;
}

TEST(FactorialTest, Negative) {
  // This test is named "Negative", and belongs to the "FactorialTest"
  // test case.
  EXPECT_EQ(1, test1(1));
}