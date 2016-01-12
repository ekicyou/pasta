#define ON_PROFILE

using System;
using System.Diagnostics;
using System.Windows.Forms;

namespace CSUtil.Profile
{
    /// <summary>
    /// プロファイルユーティリティ。
    /// </summary>
    public static class ProfileUtil
    {
        /// <summary>
        /// 与えられたイベントにプロファイル処理をラップします。
        /// </summary>
        /// <param name="profileEvent"></param>
        /// <returns></returns>
        public static EventHandler<T> WrapEvent<T>(EventHandler<T> profileEvent) where T : EventArgs
        {
#if ON_PROFILE
            EventProfiler<T> pEvent = new EventProfiler<T>(profileEvent);
            return pEvent.OnProfile;
#else
      return profileEvent;
#endif
        }

        /// <summary>
        /// 引数のないデリゲート。
        /// </summary>
        public delegate void InvoleMethod();

        /// <summary>
        /// MethodInvokerにプロファイルをラップして返します。
        /// </summary>
        /// <param name="func"></param>
        /// <returns></returns>
        public static MethodInvoker WrapInvoke(InvoleMethod func)
        {
#if ON_PROFILE
            String Name = string.Format("[{0}.{1}]", func.Target.GetType().Name, func.Method.Name);
            return new MethodInvoker(delegate()
            {
                Stopwatch sw = Stopwatch.StartNew();
                try {
                    func();
                }
                finally {
                    sw.Stop();
                    TimeSpan time = sw.Elapsed;
                    if (time > WrapInvokeWarningTime) {
                        Trace.WriteLine(string.Format("PROFILE_INVOKE_UPDATE: {1:000.000000}sec -->{0}", Name, time.TotalSeconds));
                        WrapInvokeWarningTime = time;
                    }
                }
            });
#else
      return new MethodInvoker(func);
#endif
        }
        private static TimeSpan WrapInvokeWarningTime = TimeSpan.Zero;



        /// <summary>
        /// プロファイルを実行します。
        /// </summary>
        /// <param name="func"></param>
        /// <returns></returns>
        public static void WrapFunction(InvoleMethod func)
        {
#if ON_PROFILE
            String Name = string.Format("[{0}.{1}]", func.Target.GetType().Name, func.Method.Name);
            Stopwatch sw = Stopwatch.StartNew();
            try {
                func();
            }
            finally {
                sw.Stop();
                TimeSpan time = sw.Elapsed;
                if (time > WrapFuncWarningTime) {
                    Trace.WriteLine(string.Format("PROFILE_FUNC_UPDATE: {1:000.000000}sec -->{0}", Name, time.TotalSeconds));
                    WrapFuncWarningTime = time;
                }
            }
#else
     　func();
#endif
        }
        private static TimeSpan WrapFuncWarningTime = TimeSpan.Zero;

    }
}