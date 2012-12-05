using System;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Windows.Input;

namespace CSUtil.ComponentModel
{
    /// <summary>
    /// Represents ReactiveCommand&lt;object&gt;
    /// </summary>
    public class RxCommand : RxCommand<object>
    {
        /// <summary>
        /// CanExecute is always true. When disposed CanExecute change false called on UIDispatcherScheduler.
        /// </summary>
        public RxCommand()
            : base()
        { }

        /// <summary>
        /// CanExecute is always true. When disposed CanExecute change false called on scheduler.
        /// </summary>
        public RxCommand(IScheduler scheduler)
            : base(scheduler)
        {
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on UIDispatcherScheduler.
        /// </summary>
        public RxCommand(IObservable<bool> canExecuteSource, bool initialValue = true)
            : base(canExecuteSource, initialValue)
        {
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on scheduler.
        /// </summary>
        public RxCommand(IObservable<bool> canExecuteSource, IScheduler scheduler, bool initialValue = true)
            : base(canExecuteSource, scheduler, initialValue)
        {
        }
    }

    public class RxCommand<T> : IObservable<T>, ICommand, IDisposable
    {
        public event EventHandler CanExecuteChanged;

        readonly Subject<T> trigger = new Subject<T>();
        readonly IDisposable canExecuteSubscription;
        readonly IScheduler scheduler;
        bool isCanExecute;

        /// <summary>
        /// CanExecute is always true. When disposed CanExecute change false called on UIDispatcherScheduler.
        /// </summary>
        public RxCommand()
            : this(Observable.Never<bool>())
        {
        }

        /// <summary>
        /// CanExecute is always true. When disposed CanExecute change false called on scheduler.
        /// </summary>
        public RxCommand(IScheduler scheduler)
            : this(Observable.Never<bool>(), scheduler)
        {
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on UIDispatcherScheduler.
        /// </summary>
        public RxCommand(IObservable<bool> canExecuteSource, bool initialValue = true)
            : this(canExecuteSource, DispatcherScheduler.Current, initialValue)
        {
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on scheduler.
        /// </summary>
        public RxCommand(IObservable<bool> canExecuteSource, IScheduler scheduler, bool initialValue = true)
        {
            this.isCanExecute = initialValue;
            this.scheduler = scheduler;
            this.canExecuteSubscription = canExecuteSource
                .DistinctUntilChanged()
                .ObserveOn(scheduler)
                .Subscribe(b =>
                {
                    isCanExecute = b;
                    var handler = CanExecuteChanged;
                    if (handler != null) handler(this, EventArgs.Empty);
                });
        }

        /// <summary>Return current canExecute status. parameter is ignored.</summary>
        public bool CanExecute(object parameter)
        {
            return isCanExecute;
        }

        /// <summary>Push default value to subscribers.</summary>
        public void Execute()
        {
            trigger.OnNext(default(T));
        }

        /// <summary>Push parameter to subscribers.</summary>
        public void Execute(object parameter)
        {
            trigger.OnNext((T)parameter);
        }

        /// <summary>Subscribe execute.</summary>
        public IDisposable Subscribe(IObserver<T> observer)
        {
            return trigger.Subscribe(observer);
        }

        /// <summary>
        /// Dispose all subscription and lock CanExecute is false.
        /// </summary>
        public void Dispose()
        {
            trigger.Dispose();
            canExecuteSubscription.Dispose();

            if (isCanExecute) {
                isCanExecute = false;
                scheduler.Schedule(() =>
                {
                    var handler = CanExecuteChanged;
                    if (handler != null) handler(this, EventArgs.Empty);
                });
            }
        }
    }

    public static class RxCommandExtensions
    {
        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on UIDispatcherScheduler.
        /// </summary>
        public static RxCommand ToRxCommand(this IObservable<bool> canExecuteSource, bool initialValue = true)
        {
            return new RxCommand(canExecuteSource, initialValue);
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on scheduler.
        /// </summary>
        public static RxCommand ToRxCommand(this IObservable<bool> canExecuteSource, IScheduler scheduler, bool initialValue = true)
        {
            return new RxCommand(canExecuteSource, scheduler, initialValue);
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on UIDispatcherScheduler.
        /// </summary>
        public static RxCommand<T> ToRxCommand<T>(this IObservable<bool> canExecuteSource, bool initialValue = true)
        {
            return new RxCommand<T>(canExecuteSource, initialValue);
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on scheduler.
        /// </summary>
        public static RxCommand<T> ToRxCommand<T>(this IObservable<bool> canExecuteSource, IScheduler scheduler, bool initialValue = true)
        {
            return new RxCommand<T>(canExecuteSource, scheduler, initialValue);
        }

        /// <summary>
        /// CanExecuteChanged is called from canExecute sequence on UIDispatcherScheduler.
        /// </summary>
        public static RxCommand ToRxCommand(this RxProperty<bool> canExecuteSource)
        {
            return canExecuteSource.ToRxCommand(canExecuteSource.Value);
        }

    }
}