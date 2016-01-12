using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil
{
    /// <summary>
    /// Weak値を保持します。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class WeakValue<T> where T : class
    {
        private WeakReference weak = null;

        /// <summary>値を取得又は返します。</summary>
        public T Value
        {
            get { return GetValue(); }
            set { weak = new WeakReference(value); }
        }

        private T GetValue()
        {
            if (weak == null) return null;
            T rc = weak.Target as T;
            if (rc == null) weak = null;
            return rc;
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public WeakValue()
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="value"></param>
        public WeakValue(T value)
        {
            Value = value;
        }
    }
}
