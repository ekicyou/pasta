using System.Threading;

namespace Pasta.API
{
    public interface IPastaModule
    {
        /// <summary>
        /// 初期化処理。
        /// </summary>
        /// <param name="settingObject"></param>
        /// <param name="token"></param>
         void Init(dynamic settingObject, CancellationToken token);

    }
}
