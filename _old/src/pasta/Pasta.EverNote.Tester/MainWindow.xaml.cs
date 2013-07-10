using System;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Threading;
using System.Windows;
using System.Xml.Linq;
using CSUtil.Disposables;


namespace Pasta.EverNote.Tester
{
    /// <summary>
    /// MainWindow.xaml の相互作用ロジック
    /// </summary>
    public partial class MainWindow : Window
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();
        private readonly StackDisposable Tasks = new StackDisposable();
        private void Window_Unloaded(object sender, RoutedEventArgs e)
        {
            Tasks.DisposeNotNull();
        }



        public MainWindow()
        {
            InitializeComponent();
        }

        private void Window_Loaded(object sender, RoutedEventArgs e)
        {
            var q1 = new[] { ResourceReader.GetPastaDocument() }
                  .SelectMany(a =>
                  {
                      var root = a.Root;
                      return root.Elements("note");
                  });
            var q2 = noteConverter.Convert(q1.ToObservable());

            q2.Subscribe(next =>
            {
                logger.Trace("### title = [{0}]({1})\n{2}", next.Title, string.Join(",", next.Tags), next.Body);
            }, () =>
            {
                Dispatcher.BeginInvoke(new Action(() =>
                {
                    Application.Current.Shutdown();
                }));
            });

        }

    }
}