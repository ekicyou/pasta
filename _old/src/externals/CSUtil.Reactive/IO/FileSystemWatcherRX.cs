using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text;

namespace System.IO
{
    /// <summary>
    /// ファイルシステムウォッチャーのイベント監視。
    /// </summary>
    public static class FileSystemWatcherRX
    {
        public static IObservable<EventPattern<FileSystemEventArgs>> RxChanged(this FileSystemWatcher src)
        {
            return Observable.FromEventPattern<FileSystemEventArgs>(src, "Changed");
        }
        public static IObservable<EventPattern<FileSystemEventArgs>> RxCreated(this FileSystemWatcher src)
        {
            return Observable.FromEventPattern<FileSystemEventArgs>(src, "Created");
        }
        public static IObservable<EventPattern<FileSystemEventArgs>> RxDeleted(this FileSystemWatcher src)
        {
            return Observable.FromEventPattern<FileSystemEventArgs>(src, "Deleted");
        }
        public static IObservable<EventPattern<RenamedEventArgs>> RxRenamed(this FileSystemWatcher src)
        {
            return Observable.FromEventPattern<RenamedEventArgs>(src, "Renamed");
        }
        public static IObservable<EventPattern<FileSystemEventArgs>> RxAllWatch(this FileSystemWatcher src)
        {
            var r1 = src.RxChanged();
            var r2 = src.RxCreated();
            var r3 = src.RxDeleted();
            var r4 = src
                .RxRenamed()
                .Select(a =>
                {
                    return new EventPattern<FileSystemEventArgs>(a.Sender, a.EventArgs);
                });
            return Observable.Merge(r1, r2, r3, r4);
        }

    }
}