//! Build script for `pasta_shiori`.
//!
//! pasta.dll に VERSIONINFO リソースを埋め込み、エクスプローラーのプロパティ「詳細」タブで
//! ファイルバージョン／製品バージョンを視認できるようにする。版数はワークスペースの
//! `version`（CARGO_PKG_VERSION）が唯一の正本で、リリース時の手動同期は不要。
//!
//! .rc は OUT_DIR に生成する（rc.exe へのマクロ文字列受け渡しのエスケープを避けるため）。
//! 非 Windows ターゲット（docs.rs 等）では embed-resource が何もしない。

use std::env;
use std::path::PathBuf;

fn main() {
    let var = |k: &str| env::var(k).unwrap_or_else(|_| panic!("{k} not set by cargo"));
    // FILEVERSION/PRODUCTVERSION は数値4つ。pre-release 等は文字列側にのみ現れる。
    let num = format!(
        "{},{},{},0",
        var("CARGO_PKG_VERSION_MAJOR"),
        var("CARGO_PKG_VERSION_MINOR"),
        var("CARGO_PKG_VERSION_PATCH")
    );
    let ver = var("CARGO_PKG_VERSION");
    let desc = var("CARGO_PKG_DESCRIPTION");
    // authors の "Name <mail>" からメール部分を落として著作権表記に使う。
    let authors = var("CARGO_PKG_AUTHORS");
    let author = authors.split(':').next().unwrap_or_default();
    let author = author.split('<').next().unwrap_or_default().trim();

    // FILEOS 0x40004 = VOS_NT_WINDOWS32, FILETYPE 0x2 = VFT_DLL（winver.h 非依存にするため数値で書く）。
    // 040904B0 = 英語(US)・Unicode。
    let rc = format!(
        r#"1 VERSIONINFO
FILEVERSION {num}
PRODUCTVERSION {num}
FILEFLAGSMASK 0x3F
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x2
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "FileDescription", "{desc}"
      VALUE "FileVersion", "{ver}"
      VALUE "InternalName", "pasta"
      VALUE "LegalCopyright", "Copyright (c) {author}"
      VALUE "OriginalFilename", "pasta.dll"
      VALUE "ProductName", "pasta"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );
    let rc_path = PathBuf::from(var("OUT_DIR")).join("pasta.rc");
    std::fs::write(&rc_path, rc).expect("write pasta.rc");

    // cdylib（pasta.dll）にのみリンクする。Windows で rc.exe が無い／失敗した場合は
    // 版数なし DLL を黙って出荷しないようビルドを落とす。
    embed_resource::compile_for_cdylib(&rc_path, embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
