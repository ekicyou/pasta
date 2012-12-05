using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace System.Reactive.Disposables
{
    /// <summary>CompositeDisposable拡張</summary>
    public static class CompositeDisposableExtensions
    {
        /// <summary>
        /// CompositeDisposableに要素を登録します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="src"></param>
        /// <param name="disp"></param>
        /// <returns></returns>
        public static T Add<T>(this T src, CompositeDisposable disp)
            where T : IDisposable
        {
            disp.Add(src);
            return src;
        }

        /// <summary>
        /// CompositeDisposableにActionを登録します。
        /// </summary>
        /// <param name="disp"></param>
        /// <param name="act"></param>
        public static void Add(this CompositeDisposable disp, Action act)
        {
            disp.Add(Disposable.Create(act));
        }

    }
}
