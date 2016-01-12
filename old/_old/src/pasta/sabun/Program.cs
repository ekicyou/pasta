using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;

namespace sabun
{
    class Program
    {
        static void Main(string[] args)
        {
            var srcDir = Directory.GetCurrentDirectory();
            var dstDir = Path.Combine(srcDir, "out");
            Directory.CreateDirectory(dstDir);
            MakeDiffImage.DiffImage(srcDir, dstDir);
        }
    }
}
