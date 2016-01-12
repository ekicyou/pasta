using System;
using System.Collections.Generic;
using System.IO;

namespace CSUtil.IO
{
    /// <summary>
    /// アンマネージリソースに渡される可能性がある一時ファイルの
    /// 生成、削除に関するユーティリティ関数郡です。
    /// 任意のタイミングで一時ファイルをまとめて削除できます。
    /// マネージのみで利用される一時ファイルの作成に、
    /// この関数郡を利用する必要はありません。
    /// </summary>
    public class UnmanagedTempFile : DisposableObject
    {
        #region スタティック関数
        /// <summary>
        /// 一意な名前を持つ 0 バイトの一時ファイルをディスク上に作成し、
        /// そのファイルの完全パスを返します。
        /// 作成したファイル名は記憶され、RemoveTempFile関数の実行時に削除されます。
        /// </summary>
        /// <returns></returns>
        public static string GetTempFileName()
        {
            return util.Create();
        }

        /// <summary>
        /// これまでに作成した一時ファイルについて、
        /// 削除可能なものをすべて削除します。削除できなかったものは次回の
        /// 削除タイミングに持ち越されます。
        /// 削除可能性を高めるため、実行に先立ちガベージコレクトを実行します。
        /// </summary>
        public static void ClearTempFile()
        {
            GC.Collect();
            System.Threading.Thread.CurrentThread.Join(0);
            GC.WaitForPendingFinalizers();
            GC.Collect();
            util.Clear();
        }

        private static readonly UnmanagedTempFile util = new UnmanagedTempFile();

        #endregion
        #region メンバ・メソッド

        private List<string> fileList = new List<string>();

        private UnmanagedTempFile() { }

        /// <summary>
        /// アンマネージリソースを開放します。
        /// </summary>
        protected override void DisposeUnManage()
        {
            Clear();
            base.DisposeUnManage();
        }

        /// <summary>
        /// 一意な名前を持つ 0 バイトの一時ファイルをディスク上に作成し、
        /// そのファイルの完全パスを返します。 
        /// </summary>
        /// <returns></returns>
        public string Create()
        {
            lock (this) {
                string path = Path.GetTempFileName();
                fileList.Add(path);
                return path;
            }
        }

        /// <summary>
        /// 削除可能な一時ファイルについて全て削除します。
        /// </summary>
        public void Clear()
        {
            lock (this) {
                List<string> remain = new List<string>();
                foreach (string path in fileList) {
                    try {
                        File.Delete(path);
                    }
                    catch {
                        remain.Add(path);
                    }
                }
                fileList = remain;
            }
        }

        #endregion
    }
}