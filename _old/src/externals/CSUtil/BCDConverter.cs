using System;
using System.Collections.Generic;

namespace CSUtil
{
    /// <summary>
    /// リトルエンディアン配置のBCDバイト列を変換するコンバータです。
    /// </summary>
    public static class BCDConverter
    {
        /// <summary>
        /// uint値をbyte配列に変換します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="length"></param>
        /// <returns></returns>
        public static byte[] GetBytes(uint value, int length)
        {
            byte[] rc = new byte[length];
            for (int i = 0; i < length; i++) {
                uint lo = value % 10;
                value /= 10;
                uint hi = value % 10;
                value /= 10;
                rc[i] = (byte)(hi << 4 | lo);
                if (value == 0) break;
            }
            return rc;
        }

        /// <summary>
        /// DateTime値を8byteのBSD表現に変換した配列を返します。
        /// </summary>
        /// <param name="value">DateTime値。</param>
        /// <returns></returns>
        public static byte[] GetBytes(DateTime value)
        {
            List<byte> rc = new List<byte>();
            rc.AddRange(GetBytes((uint)value.Year % 100, 1));
            rc.AddRange(GetBytes((uint)value.Month, 1));
            rc.AddRange(GetBytes((uint)value.Day, 1));
            rc.Add(0);
            rc.AddRange(BitConverter.GetBytes((int)value.TimeOfDay.TotalMilliseconds));
            return rc.ToArray();
        }


        /// <summary>
        /// 指定長byte配列より、UInt32値を返します。
        /// </summary>
        /// <param name="value">byte配列</param>
        /// <param name="startIndex">走査開始位置</param>
        /// <param name="length">配列長</param>
        /// <returns></returns>
        public static int ToInt32(byte[] value, int startIndex, int length)
        {
            int rc = 0;
            int radio = 1;
            for (int i = 0; i < length; i++) {
                byte b = value[startIndex + i];
                int lo = ((int)b) & 0x00F;
                int hi = (((int)b) & 0x0F0) >> 4;
                rc += (lo + hi * 10) * radio;
                radio *= 100;
            }
            return rc;
        }



        /// <summary>
        /// BCD表現の8byte配列よりDateTime値を取得します。
        /// 変換に失敗した場合、DateTime.MinValueを返します。
        /// </summary>
        /// <param name="value">byte配列</param>
        /// <param name="startIndex">走査開始位置</param>
        /// <returns></returns>
        public static DateTime ToDateTime(byte[] value, int startIndex)
        {
            try {
                int yy = ToInt32(value, startIndex + 0, 1);
                int mm = ToInt32(value, startIndex + 1, 1);
                int dd = ToInt32(value, startIndex + 2, 1);
                int time = BitConverter.ToInt32(value, startIndex + 4);
                if (mm == 0 || dd == 0) return DateTime.MinValue;
                DateTime rc = new DateTime(CalcYear(yy), mm, dd);
                rc += TimeSpan.FromMilliseconds(time);
                return rc;
            }
            catch (Exception) {
                return DateTime.MinValue;
            }
        }

        /// <summary>
        /// ２桁年号を４桁に換算します。
        /// 入力される２桁年号は過去３０年より未来が指定されているとみなします。
        /// </summary>
        /// <param name="yy">２桁年号。</param>
        /// <returns></returns>
        public static int CalcYear(int yy)
        {
            int rc = yy + MIN_YEAR_OFFSET;
            if (rc < MIN_YEAR) rc += 100;
            return rc;
        }
        private static readonly int MIN_YEAR = DateTime.Now.Year - 30;
        private static readonly int MIN_YEAR_OFFSET = (MIN_YEAR / 100) * 100;

    }
}