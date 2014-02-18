// 以下の ifdef ブロックは DLL からのエクスポートを容易にするマクロを作成するための 
// 一般的な方法です。この DLL 内のすべてのファイルは、コマンド ラインで定義された SHIOLINK3_EXPORTS
// シンボルを使用してコンパイルされます。このシンボルは、この DLL を使用するプロジェクトでは定義できません。
// ソースファイルがこのファイルを含んでいる他のプロジェクトは、 
// SHIOLINK3_API 関数を DLL からインポートされたと見なすのに対し、この DLL は、このマクロで定義された
// シンボルをエクスポートされたと見なします。
#ifdef SHIOLINK3_EXPORTS
#define SHIOLINK3_API __declspec(dllexport)
#else
#define SHIOLINK3_API __declspec(dllimport)
#endif

// このクラスは SHIOLINK3.dll からエクスポートされました。
class SHIOLINK3_API CSHIOLINK3 {
public:
	CSHIOLINK3(void);
	// TODO: メソッドをここに追加してください。
};

extern SHIOLINK3_API int nSHIOLINK3;

SHIOLINK3_API int fnSHIOLINK3(void);
