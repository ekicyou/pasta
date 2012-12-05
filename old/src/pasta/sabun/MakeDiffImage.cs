using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;


namespace sabun
{
    public static class MakeDiffImage
    {
        public static void DiffImage(string srcDir, string dstDir,int threshold = 0)
        {
            var items = Directory.EnumerateFiles(srcDir, "*.png");
            var baseImagePath = items.FirstOrDefault();
            if(baseImagePath == null) return;

            var b0 = LoadImage(baseImagePath);

            foreach(var srcPath in items) {
                var dstPath = dstDir.PathCombine(srcPath.GetFileName());
                var b1 = LoadImage(srcPath);

                // 画像サイズが異なる場合はキャンセル
                if(b0.PixelWidth != b1.PixelWidth) continue;
                if(b0.PixelHeight != b1.PixelHeight) continue;

                // 領域確保
                var stride = b0.PixelWidth * 4;
                var l0 = new byte[b0.PixelHeight * stride];
                var l1 = new byte[b0.PixelHeight * stride];
                b0.CopyPixels(l0, stride, 0);
                b1.CopyPixels(l1, stride, 0);

                // 差分画像作成
                var i0 = ToUInt32(l0);
                var i1 = ToUInt32(l1);

                var l2 = i0
                    .Zip(i1, (a, b) => Tuple.Create(a, b))
                    .Select(t =>
                    {
                        // 色の差を求める
                        var aa = BitConverter.GetBytes(t.Item1).Select(a => (int)a);
                        var bb = BitConverter.GetBytes(t.Item2).Select(a => (int)a);
                        var diff = aa
                            .Zip(bb, (a, b) => Math.Abs(a - b))
                            .Sum();
                        if(diff < (threshold + 1)) return (uint)0x00FFFFFF;
                        else return ((uint)t.Item2 | (uint)0xFF000000);
                    })
                    .SelectMany(a =>
                    {
                        var x = BitConverter.GetBytes(a);
                        return new[] { x[0], x[1], x[2], x[3] };
                    })
                    .ToArray();

                // 出力
                var b2 = WriteableBitmap.Create(b0.PixelWidth, b0.PixelHeight, b0.DpiX, b0.DpiY, PixelFormats.Bgra32, null, l2, stride);
                var encoder = new PngBitmapEncoder();
                var frame = BitmapFrame.Create(b2);
                encoder.Frames.Add(frame);
                using(var st = File.OpenWrite(dstPath)) encoder.Save(st);
            }
        }

        private static IEnumerable<uint> ToUInt32(byte[] bytes)
        {
            for(int i = 0; i < bytes.Length; i += 4) {
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
