using System;

namespace CSUtil.Random
{
    /// <summary>
    /// Mother-of-Allの擬似乱数ジェネレータークラス。
    /// </summary>
    public sealed class MotherOfAll : RandomBase
    {

        /// <summary>
        /// 内部状態ベクトル。
        /// </summary>
        private UInt32 x, y, z, w, v;

        /// <summary>
        /// 現在時刻を種とした、Mother-Of-All擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public MotherOfAll() : this(Environment.TickCount) { }

        /// <summary>
        /// seedを種とした、Mother-Of-All擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public MotherOfAll(int seed)
        {
            UInt32 s = (UInt32)seed;
            x = s = 29943829 * s - 1;
            y = s = 29943829 * s - 1;
            z = s = 29943829 * s - 1;
            w = s = 29943829 * s - 1;
            v = s = 29943829 * s - 1;
            for (int i = 0; i < 19; i++) NextUInt32();
        }

        /// <summary>
        /// 符号なし32bitの擬似乱数を取得します。
        /// </summary>
        public override uint NextUInt32()
        {
            UInt64 s = 2111111111UL * w + 1492UL * z + 1776UL * y + 5115UL * x + v;
            w = z; z = y; y = x; x = (UInt32)s;
            v = (UInt32)(s >> 32);
            return x;
        }
    }

}