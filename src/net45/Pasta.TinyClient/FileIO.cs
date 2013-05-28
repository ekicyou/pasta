using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Composition;
using System.Threading.Tasks;
using Pasta.API;

namespace Pasta.TinyClient
{
    [Export(typeof(IFileIO)), Shared]
    public class FileIO : IFileIO
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        [ImportingConstructor]
        public FileIO()
        {
            logger.Trace("Load");
        }

        public Stream OpenRead(string path)
        {
            var st = new FileStream(path,
                    FileMode.Open,
                    FileAccess.Read, 
                    FileShare.ReadWrite,
                    bufferSize: 4096, useAsync: true);
            return st;
        }

        public Stream OpenAppend(string path)
        {
            var st = new FileStream(path,
                    FileMode.Append,
                    FileAccess.Write,
                    FileShare.Read,
                    bufferSize: 4096, useAsync: true);
            return st;
        }

        public Stream OpenReadWrite(string path)
        {
            var st = new FileStream(path,
                    FileMode.OpenOrCreate,
                    FileAccess.ReadWrite,
                    FileShare.None,
                    bufferSize: 4096, useAsync: true);
            return st;
        }
    }
}