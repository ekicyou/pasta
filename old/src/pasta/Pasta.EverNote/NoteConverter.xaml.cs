using System;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Threading;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Threading;
using System.Xml.Linq;
using CSUtil.Disposables;

namespace Pasta.EverNote
{
    /// <summary>
    /// NoteConverter.xaml の相互作用ロジック
    /// </summary>
    public partial class NoteConverter : UserControl
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        private readonly StackDisposable Tasks = new StackDisposable();
        private void UserControl_Unloaded(object sender, RoutedEventArgs e)
        {
            Tasks.DisposeNotNull();
        }

        public NoteConverter()
        {
            InitializeComponent();
        }

        private void UserControl_Loaded(object sender, RoutedEventArgs e)
        {

        }

        public class NoteItem
        {
            public XElement Note { get; internal set; }
            public string Title { get; internal set; }
            public string[] Tags { get; internal set; }
            public string Body { get; internal set; }
        }

        public IObservable<NoteItem> Convert(IObservable<XElement> rxNote)
        {
            var q1 = rxNote
                .ObserveOn(NewThreadScheduler.Default)
                .Select(a =>
                  {
                      var content = a.Descendants("content").First().Value;
                      content = content.Replace("&nbsp;", " ");
                      var xml = XDocument.Parse(content);
                      var body = xml.Root.Nodes()
                          .Select(b => b.ToString())
                          .ToArray();
                      var title = a.Descendants("title").First().Value;
                      var tags = a.Descendants("tag")
                          .Select(b => b.Value.Trim())
                          .Where(b => !string.IsNullOrWhiteSpace(b))
                          .Distinct()
                          .ToArray();
                      return new
                      {
                          Xml = a,
                          Title = title,
                          Tags = tags,
                          Content = content,
                          ContentXML = xml,
                          ContentBody = string.Join("", body),

                      };
                  });


            var syncObj = new object();
            webView.LoadCompleted += (s2, args) =>
            {
                lock(syncObj) Monitor.Pulse(syncObj);
            };
            var q2 = q1.Select(item =>
            {
                lock(syncObj) {
                    Dispatcher.BeginInvoke(new Action(() =>
                    {
                        var format = @"<!DOCTYPE html>
<html>
<head>
    <meta charset='utf-8'>
    <title></title>
</head>
<body><div id='contents'>{0}</div></body>
</html>";
                        var html = string.Format(format, item.ContentBody);
                        webView.NavigateToString(html);
                    }));
                    Monitor.Wait(syncObj);
                    string rc = "";

                    Dispatcher.BeginInvoke(new Action(() =>
                    {
                        lock(syncObj) {
                            dynamic doc = webView.Document;
                            dynamic el = doc.GetElementById("contents");
                            rc = el.InnerText;

                            Monitor.Pulse(syncObj);
                        }
                    }));
                    Monitor.Wait(syncObj);
                    return new
                    {
                        Xml = item.Xml,
                        Title = item.Title,
                        Tags = item.Tags,
                        Content = item.Content,
                        ContentText = rc,
                    };
                }
            });

            return q2
                .ObserveOn(TaskPoolScheduler.Default)
                .Select(a =>
                {
                    var text = a.ContentText.TrimEnd();
                    var lines = text
                        .Split('\n')
                        .Select(b => b.TrimEnd())
                        .SkipWhile(b => string.IsNullOrWhiteSpace(b));
                    var body = string.Join("\n", lines);

                    return new NoteItem
                    {
                        Note = a.Xml,
                        Title = a.Title,
                        Tags = a.Tags,
                        Body = body,
                    };
                });
        }


    }
}