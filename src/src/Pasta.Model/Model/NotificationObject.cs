using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Pasta.Model
{
    /// <summary>
    /// 通知オブジェクトの基礎実装。
    /// </summary>
    public abstract class NotificationObject : INotifyPropertyChanged
    {
        /// <summary>
        /// 通知イベント。
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// プロパティ変更を通知します。
        /// </summary>
        /// <param name="propertyName"></param>
        protected void OnPropertyChanged([CallerMemberName]string propertyName = null)
        {
            OnPropertyChangedImpl(propertyName);
        }

        internal void OnPropertyChangedImpl(string propertyName)
        {
            if (PropertyChanged != null)
            {
                PropertyChanged(this, new PropertyChangedEventArgs(propertyName));
            }
        }

    }

}