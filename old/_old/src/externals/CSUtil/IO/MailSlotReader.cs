using System;
using System.Collections.Generic;
using System.IO;

namespace CSUtil.IO
{
    /// <summary>
    /// メールスロットの読込クラスです。
    /// </summary>
    public class MailSlotReader : MailSlotBase, IEnumerable<byte[]>
    {
        private FileStream fs;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name">メールスロット名</param>
        /// <param name="maxMessageSize">最大メッセージバッファ長</param>
        public MailSlotReader(string name, int maxMessageSize)
            : base(name, maxMessageSize)
        {
            ReadOpen(TimeSpan.Zero);
            fs = new FileStream(Handle, FileAccess.Read);
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name">メールスロット名</param>
        public MailSlotReader(string name)
            : base(name, 0)
        {
            ReadOpen(TimeSpan.Zero);
            fs = new FileStream(Handle, FileAccess.Read);
        }

        /// <summary>
        /// 開放処理を行います。
        /// </summary>
        public override void Dispose()
        {
            if (fs != null) fs.Dispose();
            base.Dispose();
        }

        /// <summary>
        /// 残っているメッセージを全て読み出します。
        /// </summary>
        public void Clear()
        {
            foreach (byte[] buf in this) ;
        }

        /// <summary>
        /// メッセージがなくなるまで列挙します。
        /// </summary>
        /// <returns></returns>
        public IEnumerable<byte[]> EnBytes()
        {
            int nextSize;
            while ((nextSize = NextMessageSize) >= 0) {
                byte[] buf = new byte[nextSize];
                int len = fs.Read(buf, 0, buf.Length);
                yield return buf;
            }
        }

        private IEnumerable<object> EnBytesObject()
        {
            foreach (byte[] mes in EnBytes()) yield return mes;
        }

        /// <summary>
        /// メッセージが無くなるまで列挙します。
        /// </summary>
        /// <returns></returns>
        public IEnumerator<byte[]> GetEnumerator()
        {
            return EnBytes().GetEnumerator();
        }

        /// <summary>
        /// メッセージが無くなるまで列挙します。
        /// </summary>
        /// <returns></returns>
        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator()
        {
            return EnBytesObject().GetEnumerator();
        }



    }
}