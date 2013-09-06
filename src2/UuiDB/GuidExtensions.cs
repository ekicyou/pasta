using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace UuiDB
{
    /// <summary>
    /// Guid拡張。
    /// </summary>
    public static class GuidExtensions
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        /// <summary>
        /// GUIDを２進数テキストに変換します。
        /// </summary>
        /// <param name="guid"></param>
        /// <returns></returns>
        public static string ToBitString(this Guid guid)
        {
            var texts = guid.ToByteArray()
                .Select(a => Convert.ToString(a, 2).PadLeft(8, '0'));
            return string.Join(" ", texts);
        }

        /// <summary>
        /// Guidのbyte配列を取得し、バージョン情報を後ろにシフトします。
        /// 1.バイナリ配列の並び替え
        /// IN : 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15
        /// OUT: 00 01 02 03 04 05 06 09 10 11 12 13 14 15  x  y
        ///   x = 07_4321 , 08_4321
        ///   y = 08_65 , 07_8765, 08_87
        /// </summary>
        /// <param name="guid"></param>
        /// <returns></returns>
        private static byte[] ToSwapByteArray(this Guid guid)
        {
            var buf = guid.ToByteArray();
            var b7 = (int)buf[7];
            var b8 = (int)buf[8];
            var b7_4321 = b7 & 0x0f;
            var b7_8765 = (b7 >> 4) & 0x0f;
            var b8_4321 = b8 & 0x0f;
            var b8_65 = (b8 >> 4) & 0x03;
            var b8_87 = (b8 >> 6) & 0x03;

            var x = (b7_4321 << 4) | (b8_4321);
            var y = (b8_65 << 6) | (b7_8765 << 2) | (b8_87);

            /*
            buf[0] = buf[0];
            buf[1] = buf[1];
            buf[2] = buf[2];
            buf[3] = buf[3];
            buf[4] = buf[4];
            buf[5] = buf[5];
            buf[6] = buf[6];
            */
            buf[7] = buf[9];
            buf[8] = buf[10];
            buf[9] = buf[11];
            buf[10] = buf[12];
            buf[11] = buf[13];
            buf[12] = buf[14];
            buf[13] = buf[15];
            buf[14] = (byte)x;
            buf[15] = (byte)y;

            return buf;
        }

        /// <summary>
        /// バイト配列の後ろにシフトされたGuid Version情報をもとに戻し、Guidを作成します。
        /// </summary>
        /// <param name="buf"></param>
        /// <returns></returns>
        private static Guid FromSwapByteArrayToGuid(this byte[] buf)
        {
            var x = (int)buf[14];
            var y = (int)buf[15];

            var b7_4321 = (x >> 4) & 0x0f; var b8_4321 = (x >> 0) & 0x0f;
            var b8_65 = (y >> 6) & 0x03; var b7_8765 = (y >> 2) & 0x0f; var b8_87 = (y >> 0) & 0x03;

            var b7 = (b7_8765 << 4) | (b7_4321 << 0);
            var b8 = (b8_87 << 6) | (b8_65 << 4) | (b8_4321 << 0);

            buf[15] = buf[13];
            buf[14] = buf[12];
            buf[13] = buf[11];
            buf[12] = buf[10];
            buf[11] = buf[9];
            buf[10] = buf[8];
            buf[9] = buf[7];
            buf[8] = (byte)b8;
            buf[7] = (byte)b7;
            /*
            buf[6] = buf[6];
            buf[5] = buf[5];
            buf[4] = buf[4];
            buf[3] = buf[3];
            buf[2] = buf[2];
            buf[1] = buf[1];
            buf[0] = buf[0];
            */
            return new Guid(buf);
        }


        /// <summary>
        /// GUIDをBase64変換します。
        /// </summary>
        /// <param name="guid"></param>
        /// <returns></returns>
        public static string ToBase64String(this Guid guid)
        {
            return Convert.ToBase64String(guid.ToSwapByteArray());
        }

        /// <summary>
        /// GUIDをファイル名で扱えるBase64に変換します。
        /// 末尾の"g=="はカットし、２１文字に変換します。
        /// </summary>
        /// <param name="guid"></param>
        /// <returns></returns>
        public static string ToBase64UrlString(this Guid guid)
        {
            var text = guid.ToBase64String();
            return text.Substring(0, 21).Replace("=", String.Empty).Replace('+', '-').Replace('/', '_');
        }

        /// <summary>
        /// Base64のURLエンコードされた文字列をGUIDに変換します。
        /// </summary>
        /// <param name="base64url"></param>
        /// <returns></returns>
        public static Guid Base64UrlToGuid(this string base64url)
        {
            var base64 = base64url.Replace('_', '/').Replace('-', '+') + "g==";
            var buf = Convert.FromBase64String(base64);
            var guid = FromSwapByteArrayToGuid(buf);
            return guid;
        }

    }
}