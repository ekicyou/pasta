using System.Collections.Generic;
using System.Diagnostics;

namespace CSUtil.Collections.Generic
{
    /// <summary>
    /// キューの遅延状況をTraceログに出力するQueue
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class SyncQueue<T>
    {
        private Queue<T> queue;

        /// <summary>
        /// キューの名前（デバッグログ出力用）
        /// </summary>
        public readonly string Name;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name"></param>
        public SyncQueue(string name)
        {
            Name = name;
            queue = new Queue<T>();
        }

        /// <summary>
        /// キューに格納されている要素の数を返します。
        /// </summary>
        public int Count { get { return queue.Count; } }

        /// <summary>
        /// 同期オブジェクトを返します。
        /// </summary>
        public object SyncRoot { get { return queue; } }

        /// <summary>
        /// キューを全て取り出す列挙子を返します。
        /// </summary>
        /// <returns></returns>
        public IEnumerator<T> GetEnumerator()
        {
            return EnumerableQueue().GetEnumerator();
        }
        private IEnumerable<T> EnumerableQueue()
        {
            if (Count == 0) yield break;

            // キューに１００以上の要素がある場合、
            // 新規にキューを作って受信を止めないようにする。
            Queue<T> q = queue;
            if (q.Count > 100) {
                lock (SyncRoot) {
                    queue = new Queue<T>(q.Count);
                }
                System.Threading.Thread.Sleep(0);
            }

            lock (q) {
                while (q.Count > 0) {
                    yield return q.Dequeue();
                }
            }
        }

        /// <summary>
        /// 現在のキューの内容を配列にして返します。
        /// </summary>
        /// <returns></returns>
        public T[] ToArray()
        {
            Queue<T> q = queue;
            lock (SyncRoot) {
                queue = new Queue<T>(q.Count);
            }
            return q.ToArray();
        }

        /// <summary>
        /// キューの末尾にオブジェクトを追加します。
        /// </summary>
        /// <param name="item"></param>
        public void Enqueue(T item)
        {
            lock (SyncRoot) {
                queue.Enqueue(item);
                if (Count < warnCount) return;
                Trace.WriteLine(string.Format("[{0} Queue]:未処理キューが{1}に達しました。", Name, warnCount));
                warnCount += WARN_COUNT_SPAN;
            }
        }

        private const int WARN_COUNT_SPAN = 2000;
        private int warnCount = WARN_COUNT_SPAN;

        /// <summary>
        /// 要素数が指定サイズを超えたときに、指定サイズまで古い要素を捨てます。
        /// </summary>
        /// <param name="count"></param>
        public void FixSize(int count)
        {
            lock (SyncRoot) {
                while (queue.Count > count) queue.Dequeue();
            }
        }



    }
}