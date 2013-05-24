using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Composition;
using System.Composition.Hosting;
using Pasta.Model;

namespace Pasta
{
    public class AppCore : NotificationObject, IDisposable
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();


        private CancellationTokenSource CTS { get; set; }

        public void Dispose()
        {
            logger.Trace("Dispose Start");
            CTS.Cancel();
            CTS.Dispose();
            logger.Trace("Dispose End");
            logger.Trace("AppCore End");
        }


        public AppCore()
        {
            Thread.CurrentThread.Name = "UI";
            logger.Trace("AppCore Start");
            CTS = new CancellationTokenSource();
            var fact = new TaskFactory(CTS.Token);
            fact.StartNew(Init);
        }

        private void Init()
        {
            logger.Trace("Init Start");

            // モジュールのロード



            logger.Trace("Init End");
        }

    }
}