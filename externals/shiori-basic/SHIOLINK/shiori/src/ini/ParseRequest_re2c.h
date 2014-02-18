// iniâêÕÉÇÉWÉÖÅ[Éã(re2c)
#pragma once

class RequestScanner{

public:
	/*!types:re2c */

	enum SCAN_TOKEN {
		TOKEN_EOT = 0,
		TOKEN_LEXER_ERROR,
		TOKEN_LEXER_SKIP,
		TOKEN_PROTOCOL_NAME,
		TOKEN_PROTOCOL_METHOD,
		TOKEN_COMMENT_LINE,
		TOKEN_ITEM_KEYWORD,
		TOKEN_ITEM_SPLIT,
		TOKEN_ITEM_VALUE,
		TOKEN_ITEM_VALUE_QUOT,
		TOKEN_SECTION_HEADER,
		TOKEN_SECTION_NAME,
	};
	class Token {
	private:
		const SCAN_TOKEN mToken;
		LPCTSTR mStart;
		LPCTSTR mEnd;
	public:
		Token(SCAN_TOKEN token, LPCTSTR start, LPCTSTR end)
			:mToken(token)
			,mStart(start)
			,mEnd(end)
		{}
		SCAN_TOKEN GetToken() const { return mToken; }
		int GetTextLength() const { return (int)((mEnd-mStart) / sizeof(TCHAR) );  }
		CString GetText() const { return CString(mStart, GetTextLength()); }
	};

private:
	CString mText;
	LPTSTR YYMARKER;
	SCAN_CONDITION mCondition;
	typedef Token TT;

public:
	RequestScanner(LPCTSTR text)
	{
		mText = text;
		YYMARKER = (LPTSTR)((LPCTSTR)mText);
		SetCondition(SCAN_CONDITION_START);
	}

	SCAN_CONDITION GetCondition() const { return mCondition; }
	void SetCondition(SCAN_CONDITION cond) { mCondition = cond; }

	Token Scan()
	{
		LPTSTR st  = YYMARKER;
		LPTSTR pos = YYMARKER;
		if(*st == 0) return TT(TOKEN_EOT,st,pos);

// ============================================================
// re2c define
// ============================================================
/*!re2c
  re2c:indent:top = 3 ;
  re2c:define:YYCTYPE    = "TCHAR";
  re2c:define:YYCURSOR   = pos;
  re2c:define:YYCONDTYPE = SCAN_CONDITION;
  re2c:yyfill:enable     = 0;
  re2c:cond:divider = "// *********************************** [#]" ;
  re2c:cond:divider@cond = # ;
  re2c:condenumprefix    = "SCAN_CONDITION_" ;
  re2c:define:YYGETCONDITION       = GetCondition();
  re2c:define:YYGETCONDITION:naked = 1;
  re2c:define:YYSETCONDITION       = "SetCondition(#);";
  re2c:define:YYSETCONDITION@cond  = #;
*/

// ============================================================
// word class: Unicode category
// ============================================================
/*!re2c
  Lu = [A-Z];
  Ll = [a-z];
  Lo = [Lo];
  Nd = [0-9];
  Zs = [ \X00A0\X1680\X180E\X2000-\X200A\X202F\X205F\X3000];
  Zl = [\X2028];
  Zp = [\X2029];
*/

// ============================================================
// word class:
// ============================================================
/*!re2c
  _ws		= Zs | Zl | Zp | [\t\X0085\X00a0] ;
  _num09	= Nd ;
  _comment	= [#] ;
  _split	= [:] ;
  _quot		= ["] ;
  _noquot	= [^"] ;
  _esc		= [\\] ;
  _sec		= [[] ;
  _keyword1	= Lu | Ll | [_-.] ;
  _keywordX	= _keyword1 | _num09 ;
  _tany     = [\X0020-] ;
  _value	= _tany \( _ws | _quot ) ;
*/

// ============================================================
// token:
// ============================================================
/*!re2c
  number  = _num09+ ;
  ws      = _ws ;
  keyword = _keyword1 _keywordX* ;

  newLine       =  "\r\n" | "\r" | "\n" ;
  notCRLF       = [^\r\n] ;
  any           = .* ;
  anyMore       = .+ ;
  anyOne        = . ;
  anyTail       = any newLine ;
  wstail        = ws* newLine ;
  quot			= _quot ;
  nonquot		= ( (_quot _quot) | _noquot )* ;

  lineComment	= _comment anyTail ;
  lineBlank		= ws* newLine ;

  protocol      = keyword "/" number "." number ws* ;

  split			= ws* _split ws* ;
  value			= _value anyTail ;
  quotvalue		= quot nonquot quot ;
*/

// ============================================================
// scaner
// (SAMPLE)
// 1: SHIORI/3.0 GET	: protocol any
// 2: charset: UTF-8	: keyword: value
// 4: [section]			: [keyword]
// 5: #xxxxxx			: # any
// 6: (space)			: spaces
// ============================================================
/*!re2c
  <START,HEAD>lineComment				{ return TT(TOKEN_COMMENT_LINE,st,pos); }
  <START,HEAD>lineBlank					{ return TT(TOKEN_COMMENT_LINE,st,pos); }
  <START,HEAD>_sec			=>SECTION	{ return TT(TOKEN_SECTION_HEADER,st,pos); }
  <START>protocol  			=>PROTOCOL	{ return TT(TOKEN_PROTOCOL_NAME,st,pos); }

  <PROTOCOL>keyword			=>SKIP		{ return TT(TOKEN_PROTOCOL_METHOD,st,pos); }

  <HEAD>keyword				=>SPLIT		{ return TT(TOKEN_ITEM_KEYWORD,st,pos); }
  <SPLIT>split				=>VALUE		{ return TT(TOKEN_ITEM_SPLIT,st,pos); }
  <VALUE>value				=>SKIP		{ return TT(TOKEN_ITEM_VALUE,st,pos); }
  <VALUE>quotvalue			=>SKIP		{ return TT(TOKEN_ITEM_VALUE_QUOT,st,pos); }

  <SECTION>keyword			=>SKIP		{ return TT(TOKEN_SECTION_NAME,st,pos); }

  <START,HEAD,PROTOCOL,SPLIT,VALUE,SECTION>anyOne	=>SKIP	{ return TT(TOKEN_LEXER_ERROR,st,pos); }
  <SKIP>notCRLF*									=>HEAD	{ return TT(TOKEN_LEXER_SKIP,st,pos); }

*/
	}

};
