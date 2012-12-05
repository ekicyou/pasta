using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Pasta.Image.Test
{
    using NUnit.Framework;
    [TestFixture]
    public class MakeDiffImage
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        public class Comp1 : IEqualityComparer<Tuple<string, string, string>>
        {
            public bool Equals(Tuple<string, string, string> x, Tuple<string, string, string> y)
            {
                return x.Item1.Equals(y.Item1);
            }

            public int GetHashCode(Tuple<string, string, string> obj)
            {
                return obj.Item1.GetHashCode();
            }
        }


        [Test]
        public void ImageLoadTest()
        {
            var baseDir = CSUtil.Reflection.AssemblyUtil.GetCallingAssemblyDirctory();
            var srcDir = baseDir
                .PathCombine("..", "..", "..", "pasta", "pasta", "images", "org", "20120802")
                .GetFullPath();
            var dstDir = srcDir
                .PathCombine("..", "20120802_out")
                .GetFullPath();
            Directory.CreateDirectory(dstDir);

            DiffImage(srcDir, dstDir);

        }

        private static void DiffImage(string srcDir, string dstDir)
        {
            var items = Directory.EnumerateFiles(srcDir, "*.png");
            var comb = from src in items
                       from dst in items
                       where src != dst
                       let srcF = src.GetFileNameWithoutExtension()
                       let dstF = src.GetFileNameWithoutExtension()
                       let key = srcF.CompareTo(dstF) > 0 ? srcF + dstF : dstF + srcF
                       select Tuple.Create(key, src, dst);
            var comb2 = comb.Distinct(new Comp1());




            //var baseImagePath = srcDir.PathCombine("..", "mask.png").GetFullPath();
            var baseImagePath = srcDir.PathCombine("0.png").GetFullPath();

            var b0 = LoadImage(baseImagePath);
            logger.Trace("size: (w,h)=({0},{1})", b0.PixelWidth, b0.PixelHeight);
            logger.Trace("format: {0}", b0.Format);

            foreach(var srcPath in Directory.EnumerateFiles(srcDir, "*.png")) {
                logger.Trace("src: {0}", srcPath);
                var dstPath = dstDir.PathCombine(srcPath.GetFileName());
                logger.Trace("dst: {0}", dstPath);

                var b1 = LoadImage(srcPath);

                var stride = b0.PixelWidth * 4;
                var l0 = new byte[b0.PixelHeight * stride];
                var l1 = new byte[b0.PixelHeight * stride];
                b0.CopyPixels(l0, stride, 0);
                b1.CopyPixels(l1, stride, 0);

                var i0 = ToUInt32(l0);
                var i1 = ToUInt32(l1);

                var maskColor = i0.First();
                //bool isFirst = true;

                var l2 = i0
                    .Zip(i1, (a, b) => Tuple.Create(a, b))
                    .Select(t =>
                    {
#if true
                        // 色の差を求める
                        var aa = BitConverter.GetBytes(t.Item1).Select(a => (int)a);
                        var bb = BitConverter.GetBytes(t.Item2).Select(a => (int)a);
                        var diff = aa
                            .Zip(bb, (a, b) => a - b)
                            .Sum();
                        if(Math.Abs(diff) < 1) return (uint)0x00FFFFFF;
                        else return ((uint)t.Item2 | (uint)0xFF000000);
#else               

                        if (isFirst)
                        {
                            isFirst = false;
                            return (uint)0x00FFFFFF;
                        }
                        //if (t.Item1 == t.Item2) return (uint)0x00FFFFFF;
                        if (t.Item1 != maskColor) return (uint)0x00FFFFFF;
                        else return ((uint)t.Item2 | (uint)0xFF000000);
#endif
                    })
                    .SelectMany(a =>
                    {
                        var x = BitConverter.GetBytes(a);
                        return new[] { x[0], x[1], x[2], x[3] };
                    })
                    .ToArray();
                var b2 = WriteableBitmap.Create(b0.PixelWidth, b0.PixelHeight, b0.DpiX, b0.DpiY, PixelFormats.Bgra32, null, l2, stride);
                var encoder = new PngBitmapEncoder();
                var frame = BitmapFrame.Create(b2);
                encoder.Frames.Add(frame);
                using(var st = File.OpenWrite(dstPath)) encoder.Save(st);
            }
        }


        private static IEnumerable<uint> ToUInt32(byte[] bytes)
        {
            for (int i = 0; i < bytes.Length; i += 4)
            {
                yield return BitConverter.ToUInt32(bytes, i);
            }
        }


        private static WriteableBitmap LoadImage(string path)
        {
            var b0 = new BitmapImage(new Uri(path, UriKind.Absolute));
            return new WriteableBitmap(b0);
        }


    }
}