using System;

namespace CSUtil
{
    /// <summary>
    /// ビックエンディアンバイト列の相互変換。
    /// </summary>
    public static class BigEndianBitConverter
    {
        private static byte[] ResultReverse(byte[] buf)
        {
            if (BitConverter.IsLittleEndian) Array.Reverse(buf);
            return buf;
        }

        /// <summary>
        /// 指定したbool値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(bool value) { return BitConverter.GetBytes(value); }

        /// <summary>
        /// 指定したchar値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(char value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したshort値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(short value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したushort値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(ushort value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したint値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(int value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したuint値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(uint value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したlong値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(long value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したulong値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(ulong value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したfloat値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(float value) { return ResultReverse(BitConverter.GetBytes(value)); }

        /// <summary>
        /// 指定したdouble値をbyte配列として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <returns></returns>
        public static byte[] GetBytes(double value) { return ResultReverse(BitConverter.GetBytes(value)); }

        private static byte[] CopyReverseBytes(byte[] src, int startIndex, int length)
        {
            byte[] dest = new byte[length];
            Array.Copy(src, startIndex, dest, 0, length);
            return ResultReverse(dest);
        }

        /// <summary>
        /// byte配列の指定位置から始まる1byteのデータをbool値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static bool ToBoolean(byte[] value, int startIndex)
        {
            return BitConverter.ToBoolean(value, startIndex);
        }

        /// <summary>
        /// byte配列の指定位置から始まる2byteのデータをchar値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static char ToChar(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(char));
            return BitConverter.ToChar(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる8byteのデータをdouble値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static double ToDouble(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(double));
            return BitConverter.ToDouble(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる2byteのデータをshort値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static short ToInt16(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(short));
            return BitConverter.ToInt16(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる4byteのデータをint値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static int ToInt32(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(int));
            return BitConverter.ToInt32(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる8byteのデータをlong値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static long ToInt64(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(long));
            return BitConverter.ToInt64(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる4byteのデータをfloat値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static float ToSingle(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(float));
            return BitConverter.ToSingle(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる2byteのデータをushort値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static ushort ToUInt16(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(ushort));
            return BitConverter.ToUInt16(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる4byteのデータをuint値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static uint ToUInt32(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(uint));
            return BitConverter.ToUInt32(bigValue, 0);
        }

        /// <summary>
        /// byte配列の指定位置から始まる8byteのデータをulong値として返します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="startIndex"></param>
        /// <returns></returns>
        public static ulong ToUInt64(byte[] value, int startIndex)
        {
            byte[] bigValue = CopyReverseBytes(value, startIndex, sizeof(ulong));
            return BitConverter.ToUInt64(bigValue, 0);
        }


    }
}