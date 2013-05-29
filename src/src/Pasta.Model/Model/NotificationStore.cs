using System.Collections.Generic;
using System.Runtime.CompilerServices;

namespace Pasta.Model
{
    /// <summary>
    /// 通知する値を管理する構造体。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public struct NotificationStore<T>
    {
        /// <summary>値。default(T)で初期化されます。</summary>
        private T store;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="initValue"></param>
        public NotificationStore(T initValue)
        {
            store = initValue;
        }

        /// <summary>
        /// 値の取得。getterで利用してください。
        /// </summary>
        /// <returns></returns>
        public T Get()
        {
            return store;
        }

        /// <summary>
        /// 値の取得。getterで利用してください。
        /// </summary>
        /// <param name="i"></param>
        /// <returns></returns>
        public static implicit operator T(NotificationStore<T> i)
        {
            return i.Get();
        }

        /// <summary>
        /// 値の設定。更新が行われればtrue。setterで利用してください。
        /// </summary>
        /// <typeparam name="TO"></typeparam>
        /// <param name="value"></param>
        /// <param name="THIS"></param>
        /// <param name="propertyName"></param>
        /// <returns></returns>
        public bool Set<TO>(T value, TO THIS, [CallerMemberName]string propertyName = null)
            where TO : NotificationObject
        {
            if(EQ.Equals(store, value)) return false;
            store = value;
            THIS.OnPropertyChangedImpl(propertyName);
            return true;
        }

        /// <summary>コンパレータ</summary>
        private static readonly IEqualityComparer<T> EQ = EqualityComparer<T>.Default;

    }
}