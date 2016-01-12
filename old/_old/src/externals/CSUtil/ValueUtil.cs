
namespace CSUtil
{
    /// <summary>
    /// 値型に関するユーティリティ。
    /// </summary>
    public static class ValueUtil
    {
        /// <summary>
        /// 値型の２値を入れ替えます。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="a"></param>
        /// <param name="b"></param>
        public static void Swap<T>(ref T a, ref T b)
        {
            T c = a;
            a = b;
            b = c;
        }
    }
}