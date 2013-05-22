using Pasta.API;
using Pasta.Util.Disposables;
using System;
using System.Collections.Generic;
using System.Composition;
using System.Linq;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;
using Pasta.Model;
using ProtoBuf;


namespace Pasta.Logging
{
    /// <summary>
    /// ロガー。
    /// 解析の呼び出し、及びファイルへの保管処理を行う。
    /// </summary>
    [Export("PastaStore", typeof(IPastaStore))]
    public sealed class PastaStore : IPastaStore, IDisposable
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        #region プロパティ

        /// <summary>受信ターゲットを接続します。</summary>
        public ISourceBlock<PastaLog> Input { get; private set; }

        /// <summary>FileIOオブジェクト。</summary>
        private IFileIO FileIO { get; set; }

        /// <summary>現在保存対象になっているログストリーム。</summary>
        private Stream SaveStream { get; set; }


        #endregion
        #region 初期化・開放

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="token"></param>
        public PastaStore(CancellationToken token, IFileIO io)
        {
            token.Register(Dispose);

            var opt = new DataflowBlockOptions
            {
                CancellationToken = token,
            };

            var buffer = new BufferBlock<PastaLog>(opt);
            var act = new ActionBlock<PastaLog>(Save, opt);
            buffer.LinkTo(act);
            Input = buffer;
        }


        #endregion
        #region 開放

        public void Dispose()
        {
            if (SaveStream != null)
            {
                SaveStream.Dispose();
                SaveStream = null;
            }
        }


        #endregion
        #region メソッド

        private async void Save(PastaLog log)
        {
        }



        #endregion


    }
}