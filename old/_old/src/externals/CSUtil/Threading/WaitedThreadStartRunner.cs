using System.Threading;

namespace CSUtil.Threading
{
    /// <summary>
    /// スレッドの起動待機を行うための処理クラスです。
    /// </summary>
    public sealed class WaitedThreadStartRunner
    {
        private object l = new object();
        private Thread thread;

        /// <summary>
        /// 新しく作成したスレッド。
        /// </summary>
        public Thread Thread { get { return thread; } }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="start"></param>
        public WaitedThreadStartRunner(ThreadStart start)
        {
            thread = new Thread(GetWaitThreadStart(start));
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="start"></param>
        /// <param name="maxStackSize"></param>
        public WaitedThreadStartRunner(ThreadStart start, int maxStackSize)
        {
            thread = new Thread(GetWaitThreadStart(start), maxStackSize);
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="start"></param>
        public WaitedThreadStartRunner(ParameterizedThreadStart start)
        {
            thread = new Thread(GetWaitThreadStart(start));
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="start"></param>
        /// <param name="maxStackSize"></param>
        public WaitedThreadStartRunner(ParameterizedThreadStart start, int maxStackSize)
        {
            thread = new Thread(GetWaitThreadStart(start), maxStackSize);
        }

        /// <summary>
        /// スレッドを開始し、スレッドが実際に開始するまで待機します。
        /// </summary>
        public void Start()
        {
            lock (l) {
                Thread.Start();
                Wait();
            }
        }

        /// <summary>
        /// スレッドを開始し、スレッドが実際に開始するまで待機します。
        /// </summary>
        /// <param name="parameter"></param>
        public void Start(object parameter)
        {
            lock (l) {
                Thread.Start(parameter);
                Wait();
            }
        }

        private ParameterizedThreadStart GetWaitThreadStart(ParameterizedThreadStart start)
        {
            return delegate(object arg) { Pulse(); start(arg); };
        }
        private ThreadStart GetWaitThreadStart(ThreadStart start)
        {
            return delegate() { Pulse(); start(); };
        }

        private void Wait()
        {
        }

        private void Pulse()
        {
        }

    }
}