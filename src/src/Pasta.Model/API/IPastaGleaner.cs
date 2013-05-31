using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;

namespace Pasta.API
{
    /// <summary>
    /// ログ収集モジュール。
    /// ログ監視モジュールは単独で終了することがあるので、
    /// IDisposableを実装する。
    /// </summary>
    public interface IPastaGleaner : IPastaModule, IPastaSource, IDisposable, INotifyPropertyChanged
    {
        /// <summary>設定情報</summary>
        dynamic SettingObject { get; }

        /// <summary>収集処理の動作状態を取得・設定します。</summary>
        bool Enabled { get; set; }

    }
}
