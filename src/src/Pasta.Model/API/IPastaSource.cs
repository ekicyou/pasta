using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// パスタログの発信ソース。
    /// </summary>
    public interface IPastaSource
    {
        /// <summary>パスタログの発信ソース。</summary>
        ISourceBlock<PastaLog> Source { get; }
    }
}