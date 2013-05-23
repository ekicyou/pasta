using Pasta.Model;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// ロガーインターフェース
    /// ログを受信し、書き込み先などに分配します。
    /// </summary>
    public interface IPastaLogger
    {
        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="token"></param>
        void Init(CancellationToken token);

        /// <summary>受信ターゲットを接続します。</summary>
        ISourceBlock<PastaLog> Input { get; }

        /// <summary>送信ソースを接続します。</summary>
        ITargetBlock<PastaLog> Output { get; }

    }
}