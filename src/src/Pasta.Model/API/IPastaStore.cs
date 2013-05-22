using Pasta.Model;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// データ保存インターフェース
    /// ログを保存し、必要に応じて検索します。
    /// </summary>
    public interface IPastaStore
    {
        /// <summary>受信ターゲットを接続します。</summary>
        ISourceBlock<PastaLog> Input { get; }
    }
}