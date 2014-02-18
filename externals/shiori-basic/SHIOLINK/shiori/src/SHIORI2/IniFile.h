#pragma once

class CIniFile
{
public:
	enum LINKTYPE{
		LINKTYPE_NONE = 0,
		LINKTYPE_PROCESS,		// プロセス起動のリンク
		LINKTYPE_ACTIVE_CODE,	// ActiveCodeによるリンク
	};

private:
	// Error
	CString mError;

	// SHIOLINK2::Charset
	UINT mCodePage;

	// SHIOLINK2::LinkType
	LINKTYPE mLinkType;

	// SHIOLINK2::ActiveCodeGUID
	CString mActiveCodeGUID;

public:
	// コンストラクタ
	CIniFile(void);
	~CIniFile(void);

	// getter
	UINT GetCodePage() const { return mCodePage; }
	LINKTYPE GetLinkType() const { return mLinkType; }
	LPCTSTR GetActiveCodeGUID() const { return mActiveCodeGUID; }

	// 読み込み
	void Load(const CString& path, const UINT codePage);

private:
	enum SECTION{
		SECTION_NONE = 0,
		SECTION_SHIOLINK2,
	};
	enum DESC_KEY{
		DESC_KEY_NONE = 0,
		DESC_KEY_ERROR,
		DESC_KEY_SHIOLINK2_CHARSET,
		DESC_KEY_SHIOLINK2_LINK_TYPE,
		DESC_KEY_SHIOLINK2_ACTIVE_CODE_GUID,
	};

	void SetValue(const DESC_KEY key, const CString& value, const bool isQuot);
	void SetCharSet(const CString& value);
	void SetLinkType(const CString& value);
	void SetActiveCodeGUID(const CString& value);
};
