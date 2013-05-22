using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Threading.Tasks;

namespace Pasta.API
{
    /// <summary>
    /// OSに依存しないFileIOのAPI。すべてasync。
    /// </summary>
    public interface IFileIO
    {
        /// <summary>
        /// 読み込み用ファイルをオープンします。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Task<Stream> OpenRead(string path);

        /// <summary>
        /// 書き込み用ファイルをオープンします。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Task<Stream> OpenWrite(string path);

        /// <summary>
        /// 読み書き用ファイルをオープンします。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        Task<Stream> OpenReadWrite(string path);

    }
}
