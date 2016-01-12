
namespace CSUtil
{
    /// <summary>
    /// ２値のタプルをあらわします。
    /// </summary>
    /// <typeparam name="TA"></typeparam>
    /// <typeparam name="TB"></typeparam>
    public struct Tuple2<TA, TB>
    {
        /// <summary>
        /// 要素A。
        /// </summary>
        public readonly TA A;

        /// <summary>
        /// 要素B。
        /// </summary>
        public readonly TB B;

        /// <summary>
        /// 要素X(=A)
        /// </summary>
        public TA X { get { return A; } }

        /// <summary>
        /// 要素Y(=B)
        /// </summary>
        public TB Y { get { return B; } }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="a"></param>
        /// <param name="b"></param>
        public Tuple2(TA a, TB b)
        {
            A = a;
            B = b;
        }

        /// <summary>
        /// 文字列表現を返します。
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return string.Format("[{0},{1}]", A, B);
        }
    }


    /// <summary>
    /// ３値のタプルをあらわします。
    /// </summary>
    /// <typeparam name="TA"></typeparam>
    /// <typeparam name="TB"></typeparam>
    /// <typeparam name="TC"></typeparam>
    public struct Tuple3<TA, TB, TC>
    {
        /// <summary>
        /// 要素A。
        /// </summary>
        public readonly TA A;

        /// <summary>
        /// 要素B。
        /// </summary>
        public readonly TB B;

        /// <summary>
        /// 要素C。
        /// </summary>
        public readonly TC C;

        /// <summary>
        /// 要素X(=A)
        /// </summary>
        public TA X { get { return A; } }

        /// <summary>
        /// 要素Y(=B)
        /// </summary>
        public TB Y { get { return B; } }

        /// <summary>
        /// 要素Z(=C)
        /// </summary>
        public TC Z { get { return C; } }


        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="a"></param>
        /// <param name="b"></param>
        /// <param name="c"></param>
        public Tuple3(TA a, TB b, TC c)
        {
            A = a;
            B = b;
            C = c;
        }

        /// <summary>
        /// 文字列表現を返します。
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return string.Format("[{0},{1},{2}]", A, B, C);
        }
    }


    /// <summary>
    /// ４値のタプルをあらわします。
    /// </summary>
    /// <typeparam name="TA"></typeparam>
    /// <typeparam name="TB"></typeparam>
    /// <typeparam name="TC"></typeparam>
    /// <typeparam name="TD"></typeparam>
    public struct Tuple4<TA, TB, TC, TD>
    {
        /// <summary>
        /// 要素A。
        /// </summary>
        public readonly TA A;

        /// <summary>
        /// 要素B。
        /// </summary>
        public readonly TB B;

        /// <summary>
        /// 要素C。
        /// </summary>
        public readonly TC C;

        /// <summary>
        /// 要素D。
        /// </summary>
        public readonly TD D;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="a"></param>
        /// <param name="b"></param>
        /// <param name="c"></param>
        /// <param name="d"></param>
        public Tuple4(TA a, TB b, TC c, TD d)
        {
            A = a;
            B = b;
            C = c;
            D = d;
        }

        /// <summary>
        /// 文字列表現を返します。
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return string.Format("[{0},{1},{2},{3}]", A, B, C, D);
        }
    }

}