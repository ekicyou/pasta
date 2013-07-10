using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Reactive.Concurrency;
using System.Threading;

namespace System.Reactive.Concurrency
{
    /// <summary>
    /// 現在のSchedule数をカウントします。
    /// </summary>
    public class ScheduleCounter
    {
        private object lockObj = new object();

        /// <summary>実行中カウンタ</summary>
        public int Count
        {
            get { return count; }
            set
            {
                lock (lockObj) {
                    count = value;
                    if (count == 0 && WaitJoinCount > 0) Monitor.PulseAll(lockObj);
                }
            }
        }
        private int count = 0;

        private int WaitJoinCount = 0;

        /// <summary>
        /// スケジューラをラップし、Schedule数の測定対象とします。
        /// </summary>
        /// <param name="sc"></param>
        /// <returns></returns>
        public IScheduler Create(IScheduler sc)
        {
            return new CounterScheduler(this, sc);
        }

        /// <summary>
        /// Schedule数が０になるまで待機します。
        /// </summary>
        public void JoinZeroTask()
        {
            lock (lockObj) {
                if (Count == 0) return;
                WaitJoinCount++;
                try { Monitor.Wait(lockObj); }
                finally { WaitJoinCount--; }
            }
        }

        private void Inc()
        {
            lock (lockObj) Count++;
        }

        private void Dec()
        {
            lock (lockObj) Count--;
        }

        private class ScheduleAction<TState> : IDisposable
        {
            private readonly ScheduleCounter Counter;
            private readonly Func<IScheduler, TState, IDisposable> Action;
            private bool HasDec = false;
            public IDisposable CancelDisposable { get; set; }

            public ScheduleAction(ScheduleCounter counter, Func<IScheduler, TState, IDisposable> act)
            {
                Counter = counter;
                Action = act;
                Counter.Inc();
            }

            public void Dispose()
            {
                Dec();
                CancelDisposable.Dispose();
            }

            private void Dec()
            {
                lock (this) {
                    if (HasDec) return;
                    HasDec = true;
                    Counter.Dec();
                }
            }

            public IDisposable Func(IScheduler sc, TState state)
            {
                try {
                    return Action(sc, state);
                }
                finally { Dec(); }
            }
        }

        private class CounterScheduler : IScheduler
        {
            private readonly IScheduler SC;
            private readonly ScheduleCounter Counter;

            public CounterScheduler(ScheduleCounter counter, IScheduler sc)
            {
                SC = sc;
                Counter = counter;
            }


            #region IScheduler メンバー

            public DateTimeOffset Now
            {
                get { return SC.Now; }
            }


            public IDisposable Schedule<TState>(TState state, DateTimeOffset dueTime, Func<IScheduler, TState, IDisposable> action)
            {
                var d = new ScheduleAction<TState>(Counter, action);
                d.CancelDisposable = SC.Schedule(state, dueTime, d.Func);
                return d;
            }

            public IDisposable Schedule<TState>(TState state, TimeSpan dueTime, Func<IScheduler, TState, IDisposable> action)
            {
                var d = new ScheduleAction<TState>(Counter, action);
                d.CancelDisposable = SC.Schedule(state, dueTime, d.Func);
                return d;
            }

            public IDisposable Schedule<TState>(TState state, Func<IScheduler, TState, IDisposable> action)
            {
                var d = new ScheduleAction<TState>(Counter, action);
                d.CancelDisposable = SC.Schedule(state, d.Func);
                return d;
            }

            #endregion
        }

    }
}
