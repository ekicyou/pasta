using System;

namespace CSUtil
{
    /// <summary>
    /// "弱い参照" を現すジェネリッククラスです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class WeakReference<T> : WeakReference where T : class
    {
        /// <summary>
        /// コンストラクタ
        /// </summary>
        /// <param name="target"></param>
        public WeakReference(T target)
            : base(target)
        {
        }

        /// <summary>
        /// 現在の WeakReference オブジェクトが参照するオブジェクトを、その終了後に追跡するかどうかを示す値を取得します。
        /// </summary>
        public new T Target
        {
            get { return (T)base.Target; }
            set { base.Target = value; }
        }

    }
}