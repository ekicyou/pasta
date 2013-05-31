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

namespace Pasta.Gleaners
{
    public abstract class BasePastaGleaner<TSetting> : NotificationObject, IPastaGleaner
        where TSetting : class
    {
        /// <summary>設定情報。</summary>
        protected TSetting Setting { get; private set; }
        public dynamic SettingObject { get { return Setting; } }

        /// <summary>キャンセルトークンソース</summary>
        public CancellationTokenSource CTS { get; private set; }

        /// <summary>データソース</summary>
        public ISourceBlock<PastaLog> Source { get; private set; }

        /// <summary>
        /// 開放処理。
        /// </summary>
        public void Dispose()
        {
            if (CTS == null) return;
            CTS.Cancel();
            CTS.Dispose();
            CTS = null;
        }

        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="settingObject"></param>
        /// <param name="token"></param>
        public void Init(dynamic settingObject, CancellationToken token)
        {
            // 設定情報
            Setting = settingObject as TSetting;

            // 独自CTSの作成。
            CTS = new CancellationTokenSource();
            token.Register(() => Dispose());

            // ログの送受信バッファ
            var buf = new BufferBlock<PastaLog>(new DataflowBlockOptions
            {
                CancellationToken = CTS.Token,
            });

            // 残りの初期化
            Init2(CTS.Token, buf);
            Source = buf;
        }

        /// <summary>
        /// 残りの初期化。
        /// </summary>
        /// <param name="token"></param>
        /// <param name="target"></param>
        protected abstract void Init2(CancellationToken token,ITargetBlock<PastaLog> target);


        #region 収集処理の開始・停止制御

        /// <summary>収集処理の動作状態を取得・設定します。</summary>
        public bool Enabled
        {
            get { return _Enabled; }
            set
            {
                if (_Enabled.Set(value, this))
                {
                    if (Enabled) Start();
                    else Stop();
                }
            }
        }
        private NotificationStore<bool> _Enabled;

        /// <summary>
        /// 収集処理を開始します。
        /// </summary>
        protected abstract void Start();

        /// <summary>
        /// 収集処理を終了します。
        /// </summary>
        protected abstract void Stop();

        #endregion



    }
}
