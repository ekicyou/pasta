using System;
using System.Diagnostics;

namespace CSUtil.Profile
{
    /// <summary>
    /// イベントの実行時間を計測し、警告を出すためのプロファイラです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    internal class EventProfiler<T> where T : EventArgs
    {
        private EventHandler<T> baseEvent;
        private readonly string Name;
        private TimeSpan maxTime = TimeSpan.Zero;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="profileEvent"></param>
        internal EventProfiler(string name, EventHandler<T> profileEvent)
        {
            Name = string.Format("[{0}]({1})", name, profileEvent.Method.Name);
            baseEvent = profileEvent;
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="profileEvent"></param>
        internal EventProfiler(EventHandler<T> profileEvent)
        {
            Name = string.Format("[{0}.{1}]", profileEvent.Target.GetType().Name, profileEvent.Method.Name);
            baseEvent = profileEvent;
        }


        /// <summary>
        /// イベントを実行します。
        /// 実行時間を計測し、最高処理時間を更新した場合にログを出力します。
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="args"></param>
        internal void OnProfile(object sender, T args)
        {
            if (baseEvent == null) return;
            Stopwatch sw = Stopwatch.StartNew();
            try {
                baseEvent(sender, args);
            }
            finally {
                sw.Stop();
                TimeSpan time = sw.Elapsed;
                if (time > maxTime) {
                    Trace.WriteLine(string.Format("PROFILE_EVENT_UPDATE: {1:000.000000}sec -->{0}", Name, time.TotalSeconds));
                    maxTime = time;
                }
            }
        }
    }
}