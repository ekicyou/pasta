using System;

namespace CSUtil.Random
{
    /// <summary>
    /// LCGの擬似乱数ジェネレータークラス。
    /// </summary>
    public sealed class LCG : RandomBase
    {

        #region Field

        /// <summary>
        /// LCGのパラメーターの一つ。
        /// </summary>
        private UInt32 A;
        /// <summary>
        /// LCGのパラメーターの一つ。
        /// </summary>
        private UInt32 C;
        /// <summary>
        /// 内部状態。
        /// </summary>
        private UInt32 x;

        #endregion

        /// <summary>
        /// 現在時刻を種とした、LCG擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public LCG() : this(Environment.TickCount) { }

        /// <summary>
        /// seedを種とした、LCG擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public LCG(int seed) : this(seed, 1664525, 1013904223) { }

        /// <summary>
        /// seedを種とし、paramAとparamCで表されるLCG擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public LCG(int seed, UInt32 paramA, UInt32 paramC)
        {
            A = paramA;
            C = paramC;
            x = (UInt32)seed;
        }

        /// <summary>
        /// 符号なし32bitの擬似乱数を取得します。
        /// </summary>
        public override uint NextUInt32()
        {
            x = (UInt32)((UInt64)x * A + C);
            return x;
        }

    }

}