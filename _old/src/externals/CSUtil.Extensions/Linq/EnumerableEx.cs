using System.Collections.Generic;

namespace System.Linq
{
    /// <summary>
    /// 独自追加のEnumerableモジュールです。
    /// </summary>
    public static class EnumerableEx
    {
        /// <summary>
        /// IEnumerable型に変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="src"></param>
        /// <returns></returns>
        public static IEnumerable<T> ToEnumerable<T>(this IEnumerable<T> src) { return src; }

        /// <summary>
        /// 要素をいちばんうしろから指定した数だけ取得します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="items"></param>
        /// <param name="count"></param>
        /// <returns></returns>
        public static IEnumerable<T> TakeLast<T>(this IEnumerable<T> items, int count)
        {
            return items
                .Reverse()
                .Take(count)
                .Reverse();
        }

        /// <summary>
        /// 要素を追加します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="items"></param>
        /// <param name="add"></param>
        /// <returns></returns>
        public static IEnumerable<T> Add<T>(this IEnumerable<T> items, params T[] add)
        {
            return items.Concat(add);
        }

        /// <summary>
        /// 現在と次の値をペアにして返します。
        /// </summary>
        /// <param name="items"></param>
        /// <returns></returns>
        public static IEnumerable<NowNextPair<T>> NowNext<T>(this IEnumerable<T> items)
        {
            var gen = items.GetEnumerator();
            // 初回
            if (!gen.MoveNext()) yield break;
            T now = gen.Current;

            // 次回以降
            while (gen.MoveNext()) {
                var next=gen.Current;
                yield return new NowNextPair<T>(now, next);
                now = next;
            }
        }

        /// <summary>
        /// 現在と次の値のペア
        /// </summary>
        /// <typeparam name="T"></typeparam>
        public struct NowNextPair<T>
        {
            private readonly T now;
            private readonly T next;

            /// <summary>今の値</summary>
            public T Now { get { return now; } }

            /// <summary>次の値</summary>
            public T Next { get { return next; } }

            public NowNextPair(T now, T next)
            {
                this.now = now;
                this.next = next;
            }
        }

        /// <summary>
        /// 渡されたシーケンスがソースに含まれていれば、
        /// シーケンスが始まる場所を返します。
        /// 含まれていなければ-1を返します。
        /// </summary>
        /// <typeparam name="TSource"></typeparam>
        /// <param name="source"></param>
        /// <param name="target"></param>
        /// <returns></returns>
        public static int ContainsIndex<TSource>(
            this IEnumerable<TSource> source,
            IEnumerable<TSource> target)
        {
            var srcCnt = source.Count();
            var targetCnt = target.Count();
            for (int i = 0; i <= srcCnt - targetCnt; i++) {
                var qCheck = source.Skip(i).Take(targetCnt);
                if (target.SequenceEqual(qCheck)) return i;
            }
            return -1;
        }

        /// <summary>
        /// 要素の列挙後、指定された条件が満たされなくなったら、列挙を停止します。
        /// </summary>
        /// <typeparam name="TSource"></typeparam>
        /// <param name="source"></param>
        /// <param name="predicate"></param>
        /// <returns></returns>
        public static IEnumerable<TSource> TakeDoWhile<TSource>(
              this IEnumerable<TSource> source,
              Func<TSource, bool> predicate
          )
        {
            foreach(var item in source) {
                yield return item;
                if(!predicate(item)) yield break;
            }
        }
  

        /// <summary>
        /// 配列の要素を反転して列挙します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="items"></param>
        /// <returns></returns>
        public static IEnumerable<T> Reverse<T>(this T[] items)
        {
            for(var i = items.Length - 1; i >= 0; i--) yield return items[i];
        }

    }
}