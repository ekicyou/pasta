using System;
using System.Collections.Generic;
using System.Text;

namespace CSUtil
{
    /// <summary>
    /// 値値の配列走査のためのジェネリックユーティティです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public static class ArrayUtil<T> where T : IEquatable<T>
    {
        /// <summary>
        /// お互いの要素がすべて一致したときにtrueを返します。
        /// いずれかがnullの場合は常にfalseを返します。
        /// </summary>
        /// <param name="src"></param>
        /// <param name="dest"></param>
        /// <returns></returns>
        public static bool Equals(T[] src, T[] dest)
        {
            if (src == null) return false;
            if (dest == null) return false;
            if (src.Length != dest.Length) return false;
            for (int i = 0; i < src.Length; i++) {
                if (!src[i].Equals(dest[i])) return false;
            }
            return true;
        }

        /// <summary>
        /// お互いの要素がすべて一致したときにtrueを返します。
        /// いずれかがnullの場合は常にfalseを返します。
        /// </summary>
        /// <param name="src"></param>
        /// <param name="dest"></param>
        /// <returns></returns>
        public static bool Equals(ICollection<T> src, ICollection<T> dest)
        {
            if (src == null) return false;
            if (dest == null) return false;
            if (src.Count != dest.Count) return false;
            return EqualsSubIEnumerable(src, dest);
        }

        /// <summary>
        /// お互いの要素がすべて一致したときにtrueを返します。
        /// いずれかがnullの場合は常にfalseを返します。
        /// </summary>
        /// <param name="src"></param>
        /// <param name="dest"></param>
        /// <returns></returns>
        public static bool Equals(IEnumerable<T> src, IEnumerable<T> dest)
        {
            if (src == null) return false;
            if (dest == null) return false;
            return EqualsSubIEnumerable(src, dest);
        }

        private static bool EqualsSubIEnumerable(IEnumerable<T> src, IEnumerable<T> dest)
        {
            IEnumerator<T> dEn = dest.GetEnumerator();
            foreach (T sVal in src) {
                if (!dEn.MoveNext()) return false;
                if (!sVal.Equals(dEn.Current)) return false;
            }
            return !dEn.MoveNext();
        }

        /// <summary>
        /// 値を文字列に変換するデリゲート
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public delegate string ToStringDelgate(T value);

        /// <summary>
        /// 列挙要素をダンプしてテキストに変換します。
        /// </summary>
        /// <param name="values"></param>
        /// <param name="toString"></param>
        /// <param name="lineFeedSpan"></param>
        /// <param name="lineHeaderFormat"></param>
        /// <returns></returns>
        public static string DumpText(IEnumerable<T> values, ToStringDelgate toString, int lineFeedSpan, string lineHeaderFormat)
        {
            StringBuilder buf = new StringBuilder();
            int count = 0;
            foreach (T value in values) {
                if ((count % lineFeedSpan) == 0) {
                    if (count != 0) buf.AppendLine();
                    buf.AppendFormat(count.ToString(lineHeaderFormat) + ": ");
                }
                else {
                    buf.Append(' ');
                }
                count++;
                buf.AppendFormat(toString(value));
            }
            buf.AppendLine();
            return buf.ToString();
        }

        /// <summary>
        /// １６進ダンプのテキストを作成します。
        /// </summary>
        /// <param name="values"></param>
        /// <returns></returns>
        public static string DumpHexText(IEnumerable<T> values)
        {
            return DumpText(
              values, ToHex2Text, 16, "X8");
        }

        private static string ToHex2Text(T value)
        {
            return string.Format("{0:X2}", value);
        }

    }
}