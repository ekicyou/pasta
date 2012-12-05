using System;
using System.Collections.Generic;
using System.Text;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
using System.IO;

namespace CSUtil.IO
{
  /// <summary>
  /// メールスロットの送受信を行うクラスです。
  /// </summary>
  public class MailSlot
  {
    #region Win32 API宣言

    [DllImport("kernel32.dll", CharSet = CharSet.Auto)]
    private static extern SafeFileHandle CreateFile(
        string lpFileName, // ファイル名 
        DesiredAccess dwDesiredAccess, // アクセスモード 
        ShareMode dwShareMode, // 共有モード 
        int lpSecurityAttributes, // セキュリティ記述子 
        CreationDisposition dwCreationDisposition, // 作成方法 
        FlagsAndAttributes dwFlagsAndAttributes, // ファイル属性 
        IntPtr hTemplateFile // テンプレートファイルのハンドル 
        );

    [DllImport("kernel32.dll", CharSet = CharSet.Auto)]
    static extern SafeFileHandle CreateMailslot(
        string lpName,
        uint nMaxMessageSize,
        uint lReadTimeout,
        IntPtr lpSecurityAttributes);

    #endregion

    #region 列挙体

    private enum DesiredAccess : uint
    {
      GENERIC_READ = 0x80000000,
      GENERIC_WRITE = 0x40000000,
      GENERIC_EXECUTE = 0x20000000
    }

    private enum ShareMode : uint
    {
      FILE_SHARE_READ = 0x00000001,
      FILE_SHARE_WRITE = 0x00000002,
      FILE_SHARE_DELETE = 0x00000004
    }

    private enum CreationDisposition : uint
    {
      CREATE_NEW = 1,
      CREATE_ALWAYS = 2,
      OPEN_EXISTING = 3,
      OPEN_ALWAYS = 4,
      TRUNCATE_EXISTING = 5
    }

    private enum FlagsAndAttributes : uint
    {
      FILE_ATTRIBUTE_ARCHIVE = 0x00000020,
      FILE_ATTRIBUTE_ENCRYPTED = 0x00004000,
      FILE_ATTRIBUTE_HIDDEN = 0x00000002,
      FILE_ATTRIBUTE_NORMAL = 0x00000080,
      FILE_ATTRIBUTE_NOT_CONTENT_INDEXED = 0x00002000,
      FILE_ATTRIBUTE_OFFLINE = 0x00001000,
      FILE_ATTRIBUTE_READONLY = 0x00000001,
      FILE_ATTRIBUTE_SYSTEM = 0x00000004,
      FILE_ATTRIBUTE_TEMPORARY = 0x00000100
    }

    #endregion

    private string slot = string.Empty;
    /// <summary>
    /// スロット名称を取得又は設定します。
    /// </summary>
    public string Slot
    {
      get { return slot; }
      set { slot = value; }
    }

    /// <summary>
    /// コンストラクタ
    /// </summary>
    public MailSlot()
    {
    }

    /// <summary>
    /// メールスロット名付コンストラクタ
    /// </summary>
    /// <param name="strSlot">メールスロット</param>
    public MailSlot(string strSlot)
      : this()
    {
      Slot = strSlot;
    }

    /// <summary>
    /// メッセージの送信
    /// </summary>
    /// <param name="strMessage">送信する文字列</param>
    public void WriteMailSlot(string strMessage)
    {

      SafeFileHandle fileHandle = null;

      try {
        fileHandle = CreateFile(Slot,
                DesiredAccess.GENERIC_READ | DesiredAccess.GENERIC_WRITE,
                ShareMode.FILE_SHARE_READ | ShareMode.FILE_SHARE_WRITE,
                0,
                CreationDisposition.OPEN_EXISTING,
                FlagsAndAttributes.FILE_ATTRIBUTE_NORMAL,
                (IntPtr)0);


        using (FileStream fs = new FileStream(fileHandle, FileAccess.Write)) {
          byte[] msg = Encoding.Default.GetBytes(strMessage);
          fs.Write(msg, 0, msg.Length);
        }
      }
      catch (Exception ex) {
        System.Diagnostics.Debug.WriteLine(ex.Message);
        System.Diagnostics.Debug.WriteLine(ex.StackTrace);
        throw ex;
      }
    }

    /// <summary>
    /// メールスロット受信待ち
    /// </summary>
    /// <param name="maxMessageSize">最大メッセージ数</param>
    /// <param name="readTimeout">タイムアウト</param>
    public string ReadMailSlot(uint maxMessageSize, uint readTimeout)
    {
      string result = string.Empty;
      SafeFileHandle mailslotHandle = null;
      try {
        mailslotHandle = CreateMailslot(Slot, maxMessageSize, readTimeout, (IntPtr)0);

        using (FileStream fs = new FileStream(mailslotHandle, FileAccess.Read)) {
          byte[] buf = new byte[maxMessageSize];
          int len = fs.Read(buf, 0, buf.Length);
          result = Encoding.Default.GetString(buf, 0, len);

          fs.Close();
        }
      }
      catch (Exception ex) {
        System.Diagnostics.Debug.WriteLine(ex.Message);
        System.Diagnostics.Debug.WriteLine(ex.StackTrace);
        throw ex;
      }
      return result;
    }
  }
}
