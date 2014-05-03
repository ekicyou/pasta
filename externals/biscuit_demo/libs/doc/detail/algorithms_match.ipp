BOOST_CHECK( match<xml_comment>("<!-- hello, xml comment -->") );
BOOST_CHECK( !match<xml_comment>("<!-- not well-formed comment -- -->") );
