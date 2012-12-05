using System;
using System.Collections.Generic;
using System.Text;

namespace System.Collections.Generic
{
    /// <summary>
    /// KeyValuePairユーティリティ。
    /// </summary>
    public static class KVUtil
    {
        /// <summary>
        /// KeyValuePairを作成します。
        /// </summary>
        /// <typeparam name="TKey"></typeparam>
        /// <typeparam name="TValue"></typeparam>
        /// <param name="key"></param>
        /// <param name="value"></param>
        /// <returns></returns>
        public static KeyValuePair<TKey, TValue> Create<TKey, TValue>(TKey key, TValue value)
        {
            return new KeyValuePair<TKey, TValue>(key, value);
        }
    }
}
