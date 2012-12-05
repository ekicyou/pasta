using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace System.Linq
{
    /// <summary>
    /// 独自追加のEnumerableモジュールです。.Net4.0以降のメソッドを利用します。
    /// </summary>
    public static class EnumerableEx40
    {
        /// <summary>
        /// HashSet 型に変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="src"></param>
        /// <returns></returns>
        public static HashSet<T> ToHashSet<T>(this IEnumerable<T> src) {
            return new HashSet<T>(src);
        }

    }
}
