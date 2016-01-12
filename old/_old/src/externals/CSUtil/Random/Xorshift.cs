using System;

namespace CSUtil.Random
{
    /// <summary>
    /// Xorshiftの擬似乱数ジェネレータークラス。
    /// </summary>
    public sealed class Xorshift : RandomBase
    {

        /// <summary>
        /// 内部状態ベクトル。
        /// </summary>
        private UInt32 x, y, z, w;

        /// <summary>
        /// 現在時刻を種とした、Xorshift擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public Xorshift() : this(Environment.TickCount) { }

        //static unsigned long x=123456789,y=362436069,z=521288629,w=88675123;

        /// <summary>
        /// seedを種とした、Xorshift擬似乱数ジェネレーターを初期化します。
        /// </summary>
        public Xorshift(int seed) : this((UInt32)seed, 362436069, 521288629, 88675123) { }

        /// <summary>
        /// seedを種とした、Xorshift擬似乱数ジェネレーターを初期化します。
        /// George Marsagliaによるオリジナルはseed1=123456789,seed2=362436069,seed3=521288629,seed4=88675123を用いています。
        /// </summary>
        /// <param name="seed1"></param>
        /// <param name="seed2"></param>
        /// <param name="seed3"></param>
        /// <param name="seed4"></param>
        public Xorshift(UInt32 seed1, UInt32 seed2, UInt32 seed3, UInt32 seed4)
        {
            x = seed1; y = seed2; z = seed3; w = seed4;
        }

        /// <summary>
        /// 符号なし32bitの擬似乱数を取得します。
        /// </summary>
        public override uint NextUInt32()
        {
            UInt32 t;
            t = (x ^ (x << 11));
            x = y; y = z; z = w;
            return (w = (w ^ (w >> 19)) ^ (t ^ (t >> 8)));
        }
    }
}