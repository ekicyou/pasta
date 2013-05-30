using System;
using System.Collections.Generic;
using System.Configuration;
using System.Data;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using Pasta;

namespace Pasta.TinyClient
{
    /// <summary>
    /// App.xaml の相互作用ロジック
    /// </summary>
    public partial class App : Application
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        public static AppLoader Loader { get; private set; }


        private void Application_Startup(object sender, StartupEventArgs e)
        {
            logger.Trace("Application_Startup Start");
            Loader = new AppLoader();
            this.Exit += (s, args) =>
            {
                logger.Trace("Exit Start");
                Loader.Dispose();
                Loader = null;
                logger.Trace("Exit End");
            };
        }
    }
}
