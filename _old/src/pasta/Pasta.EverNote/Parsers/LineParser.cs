using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using Sprache;

namespace Pasta.EverNote.Parsers
{
    /// <summary>
    /// 改行文字で終わる１行をパースします。
    /// 入力には必ず改行文字を含めてください。
    /// </summary>
    public static class LineParser
    {
        private static bool IsAT(char a)
        {
            switch (a)
            {
                case '@':
                case '＠':
                case '\\':
                case '￥':
                    return true;
            }
            return false;
        }

        private static bool IsSPACE(char a)
        {
            switch (a)
            {
                case ' ':
                case '　':
                case '\r':
                case '\n':
                    return true;
            }
            return false;
        }


        private static readonly Parser<char> AT = Parse.Char(IsAT, "＠");

        private static readonly Parser<char> SP = Parse.Char(IsSPACE, "SPACE");

        private static readonly Parser<char> NotAT = Parse.Char(a => !IsAT(a), "not @");

        private static readonly Parser<char> NotSP = Parse.Char(a => !IsSPACE(a), "not SPACE");

        private static readonly Parser<Word> KEYWORD = from at in AT
                                                      from key in NotSP.AtLeastOnce().Text()
                                                      from sp in SP.AtLeastOnce().Text()
                                                      select (Word)(new KeyWord { Key = key });

        private static readonly Parser<Word> TEXT = from text in NotAT.AtLeastOnce().Text()
                                                   select (Word)(new TextWord { Text = text });

        private static readonly Parser<Word> KEYorTEXT = KEYWORD.Or(TEXT);

        public static readonly Parser<IEnumerable<Word>> WORDS = KEYorTEXT.Many();



    }
}