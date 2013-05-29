using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// パスタログの受信ターゲット。
    /// </summary>
    public interface IPastaTarget
    {
        /// <summary>パスタログの受信ターゲット</summary>
        ITargetBlock<PastaLog> Target { get; }
    }
}