using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace CSUtil.IO
{
    /// <summary>
    /// メールスロット管理基本クラス。
    /// </summary>
    public abstract class MailSlotBase : IDisposable
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
        private static extern SafeFileHandle CreateMailslot(
            string lpName,
            uint nMaxMessageSize,
            uint lReadTimeout,
            IntPtr lpSecurityAttributes);

        [DllImport("kernel32.dll", CharSet = CharSet.Auto)]
        private static extern bool GetMailslotInfo(
            SafeFileHandle hMailslot,   // メールスロットのハンドル
            IntPtr lpMaxMessageSize,    // 最大メッセージサイズ
            out int lpNextSize,         // 次のメッセージのサイズ
            IntPtr lpMessageCount,      // メッセージ数
            IntPtr lpReadTimeout        // 読み取りタイムアウトの間隔
          );

        #endregion

        #region 列挙体

        internal enum DesiredAccess : uint
        {
            GENERIC_READ = 0x80000000,
            GENERIC_WRITE = 0x40000000,
            GENERIC_EXECUTE = 0x20000000
        }

        internal enum ShareMode : uint
        {
            FILE_SHARE_READ = 0x00000001,
            FILE_SHARE_WRITE = 0x00000002,
            FILE_SHARE_DELETE = 0x00000004
        }

        internal enum CreationDisposition : uint
        {
            CREATE_NEW = 1,
            CREATE_ALWAYS = 2,
            OPEN_EXISTING = 3,
            OPEN_ALWAYS = 4,
            TRUNCATE_EXISTING = 5
        }

        internal enum FlagsAndAttributes : uint
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
        /// <summary>メールスロットハンドルを取得します。</summary>
        public SafeFileHandle Handle { get { return handle; } }
        private SafeFileHandle handle = null;

        /// <summary>スロット名称を取得します。 </summary>
        public string Name { get { return name; } }
        private readonly string name;

        /// <summary>最大メッセージバッファ長(ANSI文字長)を取得します。 </summary>
        public int MaxMessageSize { get { return maxMessageSize; } }
        private readonly int maxMessageSize;

        /// <summary>次のメッセージサイズを取得します。存在しない場合-1を返します。 </summary>
        public int NextMessageSize
        {
            get
            {
                int nextSize;
                bool rc = GetMailslotInfo(Handle, IntPtr.Zero, out nextSize, IntPtr.Zero, IntPtr.Zero);
                if (!rc) return -1;
                return nextSize;
            }
        }

        /// <summary>
        /// コンストラクタです。
        /// </summary>
        /// <param name="name">メールスロット名</param>
        /// <param name="maxMessageSize">最大メッセージバッファ長</param>
        public MailSlotBase(string name, int maxMessageSize)
        {
            this.name = name;
            this.maxMessageSize = maxMessageSize;
        }

        /// <summary>
        /// オブジェクトを破棄します。
        /// </summary>
        public virtual void Dispose()
        {
            CloseHandle();
        }
        private void CloseHandle()
        {
            if (handle == null) return;
            handle.Dispose();
            handle = null;
        }

        /// <summary>
        /// 読み込み用にメールスロットを開きます。
        /// </summary>
        protected void ReadOpen(TimeSpan readTimeout)
        {
            CloseHandle();
            int timeout = (int)readTimeout.TotalMilliseconds;
            if (readTimeout == TimeSpan.MaxValue) timeout = -1;
            if (timeout < 0) timeout = -1;
            handle = CreateMailslot(Name, (uint)MaxMessageSize, (uint)timeout, (IntPtr)0);
        }

    }
}