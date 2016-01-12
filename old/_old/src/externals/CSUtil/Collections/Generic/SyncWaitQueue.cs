using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;

namespace CSUtil.Collections.Generic
{

    /// <summary>
    /// キューが存在しない場合にスレッドを停止して待機するキューです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class SyncWaitQueue<T> : DisposableObject
    {
        private Queue<T> queue = new Queue<T>();
        private int maxCount = 0;
        private const int ReadTimeout = 3000;
        private object lockObject = new object();

        /// <summary>
        /// SyncWaitQueue に格納されている要素の数を取得します。
        /// </summary>
        public int Count { get { return queue.Count; } }

        /// <summary>
        /// キュー要素の最大値を設定または取得します。
        /// ０の場合、サイズに制限はありません。
        /// </summary>
        public int MaxCount
        {
            get { return maxCount; }
            set { maxCount = value; }
        }

        /// <summary>
        /// Dispose処理を実装します。以下のモードに切り替わります。
        /// ・Enqueueを無視
        /// ・DequeueでキューがなくなったときにObjectDisposedException例外
        /// ・Dequeueのロックを解除する
        /// </summary>
        protected override void DisposeManage()
        {
            lock (lockObject) {
                Monitor.PulseAll(lockObject);
            }
        }

        /// <summary>
        /// SyncWaitQueue の先頭にあるオブジェクトを削除し、返します。
        /// データが無い場合、データが追加されるまでロックします。
        /// オブジェクトが破棄され、かつキューが０になった場合、
        /// ObjectDisposedException例外をスローします。
        /// </summary>
        /// <returns></returns>
        public T Dequeue()
        {
            WaitEnqueue();
            lock (lockObject) {
                try {
                    return queue.Dequeue();
                }
                catch (InvalidOperationException) {
                    if (IsDisposed) throw new ObjectDisposedException("SyncWaitQueueは停止状態です。");
                    throw;
                }
            }
        }

        /// <summary>
        /// SyncWaitQueue に値が設定されるまで待機します。
        /// キューにストックが無く、オブジェクトが破棄された場合、
        /// ObjectDisposedException例外をスローします。
        /// </summary>
        private void WaitEnqueue()
        {
            if (queue.Count > 0) return;
            while (true) {
                bool isTimeout;
                lock (lockObject) {
                    if (queue.Count > 0) return;
                    if (IsDisposed) throw new ObjectDisposedException("SyncWaitQueueは停止状態です。");
                    isTimeout = Monitor.Wait(lockObject, ReadTimeout);
                }
                if (isTimeout) Thread.Sleep(100);
            }
        }

        /// <summary>
        /// SyncWaitQueue の末尾にオブジェクトを追加します。。
        /// 取り出し処理がロックされている場合、解除します。
        /// </summary>
        /// <param name="item"></param>
        public void Enqueue(T item)
        {
            if (IsDisposed) return;
            lock (lockObject) {
                if (MaxCount > 0) {
                    if (queue.Count >= MaxCount) {
                        throw new ArgumentOutOfRangeException("MaxCount", "MaxCountを超えてキューに登録することは出来ません。");
                    }
                }
                queue.Enqueue(item);
                if (queue.Count >= warnCount) {
                    string name = this.GetType().Name;
                    StackTrace st = new StackTrace(2, true);
                    Trace.WriteLine(string.Format("[{0}]:未処理キューが{1}に達しました。\n{2}", name, warnCount, st));
                    warnCount += WARN_COUNT_SPAN;
                }
                Monitor.PulseAll(lockObject);
            }
        }

        private const int WARN_COUNT_SPAN = 1000;
        private int warnCount = WARN_COUNT_SPAN;

    }

}