using System.Collections.Generic;
using System.Linq;
using System;

namespace System
{
    public static class ConvertExtensions
    {
        /// <summary>
        /// 文字列をTrimします。null文字列の場合は空白文字列を返します。
        /// </summary>
        /// <param name="text"></param>
        /// <returns></returns>
        public static string TrimEx(this string text)
        {
            if (text == null) return "";
            return text.Trim();
        }


        /// <summary>int32 に変換します。</summary>
        public static int ToInt32(this string s) { return int.Parse(s.Trim()); }

        /// <summary>
        /// Int32値に変換します。
        /// 変換出来無い場合はデフォルト値を返します。
        /// </summary>
        /// <param name="text"></param>
        /// <param name="defValue"></param>
        /// <returns></returns>
        public static int ToInt32OrDefault(this string text, int defValue)
        {
            text = text.TrimEx();
            int rc;
            if (int.TryParse(text, out rc)) return rc;
            return defValue;
        }

        /// <summary>int32 に変換します。変換できない場合はデフォルト値とします。</summary>
        public static int ToInt32OrDefault(this string s) { return s.ToInt32OrDefault(default(int)); }



        /// <summary>double に変換します。</summary>
        public static double ToDouble(this string s) { return double.Parse(s.Trim()); }

        /// <summary>
        /// Double値に変換します。
        /// 変換出来無い場合はデフォルト値を返します。
        /// </summary>
        /// <param name="text"></param>
        /// <param name="defValue"></param>
        /// <returns></returns>
        public static double ToDoubleOrDefault(this string text, double defValue)
        {
            text = text.TrimEx();
            double rc;
            if (double.TryParse(text, out rc)) return rc;
            return defValue;
        }
        /// <summary>
        /// Double値に変換します。
        /// 変換出来無い場合はdefault(T)を返します。
        /// </summary>
        /// <param name="text"></param>
        /// <returns></returns>
        public static double ToDoubleOrDefault(this string text)
        {
            return ToDoubleOrDefault(text, default(double));
        }



        /// <summary>DateTime に変換します。</summary>
        public static DateTime ToDateTime(this string s) { return DateTime.Parse(s.Trim()); }

        /// <summary>DateTime に変換します。変換できない場合はデフォルト値とします。</summary>
        public static DateTime ToDateTimeOrDefault(this string s, DateTime def) { try { return s.ToDateTime(); } catch { return def; } }

        /// <summary>DateTime に変換します。変換できない場合はデフォルト値とします。</summary>
        public static DateTime ToDateTimeOrDefault(this string s) { return s.ToDateTimeOrDefault(default(DateTime)); }



        /// <summary>
        /// セパレータを利用して文字列を列挙します。空白は無視します。
        /// </summary>
        /// <param name="s"></param>
        /// <param name="separator"></param>
        /// <returns></returns>
        public static IEnumerable<string> SplitSkipBlank(this string s, params char[] separator)
        {
            if(string.IsNullOrEmpty(s)) return Enumerable.Empty<string>();
            return s
                .Split(separator)
                .Where(a => !string.IsNullOrEmpty(a))
                .Select(a => a.Trim())
                .Where(a => !string.IsNullOrEmpty(a));
        }

        /// <summary>
        /// セパレータを利用してintに変換します。
        /// </summary>
        /// <param name="s"></param>
        /// <param name="separator"></param>
        /// <returns></returns>
        public static IEnumerable<int> SplitToInt32(this string s, params char[] separator)
        {
            return s.SplitSkipBlank(separator)
                .Select(a => int.Parse(a.Trim()));
        }

        /// <summary>
        /// セパレータを利用してdoubleに変換します。
        /// </summary>
        /// <param name="s"></param>
        /// <param name="separator"></param>
        /// <returns></returns>
        public static IEnumerable<double> SplitToDouble(this string s, params char[] separator)
        {
            return s.SplitSkipBlank(separator)
                .Select(a => double.Parse(a.Trim()));
        }

    }
}