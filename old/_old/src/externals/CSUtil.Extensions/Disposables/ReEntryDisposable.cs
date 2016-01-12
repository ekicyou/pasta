using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;

namespace CSUtil.Disposables
{
    /// <summary>
    /// 開放時に実行するアクションを管理します。
    /// 多重に呼び出された場合、いちばん外側のIDisposableの開放時に実行します。
    /// マルチスレッドの呼び出しは考慮されていません。
    /// </summary>
    public class ReEntryDisposable
    {
        private Action Enter { get; set; }
        private Action Leave { get; set; }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="act"></param>
        public ReEntryDisposable(Action enter, Action leave)
        {
            Enter = enter;
            Leave = leave;
        }

        /// <summary>
        /// 処理の開始を登録します。
        /// 多重に呼び出された場合は何もしないIDisposableを返します。
        /// </summary>
        /// <returns></returns>
        public IDisposable Begin()
        {
            if(  ReEntryCount++ > 1) {
                ReEntryCount--;
                return CSUtilDisposable.Empty;
            }
            return CSUtilDisposable.Create(Enter, () =>
            {
                ReEntryCount--;
                Leave();
            });
        }
        private int ReEntryCount = 0;

    }
}
