using System;
using System.ComponentModel;
using System.Windows.Threading;
using System.Collections.Generic;

namespace CSUtil.ComponentModel
{
    /// <summary>
    /// 通知を行なうプロパティ。
    /// 通知処理はコンストラクタで指定したスレッドに行われます。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class NotifyProperty<T> : INotifyPropertyChanged, IValue
    {
        private T nowValue;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="value"></param>
        public NotifyProperty(Dispatcher dispatcher, T value)
        {
            nowValue = value;
            Dispatcher = dispatcher;
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public NotifyProperty(Dispatcher dispatcher)
            : this(dispatcher, default(T))
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public NotifyProperty(T value)
            : this(Dispatcher.CurrentDispatcher, value)
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public NotifyProperty()
            : this(default(T))
        {
        }

        /// <summary>ディスパッチャ。</summary>
        public Dispatcher Dispatcher { get; private set; }


        /// <summary>値。</summary>
        public T Value
        {
            get { return nowValue; }
            set
            {
                if (Comparer.Equals(nowValue, value)) return;
                nowValue = value;
                RaiseChanged();
            }
        }

        private static readonly IEqualityComparer<T> Comparer = EqualityComparer<T>.Default;

        object IValue.Value
        {
            get { return (T)Value; }
            set { Value = (T)value; }
        }


        /// <summary>
        /// 通知処理の実装。
        /// </summary>
        public virtual void RaiseChanged()
        {
            if (PropertyChanged == null) return;
            if (Dispatcher.HasShutdownStarted) return;
            Dispatcher.BeginInvoke(new Action(() =>
            {
                if (PropertyChanged == null) return;
                if (Dispatcher.HasShutdownStarted) return;
                PropertyChanged(this, new PropertyChangedEventArgs("Value"));
            }));
        }

        #region INotifyPropertyChanged メンバー

        /// <summary>
        /// プロパティ変更イベントを、指定したDispatcherから発行します。
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;


        #endregion
    }
}
