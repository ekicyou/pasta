using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;
using System.Composition;
using System.Composition.Hosting;
using Pasta.Model;
using Pasta.API;

namespace Pasta
{
    [Export, Shared]
    public class AppCore : NotificationObject
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        [ImportingConstructor]
        public AppCore()
        {
            logger.Trace("Load");
        }

        /// <summary>
        /// 遅延初期化処理。
        /// </summary>
        /// <param name="token"></param>
        /// <param name="uiSyncContext"></param>
        public void Init(CancellationToken token, TaskScheduler uiSyncContext)
        {
            try
            {
                logger.Trace("Init Start");

                UISyncContext = uiSyncContext;
                GleanerFactoryDic = GleanerFactories.ToDictionary(a => a.GleanerName);

                PastaStore.Init(token);
                PastaLogger.Init(token);
                PastaLogger.Source.LinkTo(PastaStore.Target);

                logger.Trace("Init Start");
            }
            catch (Exception ex)
            {
                logger.Error(ex);
            }

        }

        /// <summary>UIスケジューラ</summary>
        public TaskScheduler UISyncContext { get; private set; }

        /// <summary>FileIOモジュール</summary>
        [Import]
        public IFileIO FileIO { get; set; }

        /// <summary>ログストア</summary>
        [Import("PastaStore")]
        public IPastaStore PastaStore { get; set; }

        /// <summary>ログ管理モジュール</summary>
        [Import("PastaLogger")]
        public IPastaLogger PastaLogger { get; set; }

        /// <summary>ログ収集モジュールのファクトリ</summary>
        [ImportMany]
        public IEnumerable<IPastaGleanerFactory> GleanerFactories { get; set; }

        /// <summary>ログ収集モジュールのファクトリ辞書</summary>
        public IDictionary<string, IPastaGleanerFactory> GleanerFactoryDic { get; private set; }

    }
}