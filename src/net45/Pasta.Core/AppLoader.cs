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
        }


        /// <summary>
        /// コンストラクタ。
        /// モデルロードのための最低限の初期化を行い、遅延初期化処理の起動を予約する。
        /// </summary>
        public AppLoader()
        {
            Thread.CurrentThread.Name = "UI";
            logger.Trace("Load Start");
            CTS = new CancellationTokenSource();
            var uiSyncContext = TaskScheduler.FromCurrentSynchronizationContext();
            var fact = new TaskFactory(CTS.Token);
            var callerAssembly = Assembly.GetCallingAssembly();
            fact.StartNew(() => Init(CTS.Token, uiSyncContext, callerAssembly));
            logger.Trace("Load End");
        }

        /// <summary>
        /// 遅延初期化処理。
        /// </summary>
        /// <param name="token"></param>
        private void Init(CancellationToken token, TaskScheduler uiSyncContext, Assembly callerAssembly)
        {
            logger.Trace("Init Start");
            try
            {
                // MEFの構成
                var qAssm = new[] { 
                    this.GetType().Assembly,
                    Assembly.GetEntryAssembly(),
                    callerAssembly, 
                    typeof(Pasta.Logging.PastaLogger).Assembly,
                    typeof(Pasta.Gleaners.Net.MailGleaner).Assembly,
                }
                    .Where(a => a != null)
                    .Distinct();
                var configuration = new ContainerConfiguration()
                    .WithAssemblies(qAssm)
                    ;
                var host = configuration.CreateContainer();
                token.Register(() => host.Dispose());

                // appの取得と構成
                var app = host.GetExport<AppCore>();
                app.Init(token,uiSyncContext);
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