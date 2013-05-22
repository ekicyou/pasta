using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Pasta.Util.Disposables
{
    public static class DisposableUtil
    {
        /// <summary>
        /// Gets the disposable that does nothing when disposed.
        /// </summary>
        public static IDisposable Empty
        {
            get
            {
                return DefaultDisposable.Instance;
            }
        }
        /// <summary>
        /// 開放時にアクションが実行されるIDisposableを作成します。
        /// </summary>
        /// <param name="dispose">開放時に実行するアクション</param>
        /// <returns>The disposable object that runs the given action upon disposal.</returns>
        public static IDisposable ToDisposable(this Action dispose)
        {
            if (dispose == null)
            {
                throw new ArgumentNullException("dispose");
            }
            return new AnonymousDisposable(dispose);
        }


        /// <summary>
        /// StackDisposableに要素を登録します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="src"></param>
        /// <param name="disp"></param>
        /// <returns></returns>
        public static T Add<T>(this T src, StackDisposable disp)
            where T : IDisposable
        {
            disp.Add(src);
            return src;
        }

        /// <summary>
        /// StackDisposableにActionを登録します。
        /// </summary>
        /// <param name="disp"></param>
        /// <param name="act"></param>
        public static void Add(this StackDisposable disp, Action act)
        {
            disp.Add(act.ToDisposable());
        }

        /// <summary>
        /// Enter,Leaveアクションを指定したIDisposableを作成します。
        /// </summary>
        /// <param name="enter"></param>
        /// <param name="leave"></param>
        /// <returns></returns>
        public static IDisposable Create(Action enter, Action leave)
        {
            enter();
            return leave.ToDisposable();
        }

        /// <summary>
        /// Lazyオブジェクトを作成し、StackDisposableに登録します。
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="disp"></param>
        /// <param name="create"></param>
        /// <param name="leave"></param>
        /// <returns></returns>
        public static Lazy<T> CreateLazy<T>(this StackDisposable disp, Func<T> create, Action leave)
        {
            var lazy = new Lazy<T>(create);
            Action act = () =>
            {
                if (!lazy.IsValueCreated) return;
                leave();
            };
            disp.Add(act);
            return lazy;
        }
    }
}