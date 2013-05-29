using System.Threading;

namespace Pasta.API
{
    public interface IPastaModule
    {
        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="token"></param>
        /// <param name="io"></param>
        /// <returns></returns>
        void Init(CancellationToken token);

    }
}
