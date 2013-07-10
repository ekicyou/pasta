using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Text.RegularExpressions;
using Pasta.EverNote.Parsers;
using Newtonsoft.Json;
using Sprache;

namespace Pasta.EverNote.Test
{
    using NUnit.Framework;
    [TestFixture]
    public class PastaDicConvertTest
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        [Test]
        public void Convert()
        {
            using(var converter = NoteConverterWindow.GetConverter()) {
                var q1 = new[] { ResourceReader.GetPastaDocument() }
                      .SelectMany(a =>
                      {
                          var root = a.Root;
                          return root.Elements("note");
                      });
                var q2 = converter.Convert(q1.ToObservable())
                    .ToEnumerable();
                foreach(var item in q2) {
                    logger.Trace("######## {0}({1})", item.Title, string.Join(",", item.Tags));
                    var script =
                        item.Tags.Contains("会話") ? TALK(item) :
                        item.Tags.Contains("単語") ? WORD(item) : "";
                    logger.Trace(script);
                }
            }
        }

        private static string TALK(NoteConverter.NoteItem item)
        {
            var buf = new StringBuilder();
            
            // タイトル
            buf.AppendLine("  var title = " + JsonConvert.ToString(item.Title.Trim()) + ";");

            //
            var body = item.Body.TrimEnd();
            var lines = body.Split('\n')
                .Select(a => a + "\n")
                .Select(a => LineParser.WORDS.Parse(a).ToArray());
            var scraps = EnScrap(lines);

            foreach(var scrap in scraps) {




            }
            return SCOPE(buf.ToString());
        }
        private static IEnumerable<IEnumerable<Word[]>> EnScrap(IEnumerable<Word[]> lines)
        {
            List<Word[]> scrap = null;
            foreach(var line in lines) {
                if(line.Length == 0) {
                    if(scrap != null) {
                        yield return scrap;
                        scrap = null;
                    }
                }
                else {
                    if(scrap == null) scrap = new List<Word[]>();
                    scrap.Add(line);
                }
            }
            if(scrap != null) yield return scrap;
        }



        private static string WORD(NoteConverter.NoteItem item)
        {
            var qTag = item.Tags
                .Where(a => a != "単語")
                .Select(a => JsonConvert.ToString(a));
            var qItem = item.Body.Split('\n')
                .Select(a => a.Trim())
                .Where(a => !string.IsNullOrWhiteSpace(a))
                .Select(a => JsonConvert.ToString(a.Trim()));

            var buf = new StringBuilder();
            buf.AppendLine("  var tags=[" + string.Join(",", qTag) + "];");
            buf.AppendLine("  var items=[" + string.Join(",", qItem) + "];");
            buf.AppendLine("  dic.word(tags,items);");
            return SCOPE(buf.ToString());
        }



        private static string SCOPE(string code)
        {
            return
                "(function(){\n" +
                code +
                "})();\n";
        }

    }
}