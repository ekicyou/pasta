using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;
using CSUtil.Collections.Generic;

namespace CSUtil.Threading
{
    /// <summary>
    /// イベントをキューにためて別スレッドで実行するためのクラスです。
    /// </summary>
    /// <typeparam name="T">EventArgsの派生クラス</typeparam>
    public abstract class EventQueueThread<T> : DisposableObject
    {
        /// <summary>
        /// スレッドのスタートアップコードを設定するメソッドです。
        /// オーバーライドしてください。
        /// </summary>
        protected abstract void StartUp();

        /// <summary>
        /// キューから値を受け取ったときの処理を記載するメソッドです。
        /// オーバーライドしてください。
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="ev"></param>
        protected abstract void Dequeue(object sender, T ev);


        /// <summary>
        /// イベントを保管している同期キューを取得します。
        /// </summary>
        private SyncWaitQueue<KeyValuePair<object, T>> queue = new SyncWaitQueue<KeyValuePair<object, T>>();

        /// <summary>
        /// キューの現在容量を取得します。
        /// </summary>
        public int Count { get { return queue.Count; } }

        /// <summary>
        /// キューの最大容量を取得または設定します。
        /// </summary>
        public int MaxCount
        {
            get { return queue.MaxCount; }
            set { queue.MaxCount = value; }
        }

        /// <summary>
        /// スレッドを取得します。
        /// </summary>
        public Thread Thread { get { return thread; } }
        private Thread thread = null;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public EventQueueThread()
        {
            thread = new Thread(ThreadMain);
            thread.Name = this.GetType().Name;
            thread.IsBackground = true;
        }

        /// <summary>
        /// マネージオブジェクトのDispose処理を行ないます。
        /// スレッドの停止要求を発行します。
        /// </summary>
        protected override void DisposeManage()
        {
            Debug.WriteLine(GetType().Name + ":Dispose開始");
            try {
                queue.Dispose();
                base.DisposeManage();
            }
            finally {
                Debug.WriteLine(GetType().Name + ":Dispose終了");
            }
        }

        /// <summary>
        /// スレッドの停止を待ちます。
        /// </summary>
        public void Join()
        {
            Debug.WriteLine(GetType().Name + ":Join開始");
            try {
                thread.Join();
                thread = null;
                queue = null;
            }
            finally {
                Debug.WriteLine(GetType().Name + ":Join終了");
            }
        }


        /// <summary>
        /// スレッドを開始します。
        /// </summary>
        public void Start()
        {
            Thread.Start();
        }

        /// <summary>
        /// キューの最後にイベントを登録します。
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="ev"></param>
        public void Enqueue(object sender, T ev)
        {
            queue.Enqueue(new KeyValuePair<object, T>(sender, ev));
            if (queue.Count > warningQueueCount) {
                Trace.WriteLine(GetType().Name + ":未処理キューが[" + warningQueueCount + "]を超えました。");
                warningQueueCount += WARNING_QUEUE_COUNT_SPAN;
            }
        }
        private const int WARNING_QUEUE_COUNT_SPAN = 1000;
        private int warningQueueCount = WARNING_QUEUE_COUNT_SPAN;

        /// <summary>
        /// スレッド本体です。
        /// キューから値を取り出し、イベントハンドラに渡します。
        /// </summary>
        protected virtual void ThreadMain()
        {
            Debug.WriteLine(GetType().Name + ":スレッド開始");
            try {
                StartUp();
                while (Thread.IsAlive) {
                    // キューから要素をひとつ取り出して処理する
                    KeyValuePair<object, T> item;
                    try {
                        item = queue.Dequeue();
                    }
                    catch (ObjectDisposedException) {
                        // キューが停止したため終了
                        return;
                    }
                    Dequeue(item.Key, item.Value);
                }
            }
            finally {
                Debug.WriteLine(GetType().Name + ":スレッド終了");
            }
        }
    }
}