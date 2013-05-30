using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Composition;
using Pasta.API;
using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;

namespace Pasta.Gleaners.Net
{
    /// <summary>
    /// メール監視。
    /// </summary>
    public class MailGleaner : IPastaGleaner
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();
        private const string Name = "メール";

        /// <summary>
        /// メールリスナのファクトリ。
        /// </summary>
        [Export(typeof(IPastaGleanerFactory))]
        public class Factory : IPastaGleanerFactory
        {
            /// <summary>リスナ名</summary>
            public string GleanerName { get { return Name; } }

            /// <summary>
            /// コンストラクタ。
            /// </summary>
            /// <param name="setting"></param>
            /// <returns></returns>
            public IPastaGleaner CreateGleaner(string setting) { return new MailGleaner(setting); }
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public MailGleaner(string setting)
        {
            Setting = setting;
        }


        public void Init(CancellationToken token)
        {
            logger.Trace("Init Start");
            throw new NotImplementedException();
        }

        public ISourceBlock<PastaLog> Source
        {
            get { throw new NotImplementedException(); }
        }

        /// <summary>
        /// 設定情報。
        /// </summary>
        public string Setting { get; private set; }
    }

}