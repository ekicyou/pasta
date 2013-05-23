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

        #endregion
        #region 初期化・開放

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public PastaStore()
        {
        }

        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="token"></param>
        /// <param name="io"></param>
        /// <returns></returns>
        public void Init(CancellationToken token, IFileIO io)
        {
            token.Register(Dispose);
            FileIO = io;

            var opt = new DataflowBlockOptions
            {
                CancellationToken = token,
            };
            var buffer = new BufferBlock<PastaLog>(opt);

            var actOpt = new ExecutionDataflowBlockOptions()
            {
                CancellationToken = token,
                SingleProducerConstrained = true,
            };
            var act = new ActionBlock<PastaLog>(async (a) => await Save(a), actOpt);
            buffer.LinkTo(act);
            Input = buffer;
        }



        #endregion
        #region 開放

        public void Dispose()
        {
            CloseSaveStream();
        }


        #endregion
        #region メソッド：保存関係

        private async Task Save(PastaLog item)
        {
            var st = await GetSaveStream(item.UTC);
            Serializer.SerializeWithLengthPrefix<PastaLog>(st, item, PrefixStyle.Base128);
        }


        /// <summary>現在保存対象になっているログストリーム。</summary>
        private Stream SaveStream { get; set; }

        /// <summary>現在保存対象になっているログストリームの保存日付。</summary>
        private DateTime SaveDay { get; set; }


        private async Task<Stream> GetSaveStream(DateTime time)
        {
            // 日付不一致ならクローズ
            var day = time.Date;
            if (SaveDay != day)
            {
                CloseSaveStream();
                SaveDay = day;
            }

            // クローズされているなら開く
            if (SaveStream == null)
            {
                var path = GetSavePath();
                SaveStream = await FileIO.OpenWrite(path);
            }

            return SaveStream;
        }

        private string GetSavePath()
        {
            var dir = "";
            var t1 = SaveDay.ToString("yyyy-MM");
            var t2 = SaveDay.ToString("-dd");
            var path = Path.Combine(dir, t1, t1 + t2 + ".log");
            return path;
        }


        private void CloseSaveStream()
        {
            if (SaveStream == null) return;
            SaveStream.Close();
            SaveStream = null;
        }







        #endregion


    }
}