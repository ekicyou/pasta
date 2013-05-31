using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Threading.Tasks;
using System.Composition;
using Pasta.API;
using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;
using ProtoBuf;

namespace Pasta.Gleaners.Net
{
    /// <summary>
    /// メール監視の設定情報。
    /// </summary>
    [ProtoContract]
    public class MailGleanerSetting
    {
        /// <summary>監視間隔</summary>
        [ProtoMember(1)]
        public TimeSpan DieTime { get; set; }


        internal static MailGleanerSetting CreateDefault()
        {
            return new MailGleanerSetting
            {

            };
        }
    }

    /// <summary>
    /// メール監視。
    /// </summary>
    public class MailGleaner : BasePastaGleaner<MailGleanerSetting>
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();
        private const string Name = "メール";



        /// <summary>
        /// メールリスナのファクトリ。
        /// </summary>
        [Export(typeof(IPastaGleanerFactory))]
        public class Factory : BasePastaGleanerFactory<MailGleaner, MailGleanerSetting>
        {
            /// <summary>リスナ名</summary>
            public override string GleanerName { get { return Name; } }


            public override dynamic CreateDefaultSetting()
            {
                return MailGleanerSetting.CreateDefault();
            }
        }

        #region プロパティ


        #endregion
        #region 初期化：開放関係

        protected override void Init2(CancellationToken token, ITargetBlock<PastaLog> target)
        {
            throw new NotImplementedException();
        }


        #endregion
        #region 開始・停止処理

        protected override void Start()
        {
            throw new NotImplementedException();
        }

        protected override void Stop()
        {
            throw new NotImplementedException();
        }


        #endregion



    }

}