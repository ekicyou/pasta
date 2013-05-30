using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;

namespace Pasta.API
{
    /// <summary>
    /// ログ収集モジュール。
    /// </summary>
    public interface IPastaGleaner : IPastaModule, IPastaSource
    {
        /// <summary>設定情報</summary>
        string Setting { get; }
    }
}
