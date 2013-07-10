using System.Collections.Generic;

namespace System.Linq
{
    /// <summary>
    /// .net4.0に存在する関数を模擬します。
    /// </summary>
    public static class Enumerable4
    {
        /// <summary>
        /// 指定された述語関数を使用して 2 つのシーケンスをマージします。
        /// </summary>
        /// <typeparam name="TFirst">1 番目の入力シーケンスの要素の型。</typeparam>
        /// <typeparam name="TSecond">2 番目の入力シーケンスの要素の型。</typeparam>
        /// <typeparam name="TResult">結果のシーケンスの要素の型。</typeparam>
        /// <param name="first">マージする 1 番目のシーケンス。</param>
        /// <param name="second">マージする 2 番目のシーケンス。</param>
        /// <param name="resultSelector">2 つのシーケンスの要素をマージする方法を指定する関数。</param>
        /// <returns>2 つの入力シーケンスのマージされた要素が格納されている IEnumerable(T)。</returns>
        public static IEnumerable<TResult> Zip<TFirst, TSecond, TResult>(
            this IEnumerable<TFirst> first,
            IEnumerable<TSecond> second,
            Func<TFirst, TSecond, TResult> resultSelector
        )
        {
            var g1 = first.GetEnumerator();
            var g2 = second.GetEnumerator();
            while (true) {
                if (!g1.MoveNext()) yield break;
                if (!g2.MoveNext()) yield break;
                yield return resultSelector(g1.Current, g2.Current);
            }
        }


    }
}