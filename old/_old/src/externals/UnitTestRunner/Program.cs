using System;
using System.Collections.Generic;
using System.Text;
using System.IO;
using System.Diagnostics;
using NUnit.ConsoleRunner;

namespace NUnit.ConsoleRunner
{
    internal class Program
    {
        [STAThread]
        static int Main(string[] args)
        {
            return Run(args);
        }

        internal static int Run(string[] args)
        {
            List<string> p = new List<string>();
            // [*.Test.dll]な名前のファイルをすべて追加
            foreach (string path in Directory.GetFiles(Directory.GetCurrentDirectory(), "*.test.dll"))
            {
                p.Add(path);
            }
            p.Add("/nologo");
            p.Add("/nodots");
            bool failStop = false;
            foreach (string arg in args)
            {
                if (arg == "/failstop")
                {
                    failStop = true;
                    continue;
                }
                p.Add(arg);
            }

            //ConsoleUi test = new ConsoleUi();
            //int rc = test.Execute(new ConsoleOptions(p.ToArray()));
            int rc = Runner.Main(p.ToArray());
            if (rc != 0)
            {
                if (failStop)
                {
                    Console.WriteLine("ユニットテストに失敗しました。何かKeyを押してください。");
                    Console.ReadKey();
                }
            }
            return rc;
        }
    }
}
