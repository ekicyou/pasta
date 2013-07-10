using System;
using System.IO;

namespace CSUtil.IO
{
    /// <summary>
    /// 現在のディレクトリを保管し、カレントディレクトリを変更します。
    /// Dispose時に元に戻します。
    /// </summary>
    public sealed class Pushd : IDisposable
    {
        /// <summary>
        /// 元のディレクトリ。
        /// </summary>
        public readonly string OrgPath = null;

        /// <summary>
        /// デフォルトコンストラクタ。
        /// </summary>
        public Pushd()
        {
            OrgPath = Directory.GetCurrentDirectory();
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="newPath"></param>
        public Pushd(string newPath)
            : this()
        {
            Directory.SetCurrentDirectory(newPath);
        }

        /// <summary>
        /// 開放処理。
        /// </summary>
        public void Dispose()
        {
            Directory.SetCurrentDirectory(OrgPath);
        }

        /// <summary>
        /// ディレクトリを移動し、関数を実行します。
        /// 関数実行後、ディレクトリを元に戻します。
        /// </summary>
        /// <param name="newPath"></param>
        /// <param name="func"></param>
        public static void Execute(string newPath, System.Threading.ThreadStart func)
        {
            using (Pushd p = new Pushd(newPath)) {
                func();
            }
        }
    }
}