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
        public void Init(CancellationToken token)
        {
            try
            {
                logger.Trace("Init Start");

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

        [Import]
        public IFileIO FileIO { get; set; }

        [Import("PastaLogger")]
        public IPastaLogger PastaLogger { get; set; }

        [Import("PastaStore")]
        public IPastaStore PastaStore { get; set; }


    }
}