using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Composition;
using System.Composition.Hosting;
using Pasta.Model;
using Pasta.API;

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

            var configuration = new ContainerConfiguration()
                .WithAssembly(this.GetType().Assembly);


            var catalog = new AggregateCatalog();
            catalog.Catalogs.Add(new AssemblyCatalog(typeof(Program).Assembly));

            //Create the CompositionContainer with the parts in the catalog
            _container = new CompositionContainer(catalog);

            //Fill the imports of this object
            try
            {
                this._container.ComposeParts(this);
            }
            catch (CompositionException compositionException)
            {
                Console.WriteLine(compositionException.ToString());
            }


            logger.Trace("Init End");
        }


        [Import("PastaLogger")]
        public IPastaLogger PastaLogger { get; set; }


    }
}