#pragma once

#include <Windows.h>
#include <ostream>
#include <sstream>
#include <string>

#include <boost/noncopyable.hpp>

// See: http://www.codeproject.com/debug/debugout.asp

namespace biscuit
{

template< class CharT, class TraitsT = std::char_traits<CharT> >
struct basic_debugbuf : 
	std::basic_stringbuf<CharT, TraitsT>,
	private boost::noncopyable
{
	virtual ~basic_debugbuf()
	{
		sync();
	}

protected:
	int sync()
	{
		output_debug_string(str().c_str());
		str(std::basic_string<CharT>()); // Clear the string buffer

		return 0;
	}

	void output_debug_string(CharT const *text) { }
};

template<>
void basic_debugbuf<char>::output_debug_string(char const *text)
{
	::OutputDebugStringA(text);
}

template<>
void basic_debugbuf<wchar_t>::output_debug_string(wchar_t const *text)
{
	::OutputDebugStringW(text);
}

template< class CharT, class TraitsT = std::char_traits<CharT> >
struct basic_dostream : 
	std::basic_ostream<CharT, TraitsT>,
	private boost::noncopyable
{
	basic_dostream() : std::basic_ostream<CharT, TraitsT>
		(new basic_debugbuf<CharT, TraitsT>())
	{ }

	~basic_dostream() 
	{
		delete rdbuf(); 
	}
};

typedef basic_dostream<char> dostream;
typedef basic_dostream<wchar_t> wdostream;

	namespace
	{
		dostream dout; // flush may be required for multiple .cpp
	}
} // namespace biscuit


