using System;
using System.Threading;

namespace CSUtil.Threading
{
    /// <summary>
    /// 時計に同期して定期的な処理を行なうためのタイマーです。
    /// </summary>
    public class ClockSyncTimer : DisposableObject
    {
        /// <summary>
        /// 時計に対する同期間隔。
        /// </summary>
        public TimeSpan Interval
        {
            get { return interval; }
            set { interval = value; }
        }
        private TimeSpan interval;

        /// <summary>
        /// 間隔が経過すると発生します。
        /// </summary>
        public event EventHandler Elapsed;

        /// <summary>
        /// イベントを発生させるためのスレッド。
        /// </summary>
        private Thread thread;

        /// <summary>
        /// 時計同期タイマーを作成します。
        /// バックグラウンドスレッドとして動作します。
        /// </summary>
        public ClockSyncTimer()
        {
            thread = new Thread(ThreadMain);
            thread.SetApartmentState(ApartmentState.STA);
            thread.IsBackground = true;
        }

        /// <summary>
        /// タイマーをスタートします。
        /// </summary>
        public void Start()
        {
            thread.Start();
        }

        private void ThreadMain()
        {
            long spanTick = Interval.Ticks;
            while (true) {
                if (IsDisposed) return;
                long nowTicks = DateTime.Now.Ticks;
                long waitTicks = spanTick - (nowTicks % spanTick);
                // 待機時間が10ms以下の場合はあきらめて次の時間にする
                if (waitTicks < (TimeSpan.TicksPerMillisecond * 10)) waitTicks += spanTick;
                long targetTicks = nowTicks + waitTicks;
                while (true) {
                    int waitMS = (int)Math.Ceiling(((double)waitTicks) / ((double)TimeSpan.TicksPerMillisecond));
                    if (waitMS < 1) waitMS = 1;
                    Thread.Sleep(waitMS);
                    nowTicks = DateTime.Now.Ticks;
                    if (nowTicks > targetTicks) break;
                    waitTicks = targetTicks - nowTicks;
                }

                if (Elapsed == null) continue;
                EventArgs args = new EventArgs();
                Elapsed(this, args);
            }
        }

    }
}