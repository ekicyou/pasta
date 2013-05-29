using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// ロガーインターフェース
    /// ログを受信し、書き込み先などに分配します。
    /// </summary>
    public interface IPastaLogger : IPastaModule, IPastaSource, IPastaTarget
    {
    }
}