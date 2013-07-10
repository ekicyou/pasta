using System;
using System.Collections.Generic;
using System.Text;
using System.Threading;
using System.Globalization;

namespace CSUtil.Globalization
{
    /// <summary>
    /// カルチャーユーティリティ
    /// </summary>
    public static class CultureUtil
    {
        /// <summary>
        /// 言語環境が日本語以外の場合、強制的に英語にする。
        /// </summary>
        public static void SetEnIfNotJa()
        {
            var t = Thread.CurrentThread;
            if (!t.CurrentUICulture.Name.StartsWith("ja") &&
                !t.CurrentUICulture.Name.StartsWith("en")) {
                t.CurrentUICulture = new CultureInfo("en", false);
            }
        }
    }
}
