using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Reflection;
using System.Composition;
using System.Threading.Tasks;
using Pasta.API;
using Pasta.Model;

namespace Pasta.TinyClient
{
    /// <summary>
    /// 非同期ファイルIOの実装。
    /// </summary>
    [Export(typeof(IFileIO)), Shared]
    public class FileIO : NotificationObject, IFileIO
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        /// <summary>アプリケーションデータの保存フォルダ</summary>
        public string AppDataFolder { get { return _AppDataFolder; } set { _AppDataFolder.Set(Path.GetFullPath(value), this); } }
        private NotificationStore<string> _AppDataFolder;


        [ImportingConstructor]
        public FileIO()
        {
            logger.Trace("Load");
            AppDataFolder = Properties.Settings.Default.AppDataFolder;
            CheckAppDataFolder();
        }

        /// <summary>
        /// アプリケーションデータフォルダが存在しない場合、規定のフォルダを設定します。
        /// </summary>
        public void CheckAppDataFolder()
        {
            try
            {
                if (!string.IsNullOrWhiteSpace(AppDataFolder))
                {
                    if (Directory.Exists(AppDataFolder)) return;
                }
                var baseFolder = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
                var assm = Assembly.GetExecutingAssembly();
                var company = Attribute.GetCustomAttribute(assm, typeof(AssemblyCompanyAttribute)) as AssemblyCompanyAttribute;
                var title = Attribute.GetCustomAttribute(assm, typeof(AssemblyTitleAttribute)) as AssemblyTitleAttribute;
                var folder = Path.Combine(baseFolder, company.Company, title.Title);
                AppDataFolder = folder;
                SaveSetting();
            }
            catch (Exception ex)
            {
                logger.Error(ex);
            }
        }

        /// <summary>
        /// 設定情報を保存します。
        /// </summary>
        public void SaveSetting()
        {
            if (!Directory.Exists(AppDataFolder)) Directory.CreateDirectory(AppDataFolder);

            Properties.Settings.Default.AppDataFolder = AppDataFolder;
            Properties.Settings.Default.Save();
        }


#pragma warning disable 1998
        public async Task<Stream> OpenReadAsync(string path)
        {
            var st = new FileStream(path,
                    FileMode.Open,
                    FileAccess.Read, 
                    FileShare.ReadWrite,
                    bufferSize: 4096, useAsync: true);
            return st;
        }

        public async Task<Stream> OpenAppendAsync(string path)
        {
            var st = new FileStream(path,
                    FileMode.Append,
                    FileAccess.Write,
                    FileShare.Read,
                    bufferSize: 4096, useAsync: true);
            return st;
        }

        public async Task<Stream> OpenReadWriteAsync(string path)
        {
            var st = new FileStream(path,
                    FileMode.OpenOrCreate,
                    FileAccess.ReadWrite,
                    FileShare.None,
                    bufferSize: 4096, useAsync: true);
            return st;
        }
#pragma warning restore 1998
    }
}