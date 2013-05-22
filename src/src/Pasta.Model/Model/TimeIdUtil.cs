using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Pasta.Model
{
    /// <summary>
    /// 時計を利用したユニークＩＤ発番を実装。
    /// </summary>
    public static class TimeIdUtil
    {
        /// <summary>
        /// 時計を利用して、ユニークなＩＤを発番します。
        /// </summary>
        /// <returns></returns>
        public static long GetUniqueTimeID()
        {
            var time = DateTime.UtcNow;
            lock (GetTimeLock)
            {
                if (time <= LastTime) time = LastTime + OneTick;
                LastTime = time;
            }
            return time.ToBinary();
        }
        private static DateTime LastTime = DateTime.MinValue;
        private static readonly TimeSpan OneTick = TimeSpan.FromTicks(1);
        private static readonly object GetTimeLock = new object();
    }
}