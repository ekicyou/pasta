using System;
using System.Collections.Generic;
using System.Text;
using System.Threading;

namespace CSUtil.Threading
{
    /// <summary>
    /// 再入実行かどうかを判定するカウンタ。
    /// Interlockedを利用していますが、lockは行っていません。
    /// マルチスレッドには対応して居ません。
    /// </summary>
    public class ReentryCounter
    {
        private int firstEntry = 0;
        private int count = 0;


        /// <summary>再入カウント。値が０にデクリメントされたときに再入フラグをリセットする。</summary>
        public int Count { get { return count; } }

        /// <summary>
        /// インクリメント。
        /// </summary>
        /// <returns></returns>
        public int Increment()
        {
            return Interlocked.Increment(ref count);
        }

        /// <summary>
        /// デクリメント。０になったときに再入フラグをリセットします。
        /// </summary>
        /// <returns></returns>
        public int Decrement()
        {
            var cnt = Interlocked.Decrement(ref count);
            if (cnt < 0) throw new InvalidOperationException("再入カウンタがマイナス値を取りました。");
            if (cnt == 0) Reset();
            return cnt;
        }

        /// <summary>
        /// カウンタをリセットする。
        /// </summary>
        public void Reset()
        {
            firstEntry = 0;
            count = 0;
        }

        /// <summary>
        /// １回目の実行ならtrue。
        /// </summary>
        /// <returns></returns>
        public bool IsFirstEntry()
        {
            var org = Interlocked.CompareExchange(ref firstEntry, 1, 0);
            return org == 0;
        }

        /// <summary>
        /// アクションの実行。
        /// </summary>
        /// <param name="act"></param>
        public void Act(ThreadStart act)
        {
            try {
                Increment();
                act();
            }
            finally {
                Decrement();
            }
        }

    }
}
