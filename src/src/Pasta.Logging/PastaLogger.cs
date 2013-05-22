using Pasta.API;
using Pasta.Util.Disposables;
using System;
using System.Collections.Generic;
using System.Composition;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;
using Pasta.Model;


namespace Pasta.Logging
{
    /// <summary>
    /// ロガー。
    /// 解析の呼び出し、及びファイルへの保管処理を行う。
    /// </summary>
    [Export("PastaLogger", typeof(IPastaLogger))]
    public sealed class PastaLogger : IPastaLogger
    {
        #region プロパティ

        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        /// <summary>受信ターゲットを接続します。</summary>
        public ISourceBlock<PastaLog> Input { get; private set; }

        /// <summary>送信ソースを接続します。</summary>
        public ITargetBlock<PastaLog> Output { get; private set; }



        #endregion
        #region 初期化・開放

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="token"></param>
        public PastaLogger(CancellationToken token)
        {
            var opt = new DataflowBlockOptions
            {
                CancellationToken = token,
            };
            var buffer = new BufferBlock<PastaLog>(opt);
            var bloadcast = new BroadcastBlock<PastaLog>(CloneLog, opt);
            buffer.LinkTo(bloadcast);
            Input = buffer;
            Output = bloadcast;
        }
        #endregion
        #region メソッド

        private static PastaLog CloneLog(PastaLog log)
        {
            logger.Trace(log);
            return log;
        }

        #endregion
    }
}