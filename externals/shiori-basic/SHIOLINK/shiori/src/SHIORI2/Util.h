#pragma once

namespace Util
{
	namespace File
	{
		// 指定したcodepageのテキストとしてファイルを読み込みます
		CString ReadAllText(const CString& path, const UINT codePage);
	}

	namespace Text
	{
		// 文字列から[""] を除去します。
		CString UnQuot(const CString& text);

		// コードページを取得します。
		UINT GetCP(const CString& text);
	}
}
