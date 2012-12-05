using System;
using System.Collections.Generic;
using System.Text;

namespace CSUtil
{
    /// <summary>
    /// オブジェクト操作ユーティリティ。
    /// </summary>
    public static class ObjectUtil
    {
        /// <summary>
        /// 値型の２値を入れ替えます。
        /// </summary>
        /// <param name="a"></param>
        /// <param name="b"></param>
        public static void Swap<T>(ref T a, ref T b)
        {
            T c = a;
            a = b;
            b = c;
        }

        /// <summary>
        /// Disposeを行い、変数を初期化します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="obj"></param>
        public static void CheckDispose<T>(ref T obj)
            where T : IDisposable
        {
            if (obj == null) return;
            obj.Dispose();
            obj = default(T);
        }

    }
}
