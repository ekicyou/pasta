
namespace CSUtil
{
    /// <summary>
    /// 再入防止ロック。
    /// 最初に呼び出されたときだけRunメソッドのデリゲートを実行します。
    /// </summary>
    public class ReEntryLock
    {
        /// <summary>
        /// Runメソッドで実行するデリゲートです。
        /// </summary>
        public delegate void EnterFunc();
        private volatile bool isEnter = false;

        /// <summary>
        /// 最初の呼び出し（再入呼び出しではない）の時だけ、
        /// 指定したデリゲートを実行します。
        /// </summary>
        /// <param name="enterFunc"></param>
        public void Run(EnterFunc enterFunc)
        {
            lock (this) {
                if (isEnter) return;
                try {
                    isEnter = true;
                    enterFunc();
                }
                finally {
                    isEnter = false;
                }
            }
        }
    }
}