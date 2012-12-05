using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil
{
    /// <summary>
    /// 日付のコンバータ。
    /// </summary>
    public static class DateTimeUtil
    {
        /// <summary>
        /// 月・日・曜日より、年月日を求めてDateTimeを作成します。
        /// 180日先の年より過去にさかのぼり、最初に曜日が一致した年にします。
        /// </summary>
        /// <param name="month"></param>
        /// <param name="day"></param>
        /// <param name="week"></param>
        /// <returns></returns>
        public static DateTime ToDate(int month, int day, DayOfWeek week)
        {
            for(int y = (DateTime.Now + Span180Days).Year; ; y--) {
                DateTime rc = new DateTime(y, month, day);
                if(rc.DayOfWeek == week) return rc;
            }
        }
        private static readonly TimeSpan Span180Days = TimeSpan.FromDays(180);
    }
}