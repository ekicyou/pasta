using System;

namespace CSUtil.Random
{
    /// <summary>
    /// Ranrotの擬似乱数ジェネレータークラス。
    /// </summary>
    public sealed class RanrotB : RandomBase
    {

        #region Field

        /// <summary>
        /// 内部状態ベクトルの個数。
        /// </summary>
        private const int KK = 17;
        /// <summary>
        /// RanrotBのパラメーターの一つ。
        /// </summary>
        private const int JJ = 10;
        /// <summary>
        /// RanrotBのパラメーターの一つ。
        /// </summary>
        private const int R1 = 13;
        /// <summary>
        /// RanrotBのパラメーターの一つ。
        /// </summary>
        private const int R2 = 9;
        /// <summary>
        /// 内部状態ベクトル。
        /// </summary>
        private UInt32[] randbuffer;
        /// <summary>
        /// リングバッファのインデックス。
        /// </summary>
        private int p1, p2;

        #endregion

        /// <summary>
        /// 現在時刻を種とした、Well擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public RanrotB() : this(Environment.TickCount) { }

        /// <summary>
        /// seedを種とした、Well擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public RanrotB(int seed)
        {
            UInt32 s = (UInt32)seed;
            randbuffer = new UInt32[KK];
            for (int i = 0; i < KK; i++)
                randbuffer[i] = s = s * 2891336453 + 1;
            p1 = 0; p2 = JJ;
            for (int i = 0; i < 9; i++) NextUInt32();
        }

        /// <summary>
        /// 符号なし32bitの擬似乱数を取得します。
        /// </summary>
        public override uint NextUInt32()
        {
            UInt32 x;
            x = randbuffer[p1] = ((randbuffer[p2] << R1) | (randbuffer[p2] >> (32 - R1))) + ((randbuffer[p1] << R2) | (randbuffer[p1] >> (32 - R2)));
            if (--p1 < 0) p1 = KK - 1;
            if (--p2 < 0) p2 = KK - 1;
            return x;
        }

    }

}