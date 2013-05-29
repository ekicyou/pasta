using Pasta.Model;
using System.Threading.Tasks.Dataflow;


namespace Pasta.API
{
    /// <summary>
    /// データ保存インターフェース
    /// ログを保存し、必要に応じて検索します。
    /// </summary>
    public interface IPastaStore : IPastaModule, IPastaTarget
    {
    }
}