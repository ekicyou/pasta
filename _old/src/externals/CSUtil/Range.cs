using System.Collections.Generic;

namespace CSUtil
{
    /// <summary>
    /// 範囲指定した値のジェネレータ
    /// </summary>
    public static class Range
    {
        /// <summary>
        /// ２次元値範囲を列挙します。
        /// </summary>
        /// <param name="r1s"></param>
        /// <param name="r1e"></param>
        /// <param name="r2s"></param>
        /// <param name="r2e"></param>
        /// <returns></returns>
        public static IEnumerable<Tuple2<int, int>> Gen(
            int r1s, int r1e,
            int r2s, int r2e)
        {
            for (int r1 = r1s; r1 <= r1e; r1++)
                for (int r2 = r2s; r2 <= r2e; r2++)
                    yield return new Tuple2<int, int>(r1, r2);
        }

        /// <summary>
        /// ３次元値範囲を列挙します。
        /// </summary>
        /// <param name="r1s"></param>
        /// <param name="r1e"></param>
        /// <param name="r2s"></param>
        /// <param name="r2e"></param>
        /// <param name="r3s"></param>
        /// <param name="r3e"></param>
        /// <returns></returns>
        public static IEnumerable<Tuple3<int, int, int>> Gen(
            int r1s, int r1e,
            int r2s, int r2e,
            int r3s, int r3e)
        {
            for (int r1 = r1s; r1 <= r1e; r1++)
                for (int r2 = r2s; r2 <= r2e; r2++)
                    for (int r3 = r3s; r3 <= r3e; r3++)
                        yield return new Tuple3<int, int, int>(r1, r2, r3);
        }
    }
}