using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Threading
{
    /// <summary>
    /// 再入実行チェックカウンタの拡張。
    /// </summary>
    public static class ReentryCounterExtensions
    {
        /// <summary>
        /// 関数に再入チェックを追加します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="counter"></param>
        /// <param name="func"></param>
        /// <returns></returns>
        public static T Func<T>(this ReentryCounter counter, Func<T> func)
        {
            try {
                counter.Increment();
                return func();
            }
            finally {
                counter.Decrement();
            }
        }

    }
}
