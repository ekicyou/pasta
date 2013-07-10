#define ON_PROFILE

using System;
using System.Diagnostics;

namespace CSUtil.Profile
{
    /// <summary>
    /// 関数の呼び出し状況を1000回呼出し単位で通知します。
    /// </summary>
    public class CallMonitor
    {
#if ON_PROFILE
        private int count = -1;
        private DateTime lastSpanDT;
        /// <summary>
        /// モニタの名前。
        /// </summary>
        public readonly string Name;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name"></param>
        public CallMonitor(string name)
        {
            Name = name;
        }

        /// <summary>
        /// １回の呼び出しをカウントします。
        /// </summary>
        public void CallOne()
        {
            count++;
            if (count % 1000 == 0) {
                DateTime now = DateTime.Now;
                if (count != 0) {
                    // 呼び出し時間間隔を通知
                    TimeSpan span1000 = now - lastSpanDT;
                    TimeSpan span1 = new TimeSpan(span1000.Ticks / 1000);
                    string text = string.Format("呼出カウンタ[{0}]: CALL COUNT={1}  1CALL TIME:{2}s",
                      Name, count, span1.TotalSeconds);
                    Debug.WriteLine(text);
                }
                lastSpanDT = now;
            }
        }

#else
    /// <summary>
    /// コンストラクタ。
    /// </summary>
    /// <param name="name"></param>
    public CallMonitor(string name){}

    /// <summary>
    /// １回の呼び出しをカウントします。
    /// </summary>
    public void CallOne(){}
#endif

    }

}