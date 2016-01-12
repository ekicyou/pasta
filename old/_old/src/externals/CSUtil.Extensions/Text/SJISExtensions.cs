using System;
using System.IO;
using System.Text;
using System.Linq;
using System.Collections.Generic;

namespace CSUtil.Text
{
    /// <summary>
    /// SJIS限定便利関数
    /// </summary>
    public static class SJISExtensions
    {
        /// <summary>
        /// SJISエンコーディングの実装。
        /// </summary>
        public static readonly Encoding CP932 = Encoding.GetEncoding(932);

        /// <summary>
        /// ShiftJISエンコーディング。
        /// </summary>
        public static Encoding ShiftJIS { get { return CP932; } }


        /// <summary>
        /// 入力文字列をSJIS文字列とみなし、指定半角文字列数となるように半角スペースを詰める。
        /// SJIS以外の文字列の場合は何も行わない。
        /// </summary>
        /// <param name="text"></param>
        /// <param name="length"></param>
        /// <returns></returns>
        public static string PadRightSJIS(this string text, int length)
        {
            try {
                var padding = GetPadCount(text, length);
                if (padding <= 0) return text;
                return text + new String(' ', padding);
            }
            catch (EncoderFallbackException) {
                return text;
            }
        }

        /// <summary>
        /// 入力文字列をSJIS文字列とみなし、指定半角文字列数となるように半角スペースを詰める。
        /// SJIS以外の文字列の場合は何も行わない。
        /// </summary>
        /// <param name="text"></param>
        /// <param name="length"></param>
        /// <returns></returns>
        public static string PadLeftSJIS(this string text, int length)
        {
            try {
                var padding = GetPadCount(text, length);
                if (padding <= 0) return text;
                return new String(' ', padding) + text;
            }
            catch (EncoderFallbackException) {
                return text;
            }
        }

        private static int GetPadCount(string text, int length)
        {
            var cnt = GetSJISLength(text);
            var padding = length - cnt;
            return padding;
        }

        private static int GetSJISLength(string text)
        {
            var cnt = CP932.GetByteCount(text);
            return cnt;
        }

        /// <summary>
        /// 改行を含んだテキストブロックを横に連結します。
        /// </summary>
        /// <param name="arg1">１つ目のテキストブロック</param>
        /// <param name="space">ブロックごとに空けるスペースの幅</param>
        /// <param name="args">２つ目以降のテキストブロック</param>
        /// <returns></returns>
        public static string JoinSJISBlock(this string arg1, int space, params string[] args)
        {
            var q1 = (new[] { arg1 }).Concat(args);
            var sp = 0;
            var q2 = q1.Select(a =>
            {
                var lines = a.ReadLine()
                    .Select(b => new
                    {
                        Line = b,
                        LineLength = GetSJISLength(b)
                    })
                    .ToArray();
                var maxLength = lines
                    .Select(b => b.LineLength)
                    .DefaultIfEmpty(0)
                    .Max();
                var block = new
                {
                    Lines = lines,
                    MaxLineLength = maxLength,
                    Space = sp,
                };
                sp = space;
                return block;
            })
            .ToArray();

            var blockBuf = new StringBuilder();
            for(var i = 0; ; i++) {
                var buf = new StringBuilder();
                if(!q2.Any(a => i < a.Lines.Length)) break;
                foreach(var block in q2) {
                    if(block.Space > 0) buf.Append(new String(' ', block.Space));
                    var line = i < block.Lines.Length ? block.Lines[i].Line : "";
                    var lineLength = i < block.Lines.Length ? block.Lines[i].LineLength : 0;
                    buf.Append(line);
                    var padding = block.MaxLineLength - lineLength;
                    if(padding > 0) buf.Append(new String(' ', padding));
                }
                blockBuf.AppendLine(buf.ToString().TrimEnd());
            }
            return blockBuf.ToString();
        }

        private static IEnumerable<string> ReadLine(this string lines)
        {
            using(var reader = new StringReader(lines)) {
                string line = null;
                while((line = reader.ReadLine()) != null) {
                    yield return line.TrimEnd();
                }
            }
        }

    }
}