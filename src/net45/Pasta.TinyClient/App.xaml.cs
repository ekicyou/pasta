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

        public static AppCore Core { get; private set; }


        private void Application_Startup(object sender, StartupEventArgs e)
        {
            Core = new AppCore();
            this.Exit += (s, args) =>
            {
                logger.Trace("Exit Start");
                Core.Dispose();
                Core = null;
                logger.Trace("Exit End");
            };
        }
    }
}
