using System.Reactive.Disposables;
using System.Windows.Threading;

namespace System.Reactive.Concurrency
{
    /// <summary>
    /// 優先度付きDispatcherスケジューラ。
    /// </summary>
    public class DispatcherPriorityScheduler : IScheduler
    {
        private readonly Dispatcher dispatcher;
        private readonly DispatcherPriority priority;

        /// <summary>ディスパッチャを取得します。</summary>
        public Dispatcher Dispatcher { get { return dispatcher; } }

        /// <summary>優先度を取得します。</summary>
        public DispatcherPriority Priority { get { return priority; } }

        /// <summary>現在時刻を取得します。</summary>
        public DateTimeOffset Now { get { return DateTimeOffset.Now; } }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="dispatcher"></param>
        /// <param name="priority"></param>
        public DispatcherPriorityScheduler(Dispatcher dispatcher, DispatcherPriority priority)
        {
            if (dispatcher == null) throw new ArgumentNullException("dispatcher");
            this.dispatcher = dispatcher;
            this.priority = priority;
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="dispatcher"></param>
        public DispatcherPriorityScheduler(Dispatcher dispatcher)
            : this(dispatcher, DispatcherPriority.Normal)
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="priority"></param>
        public DispatcherPriorityScheduler(DispatcherPriority priority)
            : this(Dispatcher.CurrentDispatcher, priority)
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public DispatcherPriorityScheduler()
            : this(Dispatcher.CurrentDispatcher, DispatcherPriority.Normal)
        {
        }

        /// <summary>
        /// Scheduleを登録します。
        /// </summary>
        /// <typeparam name="TState"></typeparam>
        /// <param name="state"></param>
        /// <param name="action"></param>
        /// <returns></returns>
        public IDisposable Schedule<TState>(TState state, Func<IScheduler, TState, IDisposable> action)
        {
            if (action == null) throw new ArgumentNullException("action");
            var d = new SingleAssignmentDisposable();
            Action invokeAction = () =>
            {
                if (!d.IsDisposed) {
                    d.Disposable = action(this, state);
                }
            };
            this.dispatcher.BeginInvoke(invokeAction, Priority);
            return d;
        }

        /// <summary>
        /// 指定時間後に実行される処理を登録します。
        /// </summary>
        /// <typeparam name="TState"></typeparam>
        /// <param name="state"></param>
        /// <param name="dueTime"></param>
        /// <param name="action"></param>
        /// <returns></returns>
        public IDisposable Schedule<TState>(TState state, TimeSpan dueTime, Func<IScheduler, TState, IDisposable> action)
        {
            if (action == null) {
                throw new ArgumentNullException("action");
            }
            var interval = Scheduler.Normalize(dueTime);
            if (interval.Ticks == 0L) {
                return this.Schedule<TState>(state, action);
            }
            var d = new MultipleAssignmentDisposable();
            var timer = new DispatcherTimer(this.priority, this.dispatcher);
            timer.Tick += (s, e) =>
            {
                var timer1 = timer;
                if (timer1 != null) timer1.Stop();
                timer = null;
                d.Disposable = action(this, state);
            };
            timer.Interval = interval;
            timer.Start();
            d.Disposable = Disposable.Create(() =>
            {
                var timer1 = timer;
                if (timer1 != null) timer1.Stop();
                timer = null;
            }
            );
            return d;
        }

        /// <summary>
        /// 指定時刻になってから実行される処理を登録します。
        /// </summary>
        /// <typeparam name="TState"></typeparam>
        /// <param name="state"></param>
        /// <param name="dueTime"></param>
        /// <param name="action"></param>
        /// <returns></returns>
        public IDisposable Schedule<TState>(TState state, DateTimeOffset dueTime, Func<IScheduler, TState, IDisposable> action)
        {
            return Schedule(state, dueTime - Now, action);
        }

    }
}