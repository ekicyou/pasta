using System.Linq;
using Pasta.EverNote.Parsers;
using Sprache;

namespace Pasta.EverNote.Test
{
    using NUnit.Framework;
    [TestFixture]
    public class ParserTest
    {
        [Test]
        public void Test1()
        {
            var rc = LineParser.WORDS.Parse("＠テスト　　 てすとなんですよー。￥えっ？　ほんと。\n").ToArray();
            rc.Length.Is(4);
            KeyWord w = rc[0] as KeyWord;
            w.IsNot(null);
            w.Key.Is("テスト");
            TextWord t = rc[1] as TextWord;
            t.IsNot(null);
            t.Text.Is("てすとなんですよー。");
            KeyWord w2 = rc[2] as KeyWord;
            w2.IsNot(null);
            w2.Key.Is("えっ？");
            TextWord t2 = rc[3] as TextWord;
            t2.IsNot(null);
            t2.Text.Is("ほんと。");



        }
    }
}
