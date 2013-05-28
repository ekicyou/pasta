using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Threading.Tasks;

namespace Pasta.API
{
    /// <summary>
    /// OSに依存しないFileIOのAPI。すべてasyncモードで行う。
    /// </summary>
    public interface IFileIO
    {
        /// <summary>
        /// 読み込み用ファイルをオープンします。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Stream OpenRead(string path);

        /// <summary>
        /// 追記書き込み用ファイルをオープンします。
        /// 存在しない場合はファイルを作成します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Stream OpenAppend(string path);

        /// <summary>
        /// 読み書き用ファイルをオープンします。
        /// 存在しない場合はファイルを作成します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Stream OpenReadWrite(string path);

    }
}
