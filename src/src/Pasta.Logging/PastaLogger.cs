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
    [Export("PastaLogger", typeof(IPastaLogger)), Shared]
    public sealed class PastaLogger : IPastaLogger
    {
        #region プロパティ

        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        /// <summary>受信ターゲットを接続します。</summary>
        public ITargetBlock<PastaLog> Target { get; private set; }

        /// <summary>送信ソースを接続します。</summary>
        public ISourceBlock<PastaLog> Source { get; private set; }



        #endregion
        #region 初期化・開放

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        [ImportingConstructor]
        public PastaLogger()
        {
            logger.Trace("Load");
        }

        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="settingObject"></param>
        /// <param name="token"></param>
        public void Init(dynamic settingObject, CancellationToken token)
        {
            logger.Trace("Init Start");
            var opt = new DataflowBlockOptions
            {
                CancellationToken = token,
            };
            var buffer = new BufferBlock<PastaLog>(opt);
            var bloadcast = new BroadcastBlock<PastaLog>(CloneLog, opt);
            buffer.LinkTo(bloadcast);
            Target = buffer;
            Source = bloadcast;
            logger.Trace("Init End");
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