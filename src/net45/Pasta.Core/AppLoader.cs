using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using System.Composition;
using System.Composition.Hosting;
using Pasta.Model;
using Pasta.API;

namespace Pasta
{
    public class AppLoader : NotificationObject, IDisposable
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();


        private CancellationTokenSource CTS { get; set; }

        /// <summary>
        /// 開放。
        /// </summary>
        public void Dispose()
        {
            logger.Trace("Dispose Start");
            CTS.Cancel();
            CTS.Dispose();
            logger.Trace("Dispose End");
            logger.Trace("AppCore End");
        }


        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public AppLoader()
        {
            Thread.CurrentThread.Name = "UI";
            logger.Trace("MEFLoader Load");
            CTS = new CancellationTokenSource();
            var fact = new TaskFactory(CTS.Token);
            var callerAssembly = Assembly.GetCallingAssembly();
            fact.StartNew(() => Init(CTS.Token, callerAssembly));
        }

        /// <summary>
        /// 遅延初期化処理。
        /// </summary>
        /// <param name="token"></param>
        private void Init(CancellationToken token,Assembly callerAssembly)
        {
            logger.Trace("Init Start");
            try
            {
                // MEFの構成
                var configuration = new ContainerConfiguration()
                    .WithAssemblies(new[] { this.GetType().Assembly, callerAssembly, typeof(Pasta.Logging.PastaLogger).Assembly })
                    ;
                var host = configuration.CreateContainer();
                token.Register(() => host.Dispose());

                // appの取得と構成
                var app = host.GetExport<AppCore>();
                app.Init(token);
                Application = app;
                logger.Trace("Init End");
            }
            catch (Exception ex)
            {
                logger.Error(ex);
            }
        }

        /// <summary>アプリケーション</summary>
        public AppCore Application { get { return _Application; } set { _Application.Set(value, this); } }
        private NotificationStore<AppCore> _Application;


    }
}