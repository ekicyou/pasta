using System;
using CSUtil.Disposables;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Windows.Threading;

namespace CSUtil.ComponentModel
{
    /// <summary>
    /// IObservableを実装するプロパティ。
    /// IObservable通知はスレッド切り替えを伴わず、現在のスレッドで行われる。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class RxProperty<T> : NotifyProperty<T>, IObservable<T>, IDisposable
    {
        public StackDisposable Tasks { get; private set; }
        internal protected Subject<T> trigger;

        public void Dispose()
        {
            Tasks.Dispose();
            
        }

        private void Init()
        {
            Tasks = new StackDisposable();
            trigger = new Subject<T>();
            Tasks.Add(trigger);
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="value"></param>
        public RxProperty(Dispatcher dispatcher, T value) : base(dispatcher, value) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxProperty(Dispatcher dispatcher) : base(dispatcher) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxProperty(T value) : base(value) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxProperty() : base() { Init(); }



        /// <summary>
        /// 通知処理の実装。
        /// </summary>
        public override void RaiseChanged()
        {
            if (!Tasks.IsDisposed) trigger.OnNext(Value);
            base.RaiseChanged();
        }


        #region IObservable<T> メンバー

        /// <summary>
        /// 通知を作成。最初に現在の値を必ず返す。
        /// </summary>
        /// <param name="observer"></param>
        /// <returns></returns>
        public IDisposable Subscribe(IObserver<T> observer)
        {
            return Observable
                .Return(Value)
                .Concat(trigger)
                .Subscribe(observer);
        }

        #endregion
    }

    public static class RxPropertyExtensions
    {
        /// <summary>
        /// RxPropertyに変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="source"></param>
        /// <param name="dispatcher"></param>
        /// <param name="initValue"></param>
        /// <returns></returns>
        public static RxProperty<T> ToRxProperty<T>(this IObservable<T> source, Dispatcher dispatcher, T initValue)
        {
            var rc = new RxProperty<T>(dispatcher, initValue);
            source
                .Subscribe(
                a => rc.Value = a,
                err => rc.trigger.OnError(err),
                () => rc.trigger.OnCompleted())
                .Add(rc.Tasks);
            return rc;
        }

        /// <summary>
        /// RxPropertyに変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="source"></param>
        /// <param name="dispatcher"></param>
        /// <returns></returns>
        public static RxProperty<T> ToRxProperty<T>(this IObservable<T> source, Dispatcher dispatcher)
        {
            return source.ToRxProperty(dispatcher, default(T));
        }

        /// <summary>
        /// RxPropertyに変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="source"></param>
        /// <param name="initValue"></param>
        /// <returns></returns>
        public static RxProperty<T> ToRxProperty<T>(this IObservable<T> source, T initValue)
        {
            return source.ToRxProperty(Dispatcher.CurrentDispatcher, initValue);
        }

        /// <summary>
        /// RxPropertyに変換します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="source"></param>
        /// <returns></returns>
        public static RxProperty<T> ToRxProperty<T>(this IObservable<T> source)
        {
            return source.ToRxProperty(Dispatcher.CurrentDispatcher);
        }



    }
}