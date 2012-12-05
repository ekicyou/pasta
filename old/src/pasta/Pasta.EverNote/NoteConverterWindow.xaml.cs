using System;
using System.Threading;
using System.Windows;
using System.Windows.Threading;
using System.Xml.Linq;
using CSUtil.Disposables;


namespace Pasta.EverNote
{
    /// <summary>
    /// MainWindow.xaml の相互作用ロジック
    /// </summary>
    public partial class NoteConverterWindow : Window
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        public class ConverterInterface : IDisposable
        {
            public void Dispose()
            {
                lock (this)
                {
                    Dispatcher.BeginInvokeShutdown(DispatcherPriority.Normal);
                    Monitor.Wait(this);
                }
            }
            public Thread Thread { get; set; }
            public Dispatcher Dispatcher { get; set; }
            public NoteConverterWindow Win { get; set; }
            public IObservable<NoteConverter.NoteItem> Convert(IObservable<XElement> rxNote)
            {
                return Win.Convert(rxNote);
            }
        }
        public static ConverterInterface GetConverter()
        {
            var rc = new ConverterInterface();
            var thread = new Thread(() =>
            {
                try
                {
                    rc.Dispatcher = Dispatcher.CurrentDispatcher;
                    lock (rc) Monitor.Pulse(rc);
                    Dispatcher.Run();
                    lock (rc) Monitor.Pulse(rc);
                }
                catch (Exception ex)
                {
                    logger.Warn(ex);
                }
            });
            thread.SetApartmentState(ApartmentState.STA);
            thread.IsBackground = true;
            rc.Thread = thread;
            lock (rc)
            {
                thread.Start();
                Monitor.Wait(rc);
            }
            rc.Dispatcher.Invoke(new Action(() => {
                rc.Win = new NoteConverterWindow();
                rc.Win.Show();
                rc.Win.Hide();
            }));
            return rc;
        }


        private readonly StackDisposable Tasks = new StackDisposable();
        private void Window_Unloaded(object sender, RoutedEventArgs e)
        {
            Tasks.DisposeNotNull();
        }

        public NoteConverterWindow()
        {
            InitializeComponent();
        }

        private void Window_Loaded(object sender, RoutedEventArgs e)
        {

        }

        public IObservable<NoteConverter.NoteItem> Convert(IObservable<XElement> rxNote)
        {
            return noteConverter.Convert(rxNote);
        }


    }
}