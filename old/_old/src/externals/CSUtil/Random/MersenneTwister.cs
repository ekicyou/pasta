using System;

namespace CSUtil.Random
{
    /// <summary>
    /// MersenneTwisterの擬似乱数ジェネレータークラス。
    /// </summary>
    public sealed class MersenneTwister : RandomBase
    {

        #region Field

        /// <summary>
        /// 内部状態ベクトル総数
        /// </summary>
        private const int N = 624;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const int M = 397;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const uint MATRIX_A = 0x9908b0dfU;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const uint UPPER_MASK = 0x80000000U;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const uint LOWER_MASK = 0x7fffffffU;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const uint TEMPER1 = 0x9d2c5680U;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const uint TEMPER2 = 0xefc60000U;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const int TEMPER3 = 11;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const int TEMPER4 = 7;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const int TEMPER5 = 15;
        /// <summary>
        /// MTを決定するパラメーターの一つ。
        /// </summary>
        private const int TEMPER6 = 18;

        /// <summary>
        /// 内部状態ベクトル。
        /// </summary>
        private UInt32[] mt;
        /// <summary>
        /// 内部状態ベクトルのうち、次に乱数として使用するインデックス。
        /// </summary>
        private int mti;
        private UInt32[] mag01;

        #endregion

        /// <summary>
        /// 現在時刻を種とした、MersenneTwister擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public MersenneTwister() : this(Environment.TickCount) { }

        /// <summary>
        /// seedを種とした、MersenneTwister擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public MersenneTwister(int seed)
        {
            mt = new UInt32[N];
            mti = N + 1;
            mag01 = new UInt32[] { 0x0U, MATRIX_A };
            //内部状態配列初期化
            mt[0] = (UInt32)seed;
            for (int i = 1; i < N; i++)
                mt[i] = (UInt32)(1812433253 * (mt[i - 1] ^ (mt[i - 1] >> 30)) + i);
        }

        /// <summary>
        /// 符号なし32bitの擬似乱数を取得します。
        /// </summary>
        public override uint NextUInt32()
        {
            UInt32 y;
            if (mti >= N) { gen_rand_all(); mti = 0; }
            y = mt[mti++];
            y ^= (y >> TEMPER3);
            y ^= (y << TEMPER4) & TEMPER1;
            y ^= (y << TEMPER5) & TEMPER2;
            y ^= (y >> TEMPER6);
            return y;
        }

        /// <summary>
        /// 内部状態ベクトルを更新します。
        /// </summary>
        private void gen_rand_all()
        {
            int kk = 1;
            UInt32 y;
            UInt32 p;
            y = mt[0] & UPPER_MASK;
            do {
                p = mt[kk];
                mt[kk - 1] = mt[kk + (M - 1)] ^ ((y | (p & LOWER_MASK)) >> 1) ^ mag01[p & 1];
                y = p & UPPER_MASK;
            } while (++kk < N - M + 1);
            do {
                p = mt[kk];
                mt[kk - 1] = mt[kk + (M - N - 1)] ^ ((y | (p & LOWER_MASK)) >> 1) ^ mag01[p & 1];
                y = p & UPPER_MASK;
            } while (++kk < N);
            p = mt[0];
            mt[N - 1] = mt[M - 1] ^ ((y | (p & LOWER_MASK)) >> 1) ^ mag01[p & 1];
        }

    }
}