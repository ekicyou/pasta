using System;
using System.Diagnostics;
using System.Reflection;

namespace CSUtil.Profile
{
    /// <summary>
    /// メッセージ出力つきのストップウォッチ
    /// </summary>
    public class StopwatchEx : Stopwatch
    {
        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public StopwatchEx()
            : base()
        {
        }

        /// <summary>
        /// ストップウォッチを作成し、スタートします。
        /// </summary>
        /// <returns></returns>
        public static new StopwatchEx StartNew()
        {
            StopwatchEx rc = new StopwatchEx();
            rc.Start();
            return rc;
        }

        /// <summary>
        /// 経過時間を取得し、遅延が大きければログを出力後、時間をリセットし再起動します。
        /// </summary>
        /// <param name="text"></param>
        public void CheckAndRestart(string text)
        {
            Stop();
            if (Elapsed > WarningTime) {
                StackTrace st = new StackTrace(1, true);
                StackFrame frame = st.GetFrame(0);
                MethodBase method = frame.GetMethod();
                Trace.WriteLine(string.Format("[{0}.{1}]({2})経過時間={3}",
                  method.ReflectedType.Name,
                  method.Name,
                  text,
                  Elapsed));
            }
            Reset();
            Start();
        }

        private static readonly TimeSpan WarningTime = TimeSpan.FromSeconds(0.5);

    }
}